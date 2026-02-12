use crate::subscriptions::notifications;
use cosmic::surface;
use cosmic_ext_notifications_util::ActionId;
use cosmic_time::Instant;

/// Application message types
#[derive(Debug, Clone)]
pub enum Message {
    /// Activate a notification (request activation token)
    ActivateNotification(u32),
    /// Activation token received for notification
    ActivationToken(Option<String>, u32, Option<ActionId>),
    /// Notification dismissed by user
    Dismissed(u32),
    /// Notification event from subscription
    Notification(notifications::Event),
    /// Notification timeout expired
    Timeout(u32),
    /// Configuration updated
    Config(cosmic_ext_notifications_config::NotificationsConfig),
    /// Panel configuration updated
    PanelConfig(cosmic_panel_config::CosmicPanelConfig),
    /// Dock configuration updated
    DockConfig(cosmic_panel_config::CosmicPanelConfig),
    /// Animation frame update
    Frame(Instant),
    /// No-op message
    Ignore,
    /// Surface action
    #[allow(dead_code)]
    Surface(surface::Action),
    /// Link clicked in notification body
    LinkClicked(String),
    /// Action button clicked (notification_id, action_id)
    ActionClicked(u32, String),
}
