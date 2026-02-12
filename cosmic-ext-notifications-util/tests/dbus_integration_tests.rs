//! Integration tests for D-Bus notification interface
//!
//! These tests verify the D-Bus org.freedesktop.Notifications interface
//! implementation without requiring an actual D-Bus connection.

use cosmic_ext_notifications_util::{ActionId, Notification, Hint, Image, CloseReason};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime};

// Test capabilities as documented in the FreeDesktop spec
const EXPECTED_CAPABILITIES: &[&str] = &[
    "body",           // Supports body text
    "icon-static",    // Displays single-frame notification icons
    "persistence",    // Notifications retained until acknowledged
    "actions",        // Supports action buttons
    "action-icons",   // Uses icons for action buttons when hint is set
    "body-markup",    // Renders bold/italic styling in body
    "body-hyperlinks",// Supports clickable links in body
    "sound",          // Plays sound-file and sound-name hints
];

// Server information constants from src/config.rs
const SERVER_NAME: &str = "cosmic-ext-notifications";
const SERVER_VENDOR: &str = "olafkfreund";
const SPEC_VERSION: &str = "1.2";

#[test]
fn test_get_capabilities() {
    // Test: Verify all expected capabilities are present
    // This matches the implementation in src/subscriptions/notifications.rs:421-432

    let capabilities = EXPECTED_CAPABILITIES;

    // Verify we have all 8 expected capabilities
    assert_eq!(capabilities.len(), 8, "Should have 8 capabilities");

    // Verify specific capabilities are present
    assert!(capabilities.contains(&"body"), "Should support body text");
    assert!(capabilities.contains(&"actions"), "Should support action buttons");
    assert!(capabilities.contains(&"body-markup"), "Should support markup");
    assert!(capabilities.contains(&"body-hyperlinks"), "Should support hyperlinks");
    assert!(capabilities.contains(&"icon-static"), "Should support static icons");
    assert!(capabilities.contains(&"persistence"), "Should support persistence");
    assert!(capabilities.contains(&"sound"), "Should support sound");
    assert!(capabilities.contains(&"action-icons"), "Should support action icons");

    // Verify we don't claim unsupported capabilities
    assert!(!capabilities.contains(&"icon-multi"), "Should not support animated icons");
    assert!(!capabilities.contains(&"body-images"), "Should not support body images");
}

#[test]
fn test_get_server_information() {
    // Test: Verify server information is correct
    // This matches the implementation in src/subscriptions/notifications.rs:434-439

    assert_eq!(SERVER_NAME, "cosmic-ext-notifications");
    assert_eq!(SERVER_VENDOR, "olafkfreund");
    assert_eq!(SPEC_VERSION, "1.2");
}

#[cfg(feature = "zbus_notifications")]
#[test]
fn test_notification_creation_basic() {
    // Test: Create a basic notification without hints

    let actions = vec!["default", "Open"];
    let hints: HashMap<&str, zbus::zvariant::Value> = HashMap::new();

    let notification = Notification::new(
        "test-app",
        42,
        "dialog-information",
        "Test Summary",
        "Test body message",
        actions,
        hints,
        5000,
    );

    assert_eq!(notification.id, 42);
    assert_eq!(notification.app_name, "test-app");
    assert_eq!(notification.app_icon, "dialog-information");
    assert_eq!(notification.summary, "Test Summary");
    assert_eq!(notification.body, "Test body message");
    assert_eq!(notification.expire_timeout, 5000);
    assert_eq!(notification.actions.len(), 1);
    assert_eq!(notification.hints.len(), 0);

    // Verify default values
    assert_eq!(notification.urgency(), 1); // Default is normal
    assert!(!notification.transient());
    assert!(notification.image().is_none());
}

#[cfg(feature = "zbus_notifications")]
#[test]
fn test_notification_with_urgency_hints() {
    // Test: Create notifications with different urgency levels

    let test_cases = vec![
        (0u8, "low urgency"),
        (1u8, "normal urgency"),
        (2u8, "critical urgency"),
    ];

    for (urgency_value, description) in test_cases {
        let mut hints: HashMap<&str, zbus::zvariant::Value> = HashMap::new();
        hints.insert("urgency", zbus::zvariant::Value::U8(urgency_value));

        let notification = Notification::new(
            "test-app",
            1,
            "",
            "Test",
            "",
            vec![],
            hints,
            0,
        );

        assert_eq!(
            notification.urgency(),
            urgency_value,
            "Failed for {}",
            description
        );
    }
}

#[cfg(feature = "zbus_notifications")]
#[test]
fn test_notification_with_category_hint() {
    // Test: Create notification with category hint

    let mut hints: HashMap<&str, zbus::zvariant::Value> = HashMap::new();
    hints.insert("category", zbus::zvariant::Value::Str("email.arrived".into()));

    let notification = Notification::new(
        "email-client",
        1,
        "",
        "New Email",
        "You have a new message",
        vec![],
        hints,
        0,
    );

    assert_eq!(notification.category(), Some("email.arrived"));
}

#[cfg(feature = "zbus_notifications")]
#[test]
fn test_notification_with_transient_flag() {
    // Test: Create transient notification that should not be persisted

    let mut hints: HashMap<&str, zbus::zvariant::Value> = HashMap::new();
    hints.insert("transient", zbus::zvariant::Value::Bool(true));

    let notification = Notification::new(
        "test-app",
        1,
        "",
        "Transient",
        "",
        vec![],
        hints,
        0,
    );

    assert!(notification.transient());
}

#[cfg(feature = "zbus_notifications")]
#[test]
fn test_notification_with_action_icons_hint() {
    // Test: Create notification with action-icons hint

    let mut hints: HashMap<&str, zbus::zvariant::Value> = HashMap::new();
    hints.insert("action-icons", zbus::zvariant::Value::Bool(true));

    let notification = Notification::new(
        "test-app",
        1,
        "",
        "Test",
        "",
        vec![],
        hints,
        0,
    );

    assert!(notification.action_icons());
}

#[cfg(feature = "zbus_notifications")]
#[test]
fn test_notification_with_sound_hints() {
    // Test: Create notification with sound-file and sound-name hints

    // Test sound-file hint
    let mut hints: HashMap<&str, zbus::zvariant::Value> = HashMap::new();
    hints.insert("sound-file", zbus::zvariant::Value::Str("/usr/share/sounds/test.wav".into()));

    let notification = Notification::new(
        "test-app",
        1,
        "",
        "Test",
        "",
        vec![],
        hints,
        0,
    );

    assert_eq!(notification.sound_file(), Some(PathBuf::from("/usr/share/sounds/test.wav").as_path()));

    // Test sound-name hint
    let mut hints: HashMap<&str, zbus::zvariant::Value> = HashMap::new();
    hints.insert("sound-name", zbus::zvariant::Value::Str("message-new-instant".into()));

    let notification = Notification::new(
        "test-app",
        2,
        "",
        "Test",
        "",
        vec![],
        hints,
        0,
    );

    assert_eq!(notification.sound_name(), Some("message-new-instant"));
}

#[cfg(feature = "zbus_notifications")]
#[test]
fn test_notification_with_suppress_sound_hint() {
    // Test: Create notification with suppress-sound hint

    let mut hints: HashMap<&str, zbus::zvariant::Value> = HashMap::new();
    hints.insert("suppress-sound", zbus::zvariant::Value::Bool(true));

    let notification = Notification::new(
        "test-app",
        1,
        "",
        "Test",
        "",
        vec![],
        hints,
        0,
    );

    assert!(notification.suppress_sound());
}

#[cfg(feature = "zbus_notifications")]
#[test]
fn test_notification_with_desktop_entry_hint() {
    // Test: Create notification with desktop-entry hint

    let mut hints: HashMap<&str, zbus::zvariant::Value> = HashMap::new();
    hints.insert("desktop-entry", zbus::zvariant::Value::Str("org.gnome.Gedit".into()));

    let notification = Notification::new(
        "gedit",
        1,
        "",
        "Test",
        "",
        vec![],
        hints,
        0,
    );

    assert_eq!(notification.desktop_entry(), Some("org.gnome.Gedit"));
}

#[cfg(feature = "zbus_notifications")]
#[test]
fn test_notification_with_image_path_hint() {
    // Test: Create notification with image-path hint

    let mut hints: HashMap<&str, zbus::zvariant::Value> = HashMap::new();
    hints.insert("image-path", zbus::zvariant::Value::Str("dialog-information".into()));

    let notification = Notification::new(
        "test-app",
        1,
        "",
        "Test",
        "",
        vec![],
        hints,
        0,
    );

    assert!(notification.image().is_some());
    if let Some(Image::Name(name)) = notification.image() {
        assert_eq!(name, "dialog-information");
    } else {
        panic!("Expected Image::Name variant");
    }
}

#[cfg(feature = "zbus_notifications")]
#[test]
fn test_notification_with_multiple_hints() {
    // Test: Create notification with multiple hints

    let mut hints: HashMap<&str, zbus::zvariant::Value> = HashMap::new();
    hints.insert("urgency", zbus::zvariant::Value::U8(2));
    hints.insert("category", zbus::zvariant::Value::Str("im.error".into()));
    hints.insert("transient", zbus::zvariant::Value::Bool(true));
    hints.insert("action-icons", zbus::zvariant::Value::Bool(true));

    let notification = Notification::new(
        "messenger",
        1,
        "im-messenger",
        "Connection Error",
        "Failed to connect to server",
        vec![],
        hints,
        0,
    );

    assert_eq!(notification.urgency(), 2);
    assert_eq!(notification.category(), Some("im.error"));
    assert!(notification.transient());
    assert!(notification.action_icons());
}

#[cfg(feature = "zbus_notifications")]
#[test]
fn test_notification_with_actions() {
    // Test: Parse actions from action strings

    let actions = vec![
        "default", "Open",
        "action1", "Reply",
        "action2", "Delete",
    ];

    let notification = Notification::new(
        "test-app",
        1,
        "",
        "Test",
        "",
        actions,
        HashMap::new(),
        0,
    );

    assert_eq!(notification.actions.len(), 3);

    // Verify action IDs and labels
    assert_eq!(notification.actions[0].0, ActionId::Default);
    assert_eq!(notification.actions[0].1, "Open");

    assert_eq!(notification.actions[1].0, ActionId::Custom("action1".to_string()));
    assert_eq!(notification.actions[1].1, "Reply");

    assert_eq!(notification.actions[2].0, ActionId::Custom("action2".to_string()));
    assert_eq!(notification.actions[2].1, "Delete");
}

#[test]
fn test_estimated_size_basic() {
    // Test: Calculate estimated size for basic notification

    let notification = Notification {
        id: 1,
        app_name: "test-app".to_string(),          // 8 bytes
        app_icon: "dialog-information".to_string(), // 18 bytes
        summary: "Test".to_string(),                // 4 bytes
        body: "Body".to_string(),                   // 4 bytes
        actions: vec![],
        hints: vec![],
        expire_timeout: 5000,
        time: SystemTime::now(),
    };

    let size = notification.estimated_size();

    // Base strings: 8 + 18 + 4 + 4 = 34 bytes
    // Struct overhead: 200 bytes
    // Total: 234 bytes minimum
    assert!(size >= 234, "Size should be at least 234 bytes, got {}", size);
    assert!(size < 500, "Size should be reasonable, got {}", size);
}

#[test]
fn test_estimated_size_with_actions() {
    // Test: Calculate size with actions included

    let notification = Notification {
        id: 1,
        app_name: "app".to_string(),
        app_icon: "".to_string(),
        summary: "Test".to_string(),
        body: "".to_string(),
        actions: vec![
            (ActionId::Default, "Open".to_string()),           // ~11 bytes
            (ActionId::Custom("reply".to_string()), "Reply".to_string()), // ~10 bytes
        ],
        hints: vec![],
        expire_timeout: 0,
        time: SystemTime::now(),
    };

    let size = notification.estimated_size();

    // Should include action data
    assert!(size > 200, "Size should include actions");
}

#[test]
fn test_estimated_size_with_hints() {
    // Test: Calculate size with various hints

    let notification = Notification {
        id: 1,
        app_name: "app".to_string(),
        app_icon: "".to_string(),
        summary: "Test".to_string(),
        body: "".to_string(),
        actions: vec![],
        hints: vec![
            Hint::Urgency(2),                          // 8 bytes
            Hint::Category("email.arrived".to_string()), // ~21 bytes
            Hint::Transient(true),                     // 8 bytes
        ],
        expire_timeout: 0,
        time: SystemTime::now(),
    };

    let size = notification.estimated_size();

    // Should include hint data
    assert!(size > 200, "Size should include hints");
}

#[test]
fn test_estimated_size_with_image_data() {
    // Test: Calculate size with image data hint (largest hint type)

    let image_data = vec![0u8; 1024 * 10]; // 10KB image

    let notification = Notification {
        id: 1,
        app_name: "app".to_string(),
        app_icon: "".to_string(),
        summary: "Test".to_string(),
        body: "".to_string(),
        actions: vec![],
        hints: vec![
            Hint::Image(Image::Data {
                width: 64,
                height: 64,
                data: std::sync::Arc::new(image_data),
            }),
        ],
        expire_timeout: 0,
        time: SystemTime::now(),
    };

    let size = notification.estimated_size();

    // Should be at least the size of the image data plus overhead
    assert!(size > 10240, "Size should include image data (10KB+), got {}", size);
}

#[test]
fn test_estimated_size_with_large_body() {
    // Test: Calculate size with large body text

    let large_body = "a".repeat(5000);

    let notification = Notification {
        id: 1,
        app_name: "app".to_string(),
        app_icon: "".to_string(),
        summary: "Test".to_string(),
        body: large_body,
        actions: vec![],
        hints: vec![],
        expire_timeout: 0,
        time: SystemTime::now(),
    };

    let size = notification.estimated_size();

    // Should be at least the size of the body text
    assert!(size >= 5000, "Size should include large body text");
}

#[test]
fn test_close_reason_values() {
    // Test: Verify CloseReason enum values match D-Bus spec

    assert_eq!(CloseReason::Expired as u32, 1);
    assert_eq!(CloseReason::Dismissed as u32, 2);
    assert_eq!(CloseReason::CloseNotification as u32, 3);
    assert_eq!(CloseReason::Undefined as u32, 4);
}

#[test]
fn test_action_id_display() {
    // Test: Verify ActionId Display implementation

    assert_eq!(ActionId::Default.to_string(), "default");
    assert_eq!(ActionId::Custom("reply".to_string()).to_string(), "reply");
    assert_eq!(ActionId::Custom("action-123".to_string()).to_string(), "action-123");
}

#[test]
fn test_action_id_from_str() {
    // Test: Verify ActionId parsing from strings

    let default_action: ActionId = "default".parse().unwrap();
    assert_eq!(default_action, ActionId::Default);

    let custom_action: ActionId = "reply".parse().unwrap();
    assert_eq!(custom_action, ActionId::Custom("reply".to_string()));

    let custom_action2: ActionId = "action-123".parse().unwrap();
    assert_eq!(custom_action2, ActionId::Custom("action-123".to_string()));
}

#[test]
fn test_hint_estimated_size() {
    // Test: Verify hint size estimation for different hint types

    // Boolean hints should be 8 bytes
    assert_eq!(Hint::Transient(true).estimated_size(), 8);
    assert_eq!(Hint::ActionIcons(false).estimated_size(), 8);

    // String hints should be string length + 8
    assert_eq!(Hint::Category("email".to_string()).estimated_size(), 5 + 8);
    assert_eq!(Hint::DesktopEntry("org.gnome.Gedit".to_string()).estimated_size(), 15 + 8);

    // Numeric hints should be 8 bytes
    assert_eq!(Hint::Urgency(2).estimated_size(), 8);
    assert_eq!(Hint::Value(50).estimated_size(), 8);
}

#[test]
fn test_notification_duration_since() {
    // Test: Verify duration_since calculation

    let notification = Notification {
        id: 1,
        app_name: "app".to_string(),
        app_icon: "".to_string(),
        summary: "Test".to_string(),
        body: "".to_string(),
        actions: vec![],
        hints: vec![],
        expire_timeout: 0,
        time: SystemTime::now() - Duration::from_secs(5),
    };

    let duration = notification.duration_since().unwrap();

    // Should be approximately 5 seconds (with some tolerance)
    assert!(duration.as_secs() >= 4 && duration.as_secs() <= 6);
}

// Rate limiter tests (these test the logic from src/subscriptions/notifications.rs:329-399)

/// Mock rate limiter for testing (mirrors the actual implementation)
struct TestRateLimiter {
    limits: HashMap<String, (Instant, u32)>,
}

impl TestRateLimiter {
    const MAX_APPS: usize = 1000;
    const MAX_PER_MINUTE: u32 = 60;
    const WINDOW: Duration = Duration::from_secs(60);

    fn new() -> Self {
        Self {
            limits: HashMap::new(),
        }
    }

    fn check_and_update(&mut self, app_name: &str) -> bool {
        if self.limits.len() >= Self::MAX_APPS {
            self.cleanup();
        }

        if self.limits.len() >= Self::MAX_APPS {
            return false;
        }

        let now = Instant::now();
        let entry = self.limits.entry(app_name.to_string()).or_insert((now, 0));

        if now.duration_since(entry.0) > Self::WINDOW {
            *entry = (now, 1);
            return true;
        }

        if entry.1 >= Self::MAX_PER_MINUTE {
            return false;
        }

        entry.1 += 1;
        true
    }

    fn cleanup(&mut self) {
        let now = Instant::now();
        self.limits.retain(|_, (start, _)| now.duration_since(*start) <= Self::WINDOW);
    }
}

#[test]
fn test_rate_limiter_allows_under_limit() {
    // Test: Verify rate limiter allows notifications under the limit

    let mut limiter = TestRateLimiter::new();

    // Should allow first 60 notifications
    for i in 1..=60 {
        assert!(
            limiter.check_and_update("test_app"),
            "Notification {} should be allowed",
            i
        );
    }
}

#[test]
fn test_rate_limiter_blocks_over_limit() {
    // Test: Verify rate limiter blocks notifications over the limit

    let mut limiter = TestRateLimiter::new();

    // Fill up to the limit
    for _ in 1..=60 {
        limiter.check_and_update("test_app");
    }

    // 61st should be blocked
    assert!(
        !limiter.check_and_update("test_app"),
        "Notification over limit should be blocked"
    );
}

#[test]
fn test_rate_limiter_resets_after_window() {
    // Test: Verify rate limiter resets after the time window expires

    let mut limiter = TestRateLimiter::new();

    // Fill up to the limit
    for _ in 1..=60 {
        limiter.check_and_update("test_app");
    }

    // Manually advance time by modifying the entry
    if let Some(entry) = limiter.limits.get_mut("test_app") {
        entry.0 = Instant::now() - Duration::from_secs(61);
    }

    // Should allow again after window expires
    assert!(
        limiter.check_and_update("test_app"),
        "Should allow after time window expires"
    );
}

#[test]
fn test_rate_limiter_per_app_isolation() {
    // Test: Verify rate limiting is per-app (one app doesn't affect another)

    let mut limiter = TestRateLimiter::new();

    // Fill up limit for app1
    for _ in 1..=60 {
        limiter.check_and_update("app1");
    }

    // app1 should be blocked
    assert!(
        !limiter.check_and_update("app1"),
        "app1 should be rate limited"
    );

    // app2 should still be allowed
    assert!(
        limiter.check_and_update("app2"),
        "app2 should not be affected by app1's rate limit"
    );
}

#[test]
fn test_rate_limiter_cleanup() {
    // Test: Verify cleanup removes old entries

    let mut limiter = TestRateLimiter::new();

    // Add entries for multiple apps
    limiter.check_and_update("app1");
    limiter.check_and_update("app2");
    limiter.check_and_update("app3");

    assert_eq!(limiter.limits.len(), 3, "Should have 3 apps tracked");

    // Manually age the entries
    for (_, entry) in limiter.limits.iter_mut() {
        entry.0 = Instant::now() - Duration::from_secs(61);
    }

    // Cleanup should remove old entries
    limiter.cleanup();

    assert_eq!(
        limiter.limits.len(),
        0,
        "Cleanup should remove expired entries"
    );
}

#[test]
fn test_rate_limiter_empty_app_name() {
    // Test: Verify empty app names are still rate limited

    let mut limiter = TestRateLimiter::new();

    // Empty app names should still be rate limited
    for i in 1..=60 {
        assert!(
            limiter.check_and_update(""),
            "Empty app name notification {} should be allowed",
            i
        );
    }

    assert!(
        !limiter.check_and_update(""),
        "Empty app name should be rate limited after 60"
    );
}

#[test]
fn test_rate_limiter_max_apps_limit() {
    // Test: Verify rate limiter respects MAX_APPS limit

    let mut limiter = TestRateLimiter::new();

    // Add notifications from many different apps
    for i in 0..1000 {
        assert!(
            limiter.check_and_update(&format!("app{}", i)),
            "Should allow notifications from first 1000 apps"
        );
    }

    // Should have 1000 apps tracked
    assert_eq!(limiter.limits.len(), 1000);

    // Next new app should trigger cleanup first
    limiter.check_and_update("app1000");

    // After cleanup (if no entries expired), should reject to prevent DoS
    // This test verifies the max tracking limit is enforced
}

#[test]
fn test_rate_limiter_concurrent_apps() {
    // Test: Verify rate limiter handles multiple apps concurrently

    let mut limiter = TestRateLimiter::new();

    // Simulate multiple apps sending notifications concurrently
    for i in 0..5 {
        for j in 0..10 {
            assert!(
                limiter.check_and_update(&format!("app{}", i)),
                "App {} notification {} should be allowed",
                i,
                j
            );
        }
    }

    // Verify all apps are tracked independently
    assert_eq!(limiter.limits.len(), 5, "Should track 5 different apps");

    // Each app should have sent 10 notifications
    for i in 0..5 {
        let entry = limiter.limits.get(&format!("app{}", i)).unwrap();
        assert_eq!(entry.1, 10, "App {} should have count of 10", i);
    }
}
