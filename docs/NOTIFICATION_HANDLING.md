# Notification Handling

This document describes how cosmic-ext-notifications handles incoming notifications, from D-Bus reception through display.

## D-Bus Interface

cosmic-ext-notifications implements the [FreeDesktop Desktop Notifications Specification](https://specifications.freedesktop.org/notification-spec/latest/) version 1.2.

### Service Registration

The daemon registers on the session bus at:

- **Bus name:** `org.freedesktop.Notifications`
- **Object path:** `/org/freedesktop/Notifications`

### Advertised Capabilities

```
body            - Supports body text
icon-static     - Displays single-frame notification icons
persistence     - Retains notifications until acknowledged
actions         - Supports action buttons
action-icons    - Supports icon-based actions
body-markup     - Supports HTML markup in body text
body-hyperlinks - Supports clickable links in body
sound           - Supports sound notifications
```

### Core Methods

| Method | Description |
|--------|-------------|
| `Notify(...)` | Display a notification, returns notification ID |
| `CloseNotification(id)` | Close a notification by ID |
| `GetCapabilities()` | Return list of supported features |
| `GetServerInformation()` | Return server name, vendor, version, spec version |

### Signals

| Signal | Description |
|--------|-------------|
| `ActionInvoked(id, action_key)` | Emitted when user clicks an action |
| `ActivationToken(id, token)` | Emitted with XDG activation token for launching apps |
| `NotificationClosed(id, reason)` | Emitted when notification closes |

Close reasons: 1=expired, 2=dismissed, 3=closed via API, 4=undefined

## Body Text Processing

Notification bodies undergo multi-stage processing for safe display.

### HTML Sanitization

Uses the [ammonia](https://crates.io/crates/ammonia) library for secure HTML handling.

**Allowed tags:** `<b>`, `<i>`, `<u>`, `<a>`, `<br>`, `<p>`

**Allowed attributes:** `href` (on `<a>` only)

**Allowed URL schemes:** `http://`, `https://`, `mailto:`

**Stripped content:**
- `<script>`, `<style>`, `<iframe>`, `<object>`, `<embed>`, `<img>`, `<video>`, `<audio>`
- Event handlers (`onclick`, `onerror`, `onload`, etc.)
- Dangerous URL schemes (`javascript:`, `data:`, `vbscript:`)

**Security additions:**
- All links receive `rel="noopener noreferrer"` automatically

### Entity Decoding

HTML entities are decoded before processing to handle entity-encoded content (common from browser notifications):

| Entity | Character |
|--------|-----------|
| `&lt;` | `<` |
| `&gt;` | `>` |
| `&quot;` | `"` |
| `&amp;` | `&` |
| `&#58;` / `&#x3A;` | `:` |
| `&#39;` / `&#x27;` | `'` |

### Link Extraction

Links are extracted from two sources:

1. **`<a href="...">` tags** - Parsed from HTML, URL and display text extracted
2. **Plain text URLs** - Detected via [linkify](https://crates.io/crates/linkify), emails auto-prefixed with `mailto:`

All extracted links are validated against safe URL schemes before display.

## Supported Hints

Hints are optional key-value pairs providing additional notification metadata.

### Standard Hints

| Hint | Type | Description |
|------|------|-------------|
| `urgency` | byte (0-2) | 0=low, 1=normal (default), 2=critical |
| `category` | string | Notification type (see categories below) |
| `desktop-entry` | string | Desktop file basename for app identification |
| `transient` | boolean | If true, notification is not persisted to history |
| `resident` | boolean | Notification stays after action invoked |
| `sender-pid` | uint32 | PID of sending process |

### Sound Hints

| Hint | Type | Description |
|------|------|-------------|
| `sound-file` | string | Path to sound file to play |
| `sound-name` | string | Sound theme sound name |
| `suppress-sound` | boolean | Disable sound for this notification |

### Image Hints

Image hints are processed in priority order:

1. `image-data` / `image_data` / `icon_data` - Raw RGBA pixel data (structure)
2. `image-path` / `image_path` - File path or icon name

### Progress Hint

| Hint | Type | Description |
|------|------|-------------|
| `value` | int32 | Progress value 0-100, displayed as progress bar |

### Position Hints

| Hint | Type | Description |
|------|------|-------------|
| `x` | int32 | Suggested X position |
| `y` | int32 | Suggested Y position |

### Action Icons Hint

| Hint | Type | Description |
|------|------|-------------|
| `action-icons` | boolean | Use icons instead of text for action buttons |

## Rich Content Support

### Images

Images from hints are processed for display:

- **Maximum dimensions:** 128x128 pixels
- **Aspect ratio:** Preserved during resize
- **Resize algorithm:** Lanczos3 (high quality)
- **Output format:** RGBA

Supported sources:
- Raw pixel data (RGBA or RGB with rowstride)
- File paths (PNG, JPEG, etc. via image crate)
- Icon names (resolved via icon theme)
- `file://` URLs (converted to paths)

### Progress Bars

When the `value` hint is present (0-100), a progress bar is displayed. Values are clamped to valid range.

### Actions

Actions are parsed from the D-Bus array format (alternating id/label pairs):

```
["default", "Open", "reply", "Reply", "dismiss", "Dismiss"]
```

Special handling:
- `default` action - Triggered on notification click, not shown as button
- Other actions - Displayed as buttons
- Action icons - When `action-icons` hint is true, icons replace button text

## Urgency Levels

| Level | Value | Visual Style |
|-------|-------|--------------|
| Low | 0 | Muted gray accent, reduced opacity |
| Normal | 1 | Blue accent (default) |
| Critical | 2 | Red accent, may bypass DND |

## Transient Notifications

### Overview

Transient notifications are temporary notifications that are displayed on screen but **not persisted to the notification history**. This behavior is defined by the [FreeDesktop Desktop Notifications Specification](https://specifications.freedesktop.org/notification-spec/latest/) and is intentional, not a bug.

### How It Works

When an application sends a notification with `transient=true` in the hints:

1. **Displayed normally** - The notification appears on screen like any other
2. **Not sent to applet** - The notification applet is not notified of its arrival
3. **Not in history** - The notification does not appear in the notification history panel
4. **Ephemeral by design** - Once dismissed or timed out, it's gone

### Applications Using Transient Hints

Several applications use transient notifications for temporary, non-essential alerts:

| Application | Typical Use Case |
|-------------|------------------|
| **Google Chrome / Chromium** | Web notifications, tab alerts |
| **Firefox** | Web notifications |
| **Media players** | Track change notifications |
| **Volume/brightness OSD** | Momentary feedback indicators |

### Why Chrome Notifications Don't Appear in History

Google Chrome sends web notifications with the `transient=true` hint. This is Chrome's deliberate choice to:

- Avoid cluttering the system notification history with ephemeral web content
- Respect user privacy (web notifications may contain sensitive content)
- Follow the notification spec's guidance for temporary alerts

**This is expected behavior.** If you need Chrome notifications in history, this would require Chrome to change its notification sending behavior, not a change to this daemon.

### Technical Details

The transient check occurs in `src/subscriptions/notifications.rs`:

```rust
if !n.transient() {
    // Only notify applet about non-transient notifications
    NotificationsApplet::notify(...);
}
```

This ensures:
- Transient notifications are shown to the user immediately
- The notification applet only tracks persistent notifications
- System resources aren't used storing temporary alerts

### Sending Transient Notifications

To send a transient notification via D-Bus:

```bash
gdbus call --session \
  --dest org.freedesktop.Notifications \
  --object-path /org/freedesktop/Notifications \
  --method org.freedesktop.Notifications.Notify \
  "app-name" 0 "dialog-information" "Summary" "Body text" \
  '[]' '{"transient": <true>}' 5000
```

### Checking if a Notification is Transient

Applications can determine if a notification was marked transient by examining the hints dictionary in the `Notify` method call for the key `transient` with a boolean `true` value.

## Categories

The [freedesktop notification categories](https://specifications.freedesktop.org/notification-spec/latest/categories.html) help determine appropriate icons and handling.

### Messaging Categories

- `email` / `email.arrived` - Email notifications
- `im` / `im.received` - Instant message notifications

### System Categories

- `device` / `device.added` / `device.removed` - Device events
- `network` / `network.connected` / `network.disconnected` - Network status
- `presence` / `presence.online` / `presence.offline` - User presence

### Transfer Categories

- `transfer` / `transfer.complete` - File transfers
- `transfer.error` - Transfer failures

## Security Features

### Input Validation

- Negative/invalid image dimensions rejected
- Insufficient image data detected and rejected
- Notification ID overflow handled (wraps to 1)
- Close reasons validated against spec values

### XSS Prevention

- Script injection blocked at HTML sanitization layer
- Event handlers stripped from all elements
- Only safe URL schemes allowed in links
- Entity encoding handled to prevent bypass

### URL Safety

URLs are validated before opening:
- Only `http://`, `https://`, `mailto:` schemes allowed
- `javascript:`, `data:`, `file://`, `vbscript:` blocked
- Invalid URLs rejected with error

### Link Security

All sanitized links automatically receive:

```html
<a href="..." rel="noopener noreferrer">
```

This prevents:
- `window.opener` access from opened pages
- Referrer leakage to external sites

## Protocol Reference

For complete specification details, see:
- [Desktop Notifications Specification](https://specifications.freedesktop.org/notification-spec/latest/)
- [Notification Categories](https://specifications.freedesktop.org/notification-spec/latest/categories.html)
