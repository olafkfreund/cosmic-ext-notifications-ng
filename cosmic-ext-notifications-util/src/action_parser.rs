use crate::{NotificationAction, Hint};

/// Parse DBus action array (alternating id/label pairs) into structured actions
///
/// DBus format: ["id1", "label1", "id2", "label2", ...]
pub fn parse_actions(raw_actions: &[String]) -> Vec<NotificationAction> {
    raw_actions
        .chunks_exact(2)
        .map(|chunk| NotificationAction {
            id: chunk[0].clone(),
            label: chunk[1].clone(),
        })
        .collect()
}

/// Parse actions from string slice (common DBus format)
pub fn parse_actions_from_strs(raw_actions: &[&str]) -> Vec<NotificationAction> {
    raw_actions
        .chunks_exact(2)
        .map(|chunk| NotificationAction {
            id: chunk[0].to_string(),
            label: chunk[1].to_string(),
        })
        .collect()
}

/// Check if notification has action icons hint
pub fn has_action_icons(hints: &[Hint]) -> bool {
    hints.iter().any(|h| matches!(h, Hint::ActionIcons(true)))
}

/// Get the default action if present (action with id "default")
pub fn get_default_action(actions: &[NotificationAction]) -> Option<&NotificationAction> {
    actions.iter().find(|a| a.id == "default")
}

/// Get non-default actions (for displaying as buttons)
pub fn get_button_actions(actions: &[NotificationAction]) -> Vec<&NotificationAction> {
    actions.iter().filter(|a| a.id != "default").collect()
}

/// Limit actions to a maximum count (for UI display)
pub fn limit_actions(actions: &[NotificationAction], max: usize) -> Vec<&NotificationAction> {
    actions.iter().filter(|a| a.id != "default").take(max).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_action_pairs() {
        let raw = vec![
            "reply".to_string(), "Reply".to_string(),
            "mark_read".to_string(), "Mark as Read".to_string(),
        ];
        let actions = parse_actions(&raw);

        assert_eq!(actions.len(), 2);
        assert_eq!(actions[0].id, "reply");
        assert_eq!(actions[0].label, "Reply");
        assert_eq!(actions[1].id, "mark_read");
        assert_eq!(actions[1].label, "Mark as Read");
    }

    #[test]
    fn test_parse_actions_from_strs() {
        let raw = vec!["reply", "Reply", "dismiss", "Dismiss"];
        let actions = parse_actions_from_strs(&raw);

        assert_eq!(actions.len(), 2);
        assert_eq!(actions[0].id, "reply");
    }

    #[test]
    fn test_odd_action_count_ignored() {
        let raw = vec![
            "reply".to_string(), "Reply".to_string(),
            "orphan".to_string(),  // Missing label - should be ignored
        ];
        let actions = parse_actions(&raw);

        assert_eq!(actions.len(), 1);  // Only complete pair
    }

    #[test]
    fn test_empty_actions() {
        let raw: Vec<String> = vec![];
        let actions = parse_actions(&raw);
        assert!(actions.is_empty());
    }

    #[test]
    fn test_default_action_handling() {
        let actions = vec![
            NotificationAction { id: "default".to_string(), label: "".to_string() },
            NotificationAction { id: "reply".to_string(), label: "Reply".to_string() },
        ];

        let default = get_default_action(&actions);
        assert!(default.is_some());
        assert_eq!(default.unwrap().id, "default");
    }

    #[test]
    fn test_get_button_actions_excludes_default() {
        let actions = vec![
            NotificationAction { id: "default".to_string(), label: "".to_string() },
            NotificationAction { id: "reply".to_string(), label: "Reply".to_string() },
            NotificationAction { id: "dismiss".to_string(), label: "Dismiss".to_string() },
        ];

        let buttons = get_button_actions(&actions);
        assert_eq!(buttons.len(), 2);
        assert!(buttons.iter().all(|a| a.id != "default"));
    }

    #[test]
    fn test_limit_actions() {
        let actions = vec![
            NotificationAction { id: "a".to_string(), label: "A".to_string() },
            NotificationAction { id: "b".to_string(), label: "B".to_string() },
            NotificationAction { id: "c".to_string(), label: "C".to_string() },
            NotificationAction { id: "d".to_string(), label: "D".to_string() },
        ];

        let limited = limit_actions(&actions, 2);
        assert_eq!(limited.len(), 2);
    }

    #[test]
    fn test_has_action_icons() {
        use crate::Hint;

        let hints_with = vec![Hint::ActionIcons(true)];
        let hints_without = vec![Hint::ActionIcons(false)];
        let hints_empty: Vec<Hint> = vec![];

        assert!(has_action_icons(&hints_with));
        assert!(!has_action_icons(&hints_without));
        assert!(!has_action_icons(&hints_empty));
    }
}
