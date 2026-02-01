#!/usr/bin/env bash

# Test script for COSMIC Rich Notifications
# Tests various notification types to verify rich notification functionality
# Note: Script continues even if individual notifications fail

echo "=== COSMIC Rich Notifications Test Suite ==="
echo ""

# Check if notify-send is available
if ! command -v notify-send &> /dev/null; then
    echo "Error: notify-send not found. Please install libnotify."
    exit 1
fi

# Check if notification service is running
if ! dbus-send --session --print-reply --dest=org.freedesktop.Notifications \
    /org/freedesktop/Notifications org.freedesktop.Notifications.GetServerInformation &>/dev/null; then
    echo "Warning: No notification service detected. Starting tests anyway..."
    echo ""
fi

echo "Testing notifications..."
sleep 1

# Test 1: Basic notification
echo "[1/12] Basic text notification"
notify-send "Basic Notification" "This is a simple notification with just text." 2>/dev/null && echo "    ✓ Sent" || echo "    ✗ Failed"
sleep 2

# Test 2: Notification with icon
echo "[2/12] Notification with icon"
notify-send -i dialog-information "Info Notification" "This notification has an info icon." 2>/dev/null && echo "    ✓ Sent" || echo "    ✗ Failed"
sleep 2

# Test 3: Low urgency notification
echo "[3/12] Low urgency notification"
notify-send -u low "Low Urgency" "This notification has low urgency." 2>/dev/null && echo "    ✓ Sent" || echo "    ✗ Failed"
sleep 2

# Test 4: Normal urgency notification
echo "[4/12] Normal urgency notification"
notify-send -u normal "Normal Urgency" "This notification has normal urgency (default)." 2>/dev/null && echo "    ✓ Sent" || echo "    ✗ Failed"
sleep 2

# Test 5: Critical urgency notification
echo "[5/12] Critical urgency notification"
notify-send -u critical "Critical Urgency" "This notification has critical urgency!" 2>/dev/null && echo "    ✓ Sent" || echo "    ✗ Failed"
sleep 2

# Test 6: Notification with longer body
echo "[6/12] Notification with body text"
notify-send "Summary Title" "This is the body text. It can contain more details about what happened." 2>/dev/null && echo "    ✓ Sent" || echo "    ✗ Failed"
sleep 2

# Test 7: Notification with action buttons
# Note: Uses timeout because --action waits for user interaction
echo "[7/12] Notification with action buttons (3s timeout)"
timeout 3 notify-send "Action Test" "Click a button within 3 seconds." \
    --action="reply=Reply" \
    --action="mark-read=Mark Read" 2>/dev/null
echo "    ✓ Sent (timeout elapsed)"
sleep 1

# Test 8: Notification with links in body
echo "[8/12] Notification with clickable links"
notify-send "Link Test" "Check out https://github.com/pop-os/cosmic-notifications - links should be clickable!" 2>/dev/null && echo "    ✓ Sent" || echo "    ✗ Failed"
sleep 2

# Test 9: Notification with category (email)
echo "[9/12] Notification with category (email)"
notify-send -c email "New Email" "You have a new message from test@example.com" 2>/dev/null && echo "    ✓ Sent" || echo "    ✗ Failed"
sleep 2

# Test 10: Notification with category (im)
echo "[10/12] Notification with category (instant message)"
notify-send -c im.received "New Message" "Alice: Hey, are you there?" 2>/dev/null && echo "    ✓ Sent" || echo "    ✗ Failed"
sleep 2

# Test 11: Notification with image
echo "[11/12] Notification with image"
if [ -f "/usr/share/icons/hicolor/48x48/apps/firefox.png" ]; then
    notify-send -i /usr/share/icons/hicolor/48x48/apps/firefox.png "Image Test" "This has a Firefox icon." 2>/dev/null && echo "    ✓ Sent" || echo "    ✗ Failed"
elif [ -f "/usr/share/pixmaps/debian-logo.png" ]; then
    notify-send -i /usr/share/pixmaps/debian-logo.png "Image Test" "This has a Debian logo." 2>/dev/null && echo "    ✓ Sent" || echo "    ✗ Failed"
else
    notify-send -i dialog-information "Image Test" "Using fallback icon." 2>/dev/null && echo "    ✓ Sent (fallback)" || echo "    ✗ Failed"
fi
sleep 2

# Test 12: Transient notification
echo "[12/12] Transient notification (3s timeout)"
notify-send -t 3000 "Transient" "This notification expires in 3 seconds." 2>/dev/null && echo "    ✓ Sent" || echo "    ✗ Failed"
sleep 4

echo ""
echo "=== Basic tests complete ==="
echo ""
echo "For advanced tests (progress bars, image data, animated GIFs),"
echo "use DBus directly or applications that support the full"
echo "freedesktop notification specification."
echo ""
echo "Example DBus progress notification:"
echo '  gdbus call --session --dest org.freedesktop.Notifications \'
echo '    --object-path /org/freedesktop/Notifications \'
echo '    --method org.freedesktop.Notifications.Notify \'
echo '    "TestApp" 0 "" "Download" "45% complete" "[]" "{\"value\": <45>}" 5000'
echo ""
