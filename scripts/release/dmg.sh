#!/bin/bash
# Create beautiful DMG installer for GoNhanh
# macOS standard style with drag-to-install visual guidance

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$SCRIPT_DIR/.."
MACOS_DIR="$PROJECT_ROOT/platforms/macos"
DMG_RESOURCES="$MACOS_DIR/dmg-resources"

# Configuration
APP_NAME="GoNhanh"
VOL_NAME="GoNhanh"
DMG_TEMP="$MACOS_DIR/build/${APP_NAME}_temp.dmg"
DMG_FINAL="$MACOS_DIR/build/${APP_NAME}.dmg"

# Window dimensions and positions (matching background.png)
WINDOW_WIDTH=540
WINDOW_HEIGHT=360
ICON_SIZE=100
APP_X=135
APP_Y=160
APPS_X=405
APPS_Y=160

# Check arguments
APP_PATH="${1:-$MACOS_DIR/build/Release/${APP_NAME}.app}"

if [ ! -d "$APP_PATH" ]; then
    echo "Error: App not found at $APP_PATH"
    echo "Usage: $0 [path/to/GoNhanh.app]"
    exit 1
fi

echo "📦 Creating DMG installer for GoNhanh..."
echo "   App: $APP_PATH"

# Clean up any existing DMG
rm -f "$DMG_TEMP" "$DMG_FINAL"

# Create staging directory
STAGING_DIR=$(mktemp -d)
trap "rm -rf $STAGING_DIR" EXIT

echo "   Staging directory: $STAGING_DIR"

# Copy app to staging
cp -R "$APP_PATH" "$STAGING_DIR/"

# Create Applications symlink
ln -s /Applications "$STAGING_DIR/Applications"

# Copy background if exists
if [ -f "$DMG_RESOURCES/background.tiff" ]; then
    mkdir -p "$STAGING_DIR/.background"
    cp "$DMG_RESOURCES/background.tiff" "$STAGING_DIR/.background/background.tiff"
    BACKGROUND_FILE="background.tiff"
elif [ -f "$DMG_RESOURCES/background.png" ]; then
    mkdir -p "$STAGING_DIR/.background"
    cp "$DMG_RESOURCES/background.png" "$STAGING_DIR/.background/background.png"
    BACKGROUND_FILE="background.png"
else
    BACKGROUND_FILE=""
    echo "   Warning: No background image found"
fi

# Calculate size needed (app size + 50MB buffer)
APP_SIZE=$(du -sm "$APP_PATH" | cut -f1)
DMG_SIZE=$((APP_SIZE + 50))
echo "   DMG size: ${DMG_SIZE}MB"

# Create temporary DMG
echo "   Creating temporary DMG..."
hdiutil create -srcfolder "$STAGING_DIR" \
    -volname "$VOL_NAME" \
    -fs HFS+ \
    -fsargs "-c c=64,a=16,e=16" \
    -format UDRW \
    -size ${DMG_SIZE}m \
    "$DMG_TEMP"

# Mount the DMG
echo "   Mounting DMG..."
MOUNT_DIR=$(hdiutil attach -readwrite -noverify -noautoopen "$DMG_TEMP" | grep "/Volumes/$VOL_NAME" | awk '{print $3}')

if [ -z "$MOUNT_DIR" ]; then
    # Try alternative parsing
    MOUNT_DIR="/Volumes/$VOL_NAME"
fi

echo "   Mounted at: $MOUNT_DIR"

# Wait for mount
sleep 2

# Apply AppleScript to customize the DMG window
echo "   Customizing DMG appearance..."

if [ -n "$BACKGROUND_FILE" ]; then
    BACKGROUND_CLAUSE="set background picture of viewOptions to file \".background:$BACKGROUND_FILE\""
else
    BACKGROUND_CLAUSE=""
fi

osascript <<EOF
tell application "Finder"
    tell disk "$VOL_NAME"
        open
        set current view of container window to icon view
        set toolbar visible of container window to false
        set statusbar visible of container window to false
        set the bounds of container window to {100, 100, $((100 + WINDOW_WIDTH)), $((100 + WINDOW_HEIGHT))}
        set viewOptions to the icon view options of container window
        set arrangement of viewOptions to not arranged
        set icon size of viewOptions to $ICON_SIZE
        $BACKGROUND_CLAUSE
        set position of item "${APP_NAME}.app" of container window to {$APP_X, $APP_Y}
        set position of item "Applications" of container window to {$APPS_X, $APPS_Y}
        close
        open
        update without registering applications
        delay 2
    end tell
end tell
EOF

# Sync and unmount
echo "   Finalizing..."
sync
sleep 3
hdiutil detach "$MOUNT_DIR" -quiet 2>/dev/null || hdiutil detach "$MOUNT_DIR" -force 2>/dev/null || true
sleep 2

# Convert to compressed DMG
echo "   Compressing DMG..."
hdiutil convert "$DMG_TEMP" \
    -format UDZO \
    -imagekey zlib-level=9 \
    -o "$DMG_FINAL"

# Clean up temp DMG
rm -f "$DMG_TEMP"

# Get final size
FINAL_SIZE=$(du -h "$DMG_FINAL" | cut -f1)

echo ""
echo "✅ DMG created successfully!"
echo "   Output: $DMG_FINAL"
echo "   Size: $FINAL_SIZE"
echo ""
echo "To test the DMG:"
echo "   open $DMG_FINAL"
