#!/bin/bash
# Gõ Nhanh Linux Installer
# curl -fsSL https://raw.githubusercontent.com/khaphanspace/gonhanh.org/main/scripts/install-linux.sh | bash

REPO="khaphanspace/gonhanh.org"
VERSION="1.0.0"
TMP=$(mktemp -d)
trap "rm -rf $TMP" EXIT

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info()  { echo -e "${BLUE}[*]${NC} $1"; }
log_ok()    { echo -e "${GREEN}[✓]${NC} $1"; }
log_warn()  { echo -e "${YELLOW}[!]${NC} $1"; }
log_error() { echo -e "${RED}[✗]${NC} $1"; }

header() {
    echo ""
    echo -e "${GREEN}╔═══════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║${NC}       Gõ Nhanh v$VERSION - Linux        ${GREEN}║${NC}"
    echo -e "${GREEN}╚═══════════════════════════════════════╝${NC}"
    echo ""
}

# Install Fcitx5 if needed
install_fcitx5() {
    if command -v fcitx5 &>/dev/null; then
        log_ok "Fcitx5 đã có sẵn"
        return 0
    fi

    log_info "Cài đặt Fcitx5..."
    if command -v apt &>/dev/null; then
        sudo apt update -qq || { log_error "apt update thất bại"; exit 1; }
        sudo apt install -y fcitx5 || { log_error "Không thể cài fcitx5"; exit 1; }
    elif command -v dnf &>/dev/null; then
        sudo dnf install -y fcitx5 || { log_error "Không thể cài fcitx5"; exit 1; }
    elif command -v pacman &>/dev/null; then
        sudo pacman -S --noconfirm fcitx5 || { log_error "Không thể cài fcitx5"; exit 1; }
    else
        log_error "Distro không được hỗ trợ"
        exit 1
    fi
    log_ok "Fcitx5 đã cài đặt"
}

# Download and install GoNhanh addon
install_addon() {
    log_info "Tải Gõ Nhanh addon..."
    cd "$TMP"
    if curl -fsSL "https://github.com/$REPO/releases/latest/download/gonhanh-linux.tar.gz" | tar xz; then
        cd gonhanh-linux && ./install.sh
        log_ok "Addon đã cài đặt"
    else
        log_error "Không thể tải addon"
        exit 1
    fi
}

# Install CLI tool
install_cli() {
    log_info "Cài đặt CLI..."
    mkdir -p ~/.local/bin
    if ! curl -fsSL "https://raw.githubusercontent.com/$REPO/main/platforms/linux/scripts/gonhanh-cli.sh" -o ~/.local/bin/gn; then
        log_error "Không thể tải CLI"
        exit 1
    fi
    chmod +x ~/.local/bin/gn

    # Ensure ~/.local/bin is in PATH
    if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
        SHELL_RC=""
        [[ -f ~/.zshrc ]] && SHELL_RC=~/.zshrc
        [[ -f ~/.bashrc ]] && SHELL_RC=~/.bashrc

        if [[ -n "$SHELL_RC" ]] && ! grep -q 'PATH="$HOME/.local/bin' "$SHELL_RC" 2>/dev/null; then
            echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$SHELL_RC"
        fi
        export PATH="$HOME/.local/bin:$PATH"
    fi
    log_ok "CLI (gn) đã cài đặt"
}

# Setup environment variables for input method
setup_environment() {
    log_info "Cấu hình môi trường..."

    ENV_BLOCK='# Gõ Nhanh
export GTK_IM_MODULE=fcitx
export QT_IM_MODULE=fcitx
export XMODIFIERS=@im=fcitx'

    for rc in ~/.bashrc ~/.zshrc ~/.profile; do
        if [[ -f "$rc" ]] && ! grep -q "GTK_IM_MODULE=fcitx" "$rc" 2>/dev/null; then
            echo "" >> "$rc"
            echo "$ENV_BLOCK" >> "$rc"
        fi
    done

    export GTK_IM_MODULE=fcitx
    export QT_IM_MODULE=fcitx
    export XMODIFIERS=@im=fcitx
    log_ok "Môi trường đã cấu hình"
}

# Configure Fcitx5 to use GoNhanh
configure_fcitx5() {
    log_info "Cấu hình Fcitx5..."

    FCITX5_DIR="$HOME/.config/fcitx5"
    PROFILE="$FCITX5_DIR/profile"
    mkdir -p "$FCITX5_DIR"

    if [[ -f "$PROFILE" ]] && grep -q "gonhanh" "$PROFILE" 2>/dev/null; then
        log_ok "Fcitx5 đã được cấu hình"
        return 0
    fi

    # Create fresh profile
    cat > "$PROFILE" << 'EOF'
[Groups/0]
Name=Default
Default Layout=us
DefaultIM=gonhanh

[Groups/0/Items/0]
Name=keyboard-us
Layout=

[Groups/0/Items/1]
Name=gonhanh
Layout=

[GroupOrder]
0=Default
EOF
    log_ok "Fcitx5 đã được cấu hình"
}

# Set Fcitx5 as default IM
set_default_im() {
    command -v im-config &>/dev/null && im-config -n fcitx5 2>/dev/null || true
    command -v imsettings-switch &>/dev/null && imsettings-switch fcitx5 2>/dev/null || true
}

# Start Fcitx5
start_fcitx5() {
    log_info "Khởi động Fcitx5..."
    pkill -9 fcitx5 2>/dev/null || true
    sleep 0.3
    nohup fcitx5 -d &>/dev/null &
    sleep 0.5

    if pgrep -x fcitx5 &>/dev/null; then
        log_ok "Fcitx5 đang chạy"
    else
        log_warn "Fcitx5 chưa chạy (cần GUI)"
    fi
}

# Print final summary
print_summary() {
    echo ""
    echo -e "${GREEN}╔═══════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║${NC}         Cài đặt hoàn tất!             ${GREEN}║${NC}"
    echo -e "${GREEN}╚═══════════════════════════════════════╝${NC}"
    echo ""
    echo -e "  ${BLUE}Phím tắt:${NC}  Ctrl+Space hoặc Super+Space"
    echo ""
    echo -e "  ${BLUE}Lệnh:${NC}"
    echo "    gn           Toggle bật/tắt"
    echo "    gn vni       Chuyển VNI"
    echo "    gn telex     Chuyển Telex"
    echo "    gn status    Xem trạng thái"
    echo "    gn help      Trợ giúp"
    echo ""

    if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
        echo -e "  ${YELLOW}Chạy lệnh sau để dùng ngay:${NC}"
        echo "    source ~/.bashrc"
        echo ""
    fi

    log_warn "Đăng xuất/đăng nhập lại để áp dụng đầy đủ"
    echo ""
}

# Main
main() {
    header
    install_fcitx5
    install_addon
    install_cli
    setup_environment
    configure_fcitx5
    set_default_im
    start_fcitx5
    print_summary
}

main
