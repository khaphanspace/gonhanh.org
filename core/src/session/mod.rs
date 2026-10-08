//! Engine v2 session: the lifecycle around the typing core.
//!
//! `compose` knows how to turn the keys of ONE word into text. The session knows everything
//! that is not phonology: word boundaries, shortcuts, auto-capitalize, ESC, Backspace history,
//! brackets, English restore. It exposes the same API as engine v1 so platforms and tests can
//! switch with a feature flag.
//!
//! Screen invariant: `screen` is exactly what is displayed for the current word; every result
//! is `diff(screen, new display)`, so no backspace is ever counted by hand.

mod autocap;
mod boundary;
mod disabled;
mod history;
mod keymap;
mod out;
mod restore;
mod word;

use crate::compose::{Compose, Display, Method, Options};
use crate::engine::shortcut::{InputMethod, ShortcutTable};
use crate::engine::Result;
use history::History;
use out::Out;

pub struct Session {
    method: u8,
    enabled: bool,
    skip_w_shortcut: bool,
    bracket_shortcut: bool,
    esc_restore: bool,
    free_tone: bool,
    modern_tone: bool,
    english_restore: bool,
    auto_capitalize: bool,
    allow_foreign: bool,
    shortcuts: ShortcutTable,

    word: Compose,
    screen: Display,
    history: History,
    spaces_after_commit: u8,
    restored_pending_clear: bool,
    restored_is_ascii: bool,
    has_non_letter_prefix: bool,
    shortcut_prefix: String,
    pending_capitalize: bool,
    auto_capitalize_used: bool,
    saw_sentence_ending: bool,
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

impl Session {
    pub fn new() -> Self {
        Session {
            method: 0,
            enabled: true,
            skip_w_shortcut: false,
            bracket_shortcut: false,
            esc_restore: false,
            free_tone: false,
            modern_tone: true,
            english_restore: false,
            auto_capitalize: false,
            allow_foreign: false,
            shortcuts: ShortcutTable::with_defaults(),
            word: Compose::new(Options::default()),
            screen: Display::empty(),
            history: History::new(),
            spaces_after_commit: 0,
            restored_pending_clear: false,
            restored_is_ascii: false,
            has_non_letter_prefix: false,
            shortcut_prefix: String::new(),
            pending_capitalize: false,
            auto_capitalize_used: false,
            saw_sentence_ending: false,
        }
    }

    // ---- settings ---------------------------------------------------------------------------

    fn options(&self) -> Options {
        Options {
            method: if self.method == 1 {
                Method::Vni
            } else {
                Method::Telex
            },
            modern_tone: self.modern_tone,
            free: self.free_tone,
            foreign_initials: self.foreign_initials(),
            w_as_vowel: !self.skip_w_shortcut,
            bracket: self.bracket_shortcut && self.method == 0,
            english_guard: self.english_restore,
        }
    }

    fn apply_options(&mut self) {
        let o = self.options();
        self.word.update_options(o);
    }

    pub fn set_method(&mut self, method: u8) {
        self.method = method;
        self.apply_options();
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.word.clear();
            self.screen = Display::empty();
            self.history.clear();
            self.spaces_after_commit = 0;
        }
    }

    pub fn set_skip_w_shortcut(&mut self, skip: bool) {
        self.skip_w_shortcut = skip;
        self.apply_options();
    }

    pub fn set_bracket_shortcut(&mut self, enabled: bool) {
        self.bracket_shortcut = enabled;
        self.apply_options();
    }

    pub fn set_esc_restore(&mut self, enabled: bool) {
        self.esc_restore = enabled;
    }

    pub fn set_free_tone(&mut self, enabled: bool) {
        self.free_tone = enabled;
        self.apply_options();
    }

    pub fn set_modern_tone(&mut self, modern: bool) {
        self.modern_tone = modern;
        self.apply_options();
    }

    pub fn set_english_auto_restore(&mut self, enabled: bool) {
        self.english_restore = enabled;
        self.apply_options();
    }

    pub fn set_auto_capitalize(&mut self, enabled: bool) {
        self.auto_capitalize = enabled;
        if !enabled {
            self.pending_capitalize = false;
            self.saw_sentence_ending = false;
        }
    }

    /// f j w z may start a syllable: asked for explicitly, or implied by free typing (zị, wé).
    fn foreign_initials(&self) -> bool {
        self.allow_foreign || self.free_tone
    }

    pub fn set_allow_foreign_consonants(&mut self, enabled: bool) {
        self.allow_foreign = enabled;
        self.apply_options();
    }

    pub fn allow_foreign_consonants(&self) -> bool {
        self.allow_foreign
    }

    pub fn shortcuts(&self) -> &ShortcutTable {
        &self.shortcuts
    }

    pub fn shortcuts_mut(&mut self) -> &mut ShortcutTable {
        &mut self.shortcuts
    }

    fn input_method(&self) -> InputMethod {
        match self.method {
            0 => InputMethod::Telex,
            1 => InputMethod::Vni,
            _ => InputMethod::All,
        }
    }

    // ---- entry points -----------------------------------------------------------------------

    pub fn on_key(&mut self, key: u16, caps: bool, ctrl: bool) -> Result {
        self.on_key_ext(key, caps, ctrl, false)
    }

    pub fn on_key_ext(&mut self, key: u16, caps: bool, ctrl: bool, shift: bool) -> Result {
        self.key_out(key, caps, ctrl, shift).into()
    }

    fn key_out(&mut self, key: u16, caps: bool, ctrl: bool, shift: bool) -> Out {
        use crate::data::keys;
        if ctrl {
            self.clear();
            self.history.clear();
            self.spaces_after_commit = 0;
            return Out::none();
        }
        if !self.enabled {
            return self.disabled_key(key, caps, shift);
        }
        if key == keys::SPACE {
            return self.on_space();
        }
        if key == keys::ESC {
            return self.on_esc();
        }
        if self.method == 0 && (key == keys::RBRACKET || key == keys::LBRACKET) {
            if let Some(r) = self.try_bracket(key, caps) {
                return r;
            }
        }
        if keys::is_break_ext(key, shift) {
            return self.on_break(key, shift);
        }
        if key == keys::DELETE {
            return self.on_delete();
        }
        self.on_word_key(key, caps, shift)
    }

    /// Option-modified keys: `ch` is the real character, used for shortcut suffix matching.
    pub fn on_key_with_char(
        &mut self,
        key: u16,
        caps: bool,
        ctrl: bool,
        shift: bool,
        ch: Option<char>,
    ) -> Result {
        let Some(ch) = ch else {
            return self.on_key_ext(key, caps, ctrl, shift);
        };
        self.char_out(key, caps, ctrl, shift, ch).into()
    }

    fn char_out(&mut self, _key: u16, _caps: bool, ctrl: bool, _shift: bool, ch: char) -> Out {
        if ctrl {
            self.word.clear();
            self.screen = Display::empty();
            self.history.clear();
            self.spaces_after_commit = 0;
        }
        self.shortcut_prefix.push(ch);
        let im = self.input_method();
        for (idx, _) in self.shortcut_prefix.char_indices() {
            let suffix = &self.shortcut_prefix[idx..];
            if let Some(m) = self.shortcuts.try_match_for_method(suffix, None, false, im) {
                let output: Vec<char> = m.output.chars().collect();
                let backspace = (m.backspace_count as u8).saturating_sub(1);
                self.shortcut_prefix.clear();
                return Out::send_consumed(backspace, &output);
            }
        }
        Out::none()
    }

    // ---- state ------------------------------------------------------------------------------

    /// Drop the current word (not the history).
    pub fn clear(&mut self) {
        if self.auto_capitalize_used {
            self.pending_capitalize = true;
            self.auto_capitalize_used = false;
        }
        self.word.clear();
        self.screen = Display::empty();
        self.has_non_letter_prefix = false;
        self.restored_pending_clear = false;
        self.restored_is_ascii = false;
        self.shortcut_prefix.clear();
    }

    /// Cursor moved or text pasted: forget everything.
    pub fn clear_all(&mut self) {
        self.clear();
        self.history.clear();
        self.spaces_after_commit = 0;
        self.pending_capitalize = false;
        self.saw_sentence_ending = false;
    }

    pub fn get_buffer_string(&self) -> String {
        self.screen.as_slice().iter().collect()
    }

    pub fn raw_input_len(&self) -> usize {
        self.word.len()
    }

    pub fn is_raw_english(&self) -> bool {
        let raw = self.word.raw();
        !raw.is_empty()
            && raw.iter().all(|k| k.ch.is_ascii_alphabetic())
            && (raw.len() <= 2
                || raw
                    .iter()
                    .any(|k| matches!(k.ch, b'a' | b'e' | b'i' | b'o' | b'u' | b'y')))
    }

    pub fn had_vowel_circumflex(&self) -> bool {
        self.word
            .best()
            .units()
            .iter()
            .any(|u| u.md == crate::phonology::Mod::Circ)
    }

    /// Resume editing a word that is on screen (cursor placed at its end).
    pub fn restore_word(&mut self, word: &str) {
        self.clear();
        let Some((keys, n)) = keymap::keys_for_word(word, self.options().method, self.options())
        else {
            return;
        };
        for k in &keys[..n] {
            self.word.push(*k);
        }
        self.screen = self.word.display();
        self.restored_pending_clear = true;
        self.restored_is_ascii = word.is_ascii();
    }

    fn screen_string(&self) -> String {
        self.screen.as_slice().iter().collect()
    }
}
