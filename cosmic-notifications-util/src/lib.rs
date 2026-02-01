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

pub mod action;
pub mod action_parser;
pub mod link;
pub mod link_detector;
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
    collections::HashMap, convert::Infallible, fmt, path::PathBuf, str::FromStr, time::SystemTime,
};

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
                                data: image.data,
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
            }) => Some(icon::from_raster_pixels(*width, *height, data.clone()).icon()),
            None => {
                (!self.app_icon.is_empty()).then(|| icon::from_name(self.app_icon.as_str()).icon())
            }
        }
    }

    pub fn duration_since(&self) -> Option<std::time::Duration> {
        SystemTime::now().duration_since(self.time).ok()
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]

pub enum Image {
    Name(String),
    File(PathBuf),
    /// RGBA
    Data {
        width: u32,
        height: u32,
        data: Vec<u8>,
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
