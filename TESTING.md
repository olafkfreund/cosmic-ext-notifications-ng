# Testing COSMIC Rich Notifications

This document describes how to test the rich notification features with real applications and tools.

## Quick Start

Run the automated test script:

```bash
./scripts/test_rich_notifications.sh
```

This will send 12 different notification types to verify basic functionality.

## Testing with Real Applications

### Firefox

Firefox sends rich notifications for various events:

#### Download Notifications
1. Start a download in Firefox
2. **Expected behavior:**
   - Notification shows with Firefox icon
   - Progress bar updates as download progresses
   - Body text shows filename and size
   - Action buttons: "Open" and "Show in Folder"

#### Media Notifications
1. Play a video or audio on a website
2. **Expected behavior:**
   - Notification shows with media controls
   - Action buttons for play/pause
   - Progress indicator for playback position

**Compatibility notes:**
- Firefox uses standard freedesktop.org notification spec
- Supports image-data hints for custom icons
- Action buttons work correctly
- Progress hints are sent for downloads

### Thunderbird

Thunderbird sends notifications for email and calendar events:

#### Email Notifications
1. Receive a new email
2. **Expected behavior:**
   - Notification shows sender name and subject
   - Category hint: `email.arrived`
   - Action buttons: "Read" and "Delete"
   - Shows sender avatar if available (via image-data hint)

#### Calendar Reminders
1. Create a calendar event with a reminder
2. **Expected behavior:**
   - Notification shows event title and time
   - Category hint: `calendar`
   - Action buttons: "Dismiss" and "Snooze"
   - Urgency level: normal or critical for imminent events

**Compatibility notes:**
- Thunderbird supports full notification spec
- Sends avatar images via image-data hint
- Uses category hints for email/calendar
- Action buttons fully functional

### Spotify

Spotify (via MPRIS) sends media notifications:

#### Now Playing Notifications
1. Play a song in Spotify
2. **Expected behavior:**
   - Notification shows album art
   - Song title and artist in body
   - Action buttons: "Previous", "Play/Pause", "Next"
   - Updates when track changes

**Compatibility notes:**
- Spotify uses MPRIS D-Bus interface
- Album art sent via image-path hint
- Action buttons for media controls
- Notifications update on track change

### System Notifications

#### Battery Notifications
1. Unplug laptop or let battery drain to 20%
2. **Expected behavior:**
   - Notification shows battery icon
   - Category hint: `device.battery`
   - Urgency: critical when low
   - Body shows battery percentage

#### Network Notifications
1. Connect/disconnect WiFi or Ethernet
2. **Expected behavior:**
   - Notification shows network icon
   - Category hint: `network.connected` or `network.disconnected`
   - Body shows network name
   - Urgency: normal

#### Volume/Brightness Notifications
1. Change volume or brightness
2. **Expected behavior:**
   - Transient notification (auto-dismisses)
   - Progress bar shows current level
   - Updates smoothly as you adjust
   - No action buttons

**Compatibility notes:**
- System notifications use standard hints
- Category hints properly set
- Urgency levels appropriate
- Transient flag set for temporary notifications

## Testing Advanced Features

### Animated GIF Notifications

Test animated image support:

```bash
# Using a local GIF file
notify-send -i /path/to/animation.gif "Animated Test" "This should show an animated GIF"
```

Or via D-Bus:

```bash
dbus-send --session --print-reply \
  --dest=org.freedesktop.Notifications \
  /org/freedesktop/Notifications \
  org.freedesktop.Notifications.Notify \
  string:"TestApp" \
  uint32:0 \
  string:"" \
  string:"Animated GIF Test" \
  string:"Testing animation support" \
  array:string: \
  dict:string:variant:"image-path",variant:string:"/path/to/animation.gif" \
  int32:5000
```

**Expected behavior:**
- GIF plays in notification
- Animation loops continuously
- Frame rate matches original GIF
- Memory usage stays reasonable (<100MB)

**Known limitations:**
- Maximum 100 frames
- Maximum 30 seconds duration
- Very large GIFs may be rejected

### Progress Bar Notifications

Test progress indicators:

```bash
# Send progress notification via D-Bus
dbus-send --session --print-reply \
  --dest=org.freedesktop.Notifications \
  /org/freedesktop/Notifications \
  org.freedesktop.Notifications.Notify \
  string:"TestApp" \
  uint32:0 \
  string:"" \
  string:"Download Progress" \
  string:"Downloading large-file.iso" \
  array:string: \
  dict:string:variant:"value",variant:int32:45 \
  int32:5000
```

**Expected behavior:**
- Progress bar shows at 45%
- Smooth animation when value updates
- Color matches theme
- Updates when notification is replaced with same ID

### Link Detection

Test clickable links in notification body:

```bash
notify-send "Link Test" "Visit https://github.com/pop-os/cosmic for more information"
```

**Expected behavior:**
- Link is detected and styled differently
- Link is clickable (opens in default browser)
- Only http:// and https:// URLs are clickable
- Malicious URLs (javascript:, file:, etc.) are blocked

### Multiple Action Buttons

Test notifications with many actions:

```bash
# Via D-Bus (notify-send doesn't support multiple actions well)
dbus-send --session --print-reply \
  --dest=org.freedesktop.Notifications \
  /org/freedesktop/Notifications \
  org.freedesktop.Notifications.Notify \
  string:"TestApp" \
  uint32:0 \
  string:"dialog-question" \
  string:"Multiple Actions" \
  string:"Choose an action:" \
  array:string:"action1","Reply","action2","Forward","action3","Delete","action4","Archive" \
  dict:string:variant: \
  int32:5000
```

**Expected behavior:**
- First 3 actions shown as buttons
- Additional actions accessible via overflow menu (if implemented)
- Buttons styled with theme
- Clicking button invokes action and dismisses notification

### HTML Sanitization

Test that malicious HTML is properly sanitized:

```bash
# This should be safe - HTML is sanitized
notify-send "HTML Test" "<script>alert('xss')</script><b>Bold text</b>"
```

**Expected behavior:**
- Script tags are removed
- Bold formatting is preserved
- No JavaScript execution
- Safe HTML tags (b, i, u, a) work correctly

## Manual Testing Checklist

- [ ] Basic text notifications display correctly
- [ ] Urgency levels show different colors
- [ ] Images display from file paths
- [ ] Images display from image-data hints
- [ ] Animated GIFs play correctly
- [ ] Progress bars update smoothly
- [ ] Links are clickable and safe
- [ ] Action buttons work and dismiss notification
- [ ] Category icons display correctly
- [ ] Transient notifications auto-dismiss
- [ ] Multiple notifications stack correctly
- [ ] Animations are smooth (30fps)
- [ ] Memory usage is reasonable
- [ ] Configuration options work (show_images, show_actions, etc.)

## Performance Testing

### Memory Usage

Monitor memory usage with multiple notifications:

```bash
# Send 10 notifications with images
for i in {1..10}; do
  notify-send -i dialog-information "Test $i" "Notification with icon"
done

# Check memory usage
ps aux | grep cosmic-ext-notifications
```

**Expected:**
- Memory usage < 100MB with 10 notifications
- Memory released when notifications expire
- No memory leaks over time

### Animation Performance

Test animation frame rate:

```bash
# Send animated GIF notification
notify-send -i /path/to/large-animation.gif "Animation Test" "Performance test"

# Monitor CPU usage
top -p $(pgrep cosmic-ext-notifications)
```

**Expected:**
- 30 FPS animation
- CPU usage < 5% for single animation
- No frame drops or stuttering

## Troubleshooting

### Notifications not appearing
- Check if cosmic-ext-notifications daemon is running: `pgrep cosmic-ext-notifications`
- Check logs: `journalctl -u cosmic-ext-notifications -f`
- Verify D-Bus service: `dbus-send --session --print-reply --dest=org.freedesktop.DBus / org.freedesktop.DBus.ListNames | grep Notifications`

### Images not displaying
- Check `show_images` config option
- Verify image file exists and is readable
- Check file format is supported (PNG, JPEG, GIF)
- Check image size is not too large (max 128x128 by default)

### Actions not working
- Check `show_actions` config option
- Verify application sends actions in correct format (pairs of id, label)
- Check D-Bus signal handlers are registered

### Links not clickable
- Check `enable_links` config option
- Verify URL is http:// or https://
- Check default browser is set

### Animations not playing
- Check `enable_animations` config option
- Verify GIF is not too large (>100 frames or >30s)
- Check memory usage is not hitting limits

## Reporting Issues

When reporting notification issues, please include:

1. Steps to reproduce
2. Expected behavior
3. Actual behavior
4. Screenshots/recordings if possible
5. Application name and version
6. Notification D-Bus message (if available)
7. Logs from `journalctl -u cosmic-ext-notifications`

Example D-Bus message capture:

```bash
dbus-monitor --session "interface='org.freedesktop.Notifications'" > notification-log.txt
# Reproduce the issue
# Check notification-log.txt
```
