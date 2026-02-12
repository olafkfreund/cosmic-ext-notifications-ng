# Changelog

All notable changes to COSMIC Ext Notifications will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added - Rich Notifications Support

This release adds comprehensive rich notification features, implementing the full freedesktop.org Notification Specification with enhanced capabilities.

#### Image Support
- **Image Display**: Support for displaying images from file paths (`image-path` hint)
- **Raw Image Data**: Support for images from raw RGBA data (`image-data`, `icon_data` hints)
- **Automatic Resizing**: Images automatically resized to configurable maximum (default 128x128)
- **Format Support**: PNG, JPEG, GIF, and other common image formats
- **Performance**: Efficient image processing with Lanczos3 resampling

#### Animated Images
- **GIF Animation**: Full support for animated GIF playback
- **APNG/WebP**: Support for APNG and WebP animated formats
- **Frame Timing**: Proper frame timing based on image metadata
- **Memory Limits**: Safe limits (max 100 frames, 30s duration) to prevent memory exhaustion
- **Configuration**: Can be disabled via `enable_animations` config option

#### Action Buttons
- **Multiple Actions**: Support for multiple action buttons per notification
- **Default Action**: Support for default action (click anywhere on notification)
- **DBus Signals**: Proper ActionInvoked signal emission
- **Themed Styling**: Action buttons styled with COSMIC theme
- **Action Icons**: Support for `action-icons` hint
- **Configuration**: Can be disabled via `show_actions` config option

#### Progress Indicators
- **Progress Bars**: Visual progress bar widget for download/upload notifications
- **Value Hint**: Support for `value` hint (0-100)
- **Smooth Animation**: Animated progress bar updates
- **Theme Integration**: Progress bars styled with COSMIC theme colors

#### Clickable Links
- **URL Detection**: Automatic detection of URLs in notification body text
- **Click Handling**: Click to open URLs in default browser
- **Security**: Only http:// and https:// URLs are clickable (blocks javascript:, file:, etc.)
- **Visual Styling**: Links visually distinguished from normal text
- **Configuration**: Can be disabled via `enable_links` config option

#### Urgency Levels
- **Visual Distinction**: Different colors for Low, Normal, and Critical urgency
- **Theme Integration**: Urgency colors match COSMIC theme
- **Urgency Hint**: Support for `urgency` hint (0, 1, 2)

#### Category Support
- **Category Hints**: Support for notification category hints
- **Category Icons**: Category-specific icons for common types
- **Known Categories**: Email, IM, system events, network, device, etc.
- **Visual Grouping**: Notifications styled based on category

#### HTML and Text Processing
- **HTML Sanitization**: Safe rendering of basic HTML tags (b, i, u, a)
- **XSS Protection**: Automatic removal of dangerous tags (script, iframe, style, etc.)
- **Safe HTML**: Whitelist approach using ammonia library
- **Plain Text Fallback**: Graceful handling of non-HTML notifications

#### Configuration Options
- `show_images`: Enable/disable image display (default: true)
- `show_actions`: Enable/disable action buttons (default: true)
- `max_image_size`: Maximum image size in pixels (default: 128)
- `enable_links`: Enable/disable clickable links (default: true)
- `enable_animations`: Enable/disable animated images (default: true)

#### New Modules
- `cosmic-notifications-util/src/notification_image.rs`: Image processing
- `cosmic-notifications-util/src/animated_image.rs`: Animation handling
- `cosmic-notifications-util/src/sanitizer.rs`: HTML sanitization
- `cosmic-notifications-util/src/link.rs`: Link data structures
- `cosmic-notifications-util/src/link_detector.rs`: URL detection
- `cosmic-notifications-util/src/action.rs`: Action data structures
- `cosmic-notifications-util/src/action_parser.rs`: Action parsing
- `cosmic-notifications-util/src/urgency.rs`: Urgency types
- `cosmic-notifications-util/src/urgency_style.rs`: Urgency styling
- `cosmic-notifications-util/src/rich_content.rs`: Rich content helpers

#### Dependencies Added
- `ammonia`: HTML sanitization
- `linkify`: URL detection in text
- `open`: Open URLs in default browser
- `image`: Image processing and format support (existing, enhanced usage)

#### Documentation
- `TESTING.md`: Comprehensive testing guide with real application examples
- `docs/PERFORMANCE.md`: Performance characteristics and benchmarks
- `docs/screenshots/README.md`: Screenshot guidelines for documentation
- `scripts/test_rich_notifications.sh`: Automated test script
- Enhanced `README.md`: Features, configuration, and usage examples

#### Performance
- **Target Frame Rate**: 30 FPS for all animations
- **Memory Usage**: < 100 MB with 10+ rich notifications
- **Latency**: < 100ms notification appearance time
- **CPU Usage**: < 5% for single animation
- **Optimizations**: Lazy loading, image caching, efficient rendering

#### Testing
- Integration tests for full notification flow
- Tests for sanitizer + link detection
- Tests for action parsing and urgency styling
- Backward compatibility tests for basic notifications

### Changed
- Enhanced `Notification` struct with rich content support
- Updated hint parsing to extract image data, actions, and urgency
- Improved notification card layout (wider cards for rich content)
- Enhanced card animations for taller rich notification cards

### Fixed
- Proper handling of malformed image data
- Safe URL validation to prevent security issues
- Correct rowstride handling for various image formats
- Memory leaks in animated image playback

### Security
- XSS protection via HTML sanitization
- URL validation to prevent malicious links
- Safe image processing to prevent buffer overflows
- Memory limits on animated images to prevent DoS

## [Previous Releases]

<!-- Add previous release notes here -->
