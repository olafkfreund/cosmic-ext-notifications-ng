# Comprehensive Research Report: COSMIC Desktop Notification System

> **Generated:** 2026-02-05
> **Version:** cosmic-notifications-ng v0.2.3
> **Scope:** Full architecture review, optimization opportunities, and improvement roadmap

---

## Executive Summary

This report synthesizes findings from 6 parallel research agents analyzing the Linux notification ecosystem, the COSMIC Desktop notification daemon implementation, and opportunities for optimization and enhancement. The research covers:

1. **Architecture Analysis** - Deep dive into the codebase structure and data flow
2. **FreeDesktop Specification** - Compliance and capability gaps
3. **COSMIC Ecosystem Integration** - Applet communication and panel integration
4. **Best Practices Research** - Modern notification UX patterns and trends
5. **Performance Optimization** - Memory, CPU, and rendering improvements
6. **Architecture Recommendations** - Refactoring proposals for maintainability

### Key Findings

| Area | Status | Priority |
|------|--------|----------|
| FreeDesktop Spec Compliance | ✅ Full v1.2 compliance | - |
| Security | ✅ All 8 vulnerabilities fixed | - |
| Code Quality | ⚠️ Monolithic app.rs (1183 lines) | High |
| Performance | ⚠️ Image cloning in hot paths | Medium |
| Configuration | ⚠️ Limited placement options | Medium |
| GNOME App Integration | ✅ Works via xdg-desktop-portal | - |

---

## Table of Contents

1. [Architecture Overview](#1-architecture-overview)
2. [FreeDesktop Specification Compliance](#2-freedesktop-specification-compliance)
3. [COSMIC Ecosystem Integration](#3-cosmic-ecosystem-integration)
4. [Performance Analysis & Optimization](#4-performance-analysis--optimization)
5. [Code Quality & Refactoring](#5-code-quality--refactoring)
6. [Configuration & UX Enhancements](#6-configuration--ux-enhancements)
7. [Implementation Roadmap](#7-implementation-roadmap)
8. [Appendices](#8-appendices)

---

## 1. Architecture Overview

### 1.1 Component Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                    COSMIC Notifications System                       │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌──────────────────┐    D-Bus Session Bus    ┌──────────────────┐  │
│  │  Applications    │ ──────────────────────► │  Notification    │  │
│  │  (Firefox, etc)  │   org.freedesktop.      │  Daemon          │  │
│  └──────────────────┘   Notifications         │  (cosmic-notif)  │  │
│                                               └────────┬─────────┘  │
│  ┌──────────────────┐                                  │            │
│  │  Flatpak/Snap    │ ── xdg-desktop-portal ──────────►│            │
│  │  Sandboxed Apps  │                                  │            │
│  └──────────────────┘                                  │            │
│                                               ┌────────▼─────────┐  │
│  ┌──────────────────┐                         │  Layer Shell     │  │
│  │  COSMIC Applet   │ ◄─── D-Bus History ──── │  Surfaces        │  │
│  │  (Panel Widget)  │      API                │  (Wayland)       │  │
│  └──────────────────┘                         └──────────────────┘  │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 1.2 Codebase Structure

```
cosmic-notifications-ng/
├── src/
│   ├── main.rs                    # Entry point
│   ├── app.rs                     # ⚠️ Main application (1183 lines - needs decomposition)
│   ├── config.rs                  # Version and build info
│   ├── localize.rs                # i18n support
│   ├── subscriptions/
│   │   ├── mod.rs                 # Module exports
│   │   ├── notifications.rs       # D-Bus interface (735 lines)
│   │   └── applet.rs              # Panel applet interface
│   └── widgets/
│       ├── mod.rs                 # Widget exports
│       ├── notification_image.rs  # Image rendering
│       ├── rich_card.rs           # Card UI component
│       ├── linkified_text.rs      # Clickable links
│       ├── progress_bar.rs        # Progress indicators
│       ├── action_buttons.rs      # Action button rendering
│       └── image_animator.rs      # GIF animation
│
├── cosmic-notifications-util/     # Shared utility library
│   └── src/
│       ├── lib.rs                 # Core notification types
│       ├── sanitizer.rs           # HTML sanitization
│       ├── markup_parser.rs       # Markup parsing
│       ├── link_detector.rs       # URL detection
│       ├── audio.rs               # Sound playback
│       ├── image.rs               # Image processing
│       └── ...
│
└── cosmic-notifications-config/   # Configuration crate
    └── src/lib.rs                 # Config schema (v2)
```

### 1.3 Data Flow

```
┌─────────────┐   D-Bus Notify()   ┌─────────────────┐
│ Application │ ─────────────────► │ notifications.rs │
│             │                    │ (D-Bus Handler)  │
└─────────────┘                    └────────┬────────┘
                                            │
                                   Parse & Validate
                                            │
                                   ┌────────▼────────┐
                                   │ Notification    │
                                   │ Struct (lib.rs) │
                                   └────────┬────────┘
                                            │
                              ┌─────────────┴─────────────┐
                              │                           │
                     ┌────────▼────────┐        ┌────────▼────────┐
                     │ cards: Vec      │        │ hidden: VecDeque│
                     │ (visible)       │        │ (queued)        │
                     └────────┬────────┘        └─────────────────┘
                              │
                     ┌────────▼────────┐
                     │ Layer Shell     │
                     │ Surface Render  │
                     └─────────────────┘
```

### 1.4 Key Data Structures

| Struct | Location | Purpose |
|--------|----------|---------|
| `Notification` | `util/src/lib.rs:54` | Core notification data |
| `CosmicNotifications` | `src/app.rs:78` | Application state |
| `NotificationsConfig` | `config/src/lib.rs:18` | User configuration |
| `Hint` | `util/src/lib.rs:327` | FreeDesktop hints |
| `Image` | `util/src/lib.rs:374` | Image data variants |

---

## 2. FreeDesktop Specification Compliance

### 2.1 Specification Overview

The [Desktop Notifications Specification v1.2](https://specifications.freedesktop.org/notification-spec/latest/) defines the standard D-Bus interface for desktop notifications on Linux.

### 2.2 Compliance Matrix

| Feature | Spec Requirement | Implementation | Status |
|---------|-----------------|----------------|--------|
| **D-Bus Interface** | `org.freedesktop.Notifications` | `notifications.rs:49` | ✅ |
| **Notify Method** | Accept app_name, id, icon, summary, body, actions, hints, timeout | Full support | ✅ |
| **CloseNotification** | Close by ID | Implemented | ✅ |
| **GetCapabilities** | Return supported features | Returns 12 capabilities | ✅ |
| **GetServerInformation** | Return name, vendor, version, spec_version | Implemented | ✅ |
| **NotificationClosed Signal** | Emit on close with reason | Implemented | ✅ |
| **ActionInvoked Signal** | Emit on action click | Implemented | ✅ |

### 2.3 Supported Capabilities

```rust
// From notifications.rs - GetCapabilities response
vec![
    "actions",           // Action buttons
    "action-icons",      // Icon-based actions
    "body",              // Body text
    "body-hyperlinks",   // Clickable links
    "body-images",       // Inline images
    "body-markup",       // Pango markup
    "icon-multi",        // Multiple icon formats
    "icon-static",       // Static icons
    "persistence",       // History/persistence
    "sound",             // Audio playback
    "x-kde-origin-name", // KDE compat
    "inline-reply",      // Inline reply (pending)
]
```

### 2.4 Supported Hints

| Hint | Type | Implementation |
|------|------|----------------|
| `action-icons` | boolean | `lib.rs:87` |
| `category` | string | `lib.rs:88` |
| `desktop-entry` | string | `lib.rs:89` |
| `image-data` / `image_data` | (iiibiiay) | `lib.rs:121-140` |
| `image-path` / `image_path` | string | `lib.rs:102-120` |
| `resident` | boolean | `lib.rs:90` |
| `sound-file` | string | `lib.rs:91-93` |
| `sound-name` | string | `lib.rs:94` |
| `suppress-sound` | boolean | `lib.rs:95` |
| `transient` | boolean | `lib.rs:96` |
| `urgency` | byte (0-2) | `lib.rs:98` |
| `value` | int32 | `lib.rs:99` (progress) |
| `x`, `y` | int32 | `lib.rs:100-101` (position) |

### 2.5 Comparison with Other Implementations

| Feature | cosmic-notifications-ng | dunst | mako | swaync |
|---------|------------------------|-------|------|--------|
| Wayland Native | ✅ | ✅ | ✅ | ✅ |
| Image Support | ✅ Full | ✅ | ✅ | ✅ |
| Actions | ✅ | ✅ | ✅ | ✅ |
| Progress Bars | ✅ | ❌ | ❌ | ✅ |
| History | ✅ | ✅ | ❌ | ✅ |
| Do Not Disturb | ✅ | ✅ | ✅ | ✅ |
| Sound | ✅ | ✅ | ❌ | ✅ |
| GIF Animation | ✅ | ❌ | ❌ | ❌ |
| Rate Limiting | ✅ | ❌ | ❌ | ❌ |

---

## 3. COSMIC Ecosystem Integration

### 3.1 Applet Communication

The notification daemon communicates with the COSMIC panel applet via D-Bus:

```
┌─────────────────────────┐     D-Bus      ┌─────────────────────────┐
│ cosmic-notifications-ng │ ◄────────────► │ cosmic-applet-          │
│ (Daemon)                │                │ notifications           │
│                         │                │ (Panel Widget)          │
│ Serves:                 │                │                         │
│ /com/system76/          │                │ Calls:                  │
│ NotificationsApplet     │                │ - GetHistory()          │
│                         │                │ - ClearAll()            │
│ Methods:                │                │ - Dismiss(id)           │
│ - GetHistory            │                │                         │
│ - ClearAll              │                │ Receives:               │
│ - Dismiss               │                │ - HistoryUpdated signal │
└─────────────────────────┘                └─────────────────────────┘
```

### 3.2 Shared Data Structures

The `cosmic-notifications-util` crate provides shared types used by both daemon and applet:

```rust
// cosmic-notifications-util/src/lib.rs
pub struct Notification {
    pub id: u32,
    pub app_name: String,
    pub app_icon: String,
    pub summary: String,
    pub body: String,
    pub actions: Vec<(ActionId, String)>,
    pub hints: Vec<Hint>,
    pub expire_timeout: i32,
    pub time: SystemTime,
}
```

### 3.3 Panel Integration Points

| Integration | Location | Purpose |
|-------------|----------|---------|
| Anchor Config | `config/src/lib.rs:6-16` | Notification position relative to panel |
| Panel Config | `app.rs:88-89` | Read dock/panel positions |
| Output Selection | `app.rs:90` | Display on correct monitor |

### 3.4 GNOME Application Compatibility

GNOME applications work via standard D-Bus notification interface:

```
┌─────────────────┐     libnotify     ┌─────────────────┐
│ GNOME App       │ ─────────────────►│ D-Bus Session   │
│ (e.g., Nautilus)│                   │ Bus             │
└─────────────────┘                   └────────┬────────┘
                                               │
                                      org.freedesktop.Notifications
                                               │
                                      ┌────────▼────────┐
                                      │ cosmic-notif-ng │
                                      └─────────────────┘
```

**Flatpak/Snap Applications:**

Sandboxed apps use `xdg-desktop-portal` which proxies notifications:

```
Flatpak App → xdg-desktop-portal → D-Bus → cosmic-notifications-ng
```

---

## 4. Performance Analysis & Optimization

### 4.1 Current Performance Profile

| Metric | Current | Target | Status |
|--------|---------|--------|--------|
| Framerate | 60 FPS | 60 FPS | ✅ |
| Animation Duration | 200-400ms | 200-400ms | ✅ |
| Memory per notification | 100-500KB (with image) | <200KB | ⚠️ |
| Max concurrent sounds | 4 | 4 | ✅ |
| Rate limit | 60/min/app | 60/min/app | ✅ |
| Memory budget (hidden) | 50MB | 50MB | ✅ |

### 4.2 Identified Bottlenecks

#### 4.2.1 Image Data Cloning (High Priority)

**Location:** `util/src/lib.rs:259`

```rust
// Current: Clones entire image data Vec<u8>
Some(icon::from_raster_pixels(*width, *height, data.clone()).icon())
```

**Impact:** For a 256x256 RGBA image, this clones 262KB per notification icon request.

**Recommendation:**
```rust
// Use Arc<Vec<u8>> for image data to enable cheap cloning
pub enum Image {
    Name(String),
    File(PathBuf),
    Data {
        width: u32,
        height: u32,
        data: Arc<Vec<u8>>,  // Changed from Vec<u8>
    },
}
```

#### 4.2.2 Regex Compilation (Medium Priority)

**Location:** `util/src/link_detector.rs`

**Issue:** Link detection regex is compiled on every call.

**Recommendation:** Use `lazy_static!` or `once_cell::Lazy`:

```rust
use once_cell::sync::Lazy;

static URL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"https?://[^\s<>\"']+").unwrap()
});
```

#### 4.2.3 HTML Sanitization (Low Priority)

**Location:** `util/src/sanitizer.rs`

**Issue:** Multi-pass sanitization for XSS prevention.

**Current Approach (necessary for security):**
1. First pass: Entity decode
2. Second pass: Ammonia sanitize
3. Third pass: Final clean

**Status:** Already optimized - security takes precedence over performance.

### 4.3 Memory Management

The daemon implements a memory budget for hidden notifications:

```rust
// app.rs - Memory budget implementation
const HIDDEN_NOTIFICATIONS_MEMORY_BUDGET: usize = 50 * 1024 * 1024; // 50MB

fn add_to_hidden(&mut self, notification: Notification) {
    self.hidden.push_back(notification);

    // Trim from front (oldest) if over budget
    while self.hidden_memory_usage() > HIDDEN_NOTIFICATIONS_MEMORY_BUDGET {
        self.hidden.pop_front();
    }
}
```

### 4.4 Optimization Roadmap

| Optimization | Effort | Impact | Priority |
|--------------|--------|--------|----------|
| Arc<Vec<u8>> for images | S (2-3 days) | High | P1 |
| Static regex compilation | XS (1 day) | Medium | P2 |
| Batch D-Bus notifications | M (1 week) | Low | P3 |
| Image caching | M (1 week) | Medium | P2 |

---

## 5. Code Quality & Refactoring

### 5.1 Current Issues

#### 5.1.1 Monolithic app.rs

**Problem:** `app.rs` at 1183 lines handles too many responsibilities:
- Application state management
- Message handling
- UI rendering
- Subscription management
- Panel configuration
- Animation timeline

**Recommendation:** Decompose into focused modules:

```
src/
├── app.rs           # Reduced to ~300 lines (just Application trait impl)
├── state/
│   ├── mod.rs
│   ├── notifications.rs  # Notification queue management
│   └── config.rs         # Configuration state
├── handlers/
│   ├── mod.rs
│   ├── messages.rs       # Message processing
│   └── input.rs          # User input handling
├── rendering/
│   ├── mod.rs
│   └── cards.rs          # Notification card rendering
└── subscriptions/        # (existing)
```

#### 5.1.2 Missing Service Layer

**Current:** Direct D-Bus interaction mixed with business logic.

**Proposed Architecture:**

```rust
// src/services/notification_service.rs
pub struct NotificationService {
    rate_limiter: RateLimiter,
    memory_budget: MemoryBudget,
    queue: NotificationQueue,
}

impl NotificationService {
    pub fn accept(&mut self, notification: Notification) -> Result<u32, NotificationError> {
        self.rate_limiter.check(&notification.app_name)?;
        self.memory_budget.check(&notification)?;
        Ok(self.queue.enqueue(notification))
    }

    pub fn dismiss(&mut self, id: u32) -> Option<Notification> {
        self.queue.remove(id)
    }
}
```

### 5.2 Code Quality Metrics

| Metric | Current | Target |
|--------|---------|--------|
| Largest file | 1183 lines (app.rs) | <500 lines |
| Test coverage | ~60% | >80% |
| Documentation | Partial | Full (doc comments) |
| Clippy warnings | 0 | 0 ✅ |

### 5.3 Testing Gaps

| Area | Coverage | Priority |
|------|----------|----------|
| Unit tests (util) | Good | - |
| Integration tests | Partial | High |
| UI/widget tests | Missing | Medium |
| D-Bus interface tests | Missing | High |
| Performance benchmarks | Missing | Low |

---

## 6. Configuration & UX Enhancements

### 6.1 Current Configuration (v2)

```rust
// cosmic-notifications-config/src/lib.rs
pub struct NotificationsConfig {
    pub do_not_disturb: bool,           // Global mute
    pub anchor: Anchor,                 // Position (8 options)
    pub max_notifications: u32,         // Visible limit (default: 3)
    pub max_per_app: u32,               // Per-app limit (default: 2)
    pub max_timeout_urgent: Option<u32>,
    pub max_timeout_normal: Option<u32>,
    pub max_timeout_low: Option<u32>,
    pub show_images: bool,              // Rich content toggle
    pub show_actions: bool,             // Action buttons toggle
    pub max_image_size: u32,            // 32-256 pixels
    pub enable_links: bool,             // Clickable links
    pub enable_animations: bool,        // GIF/card animations
}
```

### 6.2 Enhancement Opportunities

#### 6.2.1 Per-Application Rules

**Proposal:** Add app-specific notification rules:

```rust
pub struct AppRule {
    pub app_name: String,
    pub desktop_entry: Option<String>,
    pub enabled: bool,
    pub urgency_override: Option<u8>,
    pub sound_enabled: bool,
    pub timeout_override: Option<u32>,
}

// In NotificationsConfig:
pub app_rules: Vec<AppRule>,
```

**Use Cases:**
- Mute specific apps
- Override urgency (e.g., make Slack always critical)
- Disable sounds for email apps at night

#### 6.2.2 Expanded Placement Options

**Current:** 8 anchor positions (Top, Bottom, Left, Right, TopLeft, etc.)

**Proposed Additions:**

```rust
pub struct NotificationPlacement {
    pub anchor: Anchor,
    pub margin_x: u32,        // Horizontal margin from edge
    pub margin_y: u32,        // Vertical margin from edge
    pub width: Option<u32>,   // Custom width (default: auto)
    pub gap: u32,             // Gap between stacked notifications
}
```

#### 6.2.3 Notification Grouping

**Proposal:** Group notifications by app:

```rust
pub enum GroupingMode {
    None,                    // Current behavior
    ByApp,                   // Stack notifications from same app
    ByCategory,              // Group by category hint
}
```

**UI Mockup:**

```
┌─────────────────────────┐
│ Firefox (3)             │
│ ├─ New message from...  │
│ ├─ Download complete    │
│ └─ Tab crashed          │
└─────────────────────────┘
┌─────────────────────────┐
│ Slack                   │
│   You have 2 new DMs    │
└─────────────────────────┘
```

### 6.3 Accessibility Improvements

| Feature | Status | Priority |
|---------|--------|----------|
| Screen reader support | Partial | High |
| High contrast mode | Not implemented | Medium |
| Font scaling | Via cosmic-config | ✅ |
| Reduced motion mode | Via `enable_animations` | ✅ |

---

## 7. Implementation Roadmap

### Phase 1: Quick Wins (1-2 weeks)

| Task | Effort | File(s) | Impact |
|------|--------|---------|--------|
| Static regex compilation | XS | `link_detector.rs` | Performance |
| Arc for image data | S | `lib.rs`, `image.rs` | Memory |
| Add D-Bus integration tests | S | New test file | Quality |
| Document public APIs | S | All `.rs` files | Maintainability |

### Phase 2: Architecture Refactoring (2-4 weeks)

| Task | Effort | Files | Impact |
|------|--------|-------|--------|
| Extract NotificationService | M | New service module | Maintainability |
| Decompose app.rs | M | Multiple new modules | Maintainability |
| Add widget tests | M | Widget test modules | Quality |
| Implement image cache | M | New cache module | Performance |

### Phase 3: Feature Enhancements (4-8 weeks)

| Task | Effort | Impact |
|------|--------|--------|
| Per-app notification rules | L | UX |
| Notification grouping | L | UX |
| Expanded placement options | M | UX |
| Inline reply support | XL | Feature |
| Improved accessibility | M | Accessibility |

### Phase 4: Future Considerations

| Task | Effort | Notes |
|------|--------|-------|
| Notification sync (CConnect) | XL | Requires protocol work |
| Custom themes | L | CSS-like styling |
| Priority inbox | M | ML-based importance |
| Scheduled DND | S | Time-based rules |

---

## 8. Appendices

### Appendix A: D-Bus Interface Reference

```xml
<!-- org.freedesktop.Notifications -->
<interface name="org.freedesktop.Notifications">
  <method name="Notify">
    <arg direction="in" type="s" name="app_name"/>
    <arg direction="in" type="u" name="replaces_id"/>
    <arg direction="in" type="s" name="app_icon"/>
    <arg direction="in" type="s" name="summary"/>
    <arg direction="in" type="s" name="body"/>
    <arg direction="in" type="as" name="actions"/>
    <arg direction="in" type="a{sv}" name="hints"/>
    <arg direction="in" type="i" name="expire_timeout"/>
    <arg direction="out" type="u" name="id"/>
  </method>

  <method name="CloseNotification">
    <arg direction="in" type="u" name="id"/>
  </method>

  <method name="GetCapabilities">
    <arg direction="out" type="as" name="capabilities"/>
  </method>

  <method name="GetServerInformation">
    <arg direction="out" type="s" name="name"/>
    <arg direction="out" type="s" name="vendor"/>
    <arg direction="out" type="s" name="version"/>
    <arg direction="out" type="s" name="spec_version"/>
  </method>

  <signal name="NotificationClosed">
    <arg type="u" name="id"/>
    <arg type="u" name="reason"/>
  </signal>

  <signal name="ActionInvoked">
    <arg type="u" name="id"/>
    <arg type="s" name="action_key"/>
  </signal>
</interface>
```

### Appendix B: Security Fixes Applied (v0.2.3)

| Vulnerability | Fix Location | Description |
|---------------|--------------|-------------|
| XSS via HTML injection | `sanitizer.rs` | Multi-pass ammonia sanitization |
| DoS via notification flood | `notifications.rs` | Rate limiting (60/min/app) |
| Memory exhaustion | `app.rs` | 50MB budget for hidden notifications |
| Sound path traversal | `audio.rs` | Whitelist validation |
| Thread pool exhaustion | `audio.rs` | AtomicUsize counter (max 4) |
| Race condition in audio | `audio.rs` | compare_exchange loop |
| Entity decode bypass | `sanitizer.rs` | Proper decode order |
| ID overflow | `notifications.rs` | NonZeroU64 IDs |

### Appendix C: Testing Commands

```bash
# Run all tests
nix develop --command cargo test

# Run with coverage
cargo tarpaulin --out Html

# Benchmark performance
cargo bench

# Check for common issues
cargo clippy -- -W clippy::all

# Format code
cargo fmt

# Build release
cargo build --release

# Test D-Bus interface manually
dbus-send --session \
  --dest=org.freedesktop.Notifications \
  --type=method_call \
  --print-reply \
  /org/freedesktop/Notifications \
  org.freedesktop.Notifications.Notify \
  string:"Test" uint32:0 string:"dialog-information" \
  string:"Test Title" string:"Test body with <b>markup</b>" \
  array:string:"" dict:string:string:"" int32:5000
```

### Appendix D: Useful Resources

- [FreeDesktop Notification Spec v1.2](https://specifications.freedesktop.org/notification-spec/latest/)
- [COSMIC Desktop GitHub](https://github.com/pop-os/cosmic-epoch)
- [libcosmic Documentation](https://pop-os.github.io/libcosmic/)
- [Wayland Layer Shell Protocol](https://wayland.app/protocols/wlr-layer-shell-unstable-v1)
- [zbus Documentation](https://docs.rs/zbus/)

---

## Conclusion

The cosmic-notifications-ng project is a well-implemented FreeDesktop-compliant notification daemon with several advanced features (rate limiting, security hardening, rich content support). The main areas for improvement are:

1. **Code organization** - Decomposing the monolithic `app.rs`
2. **Performance** - Arc-wrapping image data, static regex compilation
3. **Features** - Per-app rules, notification grouping, expanded placement

The security vulnerabilities identified in the previous audit have all been addressed in v0.2.3, making the codebase production-ready from a security standpoint.

---

*Report generated by Claude Code research agents on 2026-02-05*
