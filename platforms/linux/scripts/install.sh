#!/bin/bash
# User-local installation script
# No sudo required

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LINUX_DIR="$(dirname "$SCRIPT_DIR")"
ROOT_DIR="$(dirname "$(dirname "$LINUX_DIR")")"
BUILD_DIR="$LINUX_DIR/build"
CORE_DIR="$ROOT_DIR/core"

# XDG paths
FCITX5_LIB_DIR="${XDG_DATA_HOME:-$HOME/.local}/lib/fcitx5"
FCITX5_ADDON_DIR="${XDG_DATA_HOME:-$HOME/.local}/share/fcitx5/addon"
FCITX5_IM_DIR="${XDG_DATA_HOME:-$HOME/.local}/share/fcitx5/inputmethod"
LIB_DIR="${XDG_DATA_HOME:-$HOME/.local}/lib"

echo "=== Installing Gõ Nhanh (Fcitx5) ==="
echo ""

# Check if built
if [[ ! -f "$BUILD_DIR/gonhanh.so" ]]; then
    echo "Error: Build not found. Run scripts/build.sh first."
    exit 1
fi

# Create directories
echo "Creating directories..."
mkdir -p "$FCITX5_LIB_DIR"
mkdir -p "$FCITX5_ADDON_DIR"
mkdir -p "$FCITX5_IM_DIR"
mkdir -p "$LIB_DIR"

# Copy files
echo "Installing addon..."
cp "$BUILD_DIR/gonhanh.so" "$FCITX5_LIB_DIR/"
cp "$LINUX_DIR/data/gonhanh-addon.conf" "$FCITX5_ADDON_DIR/gonhanh.conf"
cp "$LINUX_DIR/data/gonhanh.conf" "$FCITX5_IM_DIR/"

# Copy Rust library
RUST_LIB="$CORE_DIR/target/release/libgonhanh_core.so"
if [[ ! -f "$RUST_LIB" ]]; then
    RUST_LIB="$CORE_DIR/target/debug/libgonhanh_core.so"
fi

if [[ -f "$RUST_LIB" ]]; then
    cp "$RUST_LIB" "$LIB_DIR/"
    echo "Rust core library installed."
else
    echo "Warning: Rust library not found. Build it first."
fi

# Update LD_LIBRARY_PATH in shell config
echo ""
echo "=== Installation complete ==="
echo ""
echo "Files installed to:"
echo "  Addon: $FCITX5_LIB_DIR/gonhanh.so"
echo "  Config: $FCITX5_ADDON_DIR/gonhanh.conf"
echo "  IM: $FCITX5_IM_DIR/gonhanh.conf"
echo "  Lib: $LIB_DIR/libgonhanh_core.so"
echo ""
echo "Next steps:"
echo "  1. Add to LD_LIBRARY_PATH (add to ~/.bashrc or ~/.zshrc):"
echo "     export LD_LIBRARY_PATH=\"\$HOME/.local/lib:\$LD_LIBRARY_PATH\""
echo ""
echo "  2. Restart Fcitx5:"
echo "     fcitx5 -r &"
echo ""
echo "  3. Add Gõ Nhanh in Fcitx5 configuration:"
echo "     fcitx5-configtool"
