#!/bin/bash
set -e

# GoNhanh Windows Build Script
# Run on Windows with Git Bash or via CI/CD

# Source rustup environment
if [ -f "$HOME/.cargo/env" ]; then
    source "$HOME/.cargo/env"
fi

# Navigate to project root
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Check if running on Windows
is_windows() {
    [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]] || [[ -n "$WINDIR" ]]
}

# Parse arguments
CLEAN_INSTALL=false
for arg in "$@"; do
    case $arg in
        --clean)
            CLEAN_INSTALL=true
            shift
            ;;
        --help|-h)
            echo "Usage: build-windows.sh [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --clean    Remove existing build artifacts before building"
            echo "  --help     Show this help message"
            exit 0
            ;;
    esac
done

# Clean build artifacts
if [ "$CLEAN_INSTALL" = true ]; then
    echo "Cleaning build artifacts..."

    if is_windows; then
        # Kill running GoNhanh processes
        if tasklist 2>/dev/null | grep -qi "GoNhanh.exe"; then
            echo "  Stopping GoNhanh.exe..."
            taskkill //F //IM "GoNhanh.exe" 2>/dev/null || true
            sleep 1
        fi
    fi

    rm -rf "$PROJECT_ROOT/platforms/windows/GoNhanh/bin" 2>/dev/null || true
    rm -rf "$PROJECT_ROOT/platforms/windows/GoNhanh/obj" 2>/dev/null || true
    rm -rf "$PROJECT_ROOT/platforms/windows/publish" 2>/dev/null || true
    rm -rf "$PROJECT_ROOT/core/target" 2>/dev/null || true
    echo "  Done"
    echo ""
fi

# Get version from git tag
GIT_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "v0.0.0")
VERSION=${GIT_TAG#v}

echo "Building GoNhanh for Windows"
echo "Version: $VERSION"
echo ""

# Check platform
if ! is_windows; then
    echo "Skipped: Not running on Windows"
    echo ""
    echo "This script requires Windows (Git Bash)."
    echo "Use GitHub Actions for CI/CD builds."
    exit 0
fi

# Build Rust core
echo "[1/3] Building Rust core..."
cd "$PROJECT_ROOT/core"
cargo build --release --target x86_64-pc-windows-msvc

mkdir -p "$PROJECT_ROOT/platforms/windows/GoNhanh/Native"
cp "target/x86_64-pc-windows-msvc/release/gonhanh_core.dll" \
   "$PROJECT_ROOT/platforms/windows/GoNhanh/Native/gonhanh_core.dll"
echo "  Output: gonhanh_core.dll"

# Build WPF app
echo "[2/3] Building WPF app..."
cd "$PROJECT_ROOT/platforms/windows/GoNhanh"

if ! command -v dotnet &> /dev/null; then
    echo "Error: .NET SDK not found"
    echo "Install from: https://dotnet.microsoft.com/download"
    exit 1
fi

dotnet publish -c Release -r win-x64 --self-contained false \
    -p:Version="$VERSION" \
    -p:FileVersion="$VERSION" \
    -p:AssemblyVersion="${VERSION%%.*}.0.0.0" \
    -o ../publish \
    -v quiet

echo "  Output: platforms/windows/publish/"

# Create ZIP package
echo "[3/3] Creating package..."
cd "$PROJECT_ROOT/platforms/windows"
ZIP_NAME="GoNhanh-${VERSION}-win-x64.zip"
rm -f "$ZIP_NAME" 2>/dev/null || true

if command -v zip &> /dev/null; then
    zip -rq "$ZIP_NAME" publish/*
elif command -v 7z &> /dev/null; then
    7z a -bso0 "$ZIP_NAME" publish/*
else
    echo "  Warning: zip/7z not found, skipping package"
    ZIP_NAME=""
fi

if [ -n "$ZIP_NAME" ]; then
    echo "  Output: $ZIP_NAME"
fi

echo ""
echo "Build complete!"
