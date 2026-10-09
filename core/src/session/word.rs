//! Keys that belong to the word being typed: letters, digits, brackets, Backspace.

use super::out::Out;
use super::{keymap, Session};
use crate::compose::diff::diff;
use crate::compose::parse::Role;
use crate::compose::{Display, RawKey, MAXK};
use crate::data::keys;
use crate::utils;

impl Session {
    /// Result for a change of the shown word. A plain typed letter is left to the platform
    /// (pass-through); everything else rewrites the tail.
    fn emit(prev: &Display, next: &Display, typed: char) -> Out {
        let e = diff(prev, next);
        if e.backspace == 0 && e.count == 1 && e.chars[0] == typed {
            return Out::none();
        }
        Out::send(e.backspace, &e.chars[..e.count as usize])
    }

    pub(super) fn on_word_key(&mut self, key: u16, caps: bool, shift: bool) -> Out {
        // After Backspace brought a committed word back: a fresh letter starts a new word, a
        // modifier edits the restored one.
        if self.restored_pending_clear && keys::is_letter(key) {
            let is_modifier = keymap::raw_key(key, false).is_some_and(|k| {
                crate::compose::method::intent(self.options().method, k.ch)
                    != crate::compose::method::Intent::Plain
            });
            // "duơ" restored: a final consonant completes ươ instead of starting a new word
            let best = self.word.best();
            let u = best.units();
            let pending_uo = u.len() >= 2
                && u[u.len() - 2].ch == b'u'
                && u[u.len() - 2].md == crate::phonology::Mod::None
                && u[u.len() - 1].ch == b'o'
                && u[u.len() - 1].md == crate::phonology::Mod::Horn;
            let clear = !pending_uo
                && if self.restored_is_ascii {
                    !is_modifier
                } else {
                    keys::is_consonant(key) && !is_modifier
                };
            if clear {
                self.clear();
            }
            self.restored_pending_clear = false;
            self.restored_is_ascii = false;
        }
        if self.word.is_empty() && keys::is_letter(key) && self.has_non_letter_prefix {
            self.has_non_letter_prefix = false;
        }

        // auto-capitalize: the first letter after ". " is forced to upper case
        let was_auto = self.pending_capitalize && keys::is_letter(key) && !caps;
        let eff_caps = if self.pending_capitalize && keys::is_letter(key) {
            self.pending_capitalize = false;
            self.saw_sentence_ending = false;
            self.auto_capitalize_used = true;
            true
        } else {
            if self.pending_capitalize && keys::is_number(key) {
                self.pending_capitalize = false;
                self.saw_sentence_ending = false;
                self.auto_capitalize_used = false;
            }
            if self.saw_sentence_ending && keys::is_letter(key) {
                self.saw_sentence_ending = false;
            }
            caps
        };

        if let Some(r) = self.shifted_case_segment(key, eff_caps, shift) {
            return r;
        }

        let typed = utils::key_to_char(key, caps).unwrap_or('\0');
        let out = self.type_key(key, eff_caps, typed);
        // the platform would type a lower-case letter: if we forced a capital, say so
        if was_auto && out.is_none() && self.word.len() == 1 {
            if let Some(ch) = utils::key_to_char(key, true) {
                return Out::send(0, &[ch]);
            }
        }
        out
    }

    fn type_key(&mut self, key: u16, eff_caps: bool, typed: char) -> Out {
        let Some(raw) = keymap::raw_key(key, eff_caps).filter(|_| keymap::is_word_key(key)) else {
            // not a letter/digit: mark the word so shortcuts do not fire on "149k"
            self.has_non_letter_prefix = true;
            return Out::none();
        };
        let prev = self.screen;
        if !self.word.push(raw) {
            // longer than the typing core tracks: let the rest pass through untouched
            self.word.clear();
            self.screen = Display::empty();
            return Out::none();
        }
        if self.free_tone && !self.word.alive() {
            if let Some(out) = self.cut_run(&prev) {
                return out;
            }
        }
        let next = self.word.display();
        self.screen = next;
        Self::emit(&prev, &next, typed)
    }

    /// Free typing, several syllables typed without a space (xinchaof, thuwrgoxTieengsVieetj):
    /// when the word has no Vietnamese reading left, split it into finished syllables. Marks and
    /// modifiers then only reach the syllable being typed. Left alone (`None`) when the letters
    /// start an English word (auto-restore on) or cannot be split into syllables.
    fn cut_run(&mut self, prev: &Display) -> Option<Out> {
        let n = self.word.len();
        let mut keys = [RawKey { ch: 0, caps: false }; MAXK];
        keys[..n].copy_from_slice(self.word.raw());
        if self.english_restore {
            let raw: String = keys[..n].iter().map(|k| k.ch as char).collect();
            if crate::data::lexicon::EN.has_prefix_lower(&raw) {
                return None;
            }
        }
        let opts = self.options();
        let probe = |part: &[RawKey]| {
            let mut c = crate::compose::Compose::new(opts);
            for k in part {
                c.push(*k);
            }
            c
        };
        let mut closed: Vec<char> = Vec::new();
        let mut start = 0;
        while !probe(&keys[start..n]).alive() {
            let end = (start + 1..n)
                .rev()
                .find(|&e| probe(&keys[start..e]).finished())?;
            closed.extend_from_slice(probe(&keys[start..end]).display().as_slice());
            start = end;
        }
        if start == 0 {
            return None;
        }
        self.word.clear();
        for k in &keys[start..n] {
            self.word.push(*k);
        }
        self.screen = self.word.display();
        closed.extend_from_slice(self.screen.as_slice());
        Some(Out::send(prev.len, &closed))
    }

    /// Telex: a capital after a lower-case run starts a new case segment (`useEffect`): the
    /// finished segment is kept (or restored to raw if it spells an English word).
    fn shifted_case_segment(&mut self, key: u16, eff_caps: bool, shift: bool) -> Option<Out> {
        let starts = self.options().method == crate::compose::Method::Telex
            && shift
            && eff_caps
            && keys::is_letter(key)
            && !self.word.is_empty()
            && self.word.raw().last().is_some_and(|k| !k.caps);
        if !starts {
            return None;
        }
        let new_key = keymap::raw_key(key, eff_caps)?;
        let raw_lower: String = self.word.raw().iter().map(|k| k.ch as char).collect();
        let english =
            self.english_restore && crate::data::english_dict::is_english_word(&raw_lower);
        if english {
            let mut keys_now: Vec<RawKey> = self.word.raw().to_vec();
            keys_now.push(new_key);
            let backspace = self.screen.len;
            self.word.clear();
            for k in &keys_now {
                self.word.push_with(*k, true);
            }
            self.screen = self.word.display();
            let chars: Vec<char> = self.screen.as_slice().to_vec();
            return Some(Out::send(backspace, &chars));
        }
        self.word.clear();
        self.word.push(new_key);
        self.screen = self.word.display();
        Some(Out::none())
    }

    pub(super) fn on_delete(&mut self) -> Out {
        // Backspace after a committed word: each deleted space counts, the last one brings the word back
        if self.spaces_after_commit > 0 && self.word.is_empty() {
            self.spaces_after_commit -= 1;
            if self.spaces_after_commit == 0 {
                self.resume_last_word();
            }
            return Out::send(1, &[]);
        }
        if self.word.is_empty() {
            self.has_non_letter_prefix = true;
        } else {
            // the platform removes one character; find the typing state that shows the same
            self.word.backspace_char();
            self.screen = if self.word.is_empty() {
                Display::empty()
            } else {
                self.word.display()
            };
        }
        if self.word.is_empty() {
            // chain restore: a fully deleted restored word lets the previous one come back
            if self.restored_pending_clear && !self.history.is_empty() {
                self.spaces_after_commit = 1;
            }
            self.restored_pending_clear = false;
            if self.auto_capitalize_used {
                self.pending_capitalize = true;
                self.auto_capitalize_used = false;
            }
        }
        Out::none()
    }

    /// Put the last committed word back in the typing core.
    fn resume_last_word(&mut self) {
        let Some(e) = self.history.pop() else { return };
        self.word.clear();
        for k in &e.keys[..e.len as usize] {
            self.word.push_with(*k, e.literal);
        }
        // the committed text and a replay can differ (restored words); trust the screen
        if self.word.display() != e.shown {
            self.word.clear();
            for k in &e.keys[..e.len as usize] {
                self.word.push_with(*k, true);
            }
        }
        self.screen = self.word.display();
        self.restored_pending_clear = true;
    }

    /// Telex `]` → ư, `[` → ơ (when enabled); a second press types the bracket itself.
    pub(super) fn try_bracket(&mut self, key: u16, caps: bool) -> Option<Out> {
        if !self.bracket_shortcut {
            return None;
        }
        let ch = if key == keys::RBRACKET { b']' } else { b'[' };
        let prev = self.screen;
        if !self.word.push(RawKey { ch, caps }) {
            return None;
        }
        let i = self.word.len() - 1;
        let role = self.word.best().role(i);
        if role != Role::Bracket && role != Role::Revert {
            self.word.pop(); // not Vietnamese here: the bracket is an ordinary break key
            return None;
        }
        let next = self.word.display();
        self.screen = next;
        let e = diff(&prev, &next);
        Some(Out::send_consumed(
            e.backspace,
            &e.chars[..e.count as usize],
        ))
    }
}
