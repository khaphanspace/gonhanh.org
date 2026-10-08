#!/bin/bash
set -e

# Source rustup environment
if [ -f "$HOME/.cargo/env" ]; then
    source "$HOME/.cargo/env"
fi

echo "🦀 Building Rust core..."

cd "$(dirname "$0")/../../core"

# GN_FEATURES=engine_v2 builds the V2 typing engine into the library
FEATURES=${GN_FEATURES:+--features $GN_FEATURES}

# Build for macOS (universal binary)
echo "Building for aarch64-apple-darwin..."
cargo build --release $FEATURES --target aarch64-apple-darwin

echo "Building for x86_64-apple-darwin..."
cargo build --release $FEATURES --target x86_64-apple-darwin

# Create universal binary
echo "Creating universal binary..."
lipo -create \
    target/aarch64-apple-darwin/release/libgonhanh_core.a \
    target/x86_64-apple-darwin/release/libgonhanh_core.a \
    -output ../platforms/macos/libgonhanh_core.a

echo "✅ Rust core built successfully!"
echo "📦 Output: platforms/macos/libgonhanh_core.a"
