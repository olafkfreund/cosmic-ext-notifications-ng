#![allow(dead_code)]

use cosmic::iced::{Alignment, Length};
use cosmic::iced_widget::{column, row};
use cosmic::widget::{button, container, icon, text};
use cosmic::Element;
use cosmic_ext_notifications_config;

/// Configuration for the rich notification card
#[derive(Debug, Clone)]
pub struct RichCardConfig {
    /// Width of the card in pixels
    pub width: f32,
    /// Whether to show the progress bar area
    pub show_progress: bool,
    /// Whether to show the actions area
    pub show_actions: bool,
    /// Whether to display images in notifications
    pub show_images: bool,
    /// Maximum width/height for notification images in pixels (clamped to 32-256)
    pub max_image_size: u32,
    /// Whether links in notification body are clickable
    pub enable_links: bool,
    /// Whether animated images and card animations are enabled
    pub enable_animations: bool,
}

impl Default for RichCardConfig {
    fn default() -> Self {
        Self {
            width: 380.0,
            show_progress: false,
            show_actions: false,
            show_images: true,
            max_image_size: 128,
            enable_links: true,
            enable_animations: true,
        }
    }
}

impl RichCardConfig {
    /// Create a RichCardConfig from NotificationsConfig
    pub fn from_notifications_config(config: &cosmic_ext_notifications_config::NotificationsConfig) -> Self {
        Self {
            width: 380.0,
            show_progress: false,
            show_actions: config.show_actions,
            show_images: config.show_images,
            // Clamp max_image_size to valid range (32-256)
            max_image_size: config.max_image_size.clamp(32, 256),
            enable_links: config.enable_links,
            enable_animations: config.enable_animations,
        }
    }
}

/// Rich notification card data
#[derive(Debug, Clone)]
pub struct RichCardData {
    /// Application name
    pub app_name: String,
    /// Notification summary (title)
    pub summary: String,
    /// Notification body text
    pub body: String,
    /// Optional timestamp text (e.g., "2m ago")
    pub timestamp: Option<String>,
    /// Optional progress percentage (0-100)
    pub progress: Option<u8>,
}

/// Creates a rich notification card widget
///
/// This card has the following structure:
/// ```text
/// ┌─────────────────────────────────────────┐
/// │ [AppIcon] App Name              [X] time│ Header
/// ├─────────────────────────────────────────┤
/// │ ┌───────┐                               │
/// │ │ Image │  Summary (bold)               │ Body
/// │ │ 64x64 │  Body text...                 │
/// │ └───────┘                               │
/// ├─────────────────────────────────────────┤
/// │ ████████████████░░░░░░░░  75%           │ Progress (optional)
/// ├─────────────────────────────────────────┤
/// │ [Reply] [Mark Read] [Open]              │ Actions (optional)
/// └─────────────────────────────────────────┘
/// ```
pub fn rich_card<'a, Message: 'static + Clone>(
    data: &'a RichCardData,
    config: &RichCardConfig,
    on_close: Message,
) -> Element<'a, Message> {
    // Header section: App icon, app name, close button, timestamp
    let header = create_header(&data.app_name, data.timestamp.as_deref(), on_close);

    // Body section: Image placeholder and text content
    let body_section = create_body(&data.summary, &data.body);

    // Build card sections
    let mut card_content = column![header, body_section].spacing(8);

    // Optional progress section
    if config.show_progress {
        let progress_section = create_progress_placeholder(data.progress);
        card_content = card_content.push(progress_section);
    }

    // Optional actions section
    if config.show_actions {
        let actions_section = create_actions_placeholder();
        card_content = card_content.push(actions_section);
    }

    // Wrap in container with styling
    container(card_content)
        .padding(12)
        .width(Length::Fixed(config.width))
        .into()
}

/// Creates the header section with app icon, name, close button, and timestamp
fn create_header<'a, Message: 'static + Clone>(
    app_name: &'a str,
    timestamp: Option<&'a str>,
    on_close: Message,
) -> Element<'a, Message> {
    // App icon placeholder (using a generic icon)
    let app_icon = icon::from_name("application-x-executable-symbolic")
        .size(16)
        .symbolic(true);

    // App name text
    let app_name_text = text::caption(app_name).width(Length::Fill);

    // Timestamp text (if provided)
    let time_elem = if let Some(time) = timestamp {
        text::caption(time)
    } else {
        text::caption("")
    };

    // Close button
    let close_button = button::custom(
        icon::from_name("window-close-symbolic")
            .size(16)
            .symbolic(true),
    )
    .on_press(on_close)
    .class(cosmic::theme::Button::Text);

    row![app_icon, app_name_text, time_elem, close_button]
        .spacing(8)
        .align_y(Alignment::Center)
        .into()
}

/// Creates the body section with image placeholder and notification content
fn create_body<'a, Message: 'static>(summary: &'a str, body: &'a str) -> Element<'a, Message> {
    // Image placeholder (64x64)
    let image_placeholder = container(
        text::caption("IMG")
            .width(Length::Fixed(64.0))
            .height(Length::Fixed(64.0)),
    )
    .width(Length::Fixed(64.0))
    .height(Length::Fixed(64.0))
    .center_x(Length::Fixed(64.0))
    .center_y(Length::Fixed(64.0));

    // Text content (summary + body)
    let text_content = column![
        text::body(summary).width(Length::Fill),
        text::caption(body).width(Length::Fill)
    ]
    .spacing(4);

    row![image_placeholder, text_content]
        .spacing(12)
        .align_y(Alignment::Start)
        .into()
}

/// Creates a placeholder for the progress bar section
fn create_progress_placeholder<'a, Message: 'static>(progress: Option<u8>) -> Element<'a, Message> {
    let progress_text = if let Some(pct) = progress {
        format!("Progress: {}%", pct.min(100))
    } else {
        "Progress: --".to_string()
    };

    // Simple text placeholder for now - actual progress bar will be implemented in another task
    container(text::caption(progress_text))
        .padding(4)
        .width(Length::Fill)
        .into()
}

/// Creates a placeholder for the actions section
fn create_actions_placeholder<'a, Message: 'static>() -> Element<'a, Message> {
    // Placeholder for action buttons - will be implemented in another task
    let action_text = text::caption("Actions: [Reply] [Mark Read] [Open]");

    container(action_text)
        .padding(4)
        .width(Length::Fill)
        .into()
}
