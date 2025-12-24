# English Matrix-Based Phonotactic Constraint System

**Report Date:** 2025-12-24
**Author:** AI Assistant
**Purpose:** Matrix-based validation for English words (bidirectional restore)

---

## Research Summary

English phonotactics follows well-defined rules governing valid consonant clusters and vowel patterns. This document designs a matrix-based constraint system for English validation.

### Sources
- [Wikipedia: Phonotactics](https://en.wikipedia.org/wiki/Phonotactics)
- [Wikipedia: Consonant Cluster](https://en.wikipedia.org/wiki/Consonant_cluster)
- [Essentials of Linguistics: Syllable Structure](https://ecampusontario.pressbooks.pub/essentialsoflinguistics/chapter/3-4-syllable-structure/)
- [ENGL6360: Phonotactics](https://pressbooks.utrgv.edu/engl6360/chapter/phonotactics/)
- [Impossible Letter Combinations](https://www.jojhelfer.com/lettercombos)
- [Quora: Letter Combinations](https://www.quora.com/What-are-all-of-the-two-letter-combinations-that-never-occur-in-an-English-dictionary)

---

## English Syllable Structure

```
English syllable: (C)(C)(C)V(C)(C)(C)(C)
Maximum: CCCVCCCC (e.g., "strengths" /strɛŋkθs/)

Components:
- Onset: 0-3 consonants (e.g., "str-" in "string")
- Nucleus: 1 vowel (required)
- Coda: 0-4 consonants (e.g., "-lfths" in "twelfths")
```

---

## M1: ONSET MATRIX (Valid Initial Clusters)

### Single Consonant Onsets (All except /ŋ/)

All consonants can start a word EXCEPT /ŋ/ (ng sound).

```rust
/// Valid single consonant onsets (25 consonants)
const VALID_SINGLE_ONSETS: &[char] = &[
    'b', 'c', 'd', 'f', 'g', 'h', 'j', 'k', 'l', 'm',
    'n', 'p', 'q', 'r', 's', 't', 'v', 'w', 'x', 'y', 'z',
    // Also: ch, sh, th, wh, ph (digraphs treated separately)
];
```

### Two-Consonant Onset Matrix (C₁ + C₂)

**Rule:** If C₁ is NOT /s/, then C₂ must be a liquid (/l/, /r/) or glide (/w/, /j/).

```rust
/// M1: C1 → C2 onset compatibility matrix
/// Rows: First consonant (C1)
/// Cols: Second consonant (C2) - only l, r, w, y are valid C2 (non-/s/ onsets)
const M1_ONSET_CC: [[bool; 26]; 26] = {
    // Only show relevant columns: L(11), R(17), W(22), Y(24)
    // Full matrix would be 26x26 but most cells are FALSE

    //     a  b  c  d  e  f  g  h  i  j  k  L  m  n  o  p  q  R  s  t  u  v  W  x  Y  z
    /* b */ [F, F, F, F, F, F, F, F, F, F, F, T, F, F, F, F, F, T, F, F, F, F, F, F, T, F], // bl, br, by
    /* c */ [F, F, F, F, F, F, F, F, F, F, F, T, F, F, F, F, F, T, F, F, F, F, F, F, F, F], // cl, cr
    /* d */ [F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, T, F, F, F, F, T, F, F, F], // dr, dw
    /* f */ [F, F, F, F, F, F, F, F, F, F, F, T, F, F, F, F, F, T, F, F, F, F, F, F, F, F], // fl, fr
    /* g */ [F, F, F, F, F, F, F, F, F, F, F, T, F, F, F, F, F, T, F, F, F, F, F, F, F, F], // gl, gr
    /* k */ [F, F, F, F, F, F, F, F, F, F, F, T, F, F, F, F, F, T, F, F, F, F, T, F, F, F], // kl, kr, kw (qu)
    /* p */ [F, F, F, F, F, F, F, F, F, F, F, T, F, F, F, F, F, T, F, F, F, F, F, F, F, F], // pl, pr
    /* s */ [F, F, T, F, F, F, F, T, F, F, T, T, T, T, F, T, F, F, F, T, F, F, T, F, F, F], // sc,sh,sk,sl,sm,sn,sp,st,sw
    /* t */ [F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, T, F, F, F, F, T, F, F, F], // tr, tw
    /* v */ [F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F], // vl, vr INVALID
    /* w */ [F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, F, T, F, F, F, F, F, F, F, F], // wr
    // ... other consonants mostly FALSE
};
```

### Valid Two-Consonant Onsets (Complete List)

```rust
/// All valid two-consonant onset clusters in English
const VALID_CC_ONSETS: &[[char; 2]] = &[
    // Stop + Liquid/Glide
    ['b', 'l'], ['b', 'r'],           // bl, br
    ['c', 'l'], ['c', 'r'],           // cl, cr (also ch- digraph)
    ['d', 'r'], ['d', 'w'],           // dr, dw
    ['f', 'l'], ['f', 'r'],           // fl, fr
    ['g', 'l'], ['g', 'r'],           // gl, gr (also gh- in loanwords)
    ['k', 'l'], ['k', 'r'], ['k', 'w'], // kl, kr, kw (qu = kw)
    ['p', 'l'], ['p', 'r'],           // pl, pr (also ph- digraph)
    ['t', 'r'], ['t', 'w'],           // tr, tw (also th- digraph)

    // S + Consonant (special case)
    ['s', 'c'], ['s', 'k'],           // sc, sk
    ['s', 'l'],                       // sl
    ['s', 'm'], ['s', 'n'],           // sm, sn
    ['s', 'p'], ['s', 't'],           // sp, st
    ['s', 'w'],                       // sw

    // Other valid clusters
    ['w', 'r'],                       // wr (silent w: write)
    ['g', 'n'],                       // gn (silent g: gnome) - marginal
    ['k', 'n'],                       // kn (silent k: know) - marginal
    ['p', 'n'],                       // pn (silent p: pneumatic) - marginal
    ['p', 's'],                       // ps (silent p: psychology) - marginal

    // Digraph combinations (represented as single units)
    // sh-, ch-, th-, wh-, ph- are treated as single onset units
];

/// INVALID two-consonant onsets (explicitly banned)
const INVALID_CC_ONSETS: &[[char; 2]] = &[
    ['t', 'l'], ['d', 'l'],           // tl, dl - NEVER valid in English
    ['v', 'l'], ['v', 'r'],           // vl, vr - not native English
    ['s', 'r'],                       // sr - not native English
    ['z', 'b'], ['z', 'd'], ['z', 'g'], // zb, zd, zg - not native English
    // Most other C+C not listed in VALID are also invalid
];
```

### Three-Consonant Onsets (sCC Pattern)

**Rule:** Must be: /s/ + voiceless stop (/p/, /t/, /k/) + liquid/glide (/l/, /r/, /w/)

```rust
/// Valid three-consonant onset clusters
/// Pattern: s + {p,t,k} + {l,r,w,j}
const VALID_CCC_ONSETS: &[[char; 3]] = &[
    // s + p + {l,r}
    ['s', 'p', 'l'],  // spl- (split, splash)
    ['s', 'p', 'r'],  // spr- (spring, spray)

    // s + t + r
    ['s', 't', 'r'],  // str- (string, strong)

    // s + k + {l,r,w}
    ['s', 'k', 'l'],  // Not common but theoretically valid
    ['s', 'k', 'r'],  // scr- (scream, scroll)
    ['s', 'k', 'w'],  // squ- (square, squirrel) - /skw/
];
```

---

## M2: CODA MATRIX (Valid Final Clusters)

### Single Consonant Codas

All consonants can end a word, including /ŋ/ (ng).

### Two-Consonant Coda Matrix (C₁ + C₂)

**Rules:**
1. Liquids/nasals typically precede stops/fricatives
2. Two obstruents must share voicing
3. Nasal + obstruent must be homorganic

```rust
/// M2: C1 → C2 coda compatibility matrix
/// Pattern: (liquid/nasal) + (stop/fricative)
const VALID_CC_CODAS: &[[char; 2]] = &[
    // Liquid + Stop/Fricative
    ['l', 'b'],                       // -lb (bulb)
    ['l', 'd'],                       // -ld (old, bold)
    ['l', 'f'],                       // -lf (self, golf)
    ['l', 'k'],                       // -lk (milk, silk)
    ['l', 'm'],                       // -lm (film, calm)
    ['l', 'n'],                       // -ln (kiln) - rare
    ['l', 'p'],                       // -lp (help, gulp)
    ['l', 't'],                       // -lt (salt, bolt)
    ['l', 'v'],                       // -lv (solve, shelve)

    ['r', 'b'],                       // -rb (curb, verb)
    ['r', 'd'],                       // -rd (hard, word)
    ['r', 'f'],                       // -rf (scarf, turf)
    ['r', 'g'],                       // -rg (org) - rare
    ['r', 'k'],                       // -rk (work, dark)
    ['r', 'l'],                       // -rl (girl, curl)
    ['r', 'm'],                       // -rm (arm, form)
    ['r', 'n'],                       // -rn (born, turn)
    ['r', 'p'],                       // -rp (harp, warp)
    ['r', 's'],                       // -rs (cars, bars)
    ['r', 't'],                       // -rt (art, part)
    ['r', 'v'],                       // -rv (curve, serve)

    // Nasal + Homorganic Stop
    ['m', 'p'],                       // -mp (jump, lamp)
    ['n', 'd'],                       // -nd (and, end)
    ['n', 't'],                       // -nt (ant, went)
    ['n', 'k'],                       // -nk (think, bank) - /ŋk/

    // Stop/Fricative + Stop/Fricative (same voicing)
    ['p', 't'],                       // -pt (kept, slept)
    ['k', 't'],                       // -ct (act, fact)
    ['k', 's'],                       // -x (box, fox) /ks/
    ['f', 't'],                       // -ft (left, soft)
    ['s', 'k'],                       // -sk (ask, desk)
    ['s', 'p'],                       // -sp (grasp, crisp)
    ['s', 't'],                       // -st (fast, best)

    // Voiced pairs
    ['d', 'z'],                       // -ds (kids, beds)
    ['g', 'z'],                       // -gs (bags, dogs)
    ['b', 'z'],                       // -bs (cabs, ribs)
    ['v', 'z'],                       // -ves (saves, lives)
];

/// INVALID coda clusters
const INVALID_CC_CODAS: &[[char; 2]] = &[
    // Voicing mismatch
    ['p', 'd'], ['t', 'd'], ['k', 'd'],  // voiceless + voiced
    ['b', 't'], ['d', 't'], ['g', 't'],  // voiced + voiceless

    // Nasal + non-homorganic
    ['m', 't'], ['m', 'k'],              // m not before t/k
    ['n', 'p'],                          // n not before p

    // Other invalid patterns
    ['r', 'ŋ'],                          // -rng not valid
    ['l', 'ŋ'],                          // -lng not valid
];
```

---

## M3: VOWEL DIGRAPH MATRIX

### Valid Vowel Combinations (Digraphs)

```rust
/// M3: V1 → V2 vowel digraph validity
/// 1 = common digraph, 2 = exists but rare, 0 = invalid
const M3_VOWEL_DIGRAPH: [[u8; 5]; 5] = {
    //     A   E   I   O   U
    /* A */ [0, 1, 1, 0, 1], // ae(rare), ai, au
    /* E */ [1, 1, 1, 0, 1], // ea, ee, ei, eu
    /* I */ [0, 1, 0, 0, 0], // ie
    /* O */ [1, 1, 1, 1, 1], // oa, oe, oi, oo, ou
    /* U */ [0, 1, 1, 0, 0], // ue, ui
};

/// Complete valid vowel digraphs in English
const VALID_VOWEL_DIGRAPHS: &[[char; 2]] = &[
    // A combinations
    ['a', 'i'],  // ai (rain, wait, tail)
    ['a', 'u'],  // au (cause, haul, author)
    ['a', 'y'],  // ay (play, day, say) - y as vowel
    ['a', 'e'],  // ae (rare: aeon, aegis)
    ['a', 'w'],  // aw (saw, law, draw)

    // E combinations
    ['e', 'a'],  // ea (beach, read, bread)
    ['e', 'e'],  // ee (tree, see, feet)
    ['e', 'i'],  // ei (receive, ceiling, weird)
    ['e', 'u'],  // eu (feud, neutral) - rare
    ['e', 'w'],  // ew (new, few, dew)
    ['e', 'y'],  // ey (key, money, hey)

    // I combinations
    ['i', 'e'],  // ie (pie, tie, field)
    ['i', 'a'],  // ia (in words like "dial" - rare as true digraph)

    // O combinations
    ['o', 'a'],  // oa (boat, coat, road)
    ['o', 'e'],  // oe (toe, foe, goes)
    ['o', 'i'],  // oi (oil, coin, point)
    ['o', 'o'],  // oo (book, moon, food)
    ['o', 'u'],  // ou (house, loud, soup)
    ['o', 'w'],  // ow (cow, now, show)
    ['o', 'y'],  // oy (boy, toy, enjoy)

    // U combinations
    ['u', 'e'],  // ue (blue, true, cue)
    ['u', 'i'],  // ui (fruit, suit, juice)
];

/// INVALID vowel combinations (never valid digraphs)
const INVALID_VOWEL_PAIRS: &[[char; 2]] = &[
    ['a', 'a'],  // aa - not an English digraph
    ['i', 'i'],  // ii - not an English digraph
    ['u', 'u'],  // uu - not an English digraph
    ['i', 'o'],  // io - not a standard digraph (occurs in compounds)
    ['u', 'o'],  // uo - not a standard digraph
    ['e', 'o'],  // eo - rare, mainly in proper nouns
];
```

---

## M4: IMPOSSIBLE BIGRAM MATRIX

Based on empirical analysis of English dictionaries, these two-letter combinations **NEVER** occur.

```rust
/// M4: Bigrams that NEVER appear in English words
/// If found, word is DEFINITELY not English
const IMPOSSIBLE_BIGRAMS: &[[char; 2]] = &[
    // Q combinations (Q always followed by U in English)
    ['q', 'a'], ['q', 'b'], ['q', 'c'], ['q', 'd'], ['q', 'e'], ['q', 'f'],
    ['q', 'g'], ['q', 'h'], ['q', 'j'], ['q', 'k'], ['q', 'l'], ['q', 'm'],
    ['q', 'n'], ['q', 'o'], ['q', 'p'], ['q', 'q'], ['q', 'r'], ['q', 's'],
    ['q', 't'], ['q', 'v'], ['q', 'w'], ['q', 'x'], ['q', 'y'], ['q', 'z'],

    // X combinations
    ['x', 'b'], ['x', 'd'], ['x', 'g'], ['x', 'j'], ['x', 'k'], ['x', 'm'],
    ['x', 'r'], ['x', 'x'],

    // J combinations
    ['j', 'b'], ['j', 'c'], ['j', 'd'], ['j', 'f'], ['j', 'g'], ['j', 'h'],
    ['j', 'j'], ['j', 'l'], ['j', 'm'], ['j', 'n'], ['j', 'p'], ['j', 'q'],
    ['j', 't'], ['j', 'v'], ['j', 'w'], ['j', 'x'], ['j', 'y'], ['j', 'z'],

    // V combinations
    ['v', 'b'], ['v', 'f'], ['v', 'j'], ['v', 'k'], ['v', 'm'], ['v', 'p'],
    ['v', 'q'], ['v', 't'], ['v', 'w'], ['v', 'x'], ['v', 'z'],

    // Z combinations
    ['z', 'f'], ['z', 'j'], ['z', 'q'], ['z', 'x'],

    // Other never-occurring pairs
    ['b', 'q'], ['b', 'x'],
    ['c', 'g'], ['c', 'j'], ['c', 'p'], ['c', 'v'], ['c', 'w'], ['c', 'x'],
    ['d', 'x'],
    ['f', 'q'], ['f', 'v'], ['f', 'x'], ['f', 'z'],
    ['g', 'q'], ['g', 'v'], ['g', 'x'],
    ['h', 'x'], ['h', 'z'],
    ['k', 'q'], ['k', 'v'], ['k', 'x'], ['k', 'z'],
    ['m', 'g'], ['m', 'j'], ['m', 'x'], ['m', 'z'],
    ['p', 'q'], ['p', 'v'], ['p', 'x'],
    ['s', 'x'],
    ['t', 'q'], ['t', 'x'],
    ['w', 'j'], ['w', 'q'], ['w', 'v'], ['w', 'x'],
];

/// Generate 26x26 impossible bigram matrix at compile time
const fn generate_impossible_matrix() -> [[bool; 26]; 26] {
    // Convert IMPOSSIBLE_BIGRAMS to matrix
    // impossible_matrix[c1][c2] = true means c1+c2 never occurs
    // ...
}
```

---

## M5: TRIGRAM PATTERNS

### Invalid Trigrams (Never occur in English)

```rust
/// Patterns that signal NON-English
const INVALID_TRIGRAMS: &[[char; 3]] = &[
    // Triple consonants (no vowel)
    ['b', 'c', 'd'], ['f', 'g', 'h'], ['j', 'k', 'l'],
    // (Most CCC without s- are invalid)

    // Triple same letter
    ['a', 'a', 'a'], ['b', 'b', 'b'], ['c', 'c', 'c'],
    ['d', 'd', 'd'], ['e', 'e', 'e'], ['f', 'f', 'f'],
    ['g', 'g', 'g'], ['h', 'h', 'h'], ['i', 'i', 'i'],
    ['j', 'j', 'j'], ['k', 'k', 'k'], ['l', 'l', 'l'],
    ['m', 'm', 'm'], ['n', 'n', 'n'], ['o', 'o', 'o'],
    ['p', 'p', 'p'], ['q', 'q', 'q'], ['r', 'r', 'r'],
    ['s', 's', 's'], ['t', 't', 't'], ['u', 'u', 'u'],
    ['v', 'v', 'v'], ['w', 'w', 'w'], ['x', 'x', 'x'],
    ['y', 'y', 'y'], ['z', 'z', 'z'],
];

/// Valid double letters in English (for comparison)
const VALID_DOUBLE_LETTERS: &[char] = &[
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'l', 'm',
    'n', 'o', 'p', 'r', 's', 't', 'z',
    // Missing: h, i, j, k, q, u, v, w, x, y (rarely/never double)
];
```

---

## Unified English Constraint Solver

```rust
/// English phonotactic validation result
#[derive(Debug, Clone, PartialEq)]
pub enum EnglishValidation {
    Valid,
    PossiblyValid,           // Matches patterns but not confirmed
    InvalidOnset { cluster: String },
    InvalidCoda { cluster: String },
    ImpossibleBigram { bigram: String },
    InvalidTripleConsonant,
    InvalidTripleLetter { letter: char },
    TooManyConsonants { count: usize },
}

impl EnglishValidation {
    pub fn is_valid(&self) -> bool {
        matches!(self, EnglishValidation::Valid | EnglishValidation::PossiblyValid)
    }

    pub fn is_definitely_invalid(&self) -> bool {
        !matches!(self, EnglishValidation::Valid | EnglishValidation::PossiblyValid)
    }
}

/// Main English validation using matrices
pub fn validate_english_word(word: &str) -> EnglishValidation {
    let word_lower = word.to_lowercase();
    let chars: Vec<char> = word_lower.chars().collect();

    if chars.is_empty() {
        return EnglishValidation::InvalidOnset { cluster: String::new() };
    }

    // Phase 1: Check impossible bigrams (M4)
    for window in chars.windows(2) {
        let bigram = [window[0], window[1]];
        if is_impossible_bigram(bigram) {
            return EnglishValidation::ImpossibleBigram {
                bigram: format!("{}{}", window[0], window[1]),
            };
        }
    }

    // Phase 2: Check triple letters (never valid)
    for window in chars.windows(3) {
        if window[0] == window[1] && window[1] == window[2] {
            return EnglishValidation::InvalidTripleLetter { letter: window[0] };
        }
    }

    // Phase 3: Validate onset cluster (M1)
    if let Some(violation) = validate_onset(&chars) {
        return violation;
    }

    // Phase 4: Validate coda cluster (M2)
    if let Some(violation) = validate_coda(&chars) {
        return violation;
    }

    // Phase 5: Check vowel patterns (M3)
    // Words must have at least one vowel
    if !has_vowel(&chars) {
        return EnglishValidation::InvalidOnset {
            cluster: word_lower.clone()
        };
    }

    // Phase 6: Morphological patterns (affixes)
    if has_english_morphology(&word_lower) {
        return EnglishValidation::Valid;
    }

    // Phase 7: Common consonant clusters
    if has_english_cluster_pattern(&word_lower) {
        return EnglishValidation::Valid;
    }

    EnglishValidation::PossiblyValid
}

/// O(1) impossible bigram check using matrix
#[inline]
fn is_impossible_bigram(bigram: [char; 2]) -> bool {
    let c1 = bigram[0] as usize - 'a' as usize;
    let c2 = bigram[1] as usize - 'a' as usize;

    if c1 >= 26 || c2 >= 26 {
        return false; // Non-letter characters
    }

    IMPOSSIBLE_BIGRAM_MATRIX[c1][c2]
}

/// Validate word onset
fn validate_onset(chars: &[char]) -> Option<EnglishValidation> {
    // Find onset (consonants before first vowel)
    let first_vowel = chars.iter().position(|c| is_vowel(*c))?;

    if first_vowel == 0 {
        return None; // No onset, valid
    }

    if first_vowel > 3 {
        return Some(EnglishValidation::TooManyConsonants { count: first_vowel });
    }

    let onset = &chars[..first_vowel];

    match onset.len() {
        1 => None, // Single consonant always valid (except ŋ)
        2 => {
            let cluster = [onset[0], onset[1]];
            if !is_valid_cc_onset(cluster) {
                return Some(EnglishValidation::InvalidOnset {
                    cluster: onset.iter().collect(),
                });
            }
            None
        }
        3 => {
            let cluster = [onset[0], onset[1], onset[2]];
            if !is_valid_ccc_onset(cluster) {
                return Some(EnglishValidation::InvalidOnset {
                    cluster: onset.iter().collect(),
                });
            }
            None
        }
        _ => Some(EnglishValidation::TooManyConsonants { count: onset.len() }),
    }
}

/// Validate word coda
fn validate_coda(chars: &[char]) -> Option<EnglishValidation> {
    // Find coda (consonants after last vowel)
    let last_vowel = chars.iter().rposition(|c| is_vowel(*c))?;

    if last_vowel == chars.len() - 1 {
        return None; // No coda, valid
    }

    let coda = &chars[last_vowel + 1..];

    if coda.len() > 4 {
        return Some(EnglishValidation::TooManyConsonants { count: coda.len() });
    }

    // Validate coda clusters
    match coda.len() {
        1 => None, // Single consonant always valid
        2 => {
            let cluster = [coda[0], coda[1]];
            if !is_valid_cc_coda(cluster) {
                return Some(EnglishValidation::InvalidCoda {
                    cluster: coda.iter().collect(),
                });
            }
            None
        }
        _ => None, // Longer codas need more complex validation
    }
}

#[inline]
fn is_vowel(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 'y')
}

fn has_vowel(chars: &[char]) -> bool {
    chars.iter().any(|c| is_vowel(*c))
}
```

---

## Performance Analysis

| Check | Complexity | Memory |
|-------|-----------|--------|
| Impossible bigram | **O(1)** matrix lookup | 676 bytes (26×26) |
| Valid onset CC | **O(1)** matrix lookup | 676 bytes |
| Valid onset CCC | **O(n)** array search | ~50 bytes |
| Valid coda CC | **O(1)** matrix lookup | 676 bytes |
| Triple letter | **O(n)** sliding window | 0 bytes |
| Vowel digraph | **O(1)** matrix lookup | 25 bytes (5×5) |
| **Total** | **O(n)** where n = word length | ~2 KB |

---

## Integration with Vietnamese Validation

### Bidirectional Validation Flow

```rust
/// Combined VN + EN validation for auto-restore
pub fn should_restore_to_raw(
    vn_result: ConstraintViolation,
    raw_input: &str,
) -> bool {
    // Step 1: If valid Vietnamese, keep as-is
    if vn_result.is_valid() {
        return false;
    }

    // Step 2: Validate English
    let en_result = validate_english_word(raw_input);

    match en_result {
        EnglishValidation::Valid => true,           // Restore
        EnglishValidation::PossiblyValid => true,   // Restore (benefit of doubt)
        _ => false,  // Invalid EN too, keep as-is
    }
}
```

---

## Summary

This matrix-based English constraint system provides:

1. **M1: Onset Matrix** - O(1) validation of initial consonant clusters
2. **M2: Coda Matrix** - O(1) validation of final consonant clusters
3. **M3: Vowel Digraph Matrix** - O(1) validation of vowel combinations
4. **M4: Impossible Bigram Matrix** - O(1) detection of impossible letter pairs
5. **M5: Trigram Patterns** - Triple letter and triple consonant detection

All validation is matrix-based, providing consistent O(1) or O(n) performance where n is word length.
