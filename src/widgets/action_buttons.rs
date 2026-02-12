#![allow(dead_code)]

use cosmic::iced::Alignment;
use cosmic::iced_widget::row;
use cosmic::widget::button;
use cosmic::Element;
use cosmic_ext_notifications_util::{ActionId, NotificationAction};

/// Maximum number of action buttons to display
const MAX_VISIBLE_ACTIONS: usize = 3;

/// Message type for action button clicks
#[derive(Debug, Clone)]
pub enum ActionMessage {
  Clicked(u32, String), // (notification_id, action_id)
}

/// Create a row of action buttons for a notification
pub fn action_buttons_row<'a, Message: Clone + 'static>(
  notification_id: u32,
  actions: &'a [NotificationAction],
  on_action: impl Fn(u32, String) -> Message + 'static + Clone,
) -> Element<'a, Message> {
  // Filter out default action and limit to MAX_VISIBLE_ACTIONS
  let visible_actions: Vec<_> = actions
    .iter()
    .filter(|a| a.id != "default")
    .take(MAX_VISIBLE_ACTIONS)
    .collect();

  if visible_actions.is_empty() {
    return cosmic::widget::Space::new(0, 0).into();
  }

  // Build buttons
  let mut elements: Vec<Element<'a, Message>> = Vec::with_capacity(visible_actions.len());

  for action in visible_actions {
    let action_id = action.id.clone();
    let label = action.label.clone();
    let on_action = on_action.clone();

    let btn: Element<'a, Message> = button::text(label)
      .on_press((on_action)(notification_id, action_id))
      .padding([6, 12])
      .into();

    elements.push(btn);
  }

  // Use row! macro with collected elements by folding
  match elements.len() {
    0 => cosmic::widget::Space::new(0, 0).into(),
    1 => elements.into_iter().next().unwrap(),
    2 => {
      let mut iter = elements.into_iter();
      row![iter.next().unwrap(), iter.next().unwrap()]
        .spacing(8)
        .align_y(Alignment::Center)
        .into()
    }
    _ => {
      let mut iter = elements.into_iter();
      row![
        iter.next().unwrap(),
        iter.next().unwrap(),
        iter.next().unwrap()
      ]
      .spacing(8)
      .align_y(Alignment::Center)
      .into()
    }
  }
}

/// Create a single action button
pub fn action_button<'a, Message: Clone + 'static>(
  action: &'a NotificationAction,
  notification_id: u32,
  on_action: impl Fn(u32, String) -> Message + 'static,
) -> Element<'a, Message> {
  let action_id = action.id.clone();
  let label = action.label.clone();

  button::text(label)
    .on_press(on_action(notification_id, action_id))
    .padding([6, 12])
    .into()
}

/// Check if there are any displayable actions (excluding default)
pub fn has_displayable_actions(actions: &[NotificationAction]) -> bool {
  actions.iter().any(|a| a.id != "default")
}

/// Convert from Notification tuple format to NotificationAction
pub fn convert_action_tuple(action: &(ActionId, String)) -> NotificationAction {
  NotificationAction {
    id: action.0.to_string(),
    label: action.1.clone(),
  }
}

/// Convert a slice of action tuples to NotificationActions
pub fn convert_actions(actions: &[(ActionId, String)]) -> Vec<NotificationAction> {
  actions.iter().map(convert_action_tuple).collect()
}
