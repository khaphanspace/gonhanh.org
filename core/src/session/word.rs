//! Keys that belong to the word being typed: letters, digits, brackets, Backspace.

use super::out::Out;
use super::restore::{decide, Decision};
use super::{keymap, Session};
use crate::compose::diff::diff;
use crate::compose::parse::Role;
use crate::compose::render::render_raw;
use crate::compose::{Display, RawKey, MAXK};
use crate::data::keys;
use crate::utils;

fn is_vowel_key(k: RawKey) -> bool {
    matches!(k.ch, b'a' | b'e' | b'i' | b'o' | b'u' | b'y')
}

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
        let mut next = self.word.display();
        // A reading that only free typing allows (west → wét) is shown as typed while the letters
        // still begin an English word: no mark appears and disappears again at the space.
        // A doubled key is a cancel typed on purpose: the word goes back to the plain letters
        // (perr → per), whatever the dictionary says.
        if self.free_tone
            && self.word.needs_free_typing()
            && !self.word.cancelled()
            && self.begins_english_word()
        {
            next = render_raw(self.word.raw());
        }
        self.screen = next;
        Self::emit(&prev, &next, typed)
    }

    /// The typed letters as text, on the stack.
    fn typed_letters<'a>(&self, buf: &'a mut [u8; MAXK]) -> &'a str {
        let raw = self.word.raw();
        for (b, k) in buf.iter_mut().zip(raw) {
            *b = k.ch;
        }
        std::str::from_utf8(&buf[..raw.len()]).unwrap_or("")
    }

    /// The typed letters are one English word or the start of one (a single lookup).
    fn begins_english_word(&self) -> bool {
        let mut buf = [0u8; MAXK];
        {
            let text = self.typed_letters(&mut buf);
            crate::data::lexicon::EN.begins_inflected(text)
                || crate::data::lexicon::REF.begins_inflected(text)
        }
    }

    /// The typed letters are English: one word, the start of one, or words stuck together
    /// (helloworld, helloworl). Parts are four letters or more, so short Vietnamese syllables are
    /// not mistaken for English words.
    pub(super) fn is_english(&self, compound_only: bool) -> bool {
        use crate::data::lexicon::{EN, REF};
        let mut buf = [0u8; MAXK];
        let text = self.typed_letters(&mut buf);
        let n = text.len();
        // the common case first: one word, or the start of one
        if !compound_only && (EN.begins_inflected(text) || REF.begins_inflected(text)) {
            return true;
        }
        let min_tail = if compound_only { 3 } else { 1 };
        // ok[i]: the letters from i on are English
        let mut ok = [false; MAXK + 1];
        ok[n] = true;
        for i in (0..n).rev() {
            // While typing, the letters after a word may be the start of the next one (dennisd).
            // At the end of the word an unfinished tail counts only with three letters: rr, xx or
            // ww are the start of some entry, not evidence of a word.
            ok[i] = (n - i >= min_tail && EN.has_prefix_lower(&text[i..]))
                || (i + 3..=n).any(|j| ok[j] && EN.contains_lower(&text[i..j]));
        }
        if compound_only {
            // at least two parts: a single word is the restore table's business
            return (3..n).any(|j| ok[j] && EN.contains_lower(&text[..j]));
        }
        ok[0]
    }

    /// Free typing, several syllables typed without a space (xinchaof, thuwrgoxTieengsVieetj):
    /// when the word has no Vietnamese reading left, split it into finished syllables. Marks and
    /// modifiers then only reach the syllable being typed. Left alone (`None`) when the letters
    /// start an English word (auto-restore on) or cannot be split into syllables.
    fn cut_run(&mut self, prev: &Display) -> Option<Out> {
        // an English word is never split, whether or not auto-restore is on
        if self.is_english(false) {
            return None;
        }
        let n = self.word.len();
        let mut keys = [RawKey { ch: 0, caps: false }; MAXK];
        keys[..n].copy_from_slice(self.word.raw());
        let free = self.options();
        let split = self.split_into_syllables(&keys[..n]);
        // the word is the probe: put it back to the typed keys, or to the last syllable
        let start = split.as_ref().map_or(0, |(_, start)| *start);
        self.word.update_options(free);
        self.word.clear();
        for k in &keys[start..n] {
            self.word.push(*k);
        }
        let (mut closed, _) = split?;
        self.screen = self.word.display();
        closed.extend_from_slice(self.screen.as_slice());
        Some(Out::send(prev.len, &closed))
    }

    /// The text of the finished syllables of `keys` and where the last one starts. Free typing
    /// relaxes the beginning of the word only (khphá): every later syllable must be Vietnamese as
    /// it stands, and a closed syllable has at least two letters. Otherwise English words stuck
    /// together (helloworld) would be chopped into pieces that merely look like syllables. Uses
    /// the word itself as the probe (no second typing core); the caller restores it.
    fn split_into_syllables(&mut self, keys: &[RawKey]) -> Option<(Vec<char>, usize)> {
        let free = self.options();
        let native = crate::compose::Options {
            free: false,
            foreign_initials: false,
            ..free
        };
        let (english, foreign) = (self.english_restore, self.foreign_initials());
        let load = |w: &mut crate::compose::Compose, part: &[RawKey], first: bool| {
            w.update_options(if first { free } else { native });
            w.clear();
            for k in part {
                w.push(*k);
            }
        };
        let mut closed: Vec<char> = Vec::new();
        let mut start = 0;
        loop {
            load(&mut self.word, &keys[start..], start == 0);
            if self.word.alive() {
                break;
            }
            let end = (start + 1..keys.len()).rev().find(|&e| {
                // A vowel next to a vowel is one nucleus typed in a odd order (tioo), not two
                // syllables: a syllable boundary has a consonant on one side.
                if is_vowel_key(keys[e - 1]) && is_vowel_key(keys[e]) {
                    return false;
                }
                load(&mut self.word, &keys[start..e], start == 0);
                self.word.finished() && self.word.display().len >= 2
            })?;
            load(&mut self.word, &keys[start..end], start == 0);
            // a closed syllable gets the verdict it would get at a space
            let text = match decide(&self.word, english, foreign, || false) {
                Decision::Keep => self.word.kept_display(),
                Decision::Raw => render_raw(self.word.raw()),
                Decision::Squeezed(d) => d,
            };
            closed.extend_from_slice(text.as_slice());
            start = end;
        }
        (start > 0).then_some((closed, start))
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
