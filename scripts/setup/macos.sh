#!/bin/bash
set -e

echo "🚀 Setting up GoNhanh development environment..."

# Source rustup if available
if [ -f "$HOME/.cargo/env" ]; then
    source "$HOME/.cargo/env"
fi

# Check Rust
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust not found. Please install: https://rustup.rs"
    exit 1
fi
echo "✅ Rust found: $(rustc --version)"
echo "📍 Cargo location: $(which cargo)"

# Install Rust targets for macOS
echo "📦 Installing Rust targets..."
rustup target add aarch64-apple-darwin
rustup target add x86_64-apple-darwin

# Check Xcode (macOS only)
if [[ "$OSTYPE" == "darwin"* ]]; then
    if ! command -v xcodebuild &> /dev/null; then
        echo "⚠️  Xcode not found. Please install from App Store."
    else
        echo "✅ Xcode found: $(xcodebuild -version | head -n 1)"
    fi
fi

# Make scripts executable
chmod +x scripts/**/*.sh

echo ""
echo "✅ Setup complete!"
echo ""
echo "Next steps:"
echo "  1. Build Rust core:  ./scripts/build/core.sh"
echo "  2. Create Xcode project in platforms/macos/"
echo "  3. Build macOS app:  ./scripts/build/macos.sh"
echo ""
echo "Documentation: docs/development.md"
