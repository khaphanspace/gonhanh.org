#!/bin/bash
set -e

# GoNhanh Windows Cross-Compile Script (from macOS/Linux)
# Uses mingw-w64 GNU toolchain for cross-compilation

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "GoNhanh Windows Cross-Compile"
echo "============================="
echo ""

# Check mingw-w64 is installed
if ! command -v x86_64-w64-mingw32-gcc &> /dev/null; then
    echo "Error: mingw-w64 not found"
    echo "Install with: brew install mingw-w64"
    exit 1
fi

# Ensure target is installed
if ! rustup target list --installed | grep -q "x86_64-pc-windows-gnu"; then
    echo "Adding Rust target: x86_64-pc-windows-gnu"
    rustup target add x86_64-pc-windows-gnu
fi

# Build Rust core
echo "[1/2] Building Rust core for Windows (GNU toolchain)..."
cd "$PROJECT_ROOT/core"
cargo build --release --target x86_64-pc-windows-gnu

# Copy DLL
echo "[2/2] Copying DLL..."
NATIVE_DIR="$PROJECT_ROOT/platforms/windows/GoNhanh/Native"
mkdir -p "$NATIVE_DIR"
cp "target/x86_64-pc-windows-gnu/release/gonhanh_core.dll" "$NATIVE_DIR/"

# Show results
DLL_PATH="$NATIVE_DIR/gonhanh_core.dll"
DLL_SIZE=$(ls -lh "$DLL_PATH" | awk '{print $5}')

echo ""
echo "Build complete!"
echo "  Output: $DLL_PATH"
echo "  Size: $DLL_SIZE"
echo ""
echo "Note: GNU-built DLL works on Windows but native MSVC build is preferred."
echo "Run 'scripts/build-core-windows.ps1' on Windows for MSVC build."
