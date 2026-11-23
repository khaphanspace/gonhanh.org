//! Keyboard shortcut: key + modifiers, or modifiers alone (vk = 0, e.g. Ctrl+Shift like Unikey)

use crate::keys;

pub const MOD_CTRL: u32 = 1;
pub const MOD_ALT: u32 = 2;
pub const MOD_SHIFT: u32 = 4;
pub const MOD_WIN: u32 = 8;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Hotkey {
    pub vk: u16,
    pub mods: u32,
}

impl Hotkey {
    pub const DEFAULT_TOGGLE: Hotkey = Hotkey {
        vk: keys::SPACE,
        mods: MOD_CTRL,
    };
    pub const DEFAULT_SECONDARY_TOGGLE: Hotkey = Hotkey {
        vk: keys::SPACE,
        mods: MOD_CTRL | MOD_SHIFT,
    };
    pub const DEFAULT_RESTORE: Hotkey = Hotkey {
        vk: keys::ESCAPE,
        mods: 0,
    };

    pub fn modifier_only(&self) -> bool {
        self.vk == 0
    }

    /// Same encoding as the earlier Windows apps (registry compatible)
    pub fn to_int(self) -> u32 {
        (self.mods << 16) | self.vk as u32
    }
    pub fn from_int(v: u32) -> Self {
        Hotkey {
            vk: (v & 0xFFFF) as u16,
            mods: v >> 16,
        }
    }

    pub fn matches(&self, vk: u16, held: u32) -> bool {
        !self.modifier_only() && self.vk == vk && self.mods == held
    }
    pub fn matches_modifier_only(&self, held: u32) -> bool {
        self.modifier_only() && self.mods != 0 && self.mods == held
    }
}

impl std::fmt::Display for Hotkey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut parts: Vec<String> = Vec::new();
        for (flag, name) in [
            (MOD_CTRL, "Ctrl"),
            (MOD_ALT, "Alt"),
            (MOD_SHIFT, "Shift"),
            (MOD_WIN, "Win"),
        ] {
            if self.mods & flag != 0 {
                parts.push(name.into());
            }
        }
        if !self.modifier_only() {
            parts.push(key_name(self.vk));
        }
        if parts.is_empty() {
            f.write_str("—")
        } else {
            f.write_str(&parts.join(" + "))
        }
    }
}

fn key_name(vk: u16) -> String {
    match vk {
        keys::SPACE => return "Space".into(),
        keys::ESCAPE => return "Esc".into(),
        keys::TAB => return "Tab".into(),
        keys::RETURN => return "Enter".into(),
        keys::BACK => return "Backspace".into(),
        keys::DELETE => return "Delete".into(),
        keys::OEM_3 => return "`".into(),
        _ => {}
    }
    if keys::is_letter(vk) || keys::is_number(vk) {
        return (vk as u8 as char).to_string();
    }
    if (0x70..=0x87).contains(&vk) {
        return format!("F{}", vk - 0x6F);
    }
    crate::win::key_name_text(vk).unwrap_or_else(|| format!("0x{vk:02X}"))
}
