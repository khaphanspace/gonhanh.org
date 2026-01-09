#include "app.h"
#include "rust_bridge.h"
#include "keyboard_hook.h"
#include "text_sender.h"
#include "settings.h"
#include "tray_icon.h"
#include "hotkey.h"

namespace gonhanh {

App& App::instance() {
    static App instance;
    return instance;
}

bool App::initialize(HINSTANCE hinstance) {
    hinstance_ = hinstance;

    // Check single instance
    if (!check_single_instance()) {
        return false;
    }

    // Initialize Rust core
    if (!RustBridge::instance().initialize()) {
        MessageBoxW(nullptr, L"Failed to load gonhanh_core.dll", APP_NAME, MB_ICONERROR);
        return false;
    }
    RustBridge::instance().init();

    // Load settings and apply to engine
    apply_settings_to_engine();

    // Create hidden window for messages
    if (!create_window()) {
        return false;
    }

    // Setup components
    setup_tray_icon();
    setup_keyboard_hook();
    setup_hotkey();

    running_ = true;
    return true;
}

int App::run() {
    MSG msg;
    while (GetMessage(&msg, nullptr, 0, 0)) {
        TranslateMessage(&msg);
        DispatchMessage(&msg);
    }
    return static_cast<int>(msg.wParam);
}

void App::shutdown() {
    running_ = false;

    KeyboardHook::instance().stop();
    HotKey::instance().shutdown();
    TrayIcon::instance().shutdown();
    RustBridge::instance().shutdown();

    if (hwnd_) {
        DestroyWindow(hwnd_);
        hwnd_ = nullptr;
    }

    if (mutex_) {
        CloseHandle(mutex_);
        mutex_ = nullptr;
    }
}

bool App::check_single_instance() {
    mutex_ = CreateMutexW(nullptr, TRUE, MUTEX_NAME);
    if (GetLastError() == ERROR_ALREADY_EXISTS) {
        // Another instance is running
        if (mutex_) {
            CloseHandle(mutex_);
            mutex_ = nullptr;
        }
        return false;
    }
    return true;
}

bool App::create_window() {
    WNDCLASSEXW wc = {};
    wc.cbSize = sizeof(wc);
    wc.lpfnWndProc = wnd_proc;
    wc.hInstance = hinstance_;
    wc.lpszClassName = WINDOW_CLASS;

    if (!RegisterClassExW(&wc)) {
        return false;
    }

    // Create hidden message-only window
    hwnd_ = CreateWindowExW(
        0,
        WINDOW_CLASS,
        APP_NAME,
        0,
        0, 0, 0, 0,
        HWND_MESSAGE,  // Message-only window
        nullptr,
        hinstance_,
        nullptr
    );

    return hwnd_ != nullptr;
}

void App::setup_keyboard_hook() {
    auto& hook = KeyboardHook::instance();
    hook.set_callback([this](KeyPressEvent& event) {
        on_key_pressed(event);
    });
    hook.start();
}

void App::setup_tray_icon() {
    auto& tray = TrayIcon::instance();
    tray.initialize(hwnd_);
    tray.update_icon(Settings::instance().is_enabled(), Settings::instance().input_method());

    tray.set_menu_callback([this](TrayMenuId id) {
        switch (id) {
            case TrayMenuId::Toggle:
                toggle_enabled();
                break;
            case TrayMenuId::Telex:
                set_method(0);
                break;
            case TrayMenuId::VNI:
                set_method(1);
                break;
            case TrayMenuId::Settings:
                show_settings();
                break;
            case TrayMenuId::About:
                show_about();
                break;
            case TrayMenuId::Exit:
                PostQuitMessage(0);
                break;
        }
    });
}

void App::setup_hotkey() {
    auto& hotkey = HotKey::instance();
    hotkey.initialize(hwnd_);
    hotkey.set_callback([this]() {
        toggle_enabled();
    });

    // Register default hotkey (Ctrl+Space) if configured
    uint32_t shortcut = Settings::instance().toggle_shortcut();
    if (shortcut != 0) {
        uint32_t modifiers = (shortcut >> 16) & 0xFFFF;
        uint32_t vk = shortcut & 0xFFFF;
        hotkey.register_toggle(modifiers, vk);
    }
}

void App::on_key_pressed(KeyPressEvent& event) {
    auto& settings = Settings::instance();

    // Skip if IME is disabled
    if (!settings.is_enabled()) {
        return;
    }

    // Clear buffer on Ctrl/Alt key combinations
    if (event.ctrl || event.alt) {
        RustBridge::instance().clear();
        return;
    }

    // Process key through Rust engine
    auto result = RustBridge::instance().process_key(event.keycode, event.shift, event.capslock);

    if (result.has_action()) {
        event.handled = true;

        if (result.action == ImeAction::Send || result.action == ImeAction::Restore) {
            TextSender::instance().send_text(result.text, result.backspace);
        }
    }
}

void App::toggle_enabled() {
    auto& settings = Settings::instance();
    bool new_state = !settings.is_enabled();
    settings.set_enabled(new_state);

    RustBridge::instance().set_enabled(new_state);
    RustBridge::instance().clear();

    TrayIcon::instance().update_icon(new_state, settings.input_method());
}

void App::set_method(int method) {
    auto& settings = Settings::instance();
    settings.set_input_method(method);

    RustBridge::instance().set_method(static_cast<InputMethod>(method));
    RustBridge::instance().clear();

    TrayIcon::instance().update_icon(settings.is_enabled(), method);
}

void App::show_settings() {
    // TODO: Implement settings window with Direct2D
    MessageBoxW(hwnd_, L"Settings window - Coming soon", APP_NAME, MB_ICONINFORMATION);
}

void App::show_about() {
    // TODO: Implement about window with Direct2D
    std::wstring msg = std::wstring(APP_NAME) + L"\nVersion " + APP_VERSION +
                       L"\n\nVietnamese Input Method Engine";
    MessageBoxW(hwnd_, msg.c_str(), L"About GoNhanh", MB_ICONINFORMATION);
}

void App::apply_settings_to_engine() {
    auto& settings = Settings::instance();
    auto& bridge = RustBridge::instance();

    bridge.set_enabled(settings.is_enabled());
    bridge.set_method(static_cast<InputMethod>(settings.input_method()));
    bridge.set_modern_tone(settings.modern_tone());
    bridge.set_skip_w_shortcut(!settings.w_shortcut());
    bridge.set_esc_restore(settings.esc_restore());
    bridge.set_english_auto_restore(settings.english_auto_restore());
    bridge.set_auto_capitalize(settings.auto_capitalize());
    bridge.set_bracket_shortcut(settings.bracket_shortcut());
}

LRESULT CALLBACK App::wnd_proc(HWND hwnd, UINT msg, WPARAM wparam, LPARAM lparam) {
    // Process tray icon messages
    if (TrayIcon::instance().process_message(msg, wparam, lparam)) {
        return 0;
    }

    // Process hotkey messages
    if (HotKey::instance().process_message(msg, wparam, lparam)) {
        return 0;
    }

    switch (msg) {
        case WM_DESTROY:
            PostQuitMessage(0);
            return 0;

        default:
            return DefWindowProcW(hwnd, msg, wparam, lparam);
    }
}

} // namespace gonhanh
