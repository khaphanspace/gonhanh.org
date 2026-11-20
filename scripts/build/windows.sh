#!/bin/bash
# GoNhanh Windows build (Git Bash / CI wrapper).
# The Windows app is a single native exe (C++ Win32 UI + Rust core) built with CMake + MSVC;
# the real work lives in scripts/build-windows.ps1. On other platforms this is a no-op so
# `make build` keeps working on macOS/Linux.
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

is_windows() {
    [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]] || [[ -n "$WINDIR" ]]
}

if ! is_windows; then
    echo "Skipped Windows build: not running on Windows (use GitHub Actions or a Windows machine)"
    exit 0
fi

ARGS=()
for arg in "$@"; do
    case $arg in
        --clean) ARGS+=("-Clean") ;;
        --package) ARGS+=("-Package") ;;
        --run) ARGS+=("-Run") ;;
        --help|-h)
            echo "Usage: scripts/build/windows.sh [--clean] [--package] [--run]"
            exit 0
            ;;
    esac
done

cd "$PROJECT_ROOT"
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build-windows.ps1 "${ARGS[@]}"
