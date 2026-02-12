# Rich Notifications Tasks

These are the tasks to be completed for the spec detailed in @.agent-os/specs/2026-02-01-rich-notifications/spec.md

> Created: 2026-02-01
> Status: Ready for Implementation
> Estimated Duration: 3-4 weeks (including animated image support)
> Reference Implementation: cosmic-connect-desktop-app (Issues #120-#126)

## Phase 1: Core Infrastructure

- [ ] 1. **Port Rich Notification Data Structures**
  - [ ] 1.1 Write tests for `NotificationLink` serialization/deserialization
  - [ ] 1.2 Write tests for `NotificationAction` struct
  - [ ] 1.3 Write tests for `NotificationUrgency` enum conversions
  - [ ] 1.4 Add `NotificationLink` struct to `cosmic-ext-notifications-util`
  - [ ] 1.5 Add `NotificationAction` struct to `cosmic-ext-notifications-util`
  - [ ] 1.6 Add `NotificationUrgency` enum with From<u8> impl
  - [ ] 1.7 Extend `Notification` struct with rich content fields
  - [ ] 1.8 Verify all tests pass

- [ ] 2. **Implement NotificationImage Module**
  - [ ] 2.1 Write tests for image resizing (various dimensions)
  - [ ] 2.2 Write tests for RGBA/RGB conversion
  - [ ] 2.3 Write tests for rowstride handling
  - [ ] 2.4 Write tests for PNG encoding/decoding
  - [ ] 2.5 Create `notification_image.rs` module in cosmic-ext-notifications-util
  - [ ] 2.6 Implement `NotificationImage` struct with `from_raw_data()` method
  - [ ] 2.7 Implement `from_path()` for file-based images
  - [ ] 2.8 Implement resize logic (max 128x128, Lanczos3)
  - [ ] 2.9 Implement `to_rgba()` for UI rendering
  - [ ] 2.10 Verify all tests pass

- [ ] 3. **Implement HTML Sanitizer**
  - [ ] 3.1 Write tests for allowed tags (b, i, u, a)
  - [ ] 3.2 Write tests for blocked tags (script, style, iframe, img)
  - [ ] 3.3 Write tests for dangerous attribute removal (onclick, onerror)
  - [ ] 3.4 Write tests for javascript: URL blocking
  - [ ] 3.5 Add `ammonia` dependency to Cargo.toml
  - [ ] 3.6 Create `sanitizer.rs` module in cosmic-ext-notifications-util
  - [ ] 3.7 Implement `sanitize_html()` function with ammonia
  - [ ] 3.8 Implement `has_rich_content()` detection helper
  - [ ] 3.9 Verify all tests pass

## Phase 2: DBus Enhancement

- [ ] 4. **Enhanced Hint Parsing**
  - [ ] 4.1 Write tests for `image-data` hint extraction
  - [ ] 4.2 Write tests for `image-path` hint extraction
  - [ ] 4.3 Write tests for `icon_data` hint extraction
  - [ ] 4.4 Write tests for `value` (progress) hint extraction
  - [ ] 4.5 Enhance `Hint` enum with new variants if needed
  - [ ] 4.6 Update hint parsing in notifications.rs to extract all image types
  - [ ] 4.7 Add progress hint parsing (value: i32 0-100)
  - [ ] 4.8 Integrate NotificationImage processing into hint parsing
  - [ ] 4.9 Verify all tests pass

- [ ] 5. **Action Button Parsing Enhancement**
  - [ ] 5.1 Write tests for action pair parsing (id, label alternating)
  - [ ] 5.2 Write tests for action icon hint
  - [ ] 5.3 Write tests for default action handling
  - [ ] 5.4 Update action parsing to create `NotificationAction` structs
  - [ ] 5.5 Handle action-icons hint for button icons
  - [ ] 5.6 Implement action ID to button mapping
  - [ ] 5.7 Verify all tests pass

- [ ] 6. **Urgency and Category Visual Distinction**
  - [ ] 6.1 Write tests for urgency level parsing (0, 1, 2)
  - [ ] 6.2 Write tests for category hint parsing
  - [ ] 6.3 Update urgency handling in Notification struct
  - [ ] 6.4 Add category field to Notification
  - [ ] 6.5 Define color/style constants for urgency levels
  - [ ] 6.6 Verify all tests pass

## Phase 3: UI Rendering

- [ ] 7. **Rich Notification Card Layout**
  - [ ] 7.1 Design new card layout mockup (wider, taller cards)
  - [ ] 7.2 Create `rich_notification_card.rs` widget module
  - [ ] 7.3 Implement card header row (icon, app name, close, time)
  - [ ] 7.4 Implement card body section (image, text, links)
  - [ ] 7.5 Implement card footer row (action buttons)
  - [ ] 7.6 Add progress bar slot in card layout
  - [ ] 7.7 Update card width from 300px to 380px
  - [ ] 7.8 Integrate with existing Cards animation system
  - [ ] 7.9 Visual testing with various notification types

- [ ] 8. **Image Rendering in Cards**
  - [ ] 8.1 Write tests for image widget creation from ProcessedImage
  - [ ] 8.2 Create image display widget using libcosmic/iced
  - [ ] 8.3 Implement inline image thumbnail (64x64)
  - [ ] 8.4 Implement expanded image view (128x128)
  - [ ] 8.5 Add fallback for failed image loads
  - [ ] 8.6 Handle app icon rendering (32x32)
  - [ ] 8.7 Visual testing with image notifications

- [ ] 9. **Action Buttons UI**
  - [ ] 9.1 Write tests for action button click handling
  - [ ] 9.2 Write tests for ActionInvoked signal emission
  - [ ] 9.3 Create action button row widget
  - [ ] 9.4 Style buttons with cosmic theme (ghost buttons)
  - [ ] 9.5 Implement button click → ActionInvoked signal
  - [ ] 9.6 Handle action icons when available
  - [ ] 9.7 Limit visible buttons (max 3, overflow menu)
  - [ ] 9.8 Visual testing with multi-action notifications

- [ ] 10. **Progress Bar Widget**
  - [ ] 10.1 Write tests for progress value parsing
  - [ ] 10.2 Write tests for progress animation
  - [ ] 10.3 Create progress bar widget using iced::widget::progress_bar or cosmic equivalent
  - [ ] 10.4 Style with cosmic theme colors
  - [ ] 10.5 Implement value updates on notification replacement
  - [ ] 10.6 Add smooth animation for value changes
  - [ ] 10.7 Visual testing with progress notifications

- [ ] 11. **Clickable Links in Body Text**
  - [ ] 11.1 Write tests for URL detection in plain text
  - [ ] 11.2 Write tests for link click handling
  - [ ] 11.3 Add `linkify` dependency to Cargo.toml
  - [ ] 11.4 Add `open` dependency to Cargo.toml
  - [ ] 11.5 Implement link detection in notification body
  - [ ] 11.6 Create clickable text spans for links
  - [ ] 11.7 Implement link click → open::that(url)
  - [ ] 11.8 Add URL validation (http/https only)
  - [ ] 11.9 Visual testing with link-containing notifications

- [ ] 12. **Animated Image Support (GIF, APNG, WebP)**
  - [ ] 12.1 Write tests for animated image detection
  - [ ] 12.2 Write tests for frame extraction and timing
  - [ ] 12.3 Write tests for animation loop behavior
  - [ ] 12.4 Implement `AnimatedImage` struct with frame storage
  - [ ] 12.5 Implement `ImageAnimator` for playback control
  - [ ] 12.6 Add animated image detection in hint parsing
  - [ ] 12.7 Integrate animation with notification image widget
  - [ ] 12.8 Add `enable_animations` config option
  - [ ] 12.9 Add `max_animation_frames` config option (default: 100)
  - [ ] 12.10 Implement animation frame subscription in app
  - [ ] 12.11 Add memory limit protection (max 100 frames, max 30s duration)
  - [ ] 12.12 Visual testing with animated GIFs
  - [ ] 12.13 Performance testing (CPU/memory impact)

## Phase 4: Integration & Polish

- [x] 13. **Animation Updates**
  - [x] 13.1 Update card entry animation for taller cards
  - [x] 13.2 Update card exit animation
  - [x] 13.3 Add image fade-in animation
  - [x] 13.4 Add progress bar animation
  - [x] 13.5 Performance testing (target: 30fps)
  - [x] 13.6 Memory profiling with multiple image notifications

- [x] 14. **Configuration Options**
  - [x] 14.1 Add `show_images` config option (default: true)
  - [x] 14.2 Add `show_actions` config option (default: true)
  - [x] 14.3 Add `max_image_size` config option (default: 128)
  - [x] 14.4 Add `enable_links` config option (default: true)
  - [x] 14.5 Add `enable_animations` config option (default: true)
  - [x] 14.6 Update cosmic-ext-notifications-config
  - [x] 14.7 Wire config to rendering logic
  - [x] 14.8 Test config changes apply correctly

- [x] 15. **Final Testing & Documentation**
  - [x] 15.1 Create test script `scripts/test_rich_notifications.sh`
  - [x] 15.2 Write integration tests for full notification flow
  - [x] 15.3 Document testing with real applications (TESTING.md)
  - [x] 15.4 Document animated GIF notification testing
  - [x] 15.5 Update README.md with rich notification features
  - [x] 15.6 Add example screenshots placeholders (docs/screenshots/)
  - [x] 15.7 Verify backward compatibility with basic notifications
  - [x] 15.8 Performance benchmarks documented (docs/PERFORMANCE.md)
  - [x] 15.9 Prepare for PR (CHANGELOG.md created, tasks updated)

## Dependencies Between Tasks

```
Phase 1 (Foundation):
  Task 1 ──┬──→ Task 4 (needs data structures)
  Task 2 ──┤
  Task 3 ──┘

Phase 2 (DBus):
  Task 4 ──┬──→ Task 7 (needs parsed data)
  Task 5 ──┤
  Task 6 ──┘

Phase 3 (UI):
  Task 7 ──┬──→ Task 13 (needs widgets)
  Task 8 ──┤
  Task 9 ──┤
  Task 10 ─┤
  Task 11 ─┤
  Task 12 ─┘ (animated images, depends on Task 8)

Phase 4 (Polish):
  Task 13 ─┬──→ Task 15 (needs complete implementation)
  Task 14 ─┘
```

## Effort Estimates

| Task | Effort | Notes |
|------|--------|-------|
| 1. Data Structures | S (1 day) | Port from cosmic-connect |
| 2. Image Module | M (2 days) | Port and adapt from cosmic-connect |
| 3. HTML Sanitizer | S (1 day) | ammonia makes this simple |
| 4. Hint Parsing | M (2 days) | Extend existing parser |
| 5. Action Parsing | S (1 day) | Minor enhancement |
| 6. Urgency/Category | XS (0.5 day) | Simple additions |
| 7. Card Layout | L (3 days) | Major UI work |
| 8. Image Rendering | M (2 days) | New widget |
| 9. Action Buttons | M (2 days) | New widget + DBus signals |
| 10. Progress Bar | S (1 day) | Use existing widget |
| 11. Clickable Links | M (2 days) | Text parsing + click handling |
| 12. Animated Images | M (2-3 days) | Frame extraction + playback |
| 13. Animations | S (1 day) | Update existing |
| 14. Configuration | S (1 day) | Extend config |
| 15. Testing/Docs | M (2 days) | Comprehensive testing |

**Total Estimate: 21-24 days**

## Files to Create/Modify

### New Files
- `cosmic-ext-notifications-util/src/notification_image.rs`
- `cosmic-ext-notifications-util/src/animated_image.rs`
- `cosmic-ext-notifications-util/src/sanitizer.rs`
- `cosmic-ext-notifications-util/src/link.rs`
- `cosmic-ext-notifications-util/src/action.rs`
- `src/widgets/rich_card.rs`
- `src/widgets/action_buttons.rs`
- `src/widgets/progress_bar.rs`
- `src/widgets/image_animator.rs`
- `scripts/test_rich_notifications.sh`

### Modified Files
- `cosmic-ext-notifications-util/src/lib.rs` (Notification struct, exports)
- `cosmic-ext-notifications-util/src/image.rs` (enhance existing)
- `cosmic-ext-notifications-util/Cargo.toml` (new dependencies)
- `cosmic-ext-notifications-config/src/lib.rs` (new config options)
- `src/app.rs` (new message types, rendering)
- `src/subscriptions/notifications.rs` (enhanced parsing)
- `Cargo.toml` (workspace dependencies)
- `README.md` (documentation)

## Reference Materials

- **cosmic-connect-desktop-app Rich Notifications**: Closed issues #120-#126
- **cosmic-connect-protocol/src/plugins/notification.rs**: Data structures
- **cosmic-connect-daemon/src/notification_image.rs**: Image processing
- **cosmic-connect-daemon/src/cosmic_ext_notifications.rs**: DBus integration
- **Freedesktop Notification Spec**: https://specifications.freedesktop.org/notification-spec/
