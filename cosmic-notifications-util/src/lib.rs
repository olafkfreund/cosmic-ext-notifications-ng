#[cfg(feature = "image")]
pub mod image;
#[cfg(feature = "image")]
pub use image::*;

#[cfg(feature = "image")]
pub mod notification_image;
#[cfg(feature = "image")]
pub use notification_image::{NotificationImage, ProcessedImage, MAX_IMAGE_HEIGHT, MAX_IMAGE_WIDTH};

#[cfg(feature = "image")]
pub mod animated_image;
#[cfg(feature = "image")]
pub use animated_image::{AnimatedImage, AnimationFrame, MAX_FRAMES, MAX_ANIMATION_DURATION};

#[cfg(feature = "audio")]
pub mod audio;
#[cfg(feature = "audio")]
pub use audio::{play_sound_file, play_sound_name, AudioError};

pub mod action;
pub mod action_parser;
pub mod link;
pub mod link_detector;
pub mod markup_parser;
pub mod rich_content;
pub mod sanitizer;
pub mod urgency;
pub mod urgency_style;

pub use action::NotificationAction;
pub use action_parser::{
    get_button_actions, get_default_action, has_action_icons, limit_actions, parse_actions,
    parse_actions_from_strs,
};
pub use link::NotificationLink;
pub use link_detector::{detect_links, is_safe_url, open_link};
pub use markup_parser::{parse_markup, segments_to_plain_text, StyledSegment, TextStyle};
pub use rich_content::RichContent;
pub use sanitizer::{extract_hrefs, has_rich_content, sanitize_html, strip_html};
pub use urgency::NotificationUrgency;
pub use urgency_style::{
    categories, category_icon, is_message_category, is_system_category, urgency_color,
    urgency_color_from_u8, urgency_colors, Color,
};

use cosmic::widget::{Icon, icon};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap, convert::Infallible, fmt, path::PathBuf, str::FromStr, sync::Arc, time::SystemTime,
};

#[cfg(feature = "zbus_notifications")]
use cosmic_notifications_config::GroupingMode;

/// A group of related notifications
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationGroup {
    /// The grouping key (app_name or category)
    pub key: String,
    /// Display name for the group
    pub display_name: String,
    /// Notifications in this group (newest first)
    pub notifications: Vec<Notification>,
    /// Whether the group is expanded
    pub expanded: bool,
}

impl NotificationGroup {
    pub fn new(key: String, display_name: String) -> Self {
        Self {
            key,
            display_name,
            notifications: Vec::new(),
            expanded: false,
        }
    }

    pub fn add(&mut self, notification: Notification) {
        self.notifications.insert(0, notification); // Newest first
    }

    pub fn count(&self) -> usize {
        self.notifications.len()
    }

    pub fn newest(&self) -> Option<&Notification> {
        self.notifications.first()
    }

    /// Get the group label with count (e.g., "Firefox (3)")
    pub fn label(&self) -> String {
        if self.notifications.len() > 1 {
            format!("{} ({})", self.display_name, self.notifications.len())
        } else {
            self.display_name.clone()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

impl Notification {
    #[allow(clippy::too_many_arguments)]
    #[cfg(feature = "zbus_notifications")]
    pub fn new(
        app_name: &str,
        id: u32,
        app_icon: &str,
        summary: &str,
        body: &str,
        actions: Vec<&str>,
        hints: HashMap<&str, zbus::zvariant::Value<'_>>,
        expire_timeout: i32,
    ) -> Self {
        let actions = actions
            .chunks_exact(2)
            .map(|a| (a[0].parse().unwrap(), a[1].to_string()))
            .collect();

        let hints = hints
            .into_iter()
            .filter_map(|(k, v)| match k {
                "action-icons" => bool::try_from(v).map(Hint::ActionIcons).ok(),
                "category" => String::try_from(v).map(Hint::Category).ok(),
                "desktop-entry" => String::try_from(v).map(Hint::DesktopEntry).ok(),
                "resident" => bool::try_from(v).map(Hint::Resident).ok(),
                "sound-file" => String::try_from(v)
                    .map(|s| Hint::SoundFile(PathBuf::from(s)))
                    .ok(),
                "sound-name" => String::try_from(v).map(Hint::SoundName).ok(),
                "suppress-sound" => bool::try_from(v).map(Hint::SuppressSound).ok(),
                "transient" => bool::try_from(v).map(Hint::Transient).ok(),
                "sender-pid" => u32::try_from(v).map(Hint::SenderPid).ok(),
                "urgency" => u8::try_from(v).map(Hint::Urgency).ok(),
                "value" => i32::try_from(v).map(Hint::Value).ok(),
                "x" => i32::try_from(v).map(Hint::X).ok(),
                "y" => i32::try_from(v).map(Hint::Y).ok(),
                "image-path" | "image_path" => String::try_from(v).ok().map(|s| {
                    Hint::Image(
                        // First try parsing as file:// URL
                        url::Url::parse(&s)
                            .ok()
                            .and_then(|u| u.to_file_path().ok())
                            .map(Image::File)
                            // Then check if it's an absolute file path
                            .or_else(|| {
                                if s.starts_with('/') {
                                    Some(Image::File(PathBuf::from(&s)))
                                } else {
                                    None
                                }
                            })
                            // Otherwise treat as icon name
                            .unwrap_or_else(|| Image::Name(s)),
                    )
                }),
                "image-data" | "image_data" | "icon_data" => match v {
                    zbus::zvariant::Value::Structure(v) => match ImageData::try_from(v) {
                        Ok(mut image) => Some({
                            image = image.into_rgba();
                            Hint::Image(Image::Data {
                                width: image.width,
                                height: image.height,
                                data: Arc::new(image.data),
                            })
                        }),
                        Err(err) => {
                            tracing::warn!("Invalid image data: {}", err);
                            None
                        }
                    },
                    _ => {
                        tracing::warn!("Invalid value for hint: {}", k);
                        None
                    }
                },
                _ => {
                    tracing::warn!("Unknown hint: {}", k);
                    None
                }
            })
            .collect();

        Notification {
            id,
            app_name: app_name.to_string(),
            app_icon: app_icon.to_string(),
            summary: summary.to_string(),
            body: body.to_string(),
            actions,
            hints,
            expire_timeout,
            time: SystemTime::now(),
        }
    }

    pub fn transient(&self) -> bool {
        self.hints.iter().any(|h| *h == Hint::Transient(true))
    }

    /// Check if action buttons should display icons instead of text labels
    pub fn action_icons(&self) -> bool {
        self.hints.iter().any(|h| *h == Hint::ActionIcons(true))
    }

    /// Check if sound should be suppressed for this notification
    pub fn suppress_sound(&self) -> bool {
        self.hints.iter().any(|h| *h == Hint::SuppressSound(true))
    }

    /// Get the sound file path hint if present
    pub fn sound_file(&self) -> Option<&std::path::Path> {
        self.hints.iter().find_map(|h| match h {
            Hint::SoundFile(path) => Some(path.as_path()),
            _ => None,
        })
    }

    /// Get the sound name hint if present (XDG sound theme name)
    pub fn sound_name(&self) -> Option<&str> {
        self.hints.iter().find_map(|h| match h {
            Hint::SoundName(name) => Some(name.as_str()),
            _ => None,
        })
    }

    /// Play the notification sound if configured
    ///
    /// Respects suppress-sound hint, and plays sound-file or sound-name if specified.
    #[cfg(feature = "audio")]
    pub fn play_sound(&self) {
        // Don't play if sound is suppressed
        if self.suppress_sound() {
            tracing::debug!("Sound suppressed for notification {}", self.id);
            return;
        }

        // Try sound-file first (takes precedence)
        if let Some(path) = self.sound_file() {
            tracing::debug!("Playing sound file: {:?}", path);
            if let Err(e) = crate::audio::play_sound_file(path) {
                tracing::warn!("Failed to play sound file {:?}: {}", path, e);
            }
            return;
        }

        // Try sound-name (XDG sound theme)
        if let Some(name) = self.sound_name() {
            tracing::debug!("Playing sound name: {}", name);
            if let Err(e) = crate::audio::play_sound_name(name) {
                tracing::warn!("Failed to play sound '{}': {}", name, e);
            }
        }
    }

    pub fn category(&self) -> Option<&str> {
        self.hints.iter().find_map(|h| match h {
            Hint::Category(s) => Some(s.as_str()),
            _ => None,
        })
    }

    pub fn desktop_entry(&self) -> Option<&str> {
        self.hints.iter().find_map(|h| match h {
            Hint::DesktopEntry(s) => Some(s.as_str()),
            _ => None,
        })
    }

    pub fn urgency(&self) -> u8 {
        self.hints
            .iter()
            .find_map(|h| match h {
                Hint::Urgency(u) => Some(*u),
                _ => None,
            })
            .unwrap_or(1)
    }

    pub fn image(&self) -> Option<&Image> {
        self.hints.iter().find_map(|h| match h {
            Hint::Image(i) => Some(i),
            _ => None,
        })
    }

    pub fn notification_icon(&self) -> Option<Icon> {
        match self.image() {
            Some(Image::File(path)) => Some(icon::from_path(PathBuf::from(path)).icon()),
            Some(Image::Name(name)) => Some(icon::from_name(name.as_str()).icon()),
            Some(Image::Data {
                width,
                height,
                data,
            }) => Some(icon::from_raster_pixels(*width, *height, (**data).clone()).icon()),
            None => {
                if !self.app_icon.is_empty() {
                    // Handle file:// URLs in app_icon
                    if self.app_icon.starts_with("file://") {
                        if let Ok(url) = url::Url::parse(&self.app_icon) {
                            if let Ok(path) = url.to_file_path() {
                                return Some(icon::from_path(path).icon());
                            }
                        }
                    }
                    // Otherwise treat as icon name
                    Some(icon::from_name(self.app_icon.as_str()).icon())
                } else {
                    None
                }
            }
        }
    }

    pub fn duration_since(&self) -> Option<std::time::Duration> {
        SystemTime::now().duration_since(self.time).ok()
    }

    /// Estimate memory usage of this notification in bytes
    ///
    /// This includes all string data, actions, and hint data (including images).
    /// Used for memory budget tracking of hidden notifications.
    pub fn estimated_size(&self) -> usize {
        let mut size = 0;

        // String data
        size += self.app_name.len();
        size += self.app_icon.len();
        size += self.summary.len();
        size += self.body.len();

        // Actions
        for (action_id, label) in &self.actions {
            size += action_id.to_string().len();
            size += label.len();
        }

        // Hints - calculate actual size per hint (includes image data)
        for hint in &self.hints {
            size += hint.estimated_size();
        }

        // Struct overhead
        size += 200;

        size
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActionId {
    Default,
    Custom(String),
}

impl fmt::Display for ActionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ActionId::Default => write!(f, "default"),
            ActionId::Custom(value) => write!(f, "{}", value),
        }
    }
}

impl FromStr for ActionId {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "default" => ActionId::Default,
            s => ActionId::Custom(s.to_string()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

impl Hint {
    /// Estimate memory usage of this hint in bytes
    pub fn estimated_size(&self) -> usize {
        match self {
            Hint::ActionIcons(_) => 8,
            Hint::Category(s) => s.len() + 8,
            Hint::DesktopEntry(s) => s.len() + 8,
            Hint::Image(img) => match img {
                Image::Name(s) => s.len() + 8,
                Image::File(p) => p.as_os_str().len() + 8,
                Image::Data { data, .. } => data.len() + 32, // Arc overhead is minimal
            },
            Hint::IconData(data) => data.len() + 8,
            Hint::Resident(_) => 8,
            Hint::SenderPid(_) => 8,
            Hint::SoundFile(p) => p.as_os_str().len() + 8,
            Hint::SoundName(s) => s.len() + 8,
            Hint::SuppressSound(_) => 8,
            Hint::Transient(_) => 8,
            Hint::Urgency(_) => 8,
            Hint::Value(_) => 8,
            Hint::X(_) => 8,
            Hint::Y(_) => 8,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]

pub enum Image {
    Name(String),
    File(PathBuf),
    /// RGBA
    Data {
        width: u32,
        height: u32,
        data: Arc<Vec<u8>>,
    },
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CloseReason {
    Expired = 1,
    Dismissed = 2,
    CloseNotification = 3,
    Undefined = 4,
}

pub const PANEL_NOTIFICATIONS_FD: &str = "PANEL_NOTIFICATIONS_FD";
pub const DAEMON_NOTIFICATIONS_FD: &str = "DAEMON_NOTIFICATIONS_FD";

/// Group notifications according to the specified mode
#[cfg(feature = "zbus_notifications")]
pub fn group_notifications(
    notifications: &[Notification],
    mode: GroupingMode,
) -> Vec<NotificationGroup> {
    use std::collections::HashMap;

    match mode {
        GroupingMode::None => {
            // Each notification is its own "group"
            notifications.iter().map(|n| {
                let mut group = NotificationGroup::new(
                    n.id.to_string(),
                    n.app_name.clone(),
                );
                group.add(n.clone());
                group
            }).collect()
        }
        GroupingMode::ByApp => {
            let mut groups: HashMap<String, NotificationGroup> = HashMap::new();
            for notification in notifications {
                let key = notification.app_name.clone();
                groups.entry(key.clone())
                    .or_insert_with(|| NotificationGroup::new(key.clone(), key))
                    .add(notification.clone());
            }
            groups.into_values().collect()
        }
        GroupingMode::ByCategory => {
            let mut groups: HashMap<String, NotificationGroup> = HashMap::new();
            for notification in notifications {
                let category = notification.category().unwrap_or("uncategorized");
                // Normalize category to base type for grouping
                let (key, display) = match category {
                    cat if cat.starts_with("email") => ("email".to_string(), "Email".to_string()),
                    cat if cat.starts_with("im") => ("im".to_string(), "Messages".to_string()),
                    cat if cat.starts_with("network") => ("network".to_string(), "Network".to_string()),
                    cat if cat.starts_with("device") => ("device".to_string(), "Devices".to_string()),
                    _ => (category.to_string(), category.to_string()),
                };
                groups.entry(key.clone())
                    .or_insert_with(|| NotificationGroup::new(key, display))
                    .add(notification.clone());
            }
            groups.into_values().collect()
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_full_notification_flow_with_links() {
        // Test: parse notification → sanitize HTML → detect links
        let body = "Check out https://github.com/pop-os/cosmic for more info!";

        // Sanitize HTML (should pass through plain text)
        let sanitized = sanitize_html(body);
        assert_eq!(sanitized, body);

        // Detect links
        let links = detect_links(&sanitized);
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].url, "https://github.com/pop-os/cosmic");

        // Verify URL is safe
        assert!(is_safe_url(&links[0].url));
    }

    #[test]
    fn test_full_notification_flow_with_html() {
        // Test: HTML with links → sanitize → detect links
        let body = r#"<b>Important:</b> Visit <a href="https://example.com">our site</a>"#;

        // Sanitize HTML
        let sanitized = sanitize_html(body);

        // Should contain the link but strip the <a> tag (ammonia handles this)
        assert!(sanitized.contains("https://example.com") || sanitized.contains("our site"));

        // Has rich content
        assert!(has_rich_content(body));
    }

    #[test]
    fn test_action_parsing_integration() {
        // Test: parse actions from strings → get button actions → limit actions
        let action_strs = vec!["default", "Open", "action1", "Reply", "action2", "Delete"];

        let actions = parse_actions_from_strs(&action_strs);
        assert_eq!(actions.len(), 3);

        // Get button actions (exclude default)
        let button_actions = get_button_actions(&actions);
        assert_eq!(button_actions.len(), 2);
        assert_eq!(button_actions[0].label, "Reply");
        assert_eq!(button_actions[1].label, "Delete");

        // Get default action
        let default_action = get_default_action(&actions);
        assert!(default_action.is_some());
        assert_eq!(default_action.unwrap().label, "Open");
    }

    #[test]
    fn test_urgency_color_integration() {
        // Test: parse urgency → get color → apply styling
        let urgency_low = NotificationUrgency::from(0u8);
        let urgency_normal = NotificationUrgency::from(1u8);
        let urgency_critical = NotificationUrgency::from(2u8);

        assert_eq!(urgency_low, NotificationUrgency::Low);
        assert_eq!(urgency_normal, NotificationUrgency::Normal);
        assert_eq!(urgency_critical, NotificationUrgency::Critical);

        // Get colors
        let color_low = urgency_color(urgency_low);
        let color_critical = urgency_color(urgency_critical);

        // Critical should be different from low
        assert_ne!(color_low, color_critical);
    }

    #[test]
    fn test_category_detection() {
        // Test: category hint → icon mapping → styling
        assert!(is_message_category("im.received"));
        assert!(is_message_category("email.arrived"));
        assert!(is_system_category("device.added"));
        assert!(is_system_category("network.connected"));

        // Get category icon
        let icon = category_icon("email.arrived");
        assert!(icon.is_some());
    }

    #[test]
    fn test_sanitizer_and_link_detection_together() {
        // Test: malicious HTML → sanitize → safe link detection
        let malicious = r#"Click <a href="javascript:alert('xss')">here</a> or visit https://safe-site.com"#;

        // Sanitize
        let sanitized = sanitize_html(malicious);

        // Should not contain javascript:
        assert!(!sanitized.contains("javascript:"));

        // Detect links in sanitized content
        let links = detect_links(&sanitized);

        // Should find the safe link
        let safe_links: Vec<_> = links.iter().filter(|l| is_safe_url(&l.url)).collect();
        assert!(!safe_links.is_empty());

        // Should not include javascript URLs
        let js_links: Vec<_> = links.iter().filter(|l| l.url.starts_with("javascript:")).collect();
        assert!(js_links.is_empty());
    }

    #[test]
    fn test_action_limiting() {
        // Test: many actions → limit to max → verify count
        let many_actions: Vec<String> = (0..10)
            .flat_map(|i| vec![format!("action{}", i), format!("Button {}", i)])
            .collect();

        let actions = parse_actions_from_strs(
            &many_actions.iter().map(|s| s.as_str()).collect::<Vec<_>>()
        );

        // Limit to 3 actions
        let limited = limit_actions(&actions, 3);
        assert!(limited.len() <= 3);
    }

    #[test]
    fn test_backward_compatibility_basic_notification() {
        // Test: basic Notification struct without rich content still works
        let notification = Notification {
            id: 1,
            app_name: "TestApp".to_string(),
            app_icon: "dialog-information".to_string(),
            summary: "Test".to_string(),
            body: "Simple notification".to_string(),
            actions: vec![],
            hints: vec![],
            expire_timeout: 5000,
            time: SystemTime::now(),
        };

        // Should work with basic methods
        assert_eq!(notification.urgency(), 1); // Default normal
        assert!(notification.image().is_none());
        assert!(notification.category().is_none());
        assert!(!notification.transient());
    }
}

#[cfg(all(test, feature = "zbus_notifications"))]
mod grouping_tests {
    use super::*;

    fn create_test_notification(id: u32, app_name: &str, category: Option<&str>) -> Notification {
        let mut hints = vec![];
        if let Some(cat) = category {
            hints.push(Hint::Category(cat.to_string()));
        }

        Notification {
            id,
            app_name: app_name.to_string(),
            app_icon: "test-icon".to_string(),
            summary: format!("Test {}", id),
            body: "Test body".to_string(),
            actions: vec![],
            hints,
            expire_timeout: 5000,
            time: SystemTime::now(),
        }
    }

    #[test]
    fn test_notification_group_new() {
        let group = NotificationGroup::new("key1".to_string(), "Display Name".to_string());

        assert_eq!(group.key, "key1");
        assert_eq!(group.display_name, "Display Name");
        assert_eq!(group.notifications.len(), 0);
        assert!(!group.expanded);
    }

    #[test]
    fn test_notification_group_add() {
        let mut group = NotificationGroup::new("firefox".to_string(), "Firefox".to_string());
        let notif1 = create_test_notification(1, "Firefox", None);
        let notif2 = create_test_notification(2, "Firefox", None);

        group.add(notif1.clone());
        assert_eq!(group.count(), 1);

        group.add(notif2.clone());
        assert_eq!(group.count(), 2);

        // Newest first
        assert_eq!(group.newest().unwrap().id, 2);
    }

    #[test]
    fn test_notification_group_label() {
        let mut group = NotificationGroup::new("firefox".to_string(), "Firefox".to_string());

        // Single notification - no count
        group.add(create_test_notification(1, "Firefox", None));
        assert_eq!(group.label(), "Firefox");

        // Multiple notifications - show count
        group.add(create_test_notification(2, "Firefox", None));
        assert_eq!(group.label(), "Firefox (2)");

        group.add(create_test_notification(3, "Firefox", None));
        assert_eq!(group.label(), "Firefox (3)");
    }

    #[test]
    fn test_grouping_mode_none() {
        let notifications = vec![
            create_test_notification(1, "Firefox", None),
            create_test_notification(2, "Chrome", None),
            create_test_notification(3, "Firefox", None),
        ];

        let groups = group_notifications(&notifications, GroupingMode::None);

        // Each notification is its own group
        assert_eq!(groups.len(), 3);
        assert_eq!(groups[0].count(), 1);
        assert_eq!(groups[1].count(), 1);
        assert_eq!(groups[2].count(), 1);
    }

    #[test]
    fn test_grouping_by_app() {
        let notifications = vec![
            create_test_notification(1, "Firefox", None),
            create_test_notification(2, "Chrome", None),
            create_test_notification(3, "Firefox", None),
            create_test_notification(4, "Chrome", None),
            create_test_notification(5, "Firefox", None),
        ];

        let groups = group_notifications(&notifications, GroupingMode::ByApp);

        // Should have 2 groups: Firefox and Chrome
        assert_eq!(groups.len(), 2);

        // Find Firefox group
        let firefox_group = groups.iter().find(|g| g.key == "Firefox");
        assert!(firefox_group.is_some());
        assert_eq!(firefox_group.unwrap().count(), 3);

        // Find Chrome group
        let chrome_group = groups.iter().find(|g| g.key == "Chrome");
        assert!(chrome_group.is_some());
        assert_eq!(chrome_group.unwrap().count(), 2);
    }

    #[test]
    fn test_grouping_by_category() {
        let notifications = vec![
            create_test_notification(1, "Thunderbird", Some("email.arrived")),
            create_test_notification(2, "Gmail", Some("email")),
            create_test_notification(3, "Telegram", Some("im.received")),
            create_test_notification(4, "Signal", Some("im")),
            create_test_notification(5, "NetworkManager", Some("network.connected")),
            create_test_notification(6, "Unknown", None), // Uncategorized
        ];

        let groups = group_notifications(&notifications, GroupingMode::ByCategory);

        // Should have groups: Email, Messages, Network, uncategorized
        assert!(groups.len() >= 4);

        // Find Email group
        let email_group = groups.iter().find(|g| g.key.contains("email"));
        assert!(email_group.is_some());
        let email_count = email_group.unwrap().count();
        assert!(email_count >= 2); // At least Thunderbird and Gmail

        // Find Messages group
        let im_group = groups.iter().find(|g| g.key.contains("im"));
        assert!(im_group.is_some());
        let im_count = im_group.unwrap().count();
        assert!(im_count >= 2); // At least Telegram and Signal

        // Find uncategorized group
        let uncat_group = groups.iter().find(|g| g.key == "uncategorized");
        assert!(uncat_group.is_some());
        assert_eq!(uncat_group.unwrap().count(), 1);
    }

    #[test]
    fn test_grouping_category_display_names() {
        let notifications = vec![
            create_test_notification(1, "App1", Some("email.arrived")),
            create_test_notification(2, "App2", Some("im.received")),
            create_test_notification(3, "App3", Some("network.connected")),
            create_test_notification(4, "App4", Some("device.added")),
        ];

        let groups = group_notifications(&notifications, GroupingMode::ByCategory);

        // Check display names are user-friendly
        let email_group = groups.iter().find(|g| g.key.contains("email"));
        if let Some(group) = email_group {
            assert_eq!(group.display_name, "Email");
        }

        let im_group = groups.iter().find(|g| g.key.contains("im"));
        if let Some(group) = im_group {
            assert_eq!(group.display_name, "Messages");
        }

        let network_group = groups.iter().find(|g| g.key.contains("network"));
        if let Some(group) = network_group {
            assert_eq!(group.display_name, "Network");
        }

        let device_group = groups.iter().find(|g| g.key.contains("device"));
        if let Some(group) = device_group {
            assert_eq!(group.display_name, "Devices");
        }
    }

    #[test]
    fn test_grouping_preserves_notification_order() {
        let mut notifications = vec![];
        for i in 1..=5 {
            notifications.push(create_test_notification(i, "Firefox", None));
        }

        let groups = group_notifications(&notifications, GroupingMode::ByApp);
        assert_eq!(groups.len(), 1);

        let group = &groups[0];
        assert_eq!(group.count(), 5);

        // Should have notifications in reverse order (newest first)
        assert_eq!(group.notifications[0].id, 5);
        assert_eq!(group.notifications[1].id, 4);
        assert_eq!(group.notifications[2].id, 3);
        assert_eq!(group.notifications[3].id, 2);
        assert_eq!(group.notifications[4].id, 1);
    }

    #[test]
    fn test_empty_notifications_list() {
        let notifications: Vec<Notification> = vec![];

        let groups_none = group_notifications(&notifications, GroupingMode::None);
        assert_eq!(groups_none.len(), 0);

        let groups_app = group_notifications(&notifications, GroupingMode::ByApp);
        assert_eq!(groups_app.len(), 0);

        let groups_cat = group_notifications(&notifications, GroupingMode::ByCategory);
        assert_eq!(groups_cat.len(), 0);
    }

    #[test]
    fn test_single_notification() {
        let notifications = vec![create_test_notification(1, "Firefox", Some("email"))];

        let groups_none = group_notifications(&notifications, GroupingMode::None);
        assert_eq!(groups_none.len(), 1);
        assert_eq!(groups_none[0].count(), 1);

        let groups_app = group_notifications(&notifications, GroupingMode::ByApp);
        assert_eq!(groups_app.len(), 1);
        assert_eq!(groups_app[0].count(), 1);

        let groups_cat = group_notifications(&notifications, GroupingMode::ByCategory);
        assert_eq!(groups_cat.len(), 1);
        assert_eq!(groups_cat[0].count(), 1);
    }
}
