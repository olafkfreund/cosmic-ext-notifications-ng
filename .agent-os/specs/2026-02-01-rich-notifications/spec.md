# Rich Notifications Implementation Spec

> Spec: Rich Notifications for cosmic-ext-notifications
> Created: 2026-02-01
> Status: Planning
> Reference: cosmic-connect-desktop-app Rich Notifications (Issues #120-#126)

## Overview

Implement rich notification support in cosmic-ext-notifications by porting the proven implementation from cosmic-connect-desktop-app. This will replace the current basic text-only notification display with a full-featured notification system supporting images, formatted text, action buttons, progress indicators, and clickable links.

## User Stories

### As a desktop user, I want notifications with images
As a desktop user, I want to see notification images and thumbnails, so that I can quickly understand the context of the notification (e.g., contact photos, app icons, file previews).

### As a desktop user, I want actionable notifications
As a desktop user, I want to click action buttons on notifications, so that I can respond without opening the application (e.g., "Reply", "Mark as Read", "Dismiss All").

### As a desktop user, I want formatted notification text
As a desktop user, I want to see formatted text with bold, italic, and links, so that important information stands out and I can click links directly.

### As a desktop user, I want progress notifications
As a desktop user, I want to see progress bars in notifications, so that I can track downloads, file transfers, and other long-running operations.

### As a desktop user, I want animated images in notifications
As a desktop user, I want to see animated GIFs and other animated images in notifications, so that I can view dynamic content like reactions, stickers, and previews without opening the source application.

## Spec Scope

### Phase 1: Core Infrastructure
1. **Data Structure Enhancement** - Port `NotificationLink`, `NotificationAction`, rich content fields to notification types
2. **Image Processing Module** - Implement `NotificationImage` for image resize/conversion
3. **HTML Sanitization** - Add secure HTML filtering (use `ammonia` crate for production)

### Phase 2: DBus Enhancement
4. **Enhanced Hint Parsing** - Extract all image-data, image-path, icon_data hints
5. **Action Button Parsing** - Parse action pairs into structured `NotificationAction` types
6. **Urgency and Category** - Full urgency level and category support with visual distinction

### Phase 3: UI Rendering
7. **Rich Card Layout** - Redesign notification cards for rich content
8. **Image Rendering** - Display notification images (resize to 64x64 or 128x128)
9. **Action Buttons UI** - Render clickable action buttons with proper styling
10. **Progress Bar Widget** - Implement progress indicator rendering
11. **Link Handling** - Make URLs in body text clickable
12. **Animated Image Support** - Display animated GIFs, APNG, and WebP with frame playback

### Phase 4: Integration & Polish
13. **Animation Updates** - Smooth transitions for richer content
14. **Configuration Options** - Settings for image sizes, action button visibility, animation toggle
15. **Testing & Documentation** - Comprehensive tests, update README

## Out of Scope

- Full video playback in notifications (animated GIFs/APNG/WebP ARE supported)
- Inline reply text input (requires additional protocol support)
- Cross-device notification forwarding (cosmic-connect handles this)
- Custom notification sounds (parse hints but don't play)
- Notification grouping/stacking redesign (separate spec)

## Expected Deliverables

1. **Enhanced Notification Struct** with rich content fields matching cosmic-connect-protocol
2. **NotificationImage module** for image processing (port from cosmic-connect-daemon)
3. **HTML sanitizer** using `ammonia` crate (safer than regex)
4. **Redesigned notification card widget** with image, actions, progress support
5. **Action button handling** with DBus `ActionInvoked` signal integration
6. **Progress bar widget** for percentage-based notifications
7. **Clickable links** with URL validation and `open::that()` integration
8. **Animated image support** for GIF, APNG, and WebP with frame playback
9. **Configuration options** for rich notification behavior (including animation toggle)
10. **Unit tests** for all new functionality (target: 60+ tests)
11. **Updated documentation** with examples and screenshots

## Architecture Overview

### Current Architecture (cosmic-ext-notifications)
```
DBus (org.freedesktop.Notifications.Notify)
    ↓
zbus handler (notifications.rs)
    ↓
Notification struct (basic fields only)
    ↓
UI rendering (text only, 300px cards)
    ↓
Layer Shell surface
```

### Target Architecture (Rich Notifications)
```
DBus (org.freedesktop.Notifications.Notify)
    ↓
zbus handler (notifications.rs)
    ↓
RichContentExtractor (NEW)
    │ ├─ Parse rich_body (HTML/Pango)
    │ ├─ Extract images (image-data, image-path, icon_data)
    │ ├─ Detect links (URLs, emails)
    │ └─ Parse actions
    ↓
NotificationImage (NEW - port from cosmic-connect)
    │ └─ Resize, convert to RGBA, cache
    ↓
Enhanced Notification struct
    ↓
RichNotificationCard widget (NEW)
    │ ├─ App icon (32x32)
    │ ├─ Image thumbnail (64x64 or 128x128)
    │ ├─ Formatted text with links
    │ ├─ Action buttons row
    │ └─ Progress bar (optional)
    ↓
Layer Shell surface
```

## Data Structures to Add

### From cosmic-connect-protocol:
```rust
pub struct NotificationLink {
    pub url: String,
    pub title: Option<String>,
    pub start: usize,
    pub length: usize,
}

pub struct NotificationAction {
    pub id: String,
    pub label: String,
}

pub enum NotificationUrgency {
    Low = 0,
    Normal = 1,
    Critical = 2,
}
```

### Enhanced Notification (add fields):
```rust
pub struct Notification {
    // ... existing fields ...

    // Rich content (NEW)
    pub rich_body: Option<String>,          // Sanitized HTML
    pub image_data: Option<ProcessedImage>, // Processed image
    pub links: Vec<NotificationLink>,       // Detected links
    pub urgency: NotificationUrgency,       // Urgency level
    pub category: Option<String>,           // Notification category
    pub progress: Option<f32>,              // Progress percentage (0.0-1.0)
}

pub struct ProcessedImage {
    pub data: Vec<u8>,      // RGBA pixels
    pub width: u32,
    pub height: u32,
}
```

## Dependencies to Add

```toml
# Image processing (already have fast_image_resize)
image = "0.25"              # Image loading/conversion

# HTML sanitization (safer than regex)
ammonia = "4"               # Production-grade HTML sanitizer

# URL handling
open = "5"                  # Open links in browser
url = "2"                   # URL parsing and validation

# Link detection
linkify = "0.10"           # Automatic URL detection in text
```

## Key Implementation Details

### Image Processing (port from cosmic-connect-daemon)
- Max dimensions: 64x64 for inline, 128x128 for expanded
- Format: RGBA32 (4 bytes per pixel)
- Resize algorithm: Lanczos3 (high quality)
- Support input formats: PNG, JPEG, raw RGBA/RGB data from DBus

### Animated Image Support
- Supported formats: GIF, APNG (animated PNG), WebP
- Frame extraction using `image` crate's animation decoders
- Maximum 100 frames per animation (memory protection)
- Maximum 30 second animation duration
- Configurable: `enable_animations` option to disable
- Playback at native frame rate with timing from file
- Automatic fallback to static first frame if disabled

### HTML Sanitization (improvement over cosmic-connect)
- Use `ammonia` crate instead of regex (handles edge cases)
- Allowed tags: `<b>`, `<i>`, `<u>`, `<a>`
- Allowed attributes: `href` on `<a>` only
- Block: `javascript:` URLs, `data:` URLs
- Strip: All event handlers (onclick, onerror, etc.)

### Action Button Handling
- Parse actions from DBus (alternating id/label pairs)
- Render as horizontal button row below body
- On click: Send `ActionInvoked` signal via DBus
- Support action icons hint

### Progress Bar
- Check for `value` hint (integer 0-100) or `progress` hint
- Render as horizontal bar below body
- Update on notification replacement (same id)
- Animate value changes smoothly

## Risk Assessment

### Technical Risks
1. **libcosmic widget limitations** - May need custom widgets for rich content
   - Mitigation: Check libcosmic capabilities, fall back to iced primitives
2. **Memory usage with images** - Large images could consume RAM
   - Mitigation: Enforce max dimensions, process/discard originals
3. **Animation performance** - Rich cards may impact animation smoothness
   - Mitigation: Profile and optimize, limit concurrent animations

### Compatibility Risks
1. **DBus spec compliance** - Must maintain freedesktop notification compatibility
   - Mitigation: All new fields optional, existing behavior preserved
2. **Existing integrations** - Apps expecting current behavior
   - Mitigation: Feature flags for rich content, gradual rollout

## Success Metrics

- All existing notification tests pass
- 50+ new unit tests for rich content
- Image notifications display correctly (manual testing)
- Action buttons trigger correct DBus signals
- Progress bars animate smoothly
- HTML content sanitized (no XSS vectors)
- Memory usage < 100MB with 10 image notifications
- Animation frame rate > 30fps on standard hardware
