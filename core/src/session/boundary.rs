//! Word boundaries: Space, punctuation, Enter, ESC. Shortcuts, English restore, history.

use super::autocap::{break_char, is_sentence_ending, should_reset_pending};
use super::history::Entry;
use super::out::Out;
use super::restore::{decide, Decision};
use super::Session;
use crate::compose::render::render_raw;
use crate::compose::{Display, RawKey, MAXK};
use crate::data::keys;

impl Session {
    /// Gõ tắt on a finished word. `trigger` is the key that ended it.
    fn word_boundary_shortcut(&mut self, trigger: char) -> Out {
        // nothing to expand: skip building the word as a String
        if self.shortcuts.is_empty() || (self.word.is_empty() && self.shortcut_prefix.is_empty()) {
            return Out::none();
        }
        if self.has_non_letter_prefix {
            return Out::none();
        }
        let full = if self.shortcut_prefix.is_empty() {
            self.screen_string()
        } else {
            format!("{}{}", self.shortcut_prefix, self.screen_string())
        };
        let key_char = (trigger == ' ').then_some(' ');
        let im = self.input_method();
        match self
            .shortcuts
            .try_match_for_method(&full, key_char, true, im)
        {
            Some(m) => {
                let output: Vec<char> = m.output.chars().collect();
                Out::send(m.backspace_count as u8, &output)
            }
            None => Out::none(),
        }
    }

    /// English restore for the finished word. Returns the edit, and the raw text when restored.
    fn restore_at_boundary(&self, with_space: bool) -> (Out, Option<Display>) {
        let raw = match decide(
            &self.word,
            self.english_restore,
            self.foreign_initials(),
            || self.free_tone && self.english_restore && self.looks_english(true),
        ) {
            // the screen may still show the typed letters (English guard): finish as Vietnamese
            Decision::Keep => self.word.kept_display(),
            Decision::Raw => render_raw(self.word.raw()),
            Decision::Squeezed(d) => d,
        };
        if raw == self.screen {
            return (Out::none(), None);
        }
        let mut chars: Vec<char> = raw.as_slice().to_vec();
        if with_space {
            chars.push(' ');
        }
        (Out::send(self.screen.len, &chars), Some(raw))
    }

    /// Remember the committed word so Backspace can bring it back.
    fn commit_word(&mut self, restored: Option<Display>) {
        let mut keys_copy = [RawKey { ch: 0, caps: false }; MAXK];
        let n = self.word.len();
        keys_copy[..n].copy_from_slice(self.word.raw());
        self.history.push(Entry {
            keys: keys_copy,
            len: n as u8,
            shown: restored.unwrap_or(self.screen),
            literal: restored.is_some(),
        });
        self.spaces_after_commit = 1;
    }

    pub(super) fn on_space(&mut self) -> Out {
        let sr = self.word_boundary_shortcut(' ');
        if !sr.is_none() {
            self.clear();
            return sr;
        }
        let (res, restored) = self.restore_at_boundary(true);
        if !self.word.is_empty() {
            self.commit_word(restored);
        } else if self.spaces_after_commit > 0 {
            self.spaces_after_commit = self.spaces_after_commit.saturating_add(1);
        }
        self.auto_capitalize_used = false;
        if self.auto_capitalize && self.saw_sentence_ending {
            self.pending_capitalize = true;
        }
        self.clear();
        res
    }

    pub(super) fn on_esc(&mut self) -> Out {
        let r = if self.esc_restore {
            self.restore_to_raw()
        } else {
            Out::none()
        };
        self.clear();
        self.history.clear();
        self.spaces_after_commit = 0;
        r
    }

    /// ESC: back to exactly what was typed.
    fn restore_to_raw(&self) -> Out {
        if self.word.is_empty() {
            return Out::none();
        }
        let raw = render_raw(self.word.raw());
        if !self.word.transformed() && raw == self.screen {
            return Out::none();
        }
        Out::send(self.screen.len, raw.as_slice())
    }

    pub(super) fn on_break(&mut self, key: u16, shift: bool) -> Out {
        let im = self.input_method();
        let at_true_start =
            self.word.is_empty() && self.history.is_empty() && self.spaces_after_commit == 0;
        let continuing_prefix = self.word.is_empty() && !self.shortcut_prefix.is_empty();

        // nothing typed yet: build symbol shortcuts such as "->" or "#fne"
        if at_true_start || continuing_prefix {
            if continuing_prefix && self.spaces_after_commit > 0 {
                self.spaces_after_commit = self.spaces_after_commit.saturating_add(1);
            }
            if at_true_start {
                self.has_non_letter_prefix = false;
            }
            if let Some(ch) = break_char(key, shift) {
                self.shortcut_prefix.push(ch);
                if let Some(m) =
                    self.shortcuts
                        .try_match_for_method(&self.shortcut_prefix, None, false, im)
                {
                    let output: Vec<char> = m.output.chars().collect();
                    let backspace = (m.backspace_count as u8).saturating_sub(1);
                    self.shortcut_prefix.clear();
                    return Out::send_consumed(backspace, &output);
                }
                self.note_sentence_end(key, shift, false);
                return Out::none();
            }
        }

        self.note_sentence_end(key, shift, true);
        self.auto_capitalize_used = false;

        let enter = key == keys::RETURN || key == keys::ENTER;
        let trigger = if enter {
            Some('\n')
        } else {
            break_char(key, shift)
        };
        if let Some(ch) = trigger {
            let sr = self.word_boundary_shortcut(ch);
            if !sr.is_none() {
                self.clear();
                self.history.clear();
                self.spaces_after_commit = 0;
                return sr;
            }
        }

        let (res, restored) = self.restore_at_boundary(false);
        if !self.word.is_empty() {
            self.commit_word(restored);
        } else if self.spaces_after_commit > 0 && break_char(key, shift).is_some() {
            self.spaces_after_commit = self.spaces_after_commit.saturating_add(1);
        } else {
            self.history.clear();
            self.spaces_after_commit = 0;
        }
        self.clear();
        if let Some(ch) = break_char(key, shift) {
            self.shortcut_prefix.push(ch);
        }
        res
    }

    /// Auto-capitalize bookkeeping for a break key. `reset_others`: other break keys cancel a
    /// pending capital (only when a word was in progress).
    fn note_sentence_end(&mut self, key: u16, shift: bool, reset_others: bool) {
        if !self.auto_capitalize {
            return;
        }
        if is_sentence_ending(key, shift) {
            self.saw_sentence_ending = true;
        } else if key == keys::RETURN || key == keys::ENTER {
            self.pending_capitalize = true;
            self.saw_sentence_ending = false;
        } else if reset_others && should_reset_pending(key, shift) {
            self.pending_capitalize = false;
            self.saw_sentence_ending = false;
        }
    }
}
