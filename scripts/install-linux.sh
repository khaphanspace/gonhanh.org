#!/bin/bash
# Gõ Nhanh Linux Installer
# curl -fsSL https://raw.githubusercontent.com/khaphanspace/gonhanh.org/main/scripts/install-linux.sh | bash
set -e

REPO="khaphanspace/gonhanh.org"
TMP=$(mktemp -d)
trap "rm -rf $TMP" EXIT

echo "Installing Gõ Nhanh..."

# Install Fcitx5 if needed
if ! command -v fcitx5 &>/dev/null; then
    echo "Installing Fcitx5..."
    if command -v apt &>/dev/null; then
        sudo apt update -qq && sudo apt install -y -qq fcitx5 fcitx5-configtool
    elif command -v dnf &>/dev/null; then
        sudo dnf install -y -q fcitx5 fcitx5-configtool
    elif command -v pacman &>/dev/null; then
        sudo pacman -S --noconfirm --quiet fcitx5 fcitx5-configtool
    fi
fi

# Download and install
cd "$TMP"
curl -fsSL "https://github.com/$REPO/releases/latest/download/gonhanh-linux.tar.gz" | tar xz
cd gonhanh-linux && ./install.sh

# Setup environment
for rc in ~/.bashrc ~/.zshrc; do
    [[ -f "$rc" ]] && ! grep -q "GTK_IM_MODULE=fcitx" "$rc" && cat >> "$rc" << 'EOF'
export GTK_IM_MODULE=fcitx QT_IM_MODULE=fcitx XMODIFIERS=@im=fcitx
EOF
done

echo ""
echo "✓ Done! Run: fcitx5 -r && fcitx5-configtool"
echo "  Add 'GoNhanh' in Input Method settings"
