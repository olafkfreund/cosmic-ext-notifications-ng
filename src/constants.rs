// Constants module for cosmic-ext-notifications
// Centralizes magic numbers for better maintainability

// ============================================================================
// UI Layout Constants
// ============================================================================

/// Width of notification cards in pixels
pub(crate) const NOTIFICATION_WIDTH: f32 = 380.0;

/// Minimum width for notification cards
pub(crate) const NOTIFICATION_MIN_WIDTH: f32 = 300.0;

/// Maximum height for notification cards
pub(crate) const NOTIFICATION_MAX_HEIGHT: f32 = 1920.0;

/// Margin around notifications (pixels)
pub(crate) const NOTIFICATION_MARGIN: i32 = 8;

/// Minimum width for autosize mode
pub(crate) const AUTOSIZE_MIN_WIDTH: f32 = 200.0;

/// Minimum height for autosize mode
pub(crate) const AUTOSIZE_MIN_HEIGHT: f32 = 100.0;

/// Padding inside notification cards
pub(crate) const CARD_PADDING: u16 = 12;

// ============================================================================
// Icon Size Constants
// ============================================================================

/// Small icon size (e.g., for app icons)
pub(crate) const ICON_SIZE_SMALL: u16 = 16;

/// Large icon size (e.g., for fallback icons)
pub(crate) const ICON_SIZE_LARGE: u16 = 96;

// ============================================================================
// Text Display Constants
// ============================================================================

/// Maximum length for app name before truncation
pub(crate) const APP_NAME_MAX_LENGTH: usize = 24;

/// Maximum visible action buttons on a notification card
pub(crate) const MAX_VISIBLE_ACTIONS: usize = 3;

// ============================================================================
// Notification Queue Constants
// ============================================================================

/// Maximum memory budget for hidden notifications (50MB)
pub(crate) const MAX_HIDDEN_MEMORY: usize = 50 * 1024 * 1024;

/// Initial capacity for notification cards vector
pub(crate) const INITIAL_CARDS_CAPACITY: usize = 50;

// ============================================================================
// Rate Limiting Constants
// ============================================================================

/// Maximum notifications per minute per app
pub(crate) const RATE_LIMIT_PER_MINUTE: u32 = 60;

/// Maximum number of apps tracked by rate limiter
pub(crate) const RATE_LIMIT_MAX_APPS: usize = 1000;

/// Interval for rate limiter cleanup (in notification count)
pub(crate) const RATE_LIMIT_CLEANUP_INTERVAL: u64 = 100;

// ============================================================================
// Channel and Buffer Constants
// ============================================================================

/// Buffer size for notification channel
pub(crate) const CHANNEL_BUFFER_SIZE: usize = 100;

// ============================================================================
// URL Display Constants
// ============================================================================

/// Maximum URL length for single URL display before truncation
pub(crate) const URL_DISPLAY_MAX_SINGLE: usize = 40;

/// Maximum URL length for multiple URL display before truncation
pub(crate) const URL_DISPLAY_MAX_MULTI: usize = 30;
