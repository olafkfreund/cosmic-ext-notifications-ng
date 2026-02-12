use serde::{Deserialize, Serialize};

/// Represents a clickable link within a notification body
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NotificationLink {
    /// The URL to open when the link is clicked
    pub url: String,
    /// Optional display title for the link
    pub title: Option<String>,
    /// Starting position of the link in the notification body
    pub start: usize,
    /// Length of the link text in characters
    pub length: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_link_creation() {
        let link = NotificationLink {
            url: "https://example.com".to_string(),
            title: Some("Example".to_string()),
            start: 0,
            length: 7,
        };

        assert_eq!(link.url, "https://example.com");
        assert_eq!(link.title, Some("Example".to_string()));
        assert_eq!(link.start, 0);
        assert_eq!(link.length, 7);
    }

    #[test]
    fn test_notification_link_without_title() {
        let link = NotificationLink {
            url: "https://example.com".to_string(),
            title: None,
            start: 5,
            length: 10,
        };

        assert_eq!(link.url, "https://example.com");
        assert_eq!(link.title, None);
        assert_eq!(link.start, 5);
        assert_eq!(link.length, 10);
    }

    #[test]
    fn test_notification_link_clone() {
        let link = NotificationLink {
            url: "https://example.com".to_string(),
            title: Some("Example".to_string()),
            start: 0,
            length: 7,
        };

        let cloned = link.clone();
        assert_eq!(link, cloned);
    }

    #[test]
    fn test_notification_link_equality() {
        let link1 = NotificationLink {
            url: "https://example.com".to_string(),
            title: Some("Example".to_string()),
            start: 0,
            length: 7,
        };

        let link2 = NotificationLink {
            url: "https://example.com".to_string(),
            title: Some("Example".to_string()),
            start: 0,
            length: 7,
        };

        let link3 = NotificationLink {
            url: "https://different.com".to_string(),
            title: Some("Example".to_string()),
            start: 0,
            length: 7,
        };

        assert_eq!(link1, link2);
        assert_ne!(link1, link3);
    }

    #[test]
    fn test_notification_link_serialization() {
        let link = NotificationLink {
            url: "https://example.com".to_string(),
            title: Some("Example".to_string()),
            start: 0,
            length: 7,
        };

        let serialized = serde_json::to_string(&link).unwrap();
        let deserialized: NotificationLink = serde_json::from_str(&serialized).unwrap();

        assert_eq!(link, deserialized);
    }

    #[test]
    fn test_notification_link_debug_format() {
        let link = NotificationLink {
            url: "https://example.com".to_string(),
            title: Some("Example".to_string()),
            start: 0,
            length: 7,
        };

        let debug_str = format!("{:?}", link);
        assert!(debug_str.contains("NotificationLink"));
        assert!(debug_str.contains("https://example.com"));
    }
}
