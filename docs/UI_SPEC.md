# UI Specification (macOS)

> Auto-generated from SwiftUI code on 2026-01-12
>
> **Note:** This spec covers macOS UI only. Linux uses Fcitx5 system UI, Windows TBD.

---

## 1. Overview

### 1.1 UI Architecture

```
Menu Bar (NSStatusItem)
    ├─ Menu (NSMenu) - Quick toggle, method switch, settings
    ├─ Onboarding Window (first run)
    ├─ Settings Window (main UI)
    └─ Update Window (download/install)
```

### 1.2 Design System

| Component | Technology | Style |
|-----------|------------|-------|
| Framework | **SwiftUI** | Native macOS |
| Visual | Light/Dark mode | System appearance |
| Effects | **NSVisualEffectView** | Sidebar, header blur |
| Icons | **SF Symbols** | System icons |
| Fonts | System fonts | San Francisco |
| Animations | SwiftUI transitions | Smooth, subtle |

---

## 2. Screen Inventory

| ID | Screen | File | Type | Size |
|----|--------|------|------|------|
| **S-01** | Menu Bar | `MenuBar.swift` | Status Bar | - |
| **S-02** | Onboarding | `OnboardingView.swift` | Window | 560×420 |
| **S-03** | Settings | `MainSettingsView.swift` | Window | 700×480 |
| **S-04** | Shortcuts Sheet | `MainSettingsView.swift:776-996` | Sheet | 480×420 |
| **S-05** | Update Window | `UpdateView.swift` | Window | ~400×300 |

---

## 3. S-01: Menu Bar

**File:** `MenuBar.swift`

**Location:** macOS status bar (top-right corner)

### 3.1 Visual States

| State | Icon | Tooltip | Description |
|-------|------|---------|-------------|
| Enabled | "🇻🇳" or "VN" | "Tiếng Việt (ON)" | Vietnamese mode active |
| Disabled | "EN" | "Tiếng Việt (OFF)" | Pass-through mode |
| Update Available | Badge | - | Orange dot indicator |

### 3.2 Menu Items

| Item | Type | Action | Shortcut | Source |
|------|------|--------|----------|--------|
| **Tiếng Việt** | Toggle | Enable/disable IME | Configurable | `MenuBar.swift:setupMenu` |
| **Telex / VNI** | Radio | Switch input method | - | " |
| Settings | Item | Open settings window | ⌘, | " |
| Check for Updates | Item | Check + download | - | " |
| About | Item | Open about page | - | " |
| Quit | Item | Exit app | ⌘Q | " |

**Source:** `MenuBar.swift:setupMenu()`

---

## 4. S-02: Onboarding Window

**File:** `OnboardingView.swift`

**Trigger:** First launch (no `hasCompletedOnboarding` flag)

**Size:** 560×420 px (fixed)

### 4.1 Steps

| Step | Title | Content | Action |
|------|-------|---------|--------|
| 1 | Welcome | App intro, logo | Next → |
| 2 | Permissions | Request Accessibility | Grant → |
| 3 | Features | Highlight key features | Done → |

### 4.2 UI Components

- Logo (96×96)
- Step indicator (dots)
- Title + description
- Action buttons (primary + secondary)
- Skip button (top-right)

**Source:** `OnboardingView.swift`

---

## 5. S-03: Settings Window

**File:** `MainSettingsView.swift`

**Trigger:** Menu → Settings, or ⌘,

**Size:** 700×480 px (fixed)

### 5.1 Layout

```
┌─────────────────────────────────────────┐
│ Sidebar (200px)   │  Content (500px)   │
│                    │                     │
│  [Logo]           │  [Settings Page]   │
│  Gõ Nhanh         │  or               │
│  v1.0.9           │  [About Page]     │
│                    │                     │
│  ┌──────────────┐ │                     │
│  │ ⚙ Cài đặt   │ │                     │
│  └──────────────┘ │                     │
│  ┌──────────────┐ │                     │
│  │ ⚡ Giới thiệu │ │                     │
│  └──────────────┘ │                     │
└─────────────────────────────────────────┘
```

### 5.2 Sidebar

**Visual Effect:** Sidebar material (blur)

**Components:**
- App logo (96×96)
- App name (20pt bold)
- Version badge (with update indicator)
- Navigation buttons (2 pages)

**Update Badge States:**
| State | Icon | Text | Color |
|-------|------|------|-------|
| Idle | - | - | - |
| Checking | ↻ (spinning) | "Kiểm tra" | Gray |
| Up-to-date | ✓ | "Mới nhất" | Green |
| Available | ↑ | "Cập nhật" | Orange |
| Error | ⚠ | "Thất bại" | Orange |

**Source:** `MainSettingsView.swift:547-566, 586-645`

---

### 5.3 Navigation

**Pages:**
| Icon | Label | Route |
|------|-------|-------|
| ⚙ | Cài đặt | `.settings` |
| ⚡ | Giới thiệu | `.about` |

**Source:** `MainSettingsView.swift:37-47`

---

## 6. S-04: Settings Page

**File:** `MainSettingsView.swift:681-772`

**Sections:** 4 cards (grouped settings)

### 6.1 Section 1: Input Method

| Row | Type | Label | Control | Default |
|-----|------|-------|---------|---------|
| 1 | Toggle | Bộ gõ tiếng Việt | Switch | ON |
| 2 | Picker | Kiểu gõ | Dropdown | Telex |
| 3 | Toggle | Gõ W thành Ư ở đầu từ | Switch | ON |
| 4 | Toggle | Gõ ] thành Ư, [ thành Ơ | Switch | OFF |

**Visibility:** Rows 3-4 only visible when Telex mode

**Source:** `MainSettingsView.swift:688-700`

---

### 6.2 Section 2: Shortcuts

| Row | Type | Label | Control | Default |
|-----|------|-------|---------|---------|
| 1 | Recorder | Phím tắt bật/tắt | Shortcut display | ⌥Space |
| 2 | Link | Bảng gõ tắt | Chevron → | - |

**Shortcut Recorder:**
- Display: Key caps (⌥, ⇧, ⌘, Space)
- Recording: "Nhấn phím..." (blue border)
- Conflict: ⚠ warning icon (if system shortcut)

**Shortcuts Link:**
- Shows count: "X/Y đang bật" or "Chưa có từ viết tắt"
- Opens sheet (S-05)

**Source:** `MainSettingsView.swift:703-709, 1073-1144, 1148-1175`

---

### 6.3 Section 3: App Behavior

| Row | Type | Label | Control | Default |
|-----|------|-------|---------|---------|
| 1 | Toggle | Khởi động cùng hệ thống | Switch | Auto-enabled |
| 2 | Toggle | Tự chuyển chế độ theo ứng dụng | Switch | ON |
| 3 | Row | Tự khôi phục từ tiếng Anh | Switch + Badge | OFF |

**Row 3 Badge:**
- "Beta · Góp ý" (orange capsule)
- Links to GitHub issue #26

**Source:** `MainSettingsView.swift:712-718, 750-767`

---

### 6.4 Section 4: Additional Options

| Row | Type | Label | Control | Default |
|-----|------|-------|---------|---------|
| 1 | Toggle | Âm thanh chuyển ngôn ngữ | Switch | OFF |
| 2 | Toggle | Đặt dấu kiểu mới (oà, uý) | Switch | ON |
| 3 | Toggle | Tự viết hoa đầu câu | Switch | OFF |
| 4 | Toggle | Gõ ESC hoàn tác dấu | Switch | OFF |

**Source:** `MainSettingsView.swift:721-731`

---

## 7. S-05: Shortcuts Sheet

**File:** `MainSettingsView.swift:776-996`

**Trigger:** Settings Page → Section 2 → "Bảng gõ tắt" row

**Size:** 480×420 px (sheet)

### 7.1 Layout

```
┌─────────────────────────────────────┐
│ Header: "Từ viết tắt" (X mục)      │
├─────────────────────────────────────┤
│ Form (Input):                       │
│   Viết tắt: [tphcm]                │
│   Nội dung: [Thành phố HCM]        │
│   [Huỷ] [Xoá]        [Thêm/Cập nhật]│
├─────────────────────────────────────┤
│ Table (Multi-select):               │
│ ☑ vn    │ Việt Nam           │ 🗑  │
│ ☑ hn    │ Hà Nội             │ 🗑  │
│ ☐ hcm   │ Hồ Chí Minh        │ 🗑  │
│ ...                                  │
├─────────────────────────────────────┤
│ Toolbar: [Nhập] [Xuất]       [Xong]│
└─────────────────────────────────────┘
```

### 7.2 Form Section

**State:** Edit vs Add
- **Add mode:** Form empty, button = "Thêm"
- **Edit mode:** Form populated, buttons = "Huỷ", "Xoá", "Cập nhật"

**Validation:**
- Save disabled if key or value empty
- Enter key = Save
- Escape key = Close sheet

**Source:** `MainSettingsView.swift:812-842`

---

### 7.3 Table Section

**Columns:**
| Width | Name | Content |
|-------|------|---------|
| 24px | Checkbox | Enable/disable shortcut |
| 80-140px | Viết tắt | Trigger (bold, medium) |
| Flex | Nội dung | Replacement (truncate) |
| 28px | Actions | Delete button (trash icon) |

**Interaction:**
- Single-select → Load into form (edit mode)
- Multi-select → No form load
- Delete key → Delete selected
- Empty state: Icon + "Chưa có từ viết tắt"

**Source:** `MainSettingsView.swift:854-900`

---

### 7.4 Toolbar

**Buttons:**
| Icon | Label | Action | Disabled |
|------|-------|--------|----------|
| ↓ | Nhập | Open file picker (.txt) | - |
| ↑ | Xuất | Save file picker (.txt) | Empty table |
| - | Xong | Close sheet | - |

**Import Format:**
```
;Gõ Nhanh - Bảng gõ tắt
vn:Việt Nam
hn:Hà Nội
```

**Source:** `MainSettingsView.swift:902-919, 958-995`

---

## 8. S-06: About Page

**File:** `MainSettingsView.swift:1000-1063`

**Content:**

### 8.1 Header
- App logo (80×80)
- App name (20pt bold)
- Tagline: "Bộ gõ tiếng Việt nhanh và nhẹ"
- Version: "Phiên bản X.X.X"

### 8.2 Links (3 cards)
| Icon | Label | URL |
|------|-------|-----|
| `</>` | GitHub | Repository |
| 🐛 | Báo lỗi | Issues page |
| ❤️ | Ủng hộ | GitHub Sponsors |

### 8.3 Footer
- "Phát triển bởi [Kha Phan]" (LinkedIn link)
- "Từ Việt Nam với ❤️"

**Source:** `MainSettingsView.swift:1000-1063`

---

## 9. S-07: Update Window

**File:** `UpdateView.swift`

**Trigger:** Update available → Badge click or auto-prompt

**Size:** ~400×300 px (inferred)

**Content:**
- Current version vs New version
- Release notes (from GitHub)
- Download button + progress bar
- Install + restart button

**Source:** `UpdateView.swift`, `UpdateManager.swift`

---

## 10. Reusable Components

### 10.1 SettingsRow

Generic row container with horizontal layout.

**Usage:**
```swift
SettingsRow {
    Text("Label")
    Spacer()
    Toggle("", isOn: $isOn)
}
```

**Source:** `MainSettingsView.swift:402-410`

---

### 10.2 SettingsToggleRow

Pre-built toggle row with title + optional subtitle.

**Props:**
- `title: String`
- `subtitle: String?`
- `isOn: Binding<Bool>`

**Source:** `MainSettingsView.swift:412-435`

---

### 10.3 KeyCap

Keyboard key visual (for shortcut display).

**Example:** ⌥, ⇧, ⌘, Space

**Style:**
- Gray background
- Rounded corners
- Border

**Source:** `MainSettingsView.swift:437-448`

---

### 10.4 CardBackground

Card visual effect (used for settings sections).

**Style:**
- Rounded corners (10px)
- Semi-transparent background
- Border (separator color)

**Source:** `MainSettingsView.swift:388-398`

---

## 11. User Journeys

### 11.1 First Launch

```
App Launch → Onboarding (S-02)
    Step 1: Welcome → Next
    Step 2: Grant Accessibility → Grant (macOS prompt)
    Step 3: Features → Done
    → Menu Bar appears (S-01)
    → Engine starts
```

---

### 11.2 Toggle Vietnamese

```
Method 1: Menu Bar → Click
Method 2: Global Shortcut (⌥Space by default)
    → Icon changes (🇻🇳 ↔ EN)
    → Sound plays (if enabled)
    → Per-app state saved (if enabled)
```

---

### 11.3 Change Input Method

```
Menu Bar → Telex/VNI Radio
    → Checkmark moves
    → Engine method updated
    → Continue typing with new method
```

---

### 11.4 Manage Shortcuts

```
Menu Bar → Settings → Settings Page
    → Section 2 → "Bảng gõ tắt" → Click
    → Shortcuts Sheet opens (S-05)

Add New:
    → Fill form (trigger + replacement)
    → Click "Thêm" or press Enter
    → Row added to table

Edit:
    → Click table row
    → Form populates
    → Modify → Click "Cập nhật"

Delete:
    → Select row(s) → Press Delete key
    → Or click trash icon

Import/Export:
    → Click "Nhập" → Select .txt file
    → Click "Xuất" → Save .txt file
```

---

### 11.5 Update App

```
Auto-check (every 24h):
    → Badge appears on version (orange dot)
    → Click badge → Download starts
    → Update Window (S-07) shows progress
    → "Cài đặt & Khởi động lại" button
    → App quits + installs + relaunches

Manual:
    → Menu Bar → "Kiểm tra cập nhật"
    → Same flow as above
```

---

## 12. Accessibility

### 12.1 Requirements

**macOS Accessibility Permission:**
- Required for keyboard monitoring (CGEventTap)
- Prompted during onboarding
- Can be granted in System Settings → Privacy & Security

**VoiceOver Support:**
- Settings toggles labeled
- Buttons have accessible labels
- Table rows readable

---

### 12.2 Keyboard Navigation

| Context | Shortcut | Action |
|---------|----------|--------|
| Global | ⌥Space (default) | Toggle Vietnamese |
| Settings | ⌘, | Open settings |
| Settings | Escape | Close window |
| Shortcuts Sheet | Enter | Save form |
| Shortcuts Sheet | Escape | Close sheet |
| Shortcuts Sheet | Delete | Delete selected rows |

---

## 13. Visual Design

### 13.1 Color Palette

**Follows macOS system colors:**
- Accent: System blue
- Labels: Primary, secondary, tertiary (adaptive)
- Backgrounds: Control background, window background
- Separators: System separator

### 13.2 Typography

| Element | Font | Size | Weight |
|---------|------|------|--------|
| Page title | System | 20pt | Bold |
| Section title | System | 15pt | Semibold |
| Row label | System | 13pt | Regular |
| Row subtitle | System | 11pt | Regular |
| Badge | System | 11pt | Medium |
| Version | System | 12pt | Regular |
| Footer | System | 11-12pt | Regular |

---

### 13.3 Spacing

| Element | Value |
|---------|-------|
| Window padding | 28px |
| Section spacing | 20px |
| Row padding (H) | 12px |
| Row padding (V) | 10px |
| Card corner radius | 10px |
| Button corner radius | 8px |
| Key cap radius | 4px |

---

## 14. Notifications

**System:** macOS NotificationCenter (internal)

| Event | Notification | Handler |
|-------|--------------|---------|
| Onboarding complete | `.onboardingCompleted` | Start engine, show menu bar |
| Toggle Vietnamese | `.toggleVietnamese` | Update icon, play sound |
| Input source changed | `.inputSourceChanged` | Auto-disable if non-EN |
| Show settings page | `.showSettingsPage` | Navigate to specific page |
| Shortcut changed | `.shortcutChanged` | Update global hotkey |
| Update state changed | `.updateStateChanged` | Refresh update badge |

**Source:** `MenuBar.swift:47-97`

---

## 15. Platform-Specific Notes

### 15.1 macOS Only

Current UI is **macOS-only** (SwiftUI + AppKit).

### 15.2 Linux

**Fcitx5** provides system UI:
- Standard Fcitx5 panel (candidate window)
- Standard Fcitx5 settings (via fcitx5-config-qt)
- No custom UI needed

### 15.3 Windows (Planned)

**TBD:** WPF/.NET 8 UI (similar to macOS design)

---

## 16. References

- Implementation: `platforms/macos/*.swift`
- Design Assets: `platforms/macos/Assets.xcassets/`
- App Metadata: `platforms/macos/AppMetadata.swift`
- Settings Keys: `platforms/macos/MainSettingsView.swift:SettingsKey`
