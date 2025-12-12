#!/bin/bash
# Test script for Linux platform
# Usage: ./scripts/test.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LINUX_DIR="$(dirname "$SCRIPT_DIR")"
ROOT_DIR="$(dirname "$(dirname "$LINUX_DIR")")"
CORE_DIR="$ROOT_DIR/core"

echo "=== Running Gõ Nhanh Linux Tests ==="
echo ""

# Step 1: Build Rust core (required for RustBridge tests)
echo "=== Step 1: Building Rust core ==="
cd "$CORE_DIR"
cargo build --release
echo ""

# Step 2: Build C++ tests
echo "=== Step 2: Building C++ tests ==="
cd "$LINUX_DIR"
mkdir -p build
cd build

cmake .. -DCMAKE_BUILD_TYPE=Release -DBUILD_TESTS=ON
make -j$(nproc)

# Step 3: Run tests
echo ""
echo "=== Step 3: Running tests ==="

# Check if tests were built
if [[ ! -f "keycodemap_test" ]]; then
    echo "Error: Tests not built. Is GTest installed?"
    echo "Install with: sudo apt install libgtest-dev"
    exit 1
fi

# Run KeycodeMap tests
echo ""
echo "--- KeycodeMap Tests ---"
./keycodemap_test --gtest_color=yes

# Run RustBridge tests (requires Rust library)
if [[ -f "rustbridge_test" ]]; then
    echo ""
    echo "--- RustBridge Tests ---"
    LD_LIBRARY_PATH="$CORE_DIR/target/release:$LD_LIBRARY_PATH" ./rustbridge_test --gtest_color=yes
else
    echo "Warning: RustBridge tests not built (Rust library may be missing)"
fi

echo ""
echo "=== All tests passed ==="
