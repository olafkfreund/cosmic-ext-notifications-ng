use crate::handlers::Message;
use crate::widgets::{notification_image, ImageSize};
use crate::constants::*;
use cosmic::iced::Length;
use cosmic::iced_widget::{column, container};
use cosmic::widget::{icon, text};
use cosmic::Element;
use cosmic_notifications_util::{
    parse_markup, sanitize_html, Image, Notification, NotificationImage,
    NotificationLink, ProcessedImage,
};

/// Render notification image from Image hint
///
/// Uses Expanded size (128x128) for better visibility with text content
pub fn render_notification_image(image: &Image) -> Option<Element<'static, Message>> {
    match image {
        Image::Data { width, height, data } => {
            // Create ProcessedImage from raw data
            // Clone the inner Vec from Arc - only happens during rendering
            let processed = ProcessedImage {
                data: (**data).clone(),
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

/// Render body text with HTML markup processing
///
/// Sanitizes HTML and extracts plain text for display.
/// The markup is processed and validated even though current cosmic widgets
/// don't support styled text rendering.
pub fn render_markup_body(body_html: &str) -> Element<'static, Message> {
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

/// Render body text with clickable link segments
///
/// For simplicity, renders the full body text followed by clickable link buttons.
/// This avoids complex text segmentation while still making links clickable.
pub fn render_body_with_links(
    body: &str,
    links: &[NotificationLink],
) -> Element<'static, Message> {
    use cosmic::widget::button;

    // Show the full body text
    let body_text: Element<'static, Message> = text::caption(body.to_string())
        .width(Length::Fill)
        .into();

    // If only one link, show body + single link button
    if links.len() == 1 {
        let link = &links[0];
        let url = link.url.clone();
        let display_url = if url.len() > URL_DISPLAY_MAX_SINGLE {
            format!("{}...", &url[..(URL_DISPLAY_MAX_SINGLE - 3)])
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
        let display_url = if url.len() > URL_DISPLAY_MAX_MULTI {
            format!("{}...", &url[..(URL_DISPLAY_MAX_MULTI - 3)])
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

/// Extract progress value from notification hints
pub fn get_progress_from_hints(n: &Notification) -> Option<f32> {
    use cosmic_notifications_util::Hint;
    use crate::widgets::should_show_progress;

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
