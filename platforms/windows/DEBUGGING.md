# GoNhanh Windows - Debugging Guide

Hướng dẫn debug khi GoNhanh không hoạt động hoặc không hiển thị gì trên Windows.

## Tại Sao Không Thấy Gì?

GoNhanh là **ứng dụng chạy nền** (background service), không có cửa sổ chính:
- ✅ **Chạy trong system tray** (góc phải dưới màn hình, gần đồng hồ)
- ✅ **Không có console window** (trừ khi build debug mode)
- ✅ **Không có splash screen hay welcome dialog**

## Kiểm Tra Nhanh (30 giây)

### 1. Kiểm Tra System Tray

```
Vị trí: Góc phải dưới màn hình → Click mũi tên (^) → Tìm icon GoNhanh
```

Nếu **THẤY icon GoNhanh**:
- ✅ App đang chạy bình thường
- Right-click icon → Xem menu
- Kiểm tra "Enabled" có checkmark không

Nếu **KHÔNG thấy icon**:
- ⚠️ App có thể crash khi khởi động
- Đọc tiếp để debug

### 2. Kiểm Tra Task Manager

```
Ctrl+Shift+Esc → Tab "Processes" → Tìm "gonhanh.exe"
```

- **Có gonhanh.exe**: App đang chạy, nhưng system tray icon có vấn đề
- **Không có gonhanh.exe**: App crash hoặc chưa chạy

### 3. Test Keyboard Hook

Mở Notepad và thử gõ:
```
Ctrl+Space (toggle IME)
tieng viet (nếu IME enabled)
```

- **Chuyển thành "tiếng việt"**: App hoạt động bình thường!
- **Không chuyển**: IME disabled hoặc keyboard hook failed

---

## Method 1: Debug Build (Nên Dùng) 🔧

Build với console window để xem logs chi tiết:

### Bước 1: Build Debug Mode

```bash
cd platforms/windows

# Configure với debug console
cmake -B build-debug -G "Visual Studio 17 2022" -A x64 -DENABLE_DEBUG_CONSOLE=ON

# Build Debug
cmake --build build-debug --config Debug
```

### Bước 2: Chạy Debug Build

```bash
# Chạy từ build directory
.\build-debug\Debug\gonhanh.exe
```

**Console window sẽ hiện ra** với logs như:

```
═══════════════════════════════════════════════════════════
  GoNhanh Debug Console
  Vietnamese Input Method - Windows C++ Implementation
═══════════════════════════════════════════════════════════

[STARTUP] GoNhanh starting...
[INFO] [2026-01-13 08:30:00] Rust engine initialized
[INFO] [2026-01-13 08:30:00] Settings loaded from Registry
[INFO] [2026-01-13 08:30:00] Per-app mode states loaded
[INFO] [2026-01-13 08:30:00] Keyboard hook installed
[INFO] [2026-01-13 08:30:00] System tray created
[INFO] [2026-01-13 08:30:00] GoNhanh started successfully
```

### Giải Thích Logs

**Logs thành công:**
```
✅ [INFO] Rust engine initialized          → Core engine OK
✅ [INFO] Settings loaded from Registry    → Registry OK
✅ [INFO] Keyboard hook installed          → Hook OK
✅ [INFO] System tray created              → Tray OK
✅ [INFO] GoNhanh started successfully     → Hoàn tất
```

**Logs lỗi thường gặp:**
```
❌ [ERROR] Failed to create message window
   → Windows API failed, có thể do system resources

❌ [ERROR] Failed to install keyboard hook
   → Đã có keyboard hook khác (AutoHotkey, gaming tools)
   → Thử chạy as Administrator

❌ [ERROR] Failed to create system tray icon
   → System tray service not available
   → Restart Windows Explorer (Ctrl+Shift+Esc → Explorer → Restart)
```

---

## Method 2: DebugView (Release Build) 📝

Xem logs từ release build (không có console):

### Bước 1: Download DebugView

```
Link: https://download.sysinternals.com/files/DebugView.zip
```

### Bước 2: Chạy DebugView

1. Extract `DebugView.zip`
2. **Run as Administrator**: Right-click `Dbgview64.exe` → "Run as administrator"
3. **Enable capture**: Menu → Capture → **Check "Capture Global Win32"**
4. **Clear buffer**: Menu → Edit → Clear Display

### Bước 3: Chạy GoNhanh

Chạy `gonhanh.exe` (release build) và xem logs xuất hiện trong DebugView:

```
[2752] [INFO] [2026-01-13 08:30:00] Rust engine initialized
[2752] [INFO] [2026-01-13 08:30:00] Settings loaded from Registry
[2752] [INFO] [2026-01-13 08:30:00] Per-app mode states loaded
[2752] [INFO] [2026-01-13 08:30:00] Keyboard hook installed
[2752] [INFO] [2026-01-13 08:30:00] System tray created
[2752] [INFO] [2026-01-13 08:30:00] GoNhanh started successfully
```

### Tips DebugView

- **Filter**: Menu → Edit → Filter/Highlight → Nhập "gonhanh" để chỉ xem GoNhanh logs
- **Save logs**: Menu → File → Save → Lưu logs để gửi bug report
- **Auto-scroll**: Menu → Edit → Auto Scroll (để luôn thấy logs mới nhất)

---

## Method 3: Windows Event Viewer 📊

Xem crash reports và system errors:

### Bước 1: Mở Event Viewer

```
Win+R → eventvwr → Enter
```

### Bước 2: Check Application Logs

```
Windows Logs → Application
```

### Bước 3: Filter Events

1. Right-click "Application" → **Filter Current Log**
2. Event level: Check **Error** và **Warning**
3. Event sources: Nhập **Application Error, Windows Error Reporting**
4. Click OK

### Bước 4: Tìm GoNhanh Crashes

Tìm entries có chứa "gonhanh.exe" trong Description:

```
Example crash entry:
Faulting application name: gonhanh.exe
Faulting module name: gonhanh_core.dll
Exception code: 0xc0000005 (Access Violation)
```

**Common exception codes:**
- `0xc0000005`: Access Violation (NULL pointer, invalid memory)
- `0xc000001d`: Illegal Instruction (CPU không support instruction)
- `0xc0000409`: Stack Buffer Overrun (stack corruption)

---

## Troubleshooting Checklist ✅

### App Không Khởi Động

- [ ] **Kiểm tra Rust core**: File `core/target/release/gonhanh_core.lib` có tồn tại không?
  ```bash
  # Rebuild Rust core nếu cần
  cd core
  cargo build --release
  ```

- [ ] **Kiểm tra dependencies**: Có file `.dll` nào thiếu không?
  ```bash
  # Check dependencies với dumpbin
  dumpbin /dependents gonhanh.exe
  ```

- [ ] **Kiểm tra process đang chạy**: Có instance GoNhanh nào đang chạy không?
  ```bash
  tasklist | findstr gonhanh
  # Nếu có → Kill process
  taskkill /F /IM gonhanh.exe
  ```

### System Tray Icon Không Hiện

- [ ] **Windows notification settings**:
  ```
  Settings → System → Notifications & actions
  → Make sure "Show app notifications" is ON
  ```

- [ ] **Taskbar settings**:
  ```
  Right-click Taskbar → Taskbar settings
  → Notification area → "Select which icons appear on the taskbar"
  → Turn ON GoNhanh
  ```

- [ ] **Restart Explorer**:
  ```
  Ctrl+Shift+Esc → Find "Windows Explorer" → Right-click → Restart
  ```

### Keyboard Hook Không Hoạt Động

- [ ] **Check conflicting tools**:
  ```
  Close: AutoHotkey, gaming macro tools, other IME software
  ```

- [ ] **Run as Administrator**:
  ```
  Right-click gonhanh.exe → Run as administrator
  ```

- [ ] **Check Registry permissions**:
  ```
  Win+R → regedit
  → Navigate: HKEY_CURRENT_USER\Software\GoNhanh
  → Right-click → Permissions → Make sure FULL CONTROL
  ```

### Typing Không Chuyển Tiếng Việt

- [ ] **Check IME enabled**:
  ```
  System tray icon → Right-click → "Enable" có checkmark?
  ```

- [ ] **Test hotkey**:
  ```
  Press Ctrl+Space (default toggle hotkey)
  ```

- [ ] **Check method**:
  ```
  System tray → Right-click → "Telex" có checkmark?
  (Hoặc VNI nếu bạn dùng VNI)
  ```

- [ ] **Test in different apps**:
  ```
  Open Notepad → Type "tieng viet"
  If works in Notepad but not Chrome → Per-app mode issue
  ```

---

## Advanced Debugging 🚀

### Attach Visual Studio Debugger

```bash
# Build debug với symbols
cmake -B build-debug -G "Visual Studio 17 2022" -A x64 -DCMAKE_BUILD_TYPE=Debug
cmake --build build-debug --config Debug

# Open in Visual Studio
devenv build-debug\gonhanh.sln

# Set breakpoints → F5 to debug
```

### Check Memory Leaks

```bash
# Build with ASAN (Address Sanitizer)
cmake -B build-asan -G "Visual Studio 17 2022" -A x64 -DENABLE_ASAN=ON
cmake --build build-asan --config Debug

# Run and watch for ASAN reports
.\build-asan\Debug\gonhanh.exe
```

### Profile Performance

```bash
# Use Windows Performance Analyzer
wpr -start CPU -filemode

# Run GoNhanh for 30 seconds

wpr -stop gonhanh_profile.etl

# Open gonhanh_profile.etl in Windows Performance Analyzer
```

---

## Gửi Bug Report 🐛

Nếu vẫn không fix được, gửi bug report với:

1. **Debug logs** (từ console hoặc DebugView)
2. **Event Viewer crash dump** (nếu có)
3. **System info**:
   ```bash
   systeminfo | findstr /B /C:"OS Name" /C:"OS Version" /C:"System Type"
   ```
4. **GoNhanh version**:
   ```bash
   System tray → Right-click → About
   ```

**GitHub Issues**: https://github.com/khaphanspace/gonhanh.org/issues

---

## FAQ ❓

**Q: Tại sao không có cửa sổ chính?**
A: GoNhanh là background service, chạy trong system tray giống AntiVirus.

**Q: Làm sao biết app đang chạy?**
A: Kiểm tra system tray icon (góc phải dưới) hoặc Task Manager.

**Q: Console window có giảm performance không?**
A: Có một chút (logging overhead), nhưng chỉ dùng khi debug. Release build không có console.

**Q: DebugView có cần chạy as Admin?**
A: Có, để capture OutputDebugString từ tất cả processes.

**Q: Logs được lưu vào file không?**
A: Hiện tại chỉ output ra console/DebugView. Có thể thêm file logging sau.
