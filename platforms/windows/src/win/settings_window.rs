//! Settings window (macOS MainSettingsView + AdvancedSettingsView) in plain Win32.
//! Every change is saved and applied immediately, like macOS.

use std::cell::{Cell, RefCell};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::ptr::null_mut;

use windows_sys::core::BOOL;
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::*;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::Controls::Dialogs::*;
use windows_sys::Win32::UI::Controls::*;
use windows_sys::Win32::UI::HiDpi::{
    AdjustWindowRectExForDpi, GetDpiForWindow, GetSystemMetricsForDpi,
};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    EnableWindow, GetKeyState, SetFocus, VK_CONTROL, VK_LWIN, VK_MENU, VK_RWIN, VK_SHIFT,
};
use windows_sys::Win32::UI::Shell::{DefSubclassProc, SetWindowSubclass};
use windows_sys::Win32::UI::WindowsAndMessaging::*;

use super::{dark, foreground, from_wide, hook, scale, ui_font, wide};
use crate::app::{self, WM_APP_HOTKEY_SET, WM_APP_RECREATE, WM_APP_UPDATE_DONE};
use crate::hotkey::{Hotkey, MOD_ALT, MOD_CTRL, MOD_SHIFT, MOD_WIN};
use crate::settings::{self, Method, ShortcutItem};
use crate::{keys, log, updater};

// ---------------------------------------------------------------- ids & layout
const ID_TAB: i32 = 1000;
// General
const ID_ENABLED: i32 = 1001;
const ID_TOGGLE_HOTKEY: i32 = 1002;
const ID_TELEX: i32 = 1003;
const ID_VNI: i32 = 1004;
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
const ID_SEPARATOR: i32 = 1099;

const SS_LEFT: u32 = 0; // static control styles (not in windows-sys)
const SS_PATHELLIPSIS: u32 = 0x8000;
const CLIENT_W: i32 = 488; // page content spans x 16..472 → equal side margins
const CLIENT_H: i32 = 450;
const PAGE_TITLES: [&str; 3] = ["Cài đặt", "Gõ tắt", "Nâng cao"];
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
    static TAB: Cell<HWND> = const { Cell::new(null_mut()) };
    static PAGES: Cell<[HWND; 3]> = const { Cell::new([null_mut(); 3]) };
    static DPI: Cell<u32> = const { Cell::new(96) };
    static FONTS: Cell<[HFONT; 3]> = const { Cell::new([null_mut(); 3]) }; // normal, bold, small
    static TAB_INDEX: Cell<usize> = const { Cell::new(0) };
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

fn checked(id: i32) -> bool {
    send(ctl(id), BM_GETCHECK, 0, 0) == BST_CHECKED as isize
}
fn set_checked(id: i32, on: bool) {
    send(
        ctl(id),
        BM_SETCHECK,
        if on { BST_CHECKED } else { BST_UNCHECKED } as usize,
        0,
    );
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
fn enable(id: i32, on: bool) {
    unsafe { EnableWindow(ctl(id), on as BOOL) };
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
fn check(page: usize, id: i32, label: &str, x: i32, y: i32, w: i32) -> HWND {
    add(
        page,
        "Button",
        label,
        BS_AUTOCHECKBOX as u32 | WS_TABSTOP,
        x,
        y,
        w,
        22,
        id,
        0,
        0,
    )
}
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
fn button(page: usize, id: i32, t: &str, x: i32, y: i32, w: i32) -> HWND {
    add(
        page,
        "Button",
        t,
        BS_PUSHBUTTON as u32 | WS_TABSTOP,
        x,
        y,
        w,
        28,
        id,
        0,
        0,
    )
}
#[allow(clippy::too_many_arguments)]
fn edit(page: usize, id: i32, x: i32, y: i32, w: i32, h: i32, style: u32, cue: &str) -> HWND {
    let e = add(
        page,
        "Edit",
        "",
        style | WS_TABSTOP,
        x,
        y,
        w,
        h,
        id,
        0,
        WS_EX_CLIENTEDGE,
    );
    if !cue.is_empty() {
        let c = wide(cue);
        send(e, EM_SETCUEBANNER, 1, c.as_ptr() as isize);
    }
    e
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
    let w = wide("Nhấn phím...");
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
                        "{} ...",
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

fn recorder(page: usize, id: i32, value: Hotkey, x: i32, y: i32) {
    HOTKEYS.with(|h| h.borrow_mut().insert(id, value));
    let b = button(page, id, &value.to_string(), x, y, 160);
    unsafe { SetWindowSubclass(b, Some(recorder_proc), 1, 0) };
}

// ---------------------------------------------------------------- lists

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
    let lv = add(
        page,
        "SysListView32",
        "",
        LVS_REPORT | LVS_SINGLESEL | LVS_SHOWSELALWAYS | WS_TABSTOP | WS_BORDER,
        x,
        y,
        w,
        h,
        id,
        0,
        0,
    );
    let ex =
        LVS_EX_FULLROWSELECT | LVS_EX_DOUBLEBUFFER | if checkboxes { LVS_EX_CHECKBOXES } else { 0 };
    send(lv, LVM_SETEXTENDEDLISTVIEWSTYLE, 0, ex as isize);

    dark::style_list(lv);
    for (i, (title, width)) in columns.iter().enumerate() {
        let t = wide(title);
        let cx = if *width < 0 {
            // last column fills the rest
            s(w) - s(4)
                - send(lv, LVM_GETCOLUMNWIDTH, 0, 0) as i32
                - unsafe { GetSystemMetricsForDpi(SM_CXVSCROLL, DPI.get()) }
        } else {
            s(*width)
        };
        let mut col: LVCOLUMNW = unsafe { std::mem::zeroed() };
        col.mask = LVCF_TEXT | LVCF_WIDTH;
        col.pszText = t.as_ptr() as *mut u16;
        col.cx = cx;
        send(lv, LVM_INSERTCOLUMNW, i, &col as *const _ as isize);
    }
    lv
}

fn lv_set_text(lv: HWND, row: usize, sub: i32, t: &str) {
    let w = wide(t);
    let mut item: LVITEMW = unsafe { std::mem::zeroed() };
    item.iSubItem = sub;
    item.pszText = w.as_ptr() as *mut u16;
    send(lv, LVM_SETITEMTEXTW, row, &item as *const _ as isize);
}

fn lv_insert(lv: HWND, row: usize, t: &str) {
    let w = wide(t);
    let mut item: LVITEMW = unsafe { std::mem::zeroed() };
    item.mask = LVIF_TEXT;
    item.iItem = row as i32;
    item.pszText = w.as_ptr() as *mut u16;
    send(lv, LVM_INSERTITEMW, 0, &item as *const _ as isize);
}

fn lv_set_state(lv: HWND, row: usize, state: u32, mask: u32) {
    let mut item: LVITEMW = unsafe { std::mem::zeroed() };
    item.state = state;
    item.stateMask = mask;
    send(lv, LVM_SETITEMSTATE, row, &item as *const _ as isize);
}

fn lv_select(lv: HWND, row: usize) {
    lv_set_state(
        lv,
        row,
        LVIS_SELECTED | LVIS_FOCUSED,
        LVIS_SELECTED | LVIS_FOCUSED,
    );
    send(lv, LVM_ENSUREVISIBLE, row, 0);
}

fn lv_checked(lv: HWND, row: usize) -> bool {
    (send(lv, LVM_GETITEMSTATE, row, LVIS_STATEIMAGEMASK as isize) as u32 >> 12) == 2
}

fn lv_text(lv: HWND, row: usize) -> String {
    let mut buf = [0u16; 512];
    let mut item: LVITEMW = unsafe { std::mem::zeroed() };
    item.pszText = buf.as_mut_ptr();
    item.cchTextMax = 512;
    send(lv, LVM_GETITEMTEXTW, row, &mut item as *mut _ as isize);
    from_wide(&buf)
}

fn selected_row(id: i32) -> Option<usize> {
    let r = send(ctl(id), LVM_GETNEXTITEM, usize::MAX, LVNI_SELECTED as isize);
    (r >= 0).then_some(r as usize)
}

fn fill_shortcuts(select: Option<usize>) {
    let lv = ctl(ID_SHORTCUT_LIST);
    let Some(items) = app::try_with(|c| c.settings.shortcuts.clone()) else {
        return;
    };
    LOADING.set(true);
    send(lv, LVM_DELETEALLITEMS, 0, 0);
    for (i, item) in items.iter().enumerate() {
        lv_insert(lv, i, &item.key);
        lv_set_text(lv, i, 1, &item.value.replace('\n', " ⏎ "));
        lv_set_state(
            lv,
            i,
            (if item.enabled { 2 } else { 1 }) << 12,
            LVIS_STATEIMAGEMASK,
        );
    }
    if let Some(row) = select {
        lv_select(lv, row);
    }
    LOADING.set(false);
}

fn state_index(state: i32) -> usize {
    match state {
        1 => 1,
        -1 => 2,
        _ => 0,
    }
}

fn fill_profiles(select: Option<&str>) {
    let lv = ctl(ID_PROFILE_LIST);
    let Some(profiles) = app::try_with(|c| c.settings.per_app_profiles.clone()) else {
        return;
    };
    LOADING.set(true);
    send(lv, LVM_DELETEALLITEMS, 0, 0);
    for (i, (app, state)) in profiles.iter().enumerate() {
        lv_insert(lv, i, app);
        lv_set_text(lv, i, 1, STATE_NAMES[state_index(*state)]);
        if select == Some(app.as_str()) {
            lv_select(lv, i);
        }
    }
    LOADING.set(false);
}

// ---------------------------------------------------------------- apply

fn update_dependent_controls() {
    let telex = checked(ID_TELEX);
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
        st.method = if checked(ID_VNI) {
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
    match send(ctl(ID_PROFILE_STATE), CB_GETCURSEL, 0, 0) {
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

fn fill_app_picker() {
    let mut apps: BTreeSet<String> = BTreeSet::new();
    unsafe { EnumWindows(Some(collect_app), &mut apps as *mut _ as LPARAM) };
    let cb = ctl(ID_APP_PICKER);
    let typed = text(ID_APP_PICKER);
    send(cb, CB_RESETCONTENT, 0, 0);
    for a in &apps {
        let w = wide(a);
        send(cb, CB_ADDSTRING, 0, w.as_ptr() as isize);
    }
    set_text(ID_APP_PICKER, &typed);
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
        "Đang tải bản cập nhật..."
    } else {
        "Đang kiểm tra..."
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

// Grid: content x 16..472; right-hand controls at x 312 (160 wide); two columns at x 16 / 248.
// Related controls share a row instead of stacking.

fn build_general(st: &settings::Settings) {
    check(0, ID_ENABLED, "Bật bộ gõ tiếng Việt", 16, 12, 220);
    label(0, "Phím tắt", 248, 15, 60, 0);
    recorder(0, ID_TOGGLE_HOTKEY, st.toggle_shortcut, 312, 8);

    header(0, "Kiểu gõ", 50);
    add(
        0,
        "Button",
        "Telex",
        BS_AUTORADIOBUTTON as u32 | WS_GROUP | WS_TABSTOP,
        112,
        48,
        80,
        22,
        ID_TELEX,
        0,
        0,
    );
    add(
        0,
        "Button",
        "VNI",
        BS_AUTORADIOBUTTON as u32,
        200,
        48,
        80,
        22,
        ID_VNI,
        0,
        0,
    );
    check(0, ID_AUTO_W, "Gõ W thành Ư", 32, 76, 200);
    check(0, ID_BRACKET, "Gõ [ ] thành Ơ Ư", 248, 76, 224);
    check(
        0,
        ID_ENGLISH,
        "Tự khôi phục tiếng Anh (text → tẽt → text)",
        32,
        100,
        440,
    );

    header(0, "Chính tả", 138);
    check(0, ID_MODERN, "Dấu kiểu mới (oà, uý)", 16, 162, 224);
    check(0, ID_FOREIGN, "Phụ âm ngoại (z, w, j, f)", 248, 162, 224);
    check(0, ID_AUTO_CAP, "Tự viết hoa đầu câu", 16, 190, 170);
    label(0, "Trừ", 196, 193, 30, 0);
    edit(
        0,
        ID_AUTO_CAP_EXCLUDED,
        232,
        188,
        240,
        24,
        ES_AUTOHSCROLL as u32,
        "code.exe, WindowsTerminal.exe",
    );
    check(0, ID_RESTORE, "Hoàn tác dấu vừa gõ", 16, 222, 280);
    recorder(0, ID_RESTORE_HOTKEY, st.restore_shortcut, 312, 218);

    header(0, "Hệ thống", 260);
    check(
        0,
        ID_PER_APP,
        "Nhớ trạng thái bật/tắt theo từng ứng dụng",
        16,
        284,
        456,
    );
    check(0, ID_SOUND, "Âm thanh khi bật/tắt", 16, 308, 224);
    check(0, ID_AUTO_START, "Khởi động cùng Windows", 248, 308, 224);

    add(
        0,
        "Static",
        "",
        SS_LEFT,
        16,
        344,
        456,
        1,
        ID_SEPARATOR,
        0,
        0,
    );
    add(
        0,
        "Static",
        "",
        SS_LEFT,
        16,
        360,
        290,
        34,
        ID_UPDATE_STATUS,
        0,
        0,
    );
    set_status("");
    button(0, ID_UPDATE_BTN, "Kiểm tra cập nhật", 312, 356, 160);
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
        224,
        &[("Viết tắt", 120), ("Nội dung", -1)],
        true,
    );
    label(1, "Viết tắt", 16, 275, 64, 0);
    edit(
        1,
        ID_SHORTCUT_KEY,
        84,
        272,
        160,
        24,
        ES_AUTOHSCROLL as u32,
        "vd: hcm",
    );
    label(1, "Nội dung", 16, 307, 64, 0);
    edit(
        1,
        ID_SHORTCUT_VALUE,
        84,
        304,
        388,
        52,
        (ES_MULTILINE | ES_AUTOVSCROLL | ES_WANTRETURN) as u32 | WS_VSCROLL,
        "",
    );
    button(1, ID_SHORTCUT_SAVE, "Thêm / Cập nhật", 84, 366, 124);
    button(1, ID_SHORTCUT_DELETE, "Xoá", 214, 366, 70);
    button(1, ID_IMPORT, "Nhập từ file...", 316, 366, 76);
    button(1, ID_EXPORT, "Xuất ra file...", 396, 366, 76);
    fill_shortcuts(None);
}

fn build_advanced(st: &settings::Settings) {
    check(2, ID_SECONDARY, "Phím tắt bật/tắt phụ", 16, 12, 280);
    recorder(2, ID_SECONDARY_HOTKEY, st.secondary_toggle_shortcut, 312, 8);
    check(2, ID_REMOTE, "Remote desktop", 16, 44, 150);
    hint(
        2,
        "Gõ qua RustDesk, AnyDesk, TeamViewer, bàn phím ảo",
        170,
        47,
        302,
        18,
        0,
    );
    check(2, ID_DEBUG_LOG, "Debug log", 16, 76, 100);
    let path = add(
        2,
        "Static",
        &log::file_path().display().to_string(),
        SS_LEFT | SS_PATHELLIPSIS,
        120,
        79,
        262,
        18,
        0,
        2,
        0,
    );
    GRAY.with(|g| g.borrow_mut().insert(path as usize));
    button(2, ID_OPEN_LOG, "Mở log", 392, 72, 80);

    header(2, "Tuỳ chỉnh theo ứng dụng", 116);
    hint(2, "Luôn bật hoặc luôn tắt tiếng Việt trong từng ứng dụng. “Tự động” = theo trạng thái đã nhớ.", 16, 140, 456, 18, 0);
    list_view(
        2,
        ID_PROFILE_LIST,
        16,
        164,
        456,
        186,
        &[("Ứng dụng", 300), ("Tiếng Việt", -1)],
        false,
    );
    let picker = add(
        2,
        "ComboBox",
        "",
        (CBS_DROPDOWN | CBS_AUTOHSCROLL | CBS_SORT) as u32 | WS_VSCROLL | WS_TABSTOP,
        16,
        362,
        210,
        240,
        ID_APP_PICKER,
        0,
        0,
    );
    let cue = wide("Chọn hoặc gõ tên .exe");
    send(picker, CB_SETCUEBANNER, 0, cue.as_ptr() as isize);
    let state = add(
        2,
        "ComboBox",
        "",
        CBS_DROPDOWNLIST as u32 | WS_TABSTOP,
        232,
        362,
        110,
        200,
        ID_PROFILE_STATE,
        0,
        0,
    );
    for n in STATE_NAMES {
        let w = wide(n);
        send(state, CB_ADDSTRING, 0, w.as_ptr() as isize);
    }
    send(state, CB_SETCURSEL, 2, 0); // usually added to turn Vietnamese off
    button(2, ID_PROFILE_ADD, "Thêm", 348, 360, 60);
    button(2, ID_PROFILE_REMOVE, "Xoá", 412, 360, 60);
    fill_profiles(None);
}

fn load_values() {
    let Some(st) = app::try_with(|c| c.settings.clone()) else {
        return;
    };
    LOADING.set(true);
    set_checked(ID_ENABLED, st.enabled);
    set_checked(ID_TELEX, st.method == Method::Telex);
    set_checked(ID_VNI, st.method == Method::Vni);
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
    set_checked(ID_SECONDARY, st.secondary_toggle_enabled);
    set_checked(ID_REMOTE, st.remote_desktop_mode);
    set_checked(ID_DEBUG_LOG, st.debug_log);
    LOADING.set(false);
    update_dependent_controls();
}

fn show_page(index: usize) {
    TAB_INDEX.set(index);
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
        )
    };
}

// ---------------------------------------------------------------- window procs

fn on_command(id: i32, code: u32) {
    match id {
        ID_ENABLED => {
            let on = checked(ID_ENABLED);
            app::try_with(|c| c.set_enabled(on));
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
                super::open_url(&log::file_path().display().to_string());
            } else {
                message("Chưa có log. Bật Debug log rồi gõ thử.");
            }
        }
        ID_APP_PICKER => {
            if code == CBN_DROPDOWN {
                fill_app_picker();
            }
        }
        ID_PROFILE_STATE => {
            if code == CBN_SELCHANGE {
                if let Some(row) = selected_row(ID_PROFILE_LIST) {
                    let app_name = lv_text(ctl(ID_PROFILE_LIST), row);
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
                let app_name = lv_text(ctl(ID_PROFILE_LIST), row);
                app::try_with(|c| {
                    c.settings.per_app_profiles.remove(&app_name);
                    c.settings.save();
                });
                fill_profiles(None);
            }
        }
        IDCANCEL => close(), // Esc
        _ => {
            // Every other checkbox / radio
            if code == BN_CLICKED && !LOADING.get() {
                commit();
            }
        }
    }
}

fn on_notify(nm: &NMHDR) {
    let id = nm.idFrom as i32;
    if id == ID_TAB && nm.code == TCN_SELCHANGE {
        show_page(send(TAB.get(), TCM_GETCURSEL, 0, 0).max(0) as usize);
        return;
    }
    if nm.code != LVN_ITEMCHANGED || LOADING.get() {
        return;
    }
    let lv = unsafe { &*(nm as *const NMHDR as *const NMLISTVIEW) };
    if lv.uChanged & LVIF_STATE == 0 || lv.iItem < 0 {
        return;
    }
    let row = lv.iItem as usize;
    let selected = lv.uNewState & LVIS_SELECTED != 0 && lv.uOldState & LVIS_SELECTED == 0;
    if id == ID_SHORTCUT_LIST {
        if (lv.uNewState ^ lv.uOldState) & LVIS_STATEIMAGEMASK != 0 {
            // checkbox toggled
            let on = lv_checked(nm.hwndFrom, row);
            app::try_with(|c| {
                if let Some(item) = c.settings.shortcuts.get_mut(row) {
                    item.enabled = on;
                    c.settings.save_shortcuts();
                    c.apply_settings();
                }
            });
        }
        if selected {
            if let Some(item) = app::try_with(|c| c.settings.shortcuts.get(row).cloned()).flatten()
            {
                set_text(ID_SHORTCUT_KEY, &item.key);
                set_text(ID_SHORTCUT_VALUE, &item.value.replace('\n', "\r\n"));
            }
        }
    } else if id == ID_PROFILE_LIST && selected {
        let app_name = lv_text(nm.hwndFrom, row);
        if let Some(state) =
            app::try_with(|c| c.settings.per_app_profiles.get(&app_name).copied()).flatten()
        {
            send(ctl(ID_PROFILE_STATE), CB_SETCURSEL, state_index(state), 0);
        }
    }
}

/// Window/page background in the palette color
fn erase(h: HWND, dc: HDC) -> LRESULT {
    let mut rc = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    unsafe {
        GetClientRect(h, &mut rc);
        FillRect(dc, &rc, dark::bg_brush());
    }
    1
}

unsafe extern "system" fn page_proc(h: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_COMMAND | WM_NOTIFY | WM_CTLCOLORSTATIC | WM_CTLCOLORBTN | WM_CTLCOLOREDIT
        | WM_CTLCOLORLISTBOX => unsafe { SendMessageW(GetParent(h), msg, wparam, lparam) },
        WM_ERASEBKGND => erase(h, wparam as HDC),
        _ => unsafe { DefWindowProcW(h, msg, wparam, lparam) },
    }
}

unsafe extern "system" fn wnd_proc(h: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if MAIN.get() != h {
        return unsafe { DefWindowProcW(h, msg, wparam, lparam) }; // still being created / already closed
    }
    match msg {
        WM_COMMAND => on_command((wparam & 0xFFFF) as i32, ((wparam >> 16) & 0xFFFF) as u32),
        WM_NOTIFY => on_notify(unsafe { &*(lparam as *const NMHDR) }),
        WM_CTLCOLORSTATIC | WM_CTLCOLORBTN => unsafe {
            let (dc, ctl) = (wparam as HDC, lparam as HWND);
            if GetDlgCtrlID(ctl) == ID_SEPARATOR {
                return dark::separator_brush() as LRESULT;
            }
            let p = dark::palette();
            let gray = GRAY.with(|g| g.borrow().contains(&(ctl as usize)));
            SetBkMode(dc, TRANSPARENT as i32);
            SetBkColor(dc, p.bg);
            SetTextColor(dc, if gray { p.hint } else { p.text });
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
            let tab = TAB_INDEX.get();
            close();
            create(tab);
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
        wc.hIcon = LoadIconW(instance, super::make_int_resource(1));
        wc.lpszClassName = main.as_ptr();
        RegisterClassExW(&wc);
        wc.lpfnWndProc = Some(page_proc);
        wc.hIcon = null_mut();
        wc.lpszClassName = page.as_ptr();
        RegisterClassExW(&wc);
    }
}

fn create(tab_index: usize) {
    register_classes();
    let Some(st) = app::try_with(|c| c.settings.clone()) else {
        return;
    };
    unsafe {
        let instance = GetModuleHandleW(std::ptr::null());
        let style = WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX | WS_CLIPCHILDREN;
        let ex_style = WS_EX_CONTROLPARENT;
        let (mut x, mut y) = REOPEN_AT.take().unwrap_or((CW_USEDEFAULT, CW_USEDEFAULT));
        let (class, title) = (wide(MAIN_CLASS), wide("Cài đặt GoNhanh"));
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
        DPI.set(GetDpiForWindow(hwnd));

        let mut rc = RECT {
            left: 0,
            top: 0,
            right: s(CLIENT_W),
            bottom: s(CLIENT_H),
        };
        AdjustWindowRectExForDpi(&mut rc, style, 0, ex_style, DPI.get());
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
        ]);

        // Tab strip only: full width, equal tabs, edge to edge. Pages sit below it on the window
        // background (no inset tab pane = no box-in-a-box look).
        let mut client = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        GetClientRect(hwnd, &mut client);
        let tab_class = wide("SysTabControl32");
        let empty = wide("");
        let tab = CreateWindowExW(
            0,
            tab_class.as_ptr(),
            empty.as_ptr(),
            WS_CHILD | WS_VISIBLE | WS_CLIPSIBLINGS | WS_TABSTOP | TCS_FIXEDWIDTH,
            0,
            0,
            client.right,
            s(36),
            hwnd,
            ID_TAB as isize as HMENU,
            instance,
            std::ptr::null(),
        );
        TAB.set(tab);
        dark::style_tab(tab);
        SendMessageW(tab, WM_SETFONT, FONTS.get()[0] as usize, 1);
        // A few px less than a third: the control adds its own edge, and overflow shows scroll arrows
        let item_w = (client.right - s(6)) / 3;
        SendMessageW(
            tab,
            TCM_SETITEMSIZE,
            0,
            ((s(30) as u32) << 16 | item_w as u32 & 0xFFFF) as isize,
        );
        for (i, t) in PAGE_TITLES.iter().enumerate() {
            let w = wide(t);
            let mut item: TCITEMW = std::mem::zeroed();
            item.mask = TCIF_TEXT;
            item.pszText = w.as_ptr() as *mut u16;
            SendMessageW(tab, TCM_INSERTITEMW, i, &item as *const _ as isize);
        }
        let mut strip = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        SendMessageW(tab, TCM_GETITEMRECT, 0, &mut strip as *mut _ as isize);
        let strip_h = strip.bottom + s(2); // tabs + the line under them
        SetWindowPos(
            tab,
            null_mut(),
            0,
            0,
            client.right,
            strip_h,
            SWP_NOZORDER | SWP_NOMOVE,
        );

        let page_class = wide(PAGE_CLASS);
        let mut pages = [null_mut(); 3];
        for p in pages.iter_mut() {
            *p = CreateWindowExW(
                WS_EX_CONTROLPARENT,
                page_class.as_ptr(),
                empty.as_ptr(),
                WS_CHILD,
                0,
                strip_h,
                client.right,
                client.bottom - strip_h,
                hwnd,
                null_mut(),
                instance,
                std::ptr::null(),
            );
            SetWindowPos(*p, HWND_TOP, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
        }
        PAGES.set(pages);
    }

    build_general(&st);
    build_shortcuts();
    build_advanced(&st);
    load_values();

    send(TAB.get(), TCM_SETCURSEL, tab_index, 0);
    show_page(tab_index);
    unsafe {
        ShowWindow(MAIN.get(), SW_SHOW);
        SetForegroundWindow(MAIN.get());
    }
}

// ---------------------------------------------------------------- public

/// Open (or bring to front). `check_updates`: jump to the first tab and check GitHub.
pub fn show(check_updates: bool) {
    if MAIN.get().is_null() {
        create(0);
    } else {
        unsafe {
            if IsIconic(MAIN.get()) != 0 {
                ShowWindow(MAIN.get(), SW_RESTORE);
            }
            SetForegroundWindow(MAIN.get());
        }
    }
    if check_updates && !MAIN.get().is_null() {
        send(TAB.get(), TCM_SETCURSEL, 0, 0);
        show_page(0);
        if UPDATE.with(|u| u.borrow().is_none()) {
            run_update_step();
        }
    }
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
    for f in FONTS.replace([null_mut(); 3]) {
        if !f.is_null() {
            unsafe { DeleteObject(f as HGDIOBJ) };
        }
    }
    PAGES.set([null_mut(); 3]);
    GRAY.with(|g| g.borrow_mut().clear());
    HOTKEYS.with(|h| h.borrow_mut().clear());
}

/// Windows switched light/dark: rebuild the open window in the new palette
pub fn theme_changed() {
    let hwnd = MAIN.get();
    if !hwnd.is_null() {
        unsafe { PostMessageW(hwnd, WM_APP_RECREATE, 0, 0) };
    }
}
