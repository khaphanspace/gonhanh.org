#!/bin/bash
set -eo pipefail

# GoNhanh Windows build: Win32 shell (Rust, windows-sys) + core in one crate → one GoNhanh.exe, no runtime.
# Needs the Rust toolchain and Visual Studio / Build Tools "Desktop development with C++" (MSVC linker, rc.exe).
# Run on Windows with Git Bash.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
SRC="$PROJECT_ROOT/platforms/windows"

if [[ "$OSTYPE" != "msys" && "$OSTYPE" != "cygwin" && -z "$WINDIR" ]]; then
    echo "Skipped: requires Windows (Git Bash)"
    exit 0
fi

if [ -f "$HOME/.cargo/env" ]; then
    source "$HOME/.cargo/env"
fi

GIT_TAG=$(git -C "$PROJECT_ROOT" describe --tags --abbrev=0 2>/dev/null || echo "v0.0.0")
VERSION=${GONHANH_VERSION:-$GIT_TAG}  # CI passes the release tag
export GONHANH_VERSION=${VERSION#v}   # read by platforms/windows/build.rs

echo "Building GoNhanh for Windows v$GONHANH_VERSION"
cd "$SRC"

echo "[1/3] Tests..."
cargo test --release --quiet

echo "[2/3] Build..."
cargo build --release --quiet

echo "[3/3] Package..."
PUBLISH="$SRC/publish"
mkdir -p "$PUBLISH" && rm -f "$PUBLISH"/*
cp "$SRC/target/release/GoNhanh.exe" "$PUBLISH/"
ZIP="$SRC/GoNhanh-$GONHANH_VERSION-win-x64.zip"
rm -f "$ZIP"
(cd "$PUBLISH" && powershell -NoProfile -Command "Compress-Archive -Path GoNhanh.exe -DestinationPath '$(cygpath -w "$ZIP")'")

echo "  Output: $(du -h "$PUBLISH/GoNhanh.exe" | cut -f1)  platforms/windows/publish/GoNhanh.exe"
echo "  Output: platforms/windows/$(basename "$ZIP")"
echo "Build complete!"
