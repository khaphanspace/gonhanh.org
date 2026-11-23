//! Settings window colors: system light look, or a Windows 11 dark look when Windows apps are dark.
//! Standard controls get the system dark themes; checkboxes, radios and the tab strip are painted here
//! because Windows ignores text colors for themed ones.

use std::cell::Cell;
use std::ptr::null_mut;

use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, SIZE, WPARAM};
use windows_sys::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_USE_IMMERSIVE_DARK_MODE};
use windows_sys::Win32::Graphics::Gdi::*;
use windows_sys::Win32::UI::Controls::*;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetFocus, IsWindowEnabled};
use windows_sys::Win32::UI::Shell::{DefSubclassProc, SetWindowSubclass};
use windows_sys::Win32::UI::WindowsAndMessaging::*;

use super::{scale, theme, wide};

/// COLORREF values (0x00BBGGRR)
#[derive(Clone, Copy)]
pub struct Palette {
    pub bg: u32,
    pub field: u32,
    pub text: u32,
    pub hint: u32,
    pub disabled: u32,
    pub separator: u32,
    pub tab_strip: u32,
}

thread_local! {
    static DARK: Cell<bool> = const { Cell::new(false) };
    static BRUSHES: Cell<[HBRUSH; 3]> = const { Cell::new([null_mut(); 3]) }; // bg, field, separator
}

const fn gray(v: u32) -> u32 {
    v | v << 8 | v << 16
}

pub fn is_dark() -> bool {
    DARK.get()
}

pub fn palette() -> Palette {
    if is_dark() {
        Palette {
            bg: gray(0x20),
            field: gray(0x2B),
            text: gray(0xFF),
            hint: gray(0xA0),
            disabled: gray(0x6E),
            separator: gray(0x3A),
            tab_strip: gray(0x1C),
        }
    } else {
        let window = unsafe { GetSysColor(COLOR_WINDOW) };
        Palette {
            bg: window,
            field: window,
            text: unsafe { GetSysColor(COLOR_WINDOWTEXT) },
            hint: 0x80_72_6B, // #6B7280
            disabled: unsafe { GetSysColor(COLOR_GRAYTEXT) },
            separator: gray(0xE5),
            tab_strip: window,
        }
    }
}

pub fn bg_brush() -> HBRUSH {
    BRUSHES.get()[0]
}
pub fn field_brush() -> HBRUSH {
    BRUSHES.get()[1]
}
pub fn separator_brush() -> HBRUSH {
    BRUSHES.get()[2]
}

/// Window created: pick light/dark from Windows, make brushes, dark title bar
pub fn begin(hwnd: HWND) {
    DARK.set(theme::apps_dark());
    let p = palette();
    BRUSHES.set(unsafe { [CreateSolidBrush(p.bg), CreateSolidBrush(p.field), CreateSolidBrush(p.separator)] });
    if is_dark() {
        theme::allow_for_window(hwnd);
        let on: i32 = 1;
        unsafe { DwmSetWindowAttribute(hwnd, DWMWA_USE_IMMERSIVE_DARK_MODE as u32, &on as *const _ as _, 4) };
    }
}

pub fn end() {
    for b in BRUSHES.replace([null_mut(); 3]) {
        if !b.is_null() {
            unsafe { DeleteObject(b as HGDIOBJ) };
        }
    }
}

fn set_theme(h: HWND, name: &str) {
    let n = wide(name);
    unsafe { SetWindowTheme(h, n.as_ptr(), std::ptr::null()) };
}

/// Fill `rect` of `hdc` with a solid color
fn fill(hdc: HDC, rect: &RECT, color: u32) {
    unsafe {
        let brush = CreateSolidBrush(color);
        FillRect(hdc, rect, brush);
        DeleteObject(brush as HGDIOBJ);
    }
}

fn window_text(h: HWND) -> Vec<u16> {
    let len = unsafe { GetWindowTextLengthW(h) } as usize;
    let mut buf = vec![0u16; len + 1];
    let n = unsafe { GetWindowTextW(h, buf.as_mut_ptr(), buf.len() as i32) } as usize;
    buf.truncate(n);
    buf
}

// ---------------------------------------------------------------- standard controls

/// Theme a freshly created control (no-op in light mode)
pub fn style_control(h: HWND, class: &str, style: u32) {
    if !is_dark() {
        return;
    }
    match class {
        "Button" => {
            set_theme(h, "DarkMode_Explorer");
            let kind = style & BS_TYPEMASK as u32;
            if kind == BS_AUTOCHECKBOX as u32 || kind == BS_AUTORADIOBUTTON as u32 {
                unsafe { SetWindowSubclass(h, Some(check_proc), 2, 0) };
            }
        }
        // DarkMode_CFD darks the field but not scrollbars; multiline edits need DarkMode_Explorer for those
        "Edit" if style & ES_MULTILINE as u32 != 0 => unsafe {
            set_theme(h, "DarkMode_Explorer");
            // that theme has no edit border: the 3D edge turns bright white, a flat 1px border stays dark
            SetWindowLongW(h, GWL_EXSTYLE, GetWindowLongW(h, GWL_EXSTYLE) & !(WS_EX_CLIENTEDGE as i32));
            SetWindowLongW(h, GWL_STYLE, GetWindowLongW(h, GWL_STYLE) | WS_BORDER as i32);
            SetWindowPos(h, null_mut(), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED);
        },
        "Edit" | "ComboBox" => set_theme(h, "DarkMode_CFD"),
        _ => {}
    }
}

/// List view colors, dark scrollbars and header
pub fn style_list(lv: HWND) {
    if !is_dark() {
        set_theme(lv, "Explorer");
        return;
    }
    let p = palette();
    set_theme(lv, "DarkMode_Explorer");
    unsafe {
        SendMessageW(lv, LVM_SETBKCOLOR, 0, p.field as isize);
        SendMessageW(lv, LVM_SETTEXTBKCOLOR, 0, p.field as isize);
        SendMessageW(lv, LVM_SETTEXTCOLOR, 0, p.text as isize);
        let header = SendMessageW(lv, LVM_GETHEADER, 0, 0) as HWND;
        set_theme(header, "DarkMode_ItemsView");
        SetWindowSubclass(lv, Some(list_proc), 3, 0); // header text color
    }
}

/// Header notifications go to the list view itself: light header text there
unsafe extern "system" fn list_proc(h: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM, _: usize, _: usize) -> LRESULT {
    if msg == WM_NOTIFY {
        let nm = unsafe { &*(lparam as *const NMHDR) };
        if nm.code == NM_CUSTOMDRAW {
            let cd = unsafe { &*(lparam as *const NMCUSTOMDRAW) };
            match cd.dwDrawStage {
                CDDS_PREPAINT => return CDRF_NOTIFYITEMDRAW as LRESULT,
                CDDS_ITEMPREPAINT => {
                    unsafe { SetTextColor(cd.hdc, palette().text) };
                    return 0; // CDRF_DEFAULT
                }
                _ => {}
            }
        }
    }
    unsafe { DefSubclassProc(h, msg, wparam, lparam) }
}

// ---------------------------------------------------------------- checkbox / radio

unsafe extern "system" fn check_proc(h: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM, _: usize, _: usize) -> LRESULT {
    match msg {
        WM_PAINT => {
            paint_check(h);
            0
        }
        WM_ERASEBKGND => 1,
        // State changes repaint through the default handler; redraw fully so our text stays
        BM_SETCHECK | BM_SETSTATE | WM_ENABLE | WM_SETTEXT | WM_SETFOCUS | WM_KILLFOCUS | WM_UPDATEUISTATE => {
            let r = unsafe { DefSubclassProc(h, msg, wparam, lparam) };
            unsafe { InvalidateRect(h, std::ptr::null(), 0) };
            r
        }
        _ => unsafe { DefSubclassProc(h, msg, wparam, lparam) },
    }
}

/// Box from the dark "Button" theme, text in the palette color
fn paint_check(h: HWND) {
    let p = palette();
    unsafe {
        let mut ps: PAINTSTRUCT = std::mem::zeroed();
        let hdc = BeginPaint(h, &mut ps);
        let mut rc = RECT { left: 0, top: 0, right: 0, bottom: 0 };
        GetClientRect(h, &mut rc);
        fill(hdc, &rc, p.bg);

        let style = GetWindowLongW(h, GWL_STYLE) as u32 & BS_TYPEMASK as u32;
        let radio = style == BS_AUTORADIOBUTTON as u32;
        let state = SendMessageW(h, BM_GETSTATE, 0, 0) as u32;
        let enabled = IsWindowEnabled(h) != 0;
        let base = if state & BST_CHECKED != 0 { 5 } else { 1 }; // *_UNCHECKEDNORMAL = 1, *_CHECKEDNORMAL = 5
        let variant = if !enabled {
            3
        } else if state & BST_PUSHED != 0 {
            2
        } else if state & BST_HOT != 0 {
            1
        } else {
            0
        };
        let part = if radio { BP_RADIOBUTTON } else { BP_CHECKBOX };

        let class = wide("Button");
        let theme = OpenThemeData(h, class.as_ptr());
        let mut size = SIZE { cx: 13, cy: 13 };
        GetThemePartSize(theme, hdc, part, base + variant, std::ptr::null(), TS_DRAW, &mut size);
        let top = (rc.bottom - size.cy) / 2;
        let bx = RECT { left: 0, top, right: size.cx, bottom: top + size.cy };
        DrawThemeBackground(theme, hdc, part, base + variant, &bx, std::ptr::null());
        CloseThemeData(theme);

        let font = SendMessageW(h, WM_GETFONT, 0, 0) as HGDIOBJ;
        let old = SelectObject(hdc, font);
        SetBkMode(hdc, TRANSPARENT as i32);
        SetTextColor(hdc, if enabled { p.text } else { p.disabled });
        let mut text = window_text(h);
        let mut tr = RECT { left: size.cx + scale(6, GetDpiForWindowSafe(h)), ..rc };
        DrawTextW(hdc, text.as_mut_ptr(), text.len() as i32, &mut tr, DT_SINGLELINE | DT_VCENTER | DT_LEFT | DT_END_ELLIPSIS);
        // Focus rectangle only while navigating with the keyboard, like the standard control
        let hide_focus = SendMessageW(h, WM_QUERYUISTATE, 0, 0) as u32 & UISF_HIDEFOCUS != 0;
        if GetFocus() == h && !hide_focus {
            let mut fr = tr;
            DrawTextW(hdc, text.as_mut_ptr(), text.len() as i32, &mut fr, DT_SINGLELINE | DT_CALCRECT);
            let fr = RECT { left: fr.left - 1, top: (rc.bottom - (fr.bottom - fr.top)) / 2 - 1, right: fr.right + 1, bottom: (rc.bottom + (fr.bottom - fr.top)) / 2 + 1 };
            DrawFocusRect(hdc, &fr);
        }
        SelectObject(hdc, old);
        EndPaint(h, &ps);
    }
}

#[allow(non_snake_case)]
fn GetDpiForWindowSafe(h: HWND) -> u32 {
    match unsafe { windows_sys::Win32::UI::HiDpi::GetDpiForWindow(h) } {
        0 => 96,
        d => d,
    }
}

// ---------------------------------------------------------------- tab strip

pub fn style_tab(tab: HWND) {
    if is_dark() {
        unsafe { SetWindowSubclass(tab, Some(tab_proc), 4, 0) };
    }
}

unsafe extern "system" fn tab_proc(h: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM, _: usize, _: usize) -> LRESULT {
    match msg {
        WM_PAINT => {
            paint_tab(h);
            0
        }
        WM_ERASEBKGND => 1,
        _ => unsafe { DefSubclassProc(h, msg, wparam, lparam) },
    }
}

/// Same shape as the system tab control, in dark tones: the selected tab is joined to the page
/// (page color, thin border, no line under it); the others sit on a slightly darker strip.
fn paint_tab(h: HWND) {
    let p = palette();
    unsafe {
        let mut ps: PAINTSTRUCT = std::mem::zeroed();
        let hdc = BeginPaint(h, &mut ps);
        let mut rc = RECT { left: 0, top: 0, right: 0, bottom: 0 };
        GetClientRect(h, &mut rc);
        fill(hdc, &rc, p.tab_strip);
        let font = SendMessageW(h, WM_GETFONT, 0, 0) as HGDIOBJ;
        let old = SelectObject(hdc, font);
        SetBkMode(hdc, TRANSPARENT as i32);
        let selected = SendMessageW(h, TCM_GETCURSEL, 0, 0);
        let mut gap = (0, 0); // bottom line is open under the selected tab
        for i in 0..SendMessageW(h, TCM_GETITEMCOUNT, 0, 0) {
            let mut r = RECT { left: 0, top: 0, right: 0, bottom: 0 };
            SendMessageW(h, TCM_GETITEMRECT, i as usize, &mut r as *mut _ as isize);
            let mut buf = [0u16; 64];
            let mut item: TCITEMW = std::mem::zeroed();
            item.mask = TCIF_TEXT;
            item.pszText = buf.as_mut_ptr();
            item.cchTextMax = 64;
            SendMessageW(h, TCM_GETITEMW, i as usize, &mut item as *mut _ as isize);
            let len = buf.iter().position(|&c| c == 0).unwrap_or(0);
            if i == selected {
                let tab = RECT { bottom: rc.bottom, ..r };
                fill(hdc, &tab, p.bg);
                for edge in [
                    RECT { right: tab.left + 1, ..tab },
                    RECT { left: tab.right - 1, ..tab },
                    RECT { bottom: tab.top + 1, ..tab },
                ] {
                    fill(hdc, &edge, p.separator);
                }
                gap = (tab.left + 1, tab.right - 1);
            }
            SetTextColor(hdc, if i == selected { p.text } else { p.hint });
            DrawTextW(hdc, buf.as_mut_ptr(), len as i32, &mut r, DT_SINGLELINE | DT_VCENTER | DT_CENTER);
        }
        for line in [RECT { left: 0, top: rc.bottom - 1, right: gap.0, bottom: rc.bottom }, RECT { left: gap.1, top: rc.bottom - 1, right: rc.right, bottom: rc.bottom }] {
            fill(hdc, &line, p.separator);
        }
        SelectObject(hdc, old);
        EndPaint(h, &ps);
    }
}
