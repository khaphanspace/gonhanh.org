#!/bin/bash
set -e

# Build script for Windows (run on Windows with Git Bash or WSL)
# Usage: ./scripts/build-windows.sh [OPTIONS]

# Source rustup environment
if [ -f "$HOME/.cargo/env" ]; then
    source "$HOME/.cargo/env"
fi

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
            echo "  --clean    Remove existing GoNhanh build artifacts before building"
            echo "  --help     Show this help message"
            exit 0
            ;;
    esac
done

# Clean install: remove existing build artifacts
if [ "$CLEAN_INSTALL" = true ]; then
    echo "Cleaning existing build artifacts..."

    # Kill running GoNhanh processes
    if tasklist 2>/dev/null | grep -i "GoNhanh.exe" > /dev/null 2>&1; then
        echo "Stopping running GoNhanh processes..."
        taskkill //F //IM "GoNhanh.exe" 2>/dev/null || true
        sleep 1
        echo "GoNhanh processes stopped."
    else
        echo "No running GoNhanh process found."
    fi

    # Remove build directories
    rm -rf "$(dirname "$0")/../platforms/windows/GoNhanh/bin" 2>/dev/null || true
    rm -rf "$(dirname "$0")/../platforms/windows/GoNhanh/obj" 2>/dev/null || true
    rm -rf "$(dirname "$0")/../platforms/windows/publish" 2>/dev/null || true
    rm -rf "$(dirname "$0")/../core/target" 2>/dev/null || true

    echo "Clean complete!"
    echo ""
fi

echo "Building Windows app..."

# Get version from git tag
GIT_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "v0.0.0")
VERSION=${GIT_TAG#v}  # Remove 'v' prefix
echo "Version from git tag: $VERSION"

# Navigate to project root
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Build Rust core
echo ""
echo "Building Rust core library..."
cd "$PROJECT_ROOT/core"

# Check if running on Windows
if [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]] || [[ -n "$WINDIR" ]]; then
    cargo build --release --target x86_64-pc-windows-msvc

    # Copy DLL to Windows project
    echo "Copying gonhanh_core.dll..."
    mkdir -p "$PROJECT_ROOT/platforms/windows/GoNhanh/Native"
    cp "target/x86_64-pc-windows-msvc/release/gonhanh_core.dll" \
       "$PROJECT_ROOT/platforms/windows/GoNhanh/Native/gonhanh_core.dll"
else
    echo "Warning: Not running on Windows. Cross-compilation not supported."
    echo "Please run this script on Windows with Git Bash or WSL."
    exit 1
fi

# Build WPF app
echo ""
echo "Building WPF application..."
cd "$PROJECT_ROOT/platforms/windows/GoNhanh"

# Check if .NET SDK is available
if ! command -v dotnet &> /dev/null; then
    echo "Error: .NET SDK not found!"
    echo "Please install .NET 8.0 SDK from: https://dotnet.microsoft.com/download"
    exit 1
fi

# Build and publish
dotnet publish -c Release -r win-x64 --self-contained false \
    -p:Version="$VERSION" \
    -p:FileVersion="$VERSION" \
    -p:AssemblyVersion="${VERSION%%.*}.0.0.0" \
    -o ../publish

echo ""
echo "Windows app built successfully!"
echo "Output: platforms/windows/publish/"

# Create ZIP package
echo ""
echo "Creating ZIP package..."
cd "$PROJECT_ROOT/platforms/windows"
ZIP_NAME="GoNhanh-${VERSION}-win-x64.zip"

if command -v zip &> /dev/null; then
    zip -r "$ZIP_NAME" publish/*
elif command -v 7z &> /dev/null; then
    7z a "$ZIP_NAME" publish/*
else
    echo "Warning: Neither zip nor 7z found. Skipping ZIP creation."
    echo "You can manually create the ZIP from: platforms/windows/publish/"
    exit 0
fi

echo ""
echo "Package created: platforms/windows/$ZIP_NAME"
