#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LINUX_DIR="$(dirname "$SCRIPT_DIR")"
ROOT_DIR="$(dirname "$(dirname "$LINUX_DIR")")"
VERSION="${1:-dev}"
OUTPUT="${2:-$LINUX_DIR/gonhanh-linux.tar.gz}"
ADDON_LIBRARY="${GONHANH_ADDON_LIBRARY:-$LINUX_DIR/build/gonhanh.so}"
CORE_LIBRARY="${GONHANH_CORE_LIBRARY:-$ROOT_DIR/core/target/release/libgonhanh_core.so}"
STAGING="$(mktemp -d)"
PACKAGE="$STAGING/gonhanh-linux"
trap 'rm -rf "$STAGING"' EXIT

require_file() {
    [[ -f "$1" ]] || { printf 'Missing package input: %s\n' "$1" >&2; exit 1; }
}

require_file "$ADDON_LIBRARY"
require_file "$CORE_LIBRARY"
require_file "$LINUX_DIR/data/gonhanh-addon.conf"
require_file "$LINUX_DIR/data/gonhanh.conf"
require_file "$SCRIPT_DIR/install.sh"
require_file "$SCRIPT_DIR/gonhanh-cli.sh"

mkdir -p "$PACKAGE/lib" "$PACKAGE/share/fcitx5/addon"
mkdir -p "$PACKAGE/share/fcitx5/inputmethod"
install -m 0755 "$ADDON_LIBRARY" "$PACKAGE/lib/gonhanh.so"
install -m 0755 "$CORE_LIBRARY" "$PACKAGE/lib/libgonhanh_core.so"
install -m 0644 "$LINUX_DIR/data/gonhanh-addon.conf" "$PACKAGE/share/fcitx5/addon/gonhanh.conf"
install -m 0644 "$LINUX_DIR/data/gonhanh.conf" "$PACKAGE/share/fcitx5/inputmethod/gonhanh.conf"
install -m 0755 "$SCRIPT_DIR/install.sh" "$PACKAGE/install.sh"
install -m 0755 "$SCRIPT_DIR/gonhanh-cli.sh" "$PACKAGE/gonhanh-cli.sh"
install -m 0644 "$LINUX_DIR/README.md" "$PACKAGE/README.md"
printf '%s\n' "${VERSION#v}" > "$PACKAGE/VERSION"

mkdir -p "$(dirname "$OUTPUT")"
tar -czf "$OUTPUT" -C "$STAGING" gonhanh-linux

for required in \
    gonhanh-linux/install.sh \
    gonhanh-linux/gonhanh-cli.sh \
    gonhanh-linux/lib/gonhanh.so \
    gonhanh-linux/lib/libgonhanh_core.so \
    gonhanh-linux/share/fcitx5/addon/gonhanh.conf \
    gonhanh-linux/share/fcitx5/inputmethod/gonhanh.conf; do
    tar -tzf "$OUTPUT" "$required" >/dev/null || {
        printf 'Archive verification failed: %s\n' "$required" >&2
        exit 1
    }
done

printf 'Created %s\n' "$OUTPUT"
