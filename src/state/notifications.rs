#![allow(dead_code)]

use cosmic_notifications_util::Notification;
use std::collections::VecDeque;
use crate::constants::*;

/// Manages the state of notification queues
///
/// Handles both visible notification cards and hidden notification history
/// with memory budget management. Some methods are prepared for future
/// integration with the notification grouping and per-app rules systems.
pub struct NotificationState {
    /// Currently visible notification cards
    cards: Vec<Notification>,
    /// Hidden notifications (dismissed or expired)
    hidden: VecDeque<Notification>,
}

impl NotificationState {
    /// Create a new notification state
    pub fn new() -> Self {
        Self {
            cards: Vec::with_capacity(INITIAL_CARDS_CAPACITY),
            hidden: VecDeque::new(),
        }
    }

    /// Get visible notifications
    pub fn visible(&self) -> &[Notification] {
        &self.cards
    }

    /// Get mutable reference to visible notifications
    pub(crate) fn visible_mut(&mut self) -> &mut Vec<Notification> {
        &mut self.cards
    }

    /// Get hidden notifications
    pub fn hidden(&self) -> &VecDeque<Notification> {
        &self.hidden
    }

    /// Get mutable reference to hidden notifications
    pub(crate) fn hidden_mut(&mut self) -> &mut VecDeque<Notification> {
        &mut self.hidden
    }

    /// Add a notification to the visible cards
    pub fn add_notification(&mut self, notification: Notification) {
        self.cards.push(notification);
    }

    /// Remove a notification by ID from both visible and hidden queues
    ///
    /// Returns the removed notification if found
    pub fn remove_notification(&mut self, id: u32) -> Option<Notification> {
        if let Some(pos) = self.cards.iter().position(|n| n.id == id) {
            Some(self.cards.remove(pos))
        } else {
            self.hidden
                .iter()
                .position(|n| n.id == id)
                .and_then(|pos| self.hidden.remove(pos))
        }
    }

    /// Move a notification from visible cards to hidden history
    ///
    /// Applies memory budget management to hidden queue
    pub fn hide_notification(&mut self, id: u32) {
        let Some(pos) = self.cards.iter().position(|n| n.id == id) else {
            return;
        };

        let notification = self.cards.remove(pos);
        self.hidden.push_front(notification);

        // Apply memory budget: allows ~500 text or ~50 image notifications
        self.apply_memory_budget(MAX_HIDDEN_MEMORY);
    }

    /// Apply memory budget to hidden notifications
    ///
    /// Keeps newest notifications that fit within the budget
    fn apply_memory_budget(&mut self, max_memory: usize) {
        let mut total_size: usize = 0;
        let mut keep_count: usize = 0;

        for n in &self.hidden {
            let size = n.estimated_size();
            if total_size + size > max_memory {
                break;
            }
            total_size += size;
            keep_count += 1;
        }

        self.hidden.truncate(keep_count);
    }

    /// Get total memory usage of hidden notifications
    pub fn hidden_memory_usage(&self) -> usize {
        self.hidden.iter().map(|n| n.estimated_size()).sum()
    }

    /// Check if visible cards is empty
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    /// Get count of visible notifications
    pub fn visible_count(&self) -> usize {
        self.cards.len()
    }

    /// Shrink visible cards capacity
    pub fn shrink_visible(&mut self) {
        if self.cards.is_empty() {
            self.cards.shrink_to(INITIAL_CARDS_CAPACITY);
        }
    }

    /// Sort visible notifications by urgency and time
    pub fn sort_visible(&mut self) {
        self.cards
            .sort_by(|a, b| match a.urgency().cmp(&b.urgency()) {
                std::cmp::Ordering::Equal => a.time.cmp(&b.time),
                other => other,
            });
    }

    /// Insert notification in sorted position
    pub fn insert_sorted(&mut self, notification: Notification) {
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

    /// Group notifications by app, limiting per-app count
    ///
    /// Returns extra notifications that were removed from visible queue
    pub fn group_by_app(&mut self, max_per_app: usize, max_total: usize) -> Vec<Notification> {
        if max_per_app == 0 {
            return Vec::new();
        }

        let mut extra_per_app = Vec::new();
        let mut cur_count = 0;
        let Some(mut cur_id) = self.cards.first().map(|n| n.app_name.clone()) else {
            return Vec::new();
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
                if cur_count > max_per_app {
                    extra_per_app.push(n.clone());
                    false
                } else {
                    true
                }
            })
            .collect();

        // Re-add extras if room in max_total
        for n in extra_per_app.iter() {
            if self.cards.len() < max_total {
                self.insert_sorted(n.clone());
            } else {
                self.cards.push(n.clone());
            }
        }

        extra_per_app
    }
}

impl Default for NotificationState {
    fn default() -> Self {
        Self::new()
    }
}
