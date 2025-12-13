#!/bin/bash
# Build script for Linux platform
# Usage: ./scripts/build.sh [--release]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LINUX_DIR="$(dirname "$SCRIPT_DIR")"
ROOT_DIR="$(dirname "$(dirname "$LINUX_DIR")")"
CORE_DIR="$ROOT_DIR/core"
BUILD_TYPE="Release"

# Parse arguments
if [[ "$1" == "--debug" ]]; then
    BUILD_TYPE="Debug"
fi

echo "=== Building Gõ Nhanh Linux (Fcitx5) ==="
echo "Build type: $BUILD_TYPE"
echo ""

# Step 1: Build Rust core
echo "=== Step 1: Building Rust core ==="
cd "$CORE_DIR"

# Detect host target triple for cross-platform support
HOST_TRIPLE=$(rustc -vV 2>/dev/null | grep 'host:' | awk '{print $2}')
if [[ -z "$HOST_TRIPLE" ]]; then
    HOST_TRIPLE="x86_64-unknown-linux-gnu"
    echo "Warning: Could not detect host triple, defaulting to $HOST_TRIPLE"
fi
echo "Host target: $HOST_TRIPLE"

if [[ "$BUILD_TYPE" == "Debug" ]]; then
    cargo build
    RUST_TARGET_DIR="target/debug"
else
    cargo build --release
    RUST_TARGET_DIR="target/release"
fi

# Verify library exists
if [[ ! -f "$RUST_TARGET_DIR/libgonhanh_core.so" ]]; then
    echo "Error: Rust library not found at $RUST_TARGET_DIR/libgonhanh_core.so"
    echo "Build failed. Check cargo output above."
    exit 1
fi

echo "Rust core built: $RUST_TARGET_DIR/libgonhanh_core.so"
echo ""

# Step 2: Build C++ addon
echo "=== Step 2: Building Fcitx5 addon ==="
cd "$LINUX_DIR"
mkdir -p build
cd build

cmake .. -DCMAKE_BUILD_TYPE=$BUILD_TYPE
make -j$(nproc)

echo ""
echo "=== Build complete ==="
echo "Addon: $LINUX_DIR/build/gonhanh.so"
echo ""
echo "To install (user-local):"
echo "  make install-user"
echo ""
echo "To install (system-wide, requires sudo):"
echo "  sudo make install"
