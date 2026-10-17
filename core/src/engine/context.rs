//! Which language the sentence is in. A word that spells English and is also a Vietnamese
//! syllable (is → í, was → ứa, bars → bả) is decided by its neighbours: after English words it is
//! English, otherwise Vietnamese stays the default.
//!
//! The run opens on an English function word (the, and, with) and on nothing else: Vietnamese
//! text borrows English nouns and verbs all the time ("update report", "expect, deadline"), but
//! it does not borrow articles and prepositions, so only a function word proves the sentence is
//! English. It stays open through further English words and words that say nothing (names, loan
//! spellings), and closes when a word is committed as Vietnamese or the text is left (Enter,
//! cursor moves).

use crate::compose::MAXK;
use crate::data::lexicon::{EN, FUNC, VI};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WordKind {
    /// English function word that cannot be read as a Vietnamese word typed without marks
    Function,
    /// committed with Vietnamese marks
    Vietnamese,
    /// says nothing about the language (English nouns and verbs, names, defaults)
    Neutral,
}

#[derive(Clone, Copy, Default)]
pub struct EnglishRun {
    open: bool,
}

impl EnglishRun {
    pub const fn new() -> Self {
        EnglishRun { open: false }
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn clear(&mut self) {
        *self = Self::new();
    }

    /// Fold in a word committed with these raw keys; `vietnamese` when what stays on screen is
    /// not the letters as typed. Runs at every word boundary, so it looks nothing up that it
    /// does not need: the function-word list (no allocation), and the English list only to
    /// tell a default from a choice while the run is open.
    pub fn note_word(&mut self, raw: &[u8], vietnamese: bool) {
        let kind = self.classify(raw, vietnamese);
        match kind {
            WordKind::Function => self.open = true,
            WordKind::Vietnamese => self.clear(),
            WordKind::Neutral => {}
        }
    }

    fn classify(&self, raw: &[u8], vietnamese: bool) -> WordKind {
        let letters =
            raw.len() >= 2 && raw.len() <= MAXK && raw.iter().all(u8::is_ascii_alphabetic);
        let word = if letters {
            std::str::from_utf8(raw).ok()
        } else {
            None
        };
        let Some(word) = word else {
            return if vietnamese {
                WordKind::Vietnamese
            } else {
                WordKind::Neutral
            };
        };
        if vietnamese {
            // letters that spell an English word and were read as Vietnamese (this → thí) were a
            // default, not a choice: they say nothing, so they neither open nor close the run
            return if self.open && (FUNC.contains_lower(word) || EN.knows_inflected(word)) {
                WordKind::Neutral
            } else {
                WordKind::Vietnamese
            };
        }
        // a short word that is also a toneless Vietnamese word (to, do, an, he) is not evidence
        if FUNC.contains_lower(word) && (raw.len() >= 3 || !VI.contains_lower(word)) {
            WordKind::Function
        } else {
            WordKind::Neutral
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(words: &[(&str, bool)]) -> bool {
        let mut r = EnglishRun::new();
        for (w, vn) in words {
            r.note_word(w.as_bytes(), *vn);
        }
        r.is_open()
    }

    #[test]
    fn a_function_word_opens_the_run() {
        assert!(run(&[("the", false)]));
        assert!(run(&[("with", false)]));
        assert!(run(&[("is", false)]));
    }

    #[test]
    fn a_short_word_that_is_also_vietnamese_does_not() {
        assert!(!run(&[("do", false)]));
        assert!(!run(&[("to", false)]));
    }

    #[test]
    fn english_content_words_do_not_open_it() {
        assert!(!run(&[("suggestion", false)]));
        assert!(!run(&[("update", false), ("report", false)]));
        assert!(run(&[("the", false), ("update", false), ("report", false)]));
    }

    #[test]
    fn a_vietnamese_word_closes_it() {
        assert!(!run(&[("the", false), ("tooi", true)]));
        // spelled like English but read as Vietnamese: no choice was made, the run stays
        assert!(run(&[("the", false), ("this", true)]));
        assert!(run(&[("the", false), ("xyzq", false)]));
    }
}
