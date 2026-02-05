use cosmic_config::{CosmicConfigEntry, cosmic_config_derive::CosmicConfigEntry};

pub const ID: &str = "com.system76.CosmicNotifications";

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Anchor {
    #[default]
    Top,
    Bottom,
    Right,
    Left,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum GroupingMode {
    /// No grouping - show each notification individually (current behavior)
    #[default]
    None,
    /// Group notifications by app_name
    ByApp,
    /// Group notifications by category hint
    ByCategory,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct AppRule {
    /// The app_name to match (from notification)
    pub app_name: String,
    /// Optional desktop entry to match (more specific)
    pub desktop_entry: Option<String>,
    /// Whether notifications from this app are enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Override the urgency level (0=low, 1=normal, 2=critical)
    pub urgency_override: Option<u8>,
    /// Whether sounds are enabled for this app
    #[serde(default = "default_true")]
    pub sound_enabled: bool,
    /// Override timeout in milliseconds
    pub timeout_override: Option<u32>,
}

impl Default for AppRule {
    fn default() -> Self {
        Self {
            app_name: String::new(),
            desktop_entry: None,
            enabled: true,
            urgency_override: None,
            sound_enabled: true,
            timeout_override: None,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, CosmicConfigEntry)]
#[version = 3]
pub struct NotificationsConfig {
    pub do_not_disturb: bool,
    pub anchor: Anchor,
    /// The maximum number of notifications that can be displayed at once.
    pub max_notifications: u32,
    /// The maximum number of notifications that can be displayed per app if not urgent and constrained by `max_notifications`.
    pub max_per_app: u32,
    /// Max time in milliseconds a critical notification can be displayed before being removed.
    pub max_timeout_urgent: Option<u32>,
    /// Max time in milliseconds a normal notification can be displayed before being removed.
    pub max_timeout_normal: Option<u32>,
    /// Max time in milliseconds a low priority notification can be displayed before being removed.
    pub max_timeout_low: Option<u32>,

    // Rich notification configuration options
    /// Whether to display images in notifications (default: true)
    #[serde(default = "default_true")]
    pub show_images: bool,
    /// Whether to display action buttons in notifications (default: true)
    #[serde(default = "default_true")]
    pub show_actions: bool,
    /// Maximum width/height for notification images in pixels (default: 128, range: 32-256)
    #[serde(default = "default_max_image_size")]
    pub max_image_size: u32,
    /// Whether links in notification body are clickable (default: true)
    #[serde(default = "default_true")]
    pub enable_links: bool,
    /// Whether animated images (GIFs) play and card animations are enabled (default: true)
    #[serde(default = "default_true")]
    pub enable_animations: bool,

    /// Per-application notification rules
    #[serde(default)]
    pub app_rules: Vec<AppRule>,

    /// How to group notifications
    #[serde(default)]
    pub grouping_mode: GroupingMode,

    /// Maximum notifications per group before collapsing (default: 3)
    #[serde(default = "default_max_per_group")]
    pub max_per_group: u32,

    /// Whether to show group count badge (e.g., "Firefox (3)")
    #[serde(default = "default_true")]
    pub show_group_count: bool,
}

impl Default for NotificationsConfig {
    fn default() -> Self {
        Self {
            do_not_disturb: false,
            anchor: Anchor::default(),
            max_notifications: 3,
            max_per_app: 2,
            max_timeout_urgent: None,
            max_timeout_normal: Some(5000),
            max_timeout_low: Some(3000),
            show_images: default_true(),
            show_actions: default_true(),
            max_image_size: default_max_image_size(),
            enable_links: default_true(),
            enable_animations: default_true(),
            app_rules: Vec::new(),
            grouping_mode: GroupingMode::default(),
            max_per_group: default_max_per_group(),
            show_group_count: default_true(),
        }
    }
}

impl NotificationsConfig {
    /// Find a rule matching the given app_name and optional desktop_entry
    pub fn find_app_rule(&self, app_name: &str, desktop_entry: Option<&str>) -> Option<&AppRule> {
        // First try to match by desktop_entry (more specific)
        if let Some(entry) = desktop_entry {
            if let Some(rule) = self.app_rules.iter().find(|r| r.desktop_entry.as_deref() == Some(entry)) {
                return Some(rule);
            }
        }
        // Fall back to app_name match
        self.app_rules.iter().find(|r| r.app_name == app_name && r.desktop_entry.is_none())
    }

    /// Check if notifications are enabled for an app
    pub fn is_app_enabled(&self, app_name: &str, desktop_entry: Option<&str>) -> bool {
        self.find_app_rule(app_name, desktop_entry)
            .map(|r| r.enabled)
            .unwrap_or(true)
    }

    /// Check if sounds are enabled for an app
    pub fn is_sound_enabled_for_app(&self, app_name: &str, desktop_entry: Option<&str>) -> bool {
        self.find_app_rule(app_name, desktop_entry)
            .map(|r| r.sound_enabled)
            .unwrap_or(true)
    }
}

// Default value helpers for serde
const fn default_true() -> bool {
    true
}

const fn default_max_image_size() -> u32 {
    128
}

const fn default_max_per_group() -> u32 {
    3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = NotificationsConfig::default();

        // Test original fields
        assert!(!config.do_not_disturb);
        assert_eq!(config.max_notifications, 3);
        assert_eq!(config.max_per_app, 2);
        assert_eq!(config.max_timeout_normal, Some(5000));
        assert_eq!(config.max_timeout_low, Some(3000));
        assert_eq!(config.max_timeout_urgent, None);

        // Test new rich notification fields
        assert!(config.show_images);
        assert!(config.show_actions);
        assert_eq!(config.max_image_size, 128);
        assert!(config.enable_links);
        assert!(config.enable_animations);
    }

    #[test]
    fn test_config_serialization() {
        let config = NotificationsConfig::default();
        let json = serde_json::to_string(&config).unwrap();

        // Should serialize all fields
        assert!(json.contains("show_images"));
        assert!(json.contains("show_actions"));
        assert!(json.contains("max_image_size"));
        assert!(json.contains("enable_links"));
        assert!(json.contains("enable_animations"));
    }

    #[test]
    fn test_config_deserialization_with_defaults() {
        // Simulate old config file (version 1) without rich notification fields
        let old_config_json = r#"{
            "do_not_disturb": false,
            "anchor": "Top",
            "max_notifications": 3,
            "max_per_app": 2,
            "max_timeout_urgent": null,
            "max_timeout_normal": 5000,
            "max_timeout_low": 3000
        }"#;

        let config: NotificationsConfig = serde_json::from_str(old_config_json).unwrap();

        // Old fields should deserialize correctly
        assert!(!config.do_not_disturb);
        assert_eq!(config.max_notifications, 3);

        // New fields should use defaults
        assert!(config.show_images);
        assert!(config.show_actions);
        assert_eq!(config.max_image_size, 128);
        assert!(config.enable_links);
        assert!(config.enable_animations);
    }

    #[test]
    fn test_config_deserialization_full() {
        // Config with all fields including rich notification options
        let full_config_json = r#"{
            "do_not_disturb": true,
            "anchor": "Bottom",
            "max_notifications": 5,
            "max_per_app": 3,
            "max_timeout_urgent": null,
            "max_timeout_normal": 6000,
            "max_timeout_low": 4000,
            "show_images": false,
            "show_actions": false,
            "max_image_size": 64,
            "enable_links": false,
            "enable_animations": false
        }"#;

        let config: NotificationsConfig = serde_json::from_str(full_config_json).unwrap();

        // All fields should deserialize correctly
        assert!(config.do_not_disturb);
        assert_eq!(config.max_notifications, 5);
        assert_eq!(config.max_per_app, 3);
        assert!(!config.show_images);
        assert!(!config.show_actions);
        assert_eq!(config.max_image_size, 64);
        assert!(!config.enable_links);
        assert!(!config.enable_animations);
    }

    #[test]
    fn test_max_image_size_range() {
        // Test various max_image_size values
        let test_cases = vec![
            (32, 32),   // Minimum valid
            (128, 128), // Default
            (256, 256), // Maximum valid
            (16, 16),   // Below minimum (should be handled by RichCardConfig::from_notifications_config)
            (512, 512), // Above maximum (should be handled by RichCardConfig::from_notifications_config)
        ];

        for (input, expected) in test_cases {
            let json = format!(r#"{{
                "do_not_disturb": false,
                "anchor": "Top",
                "max_notifications": 3,
                "max_per_app": 2,
                "max_timeout_urgent": null,
                "max_timeout_normal": 5000,
                "max_timeout_low": 3000,
                "max_image_size": {}
            }}"#, input);

            let config: NotificationsConfig = serde_json::from_str(&json).unwrap();
            assert_eq!(config.max_image_size, expected, "max_image_size should be {}", expected);
        }
    }

    #[test]
    fn test_default_helpers() {
        assert_eq!(default_true(), true);
        assert_eq!(default_max_image_size(), 128);
        assert_eq!(default_max_per_group(), 3);
    }

    #[test]
    fn test_grouping_mode_defaults() {
        let mode = GroupingMode::default();
        assert_eq!(mode, GroupingMode::None);
    }

    #[test]
    fn test_grouping_mode_serialization() {
        let none = GroupingMode::None;
        let by_app = GroupingMode::ByApp;
        let by_category = GroupingMode::ByCategory;

        let none_json = serde_json::to_string(&none).unwrap();
        let by_app_json = serde_json::to_string(&by_app).unwrap();
        let by_category_json = serde_json::to_string(&by_category).unwrap();

        assert_eq!(none_json, r#""None""#);
        assert_eq!(by_app_json, r#""ByApp""#);
        assert_eq!(by_category_json, r#""ByCategory""#);
    }

    #[test]
    fn test_grouping_mode_deserialization() {
        let none: GroupingMode = serde_json::from_str(r#""None""#).unwrap();
        let by_app: GroupingMode = serde_json::from_str(r#""ByApp""#).unwrap();
        let by_category: GroupingMode = serde_json::from_str(r#""ByCategory""#).unwrap();

        assert_eq!(none, GroupingMode::None);
        assert_eq!(by_app, GroupingMode::ByApp);
        assert_eq!(by_category, GroupingMode::ByCategory);
    }

    #[test]
    fn test_config_with_grouping_defaults() {
        let config = NotificationsConfig::default();

        assert_eq!(config.grouping_mode, GroupingMode::None);
        assert_eq!(config.max_per_group, 3);
        assert!(config.show_group_count);
    }

    #[test]
    fn test_config_deserialization_with_grouping() {
        let config_json = r#"{
            "do_not_disturb": false,
            "anchor": "Top",
            "max_notifications": 3,
            "max_per_app": 2,
            "max_timeout_urgent": null,
            "max_timeout_normal": 5000,
            "max_timeout_low": 3000,
            "grouping_mode": "ByApp",
            "max_per_group": 5,
            "show_group_count": false
        }"#;

        let config: NotificationsConfig = serde_json::from_str(config_json).unwrap();

        assert_eq!(config.grouping_mode, GroupingMode::ByApp);
        assert_eq!(config.max_per_group, 5);
        assert!(!config.show_group_count);
    }

    #[test]
    fn test_config_backward_compatibility_grouping() {
        // Simulate old config without grouping fields
        let old_config_json = r#"{
            "do_not_disturb": false,
            "anchor": "Top",
            "max_notifications": 3,
            "max_per_app": 2,
            "max_timeout_urgent": null,
            "max_timeout_normal": 5000,
            "max_timeout_low": 3000
        }"#;

        let config: NotificationsConfig = serde_json::from_str(old_config_json).unwrap();

        // Grouping fields should use defaults
        assert_eq!(config.grouping_mode, GroupingMode::None);
        assert_eq!(config.max_per_group, 3);
        assert!(config.show_group_count);
    }

    #[test]
    fn test_app_rule_defaults() {
        let rule = AppRule {
            app_name: "test-app".to_string(),
            ..Default::default()
        };

        assert_eq!(rule.app_name, "test-app");
        assert_eq!(rule.desktop_entry, None);
        assert!(rule.enabled);
        assert_eq!(rule.urgency_override, None);
        assert!(rule.sound_enabled);
        assert_eq!(rule.timeout_override, None);
    }

    #[test]
    fn test_find_app_rule_by_app_name() {
        let mut config = NotificationsConfig::default();
        config.app_rules.push(AppRule {
            app_name: "firefox".to_string(),
            desktop_entry: None,
            enabled: false,
            urgency_override: Some(1),
            sound_enabled: false,
            timeout_override: Some(10000),
        });

        // Should find rule by app_name
        let rule = config.find_app_rule("firefox", None);
        assert!(rule.is_some());
        assert_eq!(rule.unwrap().app_name, "firefox");
        assert!(!rule.unwrap().enabled);

        // Should not find non-existent app
        let rule = config.find_app_rule("chrome", None);
        assert!(rule.is_none());
    }

    #[test]
    fn test_find_app_rule_by_desktop_entry() {
        let mut config = NotificationsConfig::default();
        config.app_rules.push(AppRule {
            app_name: "firefox".to_string(),
            desktop_entry: Some("firefox.desktop".to_string()),
            enabled: false,
            urgency_override: Some(2),
            sound_enabled: false,
            timeout_override: Some(15000),
        });

        // Should find rule by desktop_entry
        let rule = config.find_app_rule("firefox", Some("firefox.desktop"));
        assert!(rule.is_some());
        assert_eq!(rule.unwrap().urgency_override, Some(2));

        // Should not find with wrong desktop_entry
        let rule = config.find_app_rule("firefox", Some("wrong.desktop"));
        assert!(rule.is_none());

        // Should not find without desktop_entry
        let rule = config.find_app_rule("firefox", None);
        assert!(rule.is_none());
    }

    #[test]
    fn test_app_rule_precedence() {
        let mut config = NotificationsConfig::default();

        // Add generic app_name rule
        config.app_rules.push(AppRule {
            app_name: "firefox".to_string(),
            desktop_entry: None,
            enabled: true,
            urgency_override: Some(0),
            sound_enabled: true,
            timeout_override: Some(5000),
        });

        // Add specific desktop_entry rule
        config.app_rules.push(AppRule {
            app_name: "firefox".to_string(),
            desktop_entry: Some("firefox.desktop".to_string()),
            enabled: false,
            urgency_override: Some(2),
            sound_enabled: false,
            timeout_override: Some(10000),
        });

        // Desktop entry rule should take precedence
        let rule = config.find_app_rule("firefox", Some("firefox.desktop"));
        assert!(rule.is_some());
        assert!(!rule.unwrap().enabled);
        assert_eq!(rule.unwrap().urgency_override, Some(2));

        // Generic rule should be used when no desktop_entry provided
        let rule = config.find_app_rule("firefox", None);
        assert!(rule.is_some());
        assert!(rule.unwrap().enabled);
        assert_eq!(rule.unwrap().urgency_override, Some(0));
    }

    #[test]
    fn test_is_app_enabled() {
        let mut config = NotificationsConfig::default();
        config.app_rules.push(AppRule {
            app_name: "muted-app".to_string(),
            desktop_entry: None,
            enabled: false,
            urgency_override: None,
            sound_enabled: true,
            timeout_override: None,
        });

        // Disabled app
        assert!(!config.is_app_enabled("muted-app", None));

        // App without rule (default enabled)
        assert!(config.is_app_enabled("some-other-app", None));
    }

    #[test]
    fn test_is_sound_enabled_for_app() {
        let mut config = NotificationsConfig::default();
        config.app_rules.push(AppRule {
            app_name: "silent-app".to_string(),
            desktop_entry: None,
            enabled: true,
            urgency_override: None,
            sound_enabled: false,
            timeout_override: None,
        });

        // Sound disabled for specific app
        assert!(!config.is_sound_enabled_for_app("silent-app", None));

        // Sound enabled by default for apps without rule
        assert!(config.is_sound_enabled_for_app("normal-app", None));
    }

    #[test]
    fn test_app_rule_serialization() {
        let rule = AppRule {
            app_name: "test-app".to_string(),
            desktop_entry: Some("test.desktop".to_string()),
            enabled: false,
            urgency_override: Some(1),
            sound_enabled: false,
            timeout_override: Some(8000),
        };

        let json = serde_json::to_string(&rule).unwrap();

        assert!(json.contains("test-app"));
        assert!(json.contains("test.desktop"));
        assert!(json.contains("\"enabled\":false"));
        assert!(json.contains("\"sound_enabled\":false"));
        assert!(json.contains("\"urgency_override\":1"));
        assert!(json.contains("\"timeout_override\":8000"));
    }

    #[test]
    fn test_app_rule_deserialization_with_defaults() {
        // Minimal JSON with only app_name
        let json = r#"{"app_name":"test-app"}"#;
        let rule: AppRule = serde_json::from_str(json).unwrap();

        assert_eq!(rule.app_name, "test-app");
        assert_eq!(rule.desktop_entry, None);
        assert!(rule.enabled);
        assert!(rule.sound_enabled);
        assert_eq!(rule.urgency_override, None);
        assert_eq!(rule.timeout_override, None);
    }

    #[test]
    fn test_config_with_app_rules() {
        let mut config = NotificationsConfig::default();
        config.app_rules.push(AppRule {
            app_name: "firefox".to_string(),
            desktop_entry: Some("firefox.desktop".to_string()),
            enabled: false,
            urgency_override: Some(2),
            sound_enabled: false,
            timeout_override: Some(10000),
        });

        let json = serde_json::to_string(&config).unwrap();

        // Should contain app_rules array
        assert!(json.contains("app_rules"));
        assert!(json.contains("firefox"));
        assert!(json.contains("firefox.desktop"));
    }

    #[test]
    fn test_config_backward_compatibility_without_app_rules() {
        // Old config without app_rules field (version 2)
        let old_config_json = r#"{
            "do_not_disturb": false,
            "anchor": "Top",
            "max_notifications": 3,
            "max_per_app": 2,
            "max_timeout_urgent": null,
            "max_timeout_normal": 5000,
            "max_timeout_low": 3000,
            "show_images": true,
            "show_actions": true,
            "max_image_size": 128,
            "enable_links": true,
            "enable_animations": true
        }"#;

        let config: NotificationsConfig = serde_json::from_str(old_config_json).unwrap();

        // All existing fields should deserialize
        assert!(!config.do_not_disturb);
        assert_eq!(config.max_notifications, 3);
        assert!(config.show_images);

        // app_rules should default to empty vector
        assert!(config.app_rules.is_empty());
    }

    #[test]
    fn test_config_with_multiple_app_rules() {
        let config_json = r#"{
            "do_not_disturb": false,
            "anchor": "Top",
            "max_notifications": 3,
            "max_per_app": 2,
            "max_timeout_urgent": null,
            "max_timeout_normal": 5000,
            "max_timeout_low": 3000,
            "show_images": true,
            "show_actions": true,
            "max_image_size": 128,
            "enable_links": true,
            "enable_animations": true,
            "app_rules": [
                {
                    "app_name": "firefox",
                    "desktop_entry": "firefox.desktop",
                    "enabled": false,
                    "urgency_override": 2,
                    "sound_enabled": false,
                    "timeout_override": 10000
                },
                {
                    "app_name": "telegram",
                    "enabled": true,
                    "sound_enabled": false
                }
            ]
        }"#;

        let config: NotificationsConfig = serde_json::from_str(config_json).unwrap();

        assert_eq!(config.app_rules.len(), 2);
        assert_eq!(config.app_rules[0].app_name, "firefox");
        assert_eq!(config.app_rules[1].app_name, "telegram");

        // Test rule matching
        let rule = config.find_app_rule("firefox", Some("firefox.desktop"));
        assert!(rule.is_some());
        assert!(!rule.unwrap().enabled);

        let rule = config.find_app_rule("telegram", None);
        assert!(rule.is_some());
        assert!(!rule.unwrap().sound_enabled);
        assert!(rule.unwrap().enabled);
    }

    #[test]
    fn test_urgency_override_values() {
        let mut config = NotificationsConfig::default();

        // Test low urgency override
        config.app_rules.push(AppRule {
            app_name: "low-priority".to_string(),
            desktop_entry: None,
            enabled: true,
            urgency_override: Some(0),
            sound_enabled: true,
            timeout_override: None,
        });

        // Test normal urgency override
        config.app_rules.push(AppRule {
            app_name: "normal-priority".to_string(),
            desktop_entry: None,
            enabled: true,
            urgency_override: Some(1),
            sound_enabled: true,
            timeout_override: None,
        });

        // Test critical urgency override
        config.app_rules.push(AppRule {
            app_name: "critical-priority".to_string(),
            desktop_entry: None,
            enabled: true,
            urgency_override: Some(2),
            sound_enabled: true,
            timeout_override: None,
        });

        let low = config.find_app_rule("low-priority", None);
        assert_eq!(low.unwrap().urgency_override, Some(0));

        let normal = config.find_app_rule("normal-priority", None);
        assert_eq!(normal.unwrap().urgency_override, Some(1));

        let critical = config.find_app_rule("critical-priority", None);
        assert_eq!(critical.unwrap().urgency_override, Some(2));
    }
}
