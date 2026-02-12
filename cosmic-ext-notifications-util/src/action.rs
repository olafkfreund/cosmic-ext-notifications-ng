use serde::{Deserialize, Serialize};

/// Represents an action button that can be displayed on a notification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NotificationAction {
    /// Unique identifier for the action
    pub id: String,
    /// User-visible label for the action button
    pub label: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_action_creation() {
        let action = NotificationAction {
            id: "reply".to_string(),
            label: "Reply".to_string(),
        };

        assert_eq!(action.id, "reply");
        assert_eq!(action.label, "Reply");
    }

    #[test]
    fn test_notification_action_with_default_id() {
        let action = NotificationAction {
            id: "default".to_string(),
            label: "Open".to_string(),
        };

        assert_eq!(action.id, "default");
        assert_eq!(action.label, "Open");
    }

    #[test]
    fn test_notification_action_clone() {
        let action = NotificationAction {
            id: "dismiss".to_string(),
            label: "Dismiss".to_string(),
        };

        let cloned = action.clone();
        assert_eq!(action, cloned);
    }

    #[test]
    fn test_notification_action_equality() {
        let action1 = NotificationAction {
            id: "view".to_string(),
            label: "View Details".to_string(),
        };

        let action2 = NotificationAction {
            id: "view".to_string(),
            label: "View Details".to_string(),
        };

        let action3 = NotificationAction {
            id: "dismiss".to_string(),
            label: "View Details".to_string(),
        };

        assert_eq!(action1, action2);
        assert_ne!(action1, action3);
    }

    #[test]
    fn test_notification_action_serialization() {
        let action = NotificationAction {
            id: "archive".to_string(),
            label: "Archive".to_string(),
        };

        let serialized = serde_json::to_string(&action).unwrap();
        let deserialized: NotificationAction = serde_json::from_str(&serialized).unwrap();

        assert_eq!(action, deserialized);
    }

    #[test]
    fn test_notification_action_debug_format() {
        let action = NotificationAction {
            id: "delete".to_string(),
            label: "Delete".to_string(),
        };

        let debug_str = format!("{:?}", action);
        assert!(debug_str.contains("NotificationAction"));
        assert!(debug_str.contains("delete"));
        assert!(debug_str.contains("Delete"));
    }

    #[test]
    fn test_notification_action_empty_strings() {
        let action = NotificationAction {
            id: String::new(),
            label: String::new(),
        };

        assert_eq!(action.id, "");
        assert_eq!(action.label, "");
    }

    #[test]
    fn test_multiple_actions_in_collection() {
        let actions = vec![
            NotificationAction {
                id: "reply".to_string(),
                label: "Reply".to_string(),
            },
            NotificationAction {
                id: "forward".to_string(),
                label: "Forward".to_string(),
            },
            NotificationAction {
                id: "delete".to_string(),
                label: "Delete".to_string(),
            },
        ];

        assert_eq!(actions.len(), 3);
        assert_eq!(actions[0].id, "reply");
        assert_eq!(actions[1].id, "forward");
        assert_eq!(actions[2].id, "delete");
    }
}
