#!/bin/bash
set -e

# Source rustup environment
if [ -f "$HOME/.cargo/env" ]; then
    source "$HOME/.cargo/env"
fi

echo "🦀 Building Rust core..."

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"

# The grammar tables are generated from vi.dic / names.dic; a stale file would ship old grammar
python3 "$ROOT/scripts/gen/phonology_tables.py" --check || {
    echo "❌ phonology tables are stale: run 'make tables'"
    exit 1
}

cd "$ROOT/core"

# Build for macOS (universal binary)
echo "Building for aarch64-apple-darwin..."
cargo build --release --target aarch64-apple-darwin

echo "Building for x86_64-apple-darwin..."
cargo build --release --target x86_64-apple-darwin

# Create universal binary
echo "Creating universal binary..."
lipo -create \
    target/aarch64-apple-darwin/release/libgonhanh_core.a \
    target/x86_64-apple-darwin/release/libgonhanh_core.a \
    -output ../platforms/macos/libgonhanh_core.a

echo "✅ Rust core built successfully!"
echo "📦 Output: platforms/macos/libgonhanh_core.a"
