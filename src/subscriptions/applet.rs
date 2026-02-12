use std::{
    collections::HashMap,
    os::fd::{BorrowedFd, IntoRawFd, RawFd},
};
use tokio::{net::UnixStream, sync::mpsc::Sender};
use tracing::{error, info};
use zbus::{
    Connection, Guid, connection::Builder, interface, object_server::SignalEmitter,
    zvariant::OwnedFd,
};

use super::notifications::Input;

use anyhow::{Result, bail};
use cosmic_ext_notifications_util::DAEMON_NOTIFICATIONS_FD;
use std::os::unix::io::FromRawFd;

use serde_json;

pub async fn setup_panel_conn(tx: Sender<Input>) -> Result<Connection> {
    let socket = setup_panel_socket()?;
    let guid = Guid::generate();
    let conn = tokio::time::timeout(
        tokio::time::Duration::from_secs(1),
        Builder::socket(socket)
            .p2p()
            .server(guid)
            .unwrap()
            .serve_at(
                "/com/system76/NotificationsSocket",
                NotificationsSocket { tx },
            )?
            .build(),
    )
    .await??;

    Ok(conn)
}

/// Creates a non-blocking [`UnixStream`] for communicating with the panel.
///
/// # Safety
///
/// It is assumed that `DAEMON_NOTIFICATIONS_FD` was set to a valid raw file descriptor ID.
pub fn setup_panel_socket() -> Result<UnixStream> {
    let Ok(raw_fd_env_var) = std::env::var(DAEMON_NOTIFICATIONS_FD) else {
        bail!("DAEMON_NOTIFICATIONS_FD is not set.");
    };

    let Ok(raw_fd) = raw_fd_env_var.parse::<RawFd>() else {
        bail!("DAEMON_NOTIFICATIONS_FD is not a valid RawFd.");
    };

    let fd = unsafe { BorrowedFd::borrow_raw(raw_fd).try_clone_to_owned().unwrap() };
    info!("Connecting to daemon on fd {}", raw_fd);

    rustix::io::fcntl_setfd(
        &fd,
        rustix::io::fcntl_getfd(&fd)? | rustix::io::FdFlags::CLOEXEC,
    )?;

    let unix_stream = std::os::unix::net::UnixStream::from(fd);
    unix_stream.set_nonblocking(true)?;

    Ok(UnixStream::from_std(unix_stream)?)
}

pub struct NotificationsSocket {
    pub tx: Sender<Input>,
}

#[interface(name = "com.system76.NotificationsSocket")]
impl NotificationsSocket {
    #[zbus(out_args("fd"))]
    async fn get_fd(&self) -> zbus::fdo::Result<OwnedFd> {
        let (mine, theirs) = std::os::unix::net::UnixStream::pair()
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
        mine.set_nonblocking(true)
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
        theirs
            .set_nonblocking(true)
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
        let mine: UnixStream =
            UnixStream::from_std(mine).map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;

        let guid = Guid::generate();

        let tx_clone = self.tx.clone();
        tokio::spawn(async move {
            let conn = match Builder::socket(mine).p2p().server(guid).unwrap().serve_at(
                "/com/system76/NotificationsApplet",
                NotificationsApplet {
                    tx: tx_clone.clone(),
                },
            ) {
                Ok(conn) => conn,
                Err(err) => {
                    error!("Failed to create applet connection {}", err);
                    return;
                }
            };

            info!("Creating applet connection");
            let conn = match conn.build().await {
                Ok(conn) => conn,
                Err(err) => {
                    error!("Failed to create applet connection {}", err);
                    return;
                }
            };
            info!("Created applet connection");

            if let Err(err) = tx_clone.send(Input::AppletConn(conn)).await {
                error!("Failed to send applet connection {}", err);
                return;
            }
            info!("Sent applet connection");
        });

        let raw = theirs.into_raw_fd();
        info!("Sending fd to applet");

        Ok(unsafe { zbus::zvariant::OwnedFd::from(std::os::fd::OwnedFd::from_raw_fd(raw)) })
    }
}

pub struct NotificationsApplet {
    pub tx: Sender<Input>,
}

#[allow(clippy::too_many_arguments)]
#[interface(name = "com.system76.NotificationsApplet")]
impl NotificationsApplet {
    #[zbus(signal)]
    pub async fn notify(
        signal_ctxt: &SignalEmitter<'_>,
        app_name: &str,
        replaces_id: u32,
        app_icon: &str,
        summary: &str,
        body: &str,
        actions: Vec<&str>,
        hints: HashMap<&str, zbus::zvariant::Value<'_>>,
        expire_timeout: i32,
    ) -> zbus::Result<()>;

    pub async fn invoke_action(&self, id: u32, action: &str) -> zbus::fdo::Result<()> {
        tracing::trace!("Received action from applet {id} {action}");
        let res = self
            .tx
            .send(Input::AppletActivated {
                id,
                action: action.parse().unwrap(),
            })
            .await;
        if let Err(err) = res {
            tracing::error!("Failed to send action invoke message to channel. {id}");
            return Err(zbus::fdo::Error::Failed(err.to_string()));
        }
        Ok(())
    }

    pub async fn get_history(&self) -> zbus::fdo::Result<Vec<(u32, String, String, String, String, i64)>> {
        tracing::trace!("Received get_history request from applet");

        let (tx, rx) = tokio::sync::oneshot::channel();

        let res = self.tx.send(Input::GetHistory { tx }).await;
        if let Err(err) = res {
            tracing::error!("Failed to send get_history message to channel");
            return Err(zbus::fdo::Error::Failed(err.to_string()));
        }

        // Wait for response with timeout
        let notifications = match tokio::time::timeout(
            tokio::time::Duration::from_secs(2),
            rx
        ).await {
            Ok(Ok(notifs)) => notifs,
            Ok(Err(err)) => {
                tracing::error!("Failed to receive history: {}", err);
                return Err(zbus::fdo::Error::Failed("Channel closed".to_string()));
            }
            Err(_) => {
                tracing::error!("Timeout waiting for history");
                return Err(zbus::fdo::Error::Failed("Timeout".to_string()));
            }
        };

        // Convert notifications to a D-Bus compatible format
        // Return (id, app_name, summary, body, app_icon, timestamp_secs)
        let result: Vec<_> = notifications.into_iter().map(|n| {
            let timestamp = n.time
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;
            (n.id, n.app_name, n.summary, n.body, n.app_icon, timestamp)
        }).collect();

        Ok(result)
    }

    pub async fn get_history_full(&self) -> zbus::fdo::Result<Vec<String>> {
        tracing::trace!("Received get_history_full request from applet");

        let (tx, rx) = tokio::sync::oneshot::channel();

        let res = self.tx.send(Input::GetHistory { tx }).await;
        if let Err(err) = res {
            tracing::error!("Failed to send get_history_full message to channel");
            return Err(zbus::fdo::Error::Failed(err.to_string()));
        }

        // Wait for response with timeout
        let notifications = match tokio::time::timeout(
            tokio::time::Duration::from_secs(2),
            rx
        ).await {
            Ok(Ok(notifs)) => notifs,
            Ok(Err(err)) => {
                tracing::error!("Failed to receive full history: {}", err);
                return Err(zbus::fdo::Error::Failed("Channel closed".to_string()));
            }
            Err(_) => {
                tracing::error!("Timeout waiting for full history");
                return Err(zbus::fdo::Error::Failed("Timeout".to_string()));
            }
        };

        // Serialize each notification to JSON
        let result: Result<Vec<_>, _> = notifications
            .into_iter()
            .map(|n| {
                serde_json::to_string(&n)
                    .map_err(|e| {
                        tracing::error!("Failed to serialize notification {}: {}", n.id, e);
                        zbus::fdo::Error::Failed(format!("Serialization error: {}", e))
                    })
            })
            .collect();

        result
    }
}
