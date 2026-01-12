#pragma once
#include "Types.h"
#include "LockFreeQueue.h"
#include <Windows.h>
#include <thread>
#include <atomic>

// Keyboard hook manager with worker thread architecture
// Hook callback: Minimal work (<1μs), queues event
// Worker thread: Processes event, calls Rust engine, sends text
class KeyboardHook {
public:
    // Delete constructors to prevent instantiation
    KeyboardHook() = delete;
    KeyboardHook(const KeyboardHook&) = delete;
    KeyboardHook& operator=(const KeyboardHook&) = delete;

    // Install system-wide keyboard hook
    static bool Install();

    // Uninstall keyboard hook
    static void Uninstall();

    // Start worker thread
    static void StartWorker();

    // Stop worker thread
    static void StopWorker();

    // Check if hook is installed
    static bool IsInstalled() { return s_hook != nullptr; }

    // Check if worker is running
    static bool IsWorkerRunning() { return s_running.load(); }

    // Injection in progress flag (prevents race condition)
    static std::atomic<bool> s_injectionInProgress;

private:
    // Hook callback (called by Windows on every keystroke)
    static LRESULT CALLBACK LowLevelKeyboardProc(int nCode, WPARAM wParam, LPARAM lParam);

    // Worker thread function
    static void WorkerThreadFunc();

    // Static members
    static HHOOK s_hook;
    static std::thread s_workerThread;
    static std::atomic<bool> s_running;
    static LockFreeQueue<KeyEvent, 512> s_eventQueue;
    static LARGE_INTEGER s_performanceFrequency;  // For latency measurement
};
