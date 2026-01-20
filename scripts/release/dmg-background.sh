#!/bin/bash
# Create simple DMG background image for GoNhanh installer
# Clean, minimal design with dashed border for drop zone

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
OUTPUT_DIR="$SCRIPT_DIR/../platforms/macos/dmg-resources"
mkdir -p "$OUTPUT_DIR"

# DMG window dimensions
WIDTH=540
HEIGHT=360

# Icon positions (center of each icon) - vertically centered
APP_X=135
APP_Y=160
APPS_X=405
APPS_Y=160

# Dashed rectangle around Applications folder position
RECT_SIZE=145
RECT_X=$((APPS_X - RECT_SIZE/2))
RECT_Y=$((APPS_Y - RECT_SIZE/2 - 5))
RECT_X2=$((RECT_X + RECT_SIZE))
RECT_Y2=$((RECT_Y + RECT_SIZE + 20))
CORNER_RADIUS=12

# Check if ImageMagick is available
if command -v magick &> /dev/null; then
    CONVERT="magick"
elif command -v convert &> /dev/null; then
    CONVERT="convert"
else
    echo "ImageMagick is required. Install with: brew install imagemagick"
    exit 1
fi

echo "Creating DMG background image..."

# Create transparent background
$CONVERT -size ${WIDTH}x${HEIGHT} xc:transparent "$OUTPUT_DIR/bg_base.png"

# Create dashed rounded rectangle border for drop zone
$CONVERT -size ${WIDTH}x${HEIGHT} xc:transparent \
    -stroke '#c7c7cc' -strokewidth 2 -fill none \
    -draw "stroke-dasharray 8 6 roundrectangle $RECT_X,$RECT_Y $RECT_X2,$RECT_Y2 $CORNER_RADIUS,$CORNER_RADIUS" \
    "$OUTPUT_DIR/border.png" 2>/dev/null || \
$CONVERT -size ${WIDTH}x${HEIGHT} xc:transparent \
    -stroke '#c7c7cc' -strokewidth 2 -fill none \
    -draw "roundrectangle $RECT_X,$RECT_Y $RECT_X2,$RECT_Y2 $CORNER_RADIUS,$CORNER_RADIUS" \
    "$OUTPUT_DIR/border.png"

# Composite
$CONVERT "$OUTPUT_DIR/bg_base.png" \
    "$OUTPUT_DIR/border.png" -composite \
    "$OUTPUT_DIR/background.png"

# Create @2x version for Retina displays
$CONVERT "$OUTPUT_DIR/background.png" -resize 200% "$OUTPUT_DIR/background@2x.png"

# Create TIFF version
$CONVERT "$OUTPUT_DIR/background.png" "$OUTPUT_DIR/background.tiff"

# Clean up
rm -f "$OUTPUT_DIR/bg_base.png" "$OUTPUT_DIR/border.png"

echo "✅ DMG background created at: $OUTPUT_DIR/background.png"
echo "   Dimensions: ${WIDTH}x${HEIGHT}"
