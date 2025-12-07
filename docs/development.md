# Development Guide

Complete guide for building, testing, and contributing to GoNhanh.

## Prerequisites

| Tool | Version | Install |
|------|---------|---------|
| Rust | 1.70+ | [rustup.rs](https://rustup.rs) |
| Xcode | 15+ | App Store |
| macOS | 13+ | System requirement |
| Git | 2.30+ | Pre-installed |

## Quick Start

```bash
# Clone repository
git clone https://github.com/khaphanspace/gonhanh.org
cd gonhanh.org

# Setup development environment (installs Rust targets)
make setup

# Run tests to verify setup
make test

# Build everything (Rust core + macOS app)
make build
```

## Project Structure

Complete directory layout with file purposes:

```
gonhanh.org/
│
├── Makefile                      # Build orchestration
├── CLAUDE.md                     # AI assistant guidelines
├── CONTRIBUTING.md               # Contributing guide
├── README.md                     # Project overview
├── LICENSE                       # GPL-3.0-or-later
│
├── core/                         # Rust core library (2068 lines)
│   ├── Cargo.toml               # Rust manifest
│   ├── src/
│   │   ├── lib.rs              # FFI exports (265 lines, 7 functions)
│   │   ├── data/
│   │   │   ├── mod.rs          # Module exports
│   │   │   ├── keys.rs         # Virtual key codes (240 lines)
│   │   │   ├── chars.rs        # Unicode character mappings
│   │   │   └── vowel.rs        # Phonology algorithm (350+ lines)
│   │   ├── engine/
│   │   │   ├── mod.rs          # Main engine (551 lines, 4-stage pipeline)
│   │   │   └── buffer.rs       # Typing buffer (max 32 chars)
│   │   └── input/
│   │       ├── mod.rs          # Input method trait
│   │       ├── telex.rs        # Telex input rules (80+ lines)
│   │       └── vni.rs          # VNI input rules (80+ lines)
│   │
│   └── tests/                  # Integration tests (160+ test cases)
│       ├── common/
│       │   └── mod.rs          # Test utilities
│       ├── basic_test.rs       # Single keystrokes (40+ tests)
│       ├── word_test.rs        # Full words (50+ tests)
│       ├── sentence_test.rs    # Multi-word sentences (20+ tests)
│       ├── behavior_test.rs    # User behaviors (20+ tests)
│       ├── common_issues_test.rs # Real bugs (15+ tests)
│       └── edge_cases_test.rs  # Boundary conditions (15+ tests)
│
├── platforms/
│   ├── linux/
│   │   └── .keep               # Linux stub (future)
│   ├── macos/                  # macOS SwiftUI app (765 lines)
│   │   ├── App.swift           # Entry point (28 lines)
│   │   ├── MenuBar.swift       # System tray (192 lines)
│   │   ├── SettingsView.swift  # Settings UI (102 lines)
│   │   ├── RustBridge.swift    # FFI bridge (443 lines)
│   │   ├── Info.plist          # App info
│   │   ├── GoNhanh.entitlements# App entitlements
│   │   └── GoNhanh.xcodeproj/  # Xcode project
│   │       ├── project.pbxproj # Build configuration
│   │       └── xcshareddata/   # Shared settings
│   └── windows/
│       └── .keep               # Windows stub (future)
│
├── scripts/                     # Build automation
│   ├── setup.sh                # Install Rust targets
│   ├── build-core.sh           # Build Rust library
│   ├── build-macos.sh          # Build SwiftUI app
│   └── build-macos-swift.sh    # Alternative build script
│
└── docs/                        # Documentation
    ├── architecture.md         # System architecture & design
    ├── development.md          # This file
    ├── vietnamese-language-system.md  # Linguistic reference
    └── common-issues.md        # Known issues & fixes
```

## Makefile Commands

Complete reference of all build targets:

### Setup & Cleanup

```bash
make help        # Show all available commands
make setup       # Install Rust targets for cross-compilation
make clean       # Remove all build artifacts
make lint        # Run cargo fmt and clippy
```

### Testing

```bash
make test        # Run all tests (main test target)
```

Runs: `cd core && cargo test --release`

### Building

```bash
make core        # Build Rust core library only
                 # Output: platforms/macos/libgonhanh_core.a (universal: arm64+x86_64)

make macos       # Build macOS SwiftUI app only
                 # Requires libgonhanh_core.a to exist
                 # Output: platforms/macos/build/Release/GoNhanh.app

make build       # Full build: test → core → macos
                 # Sequential: ensures tests pass before building app

make release     # Release workflow
                 # - Bumps version
                 # - Creates git tag
                 # - Triggers GitHub Actions CI/CD
```

### Installation

```bash
make install     # Copy app to /Applications
                 # Usage: make install [DEST=/path/to/apps]
```

## Development Workflow

### 1. Initial Setup

```bash
# First-time setup
git clone https://github.com/khaphanspace/gonhanh.org
cd gonhanh.org
make setup       # Installs rust targets (aarch64 + x86_64)
make test        # Verify setup (runs cargo test)
```

### 2. Making Changes

#### Rust Core Changes

```bash
# Edit core/src/*.rs files
vim core/src/engine/mod.rs

# Test changes
cargo test -p core

# Test specific module
cargo test -p core engine::

# See test output
cargo test -p core -- --nocapture --test-threads=1
```

#### Swift UI Changes

```bash
# Edit platforms/macos/*.swift files
vim platforms/macos/SettingsView.swift

# Build to see changes
make macos

# Run app from command line
open platforms/macos/build/Release/GoNhanh.app

# Or open in Xcode
open platforms/macos/GoNhanh.xcodeproj
```

### 3. Code Quality

```bash
# Format Rust code
cd core && cargo fmt --all

# Check for issues
cd core && cargo clippy --all -- -D warnings

# Full lint pass
make lint
```

### 4. Testing Before Commit

```bash
# Run full test suite
make test

# If tests pass, ready to commit
git add .
git commit -m "feat: add new feature"
```

## Testing Strategy

### Test Categories

| Category | File | Count | Focus |
|----------|------|-------|-------|
| **Basic** | `basic_test.rs` | 40+ | Single keystrokes, character conversions |
| **Words** | `word_test.rs` | 50+ | Full Vietnamese words (all input methods) |
| **Sentences** | `sentence_test.rs` | 20+ | Multi-word typing sequences |
| **Behavior** | `behavior_test.rs` | 20+ | User interactions (backspace, corrections) |
| **Real Issues** | `common_issues_test.rs` | 15+ | Chrome autocomplete, Excel tone loss |
| **Edge Cases** | `edge_cases_test.rs` | 15+ | Boundary conditions, buffer limits |

### Running Tests

```bash
# Run all tests (default)
make test

# Run with verbose output
cargo test -p core -- --nocapture

# Run specific test file
cargo test -p core --test word_test

# Run specific test
cargo test -p core vni_delayed_d_input

# Run with logging enabled
RUST_LOG=debug cargo test -p core -- --nocapture

# Run single-threaded (useful for debugging)
cargo test -p core -- --test-threads=1 --nocapture

# Run and stop on first failure
cargo test -p core -- --test-threads=1
```

### Writing New Tests

Example test structure:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telex_basic() {
        let result = type_text("hello", Method::Telex);
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_vni_tone() {
        let result = type_text("a1", Method::Vni);
        assert_eq!(result, "á");
    }
}
```

## Building

### Build Rust Core Only

```bash
make core

# Output location
ls -la platforms/macos/libgonhanh_core.a

# Verify it's universal (arm64 + x86_64)
lipo -info platforms/macos/libgonhanh_core.a
# Output: Architectures in the fat file are: x86_64 arm64
```

### Build macOS App

```bash
# Requires core library to exist
make macos

# Output location
ls -la platforms/macos/build/Release/GoNhanh.app

# Get app size
du -sh platforms/macos/build/Release/GoNhanh.app
# ~3-5 MB for the app bundle
```

### Full Release Build

```bash
# Build + test + package
make build

# Then open in Finder
open platforms/macos/build/Release/
```

## Debugging

### Rust Core Debugging

```bash
# Run test with debug output
RUST_LOG=debug cargo test -p core -- --nocapture

# Attach to test in debugger
lldb -- cargo test -p core

# Inside lldb
(lldb) run
(lldb) break set -n test_name
(lldb) continue
```

### macOS App Debugging

#### Method 1: Console.app

```bash
# 1. Build app
make macos

# 2. Run it
open platforms/macos/build/Release/GoNhanh.app

# 3. Open Console.app and filter by "GoNhanh"
# Look for debug messages, errors
```

#### Method 2: Xcode Debugging

```bash
# 1. Open project in Xcode
open platforms/macos/GoNhanh.xcodeproj

# 2. Edit scheme to use Release build
# (Xcode menu → Product → Scheme → Edit Scheme)

# 3. Set breakpoints in Swift code

# 4. Run with debugger (Cmd+R)

# 5. Inspect variables, step through code
```

#### Method 3: System Events

```bash
# Watch keyboard events with system profiler
log stream --predicate 'eventMessage contains "GoNhanh"'
```

### Common Debugging Tasks

**"Keyboard hook not working"**
```bash
# Check if app has Accessibility permission
# System Settings → Privacy & Security → Accessibility
# App should be listed there

# Try granting manually
tccutil reset Accessibility
# Then grant permission again by running app
```

**"Library not found" during macOS build**
```bash
# Rebuild core library
make core

# Verify file exists and is readable
ls -la platforms/macos/libgonhanh_core.a
file platforms/macos/libgonhanh_core.a
```

**"Test failures after Rust update"**
```bash
make clean
make setup
make test
```

## Code Standards

### Rust Code Style

- **Formatting**: `cargo fmt` (enforced)
- **Linting**: `cargo clippy -- -D warnings` (no warnings)
- **Testing**: All changes require passing test suite
- **Documentation**: Add doc comments for public API

```rust
/// Process Vietnamese keystroke
///
/// # Arguments
/// * `key` - macOS virtual key code
/// * `caps` - Caps Lock state
///
/// # Returns
/// Result struct with output characters and action
pub fn ime_key(key: u16, caps: bool, ctrl: bool) -> *mut Result {
    // Implementation
}
```

### Swift Code Style

- **Formatting**: Xcode default (auto-format on save)
- **Naming**: Swift convention (camelCase for vars/functions, PascalCase for types)
- **Safety**: Use optional types, avoid force unwrap except where safe

```swift
// Good: proper error handling
if let result = ime_key(key: key, caps: capsLock, ctrl: false) {
    processResult(result)
} else {
    NSLog("IME processing failed")
}

// Avoid: force unwrap
let result = ime_key(...)! // ❌ Don't do this
```

### Commit Messages

Follow conventional commits format:

```
feat: add tone repositioning for complex vowels
fix: prevent double tone marks on ươ syllables
docs: update architecture documentation
test: add edge cases for mark removal
refactor: simplify vowel detection logic
perf: optimize phonology algorithm
chore: update dependencies
```

## Troubleshooting

### Build Issues

| Problem | Solution |
|---------|----------|
| "No such file or directory: libgonhanh_core.a" | Run `make core` first |
| "Failed to find Xcode" | Install Xcode from App Store, run `xcode-select --install` |
| "Rust target not found" | Run `make setup` to install targets |
| "Permission denied" on scripts | Run `chmod +x scripts/*.sh` |

### Test Issues

| Problem | Solution |
|---------|----------|
| "Test panic" | Run with `--nocapture` to see output |
| "All tests pass locally but fail on CI" | Check Rust version with `rustc --version`, may need upgrade |
| "Timeout during test" | Run with `--test-threads=1` to reduce parallelism |
| "Memory leak reports" | Normal for IME, using Valgrind: `valgrind --leak-check=no` |

### Runtime Issues

| Problem | Solution |
|---------|----------|
| IME not typing anything | Check Accessibility permission in System Settings |
| IME enabled but slow to respond | Check CPU usage, may need Accessibility reauth |
| Keyboard hook crashes app | Update Xcode: `xcode-select --install` |
| Settings not saving | Check ~/Library/Preferences/ for plist file |

## Performance Profiling

### Profiling Build Time

```bash
# See which files take longest to compile
cargo build -p core --timings

# Profile individual tests
time cargo test -p core
```

### Profiling Runtime Performance

```bash
# Use Instruments.app (in Xcode)
# 1. Open Xcode
# 2. Xcode → Open Developer Tool → Instruments
# 3. Select "Time Profiler"
# 4. Attach to GoNhanh process
# 5. Record and analyze
```

## Contributing

1. **Fork** the repository
2. **Create feature branch**: `git checkout -b feat/my-feature`
3. **Make changes** and add tests
4. **Run tests**: `make test`
5. **Format code**: `make lint`
6. **Commit**: Follow conventional commits
7. **Push** to your fork
8. **Create Pull Request** with description

See [CONTRIBUTING.md](../CONTRIBUTING.md) for detailed guidelines.

## Release Process

Releases are automated via GitHub Actions:

```bash
# 1. Prepare release
# Update version in Cargo.toml and Info.plist

# 2. Create git tag
git tag v1.2.3

# 3. Push to trigger CI/CD
git push origin v1.2.3

# 4. GitHub Actions automatically:
#    - Runs full test suite
#    - Builds release binaries
#    - Creates DMG installer
#    - Publishes release
```

## Useful Commands Reference

```bash
# Development
make setup          # First-time setup
make test           # Run tests
make lint           # Format & lint
make build          # Full build
make clean          # Clean artifacts

# Building
make core           # Build Rust core
make macos          # Build macOS app
make install        # Install to /Applications

# Debugging
cargo test -- --nocapture
RUST_LOG=debug cargo test
open platforms/macos/build/Release/GoNhanh.app

# Code quality
cargo fmt
cargo clippy
```

## Related Documentation

- Architecture details: [`docs/architecture.md`](architecture.md)
- Vietnamese language system: [`docs/vietnamese-language-system.md`](vietnamese-language-system.md)
- Common issues & fixes: [`docs/common-issues.md`](common-issues.md)
