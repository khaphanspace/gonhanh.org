#!/bin/bash
set -e

# Source rustup environment
if [ -f "$HOME/.cargo/env" ]; then
    source "$HOME/.cargo/env"
fi

# Optional feature flags (e.g., CARGO_FEATURES="engine-v2")
FEATURES=""
if [ -n "$CARGO_FEATURES" ]; then
    FEATURES="--features $CARGO_FEATURES"
    echo "🦀 Building Rust core with features: $CARGO_FEATURES"
else
    echo "🦀 Building Rust core..."
fi

cd "$(dirname "$0")/../core"

# Build for macOS (universal binary)
echo "Building for aarch64-apple-darwin..."
cargo build --release --target aarch64-apple-darwin $FEATURES

echo "Building for x86_64-apple-darwin..."
cargo build --release --target x86_64-apple-darwin $FEATURES

# Create universal binary
echo "Creating universal binary..."
lipo -create \
    target/aarch64-apple-darwin/release/libgonhanh_core.a \
    target/x86_64-apple-darwin/release/libgonhanh_core.a \
    -output ../platforms/macos/libgonhanh_core.a

echo "✅ Rust core built successfully!"
echo "📦 Output: platforms/macos/libgonhanh_core.a"
