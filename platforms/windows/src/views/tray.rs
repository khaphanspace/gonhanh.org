//! System tray icon + menu (macOS MenuBarController)

use std::cell::{Cell, RefCell};
use std::ptr::null_mut;

use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};

use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::HiDpi::{GetDpiForSystem, GetSystemMetricsForDpi};
use windows_sys::Win32::UI::Shell::*;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

use super::{dialogs, icon, settings_window, theme};
use crate::app::{self, WM_APP_BG_UPDATE, WM_APP_SHOW_SETTINGS, WM_APP_STATE_CHANGED, WM_APP_TRAY};
use crate::services::settings::Method;
use crate::services::updater;
use crate::win::wide;

const ICON_ID: u32 = 1;
const NIN_KEYSELECT_: u32 = NIN_SELECT | 1; // NINF_KEY | NIN_SELECT (not in windows-sys)
const CLASS: &str = "GoNhanhTray";
const ID_TOGGLE: u32 = 100;
const ID_TELEX: u32 = 101;
const ID_VNI: u32 = 102;
const ID_SETTINGS: u32 = 103;
const ID_UPDATE: u32 = 104;
const ID_ABOUT: u32 = 105;
const ID_FEEDBACK: u32 = 106;
const ID_EXIT: u32 = 107;

thread_local! {
    static HWND_: Cell<HWND> = const { Cell::new(null_mut()) };
    static ICON: Cell<HICON> = const { Cell::new(null_mut()) };
    static TASKBAR_CREATED: Cell<u32> = const { Cell::new(0) };
    static BALLOON_OPENS_UPDATE: RefCell<bool> = const { RefCell::new(false) };
}

/// Letter only, like the Windows ENG/VIE indicator: "V" = Vietnamese on, "E" = off;
/// white on a dark taskbar, black on a light one
fn make_icon(on: bool) -> HICON {
    let size = unsafe { GetSystemMetricsForDpi(SM_CXSMICON, GetDpiForSystem()) };
    let color = if icon::taskbar_is_light() {
        0x00_0000
    } else {
        0xFF_FFFF
    };
    icon::make(if on { "V" } else { "E" }, size, color)
}

fn copy_wide(dst: &mut [u16], s: &str) {
    let w: Vec<u16> = s.encode_utf16().take(dst.len() - 1).collect();
    dst[..w.len()].copy_from_slice(&w);
    dst[w.len()] = 0;
}

fn base_data() -> NOTIFYICONDATAW {
    let mut nid: NOTIFYICONDATAW = unsafe { std::mem::zeroed() };
    nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = HWND_.get();
    nid.uID = ICON_ID;
    nid
}

fn add_icon() {
    let mut nid = base_data();
    nid.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP | NIF_SHOWTIP;
    nid.uCallbackMessage = WM_APP_TRAY;
    nid.hIcon = ICON.get();
    unsafe {
        Shell_NotifyIconW(NIM_ADD, &nid);
        nid.Anonymous.uVersion = NOTIFYICON_VERSION_4;
        Shell_NotifyIconW(NIM_SETVERSION, &nid);
    }
    update();
}

/// Refresh icon + tooltip from the current settings
pub fn update() {
    if HWND_.get().is_null() {
        return;
    }
    let Some((enabled, method)) = app::try_with(|c| (c.settings.enabled, c.settings.method)) else {
        return;
    };
    let old = ICON.replace(make_icon(enabled));
    if !old.is_null() {
        unsafe { DestroyIcon(old) };
    }
    let mut nid = base_data();
    nid.uFlags = NIF_ICON | NIF_TIP | NIF_SHOWTIP;
    nid.hIcon = ICON.get();
    let method = if method == Method::Telex {
        "Telex"
    } else {
        "VNI"
    };
    copy_wide(
        &mut nid.szTip,
        &format!(
            "GoNhanh [{method}] - {}",
            if enabled { "Bật" } else { "Tắt" }
        ),
    );
    unsafe { Shell_NotifyIconW(NIM_MODIFY, &nid) };
}

/// Called from the controller (inside its borrow): defer the UI refresh to the message loop
pub fn post_state_changed(on: bool) {
    unsafe { PostMessageW(HWND_.get(), WM_APP_STATE_CHANGED, on as usize, 0) };
}

fn notify_update(info: &updater::Info) {
    BALLOON_OPENS_UPDATE.with(|b| *b.borrow_mut() = true);
    let mut nid = base_data();
    nid.uFlags = NIF_INFO;
    nid.dwInfoFlags = NIIF_INFO;
    copy_wide(
        &mut nid.szInfoTitle,
        &format!("Có bản GoNhanh mới: v{}", info.version),
    );
    copy_wide(&mut nid.szInfo, "Nhấn để cập nhật.");
    unsafe { Shell_NotifyIconW(NIM_MODIFY, &nid) };
}

fn show_menu() {
    let Some((enabled, method, toggle)) = app::try_with(|c| {
        (
            c.settings.enabled,
            c.settings.method,
            c.settings.toggle_shortcut,
        )
    }) else {
        return;
    };
    unsafe {
        // Plain menu like Windows' own tray menus: no title row, no bold default item
        let menu = CreatePopupMenu();
        let add = |flags: MENU_ITEM_FLAGS, id: u32, text: &str| {
            let w = wide(text);
            AppendMenuW(menu, flags, id as usize, w.as_ptr());
        };
        let separator = || AppendMenuW(menu, MF_SEPARATOR, 0, std::ptr::null());
        let shortcut = toggle.to_string().replace(" + ", "+"); // menu accelerator style: Ctrl+Space
        add(
            MF_STRING | if enabled { MF_CHECKED } else { 0 },
            ID_TOGGLE,
            &format!("Gõ tiếng Việt\t{shortcut}"),
        );
        separator();
        add(MF_STRING, ID_TELEX, "Telex");
        add(MF_STRING, ID_VNI, "VNI");
        CheckMenuRadioItem(
            menu,
            ID_TELEX,
            ID_VNI,
            if method == Method::Telex {
                ID_TELEX
            } else {
                ID_VNI
            },
            MF_BYCOMMAND,
        );
        separator();
        add(MF_STRING, ID_SETTINGS, "Cài đặt");
        add(MF_STRING, ID_UPDATE, "Kiểm tra cập nhật");
        add(MF_STRING, ID_ABOUT, "Giới thiệu GoNhanh");
        add(MF_STRING, ID_FEEDBACK, "Góp ý && báo lỗi");
        separator();
        add(MF_STRING, ID_EXIT, "Thoát GoNhanh");

        let mut pt = POINT { x: 0, y: 0 };
        GetCursorPos(&mut pt);
        SetForegroundWindow(HWND_.get()); // required so the menu closes when clicking elsewhere
        let cmd = TrackPopupMenuEx(
            menu,
            TPM_RETURNCMD | TPM_NONOTIFY | TPM_RIGHTBUTTON,
            pt.x,
            pt.y,
            HWND_.get(),
            std::ptr::null(),
        ) as u32;
        DestroyMenu(menu);

        match cmd {
            ID_TOGGLE => {
                app::try_with(|c| c.toggle());
            }
            ID_TELEX | ID_VNI => {
                app::try_with(|c| {
                    c.settings.method = if cmd == ID_TELEX {
                        Method::Telex
                    } else {
                        Method::Vni
                    };
                    c.settings.save();
                });
                app::settings_changed();
                settings_window::reload();
            }
            ID_SETTINGS => settings_window::show(false),
            ID_UPDATE => settings_window::show(true),
            ID_ABOUT => dialogs::show_about(null_mut()),
            ID_FEEDBACK => {
                crate::win::open_url("https://github.com/khaphanspace/gonhanh.org/issues")
            }
            ID_EXIT => PostQuitMessage(0),
            _ => {}
        }
    }
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_APP_TRAY => {
            match (lparam & 0xFFFF) as u32 {
                NIN_SELECT | NIN_KEYSELECT_ => {
                    app::try_with(|c| c.toggle()); // click = toggle, like EVKey
                }
                WM_CONTEXTMENU => show_menu(),
                NIN_BALLOONUSERCLICK if BALLOON_OPENS_UPDATE.with(|b| *b.borrow()) => {
                    settings_window::show(true)
                }
                _ => {}
            }
            0
        }
        WM_APP_STATE_CHANGED => {
            update();
            settings_window::enabled_changed(wparam != 0);
            0
        }
        // Light/dark switched: redraw the letter in the matching color, re-theme menus
        WM_SETTINGCHANGE
            if unsafe { crate::win::read_wide_z(lparam as *const u16, 64) }
                == "ImmersiveColorSet" =>
        {
            theme::refresh();
            settings_window::theme_changed();
            update();
            0
        }
        WM_APP_SHOW_SETTINGS => {
            settings_window::show(false);
            0
        }
        WM_APP_BG_UPDATE => {
            let info = unsafe { Box::from_raw(lparam as *mut updater::Info) };
            notify_update(&info);
            0
        }
        _ if msg == TASKBAR_CREATED.get() && msg != 0 => {
            add_icon(); // Explorer restarted
            0
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

pub fn create() -> bool {
    unsafe {
        let instance = GetModuleHandleW(std::ptr::null());
        let class = wide(CLASS);
        let mut wc: WNDCLASSW = std::mem::zeroed();
        wc.lpfnWndProc = Some(wnd_proc);
        wc.hInstance = instance;
        wc.lpszClassName = class.as_ptr();
        RegisterClassW(&wc);
        // Hidden top-level window (not message-only: the popup menu needs SetForegroundWindow)
        let title = wide("GoNhanh");
        let hwnd = CreateWindowExW(
            0,
            class.as_ptr(),
            title.as_ptr(),
            WS_OVERLAPPED,
            0,
            0,
            0,
            0,
            null_mut(),
            null_mut(),
            instance,
            std::ptr::null(),
        );
        if hwnd.is_null() {
            return false;
        }
        HWND_.set(hwnd);
        theme::allow_for_window(hwnd);
        let msg = wide("TaskbarCreated");
        TASKBAR_CREATED.set(RegisterWindowMessageW(msg.as_ptr()));
        // When running as admin, Explorer and a second normal launch have lower rights: let their
        // messages through (icon comes back after Explorer restarts, second launch opens settings)
        for m in [TASKBAR_CREATED.get(), WM_APP_SHOW_SETTINGS] {
            ChangeWindowMessageFilterEx(hwnd, m, MSGFLT_ALLOW, null_mut());
        }
        let enabled = app::try_with(|c| c.settings.enabled).unwrap_or(true);
        ICON.set(make_icon(enabled));
        add_icon();
        true
    }
}

/// Find the tray window of a running instance (second launch)
pub fn find_running() -> HWND {
    let class = wide(CLASS);
    unsafe { FindWindowW(class.as_ptr(), std::ptr::null()) }
}

/// Quietly check GitHub 10 s after start; notify only when a newer version exists
pub fn start_background_update_check() {
    let hwnd = HWND_.get() as usize; // HWND is not Send; it's just an id here
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(10));
        if let Ok(Some(info)) = updater::check() {
            let ptr = Box::into_raw(Box::new(info));
            if unsafe { PostMessageW(hwnd as HWND, WM_APP_BG_UPDATE, 0, ptr as LPARAM) } == 0 {
                drop(unsafe { Box::from_raw(ptr) });
            }
        }
    });
}

pub fn destroy() {
    let hwnd = HWND_.replace(null_mut());
    if hwnd.is_null() {
        return;
    }
    let mut nid = base_data();
    nid.hWnd = hwnd;
    unsafe {
        Shell_NotifyIconW(NIM_DELETE, &nid);
        DestroyWindow(hwnd);
    }
    let icon = ICON.replace(null_mut());
    if !icon.is_null() {
        unsafe { DestroyIcon(icon) };
    }
}
