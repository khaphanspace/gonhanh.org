#include "keyboard_hook.h"
#include "rust_bridge.h"
#include "app_compat.h"
#include "resource.h"

namespace gonhanh {

static KeyboardHook* g_instance = nullptr;

// Every event we inject carries this tag, so the hook can tell our own output apart
// from keys injected by other software.
constexpr ULONG_PTR kSelfInjectTag = 0x474E4B48;  // "GNKH"

// Other software's injected keys are ignored, except in test mode (GONHANH_TEST_INPUT=1),
// which lets a script type into apps at any speed (platforms/windows/tools/typing-test.ps1).
static const bool g_acceptForeignInjected = [] {
    wchar_t value[4] = {};
    return GetEnvironmentVariableW(L"GONHANH_TEST_INPUT", value, 4) > 0 && value[0] == L'1';
}();

// Modifier and lock keys: they type nothing, so a replayed press is never dropped
static bool IsStateKey(DWORD vk) {
    switch (vk) {
        case VK_SHIFT: case VK_LSHIFT: case VK_RSHIFT:
        case VK_CONTROL: case VK_LCONTROL: case VK_RCONTROL:
        case VK_MENU: case VK_LMENU: case VK_RMENU:
        case VK_LWIN: case VK_RWIN:
        case VK_CAPITAL: case VK_NUMLOCK: case VK_SCROLL:
            return true;
        default:
            return false;
    }
}

// RAII guard for reentrancy protection
class ProcessingGuard {
    bool& flag_;
public:
    explicit ProcessingGuard(bool& flag) : flag_(flag) { flag_ = true; }
    ~ProcessingGuard() { flag_ = false; }
};

// ============================================================================
// TextInjector Implementation
// ============================================================================

namespace {

INPUT KeyInput(WORD vk, DWORD flags) {
    INPUT in = {};
    in.type = INPUT_KEYBOARD;
    in.ki.wVk = vk;
    in.ki.dwFlags = flags;
    return in;
}

INPUT UnicodeInput(WORD unit, DWORD flags) {
    INPUT in = {};
    in.type = INPUT_KEYBOARD;
    in.ki.wScan = unit;
    in.ki.dwFlags = KEYEVENTF_UNICODE | flags;
    return in;
}

bool NeedsPacing(const DetectionResult& d) {
    return d.method == InjectionMethod::Selection || d.timing.backspaceDelayUs || d.timing.waitDelayUs ||
           d.timing.textDelayUs;
}

}  // namespace

TextInjector& TextInjector::Instance() {
    static TextInjector instance;
    return instance;
}

TextInjector::~TextInjector() {
    Shutdown();
}

void TextInjector::AppendBackspaces(std::vector<INPUT>& out, int count) {
    for (int i = 0; i < count; ++i) {
        out.push_back(KeyInput(VK_BACK, 0));
        out.push_back(KeyInput(VK_BACK, KEYEVENTF_KEYUP));
    }
}

void TextInjector::AppendSelectLeft(std::vector<INPUT>& out, int count) {
    // Shift+Left selects the characters to replace (address bars: Backspace may navigate back)
    out.push_back(KeyInput(VK_SHIFT, 0));
    for (int i = 0; i < count; ++i) {
        out.push_back(KeyInput(VK_LEFT, KEYEVENTF_EXTENDEDKEY));
        out.push_back(KeyInput(VK_LEFT, KEYEVENTF_EXTENDEDKEY | KEYEVENTF_KEYUP));
    }
    out.push_back(KeyInput(VK_SHIFT, KEYEVENTF_KEYUP));
}

void TextInjector::AppendText(std::vector<INPUT>& out, const uint32_t* chars, size_t count) {
    for (size_t i = 0; i < count; ++i) {
        uint32_t cp = chars[i];
        if (cp <= 0xFFFF) {
            out.push_back(UnicodeInput(static_cast<WORD>(cp), 0));
            out.push_back(UnicodeInput(static_cast<WORD>(cp), KEYEVENTF_KEYUP));
        } else {
            // Supplementary plane: surrogate pair
            cp -= 0x10000;
            WORD high = static_cast<WORD>(0xD800 + (cp >> 10));
            WORD low = static_cast<WORD>(0xDC00 + (cp & 0x3FF));
            out.push_back(UnicodeInput(high, 0));
            out.push_back(UnicodeInput(high, KEYEVENTF_KEYUP));
            out.push_back(UnicodeInput(low, 0));
            out.push_back(UnicodeInput(low, KEYEVENTF_KEYUP));
        }
    }
}

void TextInjector::Send(std::vector<INPUT>& inputs) {
    if (!inputs.empty()) {
        for (auto& in : inputs) in.ki.dwExtraInfo = kSelfInjectTag;
        SendInput(static_cast<UINT>(inputs.size()), inputs.data(), sizeof(INPUT));
        inputs.clear();
    }
}

void TextInjector::WaitMicroseconds(uint32_t us) {
    if (us == 0) return;
    if (us >= 2000) {
        Sleep(us / 1000);
        return;
    }
    // Short waits: spin on the worker thread (Sleep has ~1-15 ms granularity)
    LARGE_INTEGER freq, start, now;
    QueryPerformanceFrequency(&freq);
    QueryPerformanceCounter(&start);
    const LONGLONG target = static_cast<LONGLONG>(us) * freq.QuadPart / 1000000;
    do {
        YieldProcessor();
        QueryPerformanceCounter(&now);
    } while (now.QuadPart - start.QuadPart < target);
}

void TextInjector::Inject(int backspaceCount, const uint32_t* chars, uint8_t charCount) {
    if (backspaceCount < 0) return;  // Engine sends up to 255 (u8), e.g. long shortcut triggers
    DetectionResult detection = AppCompat::Instance().GetInjectionMethod();

    if (!Busy() && !NeedsPacing(detection)) {
        // Fast path: one atomic burst, sent before the hook returns (no thread hop, no waits)
        std::vector<INPUT> inputs;
        inputs.reserve(static_cast<size_t>(backspaceCount) * 2 + static_cast<size_t>(charCount) * 4);
        AppendBackspaces(inputs, backspaceCount);
        AppendText(inputs, chars, charCount);
        Send(inputs);
        return;
    }

    Job job;
    job.kind = Job::Kind::Text;
    job.backspaces = backspaceCount;
    job.chars.assign(chars, chars + charCount);
    job.method = detection.method;
    job.timing = detection.timing;
    job.generation = generation_.load(std::memory_order_acquire);
    Enqueue(std::move(job));
}

void TextInjector::CancelPendingText() {
    generation_.fetch_add(1, std::memory_order_acq_rel);
}

void TextInjector::ReplayKey(const KBDLLHOOKSTRUCT& key, bool keyUp) {
    Job job;
    job.kind = Job::Kind::Key;
    job.key = key;
    job.keyUp = keyUp;
    job.generation = generation_.load(std::memory_order_acquire);
    if (key.vkCode == VK_CAPITAL && !keyUp) {
        pendingCaps_.fetch_add(1, std::memory_order_acq_rel);
    }
    Enqueue(std::move(job));
}

void TextInjector::Enqueue(Job job) {
    std::lock_guard<std::mutex> lock(mutex_);
    if (stopping_) return;
    if (!worker_.joinable()) {
        worker_ = std::thread(&TextInjector::WorkerLoop, this);
    }
    pending_.fetch_add(1, std::memory_order_acq_rel);
    queue_.push_back(std::move(job));
    cv_.notify_one();
}

void TextInjector::WorkerLoop() {
    for (;;) {
        Job job;
        {
            std::unique_lock<std::mutex> lock(mutex_);
            cv_.wait(lock, [this] { return stopping_ || !queue_.empty(); });
            if (stopping_ && queue_.empty()) return;
            job = std::move(queue_.front());
            queue_.pop_front();
        }
        Run(job);
        if (job.kind == Job::Kind::Key && job.key.vkCode == VK_CAPITAL && !job.keyUp) {
            pendingCaps_.fetch_sub(1, std::memory_order_acq_rel);
        }
        pending_.fetch_sub(1, std::memory_order_acq_rel);
    }
}

void TextInjector::Run(const Job& job) {
    std::vector<INPUT> inputs;

    if (job.kind == Job::Kind::Key) {
        // A key pressed before focus moved was meant for the previous field or app: drop it
        // rather than type it (or send Enter) into the new one. Key-ups, modifiers and lock
        // keys still go through so no key stays stuck down and the lock state matches.
        if (!job.keyUp && !IsStateKey(job.key.vkCode) &&
            generation_.load(std::memory_order_acquire) != job.generation) {
            return;
        }
        INPUT in = {};
        in.type = INPUT_KEYBOARD;
        in.ki.wVk = static_cast<WORD>(job.key.vkCode);
        in.ki.wScan = static_cast<WORD>(job.key.scanCode);
        in.ki.dwFlags = (job.keyUp ? KEYEVENTF_KEYUP : 0) |
                        ((job.key.flags & LLKHF_EXTENDED) ? KEYEVENTF_EXTENDEDKEY : 0);
        inputs.push_back(in);
        Send(inputs);
        return;
    }

    // Text for a window that lost the foreground is dropped, also midway through a paced job:
    // SendInput goes to whatever window is in front now.
    auto cancelled = [&] { return generation_.load(std::memory_order_acquire) != job.generation; };
    if (cancelled()) return;

    const InjectionTiming& t = job.timing;
    if (job.backspaces > 0) {
        if (job.method == InjectionMethod::Selection) {
            AppendSelectLeft(inputs, job.backspaces);
            Send(inputs);
        } else if (t.backspaceDelayUs == 0) {
            AppendBackspaces(inputs, job.backspaces);
            Send(inputs);
        } else {
            for (int i = 0; i < job.backspaces; ++i) {
                if (cancelled()) return;
                AppendBackspaces(inputs, 1);
                Send(inputs);
                if (i + 1 < job.backspaces) WaitMicroseconds(t.backspaceDelayUs);
            }
        }
        WaitMicroseconds(t.waitDelayUs);
        if (cancelled()) return;
    }

    if (t.textDelayUs == 0) {
        AppendText(inputs, job.chars.data(), job.chars.size());
        Send(inputs);
    } else {
        for (size_t i = 0; i < job.chars.size(); ++i) {
            if (cancelled()) return;
            AppendText(inputs, &job.chars[i], 1);
            Send(inputs);
            if (i + 1 < job.chars.size()) WaitMicroseconds(t.textDelayUs);
        }
    }
}

void TextInjector::Shutdown() {
    {
        std::lock_guard<std::mutex> lock(mutex_);
        stopping_ = true;
    }
    cv_.notify_all();
    if (worker_.joinable()) worker_.join();
}

// ============================================================================
// KeyboardHook Implementation
// ============================================================================

// VK→macOS keycode mapping verified against core/src/data/keys.rs
static uint16_t VkToMacKeycode(DWORD vk) {
    // Letters A-Z
    static const uint16_t letters[] = {
        0x00, 0x0B, 0x08, 0x02, 0x0E, 0x03, 0x05, 0x04, // A-H
        0x22, 0x26, 0x28, 0x25, 0x2E, 0x2D, 0x1F, 0x23, // I-P
        0x0C, 0x0F, 0x01, 0x11, 0x20, 0x09, 0x0D, 0x07, // Q-X
        0x10, 0x06                                       // Y-Z
    };
    if (vk >= 'A' && vk <= 'Z') {
        return letters[vk - 'A'];
    }

    // Numbers 0-9
    static const uint16_t numbers[] = {
        0x1D, 0x12, 0x13, 0x14, 0x15, 0x17, 0x16, 0x1A, 0x1C, 0x19
    };
    if (vk >= '0' && vk <= '9') {
        return numbers[vk - '0'];
    }

    // Special keys
    switch (vk) {
        case VK_SPACE: return 0x31;
        case VK_RETURN: return 0x24;
        case VK_BACK: return 0x33;
        case VK_ESCAPE: return 0x35;
        case VK_OEM_4: return 0x21;  // [ key
        case VK_OEM_6: return 0x1E;  // ] key
        default: return 0xFF;  // Unknown
    }
}

// Singleton getter
KeyboardHook& KeyboardHook::Instance() {
    static KeyboardHook instance;
    g_instance = &instance;
    return instance;
}

KeyboardHook::~KeyboardHook() {
    Uninstall();
}

bool KeyboardHook::Install() {
    if (hook_) return true;

    modifiers_ = 0;
    if (GetAsyncKeyState(VK_LSHIFT) & 0x8000) modifiers_ |= kLShift;
    if (GetAsyncKeyState(VK_RSHIFT) & 0x8000) modifiers_ |= kRShift;
    if (GetAsyncKeyState(VK_LCONTROL) & 0x8000) modifiers_ |= kLCtrl;
    if (GetAsyncKeyState(VK_RCONTROL) & 0x8000) modifiers_ |= kRCtrl;

    hook_ = SetWindowsHookEx(
        WH_KEYBOARD_LL,
        LowLevelKeyboardProc,
        GetModuleHandle(NULL),
        0
    );

    return hook_ != nullptr;
}

void KeyboardHook::Uninstall() {
    if (hook_) {
        UnhookWindowsHookEx(hook_);
        hook_ = nullptr;
        // Flush queued input, stop the worker. Only when a hook was installed: ~KeyboardHook
        // calls Uninstall() again at static destruction, after the TextInjector (constructed
        // later, so destroyed earlier) is gone.
        TextInjector::Instance().Shutdown();
    }
}

void KeyboardHook::TrackModifiers(const KBDLLHOOKSTRUCT& kb, bool keyDown) {
    uint8_t bit = 0;
    switch (kb.vkCode) {
        case VK_LSHIFT: bit = kLShift; break;
        case VK_RSHIFT: bit = kRShift; break;
        case VK_LCONTROL: bit = kLCtrl; break;
        case VK_RCONTROL: bit = kRCtrl; break;
        default: return;
    }
    if (keyDown) {
        modifiers_ |= bit;
    } else {
        modifiers_ &= static_cast<uint8_t>(~bit);
    }
}

void KeyboardHook::Toggle() {
    enabled_ = !enabled_;
}

void KeyboardHook::SetEnabled(bool enabled) {
    enabled_ = enabled;
}

// Hook callback
LRESULT CALLBACK KeyboardHook::LowLevelKeyboardProc(int nCode, WPARAM wParam, LPARAM lParam) {
    if (nCode != HC_ACTION) {
        return CallNextHookEx(NULL, nCode, wParam, lParam);
    }

    auto* kb = reinterpret_cast<KBDLLHOOKSTRUCT*>(lParam);

    // Ignore injected events: always our own output and replays (prevents loops), and other
    // software's unless test mode is on
    if (!g_instance ||
        ((kb->flags & LLKHF_INJECTED) && (kb->dwExtraInfo == kSelfInjectTag || !g_acceptForeignInjected))) {
        return CallNextHookEx(NULL, nCode, wParam, lParam);
    }

    const bool keyDown = (wParam == WM_KEYDOWN || wParam == WM_SYSKEYDOWN);
    g_instance->TrackModifiers(*kb, keyDown);

    // Every key the engine doesn't transform goes through here. While paced output for a slow
    // app is still being sent, later keys must not overtake it: queue them for replay instead.
    TextInjector& injector = TextInjector::Instance();
    auto passThrough = [&]() -> LRESULT {
        if (injector.Busy()) {
            injector.ReplayKey(*kb, !keyDown);
            return 1;
        }
        return CallNextHookEx(NULL, nCode, wParam, lParam);
    };

    // Only key down events are processed by the engine
    if (!keyDown) {
        return passThrough();
    }

    // Handle Ctrl+Space toggle (check BEFORE enabled check)
    if (kb->vkCode == VK_SPACE && g_instance->CtrlDown()) {
        // Let the main loop do the switch (saves settings, updates tray/UI);
        // keep the low-level hook callback itself fast
        if (g_instance->notifyHwnd_) {
            PostMessageW(g_instance->notifyHwnd_, WM_TOGGLE_IME, 0, 0);
        } else {
            g_instance->Toggle();
        }
        return 1;  // Suppress key
    }

    // Pass through if disabled
    if (!g_instance->enabled_) {
        return passThrough();
    }

    // Reentrancy guard
    if (g_instance->processing_) {
        return passThrough();
    }

    // Word tracking for restore functionality
    bool isWordBoundary = (kb->vkCode == VK_SPACE ||
                           kb->vkCode == VK_RETURN ||
                           kb->vkCode == VK_TAB ||
                           (kb->vkCode >= VK_OEM_1 && kb->vkCode <= VK_OEM_3) ||  // ;=,-./`
                           (kb->vkCode >= VK_OEM_4 && kb->vkCode <= VK_OEM_7));   // [\]'

    if (isWordBoundary && !g_instance->currentWord_.empty()) {
        // Save completed word and mark as after space
        g_instance->lastWord_ = g_instance->currentWord_;
        g_instance->currentWord_.clear();
        g_instance->afterSpace_ = true;
    } else if (kb->vkCode == VK_BACK) {
        // Handle backspace - check if we should restore word
        if (g_instance->afterSpace_ && g_instance->currentWord_.empty() && !g_instance->lastWord_.empty()) {
            // Backspace right after completing a word - restore it
            std::string utf8Word = Utf16ToUtf8(g_instance->lastWord_);
            if (!utf8Word.empty()) {
                ime_restore_word(utf8Word.c_str());
            }
            g_instance->afterSpace_ = false;
        } else if (!g_instance->currentWord_.empty()) {
            // Remove last character from current word
            g_instance->currentWord_.pop_back();
        }
        g_instance->afterSpace_ = false;
    } else if (kb->vkCode >= 'A' && kb->vkCode <= 'Z') {
        // Track letter for current word
        g_instance->afterSpace_ = false;
        // Note: actual character will be tracked after engine processes it
    } else {
        g_instance->afterSpace_ = false;
    }

    // Convert VK to macOS keycode
    uint16_t keycode = VkToMacKeycode(kb->vkCode);
    if (keycode == 0xFF) {
        return passThrough();  // Unknown key
    }

    // Modifier states. Caps Lock presses still waiting in the replay queue haven't toggled
    // the system state yet, so account for them.
    bool caps = ((GetKeyState(VK_CAPITAL) & 0x0001) != 0) != ((injector.PendingCapsToggles() & 1) != 0);
    bool ctrl = g_instance->CtrlDown();
    bool shift = g_instance->ShiftDown();

    // Call Rust engine with RAII reentrancy guard
    ProcessingGuard guard(g_instance->processing_);
    ImeResultGuard result(ime_key_ext(keycode, caps, ctrl, shift));

    if (!result) {
        return passThrough();
    }

    // Check if engine wants to transform
    if (result->action == 0) {  // Pass through
        return passThrough();
    }

    // Log transform for debugging
#ifdef _DEBUG
    wchar_t logBuf[256];
    swprintf_s(logBuf, L"[HOOK] VK=0x%02X keycode=%d backspace=%d count=%d chars=",
               kb->vkCode, keycode, result->backspace, result->count);
    std::wstring logMsg(logBuf);
    for (uint8_t i = 0; i < result->count; ++i) {
        wchar_t charBuf[16];
        swprintf_s(charBuf, L"U+%04X ", result->chars[i]);
        logMsg += charBuf;
    }
    OutputDebugStringW(logMsg.c_str());
#endif

    // Use TextInjector for app-aware text injection
    TextInjector::Instance().Inject(result->backspace, result->chars, result->count);

    // Update word tracking with output characters
    if (result->count > 0) {
        std::wstring output = Utf32ToUtf16(result->chars, result->count);
        // If there were backspaces, adjust currentWord_ accordingly
        if (result->backspace > 0 && !g_instance->currentWord_.empty()) {
            size_t removeCount = (std::min)(static_cast<size_t>(result->backspace),
                                            g_instance->currentWord_.size());
            g_instance->currentWord_.erase(g_instance->currentWord_.size() - removeCount);
        }
        g_instance->currentWord_ += output;
    }

    return 1;  // Suppress original keystroke
}

} // namespace gonhanh
