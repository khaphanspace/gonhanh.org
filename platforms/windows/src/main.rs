//! GoNhanh for Windows: Win32 shell around the Rust core, one exe, no runtime.
#![cfg_attr(not(test), windows_subsystem = "windows")]

mod app;
mod core;
mod services;
mod views;
mod win;

use std::cell::Cell;
use std::ptr::null_mut;

use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HANDLE};
use windows_sys::Win32::System::Threading::{CreateMutexW, ReleaseMutex};
use windows_sys::Win32::UI::Controls::{
    InitCommonControlsEx, ICC_LISTVIEW_CLASSES, ICC_STANDARD_CLASSES, ICC_TAB_CLASSES,
    INITCOMMONCONTROLSEX,
};
use windows_sys::Win32::UI::WindowsAndMessaging::*;

use crate::core::controller::InputController;
use crate::core::hook;
use crate::services::settings::{self, Settings};
use crate::services::{autostart, foreground, updater};
use crate::views::{settings_window, tray};
use crate::win::{is_elevated, wide};

thread_local! {
    static MUTEX: Cell<HANDLE> = const { Cell::new(null_mut()) };
}

fn release_mutex() {
    let m = MUTEX.replace(null_mut());
    if !m.is_null() {
        unsafe {
            ReleaseMutex(m);
            CloseHandle(m);
        }
    }
}

/// The new exe is in place: start it and quit (mutex freed first so it doesn't see "already running")
pub fn restart_after_update() {
    release_mutex();
    if let Ok(exe) = std::env::current_exe() {
        let _ = std::process::Command::new(exe).spawn();
    }
    unsafe { PostQuitMessage(0) };
}

/// "Chạy với quyền admin" changed: start the next instance with the new rights and quit.
/// With admin: through the task (no UAC). Without: through Explorer, which runs it with normal rights.
/// false = could not start it, keep running.
pub fn restart_with_admin(admin: bool) -> bool {
    release_mutex();
    let started = if admin {
        autostart::start_via_task()
    } else {
        std::env::current_exe().is_ok_and(|exe| {
            let root = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".into());
            std::process::Command::new(format!("{root}\\explorer.exe"))
                .arg(exe)
                .spawn()
                .is_ok()
        })
    };
    if started {
        unsafe { PostQuitMessage(0) };
    } else {
        acquire_mutex();
    }
    started
}

/// Same name as the earlier Windows apps: only one GoNhanh at a time. false = another one runs.
fn acquire_mutex() -> bool {
    let name = wide("GoNhanh_SingleInstance");
    let mutex = unsafe { CreateMutexW(std::ptr::null(), 1, name.as_ptr()) };
    if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        unsafe { CloseHandle(mutex) };
        return false;
    }
    MUTEX.set(mutex);
    true
}

fn message_box(text: &str, flags: MESSAGEBOX_STYLE) {
    let (t, c) = (wide(text), wide("GoNhanh"));
    unsafe { MessageBoxW(null_mut(), t.as_ptr(), c.as_ptr(), flags) };
}

fn main() {
    let settings = Settings::load();

    // "Chạy với quyền admin": started by hand with normal rights → let the task start the admin copy.
    // Not when the task started us (a standard user's task has no admin rights: that would loop).
    let from_task = std::env::args().any(|a| a == autostart::FROM_TASK_ARG);
    if settings.run_as_admin && !from_task && !is_elevated() && autostart::start_via_task() {
        return;
    }

    if !acquire_mutex() {
        // Launched again: open the running instance's settings instead of a second copy
        let running = tray::find_running();
        if running.is_null() {
            message_box(
                "GoNhanh đang chạy.\nKiểm tra khay hệ thống (system tray).",
                MB_ICONINFORMATION,
            );
        } else {
            unsafe {
                AllowSetForegroundWindow(ASFW_ANY);
                PostMessageW(running, app::WM_APP_SHOW_SETTINGS, 0, 0);
            }
        }
        return;
    }

    updater::cleanup_old_version();
    views::theme::init(); // menus follow the Windows light/dark app mode
    let icc = INITCOMMONCONTROLSEX {
        dwSize: std::mem::size_of::<INITCOMMONCONTROLSEX>() as u32,
        dwICC: ICC_STANDARD_CLASSES | ICC_TAB_CLASSES | ICC_LISTVIEW_CLASSES,
    };
    unsafe { InitCommonControlsEx(&icc) };

    let remote = settings.remote_desktop_mode;
    let first_run = settings.first_run;
    let mut controller = InputController::new(settings, Box::new(crate::core::send::send_text));
    controller.is_non_latin_layout = Box::new(foreground::is_non_latin_layout);
    controller.play_sound = Box::new(win::play_toggle_sound);
    controller.on_enabled_changed = Box::new(tray::post_state_changed);
    app::install(controller);

    hook::set_process_injected(remote);
    if !hook::start() {
        message_box("Không cài được hook bàn phím.", MB_ICONERROR);
        return;
    }
    tray::create();

    // Per-app state and context reset on app switch
    foreground::start();
    if let Some(name) = foreground::current_app() {
        app::try_with(|c| c.on_app_changed(&name));
    }

    if first_run {
        let current = app::try_with(|c| c.settings.method).unwrap_or(settings::Method::Telex);
        let method = views::dialogs::run_onboarding(current);
        app::try_with(|c| {
            c.settings.method = method;
            c.settings.first_run = false;
            c.settings.save();
        });
        app::settings_changed();
    }
    tray::start_background_update_check();

    unsafe {
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
            let settings_hwnd = settings_window::hwnd();
            if !settings_hwnd.is_null() && IsDialogMessageW(settings_hwnd, &msg) != 0 {
                continue; // Tab / Esc navigation
            }
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    settings_window::close();
    hook::stop();
    foreground::stop();
    tray::destroy();
    release_mutex();
}
