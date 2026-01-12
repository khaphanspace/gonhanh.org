#include "KeyboardHook.h"
#include "RustBridge.h"
#include "TextSender.h"
#include "KeycodeMap.h"
#include <Windows.h>

// Static member initialization
HHOOK KeyboardHook::s_hook = nullptr;
std::thread KeyboardHook::s_workerThread;
std::atomic<bool> KeyboardHook::s_running{false};
LockFreeQueue<KeyEvent, 512> KeyboardHook::s_eventQueue;
LARGE_INTEGER KeyboardHook::s_performanceFrequency = {0};
std::atomic<bool> KeyboardHook::s_injectionInProgress{false};

bool KeyboardHook::Install() {
    // Get performance frequency for latency measurement
    QueryPerformanceFrequency(&s_performanceFrequency);

    // Install low-level keyboard hook (system-wide)
    s_hook = SetWindowsHookEx(
        WH_KEYBOARD_LL,
        LowLevelKeyboardProc,
        GetModuleHandle(nullptr),
        0  // dwThreadId = 0 means system-wide hook
    );

    // Log error details if hook installation fails
    if (s_hook == nullptr) {
        DWORD error = GetLastError();
#ifdef _DEBUG
        char msg[256];
        sprintf_s(msg, "SetWindowsHookEx failed: error code %lu\n", error);
        OutputDebugStringA(msg);
#endif
        // Common errors:
        // ERROR_ACCESS_DENIED (5): Antivirus blocking
        // ERROR_HOOK_NEEDS_HMOD (1428): Module handle issue (rare with WH_KEYBOARD_LL)
        (void)error;  // Suppress unused variable warning in Release
    }

    return s_hook != nullptr;
}

void KeyboardHook::Uninstall() {
    if (s_hook) {
        UnhookWindowsHookEx(s_hook);
        s_hook = nullptr;
    }
}

LRESULT CALLBACK KeyboardHook::LowLevelKeyboardProc(int nCode, WPARAM wParam, LPARAM lParam) {
    // Must call CallNextHookEx for nCode < 0
    if (nCode < 0) {
        return CallNextHookEx(s_hook, nCode, wParam, lParam);
    }

    // HC_ACTION: process keystroke
    if (nCode == HC_ACTION) {
        KBDLLHOOKSTRUCT* pKeyboard = (KBDLLHOOKSTRUCT*)lParam;

        // Skip injected events (our own SendInput calls)
        // CRITICAL: Prevents infinite loop
        if (pKeyboard->flags & LLKHF_INJECTED) {
            return CallNextHookEx(s_hook, nCode, wParam, lParam);
        }

        // Skip if we're currently injecting text (race condition prevention)
        // This blocks keyboard during SendInput execution (~1-2ms)
        if (s_injectionInProgress.load(std::memory_order_acquire)) {
            return CallNextHookEx(s_hook, nCode, wParam, lParam);
        }

        // Filter out WM_SYSKEYDOWN/WM_SYSKEYUP (Alt+key combinations)
        // Vietnamese typing doesn't use Alt modifier
        if (wParam == WM_SYSKEYDOWN || wParam == WM_SYSKEYUP) {
            return CallNextHookEx(s_hook, nCode, wParam, lParam);
        }

        uint16_t vkCode = (uint16_t)pKeyboard->vkCode;

        // Ignore modifier keys, function keys, etc.
        if (ShouldIgnoreVK(vkCode)) {
            return CallNextHookEx(s_hook, nCode, wParam, lParam);
        }

        // Build event (minimal work - must return fast)
        KeyEvent event{};
        event.vkCode = vkCode;
        event.isKeyDown = (wParam == WM_KEYDOWN);
        event.isCaps = (GetKeyState(VK_CAPITAL) & 0x0001) != 0;
        event.isCtrl = (GetKeyState(VK_CONTROL) & 0x8000) != 0;
        event.isShift = (GetKeyState(VK_SHIFT) & 0x8000) != 0;
        QueryPerformanceCounter(&event.timestamp);

        // Queue event for worker thread (only process keydown events)
        if (event.isKeyDown) {
            bool pushed = s_eventQueue.Push(event);  // Lock-free, fast

            // Log warning if queue is full (should never happen at 512 size)
            if (!pushed) {
#ifdef _DEBUG
                OutputDebugStringA("WARNING: Event queue full, dropped keystroke\n");
#endif
                // Queue is full - this means worker thread is severely delayed (>50ms)
                // or user is typing >512 keys in ~50ms window (impossible)
                // Likely indicates worker thread hang or infinite loop
            }
        }
    }

    return CallNextHookEx(s_hook, nCode, wParam, lParam);
}

void KeyboardHook::WorkerThreadFunc() {
    // Set high priority for low latency
    SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_TIME_CRITICAL);

    while (s_running.load(std::memory_order_acquire)) {
        KeyEvent event;
        bool hadEvent = false;

        // Process all queued events
        while (s_eventQueue.Pop(event)) {
            hadEvent = true;

            // Convert Windows VK to macOS keycode
            uint16_t keycode = VKToMacKeycode(event.vkCode);
            if (!IsValidKey(keycode)) {
                continue;  // Skip invalid keys
            }

            // Call Rust engine (FFI)
            ImeResult* result = RustBridge::ProcessKeyExt(
                keycode,
                event.isCaps,
                event.isCtrl,
                event.isShift
            );

            // Process result
            if (result) {
                if (result->action == 1) {  // Send action
                    // Set injection flag to prevent race condition
                    s_injectionInProgress.store(true, std::memory_order_release);

                    // Send text (backspace + insert)
                    TextSender::Send(result->backspace, result->chars, result->count);

                    // Clear injection flag
                    s_injectionInProgress.store(false, std::memory_order_release);
                }
                // action == 0: None (pass through)
                // action == 2: Restore (handled internally by engine)

                // Free result
                RustBridge::Free(result);
            }

            // Measure latency (optional - can be disabled in Release builds)
#ifdef _DEBUG
            LARGE_INTEGER endTime;
            QueryPerformanceCounter(&endTime);
            double latencyMs = (endTime.QuadPart - event.timestamp.QuadPart) * 1000.0 /
                               s_performanceFrequency.QuadPart;
            if (latencyMs > 1.0) {
                // Log warning: latency exceeded 1ms target
                // OutputDebugStringA can be used for debugging
            }
#endif
        }

        // Sleep 1ms when idle to reduce CPU usage
        if (!hadEvent) {
            Sleep(1);
        }
    }
}

void KeyboardHook::StartWorker() {
    s_running.store(true, std::memory_order_release);
    s_workerThread = std::thread(WorkerThreadFunc);
}

void KeyboardHook::StopWorker() {
    s_running.store(false, std::memory_order_release);
    if (s_workerThread.joinable()) {
        s_workerThread.join();
    }
}
