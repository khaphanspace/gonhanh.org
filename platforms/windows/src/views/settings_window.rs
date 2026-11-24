//! Settings window (macOS MainSettingsView + AdvancedSettingsView) in plain Win32: a sidebar with the
//! app icon and the pages, settings as rows with switches on the right.
//! Every change is saved and applied immediately, like macOS.

use std::cell::{Cell, RefCell};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::ptr::null_mut;

use windows_sys::core::BOOL;
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::*;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::SystemInformation::GetTickCount;
use windows_sys::Win32::UI::Controls::Dialogs::*;
use windows_sys::Win32::UI::Controls::*;
use windows_sys::Win32::UI::HiDpi::{
    AdjustWindowRectExForDpi, GetDpiForWindow, GetSystemMetricsForDpi,
};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    EnableWindow, GetFocus, GetKeyState, IsWindowEnabled, SetFocus, TrackMouseEvent, TME_LEAVE,
    TRACKMOUSEEVENT, VK_CONTROL, VK_LWIN, VK_MENU, VK_RWIN, VK_SHIFT, VK_TAB,
};
use windows_sys::Win32::UI::Shell::{DefSubclassProc, SetWindowSubclass};
use windows_sys::Win32::UI::WindowsAndMessaging::*;

use super::{controls, dark, table};
use crate::app::{self, WM_APP_HOTKEY_SET, WM_APP_RECREATE, WM_APP_UPDATE_DONE};
use crate::core::hook;
use crate::core::hotkey::{Hotkey, MOD_ALT, MOD_CTRL, MOD_SHIFT, MOD_WIN};
use crate::core::keys;
use crate::services::autostart::{self, Outcome};
use crate::services::foreground;
use crate::services::settings::{self, Method, ShortcutItem};
use crate::services::{log, updater};
use crate::win::is_elevated;
use crate::win::{from_wide, scale, ui_font, wide};

// ---------------------------------------------------------------- ids & layout
// General
const ID_ENABLED: i32 = 1001;
const ID_TOGGLE_HOTKEY: i32 = 1002;
const ID_METHOD: i32 = 1003;
const ID_AUTO_W: i32 = 1005;
const ID_BRACKET: i32 = 1006;
const ID_ENGLISH: i32 = 1007;
const ID_MODERN: i32 = 1008;
const ID_FOREIGN: i32 = 1009;
const ID_AUTO_CAP: i32 = 1010;
const ID_AUTO_CAP_EXCLUDED: i32 = 1011;
const ID_RESTORE: i32 = 1012;
const ID_RESTORE_HOTKEY: i32 = 1013;
const ID_PER_APP: i32 = 1014;
const ID_SOUND: i32 = 1015;
const ID_AUTO_START: i32 = 1016;
const ID_UPDATE_BTN: i32 = 1017;
const ID_UPDATE_STATUS: i32 = 1018;
// Shortcuts
const ID_SHORTCUT_LIST: i32 = 1020;
const ID_SHORTCUT_KEY: i32 = 1021;
const ID_SHORTCUT_VALUE: i32 = 1022;
const ID_SHORTCUT_SAVE: i32 = 1023;
const ID_SHORTCUT_DELETE: i32 = 1024;
const ID_IMPORT: i32 = 1025;
const ID_EXPORT: i32 = 1026;
// Advanced
const ID_SECONDARY: i32 = 1030;
const ID_SECONDARY_HOTKEY: i32 = 1031;
const ID_REMOTE: i32 = 1032;
const ID_DEBUG_LOG: i32 = 1033;
const ID_OPEN_LOG: i32 = 1034;
const ID_PROFILE_LIST: i32 = 1035;
const ID_APP_PICKER: i32 = 1036;
const ID_PROFILE_STATE: i32 = 1037;
const ID_PROFILE_ADD: i32 = 1038;
const ID_PROFILE_REMOVE: i32 = 1039;
const ID_RUN_AS_ADMIN: i32 = 1040;
const ID_APP_LIST: i32 = 1044;
// About
const ID_WEBSITE: i32 = 1041;
const ID_GITHUB: i32 = 1042;
const ID_FEEDBACK: i32 = 1043;
const ID_SEPARATOR: i32 = 1099;
/// Sidebar items: ID_NAV + page index
const ID_NAV: i32 = 1100;
/// The text of a switch row: LABEL + switch id, so clicking the text flips the switch
const LABEL: i32 = 2000;
/// Its gray second line: SUB_LABEL + switch id
const SUB_LABEL: i32 = 4000;

const SS_LEFT: u32 = 0; // static control styles (not in windows-sys)
const SS_LEFTNOWORDWRAP: u32 = 0xC;
const SS_NOTIFY: u32 = 0x100;
const SIDEBAR_W: i32 = 192;
const CONTENT_W: i32 = 488; // page content spans x 16..472 → equal side margins
const CLIENT_H: i32 = 520;
/// Size of the window and its text, in % of the Windows scale
const ZOOM: u32 = 115;
/// Extra height over the original 474 px layout, given to the shortcut list
const GROW: i32 = CLIENT_H - 474;
const ROW_H: i32 = 40; // one settings row; with a gray second line: ROW_SUB_H
const ROW_SUB_H: i32 = 52;
const PAGE_TITLES: [&str; 4] = ["Cài đặt", "Gõ tắt", "Nâng cao", "Giới thiệu"];
const ABOUT_PAGE: usize = 3;
const STATE_NAMES: [&str; 3] = ["Tự động", "Luôn bật", "Luôn tắt"];
const MAIN_CLASS: &str = "GoNhanhSettings";
const PAGE_CLASS: &str = "GoNhanhSettingsPage";

/// Worker thread → window
pub struct UpdateResult {
    install_step: bool,
    info: Option<updater::Info>,
    restart: bool,
    error: Option<String>,
}

thread_local! {
    static MAIN: Cell<HWND> = const { Cell::new(null_mut()) };
    static PAGES: Cell<[HWND; 4]> = const { Cell::new([null_mut(); 4]) };
    static DPI: Cell<u32> = const { Cell::new(96) };
    static FONTS: Cell<[HFONT; 4]> = const { Cell::new([null_mut(); 4]) }; // normal, bold, small, title
    static ICON: Cell<HICON> = const { Cell::new(null_mut()) }; // sidebar app icon
    static PAGE_INDEX: Cell<usize> = const { Cell::new(0) };
    static SCROLL: Cell<(i32, i32)> = const { Cell::new((0, 0)) }; // Cài đặt page: offset, content height
    static SWITCHES: RefCell<HashMap<i32, bool>> = RefCell::new(HashMap::new()); // switch id → on
    static DROPDOWNS: RefCell<HashMap<i32, (Vec<String>, usize)>> = RefCell::new(HashMap::new()); // id → items, chosen
    static FIELDS: RefCell<HashMap<usize, (i32, i32, i32, i32)>> = RefCell::new(HashMap::new()); // framed control → insets
    static HOT: RefCell<HashSet<usize>> = RefCell::new(HashSet::new()); // buttons under the mouse
    static DIMMED: RefCell<HashSet<usize>> = RefCell::new(HashSet::new()); // grayed row texts
    static MENU_CLOSED: Cell<(usize, u32)> = const { Cell::new((0, 0)) }; // drop-down button, tick
    static MENU: RefCell<(Vec<String>, Option<usize>, i32)> = const { RefCell::new((Vec::new(), None, 0)) }; // open drop-down: items, chosen, button width
    static LOADING: Cell<bool> = const { Cell::new(false) }; // ignore notifications while filling controls
    static GRAY: RefCell<HashSet<usize>> = RefCell::new(HashSet::new()); // hint texts
    static HOTKEYS: RefCell<HashMap<i32, Hotkey>> = RefCell::new(HashMap::new()); // recorder id → value
    static RECORDING: Cell<HWND> = const { Cell::new(null_mut()) };
    static PENDING_MODS: Cell<u32> = const { Cell::new(0) };
    static BUSY: Cell<bool> = const { Cell::new(false) };
    static UPDATE: RefCell<Option<updater::Info>> = const { RefCell::new(None) };
    static REOPEN_AT: Cell<Option<(i32, i32)>> = const { Cell::new(None) };
}

fn s(v: i32) -> i32 {
    scale(v, DPI.get())
}

fn send(h: HWND, msg: u32, w: usize, l: isize) -> isize {
    unsafe { SendMessageW(h, msg, w, l) }
}

fn ctl(id: i32) -> HWND {
    for page in PAGES.get() {
        let h = unsafe { GetDlgItem(page, id) };
        if !h.is_null() {
            return h;
        }
    }
    null_mut()
}

fn is_switch(id: i32) -> bool {
    SWITCHES.with(|s| s.borrow().contains_key(&id))
}
fn checked(id: i32) -> bool {
    SWITCHES.with(|s| s.borrow().get(&id).copied().unwrap_or(false))
}
fn set_checked(id: i32, on: bool) {
    SWITCHES.with(|s| s.borrow_mut().insert(id, on));
    unsafe { InvalidateRect(ctl(id), std::ptr::null(), 0) };
}
fn text(id: i32) -> String {
    let h = ctl(id);
    let len = unsafe { GetWindowTextLengthW(h) } as usize;
    let mut buf = vec![0u16; len + 1];
    unsafe { GetWindowTextW(h, buf.as_mut_ptr(), buf.len() as i32) };
    from_wide(&buf)
}
fn set_text(id: i32, t: &str) {
    let w = wide(t);
    unsafe { SetWindowTextW(ctl(id), w.as_ptr()) };
}
/// Also grays the row text of a switch. The text stays enabled (a disabled static is drawn
/// embossed, ignoring our colors) and is painted gray from DIMMED instead.
fn enable(id: i32, on: bool) {
    let c = ctl(id);
    if !c.is_null() {
        unsafe { EnableWindow(c, on as BOOL) };
    }
    let label = ctl(LABEL + id);
    if !label.is_null() {
        DIMMED.with(|d| {
            let mut d = d.borrow_mut();
            if on {
                d.remove(&(label as usize));
            } else {
                d.insert(label as usize);
            }
        });
        unsafe { InvalidateRect(label, std::ptr::null(), 1) };
    }
}
fn message(text: &str) {
    let (t, c) = (wide(text), wide("GoNhanh"));
    unsafe { MessageBoxW(MAIN.get(), t.as_ptr(), c.as_ptr(), MB_ICONINFORMATION) };
}

// ---------------------------------------------------------------- control creation

#[allow(clippy::too_many_arguments)]
fn add(
    page: usize,
    class: &str,
    label: &str,
    style: u32,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    id: i32,
    font: usize,
    ex: u32,
) -> HWND {
    let (c, l) = (wide(class), wide(label));
    unsafe {
        let hwnd = CreateWindowExW(
            ex,
            c.as_ptr(),
            l.as_ptr(),
            WS_CHILD | WS_VISIBLE | style,
            s(x),
            s(y),
            s(w),
            s(h),
            PAGES.get()[page],
            id as isize as HMENU,
            GetModuleHandleW(std::ptr::null()),
            std::ptr::null(),
        );
        SendMessageW(hwnd, WM_SETFONT, FONTS.get()[font] as usize, 1);
        dark::style_control(hwnd, class, style);
        hwnd
    }
}
/// Owner-drawn switch (painted in draw_switch); its state lives in SWITCHES
fn switch(page: usize, id: i32, x: i32, y: i32) -> HWND {
    SWITCHES.with(|s| s.borrow_mut().insert(id, false));
    add(
        page,
        "Button",
        "",
        (BS_OWNERDRAW | BS_NOTIFY) as u32 | WS_TABSTOP,
        x,
        y,
        SWITCH_W,
        SWITCH_H,
        id,
        0,
        0,
    )
}
const SWITCH_W: i32 = 48; // the 40×20 switch plus room for the focus rectangle
const SWITCH_H: i32 = 26;
fn label(page: usize, t: &str, x: i32, y: i32, w: i32, id: i32) -> HWND {
    add(page, "Static", t, SS_LEFT, x, y, w, 20, id, 0, 0)
}
fn hint(page: usize, t: &str, x: i32, y: i32, w: i32, h: i32, id: i32) -> HWND {
    let c = add(page, "Static", t, SS_LEFT, x, y, w, h, id, 2, 0);
    GRAY.with(|g| g.borrow_mut().insert(c as usize));
    c
}
fn header(page: usize, t: &str, y: i32) -> HWND {
    add(page, "Static", t, SS_LEFT, 16, y, 440, 20, 0, 1, 0)
}
/// Owner-drawn push button (Windows 11 look, painted in draw_item), 28 px high
fn button(page: usize, id: i32, t: &str, x: i32, y: i32, w: i32) -> HWND {
    let b = add(
        page,
        "Button",
        t,
        (BS_OWNERDRAW | BS_NOTIFY) as u32 | WS_TABSTOP,
        x,
        y,
        w,
        BUTTON_H,
        id,
        0,
        0,
    );
    unsafe { SetWindowSubclass(b, Some(hover_proc), 5, 0) };
    b
}
const BUTTON_H: i32 = 30;

/// Button showing the chosen item with a chevron; a click opens the items as a menu
fn dropdown(page: usize, id: i32, items: &[&str], x: i32, y: i32, w: i32) -> HWND {
    DROPDOWNS.with(|d| {
        d.borrow_mut()
            .insert(id, (items.iter().map(|s| s.to_string()).collect(), 0))
    });
    button(page, id, items.first().copied().unwrap_or(""), x, y, w)
}
fn choice(id: i32) -> usize {
    DROPDOWNS.with(|d| d.borrow().get(&id).map_or(0, |(_, i)| *i))
}
fn set_choice(id: i32, index: usize) {
    let item = DROPDOWNS.with(|d| {
        let mut d = d.borrow_mut();
        let (items, sel) = d.get_mut(&id)?;
        *sel = index.min(items.len().saturating_sub(1));
        items.get(*sel).cloned()
    });
    if let Some(item) = item {
        set_text(id, &item); // the button shows its text
    }
}

/// Menu under a drop-down, as wide as the button and drawn like the rest of the window
/// (draw_menu_item); Some(index) when an item was picked
fn pick_from_menu(anchor: HWND, items: &[String], selected: Option<usize>) -> Option<usize> {
    unsafe {
        let mut r: RECT = std::mem::zeroed();
        GetWindowRect(anchor, &mut r);
        MENU.with(|m| *m.borrow_mut() = (items.to_vec(), selected, r.right - r.left));
        let menu = CreatePopupMenu();
        for i in 0..items.len() {
            // Owner-drawn: the item data is the index
            AppendMenuW(menu, MF_OWNERDRAW, i + 1, i as *const u16);
        }
        let info = MENUINFO {
            cbSize: std::mem::size_of::<MENUINFO>() as u32,
            fMask: MIM_STYLE | MIM_BACKGROUND,
            dwStyle: MNS_NOCHECK,
            cyMax: 0,
            hbrBack: dark::field_brush(),
            dwContextHelpID: 0,
            dwMenuData: 0,
        };
        SetMenuInfo(menu, &info);
        let cmd = TrackPopupMenu(
            menu,
            TPM_RETURNCMD | TPM_LEFTALIGN | TPM_TOPALIGN | TPM_NONOTIFY,
            r.left,
            r.bottom + s(4),
            0,
            MAIN.get(),
            std::ptr::null(),
        );
        DestroyMenu(menu);
        MENU_CLOSED.set((anchor as usize, GetTickCount()));
        (cmd > 0).then(|| cmd as usize - 1)
    }
}

/// WM_MEASUREITEM for drop-down menu items: the menu's outer width matches the button
fn measure_menu_item(mis: &mut MEASUREITEMSTRUCT) {
    let width = MENU.with(|m| m.borrow().2);
    // The popup adds its own border on each side
    let border = unsafe { GetSystemMetricsForDpi(SM_CXEDGE, DPI.get()) } + s(MENU_INSET);
    mis.itemWidth = (width - 2 * border).max(s(60)) as u32;
    mis.itemHeight = s(34) as u32;
}
const MENU_INSET: i32 = 7; // measured: the popup is about 10 px wider than its items at 100%

fn draw_menu_item(dis: &DRAWITEMSTRUCT) {
    let p = dark::palette();
    let index = dis.itemData;
    let (text, chosen) = MENU.with(|m| {
        let m = m.borrow();
        (
            m.0.get(index).cloned().unwrap_or_default(),
            m.1 == Some(index),
        )
    });
    let r = dis.rcItem;
    unsafe {
        FillRect(dis.hDC, &r, dark::field_brush());
        if dis.itemState & ODS_SELECTED != 0 {
            let hl = RECT {
                left: r.left + s(2),
                top: r.top + s(1),
                right: r.right - s(2),
                bottom: r.bottom - s(1),
            };
            controls::paint_highlight(dis.hDC, &hl, p.row_hot, p.field, DPI.get());
        }
        SelectObject(dis.hDC, FONTS.get()[0] as HGDIOBJ);
        SetBkMode(dis.hDC, TRANSPARENT as i32);
        let mut tr = RECT {
            left: r.left + s(10),
            right: r.right - s(28),
            ..r
        };
        SetTextColor(dis.hDC, p.text);
        let mut t: Vec<u16> = text.encode_utf16().collect();
        DrawTextW(
            dis.hDC,
            t.as_mut_ptr(),
            t.len() as i32,
            &mut tr,
            DT_SINGLELINE | DT_VCENTER | DT_LEFT | DT_NOPREFIX,
        );
        if chosen {
            let mut mark: Vec<u16> = "✓".encode_utf16().collect();
            let mut mr = RECT {
                left: r.right - s(28),
                right: r.right - s(8),
                ..r
            };
            SetTextColor(dis.hDC, dark::accent());
            DrawTextW(
                dis.hDC,
                mark.as_mut_ptr(),
                mark.len() as i32,
                &mut mr,
                DT_SINGLELINE | DT_VCENTER | DT_CENTER,
            );
        }
    }
}

/// Hover highlight for owner-drawn buttons
/// The click that closes an open menu also reaches its button: true right after the menu under
/// `anchor` closed, so that click closes the menu instead of opening it again
fn menu_just_closed(anchor: HWND) -> bool {
    let (closed_by, at) = MENU_CLOSED.get();
    closed_by == anchor as usize && unsafe { GetTickCount() }.wrapping_sub(at) < 400
}

/// Multi-line text box: Tab moves to the next control instead of typing a tab
unsafe extern "system" fn multiline_proc(
    h: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _: usize,
    _: usize,
) -> LRESULT {
    let r = unsafe { DefSubclassProc(h, msg, wparam, lparam) };
    if msg == WM_GETDLGCODE && lparam != 0 {
        let m = unsafe { &*(lparam as *const MSG) };
        if m.message == WM_KEYDOWN && m.wParam == VK_TAB as usize {
            return r & !(DLGC_WANTALLKEYS | DLGC_WANTTAB) as LRESULT;
        }
    }
    r
}

unsafe extern "system" fn hover_proc(
    h: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _: usize,
    _: usize,
) -> LRESULT {
    match msg {
        WM_MOUSEMOVE if HOT.with(|hot| hot.borrow_mut().insert(h as usize)) => unsafe {
            let mut tme = TRACKMOUSEEVENT {
                cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                dwFlags: TME_LEAVE,
                hwndTrack: h,
                dwHoverTime: 0,
            };
            TrackMouseEvent(&mut tme);
            InvalidateRect(h, std::ptr::null(), 0);
        },
        WM_MOUSELEAVE if HOT.with(|hot| hot.borrow_mut().remove(&(h as usize))) => unsafe {
            InvalidateRect(h, std::ptr::null(), 0);
        },
        _ => {}
    }
    unsafe { DefSubclassProc(h, msg, wparam, lparam) }
}

/// Borderless text box inside a painted rounded frame at (x, y, w, h)
#[allow(clippy::too_many_arguments)]
fn edit(page: usize, id: i32, x: i32, y: i32, w: i32, h: i32, style: u32, cue: &str) -> HWND {
    let multiline = style & ES_MULTILINE as u32 != 0;
    let (ix, iy, iw, ih) = if multiline {
        (x + 8, y + 5, w - 10, h - 10)
    } else {
        (x + 10, y + (h - 18) / 2, w - 20, 18)
    };
    let e = add(
        page,
        "Edit",
        "",
        style | WS_TABSTOP,
        ix,
        iy,
        iw,
        ih,
        id,
        0,
        0,
    );
    frame(e, x, y, w, h);
    if multiline {
        unsafe { SetWindowSubclass(e, Some(multiline_proc), 6, 0) };
    }
    if !cue.is_empty() {
        let c = wide(cue);
        send(e, EM_SETCUEBANNER, 1, c.as_ptr() as isize);
    }
    e
}

/// Remember the painted frame around control `c` (logical x, y, w, h) as insets from its edges
fn frame(c: HWND, x: i32, y: i32, w: i32, h: i32) {
    let inner = child_rect(c);
    let insets = (
        inner.left - s(x),
        inner.top - s(y),
        s(x) + s(w) - inner.right,
        s(y) + s(h) - inner.bottom,
    );
    FIELDS.with(|f| f.borrow_mut().insert(c as usize, insets));
}

/// Child window rect in its parent's client coordinates
fn child_rect(c: HWND) -> RECT {
    unsafe {
        let mut r: RECT = std::mem::zeroed();
        GetWindowRect(c, &mut r);
        MapWindowPoints(
            null_mut(),
            GetParent(c),
            &mut r as *mut RECT as *mut POINT,
            2,
        );
        r
    }
}

fn frame_rect(c: HWND) -> Option<RECT> {
    let (l, t, r, b) = FIELDS.with(|f| f.borrow().get(&(c as usize)).copied())?;
    let inner = child_rect(c);
    Some(RECT {
        left: inner.left - l,
        top: inner.top - t,
        right: inner.right + r,
        bottom: inner.bottom + b,
    })
}

/// Repaint the frame around `c` (focus line)
fn invalidate_frame(c: HWND) {
    if let Some(r) = frame_rect(c) {
        unsafe { InvalidateRect(GetParent(c), &r, 1) };
    }
}

/// Page WM_PAINT: the frames of its text boxes and lists
fn paint_frames(page: HWND) {
    unsafe {
        let mut ps: PAINTSTRUCT = std::mem::zeroed();
        let dc = BeginPaint(page, &mut ps);
        let focus = GetFocus();
        let fields: Vec<usize> = FIELDS.with(|f| f.borrow().keys().copied().collect());
        for c in fields {
            let c = c as HWND;
            if GetParent(c) != page {
                continue;
            }
            if let Some(r) = frame_rect(c) {
                controls::paint_field(dc, &r, focus == c, DPI.get());
            }
        }
        EndPaint(page, &ps);
    }
}

// ---------------------------------------------------------------- shortcut recorder (button subclass)

fn held_mods() -> u32 {
    let down = |vk: u16| unsafe { GetKeyState(vk as i32) } < 0;
    let mut m = 0;
    if down(VK_CONTROL) {
        m |= MOD_CTRL;
    }
    if down(VK_MENU) {
        m |= MOD_ALT;
    }
    if down(VK_SHIFT) {
        m |= MOD_SHIFT;
    }
    if down(VK_LWIN) || down(VK_RWIN) {
        m |= MOD_WIN;
    }
    m
}

fn hotkey(id: i32) -> Hotkey {
    HOTKEYS.with(|h| h.borrow().get(&id).copied().unwrap_or_default())
}

fn stop_recording() {
    let rec = RECORDING.replace(null_mut());
    if rec.is_null() {
        return;
    }
    hook::set_suspended(false);
    let id = unsafe { GetDlgCtrlID(rec) };
    let w = wide(&hotkey(id).to_string());
    unsafe { SetWindowTextW(rec, w.as_ptr()) };
}

fn start_recording(rec: HWND) {
    stop_recording();
    RECORDING.set(rec);
    PENDING_MODS.set(0);
    hook::set_suspended(true); // the old shortcut must not fire while recording
    let w = wide("Nhấn phím");
    unsafe {
        SetWindowTextW(rec, w.as_ptr());
        SetFocus(rec);
    }
}

fn finish_recording(rec: HWND, hk: Hotkey) {
    let id = unsafe { GetDlgCtrlID(rec) };
    HOTKEYS.with(|h| h.borrow_mut().insert(id, hk));
    stop_recording();
    unsafe { PostMessageW(MAIN.get(), WM_APP_HOTKEY_SET, id as usize, 0) };
}

unsafe extern "system" fn recorder_proc(
    h: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _: usize,
    _: usize,
) -> LRESULT {
    let recording = RECORDING.get() == h;
    if recording {
        match msg {
            WM_GETDLGCODE => return DLGC_WANTALLKEYS as LRESULT,
            WM_KEYDOWN | WM_SYSKEYDOWN => {
                let vk = wparam as u16;
                if keys::is_modifier(vk) {
                    PENDING_MODS.set(PENDING_MODS.get() | held_mods());
                    let w = wide(&format!(
                        "{} +",
                        Hotkey {
                            vk: 0,
                            mods: PENDING_MODS.get()
                        }
                    ));
                    unsafe { SetWindowTextW(h, w.as_ptr()) };
                } else {
                    finish_recording(
                        h,
                        Hotkey {
                            vk,
                            mods: held_mods(),
                        },
                    );
                }
                return 0;
            }
            WM_KEYUP | WM_SYSKEYUP => {
                // Modifiers released without another key: modifier-only shortcut (Ctrl+Shift)
                if keys::is_modifier(wparam as u16) && PENDING_MODS.get() != 0 {
                    finish_recording(
                        h,
                        Hotkey {
                            vk: 0,
                            mods: PENDING_MODS.get(),
                        },
                    );
                }
                return 0;
            }
            WM_CHAR | WM_SYSCHAR => return 0,
            WM_KILLFOCUS => stop_recording(),
            _ => {}
        }
    }
    unsafe { DefSubclassProc(h, msg, wparam, lparam) }
}

fn recorder(page: usize, id: i32, value: Hotkey, x: i32, y: i32, w: i32) {
    HOTKEYS.with(|h| h.borrow_mut().insert(id, value));
    let b = button(page, id, &value.to_string(), x, y, w);
    unsafe { SetWindowSubclass(b, Some(recorder_proc), 1, 0) };
}

// ---------------------------------------------------------------- settings rows

/// What sits left of a row's switch
enum Extra<'a> {
    None,
    Button(i32, &'a str, i32),
    /// Text box: id, gray hint shown while empty, width
    Edit(i32, &'a str, i32),
}

/// Lays out a page as rows: text on the left, the control against the right edge
struct Rows {
    page: usize,
    y: i32,
    right: i32,
}

impl Rows {
    fn new(page: usize, right: i32) -> Self {
        Rows { page, y: 8, right }
    }

    /// Space between groups of rows
    fn gap(&mut self) {
        self.y += 12;
    }

    fn height(sub: Option<&str>) -> i32 {
        if sub.is_some() {
            ROW_SUB_H
        } else {
            ROW_H
        }
    }

    /// Row text (and a gray second line) up to `width`; `id` LABEL + switch id makes it clickable
    fn text(&self, id: i32, label: &str, sub: Option<&str>, width: i32) {
        let h = Self::height(sub);
        let top = if sub.is_some() {
            self.y + 7
        } else {
            self.y + (h - 20) / 2
        };
        let notify = if id != 0 { SS_NOTIFY } else { 0 };
        add(
            self.page,
            "Static",
            label,
            SS_LEFTNOWORDWRAP | notify,
            16,
            top,
            width,
            20,
            id,
            0,
            0,
        );
        if let Some(sub) = sub {
            let c = add(
                self.page,
                "Static",
                sub,
                SS_LEFTNOWORDWRAP | notify,
                16,
                top + 20,
                width,
                18,
                if id != 0 { id - LABEL + SUB_LABEL } else { 0 },
                2,
                0,
            );
            GRAY.with(|g| g.borrow_mut().insert(c as usize));
        }
    }

    /// Switch row, optionally with a button left of the switch
    fn switch(&mut self, id: i32, label: &str, sub: Option<&str>, extra: Extra) {
        let h = Self::height(sub);
        let x = self.right - SWITCH_W + 4; // the switch itself ends at `right`
        switch(self.page, id, x, self.y + (h - SWITCH_H) / 2);
        let mut text_end = x - 8;
        match extra {
            Extra::None => {}
            Extra::Button(bid, t, w) => {
                text_end -= w + 8;
                button(
                    self.page,
                    bid,
                    t,
                    text_end + 8,
                    self.y + (h - BUTTON_H) / 2,
                    w,
                );
            }
            Extra::Edit(eid, cue, w) => {
                text_end -= w + 8;
                edit(
                    self.page,
                    eid,
                    text_end + 8,
                    self.y + (h - BUTTON_H) / 2,
                    w,
                    BUTTON_H,
                    ES_AUTOHSCROLL as u32,
                    cue,
                );
            }
        }
        self.text(LABEL + id, label, sub, text_end - 16 - 8);
        self.y += h;
    }

    /// Shortcut recorder row; with `switch`, the shortcut can be turned off
    fn recorder(&mut self, id: i32, label: &str, value: Hotkey, switch_id: Option<i32>) {
        let mut x = self.right;
        let mut label_id = 0;
        if let Some(sid) = switch_id {
            x = self.right - SWITCH_W + 4;
            switch(self.page, sid, x, self.y + (ROW_H - SWITCH_H) / 2);
            x -= 8;
            label_id = LABEL + sid;
        }
        recorder(
            self.page,
            id,
            value,
            x - 150,
            self.y + (ROW_H - 28) / 2,
            150,
        );
        self.text(label_id, label, None, x - 150 - 16 - 8);
        self.y += ROW_H;
    }

    /// Drop-down list row
    fn combo(&mut self, id: i32, label: &str, items: &[&str]) {
        let w = 130;
        dropdown(
            self.page,
            id,
            items,
            self.right - w,
            self.y + (ROW_H - BUTTON_H) / 2,
            w,
        );
        self.text(0, label, None, self.right - w - 16 - 8);
        self.y += ROW_H;
    }
}

// ---------------------------------------------------------------- lists

/// Table (views::table) inside a painted rounded frame at (x, y, w, h)
#[allow(clippy::too_many_arguments)]
fn list_view(
    page: usize,
    id: i32,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    columns: &[(&str, i32)],
    checkboxes: bool,
) -> HWND {
    table::register();
    let t = add(
        page,
        table::CLASS,
        "",
        WS_TABSTOP,
        x + 3,
        y + 3,
        w - 6,
        h - 6,
        id,
        0,
        0,
    );
    table::init(t, columns, checkboxes);
    send(t, WM_SETFONT, FONTS.get()[0] as usize, 0);
    frame(t, x, y, w, h); // rounded border painted by the page
    t
}

fn selected_row(id: i32) -> Option<usize> {
    table::selected(ctl(id))
}

fn fill_shortcuts(select: Option<usize>) {
    let Some(items) = app::try_with(|c| c.settings.shortcuts.clone()) else {
        return;
    };
    let t = ctl(ID_SHORTCUT_LIST);
    table::set_rows(
        t,
        items
            .iter()
            .map(|i| vec![i.key.clone(), i.value.replace('\n', " ⏎ ")])
            .collect(),
        items.iter().map(|i| i.enabled).collect(),
    );
    table::select(t, select);
}

fn state_index(state: i32) -> usize {
    match state {
        1 => 1,
        -1 => 2,
        _ => 0,
    }
}

fn fill_profiles(select: Option<&str>) {
    let Some(profiles) = app::try_with(|c| c.settings.per_app_profiles.clone()) else {
        return;
    };
    let t = ctl(ID_PROFILE_LIST);
    let rows: Vec<Vec<String>> = profiles
        .iter()
        .map(|(app, state)| vec![app.clone(), STATE_NAMES[state_index(*state)].to_string()])
        .collect();
    let selected = select.and_then(|s| rows.iter().position(|r| r[0] == s));
    table::set_rows(t, rows, Vec::new());
    table::select(t, selected);
}

// ---------------------------------------------------------------- apply

fn update_dependent_controls() {
    let telex = choice(ID_METHOD) != 1;
    enable(ID_AUTO_W, telex);
    enable(ID_BRACKET, telex);
    enable(ID_AUTO_CAP_EXCLUDED, checked(ID_AUTO_CAP));
    enable(ID_RESTORE_HOTKEY, checked(ID_RESTORE));
    enable(ID_SECONDARY_HOTKEY, checked(ID_SECONDARY));
}

/// Read every control into settings, save, and push to the engine
fn commit() {
    let excluded = text(ID_AUTO_CAP_EXCLUDED).trim().to_string();
    let (toggle, restore, secondary) = (
        hotkey(ID_TOGGLE_HOTKEY),
        hotkey(ID_RESTORE_HOTKEY),
        hotkey(ID_SECONDARY_HOTKEY),
    );
    app::try_with(|c| {
        let st = &mut c.settings;
        st.method = if choice(ID_METHOD) == 1 {
            Method::Vni
        } else {
            Method::Telex
        };
        st.auto_w_shortcut = checked(ID_AUTO_W);
        st.bracket_shortcut = checked(ID_BRACKET);
        st.english_auto_restore = checked(ID_ENGLISH);
        st.modern_tone = checked(ID_MODERN);
        st.allow_foreign_consonants = checked(ID_FOREIGN);
        st.auto_capitalize = checked(ID_AUTO_CAP);
        st.auto_capitalize_excluded_apps = excluded;
        st.restore_shortcut_enabled = checked(ID_RESTORE);
        st.per_app_mode = checked(ID_PER_APP);
        st.sound_enabled = checked(ID_SOUND);
        st.auto_start = checked(ID_AUTO_START);
        st.run_as_admin = checked(ID_RUN_AS_ADMIN);
        st.secondary_toggle_enabled = checked(ID_SECONDARY);
        st.remote_desktop_mode = checked(ID_REMOTE);
        st.debug_log = checked(ID_DEBUG_LOG);
        st.toggle_shortcut = toggle;
        st.restore_shortcut = restore;
        st.secondary_toggle_shortcut = secondary;
        st.save();
    });
    app::settings_changed();
    update_dependent_controls();
}

fn save_shortcuts(select: Option<usize>) {
    app::try_with(|c| {
        c.settings.save_shortcuts();
        c.apply_settings();
    });
    fill_shortcuts(select);
}

// ---------------------------------------------------------------- shortcut page actions

fn save_shortcut_from_fields() {
    let key = text(ID_SHORTCUT_KEY).trim().to_string();
    let value = text(ID_SHORTCUT_VALUE).replace("\r\n", "\n");
    if key.is_empty() || value.is_empty() {
        message("Nhập cả viết tắt và nội dung.");
        return;
    }
    let row = app::try_with(|c| {
        let list = &mut c.settings.shortcuts;
        match list.iter().position(|s| s.key == key) {
            Some(i) => {
                list[i].value = value;
                i
            }
            None => {
                list.push(ShortcutItem::new(&key, &value, true));
                list.len() - 1
            }
        }
    });
    save_shortcuts(row);
    set_text(ID_SHORTCUT_KEY, "");
    set_text(ID_SHORTCUT_VALUE, "");
    unsafe { SetFocus(ctl(ID_SHORTCUT_KEY)) };
}

fn file_dialog(save: bool) -> Option<String> {
    let mut path = [0u16; 260];
    if save {
        let name: Vec<u16> = "gonhanh-shortcuts.txt".encode_utf16().collect();
        path[..name.len()].copy_from_slice(&name);
    }
    let filter: Vec<u16> = "Bảng gõ tắt (*.txt)\0*.txt\0Tất cả\0*.*\0\0"
        .encode_utf16()
        .collect();
    let ext = wide("txt");
    let mut ofn: OPENFILENAMEW = unsafe { std::mem::zeroed() };
    ofn.lStructSize = std::mem::size_of::<OPENFILENAMEW>() as u32;
    ofn.hwndOwner = MAIN.get();
    ofn.lpstrFilter = filter.as_ptr();
    ofn.lpstrFile = path.as_mut_ptr();
    ofn.nMaxFile = path.len() as u32;
    ofn.lpstrDefExt = ext.as_ptr();
    ofn.Flags = if save {
        OFN_OVERWRITEPROMPT
    } else {
        OFN_FILEMUSTEXIST
    };
    let ok = unsafe {
        if save {
            GetSaveFileNameW(&mut ofn)
        } else {
            GetOpenFileNameW(&mut ofn)
        }
    } != 0;
    ok.then(|| from_wide(&path))
}

fn import_shortcuts() {
    let Some(path) = file_dialog(false) else {
        return;
    };
    let Ok(bytes) = std::fs::read(&path) else {
        message("Không đọc được file.");
        return;
    };
    let content =
        String::from_utf8_lossy(bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(&bytes)).into_owned();
    let count = app::try_with(|c| settings::import_shortcuts(&content, &mut c.settings.shortcuts))
        .unwrap_or(0);
    save_shortcuts(None);
    message(&format!("Đã nhập {count} mục gõ tắt."));
}

fn export_shortcuts() {
    let Some(path) = file_dialog(true) else {
        return;
    };
    let Some(text) = app::try_with(|c| settings::export_shortcuts(&c.settings.shortcuts)) else {
        return;
    };
    if std::fs::write(&path, text).is_err() {
        message("Không ghi được file.");
    }
}

// ---------------------------------------------------------------- per-app page actions

fn state_from_combo() -> i32 {
    match choice(ID_PROFILE_STATE) {
        1 => 1,
        2 => -1,
        _ => 0,
    }
}

unsafe extern "system" fn collect_app(w: HWND, apps: LPARAM) -> BOOL {
    unsafe {
        if IsWindowVisible(w) == 0
            || GetWindowTextLengthW(w) == 0
            || GetWindowLongW(w, GWL_EXSTYLE) as u32 & WS_EX_TOOLWINDOW != 0
        {
            return 1;
        }
        if let Some(name) = foreground::process_name(w) {
            if !name.eq_ignore_ascii_case("gonhanh.exe") {
                (*(apps as *mut BTreeSet<String>)).insert(name.to_lowercase());
            }
        }
    }
    1
}

/// Chevron next to the app box: pick one of the running apps
fn pick_app() {
    if menu_just_closed(ctl(ID_APP_LIST)) {
        return;
    }
    let mut apps: BTreeSet<String> = BTreeSet::new();
    unsafe { EnumWindows(Some(collect_app), &mut apps as *mut _ as LPARAM) };
    let apps: Vec<String> = apps.into_iter().collect();
    if let Some(i) = pick_from_menu(ctl(ID_APP_LIST), &apps, None) {
        set_text(ID_APP_PICKER, &apps[i]);
    }
}

fn add_profile() {
    let typed = text(ID_APP_PICKER).trim().to_string();
    if typed.is_empty() {
        return;
    }
    let app_name = settings::normalize_app(&typed).to_lowercase();
    let state = state_from_combo();
    app::try_with(|c| {
        c.settings.per_app_profiles.insert(app_name.clone(), state);
        c.settings.save();
    });
    fill_profiles(Some(&app_name));
    set_text(ID_APP_PICKER, "");
}

// ---------------------------------------------------------------- updates

/// Version and update status share one label: "GoNhanh v1.0.165 · Bạn đang dùng bản mới nhất."
fn set_status(status: &str) {
    let version = format!("GoNhanh v{}", updater::VERSION);
    set_text(
        ID_UPDATE_STATUS,
        &if status.is_empty() {
            version
        } else {
            format!("{version} · {status}")
        },
    );
}

fn run_update_step() {
    if BUSY.get() {
        return;
    }
    BUSY.set(true);
    enable(ID_UPDATE_BTN, false);
    let info = UPDATE.with(|u| u.borrow().clone());
    set_status(if info.is_some() {
        "Đang tải bản cập nhật"
    } else {
        "Đang kiểm tra cập nhật"
    });
    let hwnd = MAIN.get() as usize;
    std::thread::spawn(move || {
        let result = match &info {
            Some(i) => match updater::install(i) {
                Ok(restart) => UpdateResult {
                    install_step: true,
                    info: info.clone(),
                    restart,
                    error: None,
                },
                Err(e) => UpdateResult {
                    install_step: true,
                    info: None,
                    restart: false,
                    error: Some(e),
                },
            },
            None => match updater::check() {
                Ok(found) => UpdateResult {
                    install_step: false,
                    info: found,
                    restart: false,
                    error: None,
                },
                Err(e) => UpdateResult {
                    install_step: false,
                    info: None,
                    restart: false,
                    error: Some(e),
                },
            },
        };
        let ptr = Box::into_raw(Box::new(result));
        if unsafe { PostMessageW(hwnd as HWND, WM_APP_UPDATE_DONE, 0, ptr as LPARAM) } == 0 {
            drop(unsafe { Box::from_raw(ptr) });
        }
    });
}

fn on_update_done(r: UpdateResult) {
    BUSY.set(false);
    enable(ID_UPDATE_BTN, true);
    if let Some(e) = r.error {
        set_status(&format!("Lỗi: {e}"));
        log::write(&format!("Update failed: {e}"));
        return;
    }
    if r.install_step {
        if r.restart {
            crate::restart_after_update();
        } else {
            set_status("Đã mở trang tải về.");
        }
        return;
    }
    UPDATE.with(|u| *u.borrow_mut() = r.info.clone());
    match r.info {
        None => set_status("Bạn đang dùng bản mới nhất."),
        Some(info) if info.zip_url.is_some() => {
            set_status(&format!("Có bản mới v{}.", info.version));
            set_text(ID_UPDATE_BTN, &format!("Cập nhật lên v{}", info.version));
        }
        Some(info) => {
            set_status(&format!(
                "Có bản mới v{}. Bản này chưa có gói Windows tự cập nhật, sẽ mở trang tải về.",
                info.version
            ));
            set_text(ID_UPDATE_BTN, "Mở trang tải về");
        }
    }
}

// ---------------------------------------------------------------- build pages

// Rows: text at x 16, controls against the right edge (x 472).

fn build_general(st: &settings::Settings) {
    let mut rows = Rows::new(0, CONTENT_W - 16);
    rows.switch(ID_ENABLED, "Bộ gõ tiếng Việt", None, Extra::None);
    rows.recorder(
        ID_TOGGLE_HOTKEY,
        "Phím tắt bật/tắt",
        st.toggle_shortcut,
        None,
    );
    rows.combo(ID_METHOD, "Kiểu gõ", &["Telex", "VNI"]);
    rows.switch(ID_AUTO_W, "Gõ W thành Ư ở đầu từ", None, Extra::None);
    rows.switch(ID_BRACKET, "Gõ ] thành Ư, [ thành Ơ", None, Extra::None);
    rows.gap();
    rows.switch(ID_ENGLISH, "Tự khôi phục từ tiếng Anh", None, Extra::None);
    rows.switch(ID_MODERN, "Đặt dấu kiểu mới (oà, uý)", None, Extra::None);
    rows.switch(ID_FOREIGN, "Phụ âm ngoại (z, w, j, f)", None, Extra::None);
    rows.switch(
        ID_AUTO_CAP,
        "Tự viết hoa đầu câu",
        None,
        Extra::Edit(
            ID_AUTO_CAP_EXCLUDED,
            "Trừ: code.exe, WindowsTerminal.exe",
            230,
        ),
    );
    rows.recorder(
        ID_RESTORE_HOTKEY,
        "Hoàn tác dấu vừa gõ",
        st.restore_shortcut,
        Some(ID_RESTORE),
    );
    rows.gap();
    rows.switch(
        ID_PER_APP,
        "Nhớ trạng thái bật/tắt theo từng ứng dụng",
        None,
        Extra::None,
    );
    rows.switch(ID_SOUND, "Âm thanh khi bật/tắt", None, Extra::None);
    rows.switch(ID_AUTO_START, "Khởi động cùng Windows", None, Extra::None);
    rows.switch(ID_RUN_AS_ADMIN, "Chạy với quyền admin", None, Extra::None);
    set_scroll_height(s(rows.y + 8));
}

fn build_shortcuts() {
    hint(
        1,
        "Gõ viết tắt rồi nhấn Space hoặc dấu câu để thay. Tích ô để bật, chọn dòng để sửa.",
        16,
        12,
        456,
        18,
        0,
    );
    list_view(
        1,
        ID_SHORTCUT_LIST,
        16,
        36,
        456,
        224 + GROW,
        &[("Viết tắt", 120), ("Nội dung", -1)],
        true,
    );
    // Under the list: the key box, then the content box down to the button row at the bottom
    let key_y = 36 + 224 + GROW + 12;
    let value_y = key_y + BUTTON_H + 8;
    let buttons_y = CLIENT_H - 16 - BUTTON_H;
    label(1, "Viết tắt", 16, key_y + 5, 64, 0);
    edit(
        1,
        ID_SHORTCUT_KEY,
        84,
        key_y,
        388,
        BUTTON_H,
        ES_AUTOHSCROLL as u32,
        "vd: hcm",
    );
    label(1, "Nội dung", 16, value_y + 6, 64, 0);
    edit(
        1,
        ID_SHORTCUT_VALUE,
        84,
        value_y,
        388,
        buttons_y - 10 - value_y,
        (ES_MULTILINE | ES_AUTOVSCROLL | ES_WANTRETURN) as u32,
        "",
    );
    button(1, ID_SHORTCUT_SAVE, "Thêm / Cập nhật", 84, buttons_y, 124);
    button(1, ID_SHORTCUT_DELETE, "Xoá", 214, buttons_y, 70);
    button(1, ID_IMPORT, "Nhập từ file", 316, buttons_y, 76);
    button(1, ID_EXPORT, "Xuất ra file", 396, buttons_y, 76);
    fill_shortcuts(None);
}

fn build_advanced(st: &settings::Settings) {
    let mut rows = Rows::new(2, CONTENT_W - 16);
    rows.recorder(
        ID_SECONDARY_HOTKEY,
        "Phím tắt bật/tắt phụ",
        st.secondary_toggle_shortcut,
        Some(ID_SECONDARY),
    );
    rows.switch(ID_REMOTE, "Remote desktop", None, Extra::None);
    rows.switch(
        ID_DEBUG_LOG,
        "Debug log",
        None,
        Extra::Button(ID_OPEN_LOG, "Mở log", 80),
    );
    rows.gap();

    let top = rows.y + 8;
    header(2, "Tuỳ chỉnh theo ứng dụng", top);
    hint(2, "Luôn bật hoặc luôn tắt tiếng Việt trong từng ứng dụng. “Tự động” = theo trạng thái đã nhớ.", 16, top + 24, 456, 18, 0);
    let bottom = CLIENT_H - 16 - 28; // the add/remove row
    list_view(
        2,
        ID_PROFILE_LIST,
        16,
        top + 48,
        456,
        bottom - 12 - (top + 48),
        &[("Ứng dụng", 300), ("Tiếng Việt", -1)],
        false,
    );
    edit(
        2,
        ID_APP_PICKER,
        16,
        bottom,
        180,
        BUTTON_H,
        ES_AUTOHSCROLL as u32,
        "Chọn hoặc gõ tên .exe",
    );
    button(2, ID_APP_LIST, "", 200, bottom, 30); // chevron: running apps
    dropdown(2, ID_PROFILE_STATE, &STATE_NAMES, 236, bottom, 108);
    set_choice(ID_PROFILE_STATE, 2); // usually added to turn Vietnamese off
    button(2, ID_PROFILE_ADD, "Thêm", 350, bottom, 58);
    button(2, ID_PROFILE_REMOVE, "Xoá", 414, bottom, 58);
    fill_profiles(None);
}

fn build_about() {
    let p = ABOUT_PAGE;
    add(p, "Static", "Gõ Nhanh", SS_LEFT, 16, 16, 456, 32, 0, 3, 0);
    hint(
        p,
        "Bộ gõ tiếng Việt nhanh, nhẹ, mã nguồn mở",
        16,
        52,
        456,
        18,
        0,
    );
    label(
        p,
        "Phát triển bởi Kha Phan và cộng đồng, với tất cả ❤️",
        16,
        84,
        456,
        0,
    );
    button(p, ID_WEBSITE, "Website", 16, 116, 100);
    button(p, ID_GITHUB, "GitHub", 124, 116, 100);
    button(p, ID_FEEDBACK, "Góp ý và báo lỗi", 232, 116, 140);
    hint(
        p,
        "Giấy phép GPL-3.0-or-later. Viết bằng Rust.",
        16,
        160,
        456,
        18,
        0,
    );

    add(
        p,
        "Static",
        "",
        SS_LEFT,
        16,
        196,
        456,
        1,
        ID_SEPARATOR,
        0,
        0,
    );
    header(p, "Cập nhật", 212);
    add(
        p,
        "Static",
        "",
        SS_LEFT,
        16,
        240,
        456,
        36,
        ID_UPDATE_STATUS,
        0,
        0,
    );
    set_status("");
    button(p, ID_UPDATE_BTN, "Kiểm tra cập nhật", 16, 284, 160);
}

fn load_values() {
    let Some(st) = app::try_with(|c| c.settings.clone()) else {
        return;
    };
    LOADING.set(true);
    set_checked(ID_ENABLED, st.enabled);
    set_choice(ID_METHOD, (st.method == Method::Vni) as usize);
    set_checked(ID_AUTO_W, st.auto_w_shortcut);
    set_checked(ID_BRACKET, st.bracket_shortcut);
    set_checked(ID_ENGLISH, st.english_auto_restore);
    set_checked(ID_MODERN, st.modern_tone);
    set_checked(ID_FOREIGN, st.allow_foreign_consonants);
    set_checked(ID_AUTO_CAP, st.auto_capitalize);
    set_text(ID_AUTO_CAP_EXCLUDED, &st.auto_capitalize_excluded_apps);
    set_checked(ID_RESTORE, st.restore_shortcut_enabled);
    set_checked(ID_PER_APP, st.per_app_mode);
    set_checked(ID_SOUND, st.sound_enabled);
    set_checked(ID_AUTO_START, st.auto_start);
    set_checked(ID_RUN_AS_ADMIN, st.run_as_admin);
    set_checked(ID_SECONDARY, st.secondary_toggle_enabled);
    set_checked(ID_REMOTE, st.remote_desktop_mode);
    set_checked(ID_DEBUG_LOG, st.debug_log);
    LOADING.set(false);
    update_dependent_controls();
}

fn show_page(index: usize) {
    PAGE_INDEX.set(index);
    let pages = PAGES.get();
    for (i, p) in pages.iter().enumerate() {
        unsafe { ShowWindow(*p, if i == index { SW_SHOW } else { SW_HIDE }) };
    }
    // Paint the whole page now: otherwise controls can stay blank until hovered
    unsafe {
        RedrawWindow(
            pages[index],
            std::ptr::null(),
            null_mut(),
            RDW_INVALIDATE | RDW_ERASE | RDW_ALLCHILDREN | RDW_UPDATENOW,
        );
        for i in 0..PAGE_TITLES.len() as i32 {
            InvalidateRect(GetDlgItem(MAIN.get(), ID_NAV + i), std::ptr::null(), 0);
        }
    }
}

// ---------------------------------------------------------------- scrolling page

// The Cài đặt page is taller than the window. It scrolls with the mouse wheel and follows the
// keyboard focus, without a visible scrollbar.

fn scroll_pos() -> i32 {
    SCROLL.get().0
}

/// Content height in pixels
fn set_scroll_height(height: i32) {
    SCROLL.set((0, height));
}

fn scroll_to(page: HWND, pos: i32) {
    let (old, height) = SCROLL.get();
    let mut rc: RECT = unsafe { std::mem::zeroed() };
    unsafe { GetClientRect(page, &mut rc) };
    let pos = pos.clamp(0, (height - rc.bottom).max(0));
    if pos == old {
        return;
    }
    SCROLL.set((pos, height));
    unsafe {
        ScrollWindowEx(
            page,
            0,
            old - pos,
            std::ptr::null(),
            std::ptr::null(),
            null_mut(),
            null_mut(),
            SW_SCROLLCHILDREN | SW_INVALIDATE | SW_ERASE,
        );
    }
}

/// Keyboard focus moved to `c`: scroll it into view when its page scrolls
fn ensure_visible(c: HWND) {
    let page = unsafe { GetParent(c) };
    // Notification codes overlap between control types (BN_UNHILITE = CBN_SETFOCUS): only act when
    // the control really has the focus
    // Only while moving with Tab: focus that moves for other reasons (window activation) must not
    // scroll the page
    let tabbing = unsafe { GetKeyState(VK_TAB as i32) } < 0;
    if page != PAGES.get()[0] || c.is_null() || unsafe { GetFocus() } != c || !tabbing {
        return;
    }
    unsafe {
        let mut r: RECT = std::mem::zeroed();
        GetWindowRect(c, &mut r);
        MapWindowPoints(null_mut(), page, &mut r as *mut RECT as *mut POINT, 2);
        let mut client: RECT = std::mem::zeroed();
        GetClientRect(page, &mut client);
        let pos = scroll_pos();
        let margin = s(8);
        if r.top < margin {
            scroll_to(page, pos + r.top - margin);
        } else if r.bottom > client.bottom - margin {
            scroll_to(page, pos + r.bottom - client.bottom + margin);
        }
    }
}

/// "Khởi động cùng Windows" / "Chạy với quyền admin" clicked: update the Task Scheduler task (may ask
/// UAC; the checkbox goes back if that fails), then restart with the new rights when admin changed
fn toggle_task_option(id: i32) {
    let (auto_start, admin) = (checked(ID_AUTO_START), checked(ID_RUN_AS_ADMIN));
    let main = MAIN.get();
    unsafe { EnableWindow(main, 0) }; // no second click while UAC / schtasks runs
    let outcome = autostart::sync(auto_start, admin, main);
    unsafe {
        EnableWindow(main, 1);
        SetForegroundWindow(main);
    }
    match outcome {
        Outcome::Done => commit(),
        Outcome::Cancelled => return set_checked(id, !checked(id)),
        Outcome::Failed => {
            set_checked(id, !checked(id));
            return message(
                "Không cập nhật được Task Scheduler. Bật Debug log rồi thử lại để xem lỗi.",
            );
        }
    }
    if id == ID_RUN_AS_ADMIN && admin != is_elevated() && !crate::restart_with_admin(admin) {
        message("Đã lưu. Mở lại GoNhanh để áp dụng quyền mới.");
    }
}

// ---------------------------------------------------------------- window procs

fn on_command(id: i32, code: u32) {
    if id == ID_SHORTCUT_LIST || id == ID_PROFILE_LIST {
        return table_changed(id, code);
    }
    let nav = id - ID_NAV;
    if (0..PAGE_TITLES.len() as i32).contains(&nav) {
        if code == BN_CLICKED || code == BN_DOUBLECLICKED {
            show_page(nav as usize);
        }
        return;
    }
    match id {
        ID_METHOD => {
            if code == CBN_SELCHANGE && !LOADING.get() {
                commit();
            }
        }
        ID_WEBSITE => crate::win::open_url("https://gonhanh.org"),
        ID_GITHUB => crate::win::open_url("https://github.com/khaphanspace/gonhanh.org"),
        ID_FEEDBACK => crate::win::open_url("https://github.com/khaphanspace/gonhanh.org/issues"),
        ID_ENABLED => {
            let on = checked(ID_ENABLED);
            app::try_with(|c| c.set_enabled(on));
        }
        ID_AUTO_START | ID_RUN_AS_ADMIN => {
            if code == BN_CLICKED && !LOADING.get() {
                toggle_task_option(id);
            }
        }
        ID_TOGGLE_HOTKEY | ID_RESTORE_HOTKEY | ID_SECONDARY_HOTKEY => {
            if code == BN_CLICKED {
                start_recording(ctl(id));
            }
        }
        ID_AUTO_CAP_EXCLUDED => {
            if code == EN_KILLFOCUS && !LOADING.get() {
                commit();
            }
        }
        ID_UPDATE_BTN => run_update_step(),
        ID_SHORTCUT_SAVE => save_shortcut_from_fields(),
        // Enter in a one-line text box (the dialog manager reports it as IDOK)
        IDOK => match unsafe { GetDlgCtrlID(GetFocus()) } {
            ID_SHORTCUT_KEY => unsafe {
                SetFocus(ctl(ID_SHORTCUT_VALUE));
            },
            ID_APP_PICKER => add_profile(),
            ID_AUTO_CAP_EXCLUDED => unsafe {
                SetFocus(MAIN.get()); // saves it, like leaving the box
            },
            _ => {}
        },
        ID_SHORTCUT_DELETE => {
            if let Some(row) = selected_row(ID_SHORTCUT_LIST) {
                app::try_with(|c| {
                    if row < c.settings.shortcuts.len() {
                        c.settings.shortcuts.remove(row);
                    }
                });
                save_shortcuts(None);
            }
        }
        ID_IMPORT => import_shortcuts(),
        ID_EXPORT => export_shortcuts(),
        ID_OPEN_LOG => {
            if log::file_path().exists() {
                crate::win::open_url(&log::file_path().display().to_string());
            } else {
                message("Chưa có log. Bật Debug log rồi gõ thử.");
            }
        }
        ID_APP_LIST => {
            if code == BN_CLICKED {
                pick_app();
            }
        }
        ID_PROFILE_STATE => {
            if code == CBN_SELCHANGE {
                if let Some(row) = selected_row(ID_PROFILE_LIST) {
                    let app_name = table::cell(ctl(ID_PROFILE_LIST), row, 0);
                    let state = state_from_combo();
                    app::try_with(|c| {
                        c.settings.per_app_profiles.insert(app_name.clone(), state);
                        c.settings.save();
                    });
                    fill_profiles(Some(&app_name));
                }
            }
        }
        ID_PROFILE_ADD => add_profile(),
        ID_PROFILE_REMOVE => {
            if let Some(row) = selected_row(ID_PROFILE_LIST) {
                let app_name = table::cell(ctl(ID_PROFILE_LIST), row, 0);
                app::try_with(|c| {
                    c.settings.per_app_profiles.remove(&app_name);
                    c.settings.save();
                });
                fill_profiles(None);
            }
        }
        IDCANCEL => close(), // Esc
        _ => {
            // Every other switch
            if code == BN_CLICKED && !LOADING.get() && is_switch(id) {
                commit();
            }
        }
    }
}

/// Table notifications: a shortcut checked/unchecked, or a row selected
fn table_changed(id: i32, code: u32) {
    let t = ctl(id);
    if id == ID_SHORTCUT_LIST && code == table::CHECKED {
        let row = table::last_changed(t);
        let on = table::checked(t, row);
        app::try_with(|c| {
            if let Some(item) = c.settings.shortcuts.get_mut(row) {
                item.enabled = on;
                c.settings.save_shortcuts();
                c.apply_settings();
            }
        });
    }
    if code != table::SELECTED {
        return;
    }
    let Some(row) = table::selected(t) else {
        return;
    };
    if id == ID_SHORTCUT_LIST {
        if let Some(item) = app::try_with(|c| c.settings.shortcuts.get(row).cloned()).flatten() {
            set_text(ID_SHORTCUT_KEY, &item.key);
            set_text(ID_SHORTCUT_VALUE, &item.value.replace('\n', "\r\n"));
        }
    } else if id == ID_PROFILE_LIST {
        let app_name = table::cell(t, row, 0);
        if let Some(state) =
            app::try_with(|c| c.settings.per_app_profiles.get(&app_name).copied()).flatten()
        {
            set_choice(ID_PROFILE_STATE, state_index(state));
        }
    }
}

/// Window/page background in the palette color; the window's left strip is the sidebar
fn erase(h: HWND, dc: HDC) -> LRESULT {
    let mut rc: RECT = unsafe { std::mem::zeroed() };
    unsafe {
        GetClientRect(h, &mut rc);
        FillRect(dc, &rc, dark::bg_brush());
        if h == MAIN.get() {
            rc.right = s(SIDEBAR_W);
            FillRect(dc, &rc, dark::sidebar_brush());
        }
    }
    1
}

/// Sidebar top: app icon, name, version
fn paint_sidebar(h: HWND) {
    let p = dark::palette();
    unsafe {
        let mut ps: PAINTSTRUCT = std::mem::zeroed();
        let dc = BeginPaint(h, &mut ps);
        let icon_size = s(64);
        let x = (s(SIDEBAR_W) - icon_size) / 2;
        DrawIconEx(
            dc,
            x,
            s(36),
            ICON.get(),
            icon_size,
            icon_size,
            0,
            null_mut(),
            DI_NORMAL,
        );
        SetBkMode(dc, TRANSPARENT as i32);
        let fonts = FONTS.get();
        let lines = [
            ("Gõ Nhanh".to_string(), fonts[3], p.text, 112, 30),
            (format!("v{}", updater::VERSION), fonts[2], p.hint, 144, 20),
        ];
        for (text, font, color, y, h) in lines {
            let old = SelectObject(dc, font as HGDIOBJ);
            SetTextColor(dc, color);
            let mut t: Vec<u16> = text.encode_utf16().collect();
            let mut r = RECT {
                left: 0,
                top: s(y),
                right: s(SIDEBAR_W),
                bottom: s(y + h),
            };
            DrawTextW(
                dc,
                t.as_mut_ptr(),
                t.len() as i32,
                &mut r,
                DT_SINGLELINE | DT_CENTER | DT_VCENTER,
            );
            SelectObject(dc, old);
        }
        EndPaint(h, &ps);
    }
}

/// WM_DRAWITEM: sidebar items (window) and switches (pages)
fn draw_item(dis: &DRAWITEMSTRUCT) {
    if dis.CtlType == ODT_MENU {
        return draw_menu_item(dis);
    }
    let id = dis.CtlID as i32;
    let focus = dis.itemState & ODS_FOCUS != 0 && dis.itemState & ODS_NOFOCUSRECT == 0;
    let index = id - ID_NAV;
    if (0..PAGE_TITLES.len() as i32).contains(&index) {
        let mut text: Vec<u16> = PAGE_TITLES[index as usize].encode_utf16().collect();
        unsafe { SelectObject(dis.hDC, FONTS.get()[0] as HGDIOBJ) };
        controls::paint_nav_item(
            dis.hDC,
            &dis.rcItem,
            &mut text,
            PAGE_INDEX.get() == index as usize,
            focus,
            DPI.get(),
        );
        return;
    }
    if !is_switch(id) {
        // Push button, drop-down or the chevron next to the app box
        let len = unsafe { GetWindowTextLengthW(dis.hwndItem) } as usize;
        let mut text = vec![0u16; len + 1];
        let n = unsafe { GetWindowTextW(dis.hwndItem, text.as_mut_ptr(), text.len() as i32) };
        text.truncate(n.max(0) as usize);
        unsafe { SelectObject(dis.hDC, FONTS.get()[0] as HGDIOBJ) };
        let chevron = id == ID_APP_LIST || DROPDOWNS.with(|d| d.borrow().contains_key(&id));
        controls::paint_button(
            dis.hDC,
            &dis.rcItem,
            &mut text,
            controls::ButtonLook {
                hot: HOT.with(|h| h.borrow().contains(&(dis.hwndItem as usize))),
                pressed: dis.itemState & ODS_SELECTED != 0,
                enabled: dis.itemState & ODS_DISABLED == 0,
                focus,
                chevron,
            },
            DPI.get(),
        );
        return;
    }
    let r = dis.rcItem;
    unsafe { FillRect(dis.hDC, &r, dark::bg_brush()) };
    let (w, h) = (s(40), s(20));
    let switch_rect = RECT {
        left: r.right - s(4) - w,
        top: (r.top + r.bottom - h) / 2,
        right: r.right - s(4),
        bottom: (r.top + r.bottom + h) / 2,
    };
    controls::paint_switch(
        dis.hDC,
        &switch_rect,
        checked(id),
        dis.itemState & ODS_DISABLED == 0,
    );
    if focus {
        let fr = RECT {
            left: switch_rect.left - s(2),
            top: switch_rect.top - s(2),
            right: switch_rect.right + s(2),
            bottom: switch_rect.bottom + s(2),
        };
        unsafe { DrawFocusRect(dis.hDC, &fr) };
    }
}

/// WM_COMMAND: clicking a switch's text works like clicking the switch; switches flip before
/// on_command reads them. Double clicks count as two clicks.
fn command(id: i32, code: u32) {
    let (mut id, mut code) = (id, code);
    if (LABEL..LABEL + 1000).contains(&id) || (SUB_LABEL..SUB_LABEL + 1000).contains(&id) {
        if code != STN_CLICKED && code != STN_DBLCLK {
            return;
        }
        id -= if id >= SUB_LABEL { SUB_LABEL } else { LABEL };
        if unsafe { IsWindowEnabled(ctl(id)) } == 0 {
            return;
        }
        code = BN_CLICKED;
    }
    if is_switch(id) {
        match code {
            BN_CLICKED | BN_DOUBLECLICKED => {
                set_checked(id, !checked(id));
                code = BN_CLICKED;
            }
            BN_SETFOCUS => return ensure_visible(ctl(id)),
            _ => return,
        }
    }
    // Owner-drawn buttons report a fast second click as a double click: count it as a click
    if code == BN_DOUBLECLICKED {
        code = BN_CLICKED;
    }
    match code {
        BN_SETFOCUS | EN_SETFOCUS => {
            ensure_visible(ctl(id));
            invalidate_frame(ctl(id));
        }
        EN_KILLFOCUS => invalidate_frame(ctl(id)),
        _ => {}
    }
    let items = DROPDOWNS.with(|d| d.borrow().get(&id).map(|(items, _)| items.clone()));
    if let (Some(items), BN_CLICKED) = (items, code) {
        if menu_just_closed(ctl(id)) {
            return;
        }
        if let Some(i) = pick_from_menu(ctl(id), &items, Some(choice(id))) {
            set_choice(id, i);
            on_command(id, CBN_SELCHANGE);
        }
        return;
    }
    on_command(id, code);
}

unsafe extern "system" fn page_proc(h: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_COMMAND | WM_NOTIFY | WM_CTLCOLORSTATIC | WM_CTLCOLORBTN | WM_CTLCOLOREDIT
        | WM_CTLCOLORLISTBOX => unsafe { SendMessageW(GetParent(h), msg, wparam, lparam) },
        WM_DRAWITEM => {
            draw_item(unsafe { &*(lparam as *const DRAWITEMSTRUCT) });
            1
        }
        // Click on empty page space: the text box or table loses focus
        WM_LBUTTONDOWN => unsafe {
            SetFocus(MAIN.get());
            0
        },
        WM_MOUSEWHEEL => {
            let delta = ((wparam >> 16) & 0xFFFF) as u16 as i16 as i32;
            scroll_to(h, scroll_pos() - delta * s(ROW_H) * 3 / 2 / 120);
            0
        }
        WM_ERASEBKGND => erase(h, wparam as HDC),
        WM_PAINT => {
            paint_frames(h);
            0
        }
        _ => unsafe { DefWindowProcW(h, msg, wparam, lparam) },
    }
}

unsafe extern "system" fn wnd_proc(h: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if MAIN.get() != h {
        return unsafe { DefWindowProcW(h, msg, wparam, lparam) }; // still being created / already closed
    }
    match msg {
        WM_COMMAND => command((wparam & 0xFFFF) as i32, ((wparam >> 16) & 0xFFFF) as u32),

        WM_DRAWITEM => {
            draw_item(unsafe { &*(lparam as *const DRAWITEMSTRUCT) });
            return 1;
        }
        WM_PAINT => paint_sidebar(h),
        // Click on the sidebar background: nothing stays focused (like a web page)
        WM_LBUTTONDOWN => unsafe {
            SetFocus(h);
        },
        WM_MEASUREITEM => unsafe {
            let mis = &mut *(lparam as *mut MEASUREITEMSTRUCT);
            if mis.CtlType == ODT_MENU {
                measure_menu_item(mis);
            }
            return 1;
        },
        WM_ACTIVATE => unsafe {
            let r = DefWindowProcW(h, msg, wparam, lparam);
            // Activated with nothing focused: start on the sidebar item, not a control down the page
            let focus = GetFocus();
            if wparam & 0xFFFF != 0 && (focus.is_null() || focus == h) {
                SetFocus(GetDlgItem(h, ID_NAV + PAGE_INDEX.get() as i32));
            }
            return r;
        },
        WM_CTLCOLORSTATIC | WM_CTLCOLORBTN | WM_CTLCOLOREDIT | WM_CTLCOLORLISTBOX
            if FIELDS.with(|f| f.borrow().contains_key(&(lparam as usize))) =>
        unsafe {
            // Text boxes (also disabled ones, which ask with WM_CTLCOLORSTATIC) on the field color
            let (dc, ctl) = (wparam as HDC, lparam as HWND);
            let p = dark::palette();
            SetTextColor(
                dc,
                if IsWindowEnabled(ctl) == 0 {
                    p.disabled
                } else {
                    p.text
                },
            );
            SetBkColor(dc, p.field);
            return dark::field_brush() as LRESULT;
        },
        WM_CTLCOLORSTATIC | WM_CTLCOLORBTN => unsafe {
            let (dc, ctl) = (wparam as HDC, lparam as HWND);
            if GetDlgCtrlID(ctl) == ID_SEPARATOR {
                return dark::separator_brush() as LRESULT;
            }
            let p = dark::palette();
            let gray = GRAY.with(|g| g.borrow().contains(&(ctl as usize)));
            SetBkMode(dc, TRANSPARENT as i32);
            SetBkColor(dc, p.bg);
            SetTextColor(
                dc,
                if IsWindowEnabled(ctl) == 0
                    || DIMMED.with(|d| d.borrow().contains(&(ctl as usize)))
                {
                    p.disabled
                } else if gray {
                    p.hint
                } else {
                    p.text
                },
            );
            return dark::bg_brush() as LRESULT;
        },
        // Dark mode: edit boxes and combo lists on the field color
        WM_CTLCOLOREDIT | WM_CTLCOLORLISTBOX if dark::is_dark() => unsafe {
            let dc = wparam as HDC;
            let p = dark::palette();
            SetTextColor(dc, p.text);
            SetBkColor(dc, p.field);
            return dark::field_brush() as LRESULT;
        },
        WM_ERASEBKGND => return erase(h, wparam as HDC),
        WM_APP_HOTKEY_SET => commit(),
        WM_APP_UPDATE_DONE => {
            on_update_done(*unsafe { Box::from_raw(lparam as *mut UpdateResult) })
        }
        WM_DPICHANGED => {
            // Rebuild at the new DPI where Windows suggests
            let r = unsafe { &*(lparam as *const RECT) };
            REOPEN_AT.set(Some((r.left, r.top)));
            unsafe { PostMessageW(h, WM_APP_RECREATE, 0, 0) };
        }
        WM_APP_RECREATE => {
            let page = PAGE_INDEX.get();
            close();
            create(page);
        }
        WM_CLOSE => close(),
        _ => return unsafe { DefWindowProcW(h, msg, wparam, lparam) },
    }
    0
}

fn register_classes() {
    thread_local!(static DONE: Cell<bool> = const { Cell::new(false) });
    if DONE.replace(true) {
        return;
    }
    unsafe {
        let instance = GetModuleHandleW(std::ptr::null());
        let (main, page) = (wide(MAIN_CLASS), wide(PAGE_CLASS));
        let mut wc: WNDCLASSEXW = std::mem::zeroed();
        wc.cbSize = std::mem::size_of::<WNDCLASSEXW>() as u32;
        wc.lpfnWndProc = Some(wnd_proc);
        wc.hInstance = instance;
        wc.hCursor = LoadCursorW(null_mut(), IDC_ARROW);
        wc.hbrBackground = null_mut(); // painted in WM_ERASEBKGND with the light/dark palette
        wc.hIcon = LoadIconW(instance, crate::win::make_int_resource(1));
        wc.lpszClassName = main.as_ptr();
        RegisterClassExW(&wc);
        wc.lpfnWndProc = Some(page_proc);
        wc.hIcon = null_mut();
        wc.lpszClassName = page.as_ptr();
        RegisterClassExW(&wc);
    }
}

fn create(page_index: usize) {
    register_classes();
    let Some(st) = app::try_with(|c| c.settings.clone()) else {
        return;
    };
    unsafe {
        let instance = GetModuleHandleW(std::ptr::null());
        let style = WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX | WS_CLIPCHILDREN;
        let ex_style = WS_EX_CONTROLPARENT;
        let (mut x, mut y) = REOPEN_AT.take().unwrap_or((CW_USEDEFAULT, CW_USEDEFAULT));
        let (class, title) = (wide(MAIN_CLASS), wide("Gõ Nhanh - Cài đặt"));
        let hwnd = CreateWindowExW(
            ex_style,
            class.as_ptr(),
            title.as_ptr(),
            style,
            x,
            y,
            100,
            100,
            null_mut(),
            null_mut(),
            instance,
            std::ptr::null(),
        );
        if hwnd.is_null() {
            return;
        }
        MAIN.set(hwnd);
        dark::begin(hwnd); // light or dark palette, dark title bar
                           // Everything is laid out at ZOOM % of the Windows scale (a bit bigger than the system
                           // default), less when the monitor is too short for it
        let raw_dpi = GetDpiForWindow(hwnd);
        let mut cursor = POINT { x: 0, y: 0 };
        GetCursorPos(&mut cursor);
        let mut mi: MONITORINFO = std::mem::zeroed();
        mi.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
        GetMonitorInfoW(MonitorFromPoint(cursor, MONITOR_DEFAULTTONEAREST), &mut mi);
        let work_h = mi.rcWork.bottom - mi.rcWork.top;
        let frame_h = scale(48, raw_dpi); // title bar and borders
        let full_h = scale(CLIENT_H, raw_dpi);
        let zoom = ZOOM
            .min(((work_h - frame_h) * 100 / full_h.max(1)) as u32)
            .max(100);
        DPI.set(raw_dpi * zoom / 100);

        let mut rc = RECT {
            left: 0,
            top: 0,
            right: s(SIDEBAR_W + CONTENT_W),
            bottom: s(CLIENT_H),
        };
        AdjustWindowRectExForDpi(&mut rc, style, 0, ex_style, raw_dpi);
        let (w, h) = (rc.right - rc.left, rc.bottom - rc.top);
        if x == CW_USEDEFAULT {
            // center on the monitor with the cursor
            let mut cursor = POINT { x: 0, y: 0 };
            GetCursorPos(&mut cursor);
            let mut mi: MONITORINFO = std::mem::zeroed();
            mi.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
            GetMonitorInfoW(MonitorFromPoint(cursor, MONITOR_DEFAULTTONEAREST), &mut mi);
            x = mi.rcWork.left + (mi.rcWork.right - mi.rcWork.left - w) / 2;
            y = mi.rcWork.top + (mi.rcWork.bottom - mi.rcWork.top - h) / 2;
        }
        SetWindowPos(hwnd, null_mut(), x, y, w, h, SWP_NOZORDER | SWP_NOACTIVATE);

        let dpi = DPI.get();
        FONTS.set([
            ui_font(dpi, false, 0),
            ui_font(dpi, true, 1),
            ui_font(dpi, false, -1),
            ui_font(dpi, true, 6),
        ]);
        ICON.set(LoadImageW(
            instance,
            crate::win::make_int_resource(1),
            IMAGE_ICON,
            s(64),
            s(64),
            LR_DEFAULTCOLOR,
        ) as HICON);

        // Pages fill the area right of the sidebar
        let page_class = wide(PAGE_CLASS);
        let empty = wide("");
        let mut pages = [null_mut(); 4];
        for p in pages.iter_mut() {
            *p = CreateWindowExW(
                WS_EX_CONTROLPARENT,
                page_class.as_ptr(),
                empty.as_ptr(),
                WS_CHILD | WS_CLIPCHILDREN,
                s(SIDEBAR_W),
                0,
                s(CONTENT_W),
                s(CLIENT_H),
                hwnd,
                null_mut(),
                instance,
                std::ptr::null(),
            );
        }
        PAGES.set(pages);

        // Sidebar items, at the bottom like the macOS app
        let button_class = wide("Button");
        let count = PAGE_TITLES.len() as i32;
        for (i, t) in PAGE_TITLES.iter().enumerate() {
            let text = wide(t);
            let item = CreateWindowExW(
                0,
                button_class.as_ptr(),
                text.as_ptr(),
                WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_OWNERDRAW as u32,
                0,
                s(CLIENT_H - 16 - (count - i as i32) * 40),
                s(SIDEBAR_W),
                s(40),
                hwnd,
                (ID_NAV + i as i32) as isize as HMENU,
                instance,
                std::ptr::null(),
            );
            SendMessageW(item, WM_SETFONT, FONTS.get()[0] as usize, 1);
        }
    }

    build_general(&st);
    build_shortcuts();
    build_advanced(&st);
    build_about();
    load_values();

    show_page(page_index);
    unsafe {
        ShowWindow(MAIN.get(), SW_SHOW);
        SetForegroundWindow(MAIN.get());
        // Start on the current sidebar item, so no control further down grabs focus and scrolls the page
        SetFocus(GetDlgItem(MAIN.get(), ID_NAV + page_index as i32));
    }
    scroll_to(PAGES.get()[0], 0);
}

// ---------------------------------------------------------------- public

/// Open (or bring to front) on `page`
fn show_at(page: usize) {
    if MAIN.get().is_null() {
        create(page);
    } else {
        unsafe {
            if IsIconic(MAIN.get()) != 0 {
                ShowWindow(MAIN.get(), SW_RESTORE);
            }
            SetForegroundWindow(MAIN.get());
        }
        show_page(page);
    }
}

/// Open (or bring to front). `check_updates`: go to Giới thiệu and check GitHub.
pub fn show(check_updates: bool) {
    if !check_updates {
        if MAIN.get().is_null() {
            create(0);
        } else {
            show_at(PAGE_INDEX.get());
        }
        return;
    }
    show_at(ABOUT_PAGE);
    if !MAIN.get().is_null() && UPDATE.with(|u| u.borrow().is_none()) {
        run_update_step();
    }
}

/// The Giới thiệu page
pub fn show_about() {
    show_at(ABOUT_PAGE);
}

/// For IsDialogMessage in the main loop; null when closed
pub fn hwnd() -> HWND {
    MAIN.get()
}

pub fn enabled_changed(on: bool) {
    if !MAIN.get().is_null() {
        set_checked(ID_ENABLED, on);
    }
}

/// Settings changed elsewhere (tray menu): refresh the controls
pub fn reload() {
    if !MAIN.get().is_null() {
        load_values();
    }
}

pub fn close() {
    let hwnd = MAIN.replace(null_mut());
    if hwnd.is_null() {
        return;
    }
    stop_recording();
    unsafe { DestroyWindow(hwnd) };
    dark::end();
    for f in FONTS.replace([null_mut(); 4]) {
        if !f.is_null() {
            unsafe { DeleteObject(f as HGDIOBJ) };
        }
    }
    let icon = ICON.replace(null_mut());
    if !icon.is_null() {
        unsafe { DestroyIcon(icon) };
    }
    PAGES.set([null_mut(); 4]);
    GRAY.with(|g| g.borrow_mut().clear());
    for set in [&DIMMED, &HOT] {
        set.with(|s| s.borrow_mut().clear());
    }
    FIELDS.with(|f| f.borrow_mut().clear());
    DROPDOWNS.with(|d| d.borrow_mut().clear());
    HOTKEYS.with(|h| h.borrow_mut().clear());
    SWITCHES.with(|s| s.borrow_mut().clear());
}

/// Windows switched light/dark: rebuild the open window in the new palette
pub fn theme_changed() {
    let hwnd = MAIN.get();
    if !hwnd.is_null() {
        unsafe { PostMessageW(hwnd, WM_APP_RECREATE, 0, 0) };
    }
}

/// DPI the window is laid out at (Windows scale × ZOOM), for the table
pub fn dpi() -> u32 {
    DPI.get()
}
