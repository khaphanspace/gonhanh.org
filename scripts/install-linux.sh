#!/bin/bash
# Gõ Nhanh Linux Installer
# curl -fsSL https://raw.githubusercontent.com/khaphanspace/gonhanh.org/main/scripts/install-linux.sh | bash
set -e

REPO="khaphanspace/gonhanh.org"
TMP=$(mktemp -d)
trap "rm -rf $TMP" EXIT

echo "═══════════════════════════════════════"
echo "       Gõ Nhanh - Linux Installer      "
echo "═══════════════════════════════════════"
echo ""

# Detect distro and install Fcitx5 if needed
install_fcitx5() {
    if command -v fcitx5 &>/dev/null; then
        echo "✓ Fcitx5 đã được cài đặt"
        return 0
    fi

    echo "→ Đang cài đặt Fcitx5..."
    if command -v apt &>/dev/null; then
        sudo apt update -qq
        sudo apt install -y -qq fcitx5 fcitx5-configtool im-config
    elif command -v dnf &>/dev/null; then
        sudo dnf install -y -q fcitx5 fcitx5-configtool
    elif command -v pacman &>/dev/null; then
        sudo pacman -S --noconfirm --quiet fcitx5 fcitx5-configtool
    else
        echo "✗ Không hỗ trợ distro này. Vui lòng cài fcitx5 thủ công."
        exit 1
    fi
    echo "✓ Đã cài đặt Fcitx5"
}

# Download and install GoNhanh
install_gonhanh() {
    echo "→ Đang tải Gõ Nhanh..."
    cd "$TMP"
    curl -fsSL "https://github.com/$REPO/releases/latest/download/gonhanh-linux.tar.gz" | tar xz
    cd gonhanh-linux && ./install.sh
    echo "✓ Đã cài đặt Gõ Nhanh"
}

# Setup environment variables
setup_environment() {
    echo "→ Đang cấu hình môi trường..."

    ENV_VARS='export GTK_IM_MODULE=fcitx
export QT_IM_MODULE=fcitx
export XMODIFIERS=@im=fcitx'

    for rc in ~/.bashrc ~/.zshrc ~/.profile; do
        if [[ -f "$rc" ]] && ! grep -q "GTK_IM_MODULE=fcitx" "$rc"; then
            echo "" >> "$rc"
            echo "# Gõ Nhanh - Vietnamese Input Method" >> "$rc"
            echo "$ENV_VARS" >> "$rc"
        fi
    done

    # Export for current session
    export GTK_IM_MODULE=fcitx
    export QT_IM_MODULE=fcitx
    export XMODIFIERS=@im=fcitx

    echo "✓ Đã cấu hình biến môi trường"
}

# Auto-configure Fcitx5 to use GoNhanh
configure_fcitx5() {
    echo "→ Đang cấu hình Fcitx5..."

    FCITX5_CONFIG_DIR="$HOME/.config/fcitx5"
    PROFILE="$FCITX5_CONFIG_DIR/profile"

    mkdir -p "$FCITX5_CONFIG_DIR"

    # Create or update Fcitx5 profile to include GoNhanh
    if [[ -f "$PROFILE" ]]; then
        # Check if GoNhanh already in profile
        if grep -q "gonhanh" "$PROFILE"; then
            echo "✓ GoNhanh đã có trong cấu hình Fcitx5"
            return 0
        fi

        # Add GoNhanh to existing profile
        # Find the Groups section and add gonhanh
        if grep -q "^\[Groups/0/Items/0\]" "$PROFILE"; then
            # Get the next item number
            LAST_ITEM=$(grep -oP "Groups/0/Items/\K[0-9]+" "$PROFILE" | sort -n | tail -1)
            NEXT_ITEM=$((LAST_ITEM + 1))

            echo "" >> "$PROFILE"
            echo "[Groups/0/Items/$NEXT_ITEM]" >> "$PROFILE"
            echo "Name=gonhanh" >> "$PROFILE"
        fi
    else
        # Create new profile with keyboard and gonhanh
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
    fi

    echo "✓ Đã thêm GoNhanh vào Fcitx5"
}

# Start/restart Fcitx5
start_fcitx5() {
    echo "→ Đang khởi động Fcitx5..."

    # Kill existing fcitx5 if running
    pkill -9 fcitx5 2>/dev/null || true
    sleep 0.5

    # Start fcitx5 in background
    nohup fcitx5 -d &>/dev/null &
    sleep 1

    if pgrep -x fcitx5 &>/dev/null; then
        echo "✓ Fcitx5 đang chạy"
    else
        echo "! Không thể khởi động Fcitx5. Vui lòng chạy: fcitx5 -d"
    fi
}

# Set Fcitx5 as default IM (for supported systems)
set_default_im() {
    # For systems using im-config (Debian/Ubuntu)
    if command -v im-config &>/dev/null; then
        im-config -n fcitx5 2>/dev/null || true
    fi

    # For systems using imsettings (Fedora)
    if command -v imsettings-switch &>/dev/null; then
        imsettings-switch fcitx5 2>/dev/null || true
    fi
}

# Main installation
main() {
    install_fcitx5
    install_gonhanh
    setup_environment
    configure_fcitx5
    set_default_im
    start_fcitx5

    echo ""
    echo "═══════════════════════════════════════"
    echo "✓ Cài đặt hoàn tất!"
    echo "═══════════════════════════════════════"
    echo ""
    echo "Sử dụng:"
    echo "  • Ctrl+Space  - Bật/tắt gõ tiếng Việt"
    echo "  • Telex       - Mặc định (aa→â, aw→ă, dd→đ)"
    echo ""
    echo "Đổi sang VNI:"
    echo "  mkdir -p ~/.config/gonhanh && echo 'vni' > ~/.config/gonhanh/method"
    echo "  fcitx5 -r"
    echo ""
    echo "Lưu ý: Có thể cần đăng xuất/đăng nhập lại để áp dụng đầy đủ."
    echo ""
}

main
