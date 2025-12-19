# Đề xuất: Ưu tiên gõ tiếng Nhật khi gõ (Issue #19)

## Tóm tắt vấn đề

**Issue:** https://github.com/khaphanspace/gonhanh.org/issues/19

**Bối cảnh:**
- User dùng bộ gõ mặc định macOS: Tiếng Nhật (ví dụ: Hiragana, Katakana)
- User dùng Gõ Nhanh để gõ nhanh tiếng Việt
- **Vấn đề:** Khi đang ở chế độ tiếng Nhật, Gõ Nhanh vẫn can thiệp và chuyển đổi ký tự

**Mong muốn:**
| macOS Input Source | Gõ Nhanh | Hành vi mong đợi |
|-------------------|----------|------------------|
| Tiếng Nhật (Japanese) | Enabled | → **Disable tạm thời** (ưu tiên Nhật) |
| Tiếng Anh (ABC/US) | Enabled | → **Active** (ưu tiên Việt) |
| Tiếng Trung/Hàn/Thái | Enabled | → **Disable tạm thời** |

---

## Phân tích kỹ thuật

### 1. Cách phát hiện Input Source trên macOS

macOS cung cấp API để detect và listen input source changes:

```swift
import Carbon.HIToolbox

// Lấy input source hiện tại
func getCurrentInputSource() -> String? {
    guard let source = TISCopyCurrentKeyboardInputSource()?.takeRetainedValue() else {
        return nil
    }
    guard let sourceID = TISGetInputSourceProperty(source, kTISPropertyInputSourceID) else {
        return nil
    }
    return Unmanaged<CFString>.fromOpaque(sourceID).takeUnretainedValue() as String
}

// Ví dụ output:
// - "com.apple.keylayout.ABC"              → English (US)
// - "com.apple.keylayout.USInternational"  → English (International)
// - "com.apple.inputmethod.Kotoeri.RomajiTyping.Japanese" → Japanese Hiragana
// - "com.apple.inputmethod.TCIM.Pinyin"    → Chinese Pinyin
// - "com.apple.inputmethod.Korean.2SetKorean" → Korean
```

### 2. Cách listen input source changes

```swift
import Carbon.HIToolbox

class InputSourceObserver {
    private var observer: UnsafeMutableRawPointer?

    func start() {
        let callback: CFNotificationCallback = { center, observer, name, object, userInfo in
            // Input source đã thay đổi - kiểm tra và enable/disable
            InputSourceObserver.handleInputSourceChange()
        }

        CFNotificationCenterAddObserver(
            CFNotificationCenterGetDistributedCenter(),
            Unmanaged.passUnretained(self).toOpaque(),
            callback,
            kTISNotifySelectedKeyboardInputSourceChanged as CFString,
            nil,
            .deliverImmediately
        )
    }

    static func handleInputSourceChange() {
        let isLatin = isCurrentInputSourceLatin()
        if isLatin {
            // Enable Gõ Nhanh
            RustBridge.setEnabled(AppState.shared.userWantsEnabled)
        } else {
            // Disable tạm thời Gõ Nhanh
            RustBridge.setEnabled(false)
        }
    }
}
```

### 3. Cách detect Latin-based vs Non-Latin input source

```swift
/// Kiểm tra input source có phải là Latin/ASCII-based không
func isCurrentInputSourceLatin() -> Bool {
    guard let source = TISCopyCurrentKeyboardInputSource()?.takeRetainedValue() else {
        return true // Default: cho phép
    }

    // Cách 1: Kiểm tra Script Code
    if let scriptPtr = TISGetInputSourceProperty(source, kTISPropertyScriptCode) {
        let script = Unmanaged<CFNumber>.fromOpaque(scriptPtr).takeUnretainedValue()
        var scriptCode: Int32 = 0
        CFNumberGetValue(script, .sInt32Type, &scriptCode)
        // smRoman = 0 (Latin), smJapanese = 1, smTradChinese = 2, etc.
        return scriptCode == 0 // smRoman
    }

    // Cách 2: Kiểm tra Input Source Type (fallback)
    if let typePtr = TISGetInputSourceProperty(source, kTISPropertyInputSourceType) {
        let type = Unmanaged<CFString>.fromOpaque(typePtr).takeUnretainedValue() as String
        // "TISTypeKeyboardLayout" = keyboard layout (thường là Latin)
        // "TISTypeKeyboardInputMode" = input method (thường là CJK, etc.)
        return type == "TISTypeKeyboardLayout" as String
    }

    return true
}
```

---

## Đề xuất Implementation

### Phương án A: Auto-detect và tự động disable (Đề xuất chính)

**Ưu điểm:**
- Zero-configuration - hoạt động tự động
- Giống cách EVKey hoạt động
- UX tốt nhất cho user

**Logic flow:**

```
┌──────────────────────────────────────────────────────────────────┐
│                    Input Source Changed                           │
│          (kTISNotifySelectedKeyboardInputSourceChanged)          │
└──────────────────────────────────────────────────────────────────┘
                                │
                                ▼
            ┌───────────────────────────────────┐
            │   isCurrentInputSourceLatin()?    │
            └───────────────────────────────────┘
                    │                   │
              Yes (Latin)          No (CJK/etc.)
                    │                   │
                    ▼                   ▼
    ┌───────────────────────┐  ┌───────────────────────┐
    │  AppState.shared      │  │  RustBridge           │
    │  .isInputSourceLatin  │  │  .setEnabled(false)   │
    │  = true               │  │                       │
    │                       │  │  (Tạm disable,        │
    │  Restore user's       │  │   không lưu setting)  │
    │  enabled preference   │  │                       │
    └───────────────────────┘  └───────────────────────┘
```

### Các file cần thay đổi

#### 1. `AppMetadata.swift` - Thêm setting key

```swift
enum SettingsKey {
    // ... existing keys ...
    static let autoDisableForNonLatin = "gonhanh.autoDisableForNonLatin"
}
```

#### 2. `RustBridge.swift` - Thêm InputSourceObserver class

```swift
// MARK: - Input Source Observer

class InputSourceObserver {
    static let shared = InputSourceObserver()

    private var isObserving = false

    private init() {}

    /// Start observing input source changes
    func start() {
        guard !isObserving else { return }
        isObserving = true

        let callback: CFNotificationCallback = { _, observer, _, _, _ in
            DispatchQueue.main.async {
                InputSourceObserver.shared.handleInputSourceChange()
            }
        }

        CFNotificationCenterAddObserver(
            CFNotificationCenterGetDistributedCenter(),
            nil,
            callback,
            kTISNotifySelectedKeyboardInputSourceChanged as CFString,
            nil,
            .deliverImmediately
        )

        // Check initial state
        handleInputSourceChange()
        Log.info("InputSourceObserver started")
    }

    func stop() {
        guard isObserving else { return }
        isObserving = false

        CFNotificationCenterRemoveObserver(
            CFNotificationCenterGetDistributedCenter(),
            nil,
            kTISNotifySelectedKeyboardInputSourceChanged as CFString,
            nil
        )
    }

    private func handleInputSourceChange() {
        let isLatin = isCurrentInputSourceLatin()
        let appState = AppState.shared

        // Chỉ xử lý khi feature enabled
        guard appState.autoDisableForNonLatin else { return }

        if isLatin {
            // Restore user preference
            appState.setInputSourceOverride(nil)
            Log.info("InputSource: Latin detected, restoring user preference")
        } else {
            // Tạm disable (không ảnh hưởng user preference)
            appState.setInputSourceOverride(false)
            Log.info("InputSource: Non-Latin detected, temporarily disabled")
        }
    }

    /// Check if current input source is Latin/ASCII-based
    private func isCurrentInputSourceLatin() -> Bool {
        guard let source = TISCopyCurrentKeyboardInputSource()?.takeRetainedValue() else {
            return true
        }

        // Check script code - smRoman (0) = Latin
        if let scriptPtr = TISGetInputSourceProperty(source, kTISPropertyScriptCode) {
            let script = Unmanaged<CFNumber>.fromOpaque(scriptPtr).takeUnretainedValue()
            var scriptCode: Int32 = 0
            CFNumberGetValue(script, .sInt32Type, &scriptCode)

            // smRoman = 0, smJapanese = 1, smTradChinese = 2, smKorean = 3, etc.
            let isLatin = scriptCode == 0

            if let idPtr = TISGetInputSourceProperty(source, kTISPropertyInputSourceID) {
                let sourceId = Unmanaged<CFString>.fromOpaque(idPtr).takeUnretainedValue() as String
                Log.info("InputSource: \(sourceId), script=\(scriptCode), isLatin=\(isLatin)")
            }

            return isLatin
        }

        return true // Default: assume Latin
    }
}
```

#### 3. `MainSettingsView.swift` - Thêm state và UI

```swift
class AppState: ObservableObject {
    // ... existing properties ...

    /// User's actual preference (persisted)
    private var userWantsEnabled: Bool = true

    /// Override from InputSourceObserver (not persisted)
    private var inputSourceOverride: Bool? = nil

    /// Setting to enable/disable this feature
    @Published var autoDisableForNonLatin: Bool = true {
        didSet {
            UserDefaults.standard.set(autoDisableForNonLatin, forKey: SettingsKey.autoDisableForNonLatin)
            if autoDisableForNonLatin {
                InputSourceObserver.shared.start()
            } else {
                InputSourceObserver.shared.stop()
                setInputSourceOverride(nil) // Clear override
            }
        }
    }

    /// Effective enabled state (combines user preference + input source override)
    var effectiveEnabled: Bool {
        inputSourceOverride ?? userWantsEnabled
    }

    /// Called by InputSourceObserver to temporarily override
    func setInputSourceOverride(_ override: Bool?) {
        inputSourceOverride = override
        RustBridge.setEnabled(effectiveEnabled)
        NotificationCenter.default.post(name: .menuStateChanged, object: nil)
    }

    // Update isEnabled setter to track user preference
    @Published var isEnabled: Bool {
        didSet {
            userWantsEnabled = isEnabled
            RustBridge.setEnabled(effectiveEnabled)
            // ... rest of existing logic
        }
    }
}
```

#### 4. UI Settings (trong `MainSettingsView.swift`)

```swift
// Trong SettingsPage view
Toggle("Tự động tắt khi dùng IME khác (Nhật, Trung, Hàn...)", isOn: $appState.autoDisableForNonLatin)
    .help("Khi bật: Gõ Nhanh tự động tắt khi bạn chuyển sang bộ gõ tiếng Nhật, Trung, Hàn, v.v. để tránh xung đột.")
```

---

## Phương án B: Input Source Manager UI (ĐỀ XUẤT CHÍNH)

**Mô tả:** Xây dựng UI quản lý Input Sources ngay trong app, cho phép user:
1. Xem danh sách tất cả Input Sources đã cài trên máy
2. Toggle ON/OFF cho từng Input Source
3. Chuyển đổi Input Source trực tiếp từ menu bar của Gõ Nhanh

### Mockup UI

```
┌─────────────────────────────────────────────────────────────┐
│  Cài đặt > Quản lý bộ gõ                                    │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Gõ Nhanh sẽ TẮT khi bạn chuyển sang các bộ gõ sau:        │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ 🇯🇵  Japanese - Hiragana          [●] Tắt Gõ Nhanh  │   │
│  │ 🇯🇵  Japanese - Katakana          [●] Tắt Gõ Nhanh  │   │
│  │ 🇨🇳  Chinese - Pinyin             [●] Tắt Gõ Nhanh  │   │
│  │ 🇰🇷  Korean - 2-Set               [●] Tắt Gõ Nhanh  │   │
│  │ 🇹🇭  Thai - Kedmanee              [●] Tắt Gõ Nhanh  │   │
│  │ ─────────────────────────────────────────────────── │   │
│  │ 🇺🇸  ABC (English)                [○] Bật Gõ Nhanh  │   │
│  │ 🇺🇸  U.S. International           [○] Bật Gõ Nhanh  │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  [+] Thêm bộ gõ mới...  (mở System Preferences)            │
│                                                             │
│  ☑ Tự động phát hiện bộ gõ không phải Latin                │
│    (Mặc định tắt Gõ Nhanh cho các bộ gõ CJK mới thêm)      │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Menu Bar Integration

```
┌──────────────────────────────┐
│ ✓ Bật Gõ Nhanh               │
│ ─────────────────────────────│
│   Telex                      │
│ ✓ VNI                        │
│ ─────────────────────────────│
│   Chuyển bộ gõ          ▶   │  ┌────────────────────────┐
│ ─────────────────────────────│  │ ✓ 🇺🇸 ABC (English)    │
│   Cài đặt...                 │  │   🇯🇵 Japanese         │
│   Thoát                      │  │   🇨🇳 Chinese          │
└──────────────────────────────┘  └────────────────────────┘
```

### Data Model

```swift
/// Represents an input source on the system
struct InputSourceItem: Identifiable, Codable, Hashable {
    let id: String              // e.g., "com.apple.keylayout.ABC"
    let localizedName: String   // e.g., "ABC"
    let languageCode: String?   // e.g., "en", "ja", "zh"
    let scriptCode: Int32       // 0 = Latin, 1 = Japanese, 2 = Chinese...
    var disableGoNhanh: Bool    // User preference: disable Gõ Nhanh when active

    var isLatin: Bool { scriptCode == 0 }

    var flagEmoji: String {
        switch languageCode {
        case "ja": return "🇯🇵"
        case "zh": return "🇨🇳"
        case "ko": return "🇰🇷"
        case "th": return "🇹🇭"
        case "vi": return "🇻🇳"
        default: return "🇺🇸"
        }
    }
}
```

### API để lấy danh sách Input Sources

```swift
import Carbon.HIToolbox

class InputSourceManager {
    static let shared = InputSourceManager()

    /// Get all enabled input sources on the system
    func getEnabledInputSources() -> [InputSourceItem] {
        let properties: CFDictionary = [
            kTISPropertyInputSourceIsEnabled: true,
            kTISPropertyInputSourceIsSelectCapable: true
        ] as CFDictionary

        guard let sources = TISCreateInputSourceList(properties, false)?.takeRetainedValue() as? [TISInputSource] else {
            return []
        }

        return sources.compactMap { source -> InputSourceItem? in
            guard let idPtr = TISGetInputSourceProperty(source, kTISPropertyInputSourceID),
                  let namePtr = TISGetInputSourceProperty(source, kTISPropertyLocalizedName) else {
                return nil
            }

            let id = Unmanaged<CFString>.fromOpaque(idPtr).takeUnretainedValue() as String
            let name = Unmanaged<CFString>.fromOpaque(namePtr).takeUnretainedValue() as String

            // Get script code
            var scriptCode: Int32 = 0
            if let scriptPtr = TISGetInputSourceProperty(source, kTISPropertyScriptCode) {
                let script = Unmanaged<CFNumber>.fromOpaque(scriptPtr).takeUnretainedValue()
                CFNumberGetValue(script, .sInt32Type, &scriptCode)
            }

            // Get language codes
            var languageCode: String? = nil
            if let langsPtr = TISGetInputSourceProperty(source, kTISPropertyInputSourceLanguages) {
                let langs = Unmanaged<CFArray>.fromOpaque(langsPtr).takeUnretainedValue() as? [String]
                languageCode = langs?.first
            }

            // Default: disable Gõ Nhanh for non-Latin sources
            let disableGoNhanh = scriptCode != 0

            return InputSourceItem(
                id: id,
                localizedName: name,
                languageCode: languageCode,
                scriptCode: scriptCode,
                disableGoNhanh: disableGoNhanh
            )
        }
    }

    /// Switch to a specific input source
    func selectInputSource(id: String) {
        let properties: CFDictionary = [
            kTISPropertyInputSourceID: id
        ] as CFDictionary

        guard let sources = TISCreateInputSourceList(properties, false)?.takeRetainedValue() as? [TISInputSource],
              let source = sources.first else {
            return
        }

        TISSelectInputSource(source)
    }

    /// Get current input source ID
    func getCurrentInputSourceId() -> String? {
        guard let source = TISCopyCurrentKeyboardInputSource()?.takeRetainedValue(),
              let idPtr = TISGetInputSourceProperty(source, kTISPropertyInputSourceID) else {
            return nil
        }
        return Unmanaged<CFString>.fromOpaque(idPtr).takeUnretainedValue() as String
    }
}
```

### State Management

```swift
class AppState: ObservableObject {
    // ... existing properties ...

    /// All input sources on the system
    @Published var inputSources: [InputSourceItem] = []

    /// Current active input source ID
    @Published var currentInputSourceId: String?

    /// User preferences for each input source (persisted)
    private var inputSourcePreferences: [String: Bool] = [:] // id -> disableGoNhanh

    init() {
        // ... existing init ...
        loadInputSourcePreferences()
        refreshInputSources()
    }

    func refreshInputSources() {
        let sources = InputSourceManager.shared.getEnabledInputSources()

        // Apply saved preferences
        inputSources = sources.map { source in
            var item = source
            if let saved = inputSourcePreferences[source.id] {
                item.disableGoNhanh = saved
            }
            return item
        }

        currentInputSourceId = InputSourceManager.shared.getCurrentInputSourceId()
    }

    func setInputSourcePreference(id: String, disableGoNhanh: Bool) {
        inputSourcePreferences[id] = disableGoNhanh
        saveInputSourcePreferences()

        // Update local state
        if let index = inputSources.firstIndex(where: { $0.id == id }) {
            inputSources[index].disableGoNhanh = disableGoNhanh
        }

        // If this is the current source, apply immediately
        if id == currentInputSourceId {
            RustBridge.setEnabled(!disableGoNhanh && userWantsEnabled)
        }
    }

    func switchToInputSource(id: String) {
        InputSourceManager.shared.selectInputSource(id: id)
        // Observer will handle the rest
    }

    private func loadInputSourcePreferences() {
        inputSourcePreferences = UserDefaults.standard.dictionary(forKey: SettingsKey.inputSourcePreferences) as? [String: Bool] ?? [:]
    }

    private func saveInputSourcePreferences() {
        UserDefaults.standard.set(inputSourcePreferences, forKey: SettingsKey.inputSourcePreferences)
    }
}
```

### Settings Key

```swift
enum SettingsKey {
    // ... existing keys ...
    static let inputSourcePreferences = "gonhanh.inputSourcePreferences"
}
```

### Updated Observer Logic

```swift
class InputSourceObserver {
    // ... existing code ...

    private func handleInputSourceChange() {
        let appState = AppState.shared
        let currentId = InputSourceManager.shared.getCurrentInputSourceId()

        appState.currentInputSourceId = currentId

        // Find preference for current source
        guard let currentId = currentId,
              let source = appState.inputSources.first(where: { $0.id == currentId }) else {
            return
        }

        if source.disableGoNhanh {
            // Temporarily disable
            appState.setInputSourceOverride(false)
            Log.info("InputSource: \(source.localizedName) - disabled Gõ Nhanh")
        } else {
            // Restore user preference
            appState.setInputSourceOverride(nil)
            Log.info("InputSource: \(source.localizedName) - enabled Gõ Nhanh")
        }
    }
}
```

### SwiftUI Settings View

```swift
struct InputSourceSettingsView: View {
    @ObservedObject var appState = AppState.shared

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            Text("Quản lý bộ gõ")
                .font(.headline)

            Text("Gõ Nhanh sẽ TẮT khi bạn chuyển sang các bộ gõ được đánh dấu:")
                .font(.subheadline)
                .foregroundColor(.secondary)

            List {
                ForEach(appState.inputSources) { source in
                    InputSourceRow(source: source) { newValue in
                        appState.setInputSourcePreference(id: source.id, disableGoNhanh: newValue)
                    }
                }
            }
            .frame(height: 200)

            Button("Làm mới danh sách") {
                appState.refreshInputSources()
            }

            Divider()

            Button("Thêm bộ gõ mới...") {
                // Open System Preferences > Keyboard > Input Sources
                NSWorkspace.shared.open(URL(string: "x-apple.systempreferences:com.apple.preference.keyboard?InputSources")!)
            }
        }
        .padding()
    }
}

struct InputSourceRow: View {
    let source: InputSourceItem
    let onToggle: (Bool) -> Void

    var body: some View {
        HStack {
            Text(source.flagEmoji)
            Text(source.localizedName)

            Spacer()

            Toggle("", isOn: Binding(
                get: { source.disableGoNhanh },
                set: { onToggle($0) }
            ))
            .labelsHidden()

            Text(source.disableGoNhanh ? "Tắt" : "Bật")
                .font(.caption)
                .foregroundColor(source.disableGoNhanh ? .red : .green)
        }
    }
}
```

### Menu Bar Submenu

```swift
// In MenuBar.swift - add submenu for input source switching
func buildInputSourceMenu() -> NSMenu {
    let menu = NSMenu()

    for source in AppState.shared.inputSources {
        let item = NSMenuItem(
            title: "\(source.flagEmoji) \(source.localizedName)",
            action: #selector(switchInputSource(_:)),
            keyEquivalent: ""
        )
        item.representedObject = source.id
        item.state = source.id == AppState.shared.currentInputSourceId ? .on : .off
        menu.addItem(item)
    }

    return menu
}

@objc func switchInputSource(_ sender: NSMenuItem) {
    guard let id = sender.representedObject as? String else { return }
    AppState.shared.switchToInputSource(id: id)
}
```

### Ưu điểm của Phương án B

| Aspect | Phương án A (Auto-detect) | Phương án B (UI Manager) |
|--------|---------------------------|--------------------------|
| **Setup** | Zero config | Có thể customize |
| **Flexibility** | Cứng nhắc | User toàn quyền |
| **UX** | Ẩn, magic | Rõ ràng, transparent |
| **Edge cases** | Có thể sai | User tự quyết |
| **Switching** | Dùng macOS | Có thể từ menu bar |

### Đề xuất: Kết hợp A + B

1. **Default behavior (A):** Auto-detect Latin/non-Latin, tự động set preferences
2. **Advanced UI (B):** Cho phép user override từng input source
3. **Quick switch:** Submenu trong menu bar để switch input source nhanh

---

## Testing Plan

### Manual Tests

1. **Basic flow:**
   - Cài input method tiếng Nhật (Hiragana) trong System Preferences
   - Bật Gõ Nhanh
   - Chuyển sang Japanese → Verify Gõ Nhanh tự disable
   - Chuyển về English → Verify Gõ Nhanh tự enable lại

2. **Edge cases:**
   - Switch nhanh liên tục giữa EN/JP
   - Tắt feature trong settings → không auto-disable
   - User manually disable Gõ Nhanh → khi switch về EN không auto-enable

3. **Input Sources để test:**
   - `com.apple.keylayout.ABC` (US English)
   - `com.apple.keylayout.USInternational`
   - `com.apple.inputmethod.Kotoeri.RomajiTyping.Japanese` (Japanese)
   - `com.apple.inputmethod.TCIM.Pinyin` (Chinese)
   - `com.apple.inputmethod.Korean.2SetKorean` (Korean)

---

## Tương tác với Smart Mode (Per-App)

**Câu hỏi:** Khi kết hợp với Smart Mode (per-app), ưu tiên nào cao hơn?

**Đề xuất:** Input Source priority > Per-App Smart Mode

```
Effective Enabled =
    if (isNonLatinInputSource && autoDisableForNonLatin) → false
    else if (smartModeEnabled) → perAppPreference[currentApp]
    else → userWantsEnabled
```

**Lý do:**
- Khi user chuyển sang Japanese IME, họ rõ ràng muốn gõ tiếng Nhật
- Việc này xảy ra ở "system level", cao hơn app-level preference

---

## Timeline & Priority

- **Mức độ phức tạp:** Trung bình (2-3 ngày dev)
- **Rủi ro:** Thấp (chỉ thêm feature mới, không break existing)
- **Impact:** Cao (nhiều user sử dụng multi-language)

### Tasks breakdown:

#### Phase 1: Core Infrastructure
1. [ ] Implement `InputSourceManager` class (TIS API wrapper)
2. [ ] Implement `InputSourceObserver` class (listen changes)
3. [ ] Add `InputSourceItem` data model
4. [ ] Add `SettingsKey.inputSourcePreferences`

#### Phase 2: State Management
5. [ ] Update `AppState` với:
   - `inputSources: [InputSourceItem]`
   - `currentInputSourceId: String?`
   - `inputSourcePreferences: [String: Bool]`
   - `setInputSourceOverride()` logic
6. [ ] Persist preferences to UserDefaults

#### Phase 3: UI
7. [ ] Create `InputSourceSettingsView` trong Settings
8. [ ] Add "Quản lý bộ gõ" section
9. [ ] Add submenu "Chuyển bộ gõ" trong Menu Bar

#### Phase 4: Integration
10. [ ] Update `MenuBarController.startEngine()` để start observer
11. [ ] Handle edge cases (new input source added, removed)
12. [ ] Testing với các input source: EN, JP, CN, KR, TH

#### Phase 5: Polish
13. [ ] Add flag emoji cho các ngôn ngữ phổ biến
14. [ ] Indicator trên menu bar khi bị disable do input source
15. [ ] Update documentation

---

## References

- [TISSelectInputSource](https://developer.apple.com/documentation/carbon/1390537-tisselectinputsource)
- [kTISNotifySelectedKeyboardInputSourceChanged](https://developer.apple.com/documentation/carbon/ktisnotifyselectedkeyboardinputsourcechanged)
- [InputSourceKit (open source reference)](https://github.com/lafrenierejm/InputSourceKit)
- EVKey behavior demo: https://www.loom.com/share/ea185f7cfd584ccda6f7f4fcb2a260bb
