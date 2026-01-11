#!/bin/bash
# build-macos-appkit.sh - Build pure AppKit version of GoNhanh
# Target: <10MB RAM

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$SCRIPT_DIR/.."
PLATFORM_DIR="$PROJECT_DIR/platforms/macos-appkit"
BUILD_DIR="$PLATFORM_DIR/build"
CORE_DIR="$PROJECT_DIR/core"

echo "Building GoNhanh AppKit (Lightweight)..."

# Get version from git tag
GIT_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "v1.0.0")
VERSION=${GIT_TAG#v}
echo "Version: $VERSION"

# Build Rust core library for both architectures
echo ""
echo "Building Rust core library..."
cd "$CORE_DIR"

# Build for both architectures
echo "  Building for arm64..."
cargo build --release --lib --target aarch64-apple-darwin

echo "  Building for x86_64..."
cargo build --release --lib --target x86_64-apple-darwin

# Create universal library
echo "  Creating universal library..."
mkdir -p target/release
lipo -create \
    target/aarch64-apple-darwin/release/libgonhanh_core.a \
    target/x86_64-apple-darwin/release/libgonhanh_core.a \
    -output target/release/libgonhanh_core.a

# Remove dylib to force static linking
rm -f target/release/libgonhanh_core.dylib

# Create build directory
rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR/GoNhanh.app/Contents/MacOS"
mkdir -p "$BUILD_DIR/GoNhanh.app/Contents/Resources"

# Create Info.plist
cat > "$BUILD_DIR/GoNhanh.app/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>vi</string>
    <key>CFBundleExecutable</key>
    <string>GoNhanh</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>CFBundleIdentifier</key>
    <string>org.gonhanh.GoNhanh</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>Gõ Nhanh</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>$VERSION</string>
    <key>CFBundleVersion</key>
    <string>$VERSION</string>
    <key>LSApplicationCategoryType</key>
    <string>public.app-category.productivity</string>
    <key>LSMinimumSystemVersion</key>
    <string>13.0</string>
    <key>LSUIElement</key>
    <true/>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSAppleEventsUsageDescription</key>
    <string>Gõ Nhanh cần quyền này để gõ tiếng Việt.</string>
</dict>
</plist>
EOF

# Compile Swift files
echo ""
echo "Compiling Swift files..."
cd "$PLATFORM_DIR"

SWIFT_FILES=(
    main.swift
    AppDelegate.swift
    AppState.swift
    MenuBarController.swift
    SettingsWindowController.swift
    OnboardingWindowController.swift
    RustBridge.swift
    InputSourceManager.swift
    SpecialPanelAppDetector.swift
    LaunchAtLogin.swift
)

# Build for both architectures
for ARCH in arm64 x86_64; do
    echo "Building for $ARCH..."
    swiftc -O \
        -target "${ARCH}-apple-macosx13.0" \
        -sdk "$(xcrun --show-sdk-path --sdk macosx)" \
        -I "$CORE_DIR/target/release" \
        -L "$CORE_DIR/target/release" \
        -lgonhanh_core \
        -framework Cocoa \
        -framework Carbon \
        -framework ServiceManagement \
        -framework ApplicationServices \
        -o "$BUILD_DIR/GoNhanh-$ARCH" \
        "${SWIFT_FILES[@]}"
done

# Create universal binary
echo ""
echo "Creating universal binary..."
lipo -create \
    "$BUILD_DIR/GoNhanh-arm64" \
    "$BUILD_DIR/GoNhanh-x86_64" \
    -output "$BUILD_DIR/GoNhanh.app/Contents/MacOS/GoNhanh"

# Cleanup temp binaries
rm -f "$BUILD_DIR/GoNhanh-arm64" "$BUILD_DIR/GoNhanh-x86_64"

# Copy resources
echo "Copying resources..."
if [ -d "$PROJECT_DIR/platforms/macos/Assets.xcassets/AppIcon.appiconset" ]; then
    # Create icns from iconset
    ICONSET_DIR="/tmp/AppIcon.iconset"
    mkdir -p "$ICONSET_DIR"

    # Copy and rename icons for iconutil
    cp "$PROJECT_DIR/platforms/macos/Assets.xcassets/AppIcon.appiconset/AppIcon-16.png" "$ICONSET_DIR/icon_16x16.png" 2>/dev/null || true
    cp "$PROJECT_DIR/platforms/macos/Assets.xcassets/AppIcon.appiconset/AppIcon-32.png" "$ICONSET_DIR/icon_16x16@2x.png" 2>/dev/null || true
    cp "$PROJECT_DIR/platforms/macos/Assets.xcassets/AppIcon.appiconset/AppIcon-32.png" "$ICONSET_DIR/icon_32x32.png" 2>/dev/null || true
    cp "$PROJECT_DIR/platforms/macos/Assets.xcassets/AppIcon.appiconset/AppIcon-64.png" "$ICONSET_DIR/icon_32x32@2x.png" 2>/dev/null || true
    cp "$PROJECT_DIR/platforms/macos/Assets.xcassets/AppIcon.appiconset/AppIcon-128.png" "$ICONSET_DIR/icon_128x128.png" 2>/dev/null || true
    cp "$PROJECT_DIR/platforms/macos/Assets.xcassets/AppIcon.appiconset/AppIcon-256.png" "$ICONSET_DIR/icon_128x128@2x.png" 2>/dev/null || true
    cp "$PROJECT_DIR/platforms/macos/Assets.xcassets/AppIcon.appiconset/AppIcon-256.png" "$ICONSET_DIR/icon_256x256.png" 2>/dev/null || true
    cp "$PROJECT_DIR/platforms/macos/Assets.xcassets/AppIcon.appiconset/AppIcon-512.png" "$ICONSET_DIR/icon_256x256@2x.png" 2>/dev/null || true
    cp "$PROJECT_DIR/platforms/macos/Assets.xcassets/AppIcon.appiconset/AppIcon-512.png" "$ICONSET_DIR/icon_512x512.png" 2>/dev/null || true
    cp "$PROJECT_DIR/platforms/macos/Assets.xcassets/AppIcon.appiconset/AppIcon-1024.png" "$ICONSET_DIR/icon_512x512@2x.png" 2>/dev/null || true

    if [ "$(ls -A $ICONSET_DIR)" ]; then
        iconutil -c icns -o "$BUILD_DIR/GoNhanh.app/Contents/Resources/AppIcon.icns" "$ICONSET_DIR" 2>/dev/null || true
    fi
    rm -rf "$ICONSET_DIR"
fi

# Sign app (ad-hoc for development)
echo ""
echo "Signing app..."
codesign --force --deep --sign - \
    --entitlements "$PROJECT_DIR/platforms/macos/GoNhanh.entitlements" \
    "$BUILD_DIR/GoNhanh.app"

# Verify signature
codesign -vvv --deep --strict "$BUILD_DIR/GoNhanh.app"

echo ""
echo "Build completed!"
echo "App: $BUILD_DIR/GoNhanh.app"

# Show size comparison
echo ""
echo "=== Size Comparison ==="
if [ -d "$PROJECT_DIR/platforms/macos/build/Release/GoNhanh.app" ]; then
    SWIFTUI_SIZE=$(du -sh "$PROJECT_DIR/platforms/macos/build/Release/GoNhanh.app" | cut -f1)
    echo "SwiftUI version: $SWIFTUI_SIZE"
fi
APPKIT_SIZE=$(du -sh "$BUILD_DIR/GoNhanh.app" | cut -f1)
echo "AppKit version:  $APPKIT_SIZE"
