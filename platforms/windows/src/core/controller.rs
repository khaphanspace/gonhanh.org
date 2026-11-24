//! Decides what happens to every key: shortcuts, buffer resets, engine processing, text output.
//! Windows counterpart of macOS keyboardCallback (RustBridge.swift) + AppState per-app logic.
//! Owns the core `Engine` directly (no FFI).

use gonhanh_core::engine::shortcut::Shortcut;
use gonhanh_core::engine::{Engine, FLAG_KEY_CONSUMED};

use crate::core::hotkey::{MOD_ALT, MOD_CTRL, MOD_SHIFT, MOD_WIN};
use crate::core::keys;
use crate::services::log;
use crate::services::settings::Settings;

const OWN_PROCESS: &str = "gonhanh.exe";

pub struct KeyEvent {
    pub vk: u16,
    /// Modifiers held before this key went down
    pub mods: u32,
    pub caps_lock: bool,
    /// Set → the key is blocked
    pub handled: bool,
}

impl KeyEvent {
    pub fn new(vk: u16, mods: u32, caps_lock: bool) -> Self {
        KeyEvent {
            vk,
            mods,
            caps_lock,
            handled: false,
        }
    }
    fn shift(&self) -> bool {
        self.mods & MOD_SHIFT != 0
    }
}

/// Engine output for one key
struct Reply {
    text: String,
    backspace: usize,
    consumed: bool,
}

/// (text, backspaces, key to type after the text or 0) → screen; false = not (fully) sent
pub type SendFn = Box<dyn FnMut(&str, usize, u16) -> bool>;

pub struct InputController {
    pub settings: Settings,
    engine: Engine,
    send: SendFn,
    current_app: String,
    /// Global on/off before a per-app profile forced a state
    profile_saved_enabled: Option<bool>,
    /// false in tests: never touch the registry
    pub persist: bool,
    pub is_non_latin_layout: Box<dyn Fn() -> bool>,
    pub on_enabled_changed: Box<dyn Fn(bool)>,
    pub play_sound: Box<dyn Fn(bool)>,
}

impl InputController {
    pub fn new(settings: Settings, send: SendFn) -> Self {
        let mut c = InputController {
            settings,
            engine: Engine::new(),
            send,
            current_app: String::new(),
            profile_saved_enabled: None,
            persist: true,
            is_non_latin_layout: Box::new(|| false),
            on_enabled_changed: Box::new(|_| {}),
            play_sound: Box::new(|_| {}),
        };
        c.apply_settings();
        c
    }

    /// Push every setting to the engine (after any settings change)
    pub fn apply_settings(&mut self) {
        let s = &self.settings;
        log::set_enabled(s.debug_log);
        let e = &mut self.engine;
        e.set_method(s.method as u8);
        e.set_enabled(s.enabled);
        e.set_modern_tone(s.modern_tone);
        e.set_skip_w_shortcut(!s.auto_w_shortcut);
        e.set_bracket_shortcut(s.bracket_shortcut);
        e.set_english_auto_restore(s.english_auto_restore);
        e.set_allow_foreign_consonants(s.allow_foreign_consonants);
        e.set_esc_restore(s.restore_shortcut_enabled);
        let table = e.shortcuts_mut();
        table.clear();
        for item in s
            .shortcuts
            .iter()
            .filter(|i| i.enabled && !i.key.is_empty() && !i.value.is_empty())
        {
            // Symbol-only triggers ("->") fire immediately, words ("hcm") at word end — same rule as ime_add_shortcut
            table.add(if item.key.chars().all(|c| !c.is_alphabetic()) {
                Shortcut::immediate(&item.key, &item.value)
            } else {
                Shortcut::new(&item.key, &item.value)
            });
        }
        self.apply_auto_capitalize();
    }

    fn apply_auto_capitalize(&mut self) {
        let on = self.settings.auto_capitalize
            && !self.settings.is_auto_capitalize_excluded(&self.current_app);
        self.engine.set_auto_capitalize(on);
    }

    // ------------------------------------------------------------ on/off

    /// User action (hotkey, tray, settings): persist, remember for this app, sound
    pub fn set_enabled(&mut self, on: bool) {
        self.settings.enabled = on;
        self.engine.set_enabled(on);
        self.engine.clear_all();
        if self.persist {
            self.settings.save();
            if self.settings.per_app_mode && !self.current_app.is_empty() {
                let app = self.current_app.clone();
                self.settings.save_per_app_mode(&app, on);
            }
        }
        if self.settings.sound_enabled {
            (self.play_sound)(on);
        }
        (self.on_enabled_changed)(on);
    }

    pub fn toggle(&mut self) {
        self.set_enabled(!self.settings.enabled);
    }

    /// App switch: not persisted, no sound
    fn set_enabled_silently(&mut self, on: bool) {
        if self.settings.enabled == on {
            return;
        }
        self.settings.enabled = on;
        self.engine.set_enabled(on);
        (self.on_enabled_changed)(on);
    }

    // ------------------------------------------------------------ app switching

    pub fn on_app_changed(&mut self, app: &str) {
        let app = app.to_lowercase();
        if app == OWN_PROCESS {
            return; // our own windows
        }
        self.current_app = app.clone();
        self.engine.clear_all();
        self.apply_auto_capitalize();

        // Per-app profile (advanced): always on / always off
        if let Some(&state) = self.settings.per_app_profiles.get(&app) {
            if state != 0 {
                self.profile_saved_enabled
                    .get_or_insert(self.settings.enabled);
                self.set_enabled_silently(state == 1);
                return;
            }
        }
        if let Some(saved) = self.profile_saved_enabled.take() {
            self.set_enabled_silently(saved);
        }

        // Remember on/off per app (macOS "Nhớ trạng thái theo app")
        if self.settings.per_app_mode {
            match self.settings.per_app_modes.get(&app) {
                Some(&on) => self.set_enabled_silently(on),
                None if self.persist => {
                    let on = self.settings.enabled;
                    self.settings.save_per_app_mode(&app, on);
                }
                None => {}
            }
        }
    }

    // ------------------------------------------------------------ keys

    pub fn on_mouse_clicked(&mut self) {
        self.engine.clear();
    }

    /// Modifiers pressed and released alone (e.g. Ctrl+Shift)
    pub fn on_modifiers_tapped(&mut self, mods: u32) {
        let s = &self.settings;
        if s.toggle_shortcut.matches_modifier_only(mods)
            || (s.secondary_toggle_enabled
                && s.secondary_toggle_shortcut.matches_modifier_only(mods))
        {
            self.toggle();
        } else if s.restore_shortcut_enabled && s.restore_shortcut.matches_modifier_only(mods) {
            self.restore();
        }
    }

    pub fn on_key(&mut self, e: &mut KeyEvent) {
        let (vk, mods) = (e.vk, e.mods);

        // Modifier keys themselves: Ctrl/Alt/Win alone end the word (Issue #150), Shift/CapsLock don't
        if keys::is_modifier(vk) {
            if !keys::is_shift(vk) {
                self.engine.clear();
            }
            return;
        }
        if vk == keys::CAPITAL {
            return;
        }

        // On/off shortcut
        let s = &self.settings;
        if s.toggle_shortcut.matches(vk, mods)
            || (s.secondary_toggle_enabled && s.secondary_toggle_shortcut.matches(vk, mods))
        {
            self.toggle();
            e.handled = true;
            return;
        }

        // Restore shortcut (Issue #149). Swallow the key only when something was restored, otherwise ESC
        // would also close the search box / dialog being typed in. With nothing to restore, special keys
        // keep their normal job (Issue #157).
        if s.restore_shortcut_enabled && s.restore_shortcut.matches(vk, mods) {
            let restored = self.restore();
            e.handled = restored || !keys::is_special_passthrough(vk);
            return;
        }
        // ESC must not restore when the restore shortcut is another key
        if vk == keys::ESCAPE && s.restore_shortcut_enabled {
            self.engine.clear();
            return;
        }

        // Japanese, Russian, Thai keyboards: stay out of the way
        if (self.is_non_latin_layout)() {
            self.engine.clear();
            return;
        }
        // Ctrl/Win shortcuts change context (Issue #365); AltGr = Ctrl+Alt
        if mods & (MOD_CTRL | MOD_WIN) != 0 {
            self.engine.clear_all();
            return;
        }
        // Alt+key: menu accelerators
        if mods & MOD_ALT != 0 {
            self.engine.clear();
            return;
        }
        // Shift+arrow/Home/End selects text (Issue #251)
        if e.shift() && keys::is_navigation(vk) {
            self.engine.clear();
            return;
        }

        // Home, End, Delete, F-keys, numpad: caret moved or unknown key
        let Some(reply) = self.process_key(vk, e.shift(), e.caps_lock) else {
            return; // pass the key through
        };

        // Word-ending keys (punctuation, Enter, Tab, arrows) are typed after the replacement,
        // unless a shortcut consumed them (e.g. "->" → "→")
        let retype = keys::is_break_key(vk, e.shift()) && !reply.consumed;
        if (self.send)(&reply.text, reply.backspace, if retype { vk } else { 0 }) {
            e.handled = true;
        } else {
            // Injection failed: let the key through and start a fresh word so engine and screen agree
            if log::enabled() {
                log::write("SendInput failed, passing key through");
            }
            self.engine.clear();
        }
    }

    /// Engine reply for a Windows key, None = nothing to replace
    fn process_key(&mut self, vk: u16, shift: bool, caps_lock: bool) -> Option<Reply> {
        let Some(mac) = keys::to_mac_key(vk) else {
            self.engine.clear();
            return None;
        };
        // Shift XOR CapsLock = uppercase (Shift+A with CapsLock on = lowercase), like Linux
        let r = self
            .engine
            .on_key_ext(mac, shift != caps_lock, false, shift);
        let text: String = r.chars[..r.count as usize]
            .iter()
            .filter(|&&c| c != 0)
            .filter_map(|&c| char::from_u32(c))
            .collect();
        if log::enabled() {
            log::write(&format!(
                "key vk=0x{vk:02X} shift={shift} caps={caps_lock} → action={} bs={} flags={} text='{text}'",
                r.action, r.backspace, r.flags
            ));
        }
        (r.action == 1).then_some(Reply {
            text,
            backspace: r.backspace as usize,
            consumed: r.flags & FLAG_KEY_CONSUMED != 0,
        })
    }

    /// Undo Vietnamese transforms of the current word (engine treats it as ESC). false = nothing to undo.
    fn restore(&mut self) -> bool {
        // A failed send counts as nothing restored, so the shortcut key keeps its normal job
        let restored = self
            .process_key(keys::ESCAPE, false, false)
            .is_some_and(|r| (self.send)(&r.text, r.backspace, 0));
        self.engine.clear();
        restored
    }
}

#[cfg(test)]
#[path = "controller_tests.rs"]
mod tests;
