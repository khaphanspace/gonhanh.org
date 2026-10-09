//! One interpretation of the keys typed so far, and how a new key extends it.
//!
//! A parse owns the *letters* that would be shown (`units`), the tone, and a role per raw key.
//! `extend` returns every sensible reading of the next key; it does not decide between them and
//! does not know the dictionary of English: `phonology::validate` prunes, `lattice` ranks.

use super::method::{intent, Intent};
use super::{Options, RawKey, MAXK};
use crate::phonology::{tone_index, Mod, Tone, Unit};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Role {
    Literal,
    Tone,
    Circ,
    Horn,
    Breve,
    Stroke,
    Remove,
    /// `w` shown as ư on its own
    BaseHorn,
    /// second press of a modifier: cancels it and shows the key literally
    Revert,
    /// redundant repeat of a tone key inside the same syllable: eaten
    Swallow,
    /// `]` / `[` shown as ư / ơ
    Bracket,
}

/// The most recent modifier, so that pressing the same key again can undo exactly it.
#[derive(Clone, Copy)]
struct Last {
    role: Role,
    key: u8,
    /// the letter that was typed, so only that letter undoes it
    ch: u8,
    targets: [u8; 2],
    n: u8,
}

const NO_LAST: Last = Last {
    role: Role::Literal,
    key: 0,
    ch: 0,
    targets: [0; 2],
    n: 0,
};
const BLANK: Unit = Unit::new(0, Mod::None);

#[derive(Clone, Copy)]
pub struct Parse {
    pub roles: [Role; MAXK],
    pub units: [Unit; MAXK],
    pub unit_key: [u8; MAXK],
    pub n: u8,
    pub tone: Tone,
    pub score: i32,
    /// a modifier was cancelled: the word is literal from here on (English intent)
    pub reverted: bool,
    /// the cancel came later than the key right after (herere), not a double-press
    pub late_revert: bool,
    /// Unit that carries the tone once the word stopped being Vietnamese (`NO_TONE_AT` while it
    /// is: the grammar places the tone then). The mark stays where it was shown, so letters
    /// typed afterwards (háaa + e) never make it jump.
    pub tone_at: u8,
    last: Last,
}

pub const NO_TONE_AT: u8 = u8::MAX;

impl Parse {
    pub const fn empty() -> Self {
        Parse {
            roles: [Role::Literal; MAXK],
            units: [BLANK; MAXK],
            unit_key: [0; MAXK],
            n: 0,
            tone: Tone::Ngang,
            score: 0,
            reverted: false,
            late_revert: false,
            tone_at: NO_TONE_AT,
            last: NO_LAST,
        }
    }

    pub fn units(&self) -> &[Unit] {
        &self.units[..self.n as usize]
    }

    /// The word is no longer Vietnamese: pin the tone to the vowel it was shown on (`prev` is the
    /// reading before the key that produced `self`).
    ///
    /// A cancelled modifier puts the word back to how it was before the modifier, so the tone
    /// goes back there too (mùa + a → muầ, + a → mùaa: it does not stay on the vanished â).
    pub fn freeze_tone(&mut self, prev: &Parse, modern: bool, cancelled: bool) {
        if self.tone == Tone::Ngang {
            self.tone_at = NO_TONE_AT;
        } else if cancelled || self.tone_at == NO_TONE_AT {
            // the cancel appended one literal letter: the tone sits where the word without it had it
            let units = if cancelled {
                let n = self.n as usize;
                &self.units[..n.saturating_sub(1).max(1)]
            } else if prev.tone == self.tone {
                prev.units()
            } else {
                self.units()
            };
            self.tone_at = tone_index(units, modern).map_or(NO_TONE_AT, |k| k as u8);
        }
    }

    /// The word is Vietnamese (again): the grammar decides where the tone goes.
    pub fn release_tone(&mut self) {
        self.tone_at = NO_TONE_AT;
    }

    /// A stroke, breve or horn on a typed vowel: Vietnamese on purpose (a lone `w` is not).
    pub fn strong_intent(&self, keys: usize) -> bool {
        let r = &self.roles[..keys];
        r.iter()
            .any(|x| matches!(x, Role::Stroke | Role::Breve | Role::Bracket))
            || (r.contains(&Role::Horn) && !r.contains(&Role::BaseHorn))
    }

    /// Role of key `i`.
    pub fn role(&self, i: usize) -> Role {
        self.roles[i]
    }

    /// Same letters, tone and undo state: interchangeable for the future.
    pub fn same_future(&self, o: &Parse) -> bool {
        self.n == o.n
            && self.tone == o.tone
            && self.reverted == o.reverted
            && self.late_revert == o.late_revert
            && self.tone_at == o.tone_at
            && self.last.role == o.last.role
            && self.last.key == o.last.key
            && self.last.ch == o.last.ch
            && self.units() == o.units()
    }

    fn has_vowel(&self) -> bool {
        self.units().iter().any(|u| u.is_vowel())
    }

    fn push_unit(&mut self, key: usize, u: Unit) {
        self.units[self.n as usize] = u;
        self.unit_key[self.n as usize] = key as u8;
        self.n += 1;
    }

    /// Key `i` shown as the letter itself, in place.
    pub fn push_literal(&mut self, i: usize, ch: u8) {
        self.roles[i] = Role::Literal;
        self.push_unit(i, Unit::new(ch, Mod::None));
        self.last = NO_LAST;
    }

    pub fn literal(&self, i: usize, ch: u8) -> Parse {
        let mut c = *self;
        c.roles[i] = Role::Literal;
        c.push_unit(i, Unit::new(ch, Mod::None));
        c.last = NO_LAST;
        c
    }

    fn modified(&self, i: usize, role: Role, score: i32, targets: &[usize]) -> Parse {
        let mut c = *self;
        c.roles[i] = role;
        c.score += score;
        c.last = Last {
            role,
            key: i as u8,
            ch: 0,
            targets: [0; 2],
            n: targets.len() as u8,
        };
        for (k, t) in targets.iter().enumerate() {
            c.last.targets[k] = *t as u8;
        }
        c
    }

    /// Finish a cancellation: this key is shown literally and the word turns literal.
    fn into_revert(self, i: usize, ch: u8) -> Parse {
        self.into_revert_as(i, i, ch)
    }

    /// Like `into_revert`, the shown letter takes its case from raw key `from`.
    fn into_revert_as(mut self, i: usize, from: usize, ch: u8) -> Parse {
        self.roles[i] = Role::Revert;
        self.score += 20;
        self.reverted = true;
        self.last = NO_LAST;
        self.push_unit(from, Unit::new(ch, Mod::None));
        self
    }
}

const MODIFIER_SCORE: i32 = 10;

/// All readings of key `i` on top of `p`. Always includes the literal reading.
pub fn extend(p: &Parse, i: usize, key: RawKey, o: &Options, out: &mut Children) {
    out.push(p.literal(i, key.ch));
    if key.ch == b'[' || key.ch == b']' {
        if o.bracket && o.method == super::method::Method::Telex {
            bracket(p, i, key, out);
        }
        return;
    }
    // No special "word is literal after a revert" rule: if the letters left over are Vietnamese
    // (choòng = ch+oo+ng) later tone keys still apply, otherwise grammar prunes them.
    let first = out.n;
    horn_follows_horn(p, i, key, out);
    match intent(o.method, key.ch) {
        Intent::Plain => {}
        Intent::Tone(t) => tone(p, i, key, t, o, out),
        Intent::Remove => remove(p, i, out),
        Intent::Circ(letter) => circ(p, i, key, letter, out),
        Intent::W => w_key(p, i, key, o, out),
        Intent::Horn => horn(p, i, key, out, b"ou", Role::Horn),
        Intent::Breve => horn(p, i, key, out, b"a", Role::Breve),
        Intent::Stroke => stroke(p, i, key, out),
    }
    for c in &mut out.items[first..out.n] {
        if c.last.key as usize == i && c.last.role != Role::Literal {
            c.last.ch = key.ch;
        }
    }
}

/// `w` / `7` gave ư and `o` follows at once: the pair is the ươ compound (uwo, twong → tương).
fn horn_follows_horn(p: &Parse, i: usize, key: RawKey, out: &mut Children) {
    let l = p.last;
    let after_u_horn = matches!(l.role, Role::BaseHorn | Role::Horn) && l.key as usize + 1 == i;
    if key.ch != b'o' || !after_u_horn || p.n == 0 {
        return;
    }
    let prev = p.units[p.n as usize - 1];
    if prev.ch == b'u' && prev.md == Mod::Horn {
        let mut c = p.modified(i, Role::Horn, MODIFIER_SCORE + 1, &[p.n as usize]);
        c.push_unit(i, Unit::new(b'o', Mod::Horn));
        c.last = NO_LAST; // the compound is complete: a further w is redundant, not an undo
        out.push(c);
    }
}

/// Most readings one key can add to one parse (letter, tone, circumflex/horn targets, ...).
const CHILDREN: usize = 12;

/// Fixed-capacity list of candidate parses (no heap).
pub struct Children {
    pub items: [Parse; CHILDREN],
    pub n: usize,
}

impl Default for Children {
    fn default() -> Self {
        Self::new()
    }
}

impl Children {
    pub fn new() -> Self {
        Children {
            items: [Parse::empty(); CHILDREN],
            n: 0,
        }
    }

    fn push(&mut self, p: Parse) {
        if self.n < self.items.len() {
            self.items[self.n] = p;
            self.n += 1;
        }
    }
}

/// `]` → ư, `[` → ơ; the same bracket again right after gives the bracket itself.
fn bracket(p: &Parse, i: usize, key: RawKey, out: &mut Children) {
    let vowel = if key.ch == b']' { b'u' } else { b'o' };
    let l = p.last;
    if l.role == Role::Bracket
        && l.key as usize + 1 == i
        && p.n > 0
        && p.units[p.n as usize - 1].ch == vowel
    {
        let mut c = *p;
        c.n -= 1;
        out.push(c.into_revert(i, key.ch));
        return;
    }
    let mut c = *p;
    c.roles[i] = Role::Bracket;
    c.score += MODIFIER_SCORE;
    c.push_unit(i, Unit::new(vowel, Mod::Horn));
    c.last = Last {
        role: Role::Bracket,
        key: i as u8,
        ch: 0,
        targets: [0; 2],
        n: 0,
    };
    out.push(c);
}

fn tone(p: &Parse, i: usize, key: RawKey, t: Tone, o: &Options, out: &mut Children) {
    if !p.has_vowel() {
        return;
    }
    if p.tone == t {
        let at = tone_index(p.units(), o.modern_tone);
        // The same tone key again cancels the tone when it follows at once (raisse → raise) or
        // the marked vowel ends the word (herere); with only vowels after the marked one it is
        // redundant and eaten (roofif → rồi); with a consonant after it is just a letter.
        let adjacent = p.last.role == Role::Tone && p.last.key as usize + 1 == i;
        let rest_is_empty = at.is_some_and(|k| k + 1 == p.units().len());
        let only_vowels_after = at.is_some_and(|k| {
            let rest = &p.units()[k + 1..];
            !rest.is_empty() && rest.iter().all(|u| u.is_vowel())
        });
        if adjacent || rest_is_empty {
            let mut c = *p;
            c.late_revert = !adjacent;
            c.tone = Tone::Ngang;
            // a cancelled tone makes the word English-looking: a circumflex that arrived late
            // (herere → here) goes with it
            for u in &mut c.units[..c.n as usize] {
                if u.md == Mod::Circ {
                    u.md = Mod::None;
                }
            }
            out.push(c.into_revert(i, key.ch));
        } else if only_vowels_after {
            let mut c = *p;
            c.roles[i] = Role::Swallow;
            c.score += 5;
            out.push(c);
        }
        return;
    }
    let mut c = p.modified(i, Role::Tone, MODIFIER_SCORE, &[]);
    c.tone = t;
    out.push(c);
}

fn remove(p: &Parse, i: usize, out: &mut Children) {
    if p.tone != Tone::Ngang {
        let mut c = p.modified(i, Role::Remove, MODIFIER_SCORE, &[]);
        c.tone = Tone::Ngang;
        out.push(c);
    } else if let Some(k) = (0..p.n as usize).rev().find(|&k| {
        // a lone w shown as ư is the letter w, not a diacritic to take off
        p.units[k].is_vowel()
            && p.units[k].md != Mod::None
            && p.roles[p.unit_key[k] as usize] != Role::BaseHorn
    }) {
        let mut c = p.modified(i, Role::Remove, MODIFIER_SCORE, &[]);
        c.units[k].md = Mod::None;
        out.push(c);
    }
}

/// Undo the last modifier when the same key follows it immediately.
fn undo_last(p: &Parse, i: usize, key: RawKey, roles: &[Role], out: &mut Children) -> bool {
    let l = p.last;
    if !roles.contains(&l.role) || l.key as usize + 1 != i || l.ch != key.ch {
        return false;
    }
    let mut c = *p;
    let mut from = i;
    match l.role {
        Role::BaseHorn => {
            c.n -= 1; // the ư shown for the first w disappears, the w comes back as first typed
            from = l.key as usize;
        }
        Role::Stroke => c.units[l.targets[0] as usize].stroke = false,
        _ => {
            for k in 0..l.n as usize {
                c.units[l.targets[k] as usize].md = Mod::None;
            }
        }
    }
    out.push(c.into_revert_as(i, from, key.ch));
    true
}

fn circ(p: &Parse, i: usize, key: RawKey, letter: Option<u8>, out: &mut Children) {
    if undo_last(p, i, key, &[Role::Circ], out) {
        return;
    }
    let units = p.units();
    match letter {
        // Telex: the nearest earlier unit with the same letter takes the circumflex; pressing the
        // letter right after it cancels it (dataa → data, handled by `undo_last`)
        Some(l) => {
            if let Some(t) = units.iter().rposition(|u| u.ch == l) {
                match units[t].md {
                    // ơ + o switches to the circumflex (owo → ô); a + w + a does not
                    Mod::None => {
                        let mut c = p.modified(i, Role::Circ, MODIFIER_SCORE, &[t]);
                        c.units[t].md = Mod::Circ;
                        out.push(c);
                    }
                    Mod::Horn if l == b'o' => {
                        let mut c = p.modified(i, Role::Circ, MODIFIER_SCORE, &[t]);
                        c.units[t].md = Mod::Circ;
                        out.push(c);
                    }
                    // already circumflexed (the letter again is just a letter), or a breve / ư
                    Mod::Horn | Mod::Breve | Mod::Circ => {}
                }
            }
        }
        // VNI 6: any a/e/o may take it; grammar keeps the one that makes a syllable
        None => {
            for (t, u) in units.iter().enumerate().rev() {
                if matches!(u.ch, b'a' | b'e' | b'o') && u.md != Mod::Circ {
                    let mut c = p.modified(i, Role::Circ, MODIFIER_SCORE, &[t]);
                    c.units[t].md = Mod::Circ;
                    out.push(c);
                }
            }
        }
    }
}

/// Horn (o, u) or breve (a) on every eligible vowel, plus the uo → ươ compound.
fn horn(p: &Parse, i: usize, key: RawKey, out: &mut Children, letters: &[u8], role: Role) {
    if undo_last(p, i, key, &[Role::Horn, Role::Breve], out) {
        return;
    }
    let units = p.units();
    for (t, u) in units.iter().enumerate() {
        // a circumflex can be switched to horn/breve (ô → ơ, â → ă) by the other modifier
        let eligible = letters.contains(&u.ch)
            && (u.md == Mod::None || (u.md == Mod::Circ && matches!(u.ch, b'a' | b'o')));
        // uô + horn is the compound ươ (below), not uơ
        let compound_switch = u.md == Mod::Circ
            && u.ch == b'o'
            && t > 0
            && units[t - 1].ch == b'u'
            && units[t - 1].md == Mod::None;
        if eligible && !compound_switch {
            let mut c = p.modified(i, role, MODIFIER_SCORE, &[t]);
            c.units[t].md = if u.ch == b'a' { Mod::Breve } else { Mod::Horn };
            out.push(c);
        }
    }
    if role == Role::Horn {
        for t in 1..units.len() {
            if units[t - 1].ch == b'u'
                && units[t - 1].md == Mod::None
                && units[t].ch == b'o'
                && matches!(units[t].md, Mod::None | Mod::Circ)
            {
                let mut c = p.modified(i, Role::Horn, MODIFIER_SCORE + 1, &[t - 1, t]);
                c.units[t - 1].md = Mod::Horn;
                c.units[t].md = Mod::Horn;
                out.push(c);
            }
        }
    }
}

fn w_key(p: &Parse, i: usize, key: RawKey, o: &Options, out: &mut Children) {
    if undo_last(p, i, key, &[Role::Horn, Role::Breve, Role::BaseHorn], out) {
        return;
    }
    let before = out.n;
    horn(p, i, key, out, b"aou", Role::Horn);
    // every vowel that can take a horn/breve already has one (uwow): the extra w is eaten
    if out.n == before
        && p.units()
            .iter()
            .any(|u| u.md == Mod::Horn || u.md == Mod::Breve)
    {
        let mut c = *p;
        c.roles[i] = Role::Swallow;
        c.score += 5;
        out.push(c);
    }
    // A lone w that is already a consonant (w allowed as an initial) never becomes ư after itself:
    // ww cancels the ư, it does not spell "wư".
    let after_w = p.units().last().is_some_and(|u| u.ch == b'w');
    if o.w_as_vowel && !after_w {
        let mut c = *p;
        c.roles[i] = Role::BaseHorn;
        c.score += MODIFIER_SCORE - 1;
        c.push_unit(i, Unit::new(b'u', Mod::Horn));
        c.last = Last {
            role: Role::BaseHorn,
            key: i as u8,
            ch: 0,
            targets: [0; 2],
            n: 0,
        };
        out.push(c);
    }
}

fn stroke(p: &Parse, i: usize, key: RawKey, out: &mut Children) {
    if undo_last(p, i, key, &[Role::Stroke], out) {
        return;
    }
    if let Some(t) = p.units().iter().position(|u| u.ch == b'd' && !u.stroke) {
        let mut c = p.modified(i, Role::Stroke, MODIFIER_SCORE, &[t]);
        c.units[t].stroke = true;
        out.push(c);
    }
}
