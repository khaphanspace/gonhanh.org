#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [[ -n "${GONHANH_PREFIX:-}" ]]; then
    PREFIX="$GONHANH_PREFIX"
elif [[ "$(basename "$SCRIPT_DIR")" == "gonhanh" && "$(basename "$(dirname "$SCRIPT_DIR")")" == "share" ]]; then
    PREFIX="$(cd "$SCRIPT_DIR/../.." && pwd)"
else
    PREFIX="$HOME/.local"
fi
CONFIG_HOME="${XDG_CONFIG_HOME:-$HOME/.config}"
STATE_DIR="$PREFIX/share/gonhanh"
PROFILE="$CONFIG_HOME/fcitx5/profile"
PROFILE_BACKUP="$CONFIG_HOME/fcitx5/profile.gonhanh.bak"
SHELL_PROFILE="$HOME/.profile"
BLOCK_START="# >>> Gõ Nhanh Fcitx5 >>>"
BLOCK_END="# <<< Gõ Nhanh Fcitx5 <<<"

log() { printf '%s\n' "$*"; }
fail() { printf 'Error: %s\n' "$*" >&2; exit 1; }

restart_fcitx() {
    [[ "${GONHANH_SKIP_RESTART:-0}" == "1" ]] && return 0
    command -v fcitx5-remote >/dev/null 2>&1 || return 0
    fcitx5-remote -r >/dev/null 2>&1 || true
    sleep 0.2
    fcitx5-remote -s gonhanh >/dev/null 2>&1 || true
}

profile_has_gonhanh() {
    [[ -f "$PROFILE" ]] || return 1
    awk '
        /^\[Groups\/[0-9][0-9]*\/Items\/[0-9][0-9]*\]$/ { in_item = 1; next }
        /^\[/ { in_item = 0 }
        in_item && $0 == "Name=gonhanh" { found = 1 }
        END { exit(found ? 0 : 1) }
    ' "$PROFILE"
}

configure_fcitx_profile() {
    mkdir -p "$(dirname "$PROFILE")"
    touch "$PROFILE"
    profile_has_gonhanh && return 0

    [[ -s "$PROFILE" && ! -f "$PROFILE_BACKUP" ]] && cp "$PROFILE" "$PROFILE_BACKUP"

    local group max_item next_item
    group="$(sed -n 's/^\[Groups\/\([0-9][0-9]*\)\]$/\1/p' "$PROFILE" | head -n 1)"
    if [[ -z "$group" ]]; then
        group="$(sed -n 's/^\[Groups\/\([0-9][0-9]*\)\/Items\/[0-9][0-9]*\]$/\1/p' "$PROFILE" | head -n 1)"
    fi

    if [[ -z "$group" ]]; then
        group=0
        {
            [[ ! -s "$PROFILE" ]] || printf '\n'
            printf '[Groups/0]\nName=Default\nDefault Layout=us\nDefaultIM=gonhanh\n'
            printf '\n[Groups/0/Items/0]\nName=keyboard-us\nLayout=\n'
        } >> "$PROFILE"
    fi

    max_item="$(sed -n "s|^\[Groups/$group/Items/\([0-9][0-9]*\)\]$|\1|p" "$PROFILE" | sort -n | tail -n 1)"
    if [[ -z "$max_item" ]]; then
        next_item=0
    else
        next_item=$((max_item + 1))
    fi

    {
        printf '\n[Groups/%s/Items/%s]\n' "$group" "$next_item"
        printf 'Name=gonhanh\nLayout=\n'
    } >> "$PROFILE"

    if ! sed -n '/^\[GroupOrder\]$/p' "$PROFILE" | head -n 1 | awk 'NF { found=1 } END { exit(found ? 0 : 1) }'; then
        printf '\n[GroupOrder]\n%s=Default\n' "$group" >> "$PROFILE"
    fi
}

remove_fcitx_profile_entry() {
    [[ -f "$PROFILE" ]] || return 0
    local tmp
    tmp="$(mktemp "${PROFILE}.tmp.XXXXXX")"
    awk '
        function flush() {
            if (!(section ~ /^\[Groups\/[0-9][0-9]*\/Items\/[0-9][0-9]*\]$/ && body ~ /(^|\n)Name=gonhanh(\n|$)/)) {
                printf "%s", body
            }
            body = ""
        }
        /^\[/ { flush(); section = $0 }
        { body = body $0 ORS }
        END { flush() }
    ' "$PROFILE" > "$tmp"
    mv "$tmp" "$PROFILE"
}

configure_shell_profile() {
    local path_entry
    touch "$SHELL_PROFILE"
    if sed -n "\|^${BLOCK_START}$|p" "$SHELL_PROFILE" | head -n 1 | awk 'NF { found=1 } END { exit(found ? 0 : 1) }'; then
        return 0
    fi
    if [[ "$PREFIX" == "$HOME/.local" ]]; then
        path_entry='$HOME/.local/bin'
    else
        printf -v path_entry '%q' "$PREFIX/bin"
    fi
    {
        [[ ! -s "$SHELL_PROFILE" ]] || printf '\n'
        printf '%s\n' "$BLOCK_START"
        printf 'export GTK_IM_MODULE=fcitx\n'
        printf 'export QT_IM_MODULE=fcitx\n'
        printf 'export XMODIFIERS=@im=fcitx\n'
        printf 'export PATH=%s:$PATH\n' "$path_entry"
        printf '%s\n' "$BLOCK_END"
    } >> "$SHELL_PROFILE"
}

remove_shell_profile_block() {
    [[ -f "$SHELL_PROFILE" ]] || return 0
    local tmp
    tmp="$(mktemp "${SHELL_PROFILE}.tmp.XXXXXX")"
    awk -v start="$BLOCK_START" -v end="$BLOCK_END" '
        $0 == start && !capturing {
            capturing = 1
            buffered = $0 ORS
            next
        }
        capturing {
            buffered = buffered $0 ORS
            if ($0 == end) {
                capturing = 0
                buffered = ""
            }
            next
        }
        { print }
        END {
            if (capturing) {
                printf "%s", buffered
            }
        }
    ' "$SHELL_PROFILE" > "$tmp"
    mv "$tmp" "$SHELL_PROFILE"
}

uninstall() {
    rm -f "$PREFIX/lib/fcitx5/gonhanh.so" "$PREFIX/lib/libgonhanh_core.so"
    rm -f "$PREFIX/share/fcitx5/addon/gonhanh.conf"
    rm -f "$PREFIX/share/fcitx5/inputmethod/gonhanh.conf"
    rm -f "$PREFIX/bin/gn"
    remove_fcitx_profile_entry
    remove_shell_profile_block
    rm -rf "$CONFIG_HOME/gonhanh" "$STATE_DIR"
    restart_fcitx
    log "✓ Đã gỡ cài đặt Gõ Nhanh; cấu hình Fcitx khác được giữ nguyên."
}

resolve_sources() {
    if [[ -f "$SCRIPT_DIR/lib/gonhanh.so" ]]; then
        SOURCE_ROOT="$SCRIPT_DIR"
        LIB_DIR="$SOURCE_ROOT/lib"
        DATA_DIR="$SOURCE_ROOT/share/fcitx5"
        CLI_SOURCE="$SOURCE_ROOT/gonhanh-cli.sh"
    else
        SOURCE_ROOT="$(dirname "$SCRIPT_DIR")"
        LIB_DIR="$SOURCE_ROOT/build"
        DATA_DIR="$SOURCE_ROOT/data"
        CLI_SOURCE="$SOURCE_ROOT/scripts/gonhanh-cli.sh"
    fi

    RUST_LIBRARY="$LIB_DIR/libgonhanh_core.so"
    if [[ ! -f "$RUST_LIBRARY" ]]; then
        for candidate in \
            "$SOURCE_ROOT/../../core/target/release/libgonhanh_core.so" \
            "$SOURCE_ROOT/../../core/target/debug/libgonhanh_core.so"; do
            if [[ -f "$candidate" ]]; then
                RUST_LIBRARY="$candidate"
                break
            fi
        done
    fi

    [[ -f "$LIB_DIR/gonhanh.so" ]] || fail "Không tìm thấy gonhanh.so"
    [[ -f "$RUST_LIBRARY" ]] || fail "Không tìm thấy libgonhanh_core.so"
    [[ -f "$CLI_SOURCE" ]] || fail "Không tìm thấy gonhanh-cli.sh"
}

install_files() {
    local addon_config input_method_config version
    if [[ -f "$DATA_DIR/addon/gonhanh.conf" ]]; then
        addon_config="$DATA_DIR/addon/gonhanh.conf"
        input_method_config="$DATA_DIR/inputmethod/gonhanh.conf"
    else
        addon_config="$DATA_DIR/gonhanh-addon.conf"
        input_method_config="$DATA_DIR/gonhanh.conf"
    fi
    [[ -f "$addon_config" ]] || fail "Thiếu cấu hình addon Fcitx5"
    [[ -f "$input_method_config" ]] || fail "Thiếu cấu hình input method Fcitx5"

    mkdir -p "$PREFIX/lib/fcitx5" "$PREFIX/share/fcitx5/addon"
    mkdir -p "$PREFIX/share/fcitx5/inputmethod" "$PREFIX/bin" "$STATE_DIR"
    install -m 0755 "$LIB_DIR/gonhanh.so" "$PREFIX/lib/fcitx5/gonhanh.so"
    install -m 0755 "$RUST_LIBRARY" "$PREFIX/lib/libgonhanh_core.so"
    install -m 0644 "$addon_config" "$PREFIX/share/fcitx5/addon/gonhanh.conf"
    install -m 0644 "$input_method_config" "$PREFIX/share/fcitx5/inputmethod/gonhanh.conf"
    install -m 0755 "$CLI_SOURCE" "$PREFIX/bin/gn"
    install -m 0755 "$SCRIPT_DIR/install.sh" "$STATE_DIR/install.sh"

    version="${GONHANH_VERSION:-}"
    if [[ -z "$version" && -f "$SCRIPT_DIR/VERSION" ]]; then
        version="$(sed -n '1p' "$SCRIPT_DIR/VERSION")"
    fi
    printf '%s\n' "${version#v}" > "$STATE_DIR/version"
}

case "${1:-}" in
    -u|--uninstall)
        uninstall
        exit 0
        ;;
    "") ;;
    *) fail "Tuỳ chọn không hợp lệ: $1" ;;
esac

resolve_sources
install_files
configure_fcitx_profile
configure_shell_profile
restart_fcitx
log "✓ Đã cài Gõ Nhanh mà không thay đổi các input method Fcitx hiện có."
log "  Chạy 'gn on' để chọn Gõ Nhanh, hoặc mở fcitx5-configtool."
