#!/bin/bash
set -e

# GoNhanh Windows Build Script (Native C++)
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

    rm -rf "$PROJECT_ROOT/platforms/windows/build" 2>/dev/null || true
    rm -rf "$PROJECT_ROOT/core/target" 2>/dev/null || true
    echo "  Done"
    echo ""
fi

# Get version from git tag
GIT_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "v0.0.0")
VERSION=${GIT_TAG#v}

echo "Building GoNhanh for Windows (Native C++)"
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

# Build Rust core DLL
echo "[1/3] Building Rust core DLL..."
cd "$PROJECT_ROOT/core"
RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --target x86_64-pc-windows-msvc
echo "  Output: gonhanh_core.dll"

# Build native C++ app with CMake
echo "[2/3] Building native C++ app..."
cd "$PROJECT_ROOT/platforms/windows"

if ! command -v cmake &> /dev/null; then
    echo "Error: CMake not found"
    echo "Install from: https://cmake.org/download/"
    exit 1
fi

cmake -B build -G "Visual Studio 17 2022" -A x64 -DCMAKE_BUILD_TYPE=Release
cmake --build build --config Release

# Copy DLL to output
cp "$PROJECT_ROOT/core/target/x86_64-pc-windows-msvc/release/gonhanh_core.dll" \
   "$PROJECT_ROOT/platforms/windows/build/bin/Release/"
echo "  Output: platforms/windows/build/bin/Release/"

# Create ZIP package
echo "[3/3] Creating package..."
cd "$PROJECT_ROOT/platforms/windows/build/bin/Release"
ZIP_NAME="GoNhanh-${VERSION}-win-x64.zip"
rm -f "$ZIP_NAME" 2>/dev/null || true

if command -v zip &> /dev/null; then
    zip -q "$ZIP_NAME" GoNhanh.exe gonhanh_core.dll
elif command -v 7z &> /dev/null; then
    7z a -bso0 "$ZIP_NAME" GoNhanh.exe gonhanh_core.dll
else
    echo "  Warning: zip/7z not found, skipping package"
    ZIP_NAME=""
fi

if [ -n "$ZIP_NAME" ]; then
    echo "  Output: $ZIP_NAME"
fi

echo ""
echo "Build complete!"
