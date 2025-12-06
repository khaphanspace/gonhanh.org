# GoNhanh - Project Structure

## Directory Tree

```
gonhanh.org/
│
├── README.md                    # Project overview
├── LICENSE                      # GPL-3.0 license
├── CONTRIBUTING.md              # Contribution guide
├── PROJECT_STRUCTURE.md         # This file
├── .gitignore                   # Git ignore rules
│
├── core/                        # 🦀 Rust Core Library
│   ├── Cargo.toml              # Rust dependencies
│   ├── src/
│   │   ├── lib.rs              # FFI exports
│   │   ├── engine.rs           # Vietnamese conversion (Telex/VNI)
│   │   ├── keyboard.rs         # Keyboard hooks (rdev)
│   │   └── config.rs           # Configuration management
│   └── tests/
│       └── engine_test.rs      # Unit tests
│
├── platforms/                   # Platform-specific apps
│   │
│   ├── macos/                  # 🍎 macOS Native App
│   │   ├── App.swift               # Entry point
│   │   ├── MenuBar.swift           # System tray
│   │   ├── SettingsView.swift      # Settings UI (SwiftUI)
│   │   ├── RustBridge.swift        # FFI bridge
│   │   ├── Info.plist              # App metadata
│   │   ├── GoNhanh.xcodeproj/      # Xcode project (to be created)
│   │   └── libgonhanh_core.a       # Built Rust library (gitignored)
│   │
│   └── windows/                # 🪟 Windows App (planned)
│       └── GoNhanh/
│
├── scripts/                     # 🔧 Build Scripts
│   ├── setup.sh                # Initial setup
│   ├── build-core.sh           # Build Rust core
│   └── build-macos.sh          # Build macOS app
│
├── docs/                        # 📚 Documentation
│   ├── architecture.md         # Architecture overview
│   └── development.md          # Development guide
│
└── assets/                      # 🎨 Resources
    └── icon.png                # App icon (to be added)
```

## File Count

- **Rust files**: 5
- **Swift files**: 4
- **Scripts**: 3
- **Documentation**: 5
- **Total**: ~19 files

## Key Technologies

### Core
- **Language**: Rust 2021 edition
- **Dependencies**: rdev, enigo, serde, toml
- **Build**: Static library (.a)

### macOS
- **Language**: Swift 5.9+
- **Framework**: SwiftUI + Cocoa
- **Target**: macOS 13.0+

## Build Artifacts

### Development
```
core/target/
└── release/
    └── libgonhanh_core.a

platforms/macos/
├── build/
│   └── Release/
│       └── GoNhanh.app
└── libgonhanh_core.a (universal binary)
```

### Distribution
- macOS: `GoNhanh.app` (~3 MB)
- Windows: `GoNhanh.exe` (~3 MB, planned)

## Getting Started

1. **Setup**: `./scripts/setup.sh`
2. **Build Core**: `./scripts/build-core.sh`
3. **Create Xcode Project**: Open Xcode, create project in `platforms/macos/`
4. **Build App**: `./scripts/build-macos.sh`

See `docs/development.md` for detailed instructions.

## Architecture

```
┌──────────────┐
│   SwiftUI    │  ← Platform UI
└──────┬───────┘
       │ FFI
┌──────▼───────┐
│  Rust Core   │  ← Business logic
└──────────────┘
```

## Next Steps

- [ ] Create Xcode project
- [ ] Add app icon
- [ ] Implement full Telex/VNI rules
- [ ] Add keyboard shortcuts
- [ ] Windows port
- [ ] Linux port (GTK)

---

Generated: 2024-12-06
Version: 0.1.0
