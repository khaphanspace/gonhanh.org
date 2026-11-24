//! Owner-drawn parts of the settings window: toggle switches and sidebar items, in the light/dark
//! palette with the Windows accent color. GDI has no anti-aliasing, so rounded shapes are drawn 4×
//! larger off-screen and shrunk with HALFTONE, which smooths the edges.

use std::ptr::null_mut;

use windows_sys::Win32::Foundation::{POINT, RECT};
use windows_sys::Win32::Graphics::Gdi::*;

use super::dark;
use crate::win::scale;

const K: i32 = 4; // supersampling factor
/// Corner diameter of buttons and text boxes at 100% (8 px corners, like the Claude app)
const RADIUS: i32 = 16;

fn fill(hdc: HDC, rect: &RECT, color: u32) {
    unsafe {
        let brush = CreateSolidBrush(color);
        FillRect(hdc, rect, brush);
        DeleteObject(brush as HGDIOBJ);
    }
}

/// Filled rounded rectangle (`radius` = corner diameter), no outline
fn round_rect(hdc: HDC, r: &RECT, color: u32, radius: i32) {
    unsafe {
        let brush = CreateSolidBrush(color);
        let old_brush = SelectObject(hdc, brush as HGDIOBJ);
        let old_pen = SelectObject(hdc, GetStockObject(NULL_PEN));
        RoundRect(
            hdc,
            r.left,
            r.top,
            r.right + 1,
            r.bottom + 1,
            radius,
            radius,
        );
        SelectObject(hdc, old_pen);
        SelectObject(hdc, old_brush);
        DeleteObject(brush as HGDIOBJ);
    }
}

/// Draw with `paint` on a `bg`-filled canvas K times the size of `rc`, then shrink it onto `hdc`.
/// `paint` gets the canvas DC and its size.
fn smooth(hdc: HDC, rc: &RECT, bg: u32, paint: impl FnOnce(HDC, i32, i32)) {
    let (w, h) = (rc.right - rc.left, rc.bottom - rc.top);
    if w <= 0 || h <= 0 {
        return;
    }
    unsafe {
        let mem = CreateCompatibleDC(hdc);
        let bmp = CreateCompatibleBitmap(hdc, w * K, h * K);
        let old = SelectObject(mem, bmp as HGDIOBJ);
        let canvas = RECT {
            left: 0,
            top: 0,
            right: w * K,
            bottom: h * K,
        };
        fill(mem, &canvas, bg);
        paint(mem, w * K, h * K);
        SetStretchBltMode(hdc, HALFTONE);
        SetBrushOrgEx(hdc, 0, 0, null_mut());
        StretchBlt(hdc, rc.left, rc.top, w, h, mem, 0, 0, w * K, h * K, SRCCOPY);
        SelectObject(mem, old);
        DeleteObject(bmp as HGDIOBJ);
        DeleteDC(mem);
    }
}

/// Pill-shaped switch filling `rc` (40×20 at 100%): accent track with the knob right when on,
/// gray track with the knob left when off
pub fn paint_switch(hdc: HDC, rc: &RECT, on: bool, enabled: bool) {
    let p = dark::palette();
    let (track, knob) = match (enabled, on) {
        (false, _) => (p.switch_disabled, p.knob_disabled),
        (true, true) => (dark::accent(), p.knob_on),
        (true, false) => (p.switch_off, p.knob_off),
    };
    smooth(hdc, rc, p.bg, |dc, w, h| {
        let all = RECT {
            left: 0,
            top: 0,
            right: w,
            bottom: h,
        };
        round_rect(dc, &all, track, h);
        let pad = h / 8;
        let d = h - 2 * pad;
        let x = if on { w - pad - d } else { pad };
        let knob_rect = RECT {
            left: x,
            top: pad,
            right: x + d,
            bottom: pad + d,
        };
        round_rect(dc, &knob_rect, knob, d);
    });
}

/// Sidebar item: text on the sidebar color; the selected one on a soft highlight with an accent bar
pub fn paint_nav_item(
    hdc: HDC,
    rc: &RECT,
    text: &mut [u16],
    selected: bool,
    focus: bool,
    dpi: u32,
) {
    let p = dark::palette();
    let s = |v: i32| scale(v, dpi);
    fill(hdc, rc, p.sidebar);
    if selected {
        let item = RECT {
            left: rc.left + s(8),
            top: rc.top + s(2),
            right: rc.right - s(8),
            bottom: rc.bottom - s(2),
        };
        let bar_h = s(16);
        smooth(hdc, &item, p.sidebar, |dc, w, h| {
            let all = RECT {
                left: 0,
                top: 0,
                right: w,
                bottom: h,
            };
            round_rect(dc, &all, p.nav_selected, RADIUS * s(1) * K);
            let bar = RECT {
                left: 0,
                top: (h - bar_h * K) / 2,
                right: s(3) * K,
                bottom: (h + bar_h * K) / 2,
            };
            round_rect(dc, &bar, dark::accent(), s(3) * K);
        });
    }
    unsafe {
        SetBkMode(hdc, TRANSPARENT as i32);
        SetTextColor(hdc, if selected { p.text } else { p.hint });
        let mut tr = RECT {
            left: rc.left + s(24),
            ..*rc
        };
        DrawTextW(
            hdc,
            text.as_mut_ptr(),
            text.len() as i32,
            &mut tr,
            DT_SINGLELINE | DT_VCENTER | DT_LEFT,
        );
        if focus {
            let fr = RECT {
                left: rc.left + s(6),
                top: rc.top + 1,
                right: rc.right - s(6),
                bottom: rc.bottom - 1,
            };
            DrawFocusRect(hdc, &fr);
        }
    }
}

/// Rounded box in the Claude app style: one even 1px border, `fill` inside
fn boxed(dc: HDC, w: i32, h: i32, radius: i32, fill_color: u32, border: u32) {
    let all = RECT {
        left: 0,
        top: 0,
        right: w,
        bottom: h,
    };
    round_rect(dc, &all, border, radius);
    round_rect(
        dc,
        &RECT {
            left: K,
            top: K,
            right: w - K,
            bottom: h - K,
        },
        fill_color,
        radius - 2 * K,
    );
}

pub struct ButtonLook {
    pub hot: bool,
    pub pressed: bool,
    pub enabled: bool,
    pub focus: bool,
    /// Drop-down: text on the left and a chevron on the right
    pub chevron: bool,
}

/// Push button / drop-down button filling `rc`
pub fn paint_button(hdc: HDC, rc: &RECT, text: &mut [u16], look: ButtonLook, dpi: u32) {
    let p = dark::palette();
    let s = |v: i32| scale(v, dpi);
    let fill_color = if !look.enabled {
        p.button_disabled
    } else if look.pressed {
        p.button_pressed
    } else if look.hot {
        p.button_hot
    } else {
        p.button
    };
    let chevron_w = s(12);
    smooth(hdc, rc, p.bg, |dc, w, h| {
        // Keyboard focus: a darker border instead of a dotted rectangle
        let border = if look.focus { p.border_focus } else { p.border };
        boxed(dc, w, h, RADIUS * s(1) * K, fill_color, border);
        if look.chevron {
            // ⌄ drawn as two strokes
            unsafe {
                let pen = CreatePen(PS_SOLID, (s(1) * K * 3 / 2).max(K), p.hint);
                let old = SelectObject(dc, pen as HGDIOBJ);
                let cx = w - s(14) * K;
                let cy = h / 2 + s(1) * K;
                let d = s(4) * K;
                let pts = [
                    windows_sys::Win32::Foundation::POINT {
                        x: cx - d,
                        y: cy - d / 2,
                    },
                    windows_sys::Win32::Foundation::POINT {
                        x: cx,
                        y: cy + d / 2,
                    },
                    windows_sys::Win32::Foundation::POINT {
                        x: cx + d,
                        y: cy - d / 2,
                    },
                ];
                Polyline(dc, pts.as_ptr(), 3);
                SelectObject(dc, old);
                DeleteObject(pen as HGDIOBJ);
            }
        }
    });
    unsafe {
        SetBkMode(hdc, TRANSPARENT as i32);
        SetTextColor(
            hdc,
            if !look.enabled {
                p.disabled
            } else if look.pressed {
                p.hint
            } else {
                p.text
            },
        );
        let mut tr = *rc;
        let format = if look.chevron {
            tr.left += s(11);
            tr.right -= s(12) + chevron_w;
            DT_LEFT
        } else {
            DT_CENTER
        };
        DrawTextW(
            hdc,
            text.as_mut_ptr(),
            text.len() as i32,
            &mut tr,
            DT_SINGLELINE | DT_VCENTER | format,
        );
    }
}

/// Frame of a text box or list: rounded border, field color inside; the border darkens while typing
pub fn paint_field(hdc: HDC, rc: &RECT, focus: bool, dpi: u32) {
    let p = dark::palette();
    let s = |v: i32| scale(v, dpi);
    let border = if focus { p.border_focus } else { p.border };
    smooth(hdc, rc, p.bg, |dc, w, h| {
        boxed(dc, w, h, RADIUS * s(1) * K, p.field, border);
    });
}

// ---------------------------------------------------------------- checkbox

/// Rounded checkbox filling `r` (16×16 at 100%): accent with a check mark when on, outlined when off
pub fn paint_checkbox(hdc: HDC, r: &RECT, checked: bool, bg: u32, dpi: u32) {
    let p = dark::palette();
    let s = |v: i32| scale(v, dpi);
    smooth(hdc, r, bg, |dc, w, h| {
        let radius = s(8) * K;
        if !checked {
            boxed(dc, w, h, radius, p.field, p.border_focus);
            return;
        }
        let all = RECT {
            left: 0,
            top: 0,
            right: w,
            bottom: h,
        };
        round_rect(dc, &all, dark::accent(), radius);
        unsafe {
            let pen = CreatePen(PS_SOLID, s(2) * K, p.knob_on);
            let old = SelectObject(dc, pen as HGDIOBJ);
            let pts = [
                POINT {
                    x: w * 27 / 100,
                    y: h * 52 / 100,
                },
                POINT {
                    x: w * 43 / 100,
                    y: h * 68 / 100,
                },
                POINT {
                    x: w * 74 / 100,
                    y: h * 34 / 100,
                },
            ];
            Polyline(dc, pts.as_ptr(), 3);
            SelectObject(dc, old);
            DeleteObject(pen as HGDIOBJ);
        }
    });
}

/// Soft rounded highlight filling `r` (table rows)
pub fn paint_highlight(hdc: HDC, r: &RECT, color: u32, bg: u32, dpi: u32) {
    smooth(hdc, r, bg, |dc, w, h| {
        let all = RECT {
            left: 0,
            top: 0,
            right: w,
            bottom: h,
        };
        round_rect(dc, &all, color, RADIUS * scale(1, dpi) * K);
    });
}
