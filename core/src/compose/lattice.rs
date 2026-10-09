//! The beam: a few live interpretations of the word, pruned by phonology, best one shown.
//!
//! Ranking (highest wins): a finished syllable beats a still-typing one, then a free-typing
//! (grammarless) one, then more modifiers consumed, then earlier-created (stable display).
//! When no live reading is Vietnamese the display falls back to the "keep" reading (every mark
//! that was valid when typed), or to the letters as typed when English auto-restore is on and
//! nothing proves Vietnamese intent: English words stay untouched without a "restore" step.

use super::parse::{extend, Children, Parse, Role};
use super::render::{render, Display};
use super::{Options, RawKey, MAXK};
use crate::phonology::{nucleus_len, tone_index, validate, Mod, Opts as Pho, Tone, Validity};
use std::cell::{Cell, RefCell};

const BEAM: usize = 8;

#[derive(Clone, Copy)]
struct Beam {
    live: [Parse; BEAM],
    n: usize,
    /// the best live reading is only a name prefix: show `keep` instead
    weak: bool,
}

impl Beam {
    const fn start() -> Self {
        Beam {
            live: [Parse::empty(); BEAM],
            n: 1,
            weak: false,
        }
    }
}

pub struct Compose {
    opts: Options,
    raw: [RawKey; MAXK],
    len: usize,
    /// beams[k] = interpretations after k keys; kept for backspace
    beams: [Beam; MAXK + 1],
    /// scratch for the children of one key (allocated once, reused every key)
    kids: RefCell<Box<Children>>,
    /// `keeps[k]` = the "keep" reading after k keys: every modifier that was valid when typed is
    /// kept, later letters are appended as typed (tẽt, ăi). Shown when no live reading is
    /// Vietnamese, so it is only computed (lazily, once per key) for words that need it.
    keeps: RefCell<[Parse; MAXK + 1]>,
    /// `keeps[..=keep_upto]` are valid for the keys typed so far
    keep_upto: Cell<usize>,
    /// after a Backspace over a word shown as typed: keep showing the letters as typed
    pin_literal: Cell<bool>,
}

impl Compose {
    pub fn new(opts: Options) -> Self {
        Compose {
            opts,
            raw: [RawKey { ch: 0, caps: false }; MAXK],
            len: 0,
            beams: [Beam::start(); MAXK + 1],
            kids: RefCell::new(Box::new(Children::new())),
            keeps: RefCell::new([Parse::empty(); MAXK + 1]),
            keep_upto: Cell::new(0),
            pin_literal: Cell::new(false),
        }
    }

    pub fn options(&self) -> &Options {
        &self.opts
    }

    /// Change settings without dropping the word (existing readings keep their shape; the
    /// display uses the new tone style immediately).
    pub fn update_options(&mut self, opts: Options) {
        self.opts = opts;
    }

    pub fn clear(&mut self) {
        self.len = 0;
        self.keep_upto.set(0);
        self.pin_literal.set(false);
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn raw(&self) -> &[RawKey] {
        &self.raw[..self.len]
    }

    /// Add a key. `false` when the word is full (caller shows the key literally).
    pub fn push(&mut self, key: RawKey) -> bool {
        self.push_with(key, false)
    }

    /// Like `push`; with `literal_only` the key is only ever a letter (used to replay a word that
    /// was restored to its raw spelling).
    pub fn push_with(&mut self, key: RawKey, literal_only: bool) -> bool {
        if self.len == MAXK {
            return false;
        }
        self.pin_literal.set(false);
        if literal_only {
            // a replayed word has no other reading: its keep chain is the letters themselves
            self.ensure_keep();
            self.raw[self.len] = key;
            let lit = self.literal_parse(self.len + 1);
            let next = &mut self.beams[self.len + 1];
            next.live[0] = lit;
            next.n = 1;
            next.weak = false;
            self.keeps.get_mut()[self.len + 1] = lit;
            self.keep_upto.set(self.len + 1);
        } else {
            let (done, todo) = self.beams.split_at_mut(self.len + 1);
            step(
                &done[self.len],
                &mut todo[0],
                self.len,
                key,
                &self.opts,
                self.kids.get_mut(),
            );
            self.keep_upto.set(self.keep_upto.get().min(self.len));
        }
        self.raw[self.len] = key;
        self.len += 1;
        true
    }

    /// The first `upto` keys exactly as typed (never pruned).
    fn literal_parse(&self, upto: usize) -> Parse {
        let mut p = Parse::empty();
        for (i, k) in self.raw[..upto].iter().enumerate() {
            p.push_literal(i, k.ch);
        }
        p
    }

    /// Bring `keeps` up to the current key.
    fn ensure_keep(&self) {
        let mut up = self.keep_upto.get();
        if up >= self.len {
            return;
        }
        let pho = Pho {
            foreign_initials: self.opts.foreign_initials,
            free: self.opts.free,
            names: false,
            lenient: true,
            at_end: false,
        };
        let mut keeps = self.keeps.borrow_mut();
        let mut kids = self.kids.borrow_mut();
        while up < self.len {
            let next = step_keep(&keeps[up], up, self.raw[up], &self.opts, &pho, &mut kids);
            keeps[up + 1] = next;
            up += 1;
        }
        self.keep_upto.set(up);
    }

    /// Backspace inside the word: back to exactly the state after one key less.
    pub fn pop(&mut self) {
        self.len = self.len.saturating_sub(1);
    }

    /// Delete the last *displayed* character the way the platform does (the key is not undone):
    /// go back to the state whose display is the current one minus its last char. `false` when no
    /// earlier state matches (the word can no longer be edited).
    pub fn backspace_char(&mut self) -> bool {
        let shown = self.display();
        // The word is on screen exactly as typed (English guard): one key is one letter, and it
        // stays as typed until the next key (serv⌫ shows ser, not the sẻ hiding behind it).
        if self.len > 0 && shown == render(&self.literal_parse(self.len), self.raw(), &self.opts) {
            self.len -= 1;
            self.pin_literal.set(true);
            return true;
        }
        let want = &shown.as_slice()[..shown.as_slice().len().saturating_sub(1)];
        self.ensure_keep();
        let keeps = *self.keeps.get_mut();
        for back in 1..=self.len.min(3) {
            let k = self.len - back;
            let raw = &self.raw[..k];
            // any live reading of that state may be the one the screen now matches (a compound
            // ươ that lost a rank to uơ still shows as the shorter text)
            let beam = &mut self.beams[k];
            let at = (0..beam.n)
                .find(|&j| render(&beam.live[j], raw, &self.opts).as_slice() == want)
                .or_else(|| {
                    ((beam.n == 0 || beam.weak)
                        && render(&keeps[k], raw, &self.opts).as_slice() == want)
                        .then_some(usize::MAX)
                });
            match at {
                Some(usize::MAX) => {
                    self.len = k;
                    return true;
                }
                Some(j) => {
                    beam.live.swap(0, j);
                    self.len = k;
                    return true;
                }
                None => {}
            }
        }
        // No earlier state shows that text (the deleted letter came from a key typed earlier, or
        // carried the tone): drop the key that made it and replay the rest.
        if let Some(text) = self.replay_without_last_letter(want) {
            return text;
        }
        self.len = 0;
        want.is_empty()
    }

    fn replay_without_last_letter(&mut self, want: &[char]) -> Option<bool> {
        let p = self.best();
        let last = p.n.checked_sub(1)? as usize;
        let drop = p.unit_key[last] as usize;
        let keys = self.raw;
        let n = self.len;
        let carries_tone =
            p.tone != Tone::Ngang && tone_index(p.units(), self.opts.modern_tone) == Some(last);
        self.len = 0;
        for (k, key) in keys[..n].iter().enumerate() {
            let tone_key = carries_tone && matches!(p.role(k), Role::Tone | Role::Swallow);
            if k != drop && !tone_key {
                self.push(*key);
            }
        }
        if self.display().as_slice() == want {
            return Some(true);
        }
        self.len = 0;
        None
    }

    /// Whether any key of the best reading acts as a modifier (the display is not the raw typing).
    pub fn transformed(&self) -> bool {
        let p = self.best();
        (0..self.len).any(|k| p.role(k) != Role::Literal)
    }

    /// The reading Vietnamese typing leads to: the best live one, else what was kept.
    pub fn kept(&self) -> Parse {
        if self.pin_literal.get() {
            return self.literal_parse(self.len);
        }
        let b = &self.beams[self.len];
        if b.n == 0 || b.weak {
            self.ensure_keep();
            self.keeps.borrow()[self.len]
        } else {
            b.live[0]
        }
    }

    /// Some reading of the keys typed so far is still Vietnamese (free typing counts).
    pub fn alive(&self) -> bool {
        let b = &self.beams[self.len];
        b.n > 0 && !b.weak
    }

    /// The best reading is Vietnamese only because free typing or foreign initials allow it
    /// (west → wét): weak evidence, which an English word may overrule.
    pub fn needs_free_typing(&self) -> bool {
        if !self.alive() {
            return false;
        }
        let p = self.best();
        let native = Pho {
            foreign_initials: false,
            free: false,
            names: true,
            lenient: true,
            at_end: false,
        };
        validate(p.units(), p.tone, &native) == Validity::Invalid
    }

    /// The best reading is a finished syllable (free typing counts): the word could end here.
    pub fn finished(&self) -> bool {
        if !self.alive() {
            return false;
        }
        let p = self.best();
        let pho = Pho {
            foreign_initials: self.opts.foreign_initials,
            free: self.opts.free,
            names: true,
            lenient: true,
            at_end: true,
        };
        matches!(
            validate(p.units(), p.tone, &pho),
            Validity::Complete | Validity::Loose
        )
    }

    /// The reading on screen. With English auto-restore on, a word with no Vietnamese reading
    /// stays as typed unless it carries a stroke or horn.
    pub fn best(&self) -> Parse {
        if self.pin_literal.get() {
            return self.literal_parse(self.len);
        }
        let b = &self.beams[self.len];
        if b.n == 0 || b.weak {
            let kept = self.kept();
            if self.opts.english_guard && !kept.strong_intent(self.len) {
                return self.literal_parse(self.len);
            }
            return kept;
        }
        b.live[0]
    }

    pub fn kept_display(&self) -> Display {
        render(&self.kept(), self.raw(), &self.opts)
    }

    /// Whether any key of the kept reading acts as a modifier.
    pub fn kept_transformed(&self) -> bool {
        let p = self.kept();
        (0..self.len).any(|k| p.role(k) != Role::Literal)
    }

    pub fn display(&self) -> Display {
        render(&self.best(), self.raw(), &self.opts)
    }

    /// Convenience for tests and tools: type a whole word, return what is shown.
    pub fn type_str(opts: Options, word: &str) -> Option<String> {
        let mut c = Compose::new(opts);
        for ch in word.chars() {
            if !ch.is_ascii_alphanumeric()
                || !c.push(RawKey {
                    ch: ch.to_ascii_lowercase() as u8,
                    caps: ch.is_ascii_uppercase(),
                })
            {
                return None;
            }
        }
        Some(c.display().as_slice().iter().collect())
    }
}

/// Beam after key `i`: every reading of `prev` extended by `key`, pruned by phonology, best first.
fn step(prev: &Beam, next: &mut Beam, i: usize, key: RawKey, o: &Options, kids: &mut Children) {
    let pho = Pho {
        foreign_initials: o.foreign_initials,
        free: o.free,
        names: true,
        lenient: true,
        at_end: false,
    };
    let mut ranks = [i64::MIN; BEAM];
    let mut tn = 0;
    let mut order = 0i64;
    for p in &prev.live[..prev.n] {
        kids.n = 0;
        extend(p, i, key, o, kids);
        let stretch = stretches_toned_word(p, key, &pho);
        // a circumflex that came after the tone (mùa + a → muầ) is only the other reading of a
        // stretch: the next same vowel lengthens the stretch, it does not cancel that circumflex
        let tone_then_circ = i > 0
            && p.roles[i - 1] == Role::Circ
            && p.roles[..i - 1].contains(&Role::Tone)
            && nucleus_len(p.units()) >= 2
            && validate(p.units(), p.tone, &pho) != Validity::Complete;
        for child in &kids.items[..kids.n] {
            if tone_then_circ && child.roles[i] == Role::Revert {
                continue;
            }
            order += 1;
            let validity = validate(child.units(), child.tone, &pho);
            let rank = match validity {
                Validity::Complete => 2,
                Validity::Prefix => 1,
                Validity::NamePrefix => -1,
                // free typing: only when no grammatical reading exists
                Validity::Loose => 0,
                // A cancelled modifier makes the word literal on purpose (aaa → aa, ass → as):
                // never Vietnamese, still kept, ranked below every Vietnamese reading.
                Validity::Invalid
                    if child.reverted && matches!(child.roles[i], Role::Revert | Role::Literal) =>
                {
                    0
                }
                // the same vowel again after a toned word is a stretch (mùaaa)
                Validity::Invalid if stretch && child.roles[i] == Role::Literal => 0,
                Validity::Invalid => continue,
            };
            // Free typing: a lone w is a consonant (wé, wl) unless the ư reading is grammatical
            // (a cancelled ư is a plain w again: nothing to penalize)
            let w_as_vowel_loosely = validity == Validity::Loose
                && !child.reverted
                && child.roles[..=i].contains(&Role::BaseHorn);
            let mut penalty = if w_as_vowel_loosely { 500_000 } else { 0 };
            // a circumflex reading that is not a word yet (muầ, waiting for a coda) stays alive for
            // bafan → bần but is not what is shown while the stretch reading exists; one that is a
            // word (bồ, ấ) wins as it always did
            if stretch && child.roles[i] != Role::Literal && validity != Validity::Complete {
                penalty += 2_000_000;
            }
            // a cancel the user typed on purpose (ww, ddd) outranks a free-typing guess of the same
            // rank, so the word keeps following the cancelled reading (wws → ws, dddd → ddd)
            let cancel_bonus = if child.reverted && rank == 0 {
                500_000
            } else {
                0
            };
            let k = rank * 1_000_000 + child.score as i64 * 100 - order - penalty + cancel_bonus;
            let mut c = *child;
            if validity == Validity::Invalid {
                c.freeze_tone(p, o.modern_tone, c.roles[i] == Role::Revert);
            } else {
                c.release_tone();
            }
            insert(&mut next.live, &mut ranks, &mut tn, k, &c);
        }
    }
    next.n = tn;
    // only a name prefix (rank -1, key near -1_000_000) is weak; a bare literal word has a small negative key
    next.weak = tn > 0 && ranks[0] <= -900_000;
}

/// The key repeats the last vowel of a diphthong in a word that already carries a tone and is a
/// finished syllable (mùa + a) or is already being stretched (mùaa + a). Typing the tone first and the
/// circumflex later (bafan → bần) is possible too, so the circumflex reading is kept in the beam
/// and only the display prefers the stretch.
fn stretches_toned_word(p: &Parse, key: RawKey, pho: &Pho) -> bool {
    let units = p.units();
    let Some(last) = units.last() else {
        return false;
    };
    if p.tone == Tone::Ngang
        || !last.is_vowel()
        || last.ch != key.ch
        || last.md != Mod::None
        || last.stroke
    {
        return false;
    }
    // a lone vowel keeps its extended form (hara → hẩ, afa → ầ); only the end of a diphthong
    // (mùa, hòa) is stretched
    let n = units.len();
    nucleus_len(units) >= 2
        && (units[n - 2] == *last || validate(units, p.tone, pho) == Validity::Complete)
}

/// Extend the "keep" reading: the best Vietnamese child; failing that a modifier whose intent
/// is clear even though the syllable stays odd (a cancel, a delayed horn/breve
/// after the final consonant: tơng, a tone on a word that already carries ư/ơ/â/ă/đ: tắi);
/// else the letter as typed.
fn step_keep(
    prev: &Parse,
    i: usize,
    key: RawKey,
    o: &Options,
    pho: &Pho,
    kids: &mut Children,
) -> Parse {
    let intent = (0..i).any(|k| {
        matches!(
            prev.roles[k],
            Role::Horn | Role::Breve | Role::Circ | Role::Stroke | Role::BaseHorn
        )
    });
    kids.n = 0;
    extend(prev, i, key, o, kids);
    let mut prev_ok: Option<bool> = None;
    let mut best = (i64::MIN, kids.items[0]);
    let mut best_rank = 0;
    for (order, child) in kids.items[..kids.n].iter().enumerate() {
        // a stroke typed after other letters (ded → đe) is a guess the next letter may refute
        if child.roles[i] == Role::Stroke && !strokes_previous_unit(prev, child) {
            continue;
        }
        // a tone key cannot revive a word that already broke (arts: ảt + s is not át): only
        // a word still Vietnamese, or one with a deliberate ư/ơ/â/ă/đ, takes a later tone
        if matches!(child.roles[i], Role::Tone | Role::Swallow) && !intent {
            let broke = *prev_ok.get_or_insert_with(|| {
                !matches!(
                    validate(prev.units(), prev.tone, pho),
                    Validity::Complete | Validity::Prefix | Validity::Loose
                )
            });
            if broke {
                continue;
            }
        }
        let rank = match validate(child.units(), child.tone, pho) {
            Validity::Complete => 3,
            Validity::Prefix | Validity::Loose => 2,
            Validity::Invalid | Validity::NamePrefix => match child.roles[i] {
                Role::Revert | Role::Remove | Role::Bracket => 1,
                Role::Horn | Role::Breve if child.units().last().is_some_and(|u| !u.is_vowel()) => {
                    1
                }
                Role::Tone | Role::Swallow if intent => 1,
                Role::Circ if key.ch.is_ascii_digit() => 1,
                Role::Literal => 0,
                _ => continue,
            },
        };
        let k = rank * 1_000_000 + child.score as i64 * 100 - order as i64;
        if k > best.0 {
            best = (k, *child);
            best_rank = rank;
        }
    }
    let mut next = best.1;
    if best_rank < 2 {
        let cancelled = next.roles[i] == Role::Revert;
        next.freeze_tone(prev, o.modern_tone, cancelled);
    } else {
        next.release_tone();
    }
    next
}

/// The stroke landed on the last letter before the key (dd), not on an earlier d (dod).
fn strokes_previous_unit(prev: &Parse, child: &Parse) -> bool {
    let n = prev.n as usize;
    n > 0
        && child.units()[..n]
            .iter()
            .zip(prev.units())
            .position(|(a, b)| a != b)
            == Some(n - 1)
}

/// Insert keeping `live` sorted best-first by `ranks`, skipping interchangeable duplicates.
fn insert(live: &mut [Parse; BEAM], ranks: &mut [i64; BEAM], tn: &mut usize, k: i64, p: &Parse) {
    for j in 0..*tn {
        if live[j].same_future(p) && ranks[j] >= k {
            return;
        }
    }
    let mut pos = *tn;
    while pos > 0 && ranks[pos - 1] < k {
        pos -= 1;
    }
    if pos >= BEAM {
        return;
    }
    let mut j = (*tn).min(BEAM - 1);
    while j > pos {
        live[j] = live[j - 1];
        ranks[j] = ranks[j - 1];
        j -= 1;
    }
    live[pos] = *p;
    ranks[pos] = k;
    if *tn < BEAM {
        *tn += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compose::Method;

    fn t(word: &str) -> String {
        Compose::type_str(Options::default(), word).unwrap()
    }
    fn v(word: &str) -> String {
        Compose::type_str(
            Options {
                method: Method::Vni,
                ..Options::default()
            },
            word,
        )
        .unwrap()
    }

    #[test]
    fn telex_basics() {
        for (typed, want) in [
            ("as", "á"),
            ("aa", "â"),
            ("aaa", "aa"),
            ("ass", "as"),
            ("dd", "đ"),
            ("ddd", "dd"),
            ("vieetj", "việt"),
            ("vieejt", "việt"),
            ("dduwowcj", "được"),
            ("duowc", "dươc"),
            ("duowcj", "dược"),
            ("huow", "huơ"),
            ("thuongw", "thương"),
            ("tieengs", "tiếng"),
            ("hoaf", "hoà"),
            ("tuanf", "tuàn"),
            ("tuaanf", "tuần"),
            ("nghieeng", "nghiêng"),
            ("Dduwowcj", "Được"),
            ("w", "ư"),
            ("ww", "w"),
            ("nhw", "như"),
        ] {
            assert_eq!(t(typed), want, "{typed}");
        }
    }

    #[test]
    fn vni_basics() {
        for (typed, want) in [
            ("a1", "á"),
            ("a6", "â"),
            ("viet65", "việt"),
            ("d9", "đ"),
            ("duoc75", "dược"),
            ("a66", "a6"),
        ] {
            assert_eq!(v(typed), want, "{typed}");
        }
    }

    #[test]
    fn english_stays_literal() {
        for w in ["clau", "john", "http", "view", "show"] {
            assert_eq!(t(w), w, "{w}");
        }
    }

    #[test]
    fn invalid_vietnamese_keeps_the_marks_typed_while_valid() {
        // restore at the word boundary (session) decides whether English wins
        assert_eq!(t("text"), "tẽt");
        assert_eq!(t("taiw"), "taiw");
        assert_eq!(t("tawis"), "tắi");
    }

    #[test]
    fn names_work_in_any_order() {
        for typed in ["kajn", "kanj", "Kanj"] {
            assert_eq!(t(typed).to_lowercase(), "kạn", "{typed}");
        }
        for typed in ["koong", "kongo", "konog"] {
            assert_eq!(t(typed), "kông", "{typed}");
        }
        assert_eq!(t("Hofong"), "Hồng");
    }

    #[test]
    fn backspace_returns_to_the_prefix_state() {
        let mut c = Compose::new(Options::default());
        for ch in "vieetj".chars() {
            c.push(RawKey {
                ch: ch as u8,
                caps: false,
            });
        }
        c.pop();
        c.pop();
        assert_eq!(c.display().as_slice().iter().collect::<String>(), "viê");
    }
}
