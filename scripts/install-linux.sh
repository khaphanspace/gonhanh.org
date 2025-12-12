#!/bin/bash
# Gõ Nhanh Linux Installer
# Usage: curl -fsSL https://gonhanh.vn/install-linux.sh | bash
set -e

REPO="khaphanspace/gonhanh.org"
TMP_DIR=$(mktemp -d)
trap "rm -rf $TMP_DIR" EXIT

echo "╔══════════════════════════════════════╗"
echo "║     Gõ Nhanh - Vietnamese IME        ║"
echo "║         Linux Installer              ║"
echo "╚══════════════════════════════════════╝"
echo ""

# Detect package manager and install Fcitx5 if needed
install_fcitx5() {
    if command -v fcitx5 &>/dev/null; then
        echo "✓ Fcitx5 already installed"
        return
    fi

    echo "Installing Fcitx5..."
    if command -v apt &>/dev/null; then
        sudo apt update && sudo apt install -y fcitx5 fcitx5-configtool
    elif command -v dnf &>/dev/null; then
        sudo dnf install -y fcitx5 fcitx5-configtool
    elif command -v pacman &>/dev/null; then
        sudo pacman -S --noconfirm fcitx5 fcitx5-configtool
    else
        echo "Error: Unsupported package manager. Please install fcitx5 manually."
        exit 1
    fi
    echo "✓ Fcitx5 installed"
}

# Download and install
install_gonhanh() {
    echo "Downloading Gõ Nhanh..."
    URL="https://github.com/$REPO/releases/latest/download/gonhanh-linux.tar.gz"

    cd "$TMP_DIR"
    curl -fsSL -o "gonhanh-linux.tar.gz" "$URL"

    echo "Installing..."
    tar xzf "gonhanh-linux.tar.gz"
    cd "gonhanh-linux"
    ./install.sh
}

# Setup environment
setup_env() {
    SHELL_RC=""
    if [[ -f "$HOME/.zshrc" ]]; then
        SHELL_RC="$HOME/.zshrc"
    elif [[ -f "$HOME/.bashrc" ]]; then
        SHELL_RC="$HOME/.bashrc"
    fi

    if [[ -n "$SHELL_RC" ]]; then
        if ! grep -q "GTK_IM_MODULE=fcitx" "$SHELL_RC" 2>/dev/null; then
            echo "" >> "$SHELL_RC"
            echo "# Fcitx5 IME" >> "$SHELL_RC"
            echo "export GTK_IM_MODULE=fcitx" >> "$SHELL_RC"
            echo "export QT_IM_MODULE=fcitx" >> "$SHELL_RC"
            echo "export XMODIFIERS=@im=fcitx" >> "$SHELL_RC"
            echo "✓ Environment variables added to $SHELL_RC"
        fi
    fi
}

# Main
install_fcitx5
install_gonhanh
setup_env

echo ""
echo "╔══════════════════════════════════════╗"
echo "║       Installation Complete!         ║"
echo "╚══════════════════════════════════════╝"
echo ""
echo "Next steps:"
echo "  1. Restart Fcitx5:  fcitx5 -r &"
echo "  2. Add Gõ Nhanh:    fcitx5-configtool"
echo "     → Input Method → Add → GoNhanh"
echo ""
echo "Enjoy typing Vietnamese! 🇻🇳"
