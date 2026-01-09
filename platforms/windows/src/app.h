#pragma once

#include <windows.h>
#include <string>
#include <memory>

namespace gonhanh {

class CompositionWindow;

// Application class - manages lifecycle and message loop
class App {
public:
    static App& instance();

    // Initialize application
    bool initialize(HINSTANCE hinstance);

    // Run message loop
    int run();

    // Shutdown
    void shutdown();

    // Get instance handle
    HINSTANCE hinstance() const { return hinstance_; }

    // Get main window handle
    HWND main_hwnd() const { return hwnd_; }

    // Toggle IME enabled state
    void toggle_enabled();

    // Set input method
    void set_method(int method);

    // Show settings window
    void show_settings();

    // Show about window
    void show_about();

    // Show update window
    void show_update();

    // Apply settings to Rust engine
    void apply_settings_to_engine();

    // App metadata
    static constexpr const wchar_t* APP_NAME = L"GoNhanh";
    static constexpr const wchar_t* APP_VERSION = L"1.0.0";
    static constexpr const wchar_t* WINDOW_CLASS = L"GoNhanhMainClass";
    static constexpr const wchar_t* MUTEX_NAME = L"GoNhanh_SingleInstance";

private:
    App() = default;
    ~App();
    App(const App&) = delete;
    App& operator=(const App&) = delete;

    bool create_window();
    bool check_single_instance();
    void setup_keyboard_hook();
    void setup_tray_icon();
    void setup_hotkey();
    void setup_composition_window();
    void on_key_pressed(struct KeyPressEvent& event);
    void update_composition_display();
    void hide_composition();

    static LRESULT CALLBACK wnd_proc(HWND hwnd, UINT msg, WPARAM wparam, LPARAM lparam);

    HINSTANCE hinstance_ = nullptr;
    HWND hwnd_ = nullptr;
    HANDLE mutex_ = nullptr;
    bool running_ = false;
    std::unique_ptr<CompositionWindow> composition_window_;
};

} // namespace gonhanh
