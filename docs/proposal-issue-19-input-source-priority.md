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

## Phương án B: Whitelist/Blacklist Input Sources

**Mô tả:** Cho phép user tự chọn những input source nào sẽ disable Gõ Nhanh.

**Ưu điểm:**
- Linh hoạt hơn
- User có toàn quyền kiểm soát

**Nhược điểm:**
- Phức tạp hơn để setup
- Cần UI phức tạp (list input sources, toggle từng cái)

**Đề xuất:** Bắt đầu với Phương án A, nếu có feedback cần flexibility hơn thì mở rộng sang B.

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

1. [ ] Implement `InputSourceObserver` class
2. [ ] Add `autoDisableForNonLatin` setting
3. [ ] Update `AppState` với override logic
4. [ ] Add UI toggle trong Settings
5. [ ] Update `MenuBarController.startEngine()` để start observer
6. [ ] Testing với các input source khác nhau
7. [ ] Update documentation

---

## References

- [TISSelectInputSource](https://developer.apple.com/documentation/carbon/1390537-tisselectinputsource)
- [kTISNotifySelectedKeyboardInputSourceChanged](https://developer.apple.com/documentation/carbon/ktisnotifyselectedkeyboardinputsourcechanged)
- [InputSourceKit (open source reference)](https://github.com/lafrenierejm/InputSourceKit)
- EVKey behavior demo: https://www.loom.com/share/ea185f7cfd584ccda6f7f4fcb2a260bb
