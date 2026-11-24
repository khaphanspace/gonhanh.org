//! A small table drawn entirely by GoNhanh (instead of the Windows list view): fixed columns with
//! thin dividers, a gray header, rows with a hover and a selected highlight, optional checkboxes.
//! Mouse, wheel, arrow keys and Space work. The parent gets WM_COMMAND with the table's id and
//! SELECTED / CHECKED, or EN_SETFOCUS / EN_KILLFOCUS when the focus moves.

use std::cell::RefCell;
use std::collections::HashMap;
use std::ptr::null_mut;

use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::*;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::Controls::WM_MOUSELEAVE;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    SetFocus, TrackMouseEvent, TME_LEAVE, TRACKMOUSEEVENT, VK_DOWN, VK_SPACE, VK_UP,
};
use windows_sys::Win32::UI::WindowsAndMessaging::*;

use super::{controls, dark};
use crate::win::{scale, wide};

pub const CLASS: &str = "GoNhanhTable";
/// WM_COMMAND codes sent to the parent
pub const SELECTED: u32 = 0x7001;
pub const CHECKED: u32 = 0x7002;

const HEADER_H: i32 = 34;
const ROW_H: i32 = 36;
const PAD: i32 = 12;
const CHECK: i32 = 16;

struct Table {
    /// Title and width at 100%; a negative width takes the rest
    columns: Vec<(String, i32)>,
    rows: Vec<Vec<String>>,
    checks: Option<Vec<bool>>,
    selected: Option<usize>,
    hot: Option<usize>,
    scroll: i32,
    /// Row whose checkbox changed last (for CHECKED)
    changed: usize,
    font: HFONT,
}

thread_local! {
    static TABLES: RefCell<HashMap<usize, Table>> = RefCell::new(HashMap::new());
}

fn with<R>(h: HWND, f: impl FnOnce(&mut Table) -> R) -> Option<R> {
    TABLES.with(|t| t.borrow_mut().get_mut(&(h as usize)).map(f))
}

/// Same scale as the settings window around it
fn dpi(_: HWND) -> u32 {
    super::settings_window::dpi()
}

fn client(h: HWND) -> RECT {
    let mut rc: RECT = unsafe { std::mem::zeroed() };
    unsafe { GetClientRect(h, &mut rc) };
    rc
}

fn notify(h: HWND, code: u32) {
    unsafe {
        let id = GetDlgCtrlID(h) as usize;
        SendMessageW(
            GetParent(h),
            WM_COMMAND,
            id | (code as usize) << 16,
            h as LPARAM,
        );
    }
}

pub fn register() {
    thread_local!(static DONE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) });
    if DONE.replace(true) {
        return;
    }
    let class = wide(CLASS);
    unsafe {
        let mut wc: WNDCLASSEXW = std::mem::zeroed();
        wc.cbSize = std::mem::size_of::<WNDCLASSEXW>() as u32;
        wc.lpfnWndProc = Some(proc_);
        wc.hInstance = GetModuleHandleW(std::ptr::null());
        wc.hCursor = LoadCursorW(null_mut(), IDC_ARROW);
        wc.lpszClassName = class.as_ptr();
        RegisterClassExW(&wc);
    }
}

/// Set up a freshly created table window
pub fn init(h: HWND, columns: &[(&str, i32)], checkboxes: bool) {
    let table = Table {
        columns: columns.iter().map(|(t, w)| (t.to_string(), *w)).collect(),
        rows: Vec::new(),
        checks: checkboxes.then(Vec::new),
        selected: None,
        hot: None,
        scroll: 0,
        changed: 0,
        font: null_mut(),
    };
    TABLES.with(|t| t.borrow_mut().insert(h as usize, table));
}

/// Replace all rows (and their checkboxes when the table has them)
pub fn set_rows(h: HWND, rows: Vec<Vec<String>>, checks: Vec<bool>) {
    with(h, |t| {
        if t.checks.is_some() {
            t.checks = Some(checks);
        }
        t.rows = rows;
        t.selected = t.selected.filter(|&i| i < t.rows.len());
        t.hot = None;
    });
    clamp_scroll(h);
    unsafe { InvalidateRect(h, std::ptr::null(), 0) };
}

pub fn select(h: HWND, row: Option<usize>) {
    with(h, |t| t.selected = row.filter(|&i| i < t.rows.len()));
    if let Some(row) = row {
        scroll_into_view(h, row);
    }
    unsafe { InvalidateRect(h, std::ptr::null(), 0) };
}

pub fn selected(h: HWND) -> Option<usize> {
    with(h, |t| t.selected).flatten()
}

pub fn checked(h: HWND, row: usize) -> bool {
    with(h, |t| t.checks.as_ref().and_then(|c| c.get(row).copied()))
        .flatten()
        .unwrap_or(false)
}

/// Row whose checkbox the user changed last
pub fn last_changed(h: HWND) -> usize {
    with(h, |t| t.changed).unwrap_or(0)
}

pub fn cell(h: HWND, row: usize, col: usize) -> String {
    with(h, |t| t.rows.get(row).and_then(|r| r.get(col)).cloned())
        .flatten()
        .unwrap_or_default()
}

// ---------------------------------------------------------------- geometry

/// Left edge and width of each column in pixels
fn column_edges(h: HWND, columns: &[(String, i32)]) -> Vec<(i32, i32)> {
    let d = dpi(h);
    let total = client(h).right;
    let fixed: i32 = columns
        .iter()
        .filter(|(_, w)| *w >= 0)
        .map(|(_, w)| scale(*w, d))
        .sum();
    let mut x = 0;
    columns
        .iter()
        .map(|(_, w)| {
            let w = if *w < 0 {
                (total - fixed).max(0)
            } else {
                scale(*w, d)
            };
            let edge = (x, w);
            x += w;
            edge
        })
        .collect()
}

fn row_top(h: HWND, row: usize, scroll: i32) -> i32 {
    let d = dpi(h);
    scale(HEADER_H, d) + row as i32 * scale(ROW_H, d) - scroll
}

fn row_at(h: HWND, y: i32) -> Option<usize> {
    let d = dpi(h);
    let (scroll, count) = with(h, |t| (t.scroll, t.rows.len()))?;
    let body_y = y - scale(HEADER_H, d);
    if body_y < 0 {
        return None;
    }
    let row = ((body_y + scroll) / scale(ROW_H, d)) as usize;
    (row < count).then_some(row)
}

fn max_scroll(h: HWND) -> i32 {
    let d = dpi(h);
    let count = with(h, |t| t.rows.len()).unwrap_or(0) as i32;
    (count * scale(ROW_H, d) - (client(h).bottom - scale(HEADER_H, d))).max(0)
}

fn clamp_scroll(h: HWND) {
    let max = max_scroll(h);
    with(h, |t| t.scroll = t.scroll.clamp(0, max));
}

fn scroll_by(h: HWND, delta: i32) {
    let max = max_scroll(h);
    with(h, |t| t.scroll = (t.scroll + delta).clamp(0, max));
    unsafe { InvalidateRect(h, std::ptr::null(), 0) };
}

fn scroll_into_view(h: HWND, row: usize) {
    let d = dpi(h);
    let body = client(h).bottom - scale(HEADER_H, d);
    let row_h = scale(ROW_H, d);
    let top = row as i32 * row_h;
    with(h, |t| {
        if top < t.scroll {
            t.scroll = top;
        } else if top + row_h > t.scroll + body {
            t.scroll = top + row_h - body;
        }
    });
    clamp_scroll(h);
}

/// Checkbox square of a row, in client coordinates
fn checkbox_rect(h: HWND, row: usize, scroll: i32) -> RECT {
    let d = dpi(h);
    let size = scale(CHECK, d);
    let top = row_top(h, row, scroll) + (scale(ROW_H, d) - size) / 2;
    let left = scale(PAD, d);
    RECT {
        left,
        top,
        right: left + size,
        bottom: top + size,
    }
}

// ---------------------------------------------------------------- input

fn click(h: HWND, x: i32, y: i32) {
    unsafe { SetFocus(h) };
    let Some(row) = row_at(h, y) else {
        return;
    };
    let d = dpi(h);
    let (has_checks, scroll) = with(h, |t| (t.checks.is_some(), t.scroll)).unwrap_or((false, 0));
    let cb = checkbox_rect(h, row, scroll);
    if has_checks && x < cb.right + scale(PAD, d) / 2 {
        toggle(h, row);
    }
    set_selected(h, row);
}

fn toggle(h: HWND, row: usize) {
    let done = with(h, |t| {
        let checks = t.checks.as_mut()?;
        let c = checks.get_mut(row)?;
        *c = !*c;
        t.changed = row;
        Some(())
    })
    .flatten();
    if done.is_some() {
        unsafe { InvalidateRect(h, std::ptr::null(), 0) };
        notify(h, CHECKED);
    }
}

fn set_selected(h: HWND, row: usize) {
    let changed = with(h, |t| {
        let changed = t.selected != Some(row);
        t.selected = Some(row);
        changed
    })
    .unwrap_or(false);
    scroll_into_view(h, row);
    unsafe { InvalidateRect(h, std::ptr::null(), 0) };
    if changed {
        notify(h, SELECTED);
    }
}

fn key(h: HWND, vk: u16) {
    let (selected, count) = with(h, |t| (t.selected, t.rows.len())).unwrap_or((None, 0));
    if count == 0 {
        return;
    }
    match vk {
        VK_UP => set_selected(h, selected.map_or(0, |s| s.saturating_sub(1))),
        VK_DOWN => set_selected(h, selected.map_or(0, |s| (s + 1).min(count - 1))),
        VK_SPACE => {
            if let Some(s) = selected {
                toggle(h, s);
            }
        }
        _ => {}
    }
}

fn set_hot(h: HWND, row: Option<usize>) {
    let changed = with(h, |t| std::mem::replace(&mut t.hot, row) != row).unwrap_or(false);
    if changed {
        unsafe { InvalidateRect(h, std::ptr::null(), 0) };
    }
}

// ---------------------------------------------------------------- painting

fn paint(h: HWND) {
    let p = dark::palette();
    let d = dpi(h);
    let s = |v: i32| scale(v, d);
    unsafe {
        let mut ps: PAINTSTRUCT = std::mem::zeroed();
        let screen = BeginPaint(h, &mut ps);
        let rc = client(h);
        // Off-screen, so nothing flickers
        let dc = CreateCompatibleDC(screen);
        let bmp = CreateCompatibleBitmap(screen, rc.right, rc.bottom);
        let old_bmp = SelectObject(dc, bmp as HGDIOBJ);
        let brush = CreateSolidBrush(p.field);
        FillRect(dc, &rc, brush);
        DeleteObject(brush as HGDIOBJ);

        TABLES.with(|t| {
            let t = t.borrow();
            let Some(t) = t.get(&(h as usize)) else {
                return;
            };
            let old_font = SelectObject(dc, t.font as HGDIOBJ);
            SetBkMode(dc, TRANSPARENT as i32);
            let edges = column_edges(h, &t.columns);
            let header_h = s(HEADER_H);
            let line = |dc: HDC, r: RECT, color: u32| {
                let b = CreateSolidBrush(color);
                FillRect(dc, &r, b);
                DeleteObject(b as HGDIOBJ);
            };
            let text = |dc: HDC, s: &str, mut r: RECT, color: u32| {
                let mut w: Vec<u16> = s.encode_utf16().collect();
                SetTextColor(dc, color);
                DrawTextW(
                    dc,
                    w.as_mut_ptr(),
                    w.len() as i32,
                    &mut r,
                    DT_SINGLELINE | DT_VCENTER | DT_LEFT | DT_NOPREFIX,
                );
            };
            // Where a column's text starts: after the checkbox in the first column
            let text_left = |col: usize, x: i32| {
                if col == 0 && t.checks.is_some() {
                    x + s(PAD + CHECK + 10)
                } else {
                    x + s(PAD)
                }
            };

            // Column dividers first, so a row highlight covers them
            for (x, _) in edges.iter().skip(1) {
                line(
                    dc,
                    RECT {
                        left: *x,
                        top: header_h,
                        right: x + 1,
                        bottom: rc.bottom,
                    },
                    p.separator,
                );
            }
            // Rows, clipped under the header
            let saved = SaveDC(dc);
            IntersectClipRect(dc, 0, header_h, rc.right, rc.bottom);
            for (i, row) in t.rows.iter().enumerate() {
                let top = row_top(h, i, t.scroll);
                let bottom = top + s(ROW_H);
                if bottom <= header_h || top >= rc.bottom {
                    continue;
                }
                let selected = t.selected == Some(i);
                let bg = if selected {
                    p.row_selected
                } else if t.hot == Some(i) {
                    p.row_hot
                } else {
                    p.field
                };
                if bg != p.field {
                    let hl = RECT {
                        left: s(4),
                        top: top + s(1),
                        right: rc.right - s(4),
                        bottom: bottom - s(1),
                    };
                    controls::paint_highlight(dc, &hl, bg, p.field, d);
                }
                if let Some(checks) = &t.checks {
                    let cb = checkbox_rect(h, i, t.scroll);
                    let on = checks.get(i).copied().unwrap_or(false);
                    controls::paint_checkbox(dc, &cb, on, bg, d);
                }
                for (col, (x, w)) in edges.iter().enumerate() {
                    let value = row.get(col).map(String::as_str).unwrap_or("");
                    let r = RECT {
                        left: text_left(col, *x),
                        top,
                        right: x + w - s(8),
                        bottom,
                    };
                    text(dc, value, r, p.text);
                }
            }
            RestoreDC(dc, saved);

            // Header on top
            line(
                dc,
                RECT {
                    left: 0,
                    top: 0,
                    right: rc.right,
                    bottom: header_h,
                },
                p.field,
            );
            for (col, ((title, _), (x, w))) in t.columns.iter().zip(&edges).enumerate() {
                if col > 0 {
                    line(
                        dc,
                        RECT {
                            left: *x,
                            top: s(9),
                            right: x + 1,
                            bottom: header_h - s(9),
                        },
                        p.border,
                    );
                }
                let r = RECT {
                    left: text_left(col, *x),
                    top: 0,
                    right: x + w - s(8),
                    bottom: header_h,
                };
                text(dc, title, r, p.hint);
            }
            line(
                dc,
                RECT {
                    left: 0,
                    top: header_h - 1,
                    right: rc.right,
                    bottom: header_h,
                },
                p.border,
            );
            SelectObject(dc, old_font);
        });

        BitBlt(screen, 0, 0, rc.right, rc.bottom, dc, 0, 0, SRCCOPY);
        SelectObject(dc, old_bmp);
        DeleteObject(bmp as HGDIOBJ);
        DeleteDC(dc);
        EndPaint(h, &ps);
    }
}

unsafe extern "system" fn proc_(h: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let x = (lparam & 0xFFFF) as u16 as i16 as i32;
    let y = ((lparam >> 16) & 0xFFFF) as u16 as i16 as i32;
    match msg {
        WM_PAINT => paint(h),
        WM_ERASEBKGND => return 1,
        WM_SETFONT => {
            with(h, |t| t.font = wparam as HFONT);
        }
        WM_GETFONT => return with(h, |t| t.font as LRESULT).unwrap_or(0),
        WM_LBUTTONDOWN | WM_LBUTTONDBLCLK => click(h, x, y),
        WM_MOUSEMOVE => {
            let mut tme = TRACKMOUSEEVENT {
                cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                dwFlags: TME_LEAVE,
                hwndTrack: h,
                dwHoverTime: 0,
            };
            unsafe { TrackMouseEvent(&mut tme) };
            set_hot(h, row_at(h, y));
        }
        WM_MOUSELEAVE => set_hot(h, None),
        WM_MOUSEWHEEL => {
            let delta = ((wparam >> 16) & 0xFFFF) as u16 as i16 as i32;
            scroll_by(h, -delta * scale(ROW_H, dpi(h)) * 3 / 120);
        }
        WM_GETDLGCODE => return DLGC_WANTARROWS as LRESULT,
        WM_KEYDOWN => key(h, wparam as u16),
        WM_SETFOCUS => notify(h, EN_SETFOCUS),
        WM_KILLFOCUS => notify(h, EN_KILLFOCUS),
        WM_SIZE => clamp_scroll(h),
        WM_NCDESTROY => {
            TABLES.with(|t| t.borrow_mut().remove(&(h as usize)));
            return unsafe { DefWindowProcW(h, msg, wparam, lparam) };
        }
        _ => return unsafe { DefWindowProcW(h, msg, wparam, lparam) },
    }
    0
}
