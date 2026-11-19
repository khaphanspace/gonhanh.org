#pragma once
#include <windows.h>
#include <atomic>
#include <condition_variable>
#include <cstdint>
#include <deque>
#include <mutex>
#include <string>
#include <thread>
#include <vector>
#include "app_compat.h"

namespace gonhanh {

// Sends the engine's output (backspaces + Unicode text) to the focused app.
//
// The low-level keyboard hook must return quickly: Windows silently removes hooks that keep
// exceeding LowLevelHooksTimeout, and every microsecond spent there is typing latency.
//  - Apps that accept a burst of input (the default) get everything in ONE SendInput call,
//    synchronously from the hook, with no delays.
//  - Apps that need paced input (terminals, VS Code/Electron) are served by a worker thread.
//    While that work is pending, the hook swallows later key events and queues them here to be
//    replayed after it, so the app still sees keys in the order the user typed them.
class TextInjector {
public:
    static TextInjector& Instance();

    // From the hook thread, after the engine produced output for a key
    void Inject(int backspaceCount, const uint32_t* chars, uint8_t charCount);

    // True while queued work hasn't been sent yet (hook must then queue keys via ReplayKey)
    bool Busy() const { return pending_.load(std::memory_order_acquire) > 0; }

    // Queue an original key event to be re-sent after the pending work (hook thread only)
    void ReplayKey(const KBDLLHOOKSTRUCT& key, bool keyUp);

    // Swallowed-but-not-yet-replayed Caps Lock presses (to keep the Caps state the engine sees right)
    int PendingCapsToggles() const { return pendingCaps_.load(std::memory_order_acquire); }

    void Shutdown();

private:
    TextInjector() = default;
    ~TextInjector();
    TextInjector(const TextInjector&) = delete;
    TextInjector& operator=(const TextInjector&) = delete;

    struct Job {
        enum class Kind { Text, Key } kind = Kind::Text;
        // Text
        int backspaces = 0;
        std::vector<uint32_t> chars;
        InjectionMethod method = InjectionMethod::Fast;
        InjectionTiming timing;
        // Key replay
        KBDLLHOOKSTRUCT key = {};
        bool keyUp = false;
    };

    void Enqueue(Job job);
    void WorkerLoop();
    void Run(const Job& job);

    // Input builders
    static void AppendBackspaces(std::vector<INPUT>& out, int count);
    static void AppendSelectLeft(std::vector<INPUT>& out, int count);
    static void AppendText(std::vector<INPUT>& out, const uint32_t* chars, size_t count);
    static void Send(std::vector<INPUT>& inputs);
    static void WaitMicroseconds(uint32_t us);

    std::mutex mutex_;
    std::condition_variable cv_;
    std::deque<Job> queue_;
    std::thread worker_;
    bool stopping_ = false;
    std::atomic<int> pending_{0};
    std::atomic<int> pendingCaps_{0};
};

class KeyboardHook {
public:
    static KeyboardHook& Instance();
    bool Install();
    void Uninstall();
    void Toggle();
    bool IsEnabled() const { return enabled_; }
    void SetEnabled(bool enabled);
    // Window that receives WM_TOGGLE_IME when the toggle hotkey is pressed
    void SetNotifyWindow(HWND hwnd) { notifyHwnd_ = hwnd; }

private:
    KeyboardHook() = default;
    ~KeyboardHook();
    KeyboardHook(const KeyboardHook&) = delete;
    KeyboardHook& operator=(const KeyboardHook&) = delete;

    static LRESULT CALLBACK LowLevelKeyboardProc(int nCode, WPARAM wParam, LPARAM lParam);
    void TrackModifiers(const KBDLLHOOKSTRUCT& kb, bool keyDown);

    HHOOK hook_ = nullptr;
    HWND notifyHwnd_ = nullptr;
    bool enabled_ = true;
    bool processing_ = false;

    // Modifier state tracked from the hook's own event stream (bit per physical key).
    // GetKeyState/GetAsyncKeyState lag behind events that the hook swallows and replays later.
    enum : uint8_t { kLShift = 1, kRShift = 2, kLCtrl = 4, kRCtrl = 8 };
    uint8_t modifiers_ = 0;
    bool ShiftDown() const { return (modifiers_ & (kLShift | kRShift)) != 0; }
    bool CtrlDown() const { return (modifiers_ & (kLCtrl | kRCtrl)) != 0; }

    // Word tracking for restore functionality
    std::wstring lastWord_;
    std::wstring currentWord_;
    bool afterSpace_ = false;
};

} // namespace gonhanh
