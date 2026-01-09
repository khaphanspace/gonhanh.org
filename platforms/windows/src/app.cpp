#include "app.h"
#include "logger.h"
#include "rust_bridge.h"
#include "keyboard_hook.h"
#include "text_sender.h"
#include "settings.h"
#include "tray_icon.h"
#include "hotkey.h"
#include "update_manager.h"
#include "sound_manager.h"
#include "ui/composition_window.h"
#include "ui/update_window.h"

namespace gonhanh {

App::~App() {
    composition_window_.reset();
}

App& App::instance() {
    static App instance;
    return instance;
}

bool App::initialize(HINSTANCE hinstance) {
    hinstance_ = hinstance;

    // Initialize logging first
    Logger::init(Logger::get_default_log_dir());
    Logger::info("GoNhanh starting up (version %ls)", APP_VERSION);

    // Check single instance
    if (!check_single_instance()) {
        Logger::warn("Another instance is already running");
        return false;
    }

    // Initialize Rust core
    if (!RustBridge::instance().initialize()) {
        auto err = RustBridge::instance().get_last_error();
        Logger::error("Failed to initialize Rust bridge: %s", err.error_message.c_str());
        MessageBoxW(nullptr, L"Failed to load gonhanh_core.dll", APP_NAME, MB_ICONERROR);
        return false;
    }
    RustBridge::instance().init();

    // Load settings and apply to engine
    apply_settings_to_engine();
    Logger::info("Settings applied to engine");

    // Create hidden window for messages
    if (!create_window()) {
        Logger::error("Failed to create main window");
        return false;
    }

    // Setup components
    setup_tray_icon();
    setup_keyboard_hook();
    setup_hotkey();
    setup_composition_window();

    running_ = true;
    Logger::info("GoNhanh initialized successfully");
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
    Logger::info("GoNhanh shutting down");
    running_ = false;

    hide_composition();
    composition_window_.reset();

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

    Logger::shutdown();
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
            case TrayMenuId::CheckForUpdates:
                show_update();
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

void App::setup_composition_window() {
    composition_window_ = std::make_unique<CompositionWindow>();
    if (!composition_window_->create(hinstance_)) {
        Logger::warn("Failed to create composition window - continuing without it");
        composition_window_.reset();
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
        hide_composition();
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

    // Update composition window display
    update_composition_display();
}

void App::toggle_enabled() {
    auto& settings = Settings::instance();
    bool new_state = !settings.is_enabled();
    settings.set_enabled(new_state);

    RustBridge::instance().set_enabled(new_state);
    RustBridge::instance().clear();

    // Hide composition when IME is disabled
    if (!new_state) {
        hide_composition();
    }

    TrayIcon::instance().update_icon(new_state, settings.input_method());

    // Play toggle sound feedback
    SoundManager::instance().play_toggle_sound(new_state);
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

void App::show_update() {
    auto& update_window = ui::UpdateWindow::instance();
    update_window.show();

    // Trigger check if in idle state
    auto& manager = UpdateManager::instance();
    if (manager.state() == UpdateState::Idle) {
        manager.check_for_updates_manual();
    }
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

void App::update_composition_display() {
    if (!composition_window_) return;

    auto buffer = RustBridge::instance().get_buffer();
    if (buffer.empty()) {
        hide_composition();
    } else {
        auto caret = KeyboardHook::get_caret_position();
        if (caret.valid) {
            composition_window_->show(buffer, caret.x, caret.y);
        }
    }
}

void App::hide_composition() {
    if (composition_window_) {
        composition_window_->hide();
    }
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
