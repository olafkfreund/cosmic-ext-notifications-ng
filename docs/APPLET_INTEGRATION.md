# Cosmic Applet Notification Integration Guide

> Documentation for integrating cosmic-applet-notification with the rich history API in cosmic-ext-notifications

**Version:** 1.0
**Last Updated:** 2026-02-02
**Target Daemon Version:** cosmic-ext-notifications v0.2.1+

## Overview

This guide documents how the [cosmic-applet-notification](https://github.com/olafkfreund/cosmic-applet-notification) should integrate with the new rich notification history API provided by cosmic-ext-notifications daemon. The daemon now exposes a `get_history()` D-Bus method that returns full notification metadata, which the applet can use to display historical notifications with rich content support.

## Current Architecture

### How the Applet Receives Notifications

The applet currently receives notifications through two mechanisms:

#### 1. Real-time Notification Stream (D-Bus Signal)

Located in: `src/dbus/service.rs` and `src/dbus/types.rs`

The applet subscribes to D-Bus signals from the daemon via the `org.freedesktop.Notifications` interface:

```rust
// Subscription pattern in src/main.rs
pub fn subscribe<Message>(
    mapper: impl Fn(ServerEvent) -> Message + Send + Sync + 'static + Clone,
) -> Subscription<Message>
```

When notifications arrive, they are mapped to `Message::NotificationReceived(Box<Notification>)` and processed in `update()`:

```rust
Message::NotificationReceived(notification) => {
    let notification = *notification;
    let notification_id = notification.id;

    // Add notification to manager
    let action = self.manager.add_notification(notification.clone());
    // ... handle animation, progress indicators, toast windows
}
```

#### 2. Local History Management (Disk Storage)

Located in: `src/manager/mod.rs` and `src/manager/storage.rs`

The applet maintains its own notification history using a local storage mechanism:

```rust
pub fn with_history(max_history_items: usize, retention_days: Option<u32>) -> Self {
    let storage = storage::HistoryStorage::new();
    let mut history = storage.load();

    storage::HistoryStorage::cleanup_old_notifications(&mut history, retention_days);
    storage::HistoryStorage::enforce_size_limit(&mut history, max_history_items);
    // ...
}
```

**Key behavior:**
- History is stored locally at `~/.local/share/cosmic-applet-notification/history.json`
- Notifications are added to history when dismissed, filtered out, or evicted
- History is loaded on startup and saved periodically

### Current Notification Struct (Applet)

Located in: `src/dbus/types.rs`

```rust
pub struct Notification {
    pub id: u32,
    pub app_name: String,
    pub replaces_id: u32,
    pub app_icon: String,
    pub summary: String,
    pub body: String,
    pub actions: Vec<NotificationAction>,
    pub hints: NotificationHints,
    pub raw_hints: HashMap<String, OwnedValue>,  // NOT cloned, lost on clone
    pub expire_timeout: i32,
    pub timestamp: DateTime<Local>,
}
```

**Important limitation:**
- The applet's `Notification` struct uses `chrono::DateTime<Local>` for timestamps
- The `raw_hints` field is intentionally not cloned (zbus OwnedValue limitation)
- Standard hints are parsed into the `hints` field

## New Daemon History API

### D-Bus Method Signature

Located in daemon: `src/subscriptions/applet.rs`

The daemon exposes a new D-Bus method on the `com.system76.NotificationsApplet` interface:

```rust
pub async fn get_history(&self)
    -> zbus::fdo::Result<Vec<(u32, String, String, String, String, i64)>>
```

**Returns:** Vector of tuples containing:
1. `u32` - notification ID
2. `String` - app_name
3. `String` - summary
4. `String` - body (sanitized, may contain HTML entities)
5. `String` - app_icon
6. `i64` - timestamp (Unix epoch seconds)

### Daemon's Notification Struct

Located in daemon: `cosmic-ext-notifications-util/src/lib.rs`

```rust
pub struct Notification {
    pub id: u32,
    pub app_name: String,
    pub app_icon: String,
    pub summary: String,
    pub body: String,
    pub actions: Vec<(ActionId, String)>,
    pub hints: Vec<Hint>,
    pub expire_timeout: i32,
    pub time: SystemTime,
}
```

**Key differences from applet:**
- Uses `SystemTime` instead of `DateTime<Local>`
- Actions stored as `Vec<(ActionId, String)>` not `Vec<NotificationAction>`
- Hints stored as `Vec<Hint>` (enum) not `NotificationHints` (struct)
- Body may contain HTML entities (&#58;, &#x3A;) that need decoding
- Body has HTML tags stripped but href URLs extracted

### What the Daemon Returns

The `get_history()` method currently returns a **simplified tuple format**:

```rust
// From applet.rs lines 190-196
let result: Vec<_> = notifications.into_iter().map(|n| {
    let timestamp = n.time
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    (n.id, n.app_name, n.summary, n.body, n.app_icon, timestamp)
}).collect();
```

**Missing from current response:**
- Actions (buttons)
- Hints (urgency, category, image paths, etc.)
- `expire_timeout`
- `replaces_id`

## Structural Mapping Analysis

### Field-by-Field Comparison

| Applet Field | Daemon Field | Mapping Notes |
|--------------|--------------|---------------|
| `id: u32` | `id: u32` | ✅ Direct match |
| `app_name: String` | `app_name: String` | ✅ Direct match |
| `replaces_id: u32` | N/A | ❌ Not available in daemon struct |
| `app_icon: String` | `app_icon: String` | ✅ Direct match |
| `summary: String` | `summary: String` | ✅ Direct match |
| `body: String` | `body: String` | ✅ Match, but may need HTML entity decoding |
| `actions: Vec<NotificationAction>` | `actions: Vec<(ActionId, String)>` | ⚠️ Needs conversion |
| `hints: NotificationHints` | `hints: Vec<Hint>` | ⚠️ Needs conversion |
| `raw_hints: HashMap<String, OwnedValue>` | N/A | ❌ Not preserved |
| `expire_timeout: i32` | `expire_timeout: i32` | ✅ Match, but not in current tuple |
| `timestamp: DateTime<Local>` | `time: SystemTime` | ⚠️ Needs conversion |

### Type Conversion Requirements

#### 1. Actions Conversion

**Daemon format:**
```rust
pub enum ActionId {
    Default,
    Custom(String),
}

// Stored as: Vec<(ActionId, String)>
```

**Applet format:**
```rust
pub struct NotificationAction {
    pub key: String,
    pub label: String,
}
```

**Conversion logic:**
```rust
fn convert_actions(daemon_actions: Vec<(ActionId, String)>) -> Vec<NotificationAction> {
    daemon_actions.into_iter().map(|(id, label)| {
        NotificationAction {
            key: match id {
                ActionId::Default => "default".to_string(),
                ActionId::Custom(s) => s,
            },
            label,
        }
    }).collect()
}
```

#### 2. Hints Conversion

**Daemon format (enum):**
```rust
pub enum Hint {
    ActionIcons(bool),
    Category(String),
    DesktopEntry(String),
    Image(Image),
    IconData(Vec<u8>),
    Resident(bool),
    SenderPid(u32),
    SoundFile(PathBuf),
    SoundName(String),
    SuppressSound(bool),
    Transient(bool),
    Urgency(u8),
    Value(i32),
    X(i32),
    Y(i32),
}
```

**Applet format (struct):**
```rust
pub struct NotificationHints {
    pub urgency: Urgency,
    pub category: Option<String>,
    pub desktop_entry: Option<String>,
    pub transient: bool,
    pub resident: bool,
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub sound_file: Option<String>,
    pub sound_name: Option<String>,
    pub suppress_sound: bool,
    pub action_icons: bool,
    pub image_data: Option<ImageData>,
    pub image_path: Option<String>,
}
```

**Conversion logic:**
```rust
fn convert_hints(daemon_hints: Vec<Hint>) -> NotificationHints {
    let mut hints = NotificationHints::default();

    for hint in daemon_hints {
        match hint {
            Hint::Urgency(u) => {
                hints.urgency = Urgency::from_u8(u).unwrap_or(Urgency::Normal);
            }
            Hint::Category(c) => hints.category = Some(c),
            Hint::DesktopEntry(d) => hints.desktop_entry = Some(d),
            Hint::Transient(t) => hints.transient = t,
            Hint::Resident(r) => hints.resident = r,
            Hint::X(x) => hints.x = Some(x),
            Hint::Y(y) => hints.y = Some(y),
            Hint::SoundFile(f) => hints.sound_file = Some(f.to_string_lossy().to_string()),
            Hint::SoundName(s) => hints.sound_name = Some(s),
            Hint::SuppressSound(s) => hints.suppress_sound = s,
            Hint::ActionIcons(a) => hints.action_icons = a,
            Hint::Image(img) => match img {
                Image::File(path) => {
                    hints.image_path = Some(path.to_string_lossy().to_string());
                }
                Image::Name(name) => {
                    hints.image_path = Some(name);
                }
                Image::Data { width, height, data } => {
                    hints.image_data = Some(ImageData {
                        width,
                        height,
                        rowstride: (width * 4) as i32,
                        has_alpha: true,
                        bits_per_sample: 8,
                        channels: 4,
                        data,
                    });
                }
            },
            _ => {} // Other hints not used by applet
        }
    }

    hints
}
```

#### 3. Timestamp Conversion

**From daemon `SystemTime` to applet `DateTime<Local>`:**

```rust
use chrono::{DateTime, Local, TimeZone};

fn convert_timestamp(system_time: SystemTime) -> DateTime<Local> {
    let duration = system_time
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();

    Local.timestamp_opt(
        duration.as_secs() as i64,
        duration.subsec_nanos()
    ).unwrap()
}
```

**From daemon tuple format (i64 epoch seconds):**

```rust
fn convert_timestamp_from_tuple(epoch_secs: i64) -> DateTime<Local> {
    Local.timestamp_opt(epoch_secs, 0).unwrap()
}
```

## Proposed Integration Approach

### Option 1: Enhanced D-Bus Method (Recommended)

**What needs to change:** Modify daemon to return full notification data as JSON

#### Daemon Changes

Add a new method to `src/subscriptions/applet.rs`:

```rust
#[interface(name = "com.system76.NotificationsApplet")]
impl NotificationsApplet {
    /// Get full notification history with all metadata
    /// Returns JSON-serialized notifications for easy deserialization
    pub async fn get_history_full(&self) -> zbus::fdo::Result<String> {
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
                tracing::error!("Failed to receive history: {}", err);
                return Err(zbus::fdo::Error::Failed("Channel closed".to_string()));
            }
            Err(_) => {
                tracing::error!("Timeout waiting for history");
                return Err(zbus::fdo::Error::Failed("Timeout".to_string()));
            }
        };

        // Serialize notifications to JSON
        match serde_json::to_string(&notifications) {
            Ok(json) => Ok(json),
            Err(e) => {
                tracing::error!("Failed to serialize history: {}", e);
                Err(zbus::fdo::Error::Failed(e.to_string()))
            }
        }
    }
}
```

**Rationale:**
- Avoids D-Bus type mapping complexity
- Preserves all notification metadata (actions, hints, timestamps)
- Easy to extend in the future
- Daemon's `Notification` struct already has `#[derive(Serialize, Deserialize)]`

#### Applet Changes

**Step 1:** Add daemon notification types to applet

Create `src/dbus/daemon_types.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Daemon's notification structure (from cosmic-ext-notifications-util)
/// This matches the daemon's serialized format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonNotification {
    pub id: u32,
    pub app_name: String,
    pub app_icon: String,
    pub summary: String,
    pub body: String,
    pub actions: Vec<(DaemonActionId, String)>,
    pub hints: Vec<DaemonHint>,
    pub expire_timeout: i32,
    pub time: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DaemonActionId {
    Default,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DaemonHint {
    ActionIcons(bool),
    Category(String),
    DesktopEntry(String),
    Image(DaemonImage),
    IconData(Vec<u8>),
    Resident(bool),
    SenderPid(u32),
    SoundFile(std::path::PathBuf),
    SoundName(String),
    SuppressSound(bool),
    Transient(bool),
    Urgency(u8),
    Value(i32),
    X(i32),
    Y(i32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DaemonImage {
    Name(String),
    File(std::path::PathBuf),
    Data {
        width: u32,
        height: u32,
        data: Vec<u8>,
    },
}
```

**Step 2:** Add conversion functions

Create `src/dbus/conversion.rs`:

```rust
use super::daemon_types::{DaemonNotification, DaemonActionId, DaemonHint, DaemonImage};
use super::types::{Notification, NotificationAction, NotificationHints, Urgency, ImageData};
use chrono::{DateTime, Local, TimeZone};
use std::time::SystemTime;

/// Convert daemon notification to applet notification
pub fn convert_daemon_notification(daemon_notif: DaemonNotification) -> Notification {
    Notification {
        id: daemon_notif.id,
        app_name: daemon_notif.app_name,
        replaces_id: 0, // Not available from history
        app_icon: daemon_notif.app_icon,
        summary: daemon_notif.summary,
        body: daemon_notif.body,
        actions: convert_actions(daemon_notif.actions),
        hints: convert_hints(daemon_notif.hints),
        raw_hints: std::collections::HashMap::new(), // Cannot reconstruct
        expire_timeout: daemon_notif.expire_timeout,
        timestamp: convert_timestamp(daemon_notif.time),
    }
}

fn convert_actions(daemon_actions: Vec<(DaemonActionId, String)>) -> Vec<NotificationAction> {
    daemon_actions.into_iter().map(|(id, label)| {
        NotificationAction {
            key: match id {
                DaemonActionId::Default => "default".to_string(),
                DaemonActionId::Custom(s) => s,
            },
            label,
        }
    }).collect()
}

fn convert_hints(daemon_hints: Vec<DaemonHint>) -> NotificationHints {
    let mut hints = NotificationHints::default();

    for hint in daemon_hints {
        match hint {
            DaemonHint::Urgency(u) => {
                hints.urgency = Urgency::from_u8(u).unwrap_or(Urgency::Normal);
            }
            DaemonHint::Category(c) => hints.category = Some(c),
            DaemonHint::DesktopEntry(d) => hints.desktop_entry = Some(d),
            DaemonHint::Transient(t) => hints.transient = t,
            DaemonHint::Resident(r) => hints.resident = r,
            DaemonHint::X(x) => hints.x = Some(x),
            DaemonHint::Y(y) => hints.y = Some(y),
            DaemonHint::SoundFile(f) => {
                hints.sound_file = Some(f.to_string_lossy().to_string());
            }
            DaemonHint::SoundName(s) => hints.sound_name = Some(s),
            DaemonHint::SuppressSound(s) => hints.suppress_sound = s,
            DaemonHint::ActionIcons(a) => hints.action_icons = a,
            DaemonHint::Image(img) => match img {
                DaemonImage::File(path) => {
                    hints.image_path = Some(path.to_string_lossy().to_string());
                }
                DaemonImage::Name(name) => {
                    hints.image_path = Some(name);
                }
                DaemonImage::Data { width, height, data } => {
                    hints.image_data = Some(ImageData {
                        width: width as i32,
                        height: height as i32,
                        rowstride: (width * 4) as i32,
                        has_alpha: true,
                        bits_per_sample: 8,
                        channels: 4,
                        data,
                    });
                }
            },
            _ => {} // Other hints not used by applet
        }
    }

    hints
}

fn convert_timestamp(system_time: SystemTime) -> DateTime<Local> {
    let duration = system_time
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();

    Local.timestamp_opt(
        duration.as_secs() as i64,
        duration.subsec_nanos()
    ).unwrap()
}
```

**Step 3:** Add D-Bus client method to call daemon

Modify `src/dbus/service.rs` to add a client function:

```rust
/// Fetch notification history from the daemon
///
/// Returns full notification metadata including actions and hints
pub async fn fetch_history_from_daemon() -> Result<Vec<Notification>, Box<dyn std::error::Error>> {
    // Connect to daemon's applet interface
    let connection = zbus::Connection::session().await?;

    let proxy = zbus::Proxy::new(
        &connection,
        "com.system76.NotificationsSocket",
        "/com/system76/NotificationsApplet",
        "com.system76.NotificationsApplet",
    ).await?;

    // Call get_history_full method
    let json_response: String = proxy.call("get_history_full", &()).await?;

    // Deserialize JSON response
    let daemon_notifications: Vec<DaemonNotification> = serde_json::from_str(&json_response)?;

    // Convert to applet notifications
    Ok(daemon_notifications
        .into_iter()
        .map(convert_daemon_notification)
        .collect())
}
```

**Step 4:** Integrate with NotificationManager

Modify `src/manager/mod.rs` to support daemon history loading:

```rust
impl NotificationManager {
    /// Load history from daemon instead of local disk
    ///
    /// This should be called on startup instead of with_history()
    pub async fn with_daemon_history(
        max_history_items: usize,
        retention_days: Option<u32>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Fetch history from daemon
        let mut history: VecDeque<Notification> =
            crate::dbus::fetch_history_from_daemon()
                .await?
                .into_iter()
                .collect();

        // Apply cleanup based on retention policy
        storage::HistoryStorage::cleanup_old_notifications_chrono(
            &mut history,
            retention_days
        );

        // Enforce size limit
        storage::HistoryStorage::enforce_size_limit_generic(
            &mut history,
            max_history_items
        );

        tracing::info!(
            "Initialized manager with {} notifications from daemon",
            history.len()
        );

        Ok(Self {
            active_notifications: VecDeque::new(),
            notification_history: history,
            next_id: 1,
            do_not_disturb: false,
            app_filters: HashMap::new(),
            min_urgency_level: 0,
        })
    }
}
```

**Step 5:** Update initialization in main.rs

```rust
impl Application for NotificationApplet {
    fn init(
        core: cosmic::app::Core,
        _flags: Self::Flags,
    ) -> (Self, Task<cosmic::Action<Self::Message>>) {
        let config_helper = config::ConfigHelper::new();
        let config = config_helper.load();

        tracing::info!("Configuration loaded from {:?}", config_helper.path());

        // Try daemon history first, fallback to local history
        let manager_future = async move {
            if config.history_enabled {
                match manager::NotificationManager::with_daemon_history(
                    config.max_history_items,
                    config.history_retention_days,
                ).await {
                    Ok(mgr) => {
                        tracing::info!("Loaded history from daemon");
                        mgr
                    }
                    Err(e) => {
                        tracing::warn!("Failed to load daemon history: {}, falling back to local", e);
                        manager::NotificationManager::with_history(
                            config.max_history_items,
                            config.history_retention_days,
                        )
                    }
                }
            } else {
                manager::NotificationManager::new()
            }
        };

        let manager_task = Task::future(manager_future);

        // ... rest of initialization
    }
}
```

### Option 2: Use Existing Tuple Method (Simpler, Limited)

If full metadata is not required, the applet can use the existing `get_history()` method:

```rust
/// Fetch simplified history from daemon (tuple format)
pub async fn fetch_simple_history() -> Result<Vec<Notification>, Box<dyn std::error::Error>> {
    let connection = zbus::Connection::session().await?;
    let proxy = zbus::Proxy::new(
        &connection,
        "com.system76.NotificationsSocket",
        "/com/system76/NotificationsApplet",
        "com.system76.NotificationsApplet",
    ).await?;

    // Returns: Vec<(u32, String, String, String, String, i64)>
    // (id, app_name, summary, body, app_icon, timestamp_secs)
    let tuples: Vec<(u32, String, String, String, String, i64)> =
        proxy.call("get_history", &()).await?;

    // Convert tuples to notifications
    Ok(tuples.into_iter().map(|(id, app_name, summary, body, app_icon, timestamp)| {
        Notification {
            id,
            app_name,
            replaces_id: 0,
            app_icon,
            summary,
            body,
            actions: vec![],
            hints: NotificationHints::default(),
            raw_hints: HashMap::new(),
            expire_timeout: 0,
            timestamp: Local.timestamp_opt(timestamp, 0).unwrap(),
        }
    }).collect())
}
```

**Limitations:**
- No action buttons in history
- No urgency/category/image information
- No progress indicators
- Less useful for displaying rich historical notifications

## Configuration Considerations

### New Config Options

Add to `src/config.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppletConfig {
    // ... existing fields ...

    /// Use daemon history instead of local storage
    /// When true, fetch history from daemon on startup
    #[serde(default = "default_use_daemon_history")]
    pub use_daemon_history: bool,

    /// Fallback to local history if daemon is unavailable
    #[serde(default = "default_fallback_to_local")]
    pub fallback_to_local_history: bool,
}

fn default_use_daemon_history() -> bool {
    true // Prefer daemon history
}

fn default_fallback_to_local() -> bool {
    true // Graceful degradation
}
```

### Migration Strategy

For users upgrading from local-only history:

1. **Dual Storage (Transition Period)**
   - Continue saving to local history
   - Fetch from daemon on startup
   - Merge both sources, deduplicate by ID

2. **Deprecation Warning**
   - Add config option `legacy_local_history = true`
   - Log warning on startup if local history exists
   - Provide migration tool to export local history to daemon

3. **Complete Migration**
   - Stop writing to local history
   - Only use daemon as source of truth
   - Keep local history file for rollback

## Testing Plan

### Unit Tests

**Test conversion functions:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_actions() {
        let daemon_actions = vec![
            (DaemonActionId::Default, "Open".to_string()),
            (DaemonActionId::Custom("reply".to_string()), "Reply".to_string()),
        ];

        let applet_actions = convert_actions(daemon_actions);

        assert_eq!(applet_actions.len(), 2);
        assert_eq!(applet_actions[0].key, "default");
        assert_eq!(applet_actions[0].label, "Open");
        assert_eq!(applet_actions[1].key, "reply");
    }

    #[test]
    fn test_convert_hints_urgency() {
        let daemon_hints = vec![
            DaemonHint::Urgency(2),
            DaemonHint::Category("email.arrived".to_string()),
        ];

        let applet_hints = convert_hints(daemon_hints);

        assert_eq!(applet_hints.urgency, Urgency::Critical);
        assert_eq!(applet_hints.category, Some("email.arrived".to_string()));
    }

    #[test]
    fn test_timestamp_conversion() {
        let now = SystemTime::now();
        let converted = convert_timestamp(now);

        // Should be within 1 second of now
        let diff = Local::now().signed_duration_since(converted);
        assert!(diff.num_seconds().abs() < 1);
    }
}
```

### Integration Tests

**Test daemon communication:**

```rust
#[tokio::test]
async fn test_fetch_history_from_daemon() {
    // Requires running daemon
    match fetch_history_from_daemon().await {
        Ok(notifications) => {
            println!("Fetched {} notifications", notifications.len());

            for notif in notifications {
                assert!(!notif.app_name.is_empty());
                assert!(!notif.summary.is_empty());
            }
        }
        Err(e) => {
            println!("Daemon not available: {}", e);
            // Not a failure if daemon isn't running
        }
    }
}
```

## Error Handling

### Daemon Unavailable

```rust
impl NotificationApplet {
    async fn load_history(&mut self) -> Result<(), String> {
        match fetch_history_from_daemon().await {
            Ok(history) => {
                self.manager.load_history(history);
                tracing::info!("Loaded history from daemon");
                Ok(())
            }
            Err(e) => {
                if self.config.fallback_to_local_history {
                    tracing::warn!("Daemon unavailable, using local history: {}", e);
                    let local_history = storage::HistoryStorage::new().load();
                    self.manager.load_history(local_history);
                    Ok(())
                } else {
                    Err(format!("Failed to load history: {}", e))
                }
            }
        }
    }
}
```

### Deserialization Errors

```rust
pub async fn fetch_history_from_daemon() -> Result<Vec<Notification>, Box<dyn std::error::Error>> {
    let json_response: String = proxy.call("get_history_full", &()).await?;

    let daemon_notifications: Vec<DaemonNotification> =
        serde_json::from_str(&json_response)
            .map_err(|e| {
                tracing::error!("Failed to deserialize history: {}", e);
                tracing::debug!("Response was: {}", json_response);
                e
            })?;

    Ok(daemon_notifications
        .into_iter()
        .map(convert_daemon_notification)
        .collect())
}
```

## Performance Considerations

### History Size Limits

The daemon maintains a hidden notification queue. Consider:

1. **Request Limiting**
   - Don't fetch full history on every applet restart
   - Cache history locally, only fetch delta
   - Implement pagination if history is very large

2. **JSON Serialization Overhead**
   - Daemon history might be 100+ notifications
   - JSON serialization adds ~10ms per 100 notifications
   - Consider adding `max_count` parameter to `get_history_full()`

### Startup Performance

```rust
// Add timeout to history fetch
pub async fn fetch_history_with_timeout(
    timeout_secs: u64
) -> Result<Vec<Notification>, Box<dyn std::error::Error>> {
    tokio::time::timeout(
        std::time::Duration::from_secs(timeout_secs),
        fetch_history_from_daemon()
    )
    .await?
}
```

## Backwards Compatibility

### Supporting Multiple Daemon Versions

```rust
pub async fn fetch_history_from_daemon() -> Result<Vec<Notification>, Box<dyn std::error::Error>> {
    let proxy = create_daemon_proxy().await?;

    // Try new method first (v0.2.1+)
    if let Ok(json) = proxy.call::<String>("get_history_full", &()).await {
        return Ok(parse_json_history(&json)?);
    }

    // Fallback to old tuple method (v0.2.0)
    tracing::warn!("Daemon doesn't support get_history_full, using legacy method");
    let tuples = proxy.call::<Vec<(u32, String, String, String, String, i64)>>(
        "get_history",
        &()
    ).await?;

    Ok(convert_tuple_history(tuples))
}
```

## Future Enhancements

### 1. Incremental Updates

Instead of fetching full history, add a method for delta updates:

```rust
/// Get notifications added since a specific timestamp
pub async fn get_history_since(&self, since: i64)
    -> zbus::fdo::Result<String>
```

### 2. Filtered History Queries

Allow applets to request filtered subsets:

```rust
pub async fn get_history_filtered(
    &self,
    app_name: Option<String>,
    urgency: Option<u8>,
    limit: Option<u32>,
) -> zbus::fdo::Result<String>
```

### 3. Rich Content Support

Enhance applet to display:
- Progress bars in historical notifications
- Clickable links in history view
- Animated images (if still valid)

## Summary

### What Needs to Change in the Applet

1. **Add daemon types** - Create `src/dbus/daemon_types.rs` with daemon's notification structures
2. **Add conversion logic** - Create `src/dbus/conversion.rs` to convert daemon → applet formats
3. **Add D-Bus client** - Add `fetch_history_from_daemon()` function to call new API
4. **Update manager** - Add `with_daemon_history()` method to load from daemon
5. **Update initialization** - Modify `init()` to try daemon history first
6. **Add config options** - Support `use_daemon_history` and fallback behavior
7. **Add error handling** - Gracefully degrade to local history if daemon unavailable

### What Needs to Change in the Daemon

**For Option 1 (Recommended):**
1. Add `get_history_full()` method that returns JSON-serialized notifications
2. No schema changes needed (already has Serialize/Deserialize)

**For Option 2 (Existing):**
1. No changes needed
2. Use existing `get_history()` method (limited metadata)

### Benefits of Integration

- **Single source of truth** - Daemon maintains canonical history
- **Rich content** - History includes actions, hints, urgency
- **Consistency** - Same notification data across all COSMIC components
- **Reduced duplication** - Don't need local storage
- **Better UX** - Users can see full history with all interactive elements

---

**Document Version:** 1.0
**Last Updated:** 2026-02-02
**Author:** Claude Opus 4.5
