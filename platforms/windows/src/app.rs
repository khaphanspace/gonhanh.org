//! App-wide state on the UI thread. Win32 callbacks (hooks, window procs) reach the controller through here.

use std::cell::RefCell;

use windows_sys::Win32::UI::WindowsAndMessaging::WM_APP;

use crate::controller::InputController;
use crate::win::{hook, tray};

pub const WM_APP_TRAY: u32 = WM_APP + 1; // tray icon callback
pub const WM_APP_UPDATE_DONE: u32 = WM_APP + 2; // worker → settings window, lParam = Box<UpdateResult>
pub const WM_APP_HOTKEY_SET: u32 = WM_APP + 3; // recorder → settings window, wParam = control id
pub const WM_APP_RECREATE: u32 = WM_APP + 4; // settings window: rebuild after DPI change
pub const WM_APP_SHOW_SETTINGS: u32 = WM_APP + 5; // second launch → running instance
pub const WM_APP_STATE_CHANGED: u32 = WM_APP + 6; // controller → tray, wParam = enabled
pub const WM_APP_BG_UPDATE: u32 = WM_APP + 7; // startup update check → tray, lParam = Box<Info>

thread_local! {
    static APP: RefCell<Option<InputController>> = const { RefCell::new(None) };
}

pub fn install(controller: InputController) {
    APP.with(|a| *a.borrow_mut() = Some(controller));
}

/// Run `f` with the controller. None when not installed or already borrowed (re-entrant call from
/// inside a handler, e.g. a modal loop) — callers treat that as "do nothing".
pub fn try_with<R>(f: impl FnOnce(&mut InputController) -> R) -> Option<R> {
    APP.with(|a| a.try_borrow_mut().ok()?.as_mut().map(f))
}

/// Settings changed outside the key path: engine + hook options + tray
pub fn settings_changed() {
    try_with(|c| {
        c.apply_settings();
        hook::set_process_injected(c.settings.remote_desktop_mode);
    });
    tray::update();
}
