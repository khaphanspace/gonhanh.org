//! English restore at a word boundary: one decision, a short list of rows, evaluated in order.
//!
//! Most English words never need it: a Vietnamese reading that cannot become Vietnamese is
//! pruned while typing, so the raw letters are already on screen. What is left for this table
//! are readings that *stay* alive until the word ends.
//!
//! Constitution I13: a finished Vietnamese reading is never restored, except the two shapes the
//! docs name (a trailing `w` on a known English word, a loan initial `p` on one).

use crate::compose::parse::Role;
use crate::compose::{Compose, Display, MAXK};
use crate::data::lexicon::{DOUBLES, EN, KEEP, VI};
use crate::phonology::{validate, Mod, Opts, Unit, Validity};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Decision {
    Keep,
    Raw,
    /// the raw letters with stretched runs squeezed back to an English word (chooose → choose)
    Squeezed(Display),
}

/// A word on the stack (every char is at most 4 bytes): the restore decision runs at each word
/// boundary and must not touch the heap.
struct Text {
    buf: [u8; 4 * MAXK],
    n: usize,
}

impl Text {
    fn new() -> Self {
        Text {
            buf: [0; 4 * MAXK],
            n: 0,
        }
    }

    fn from_chars(chars: impl Iterator<Item = char>) -> Self {
        let mut t = Text::new();
        for c in chars {
            t.n += c.encode_utf8(&mut t.buf[t.n..]).len();
        }
        t
    }

    fn as_str(&self) -> &str {
        std::str::from_utf8(&self.buf[..self.n]).unwrap_or("")
    }
}

/// `raw` with every run of 3+ equal letters cut to `to` letters, when that is an English word.
fn squeeze(c: &Compose, to: usize) -> Option<Display> {
    let keys = c.raw();
    // no stretch, nothing to squeeze (the common case)
    if !keys
        .windows(3)
        .any(|w| w[0].ch == w[1].ch && w[1].ch == w[2].ch)
    {
        return None;
    }
    let mut out = Display::empty();
    let mut word = Text::new();
    let mut run = 0;
    for (k, key) in keys.iter().enumerate() {
        run = if k > 0 && keys[k - 1].ch == key.ch {
            run + 1
        } else {
            1
        };
        let long_run =
            run >= 3 || keys[k + 1..].iter().take_while(|n| n.ch == key.ch).count() + run >= 3;
        if !long_run || run <= to {
            let ch = if key.caps {
                (key.ch as char).to_ascii_uppercase()
            } else {
                key.ch as char
            };
            out.push(ch);
            word.n += (key.ch as char).encode_utf8(&mut word.buf[word.n..]).len();
        }
    }
    (out.len as usize != keys.len() && word.n >= 3 && EN.contains_lower(word.as_str()))
        .then_some(out)
}

fn is_vowel_byte(b: u8) -> bool {
    matches!(b, b'a' | b'e' | b'i' | b'o' | b'u' | b'y')
}

/// `units` with every run of `min`+ equal letters cut to one.
fn collapse_runs(units: &[Unit], min: usize) -> ([Unit; MAXK], usize) {
    let mut out = [Unit::new(0, Mod::None); MAXK];
    let (mut n, mut k) = (0, 0);
    while k < units.len() {
        let run = units[k..].iter().take_while(|u| **u == units[k]).count();
        let keep = if run >= min { 1 } else { run };
        for _ in 0..keep {
            out[n] = units[k];
            n += 1;
        }
        k += run;
    }
    (out, n)
}

/// `enabled` is the user's English auto-restore setting.
pub fn decide(c: &Compose, enabled: bool, foreign: bool) -> Decision {
    if !enabled || c.is_empty() || !c.kept_transformed() {
        return Decision::Keep; // row 1-2: feature off, or nothing was transformed
    }
    let raw_text = Text::from_chars(c.raw().iter().map(|k| k.ch as char));
    let shown_text = Text::from_chars(c.kept_display().as_slice().iter().copied());
    let (raw, shown) = (raw_text.as_str(), shown_text.as_str());
    let p = c.kept();
    let units = p.units();

    // row 3: words that must stay as typed
    if KEEP.contains_lower(shown) || KEEP.contains_lower(raw) {
        return Decision::Keep;
    }
    // row 3b: stretched typing of an English word (chooose, assssess)
    if let Some(d) = squeeze(c, 2) {
        return Decision::Squeezed(d);
    }
    // row 3c: docs §10.5 — a word starting wr / wh is English
    if raw.starts_with("wr") || raw.starts_with("wh") {
        return Decision::Raw;
    }
    let valid_end = |units: &[Unit]| {
        validate(
            units,
            p.tone,
            &Opts {
                foreign_initials: foreign,
                free: c.options().free,
                names: true,
                lenient: true,
                at_end: true,
            },
        )
    };
    // row 4: stretched letters ("ơiiiii", "vàooooo") are casual Vietnamese when the word is a
    // Vietnamese syllable once every stretch is squeezed to one letter
    let (squeezed, sn) = collapse_runs(units, 3);
    if sn < units.len() && valid_end(&squeezed[..sn]) == Validity::Complete {
        return Decision::Keep;
    }
    let has_stroke = units.iter().any(|u| u.stroke);
    let validity = valid_end(units);
    // looked up only by the rows that need it
    let raw_en = || EN.contains_lower(raw);

    // row 6: a finished Vietnamese reading wins when the dictionary knows it; an English word whose
    // Vietnamese reading is unattested (bore → boẻ, sims → sím, wi → ưi) goes back to raw
    if validity == Validity::Complete && !p.reverted {
        // short words (er, aus, wn, wo) are far more often Vietnamese interjections than English
        let english_shape =
            raw.len() >= 3 && !raw.starts_with('w') && !is_vowel_byte(raw.as_bytes()[0]);
        // an unattested syllable reached by changing the tone (arts → ảt → át), or a loan initial
        // p (pais, pes): English wears these shapes far more often than Vietnamese does
        let tone_changed = p.roles[..raw.len()]
            .iter()
            .filter(|r| **r == Role::Tone)
            .count()
            >= 2;
        let loan_p = raw.starts_with('p') && !raw.starts_with("ph") && raw.len() >= 3;
        let english = tone_changed || loan_p || (english_shape && raw_en());
        return if english && !VI.contains_lower(shown) {
            Decision::Raw
        } else {
            Decision::Keep
        };
    }
    // row 7: a cancelled modifier (mass, off, buss, homoeopathy). A doubled s/f/vowel is how
    // English is spelled in Telex, so the raw spelling wins when it is a word; otherwise the
    // cancelled form stays when it is a word (dissconnect → disconnect) or is what the user
    // meant (taxxi → taxi). r / x / j typed twice mean the letter itself (varr → var, hajj → haj).
    if p.reverted {
        let letter_key = p.roles[..raw.len()]
            .iter()
            .position(|r| *r == Role::Revert)
            .map(|k| raw.as_bytes()[k]);
        let tone_letter = matches!(letter_key, Some(b'r' | b'x' | b'j'));
        if EN.contains_lower(shown) || KEEP.contains_lower(shown) {
            return Decision::Keep;
        }
        // the cancel came late (ararat → arat): letters went missing, it was never a double-press
        if p.late_revert {
            return Decision::Raw;
        }
        return if DOUBLES.contains(raw) || (!tone_letter && raw_en()) {
            Decision::Raw
        } else {
            Decision::Keep
        };
    }
    // row 8: stroke abbreviations (đc, đt)
    if has_stroke && (units.iter().all(|u| !u.is_vowel()) || VI.contains_lower(shown)) {
        return Decision::Keep;
    }
    // row 9: a lone w typed as ư with only consonants after it (wc → ưc) stays Vietnamese
    let only_w_vowel = !raw
        .bytes()
        .skip(1)
        .any(|b| matches!(b, b'a' | b'e' | b'i' | b'o' | b'u' | b'y'))
        && raw.starts_with('w');
    let coda_like = matches!(&raw[1..], "c" | "ch" | "m" | "n" | "ng" | "nh" | "p" | "t");
    if only_w_vowel && coda_like {
        return Decision::Keep;
    }
    // row 9b: one vowel with its mark (ả, ồ, ầ) is an interjection
    if units.len() == 1 && units[0].is_vowel() && !raw_en() {
        return Decision::Keep;
    }
    // row 9c: a valid syllable with its last consonant stretched (chứcc) is emphasis
    let n = units.len();
    if n >= 3
        && units[n - 1] == units[n - 2]
        && !units[n - 1].is_vowel()
        && p.strong_intent(raw.len())
        && valid_end(&units[..n - 1]) == Validity::Complete
    {
        return Decision::Keep;
    }
    // row 9d: a circumflex that arrived late after a consonant (toto → tôt) is not a deliberate
    // double vowel (teep → têp); a syllable that still lacks its tone is then English
    let delayed_circ = p.roles[..raw.len()]
        .iter()
        .enumerate()
        .any(|(k, r)| *r == Role::Circ && k > 0 && c.raw()[k - 1].ch != c.raw()[k].ch);
    if validity == Validity::Prefix && delayed_circ {
        return Decision::Raw;
    }
    // row 10: an incomplete or invalid reading at the end of the word goes back to the raw
    // letters; only a Vietnamese word that could still be finished by a tone (teep → têp) stays
    if validity == Validity::Prefix && !raw_en() {
        Decision::Keep
    } else {
        Decision::Raw
    }
}
