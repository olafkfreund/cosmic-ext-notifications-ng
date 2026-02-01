use cosmic::widget::text;
use cosmic::Element;
use cosmic_notifications_util::NotificationLink;

/// Message for link clicks
#[derive(Debug, Clone)]
pub struct LinkClicked(pub String);

/// Create body text with clickable links
/// For now, this creates plain text - full link clicking requires
/// more complex widget composition that will be added during integration
pub fn linkified_body<'a, Message: 'static>(
  body: &'a str,
  _links: &[NotificationLink],
) -> Element<'a, Message> {
  // For Phase 3, create basic text element
  // Full link interactivity will be integrated in Phase 4
  text::body(body).into()
}

/// Check if text contains any links
pub fn has_links(text: &str) -> bool {
  use cosmic_notifications_util::detect_links;
  !detect_links(text).is_empty()
}
