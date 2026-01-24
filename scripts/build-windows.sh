#!/bin/bash
set -e

# GoNhanh Windows Build Script (CMake + Rust)

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Parse arguments
CLEAN=false
PACKAGE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --clean)
            CLEAN=true
            shift
            ;;
        --package|-p)
            PACKAGE=true
            shift
            ;;
        --help|-h)
            echo "Usage: build-windows.sh [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --clean      Clean build artifacts before building"
            echo "  --package    Create ZIP package for distribution"
            echo "  --help       Show this help message"
            echo ""
            echo "Examples:"
            echo "  ./build-windows.sh              # Build only"
            echo "  ./build-windows.sh --package    # Build and create ZIP"
            exit 0
            ;;
        *)
            shift
            ;;
    esac
done

# Check Windows
is_windows() {
    [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]] || [[ -n "$WINDIR" ]]
}

if ! is_windows; then
    echo "Skipped: Not running on Windows"
    exit 0
fi

# Get version
GIT_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "v1.0.0")
VERSION=${GIT_TAG#v}

echo "Building GoNhanh for Windows"
echo "Version: $VERSION"
echo ""

# Build directory (outside project to avoid path issues)
BUILD_DIR="$LOCALAPPDATA/gonhanh-build/windows"

# Clean
if [ "$CLEAN" = true ]; then
    echo "Cleaning build artifacts..."
    rm -rf "$BUILD_DIR"
    rm -rf "$PROJECT_ROOT/core/target"
    echo ""
fi

# Configure CMake if needed
if [ ! -f "$BUILD_DIR/gonhanh.vcxproj" ]; then
    echo "[1/2] Configuring CMake..."
    mkdir -p "$BUILD_DIR"
    cd "$BUILD_DIR"
    cmake "$PROJECT_ROOT/platforms/windows" -G "Visual Studio 17 2022" -A x64
    echo ""
fi

# Build
echo "[2/2] Building..."
cd "$BUILD_DIR"
cmake --build . --config Release --target gonhanh

# Copy to publish
mkdir -p "$PROJECT_ROOT/platforms/windows/publish"
cp "$BUILD_DIR/Release/gonhanh.exe" "$PROJECT_ROOT/platforms/windows/publish/GoNhanh.exe"

echo ""
echo "Build complete!"
echo "Output: platforms/windows/publish/GoNhanh.exe"

# Package if requested
if [ "$PACKAGE" = true ]; then
    echo ""
    echo "Creating package..."
    cd "$PROJECT_ROOT/platforms/windows"
    ZIP_NAME="GoNhanh-${VERSION}-win-x64.zip"
    rm -f "$ZIP_NAME" 2>/dev/null || true

    if command -v zip &> /dev/null; then
        zip -j "$ZIP_NAME" publish/GoNhanh.exe
    elif command -v 7z &> /dev/null; then
        7z a -bso0 "$ZIP_NAME" ./publish/GoNhanh.exe
    else
        echo "Warning: zip/7z not found, skipping package"
        exit 0
    fi

    echo "Package: platforms/windows/$ZIP_NAME"
fi
