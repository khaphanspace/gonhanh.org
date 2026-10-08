//! Is this letter sequence a Vietnamese syllable? One answer, three states.
//!
//! `Invalid`  no completion can make it Vietnamese.
//! `Prefix`   not complete yet but a valid continuation exists (missing modifier, missing
//!            coda, missing tone on a stop-final syllable, or half of a proper name).
//! `Complete` a finished syllable under the current options.
//!
//! Constitution (docs/vietnamese-language-system.md §4.4, §6.5, §7.6):
//!   I1 structure (C1)(G)V(C2)+T, `qu`/`gi` swallow the u/i · I2 c/k, g/gh, ng/ngh spelling
//!   I3 nucleus whitelist · I4 stop codas take only sắc/nặng · I5 nucleus x coda matrix.
//! The nucleus x coda matrix is generated from the dictionary (tables.rs).

use super::letters::{Mod, Tone, Unit};
use super::tables::{self, *};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Validity {
    Invalid,
    /// Only the beginning of a proper name (kô of kông): kept alive, never shown by itself.
    NamePrefix,
    /// Free typing only: not a Vietnamese syllable, but the letters are in a plausible order
    /// (consonants, vowels, consonants), so the marks the user typed are applied anyway.
    Loose,
    Prefix,
    Complete,
}

#[derive(Clone, Copy, Debug)]
pub struct Opts {
    /// Accept f, j, w, z as single-letter initials (loanwords).
    pub foreign_initials: bool,
    /// Structure only: skip spelling, nucleus, coda and tone rules (user "free tone" mode).
    pub free: bool,
    /// Consult the proper-name lexicon when the regular rules say Invalid.
    pub names: bool,
    /// Typing time: accept rare multi-vowel nuclei with any plain coda (the dictionary is not the
    /// language); restore decisions and the audit use the attested matrix (`false`).
    pub lenient: bool,
    /// The word is finished (a boundary was typed): a nucleus that still needs its coda is
    /// Invalid, not a Prefix.
    pub at_end: bool,
}

impl Default for Opts {
    fn default() -> Self {
        Opts {
            foreign_initials: false,
            free: false,
            names: true,
            lenient: false,
            at_end: false,
        }
    }
}

const MAX_UNITS: usize = 10;

pub fn validate(units: &[Unit], tone: Tone, o: &Opts) -> Validity {
    // Free typing is a fallback: a reading the grammar accepts (wé with w allowed as an initial)
    // always beats one that is only free (ứe).
    let strict = Opts { free: false, ..*o };
    let r = regular(units, tone, &strict);
    if r != Validity::Invalid {
        return r;
    }
    if o.names {
        let n = name_match(units, tone);
        if n != Validity::Invalid {
            return n;
        }
    }
    if o.free && free_structure(units) {
        return Validity::Loose;
    }
    Validity::Invalid
}

/// Letters in a plausible order for free typing: consonants, vowels, consonants (khphá, qcáo, xyzá).
fn free_structure(units: &[Unit]) -> bool {
    const FREE_MAX: usize = 12;
    if units.len() > FREE_MAX || units.iter().any(|u| u.stroke && u.ch != b'd') {
        return false;
    }
    let Some(first) = units.iter().position(|u| u.is_vowel()) else {
        return true; // still typing the initial
    };
    let end = units[first..]
        .iter()
        .position(|u| !u.is_vowel())
        .map_or(units.len(), |k| first + k);
    units[end..]
        .iter()
        .all(|u| !u.is_vowel() && u.md == Mod::None && !u.stroke)
}

/// Letters of the initial as bytes (`D` for a stroked d), at most 3.
fn initial_bytes(part: &[Unit], buf: &mut [u8; 3]) -> Option<usize> {
    if part.len() > 3 {
        return None;
    }
    for (i, u) in part.iter().enumerate() {
        buf[i] = if u.stroke { b'D' } else { u.ch };
    }
    Some(part.len())
}

/// Initials as bytes (`D` stands for đ): a native one is Complete, `q` waits for its `u`.
fn initial_status(ini: &[u8], o: &Opts) -> Validity {
    match ini {
        [] => Validity::Complete,
        [b'f' | b'j' | b'w' | b'z'] if o.foreign_initials => Validity::Complete,
        [b'b' | b'c' | b'd' | b'D' | b'g' | b'h' | b'k' | b'l' | b'm' | b'n' | b'p' | b'r'
        | b's' | b't' | b'v' | b'x']
        | [b'c', b'h']
        | [b'g', b'h' | b'i']
        | [b'k', b'h']
        | [b'n', b'g' | b'h']
        | [b'n', b'g', b'h']
        | [b'p', b'h']
        | [b'q', b'u']
        | [b't', b'h' | b'r'] => Validity::Complete,
        [b'q'] => Validity::Prefix,
        _ => Validity::Invalid,
    }
}

/// (initial length, nucleus start, nucleus end). `qu` and `gi` swallow the u/i that follows when
/// more vowel comes after it (qua, gia); everything after the nucleus is the coda.
pub(super) fn split(units: &[Unit]) -> (usize, usize, usize) {
    let n = units.len();
    let mut i = 0;
    while i < n && !units[i].is_vowel() {
        i += 1;
    }
    let mut ini = i;
    if ini == 1 && i < n && units[i].md == Mod::None {
        let swallow = match (units[0].ch, units[i].ch) {
            (b'q', b'u') => true,
            // gia, giu, gieo — but gii is not gi + i (the tone stays on the first i)
            (b'g', b'i') => i + 1 < n && units[i + 1].is_vowel() && units[i + 1].ch != b'i',
            _ => false,
        };
        if swallow {
            ini = 2;
            i = 2;
        }
    }
    let nuc_start = i;
    while i < n && units[i].is_vowel() {
        i += 1;
    }
    (ini, nuc_start, i)
}

fn regular(units: &[Unit], tone: Tone, o: &Opts) -> Validity {
    let n = units.len();
    if n == 0 {
        return Validity::Prefix;
    }
    if n > MAX_UNITS || units.iter().any(|u| u.stroke && u.ch != b'd') {
        return Validity::Invalid;
    }

    let (ini, nuc_start, nuc_end) = split(units);
    let nuc = &units[nuc_start..nuc_end];
    let coda = &units[nuc_end..];

    let mut buf = [0u8; 3];
    let Some(len) = initial_bytes(&units[..ini], &mut buf) else {
        return Validity::Invalid;
    };
    let ini_chars = &buf[..len];

    // No vowel yet: only the initial can be judged.
    if nuc.is_empty() {
        return match initial_status(ini_chars, o) {
            Validity::Invalid => Validity::Invalid,
            _ => Validity::Prefix, // a valid initial still needs its vowel
        };
    }
    if coda
        .iter()
        .any(|u| u.is_vowel() || u.md != Mod::None || u.stroke)
    {
        return Validity::Invalid;
    }
    if initial_status(ini_chars, o) != Validity::Complete {
        return Validity::Invalid;
    }

    // --- spelling: c/k, g/gh, ng/ngh (I2) ---------------------------------------------------
    if !spelling_ok(ini_chars, nuc) {
        return Validity::Invalid;
    }

    // uya / uyu / ưi only ever follow an initial (khuya, khuỷu, gửi): vi.dic has none without
    if needs_initial(ini_chars, nuc) {
        return Validity::Invalid;
    }

    // --- nucleus whitelist + coda matrix (I3, I5) ---------------------------------------------
    let Some((bit, partial)) = coda_info(coda) else {
        return Validity::Invalid;
    };
    let stop = (bit | partial) & STOPS != 0;
    // Stop codas never carry huyền/hỏi/ngã (I4); a missing tone is still fine (Prefix below).
    if stop && matches!(tone, Tone::Huyen | Tone::Hoi | Tone::Nga) {
        return Validity::Invalid;
    }

    let mut m = match_nucleus(nuc);
    if o.lenient {
        let widen = |x: &mut Option<u16>| {
            if let Some(mask) = x {
                if *mask & WIDE != 0 {
                    *mask |= C | M | N | NG | P | T;
                }
            }
        };
        widen(&mut m.exact);
        widen(&mut m.pending);
    }
    // Reading 1: the nucleus as typed. `Some(true)` = coda complete, `Some(false)` = coda still
    // being typed (c → ch, n → ng/nh).
    let exact = m.exact.and_then(|mask| allows(mask, bit, partial));
    match exact {
        Some(true) if stop && tone == Tone::Ngang => return Validity::Prefix, // sắc/nặng still to come
        Some(true) => return Validity::Complete,
        Some(false) => return Validity::Prefix,
        None => {}
    }
    // Reading 2: a diacritic still to come (e → ê, ua → uâ): only ever a Prefix.
    if let Some(mask) = m.pending.filter(|_| !o.at_end) {
        if coda.is_empty() || allows(mask, bit, partial).is_some() {
            return Validity::Prefix;
        }
    }
    // Closed-only nucleus waiting for its coda (iê, uô, ươ...) or a longer vowel run.
    if coda.is_empty() && (m.exact.is_some() || m.partial) && !o.at_end {
        return Validity::Prefix;
    }
    Validity::Invalid
}

/// Nuclei that only exist after certain initials (vi.dic: khuya, khuỷu, hươu; chửi cửi gửi ngửi).
fn needs_initial(ini: &[u8], nuc: &[Unit]) -> bool {
    let plain = |u: &Unit, c: u8| u.ch == c && u.md == Mod::None;
    match nuc {
        [a, b, c] => {
            let uya_uyu = plain(a, b'u') && plain(b, b'y') && (plain(c, b'a') || plain(c, b'u'));
            let uou = a.ch == b'u'
                && a.md == Mod::Horn
                && b.ch == b'o'
                && b.md == Mod::Horn
                && plain(c, b'u');
            (uya_uyu || uou) && ini.is_empty()
        }
        [a, b] => {
            a.ch == b'u'
                && a.md == Mod::Horn
                && plain(b, b'i')
                && !matches!(ini, b"c" | b"ch" | b"g" | b"ng")
        }
        _ => false,
    }
}

/// `Some(true)` the coda is allowed as typed, `Some(false)` it is a prefix of an allowed one.
fn allows(mask: u16, bit: u16, partial: u16) -> Option<bool> {
    if mask & bit != 0 {
        Some(true)
    } else if mask & partial != 0 {
        Some(false)
    } else {
        None
    }
}

fn spelling_ok(ini: &[u8], nuc: &[Unit]) -> bool {
    let fb = nuc[0].ch;
    let front = matches!(fb, b'e' | b'i' | b'y');
    match ini {
        b"c" => !front,
        b"k" => front,
        // "gì", "gìn": the i is also the initial's own (gi); nucleus must be that plain i.
        b"g" => {
            !matches!(fb, b'e' | b'y') && (fb != b'i' || (nuc.len() == 1 && nuc[0].md == Mod::None))
        }
        b"gh" => matches!(fb, b'e' | b'i'),
        b"ng" => !front,
        b"ngh" => matches!(fb, b'e' | b'i'),
        _ => true,
    }
}

/// (coda bit, bits it may still grow into). `OPEN` when there is no consonant coda; `None` for
/// letters that are no coda.
fn coda_info(coda: &[Unit]) -> Option<(u16, u16)> {
    match coda {
        [] => Some((OPEN, 0)),
        [a] => match a.ch {
            b'c' => Some((C, CH)),
            b'm' => Some((M, 0)),
            b'n' => Some((N, NG | NH)),
            b'p' => Some((P, 0)),
            b't' => Some((T, 0)),
            b'k' => Some((K, 0)),
            _ => None,
        },
        [a, b] => match (a.ch, b.ch) {
            (b'c', b'h') => Some((CH, 0)),
            (b'n', b'g') => Some((NG, 0)),
            (b'n', b'h') => Some((NH, 0)),
            _ => None,
        },
        _ => None,
    }
}

/// How the typed vowels relate to the nucleus whitelist.
struct Nucleus {
    /// matches a nucleus exactly: union of its allowed codas
    exact: Option<u16>,
    /// matches except for diacritics still to be typed (ie → iê)
    pending: Option<u16>,
    /// proper prefix of a longer nucleus (oa → oai)
    partial: bool,
}

fn match_nucleus(nuc: &[Unit]) -> Nucleus {
    let none = Nucleus {
        exact: None,
        pending: None,
        partial: false,
    };
    if nuc.len() > 3 {
        return none;
    }
    // same encoding as scripts/gen/phonology_tables.py `nucleus_key`
    let mut key = 0u16;
    for u in nuc {
        let letter = match u.ch {
            b'a' => 0,
            b'e' => 1,
            b'i' => 2,
            b'o' => 3,
            b'u' => 4,
            _ => 5,
        };
        key = key * 25 + letter * 4 + u.md as u16 + 1;
    }
    match tables::NUCLEUS_INDEX.binary_search_by_key(&key, |e| e.0) {
        Ok(i) => {
            let (_, exact, pending, partial) = tables::NUCLEUS_INDEX[i];
            Nucleus {
                exact: (exact != 0).then_some(exact),
                pending: (pending != 0).then_some(pending),
                partial,
            }
        }
        Err(_) => none,
    }
}

fn tone_code(t: Tone) -> u8 {
    match t {
        Tone::Ngang => 0,
        Tone::Sac => 1,
        Tone::Huyen => 2,
        Tone::Hoi => 3,
        Tone::Nga => 4,
        Tone::Nang => 5,
    }
}

/// Proper names whose spelling breaks the regular rules. Prefix-closed: every key order that
/// builds the name passes through Prefix, so no per-order special case exists anywhere.
fn name_match(units: &[Unit], tone: Tone) -> Validity {
    let mut best = Validity::Invalid;
    'names: for &(pat, name_tone) in tables::NAMES {
        if units.len() > pat.len() || pat[0].0 != units[0].ch {
            continue;
        }
        let mut pending = false;
        for (k, u) in units.iter().enumerate() {
            let w = Unit::from_code(pat[k].0, pat[k].1);
            if u.ch != w.ch || u.stroke != w.stroke {
                continue 'names;
            }
            if u.md != w.md {
                if u.md == Mod::None {
                    pending = true;
                } else {
                    continue 'names;
                }
            }
        }
        let t = tone_code(tone);
        if t != 0 && t != name_tone {
            continue;
        }
        if t == 0 && name_tone != 0 {
            pending = true; // the name's tone has not been typed yet
        }
        if units.len() == pat.len() && !pending {
            return Validity::Complete;
        }
        best = Validity::NamePrefix; // keep looking: another name may match exactly (kon vs kông)
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    /// "viêt" style helper: parse a lowercase Vietnamese string (one tone allowed).
    fn parse(s: &str) -> (Vec<Unit>, Tone) {
        let mut tone = Tone::Ngang;
        let units = s
            .chars()
            .map(|c| {
                let (u, t) = Unit::from_char(c).unwrap();
                if t != Tone::Ngang {
                    tone = t;
                }
                u
            })
            .collect();
        (units, tone)
    }

    fn v(s: &str) -> Validity {
        let (u, t) = parse(s);
        validate(&u, t, &Opts::default())
    }

    #[test]
    fn constitution_examples() {
        use Validity::*;
        // I1/I3: structure and nuclei
        for ok in [
            "ba",
            "hoa",
            "qua",
            "giau",
            "nghiêng",
            "được",
            "trường",
            "khuya",
            "người",
            "gì",
            "gìn",
        ] {
            assert_eq!(v(ok), Complete, "{ok}");
        }
        // I2: spelling (regular grammar only; "ka" is a Prefix of the name "kạn" with names on)
        for bad in ["ce", "ci", "ka", "ge", "ngi", "gha", "ngha", "qa"] {
            let (u, t) = parse(bad);
            assert_eq!(
                validate(
                    &u,
                    t,
                    &Opts {
                        names: false,
                        ..Opts::default()
                    }
                ),
                Invalid,
                "{bad}"
            );
        }
        assert_eq!(v("ka"), NamePrefix);
        // I4: stop coda takes only sắc/nặng; missing tone = still typing
        assert_eq!(v("cấp"), Complete);
        assert_eq!(v("cập"), Complete);
        assert_eq!(v("cảp"), Invalid);
        assert_eq!(v("càt"), Invalid);
        assert_eq!(v("cap"), Prefix);
        // I5: nucleus x coda
        assert_eq!(v("xẻng"), Complete); // -ng after e is Vietnamese
        assert_eq!(v("ôch"), Invalid);
        assert_eq!(v("ơng"), Invalid);
        // missing modifiers / coda are Prefix, never Invalid
        assert_eq!(v("tie"), Prefix); // iê still to come
        assert_eq!(v("tieu"), Prefix);
        assert_eq!(v("tien"), Prefix);
        assert_eq!(v("uo"), Prefix);
        assert_eq!(v("tiê"), Prefix); // closed-only nucleus waiting for its coda
                                      // foreign / English shapes
        for bad in ["clau", "john", "str", "you", "sea", "bcd"] {
            assert_eq!(v(bad), Invalid, "{bad}");
        }
    }

    #[test]
    fn proper_names_are_prefix_closed() {
        use Validity::*;
        for (s, want) in [
            ("k", Prefix),
            ("ko", NamePrefix),
            ("kon", Complete),
            ("kong", NamePrefix),
            ("kông", Complete),
            ("kạn", Complete),
            ("kan", NamePrefix),
            ("krông", Complete),
            ("kro", NamePrefix),
            ("búk", Complete),
        ] {
            assert_eq!(v(s), want, "{s}");
        }
        // names are not a back door for English: kubectl, kotlin
        assert_eq!(v("ku"), Invalid);
        assert_eq!(v("kot"), Invalid);
    }

    #[test]
    fn foreign_initials_and_free_mode() {
        let (u, t) = parse("fa");
        assert_eq!(validate(&u, t, &Opts::default()), Validity::Invalid);
        assert_eq!(
            validate(
                &u,
                t,
                &Opts {
                    foreign_initials: true,
                    ..Opts::default()
                }
            ),
            Validity::Complete
        );
        let (u, t) = parse("cảp");
        assert_eq!(
            validate(
                &u,
                t,
                &Opts {
                    free: true,
                    ..Opts::default()
                }
            ),
            // hỏi on a stop coda is not Vietnamese: only free typing lets it through
            Validity::Loose
        );
    }
}
