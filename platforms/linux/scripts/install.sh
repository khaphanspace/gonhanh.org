#!/bin/bash
set -e
DIR="$(cd "$(dirname "$0")" && pwd)"

# Uninstall
[[ "$1" == "-u" || "$1" == "--uninstall" ]] && {
    rm -f ~/.local/lib/fcitx5/gonhanh.so ~/.local/lib/libgonhanh_core.so
    rm -f ~/.local/share/fcitx5/addon/gonhanh.conf ~/.local/share/fcitx5/inputmethod/gonhanh.conf
    echo "Uninstalled. Run: fcitx5 -r"
    exit 0
}

# Detect source: tarball (lib/) or build (../build/)
[[ -f "$DIR/lib/gonhanh.so" ]] && SRC="$DIR" || SRC="$(dirname "$DIR")"
[[ -f "$SRC/lib/gonhanh.so" ]] && LIB="$SRC/lib" || LIB="$SRC/build"
[[ -f "$SRC/share/fcitx5/addon/gonhanh.conf" ]] && DATA="$SRC/share/fcitx5" || DATA="$SRC/data"

# Find Rust lib
RUST=""
for p in "$LIB/libgonhanh_core.so" "$SRC/../../core/target/release/libgonhanh_core.so" "$SRC/../../core/target/debug/libgonhanh_core.so"; do
    [[ -f "$p" ]] && RUST="$p" && break
done

[[ ! -f "$LIB/gonhanh.so" || -z "$RUST" ]] && echo "Error: Build not found" && exit 1

# Install
mkdir -p ~/.local/lib/fcitx5 ~/.local/share/fcitx5/{addon,inputmethod}
cp "$LIB/gonhanh.so" ~/.local/lib/fcitx5/
cp "$RUST" ~/.local/lib/
[[ -f "$DATA/addon/gonhanh.conf" ]] && cp "$DATA/addon/gonhanh.conf" ~/.local/share/fcitx5/addon/
[[ -f "$DATA/inputmethod/gonhanh.conf" ]] && cp "$DATA/inputmethod/gonhanh.conf" ~/.local/share/fcitx5/inputmethod/
[[ -f "$DATA/gonhanh-addon.conf" ]] && cp "$DATA/gonhanh-addon.conf" ~/.local/share/fcitx5/addon/gonhanh.conf
[[ -f "$DATA/gonhanh.conf" && ! -f "$DATA/addon/gonhanh.conf" ]] && cp "$DATA/gonhanh.conf" ~/.local/share/fcitx5/inputmethod/

echo "✓ Installed. Run: fcitx5 -r && fcitx5-configtool"
