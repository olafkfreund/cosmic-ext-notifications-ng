// Performance and Animation Guidelines
// =====================================
//
// This application uses cosmic_time::Timeline for smooth animations with the
// following performance targets and considerations:
//
// ## Performance Targets
// - Minimum framerate: 30 FPS (33ms per frame)
// - Target framerate: 60 FPS (16ms per frame) for smooth animations
// - Animation durations: 200-400ms for snappy feel
//
// ## Animation System
// - Card entry/exit: Handled by anim! macro, adapts to card height
// - Image fade-in: 200ms linear opacity transition
// - Progress bars: 300ms smooth value interpolation
//
// ## Memory Considerations
// - Maximum concurrent image notifications: Recommend limiting to 10-15
//   to avoid excessive memory usage with large images
// - Each rich notification with image can use 100-500KB depending on image size
// - Progress animation state: ~24 bytes per notification (negligible)
// - Image fade state: ~16 bytes per notification (negligible)
//
// ## Performance Notes
// - All animations use lightweight linear interpolation
// - No blocking operations on UI thread
// - Image decoding happens in subscription, not in render path
// - Timeline updates are batched via Frame subscription
// - Card list animations are handled efficiently by cosmic_time::anim! macro

use crate::subscriptions::notifications;
use crate::widgets::{notification_image, ImageSize, notification_progress, should_show_progress, RichCardConfig};
use cosmic::app::{Core, Settings};
use cosmic::cosmic_config::{Config, CosmicConfigEntry};
use cosmic::iced::platform_specific::runtime::wayland::layer_surface::{
    IcedMargin, IcedOutput, SctkLayerSurfaceSettings,
};
use cosmic::iced::platform_specific::shell::wayland::commands::{
    activation,
    layer_surface::{Anchor, KeyboardInteractivity, destroy_layer_surface, get_layer_surface},
};
use cosmic::iced::{self, Length, Limits, Subscription};
use cosmic::iced_runtime::core::window::Id as SurfaceId;
use cosmic::iced_widget::{column, row, vertical_space};
use cosmic::surface;
use cosmic::widget::{autosize, button, container, icon, text};
use cosmic::{Application, Element, app::Task};
use cosmic_notifications_config::NotificationsConfig;
use cosmic_notifications_util::{
    ActionId, CloseReason, Hint, Image, Notification, NotificationImage, NotificationLink,
    parse_markup, ProcessedImage, detect_links, extract_hrefs, sanitize_html, strip_html,
};
use cosmic_panel_config::{CosmicPanelConfig, CosmicPanelOuput, PanelAnchor};
use cosmic_time::{Instant, Timeline, anim, id};
use iced::Alignment;
use std::borrow::Cow;
use std::collections::VecDeque;
use std::time::Duration;
use tokio::sync::mpsc;

static NOTIFICATIONS_APPLET: &str = "com.system76.CosmicAppletNotifications";

pub fn run() -> cosmic::iced::Result {
    cosmic::app::run::<CosmicNotifications>(
        Settings::default()
            .antialiasing(true)
            .client_decorations(true)
            .debug(false)
            .default_text_size(16.0)
            .scale_factor(1.0)
            .no_main_window(true)
            .exit_on_close(false),
        (),
    )?;
    Ok(())
}

struct CosmicNotifications {
    core: Core,
    active_surface: bool,
    autosize_id: iced::id::Id,
    window_id: SurfaceId,
    cards: Vec<Notification>,
    hidden: VecDeque<Notification>,
    notifications_id: id::Cards,
    notifications_tx: Option<mpsc::Sender<notifications::Input>>,
    config: NotificationsConfig,
    dock_config: CosmicPanelConfig,
    panel_config: CosmicPanelConfig,
    anchor: Option<(Anchor, Option<String>)>,
    timeline: Timeline,
}

#[derive(Debug, Clone)]
enum Message {
    ActivateNotification(u32),
    ActivationToken(Option<String>, u32, Option<ActionId>),
    Dismissed(u32),
    Notification(notifications::Event),
    Timeout(u32),
    Config(NotificationsConfig),
    PanelConfig(CosmicPanelConfig),
    DockConfig(CosmicPanelConfig),
    Frame(Instant),
    Ignore,
    Surface(surface::Action),
    /// Link clicked in notification body
    LinkClicked(String),
    /// Action button clicked (notification_id, action_id)
    ActionClicked(u32, String),
}

impl CosmicNotifications {
    /// Render a notification using rich notification widgets
    ///
    /// This creates a rich notification card with:
    /// - App icon and name in the header
    /// - Optional notification image (thumbnail or icon)
    /// - Summary and body text with clickable links
    /// - Progress bar if present in hints
    /// - Action buttons if present
    fn render_rich_notification(&self, n: &Notification, config: &RichCardConfig) -> Element<'static, Message> {
        // Header: App icon, app name, close button
        let app_name_text = text::caption(if n.app_name.len() > 24 {
            Cow::from(format!("{:.26}...", n.app_name.lines().next().unwrap_or_default()))
        } else {
            Cow::from(n.app_name.clone())
        })
        .width(Length::Fill);

        // App icon from notification
        let app_icon_elem: Element<'static, Message> = if let Some(icon_widget) = n.notification_icon() {
            icon_widget.size(16).into()
        } else if !n.app_icon.is_empty() {
            icon::from_name(n.app_icon.as_str()).size(16).symbolic(true).into()
        } else {
            icon::from_name("application-x-executable-symbolic").size(16).symbolic(true).into()
        };

        let close_button = button::custom(
            icon::from_name("window-close-symbolic")
                .size(16)
                .symbolic(true),
        )
        .on_press(Message::Dismissed(n.id))
        .class(cosmic::theme::Button::Text);

        // Optional timestamp
        let timestamp: Element<'static, Message> = if let Some(duration) = n.duration_since() {
            let secs = duration.as_secs();
            let time_text = if secs < 60 {
                "now".to_string()
            } else if secs < 3600 {
                format!("{}m", secs / 60)
            } else if secs < 86400 {
                format!("{}h", secs / 3600)
            } else {
                format!("{}d", secs / 86400)
            };
            text::caption(time_text).into()
        } else {
            cosmic::widget::Space::new(0, 0).into()
        };

        let header = row![app_icon_elem, app_name_text, timestamp, close_button]
            .spacing(8)
            .align_y(Alignment::Center);

        // Body section: Image + text content
        let mut body_elements: Vec<Element<'static, Message>> = Vec::new();

        // Add notification image if present and enabled
        // Check hints first, then fall back to app_icon
        // Use larger size (96x96) to better match text content
        if config.show_images {
            if let Some(image) = n.image() {
                // Image from hints (image-data, image-path) - use Expanded size (128x128)
                if let Some(img_elem) = self.render_notification_image(image) {
                    body_elements.push(img_elem);
                }
            } else if !n.app_icon.is_empty() {
                // Fallback to app_icon (from notify-send -i or app_icon parameter)
                // Use larger 96x96 size to match text height
                let icon_elem: Element<'static, Message> = container(
                    icon::from_name(n.app_icon.as_str()).size(96).icon()
                )
                .width(Length::Fixed(96.0))
                .height(Length::Fixed(96.0))
                .into();
                body_elements.push(icon_elem);
            }
        }

        // Text content: summary and body (owned strings for 'static lifetime)
        let summary_text: String = n.summary.lines().next().unwrap_or_default().to_string();
        let body_text = n.body.clone();

        // Debug: Log raw body for troubleshooting Chrome notifications
        tracing::debug!("Notification body (raw): {:?}", body_text);

        // Extract URLs from href attributes in HTML anchor tags first
        let extracted = extract_hrefs(&body_text);
        tracing::debug!("Extracted hrefs: {:?}", extracted);

        let href_links: Vec<NotificationLink> = extracted
            .into_iter()
            .map(|(url, _text)| NotificationLink {
                url,
                title: None,
                start: 0,
                length: 0,
            })
            .collect();

        // Check if body contains HTML markup for styled rendering
        let has_markup = cosmic_notifications_util::has_rich_content(&body_text);

        // Strip HTML for link detection and plain text fallback
        let display_body_str = strip_html(&sanitize_html(&body_text));

        // Detect plain text URLs in the stripped body
        let plain_links = detect_links(&display_body_str);

        // Combine href-extracted links with plain text links, preferring href links
        let links: Vec<NotificationLink> = if !href_links.is_empty() {
            href_links
        } else {
            plain_links
        };

        // Create body text - use markup rendering if HTML is present, otherwise plain text
        let body_element: Element<'static, Message> = if has_markup {
            // Render with HTML markup styling (body-markup capability)
            let markup_body = self.render_markup_body(&body_text);
            if config.enable_links && !links.is_empty() {
                // Add link buttons below styled body
                self.render_body_with_links(&display_body_str, &links)
            } else {
                markup_body
            }
        } else if config.enable_links && !links.is_empty() {
            // Build text with clickable link segments
            self.render_body_with_links(&display_body_str, &links)
        } else {
            // Show first line only when no links and no markup
            let body_display = display_body_str.lines().next().unwrap_or_default().to_string();
            text::caption(body_display).width(Length::Fill).into()
        };

        let body_content: Element<'static, Message> = column![
            text::body(summary_text).width(Length::Fill),
            body_element
        ]
        .spacing(4)
        .into();

        // Build body row with image (if any) + text
        let body_section: Element<'static, Message> = if body_elements.is_empty() {
            body_content
        } else {
            match body_elements.pop() {
                Some(img) => row![img, body_content]
                    .spacing(12)
                    .align_y(Alignment::Start)
                    .into(),
                None => {
                    tracing::warn!("Expected image element but vector was empty");
                    body_content
                }
            }
        };

        // Build card content
        let mut card_content = column![header, body_section].spacing(8);

        // Optional progress bar
        if let Some(progress_value) = self.get_progress_from_hints(n) {
            let progress_bar = notification_progress(progress_value, true);
            card_content = card_content.push(progress_bar);
        }

        // Optional action buttons - inline creation for 'static lifetime
        if config.show_actions && !n.actions.is_empty() {
            // Filter to non-default actions and take up to 3
            let visible_actions: Vec<_> = n.actions
                .iter()
                .filter(|(id, _)| !matches!(id, ActionId::Default))
                .take(3)
                .collect();

            if !visible_actions.is_empty() {
                let notification_id = n.id;

                // Build action buttons inline to avoid lifetime issues
                let mut action_elements: Vec<Element<'static, Message>> = Vec::with_capacity(visible_actions.len());

                let use_icons = n.action_icons();
                for (action_id, label) in visible_actions {
                    let action_id_str = action_id.to_string();
                    let label_str = label.clone();

                    let btn: Element<'static, Message> = if use_icons {
                        // When action-icons hint is true, interpret action ID as icon name
                        // Common icon names: "media-playback-start", "media-playback-pause", etc.
                        let icon_name = action_id_str.clone();
                        button::icon(icon::from_name(icon_name).size(16).symbolic(true))
                            .on_press(Message::ActionClicked(notification_id, action_id_str))
                            .padding([6, 12])
                            .into()
                    } else {
                        button::text(label_str)
                            .on_press(Message::ActionClicked(notification_id, action_id_str))
                            .padding([6, 12])
                            .into()
                    };
                    action_elements.push(btn);
                }

                // Build the row based on number of buttons
                let action_row: Element<'static, Message> = match action_elements.len() {
                    0 => cosmic::widget::Space::new(0, 0).into(),
                    1 => {
                        let mut iter = action_elements.into_iter();
                        match iter.next() {
                            Some(btn) => btn,
                            None => {
                                tracing::warn!("Expected 1 action button but iterator was empty");
                                cosmic::widget::Space::new(0, 0).into()
                            }
                        }
                    }
                    2 => {
                        let mut iter = action_elements.into_iter();
                        match (iter.next(), iter.next()) {
                            (Some(btn1), Some(btn2)) => row![btn1, btn2]
                                .spacing(8)
                                .align_y(Alignment::Center)
                                .into(),
                            _ => {
                                tracing::warn!("Expected 2 action buttons but not all were available");
                                cosmic::widget::Space::new(0, 0).into()
                            }
                        }
                    }
                    _ => {
                        let mut iter = action_elements.into_iter();
                        match (iter.next(), iter.next(), iter.next()) {
                            (Some(btn1), Some(btn2), Some(btn3)) => row![btn1, btn2, btn3]
                                .spacing(8)
                                .align_y(Alignment::Center)
                                .into(),
                            _ => {
                                tracing::warn!("Expected 3 action buttons but not all were available");
                                cosmic::widget::Space::new(0, 0).into()
                            }
                        }
                    }
                };
                card_content = card_content.push(action_row);
            }
        }

        // Wrap in container with padding
        container(card_content)
            .padding(12)
            .width(Length::Fill)
            .into()
    }

    /// Render notification image from Image hint
    /// Uses Expanded size (128x128) for better visibility with text content
    fn render_notification_image(&self, image: &Image) -> Option<Element<'static, Message>> {
        match image {
            Image::Data { width, height, data } => {
                // Create ProcessedImage from raw data
                let processed = ProcessedImage {
                    data: data.clone(),
                    width: *width,
                    height: *height,
                };
                Some(notification_image(&processed, ImageSize::Expanded))
            }
            Image::File(path) => {
                // Try to load image from file
                match NotificationImage::from_path(path.to_str().unwrap_or_default()) {
                    Ok(processed) => Some(notification_image(&processed, ImageSize::Expanded)),
                    Err(e) => {
                        tracing::warn!("Failed to load notification image from {}: {}", path.display(), e);
                        None
                    }
                }
            }
            Image::Name(name) => {
                // Use icon from name - 96x96 to match text height
                Some(
                    container(icon::from_name(name.as_str()).size(96).icon())
                        .width(Length::Fixed(96.0))
                        .height(Length::Fixed(96.0))
                        .into()
                )
            }
        }
    }

    /// Extract progress value from notification hints
    fn get_progress_from_hints(&self, n: &Notification) -> Option<f32> {
        for hint in &n.hints {
            if let Hint::Value(value) = hint {
                // Value hint is typically 0-100, convert to 0.0-1.0
                let progress = (*value as f32).clamp(0.0, 100.0) / 100.0;
                if should_show_progress(Some(progress)) {
                    return Some(progress);
                }
            }
        }
        None
    }

    /// Render body text with clickable link segments
    ///
    /// For simplicity, renders the full body text followed by clickable link buttons.
    /// This avoids complex text segmentation while still making links clickable.
    fn render_body_with_links(
        &self,
        body: &str,
        links: &[cosmic_notifications_util::NotificationLink],
    ) -> Element<'static, Message> {
        // Show the full body text
        let body_text: Element<'static, Message> = text::caption(body.to_string())
            .width(Length::Fill)
            .into();

        // If only one link, show body + single link button
        if links.len() == 1 {
            let link = &links[0];
            let url = link.url.clone();
            let display_url = if url.len() > 40 {
                format!("{}...", &url[..37])
            } else {
                url.clone()
            };

            let link_button: Element<'static, Message> = button::text(format!("🔗 {}", display_url))
                .on_press(Message::LinkClicked(url))
                .class(cosmic::theme::Button::Link)
                .padding([2, 4])
                .into();

            return column![body_text, link_button]
                .spacing(4)
                .width(Length::Fill)
                .into();
        }

        // Multiple links - show body + row of link buttons
        let mut link_elements: Vec<Element<'static, Message>> = Vec::with_capacity(links.len().min(3));

        for link in links.iter().take(3) {
            let url = link.url.clone();
            let display_url = if url.len() > 30 {
                format!("{}...", &url[..27])
            } else {
                url.clone()
            };

            let link_button: Element<'static, Message> = button::text(format!("🔗 {}", display_url))
                .on_press(Message::LinkClicked(url))
                .class(cosmic::theme::Button::Link)
                .padding([2, 4])
                .into();

            link_elements.push(link_button);
        }

        // Build row of link buttons
        let links_row: Element<'static, Message> = match link_elements.len() {
            1 => {
                let mut iter = link_elements.into_iter();
                match iter.next() {
                    Some(btn) => btn,
                    None => {
                        tracing::warn!("Expected 1 link button but iterator was empty");
                        cosmic::widget::Space::new(0, 0).into()
                    }
                }
            }
            2 => {
                let mut iter = link_elements.into_iter();
                match (iter.next(), iter.next()) {
                    (Some(btn1), Some(btn2)) => column![btn1, btn2]
                        .spacing(2)
                        .into(),
                    _ => {
                        tracing::warn!("Expected 2 link buttons but not all were available");
                        cosmic::widget::Space::new(0, 0).into()
                    }
                }
            }
            _ => {
                let mut iter = link_elements.into_iter();
                match (iter.next(), iter.next(), iter.next()) {
                    (Some(btn1), Some(btn2), Some(btn3)) => column![btn1, btn2, btn3]
                        .spacing(2)
                        .into(),
                    _ => {
                        tracing::warn!("Expected 3 link buttons but not all were available");
                        cosmic::widget::Space::new(0, 0).into()
                    }
                }
            }
        };

        column![body_text, links_row]
            .spacing(4)
            .width(Length::Fill)
            .into()
    }

    /// Render body text with HTML markup processing
    ///
    /// Sanitizes HTML and extracts plain text for display.
    /// The markup is processed and validated even though current cosmic widgets
    /// don't support styled text rendering.
    fn render_markup_body(&self, body_html: &str) -> Element<'static, Message> {
        let sanitized = sanitize_html(body_html);
        let segments = parse_markup(&sanitized);

        // Convert segments to plain text
        // Note: Rich text styling (bold/italic) would require cosmic widget support
        // that currently isn't available. The markup is still processed and validated.
        let plain_text: String = segments.iter().map(|s| s.text.as_str()).collect();

        if plain_text.is_empty() {
            return text::caption("").width(Length::Fill).into();
        }

        // Use first line for display
        let display_text = plain_text.lines().next().unwrap_or_default().to_string();
        text::caption(display_text).width(Length::Fill).into()
    }

    fn expire(&mut self, i: u32) {
        let Some((c_pos, _)) = self.cards.iter().enumerate().find(|(_, n)| n.id == i) else {
            return;
        };

        let notification = self.cards.remove(c_pos);
        self.sort_notifications();
        self.group_notifications();
        self.hidden.push_front(notification);

        // Keep newest notifications that fit in memory budget
        // 50MB budget allows ~500 text notifications or ~50 image notifications
        // hidden is ordered with newest at front (push_front above)
        const MAX_HIDDEN_MEMORY: usize = 50 * 1024 * 1024;
        let mut total_size: usize = 0;
        let mut keep_count: usize = 0;

        for n in &self.hidden {
            let size = n.estimated_size();
            if total_size + size > MAX_HIDDEN_MEMORY {
                break;
            }
            total_size += size;
            keep_count += 1;
        }

        // Drop older notifications beyond the budget
        self.hidden.truncate(keep_count);
    }

    fn close(&mut self, i: u32, reason: CloseReason) -> Option<Task<Message>> {
        let c_pos = self.cards.iter().position(|n| n.id == i);
        let notification = c_pos.map(|c_pos| self.cards.remove(c_pos)).or_else(|| {
            self.hidden
                .iter()
                .position(|n| n.id == i)
                .and_then(|pos| self.hidden.remove(pos))
        })?;

        if self.cards.is_empty() {
            self.cards.shrink_to(50);
        }

        self.sort_notifications();
        self.group_notifications();
        if let Some(sender) = &self.notifications_tx {
            let id = notification.id;
            let sender = sender.clone();
            tokio::spawn(async move {
                _ = sender.send(notifications::Input::Closed(id, reason));
            });
        }

        if let Some(sender) = &self.notifications_tx {
            let sender = sender.clone();
            let id = notification.id;
            tokio::spawn(async move { sender.send(notifications::Input::Dismissed(id)).await });
        }

        if self.cards.is_empty() && self.active_surface {
            self.active_surface = false;
            Some(destroy_layer_surface(self.window_id))
        } else {
            Some(Task::none())
        }
    }

    fn anchor_for_notification_applet(&self) -> (Anchor, Option<String>) {
        self.panel_config
            .plugins_left()
            .iter()
            .find_map(|p| {
                if p.iter().any(|s| s == NOTIFICATIONS_APPLET) {
                    return Some((
                        match self.panel_config.anchor {
                            PanelAnchor::Top => Anchor::TOP.union(Anchor::LEFT),
                            PanelAnchor::Bottom => Anchor::BOTTOM.union(Anchor::LEFT),
                            PanelAnchor::Left => Anchor::LEFT.union(Anchor::TOP),
                            PanelAnchor::Right => Anchor::RIGHT.union(Anchor::TOP),
                        },
                        match self.panel_config.output {
                            CosmicPanelOuput::Name(ref n) => Some(n.clone()),
                            _ => None,
                        },
                    ));
                }
                None
            })
            .or_else(|| {
                self.panel_config.plugins_right().iter().find_map(|p| {
                    if p.iter().any(|s| s == NOTIFICATIONS_APPLET) {
                        return Some((
                            match self.panel_config.anchor {
                                PanelAnchor::Top => Anchor::TOP.union(Anchor::RIGHT),
                                PanelAnchor::Bottom => Anchor::BOTTOM.union(Anchor::RIGHT),
                                PanelAnchor::Left => Anchor::LEFT.union(Anchor::BOTTOM),
                                PanelAnchor::Right => Anchor::RIGHT.union(Anchor::BOTTOM),
                            },
                            match self.panel_config.output {
                                CosmicPanelOuput::Name(ref n) => Some(n.clone()),
                                _ => None,
                            },
                        ));
                    }
                    None
                })
            })
            .or_else(|| {
                self.panel_config.plugins_center().iter().find_map(|p| {
                    if p.iter().any(|s| s == NOTIFICATIONS_APPLET) {
                        return Some((
                            match self.panel_config.anchor {
                                PanelAnchor::Top => Anchor::TOP,
                                PanelAnchor::Bottom => Anchor::BOTTOM,
                                PanelAnchor::Left => Anchor::LEFT,
                                PanelAnchor::Right => Anchor::RIGHT,
                            },
                            match self.panel_config.output {
                                CosmicPanelOuput::Name(ref n) => Some(n.clone()),
                                _ => None,
                            },
                        ));
                    }
                    None
                })
            })
            .or_else(|| {
                self.dock_config.plugins_left().iter().find_map(|p| {
                    if p.iter().any(|s| s == NOTIFICATIONS_APPLET) {
                        return Some((
                            match self.dock_config.anchor {
                                PanelAnchor::Top => Anchor::TOP.union(Anchor::LEFT),
                                PanelAnchor::Bottom => Anchor::BOTTOM.union(Anchor::LEFT),
                                PanelAnchor::Left => Anchor::LEFT.union(Anchor::TOP),
                                PanelAnchor::Right => Anchor::RIGHT.union(Anchor::TOP),
                            },
                            match self.dock_config.output {
                                CosmicPanelOuput::Name(ref n) => Some(n.clone()),
                                _ => None,
                            },
                        ));
                    }
                    None
                })
            })
            .or_else(|| {
                self.dock_config.plugins_right().iter().find_map(|p| {
                    if p.iter().any(|s| s == NOTIFICATIONS_APPLET) {
                        return Some((
                            match self.dock_config.anchor {
                                PanelAnchor::Top => Anchor::TOP.union(Anchor::RIGHT),
                                PanelAnchor::Bottom => Anchor::BOTTOM.union(Anchor::RIGHT),
                                PanelAnchor::Left => Anchor::TOP.union(Anchor::BOTTOM),
                                PanelAnchor::Right => Anchor::RIGHT.union(Anchor::BOTTOM),
                            },
                            match self.dock_config.output {
                                CosmicPanelOuput::Name(ref n) => Some(n.clone()),
                                _ => None,
                            },
                        ));
                    }
                    None
                })
            })
            .or_else(|| {
                self.dock_config.plugins_center().iter().find_map(|p| {
                    if p.iter().any(|s| s == NOTIFICATIONS_APPLET) {
                        return Some((
                            match self.dock_config.anchor {
                                PanelAnchor::Top => Anchor::TOP,
                                PanelAnchor::Bottom => Anchor::BOTTOM,
                                PanelAnchor::Left => Anchor::LEFT,
                                PanelAnchor::Right => Anchor::RIGHT,
                            },
                            match self.dock_config.output {
                                CosmicPanelOuput::Name(ref n) => Some(n.clone()),
                                _ => None,
                            },
                        ));
                    }
                    None
                })
            })
            .unwrap_or((Anchor::TOP, None))
    }

    fn push_notification(
        &mut self,
        notification: Notification,
    ) -> Task<<CosmicNotifications as cosmic::app::Application>::Message> {
        // Play notification sound if not in do-not-disturb mode
        #[cfg(feature = "audio")]
        if !self.config.do_not_disturb {
            notification.play_sound();
        }

        let mut timeout = u32::try_from(notification.expire_timeout).unwrap_or(3000);
        let max_timeout = if notification.urgency() == 2 {
            self.config.max_timeout_urgent
        } else if notification.urgency() == 1 {
            self.config.max_timeout_normal
        } else {
            self.config.max_timeout_low
        }
        .unwrap_or(u32::try_from(notification.expire_timeout).unwrap_or(3000));
        timeout = timeout.min(max_timeout);

        let mut tasks = vec![if timeout > 0 {
            iced::Task::perform(
                tokio::time::sleep(Duration::from_millis(timeout as u64)),
                move |_| cosmic::action::app(Message::Timeout(notification.id)),
            )
        } else {
            iced::Task::none()
        }];

        if self.cards.is_empty() && !self.config.do_not_disturb {
            let (anchor, _output) = self.anchor.clone().unwrap_or((Anchor::TOP, None));
            self.active_surface = true;
            tasks.push(get_layer_surface(SctkLayerSurfaceSettings {
                id: self.window_id,
                anchor,
                exclusive_zone: 0,
                keyboard_interactivity: KeyboardInteractivity::None,
                namespace: "notifications".to_string(),
                margin: IcedMargin {
                    top: 8,
                    right: 8,
                    bottom: 8,
                    left: 8,
                },
                // Updated width from 300px to 380px for rich notifications
                size: Some((Some(380), Some(1))),
                output: IcedOutput::Active, // TODO should we only create the notification on the output the applet is on?
                size_limits: Limits::NONE
                    .min_width(300.0)
                    .min_height(1.0)
                    .max_height(1920.0)
                    .max_width(380.0),
                ..Default::default()
            }));
        };

        self.sort_notifications();

        let mut insert_sorted =
            |notification: Notification| match self.cards.binary_search_by(|a| {
                match a.urgency().cmp(&notification.urgency()) {
                    std::cmp::Ordering::Equal => a.time.cmp(&notification.time),
                    other => other,
                }
            }) {
                Ok(pos) => {
                    self.cards[pos] = notification;
                }
                Err(pos) => {
                    self.cards.insert(pos, notification);
                }
            };
        insert_sorted(notification);
        self.group_notifications();

        iced::Task::batch(tasks)
    }

    fn group_notifications(&mut self) {
        if self.config.max_per_app == 0 {
            return;
        }

        let mut extra_per_app = Vec::new();
        let mut cur_count = 0;
        let Some(mut cur_id) = self.cards.first().map(|n| n.app_name.clone()) else {
            return;
        };
        self.cards = self
            .cards
            .drain(..)
            .filter(|n| {
                if n.app_name == cur_id {
                    cur_count += 1;
                } else {
                    cur_count = 1;
                    cur_id = n.app_name.clone();
                }
                if cur_count > self.config.max_per_app {
                    extra_per_app.push(n.clone());
                    false
                } else {
                    true
                }
            })
            .collect();

        for n in extra_per_app {
            if self.cards.len() < self.config.max_notifications as usize {
                self.insert_sorted(n);
            } else {
                self.cards.push(n);
            }
        }
    }

    fn insert_sorted(&mut self, notification: Notification) {
        match self
            .cards
            .binary_search_by(|a| match notification.urgency().cmp(&a.urgency()) {
                std::cmp::Ordering::Equal => notification.time.cmp(&a.time),
                other => other,
            }) {
            Ok(pos) => {
                self.cards[pos] = notification;
            }
            Err(pos) => {
                self.cards.insert(pos, notification);
            }
        }
    }

    fn sort_notifications(&mut self) {
        self.cards
            .sort_by(|a, b| match a.urgency().cmp(&b.urgency()) {
                std::cmp::Ordering::Equal => a.time.cmp(&b.time),
                other => other,
            });
    }

    fn replace_notification(&mut self, notification: Notification) -> Task<Message> {
        if let Some(notif) = self.cards.iter_mut().find(|n| n.id == notification.id) {
            *notif = notification;
            Task::none()
        } else {
            tracing::error!("Notification not found... pushing instead");
            self.push_notification(notification)
        }
    }

    fn request_activation(&mut self, i: u32, action: Option<ActionId>) -> Task<Message> {
        activation::request_token(Some(String::from(Self::APP_ID)), Some(self.window_id)).map(
            move |token| cosmic::Action::App(Message::ActivationToken(token, i, action.clone())),
        )
    }

    fn activate_notification(
        &mut self,
        token: String,
        id: u32,
        action: Option<ActionId>,
    ) -> Option<Task<Message>> {
        if let Some(tx) = self.notifications_tx.as_ref() {
            let c_pos = self.cards.iter().position(|n| n.id == id);
            let notification = c_pos.map(|c_pos| &self.cards[c_pos]).or_else(|| {
                self.hidden
                    .iter()
                    .position(|n| n.id == id)
                    .map(|pos| &self.hidden[pos])
            })?;

            let maybe_action = if action
                .as_ref()
                .is_some_and(|a| notification.actions.iter().any(|(b, _)| b == a))
            {
                action.clone().map(|a| a.to_string())
            } else if notification
                .actions
                .iter()
                .any(|a| matches!(a.0, ActionId::Default))
            {
                Some(ActionId::Default.to_string())
            } else {
                notification.actions.first().map(|a| a.0.to_string())
            };

            let Some(action) = maybe_action else {
                return self.close(id, CloseReason::Dismissed);
            };
            let tx = tx.clone();
            tracing::info!("action for {id} {action}");
            return Some(Task::future(async move {
                _ = tx
                    .send(notifications::Input::Activated { token, id, action })
                    .await;
                tracing::trace!("sent action to sub");
                cosmic::Action::App(Message::Dismissed(id))
            }));
        } else {
            tracing::error!("Failed to activate notification. No channel.");
            None
        }
    }
}

impl cosmic::Application for CosmicNotifications {
    type Message = Message;
    type Executor = cosmic::executor::single::Executor;
    type Flags = ();
    const APP_ID: &'static str = "com.system76.CosmicNotifications";

    fn init(core: Core, _flags: ()) -> (Self, Task<Message>) {
        let helper = Config::new(
            cosmic_notifications_config::ID,
            NotificationsConfig::VERSION,
        )
        .ok();

        let config: NotificationsConfig = helper
            .as_ref()
            .map(|helper| {
                NotificationsConfig::get_entry(helper).unwrap_or_else(|(errors, config)| {
                    for err in errors {
                        if err.is_err() {
                            tracing::error!("{:?}", err);
                        }
                    }
                    config
                })
            })
            .unwrap_or_default();
        (
            CosmicNotifications {
                core,
                active_surface: false,
                autosize_id: iced::id::Id::new("autosize"),
                window_id: SurfaceId::unique(),
                anchor: None,
                config,
                dock_config: CosmicPanelConfig::default(),
                panel_config: CosmicPanelConfig::default(),
                notifications_id: id::Cards::new("Notifications"),
                notifications_tx: None,
                timeline: Timeline::new(),
                cards: Vec::with_capacity(50),
                hidden: VecDeque::new(),
            },
            Task::none(),
        )
    }

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn view(&self) -> Element<Self::Message> {
        unimplemented!();
    }

    #[allow(clippy::too_many_lines)]
    fn update(&mut self, message: Message) -> Task<Self::Message> {
        match message {
            Message::ActivateNotification(id) => {
                tracing::trace!("requesting token for {id}");
                return self.request_activation(id, None);
            }
            Message::ActivationToken(token, id, action) => {
                tracing::trace!("token for {id}");
                if let Some(token) = token {
                    return self
                        .activate_notification(token, id, action)
                        .unwrap_or(Task::none());
                } else {
                    tracing::error!("Failed to get activation token for clicked notification.");
                }
            }
            Message::Notification(e) => match e {
                notifications::Event::Notification(n) => {
                    return self.push_notification(n);
                }
                notifications::Event::Replace(n) => {
                    return self.replace_notification(n);
                }
                notifications::Event::CloseNotification(id) => {
                    if let Some(c) = self.close(id, CloseReason::CloseNotification) {
                        return c;
                    }
                }
                notifications::Event::Ready(tx) => {
                    self.notifications_tx = Some(tx);
                }
                notifications::Event::AppletActivated { id, action } => {
                    tracing::trace!("requesting token for {id}");
                    return self.request_activation(id, Some(action));
                }
                notifications::Event::GetHistory { tx } => {
                    // Send the hidden notifications history
                    let history: Vec<_> = self.hidden.iter().cloned().collect();
                    if let Err(err) = tx.send(history) {
                        tracing::error!("Failed to send history response: {:?}", err);
                    }
                }
            },
            Message::Dismissed(id) => {
                if let Some(c) = self.close(id, CloseReason::Dismissed) {
                    return c;
                }
            }
            Message::Timeout(id) => {
                self.expire(id);
                if self.cards.is_empty() && self.active_surface {
                    self.active_surface = false;
                    return destroy_layer_surface(self.window_id);
                }
            }
            Message::Config(config) => {
                self.config = config;
            }
            Message::PanelConfig(c) => {
                self.panel_config = c;
                self.anchor = Some(self.anchor_for_notification_applet());
            }
            Message::DockConfig(c) => {
                self.dock_config = c;
                self.anchor = Some(self.anchor_for_notification_applet());
            }
            Message::Frame(now) => {
                self.timeline.now(now);
            }
            Message::Ignore => {}
            Message::Surface(a) => {
                return cosmic::task::message(cosmic::Action::Cosmic(
                    cosmic::app::Action::Surface(a),
                ));
            }
            Message::LinkClicked(url) => {
                // Open link in default browser
                if cosmic_notifications_util::is_safe_url(&url) {
                    if let Err(e) = cosmic_notifications_util::open_link(&url) {
                        tracing::error!("Failed to open link {}: {}", url, e);
                    }
                } else {
                    tracing::warn!("Blocked unsafe URL: {}", url);
                }
            }
            Message::ActionClicked(id, action_id) => {
                // Handle action button click - request activation with the action
                tracing::trace!("action clicked for {id}: {action_id}");
                return self.request_activation(id, Some(action_id.parse().unwrap_or(ActionId::Default)));
            }
        }
        Task::none()
    }

    #[allow(clippy::too_many_lines)]
    fn view_window(&self, _: SurfaceId) -> Element<Message> {
        if self.cards.is_empty() {
            return container(vertical_space().height(Length::Fixed(1.0)))
                .center_x(Length::Fixed(1.0))
                .center_y(Length::Fixed(1.0))
                .into();
        }

        // Get rich card config from settings
        let card_config = RichCardConfig::from_notifications_config(&self.config);

        let (ids, notif_elems): (Vec<_>, Vec<_>) = self
            .cards
            .iter()
            .rev()
            .map(|n| {
                let e = self.render_rich_notification(n, &card_config);
                (n.id, e)
            })
            .take(self.config.max_notifications as usize)
            .unzip();

        // Card list with animations - width increased from 300px to 380px
        // for rich notifications with images and progress bars.
        // The anim! macro handles smooth entry/exit animations based on card
        // height automatically. Taller rich cards animate smoothly from the edge.
        let card_list = anim!(
            //cards
            self.notifications_id.clone(),
            &self.timeline,
            notif_elems,
            Message::Ignore,
            None::<fn(cosmic_time::chain::Cards, bool) -> Message>,
            Some(move |id| Message::ActivateNotification(ids[id])),
            "",
            "",
            "",
            None,
            true,
        )
        .width(Length::Fixed(380.));

        // Autosize container updated to match new 380px card width
        autosize::autosize(card_list, self.autosize_id.clone())
            .min_width(200.)
            .min_height(100.)
            .max_width(380.)
            .max_height(1920.)
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(vec![
            self.core
                .watch_config(cosmic_notifications_config::ID)
                .map(|u| {
                    for why in u
                        .errors
                        .into_iter()
                        .filter(cosmic::cosmic_config::Error::is_err)
                    {
                        tracing::error!(?why, "config load error");
                    }
                    Message::Config(u.config)
                }),
            self.core
                .watch_config("com.system76.CosmicPanel.Panel")
                .map(|u| {
                    for why in u
                        .errors
                        .into_iter()
                        .filter(cosmic::cosmic_config::Error::is_err)
                    {
                        tracing::error!(?why, "panel config load error");
                    }
                    Message::PanelConfig(u.config)
                }),
            self.core
                .watch_config("com.system76.CosmicPanel.Dock")
                .map(|u| {
                    for why in u
                        .errors
                        .into_iter()
                        .filter(cosmic::cosmic_config::Error::is_err)
                    {
                        tracing::error!(?why, "dock config load error");
                    }
                    Message::DockConfig(u.config)
                }),
            self.timeline
                .as_subscription()
                .map(|(_, now)| Message::Frame(now)),
            notifications::notifications().map(Message::Notification),
        ])
    }
}
