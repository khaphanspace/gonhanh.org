# GoNhanh Daemon + UI Split Setup

## Overview

This document describes how to configure Xcode for the daemon + UI app split architecture.

**Goal:** Reduce background memory from ~47MB to <5MB by splitting into:
- **GoNhanhDaemon.xpc** (~3-5MB) - Headless keyboard processing
- **GoNhanh.app** (UI only) - Menu bar, settings window

## Files Created

### Daemon (`platforms/macos/Daemon/`)
| File | Purpose |
|------|---------|
| `main.swift` | XPC daemon entry point |
| `IMEDaemon.swift` | XPC listener, protocol implementation |
| `DaemonEngine.swift` | Engine code (RustBridge, TextInjector, KeyboardHookManager) |
| `DaemonHelpers.swift` | KeyboardShortcut, InputSourceObserver, SpecialPanelAppDetector |
| `Info.plist` | XPC service configuration |
| `GoNhanhDaemon.entitlements` | Daemon entitlements |

### Shared (`platforms/macos/Shared/`)
| File | Purpose |
|------|---------|
| `IMEProtocols.swift` | XPC protocols (IMEDaemonProtocol, IMEAppProtocol) |
| `IMESettings.swift` | Shared settings keys (IMESettingsKey) |

### UI App
| File | Purpose |
|------|---------|
| `IMEClient.swift` | XPC client for UI → Daemon communication |

## Xcode Configuration Steps

### Step 1: Create XPC Service Target

1. Open `GoNhanh.xcodeproj`
2. File → New → Target
3. Choose "XPC Service" template
4. Name: `GoNhanhDaemon`
5. Bundle ID: `org.gonhanh.GoNhanh.Daemon`
6. Language: Swift

### Step 2: Configure Daemon Target

1. **Add source files to Daemon target:**
   - `Daemon/main.swift`
   - `Daemon/IMEDaemon.swift`
   - `Daemon/DaemonEngine.swift`
   - `Daemon/DaemonHelpers.swift`
   - `Shared/IMEProtocols.swift`
   - `Shared/IMESettings.swift`

2. **Link Rust library:**
   - Add `libgonhanh_core.a` to "Link Binary With Libraries"
   - Add library search path: `$(PROJECT_DIR)/..` or wherever the .a file is

3. **Configure Info.plist:**
   - Use `Daemon/Info.plist`
   - Ensure `CFBundlePackageType` = `XPC!`
   - Ensure `XPCService.ServiceType` = `Application`

4. **Configure entitlements:**
   - Use `Daemon/GoNhanhDaemon.entitlements`

5. **Build Settings:**
   - `INFOPLIST_FILE` = `Daemon/Info.plist`
   - `CODE_SIGN_ENTITLEMENTS` = `Daemon/GoNhanhDaemon.entitlements`
   - `PRODUCT_BUNDLE_IDENTIFIER` = `org.gonhanh.GoNhanh.Daemon`

### Step 3: Configure UI App Target

1. **Add source files to App target:**
   - `IMEClient.swift`
   - `Shared/IMEProtocols.swift`
   - `Shared/IMESettings.swift`

2. **Remove from App target** (now in daemon):
   - Engine initialization code in `startEngine()`
   - Direct RustBridge calls (replace with IMEClient calls)
   - Remove `libgonhanh_core.a` from App linking

3. **Embed XPC Service:**
   - In App target → General → "Frameworks, Libraries, and Embedded Content"
   - Add `GoNhanhDaemon.xpc` with "Embed Without Signing"

### Step 4: Update App Code

1. **App.swift** - Initialize XPC client:
   ```swift
   func applicationDidFinishLaunching(_ notification: Notification) {
       IMEClient.shared.connect()
       // ... rest of initialization
   }
   ```

2. **MenuBar.swift** - Replace RustBridge calls:
   ```swift
   // Before:
   RustBridge.setEnabled(enabled)

   // After:
   IMEClient.shared.setEnabled(enabled)
   ```

3. **AppState** - Use XPC client:
   ```swift
   @Published var isEnabled: Bool {
       didSet {
           IMEClient.shared.setEnabled(isEnabled)
           // ...
       }
   }
   ```

### Step 5: Build & Test

1. Build both targets
2. Run the app
3. Verify daemon starts (check Activity Monitor)
4. Test Vietnamese typing
5. Check memory usage:
   ```bash
   vmmap -summary $(pgrep GoNhanhDaemon) | grep "Physical footprint"
   ```

## Expected Memory Usage

| Component | Before | After |
|-----------|--------|-------|
| GoNhanh.app (total) | ~47MB | N/A |
| GoNhanhDaemon.xpc | N/A | ~3-5MB |
| GoNhanh.app (UI only) | N/A | ~15-25MB (when open) |
| **Background memory** | **~47MB** | **~3-5MB** |

## Troubleshooting

### Daemon not starting
- Check Console.app for XPC errors
- Verify bundle ID matches in Info.plist and IMEXPCService.serviceName
- Ensure daemon is embedded in app bundle

### XPC connection failing
- Check code signing (both app and daemon must be signed)
- Verify entitlements are correct
- Check if daemon process is running: `pgrep GoNhanhDaemon`

### Accessibility not working
- Daemon needs Accessibility permission (inherited from app)
- May need to re-grant permission after code signing changes
