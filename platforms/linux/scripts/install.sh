#!/bin/bash
# Gõ Nhanh installation script for Linux
# Works with both release tarball and source build
# No sudo required - installs to user-local paths

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# XDG paths
FCITX5_LIB_DIR="${XDG_DATA_HOME:-$HOME/.local}/lib/fcitx5"
FCITX5_ADDON_DIR="${XDG_DATA_HOME:-$HOME/.local}/share/fcitx5/addon"
FCITX5_IM_DIR="${XDG_DATA_HOME:-$HOME/.local}/share/fcitx5/inputmethod"
LIB_DIR="${XDG_DATA_HOME:-$HOME/.local}/lib"

# Uninstall function
uninstall() {
    echo "=== Uninstalling Gõ Nhanh ==="
    rm -f "$FCITX5_LIB_DIR/gonhanh.so"
    rm -f "$LIB_DIR/libgonhanh_core.so"
    rm -f "$FCITX5_ADDON_DIR/gonhanh.conf"
    rm -f "$FCITX5_IM_DIR/gonhanh.conf"
    echo "Uninstallation complete."
    echo "Please restart Fcitx5: fcitx5 -r &"
    exit 0
}

# Parse arguments
if [[ "$1" == "--uninstall" || "$1" == "-u" ]]; then
    uninstall
fi

echo "=== Installing Gõ Nhanh (Fcitx5) ==="
echo ""

# Detect if running from release tarball or source
if [[ -f "$SCRIPT_DIR/lib/gonhanh.so" ]]; then
    # Release tarball structure (install.sh at root)
    ADDON_SO="$SCRIPT_DIR/lib/gonhanh.so"
    RUST_LIB="$SCRIPT_DIR/lib/libgonhanh_core.so"
    ADDON_CONF="$SCRIPT_DIR/share/fcitx5/addon/gonhanh.conf"
    IM_CONF="$SCRIPT_DIR/share/fcitx5/inputmethod/gonhanh.conf"
    echo "Installing from release package..."
elif [[ -f "$SCRIPT_DIR/../build/gonhanh.so" ]]; then
    # Source build structure
    LINUX_DIR="$(dirname "$SCRIPT_DIR")"
    ROOT_DIR="$(dirname "$(dirname "$LINUX_DIR")")"
    ADDON_SO="$LINUX_DIR/build/gonhanh.so"
    ADDON_CONF="$LINUX_DIR/data/gonhanh-addon.conf"
    IM_CONF="$LINUX_DIR/data/gonhanh.conf"
    # Find Rust lib
    if [[ -f "$ROOT_DIR/core/target/release/libgonhanh_core.so" ]]; then
        RUST_LIB="$ROOT_DIR/core/target/release/libgonhanh_core.so"
    else
        RUST_LIB="$ROOT_DIR/core/target/debug/libgonhanh_core.so"
    fi
    echo "Installing from source build..."
else
    echo "Error: Cannot find gonhanh.so"
    echo "Run from release tarball or build from source first."
    exit 1
fi

# Verify files exist
for f in "$ADDON_SO" "$RUST_LIB" "$ADDON_CONF" "$IM_CONF"; do
    if [[ ! -f "$f" ]]; then
        echo "Error: Missing file: $f"
        exit 1
    fi
done

# Create directories
echo "Creating directories..."
mkdir -p "$FCITX5_LIB_DIR"
mkdir -p "$FCITX5_ADDON_DIR"
mkdir -p "$FCITX5_IM_DIR"
mkdir -p "$LIB_DIR"

# Copy files
echo "Installing files..."
cp "$ADDON_SO" "$FCITX5_LIB_DIR/"
cp "$RUST_LIB" "$LIB_DIR/"
cp "$ADDON_CONF" "$FCITX5_ADDON_DIR/gonhanh.conf"
cp "$IM_CONF" "$FCITX5_IM_DIR/"

echo ""
echo "=== Installation complete ==="
echo ""
echo "Files installed:"
echo "  $FCITX5_LIB_DIR/gonhanh.so"
echo "  $LIB_DIR/libgonhanh_core.so"
echo "  $FCITX5_ADDON_DIR/gonhanh.conf"
echo "  $FCITX5_IM_DIR/gonhanh.conf"
echo ""
echo "Next steps:"
echo "  1. Restart Fcitx5:"
echo "     fcitx5 -r &"
echo ""
echo "  2. Add Gõ Nhanh in Fcitx5 configuration:"
echo "     fcitx5-configtool"
echo ""
echo "To uninstall: $0 --uninstall"
