use crate::NotificationUrgency;

/// RGBA color (values 0.0-1.0)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }
}

/// Colors for different urgency levels
pub mod urgency_colors {
    use super::Color;

    /// Low urgency - muted gray
    pub const LOW: Color = Color::new(0.5, 0.5, 0.5, 0.7);

    /// Normal urgency - accent blue
    pub const NORMAL: Color = Color::rgb(0.2, 0.6, 1.0);

    /// Critical urgency - alert red
    pub const CRITICAL: Color = Color::rgb(1.0, 0.3, 0.3);
}

/// Get the accent color for an urgency level
pub fn urgency_color(urgency: NotificationUrgency) -> Color {
    match urgency {
        NotificationUrgency::Low => urgency_colors::LOW,
        NotificationUrgency::Normal => urgency_colors::NORMAL,
        NotificationUrgency::Critical => urgency_colors::CRITICAL,
    }
}

/// Get the accent color for a raw urgency value (0, 1, 2)
pub fn urgency_color_from_u8(urgency: u8) -> Color {
    urgency_color(NotificationUrgency::from(urgency))
}

/// Common notification categories from freedesktop.org spec
pub mod categories {
    pub const DEVICE: &str = "device";
    pub const DEVICE_ADDED: &str = "device.added";
    pub const DEVICE_REMOVED: &str = "device.removed";

    pub const EMAIL: &str = "email";
    pub const EMAIL_ARRIVED: &str = "email.arrived";

    pub const IM: &str = "im";
    pub const IM_RECEIVED: &str = "im.received";

    pub const NETWORK: &str = "network";
    pub const NETWORK_CONNECTED: &str = "network.connected";
    pub const NETWORK_DISCONNECTED: &str = "network.disconnected";

    pub const PRESENCE: &str = "presence";
    pub const PRESENCE_ONLINE: &str = "presence.online";
    pub const PRESENCE_OFFLINE: &str = "presence.offline";

    pub const TRANSFER: &str = "transfer";
    pub const TRANSFER_COMPLETE: &str = "transfer.complete";
    pub const TRANSFER_ERROR: &str = "transfer.error";
}

/// Get a suggested icon name for a notification category
pub fn category_icon(category: &str) -> Option<&'static str> {
    match category {
        "email" | "email.arrived" => Some("mail-unread-symbolic"),
        "im" | "im.received" => Some("chat-message-new-symbolic"),
        "transfer" | "transfer.complete" => Some("folder-download-symbolic"),
        "transfer.error" => Some("dialog-error-symbolic"),
        "device" | "device.added" => Some("drive-removable-media-symbolic"),
        "device.removed" => Some("drive-removable-media-symbolic"),
        "network" | "network.connected" => Some("network-wireless-symbolic"),
        "network.disconnected" => Some("network-offline-symbolic"),
        "presence" | "presence.online" => Some("user-available-symbolic"),
        "presence.offline" => Some("user-offline-symbolic"),
        _ => None,
    }
}

/// Check if a category indicates a message/communication
pub fn is_message_category(category: &str) -> bool {
    matches!(category, "email" | "email.arrived" | "im" | "im.received")
}

/// Check if a category indicates a system/device event
pub fn is_system_category(category: &str) -> bool {
    category.starts_with("device") || category.starts_with("network")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_urgency_color_low() {
        let color = urgency_color(NotificationUrgency::Low);
        assert_eq!(color, urgency_colors::LOW);
        assert!(color.a < 1.0); // Low urgency should be muted
    }

    #[test]
    fn test_urgency_color_normal() {
        let color = urgency_color(NotificationUrgency::Normal);
        assert_eq!(color, urgency_colors::NORMAL);
    }

    #[test]
    fn test_urgency_color_critical() {
        let color = urgency_color(NotificationUrgency::Critical);
        assert_eq!(color, urgency_colors::CRITICAL);
        assert!(color.r > 0.5); // Critical should be reddish
    }

    #[test]
    fn test_urgency_color_from_u8() {
        assert_eq!(urgency_color_from_u8(0), urgency_colors::LOW);
        assert_eq!(urgency_color_from_u8(1), urgency_colors::NORMAL);
        assert_eq!(urgency_color_from_u8(2), urgency_colors::CRITICAL);
        // Invalid values should default to normal
        assert_eq!(urgency_color_from_u8(255), urgency_colors::NORMAL);
    }

    #[test]
    fn test_category_icon_email() {
        assert_eq!(category_icon("email"), Some("mail-unread-symbolic"));
        assert_eq!(category_icon("email.arrived"), Some("mail-unread-symbolic"));
    }

    #[test]
    fn test_category_icon_im() {
        assert_eq!(category_icon("im"), Some("chat-message-new-symbolic"));
        assert_eq!(category_icon("im.received"), Some("chat-message-new-symbolic"));
    }

    #[test]
    fn test_category_icon_transfer() {
        assert_eq!(category_icon("transfer"), Some("folder-download-symbolic"));
        assert_eq!(category_icon("transfer.error"), Some("dialog-error-symbolic"));
    }

    #[test]
    fn test_category_icon_unknown() {
        assert_eq!(category_icon("unknown.category"), None);
        assert_eq!(category_icon(""), None);
    }

    #[test]
    fn test_is_message_category() {
        assert!(is_message_category("email"));
        assert!(is_message_category("email.arrived"));
        assert!(is_message_category("im"));
        assert!(is_message_category("im.received"));
        assert!(!is_message_category("device"));
        assert!(!is_message_category("transfer"));
    }

    #[test]
    fn test_is_system_category() {
        assert!(is_system_category("device"));
        assert!(is_system_category("device.added"));
        assert!(is_system_category("network"));
        assert!(is_system_category("network.connected"));
        assert!(!is_system_category("email"));
        assert!(!is_system_category("im"));
    }

    #[test]
    fn test_color_constructors() {
        let rgba = Color::new(1.0, 0.5, 0.0, 0.8);
        assert_eq!(rgba.a, 0.8);

        let rgb = Color::rgb(1.0, 0.5, 0.0);
        assert_eq!(rgb.a, 1.0);
    }
}
