#!/usr/bin/env bash
set -euo pipefail

REPO="khaphanspace/gonhanh.org"
ASSET_NAME="gonhanh-linux.tar.gz"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { printf "${BLUE}[*]${NC} %s\n" "$1"; }
log_ok() { printf "${GREEN}[✓]${NC} %s\n" "$1"; }
log_warn() { printf "${YELLOW}[!]${NC} %s\n" "$1"; }
log_error() { printf "${RED}[✗]${NC} %s\n" "$1" >&2; }

require_supported_architecture() {
    case "$(uname -m)" in
        x86_64|amd64) ;;
        *)
            log_error "Bản Linux hiện chỉ hỗ trợ x86_64. Hãy build từ source cho kiến trúc này."
            exit 1
            ;;
    esac
}

latest_version() {
    curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" |
        sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"v\{0,1\}\([^"]*\)".*/\1/p' |
        head -n 1
}

install_fcitx5() {
    if command -v fcitx5 >/dev/null 2>&1 && command -v fcitx5-remote >/dev/null 2>&1; then
        log_ok "Fcitx5 đã có sẵn"
        return 0
    fi

    log_info "Cài đặt Fcitx5..."
    if command -v apt >/dev/null 2>&1; then
        sudo apt update
        sudo apt install -y fcitx5 fcitx5-config-qt
    elif command -v dnf >/dev/null 2>&1; then
        sudo dnf install -y fcitx5 fcitx5-configtool
    elif command -v pacman >/dev/null 2>&1; then
        sudo pacman -S --needed --noconfirm fcitx5 fcitx5-configtool
    else
        log_error "Không nhận diện được trình quản lý gói. Hãy cài Fcitx5 thủ công."
        exit 1
    fi
    log_ok "Fcitx5 đã được cài đặt"
}

install_release() {
    local version archive package_dir
    version="$(latest_version)"
    [[ -n "$version" ]] || { log_error "Không xác định được phiên bản mới nhất"; exit 1; }
    archive="$TMP_DIR/$ASSET_NAME"
    package_dir="$TMP_DIR/gonhanh-linux"

    printf '\n'
    printf "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}\n"
    printf "${GREEN}  Gõ Nhanh v%s - Linux x86_64${NC}\n" "$version"
    printf "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}\n\n"

    log_info "Tải $ASSET_NAME..."
    curl -fL --retry 3 \
        "https://github.com/$REPO/releases/latest/download/$ASSET_NAME" \
        -o "$archive"
    tar -xzf "$archive" -C "$TMP_DIR"
    [[ -x "$package_dir/install.sh" ]] || {
        log_error "Archive không đúng định dạng: thiếu gonhanh-linux/install.sh"
        exit 1
    }

    GONHANH_VERSION="$version" "$package_dir/install.sh"
    command -v im-config >/dev/null 2>&1 && im-config -n fcitx5 >/dev/null 2>&1 || true
    command -v imsettings-switch >/dev/null 2>&1 && imsettings-switch fcitx5 >/dev/null 2>&1 || true
    log_ok "Đã cài Gõ Nhanh v$version"
}

print_summary() {
    printf '\n'
    printf "${GREEN}Cài đặt hoàn tất.${NC}\n"
    printf '  1. Đăng xuất rồi đăng nhập lại để desktop nạp cấu hình Fcitx5.\n'
    printf '  2. Chạy: source ~/.profile\n'
    printf '  3. Chạy: gn on\n'
    printf '  4. Nếu cần, mở fcitx5-configtool để kiểm tra Gõ Nhanh.\n\n'
    log_warn "Các input method hiện có (ví dụ Bamboo) được giữ nguyên."
}

main() {
    require_supported_architecture
    install_fcitx5
    install_release
    print_summary
}

main
