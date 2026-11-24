//! Settings window colors: system light look, or a Windows 11 dark look when Windows apps are dark.
//! Standard controls get the system dark themes; switches and the sidebar are painted in controls.rs.

use std::cell::Cell;
use std::ptr::null_mut;

use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_USE_IMMERSIVE_DARK_MODE};
use windows_sys::Win32::Graphics::Gdi::*;
use windows_sys::Win32::UI::Controls::*;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

use super::theme;
use crate::services::registry;
use crate::win::wide;

/// COLORREF values (0x00BBGGRR)
#[derive(Clone, Copy)]
pub struct Palette {
    pub bg: u32,
    pub field: u32,
    pub text: u32,
    pub hint: u32,
    pub disabled: u32,
    pub separator: u32,
    pub sidebar: u32,
    pub nav_selected: u32,
    pub switch_off: u32,
    pub switch_disabled: u32,
    pub knob_on: u32,
    pub knob_off: u32,
    pub knob_disabled: u32,
    pub button: u32,
    pub button_hot: u32,
    pub button_pressed: u32,
    pub button_disabled: u32,
    pub border: u32,
    pub border_focus: u32,
    pub row_selected: u32,
    pub row_hot: u32,
}

thread_local! {
    static DARK: Cell<bool> = const { Cell::new(false) };
    static ACCENT: Cell<u32> = const { Cell::new(0) };
    static BRUSHES: Cell<[HBRUSH; 4]> = const { Cell::new([null_mut(); 4]) }; // bg, field, separator, sidebar
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
            bg: gray(0x27),
            field: gray(0x2D),
            text: gray(0xFF),
            hint: gray(0xA0),
            disabled: gray(0x6E),
            separator: gray(0x3A),
            sidebar: gray(0x1F),
            nav_selected: gray(0x2D),
            switch_off: gray(0x55),
            switch_disabled: gray(0x3A),
            knob_on: gray(0x00),
            knob_off: gray(0xD0),
            knob_disabled: gray(0x6E),
            button: gray(0x30),
            button_hot: gray(0x3A),
            button_pressed: gray(0x2B),
            button_disabled: gray(0x2F),
            border: gray(0x48),
            border_focus: gray(0x80),
            row_selected: gray(0x3C),
            row_hot: gray(0x33),
        }
    } else {
        Palette {
            bg: gray(0xFB),
            field: gray(0xFF),
            text: gray(0x1A),
            hint: 0x80_72_6B, // #6B7280
            disabled: gray(0xA0),
            separator: gray(0xE5),
            sidebar: gray(0xF0),
            nav_selected: gray(0xE3),
            switch_off: gray(0xD1),
            switch_disabled: gray(0xE6),
            knob_on: gray(0xFF),
            knob_off: gray(0xFF),
            knob_disabled: gray(0xF7),
            button: gray(0xFF),
            button_hot: gray(0xF3),
            button_pressed: gray(0xEA),
            button_disabled: gray(0xF7),
            border: gray(0xDC),
            border_focus: gray(0x9A),
            row_selected: gray(0xEC),
            row_hot: gray(0xF5),
        }
    }
}

/// Windows accent color, as Settings uses it for switches: a darker shade on light, a lighter one on dark
pub fn accent() -> u32 {
    ACCENT.get()
}

fn read_accent() -> u32 {
    // AccentPalette: 8 RGBA colors, from lightest (0) to darkest (6)
    let index = if is_dark() { 1 } else { 4 };
    registry::get_binary(
        r"Software\Microsoft\Windows\CurrentVersion\Explorer\Accent",
        "AccentPalette",
        32,
    )
    .filter(|b| b.len() >= 32)
    .map(|b| {
        let c = &b[index * 4..];
        c[0] as u32 | (c[1] as u32) << 8 | (c[2] as u32) << 16
    })
    .unwrap_or(if is_dark() { 0xFF_C2_4C } else { 0xC0_67_00 }) // Windows default blue
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
pub fn sidebar_brush() -> HBRUSH {
    BRUSHES.get()[3]
}

/// Window created: pick light/dark and the accent from Windows, make brushes, dark title bar
pub fn begin(hwnd: HWND) {
    DARK.set(theme::apps_dark());
    ACCENT.set(read_accent());
    let p = palette();
    BRUSHES.set(unsafe {
        [
            CreateSolidBrush(p.bg),
            CreateSolidBrush(p.field),
            CreateSolidBrush(p.separator),
            CreateSolidBrush(p.sidebar),
        ]
    });
    if is_dark() {
        theme::allow_for_window(hwnd);
        let on: i32 = 1;
        unsafe {
            DwmSetWindowAttribute(
                hwnd,
                DWMWA_USE_IMMERSIVE_DARK_MODE as u32,
                &on as *const _ as _,
                4,
            )
        };
    }
}

pub fn end() {
    for b in BRUSHES.replace([null_mut(); 4]) {
        if !b.is_null() {
            unsafe { DeleteObject(b as HGDIOBJ) };
        }
    }
}

fn set_theme(h: HWND, name: &str) {
    let n = wide(name);
    unsafe { SetWindowTheme(h, n.as_ptr(), std::ptr::null()) };
}

// ---------------------------------------------------------------- standard controls

/// Theme a freshly created control (no-op in light mode)
pub fn style_control(h: HWND, class: &str, style: u32) {
    if !is_dark() {
        return;
    }
    match class {
        "Button" => set_theme(h, "DarkMode_Explorer"),
        // Borderless text boxes (the frame is painted around them); multiline ones need
        // DarkMode_Explorer for a dark scrollbar
        "Edit" if style & ES_MULTILINE as u32 != 0 => set_theme(h, "DarkMode_Explorer"),
        "Edit" | "ComboBox" => set_theme(h, "DarkMode_CFD"),
        _ => {}
    }
}
