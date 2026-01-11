#!/bin/bash
# build-macos-c.sh - Build Pure C version of GoNhanh
# Target: ~15MB RAM, smallest binary

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$SCRIPT_DIR/.."
PLATFORM_DIR="$PROJECT_DIR/platforms/macos-c"
BUILD_DIR="$PLATFORM_DIR/build"
CORE_DIR="$PROJECT_DIR/core"

echo "Building GoNhanh Pure C (Experimental)..."

# Get version
GIT_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "v1.0.0")
VERSION=${GIT_TAG#v}
echo "Version: $VERSION"

# Build Rust core library for both architectures
echo ""
echo "Building Rust core library..."
cd "$CORE_DIR"

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
    <key>CFBundleIdentifier</key>
    <string>org.gonhanh.GoNhanh-C</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>Go Nhanh C</string>
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
</dict>
</plist>
EOF

# Compile Objective-C code
echo ""
echo "Compiling Objective-C..."
cd "$PLATFORM_DIR"

# Build for both architectures
for ARCH in arm64 x86_64; do
    echo "  Building for $ARCH..."
    clang -O2 -fobjc-arc \
        -target "${ARCH}-apple-macosx13.0" \
        -isysroot "$(xcrun --show-sdk-path --sdk macosx)" \
        -I "$CORE_DIR/target/release" \
        -L "$CORE_DIR/target/release" \
        -lgonhanh_core \
        -framework Cocoa \
        -framework CoreGraphics \
        -o "$BUILD_DIR/GoNhanh-$ARCH" \
        main.m
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

# Sign app (ad-hoc)
echo ""
echo "Signing app..."
codesign --force --deep --sign - "$BUILD_DIR/GoNhanh.app"

# Verify
codesign -vvv --deep --strict "$BUILD_DIR/GoNhanh.app"

echo ""
echo "Build completed!"
echo "App: $BUILD_DIR/GoNhanh.app"

# Show size comparison
echo ""
echo "=== Size Comparison ==="
if [ -d "$PROJECT_DIR/platforms/macos/build/Release/GoNhanh.app" ]; then
    echo "SwiftUI:  $(du -sh "$PROJECT_DIR/platforms/macos/build/Release/GoNhanh.app" | cut -f1)"
fi
if [ -d "$PROJECT_DIR/platforms/macos-appkit/build/GoNhanh.app" ]; then
    echo "AppKit:   $(du -sh "$PROJECT_DIR/platforms/macos-appkit/build/GoNhanh.app" | cut -f1)"
fi
echo "Pure C:   $(du -sh "$BUILD_DIR/GoNhanh.app" | cut -f1)"

echo ""
echo "Binary size: $(ls -lh "$BUILD_DIR/GoNhanh.app/Contents/MacOS/GoNhanh" | awk '{print $5}')"
