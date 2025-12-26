//! English Detection Matrices
//!
//! Matrix-based English word detection for auto-restore.
//! Uses sparse encoding (valid pairs only) for memory efficiency.
//!
//! ## Purpose
//! Positive English detection ONLY. For foreign word detection,
//! use Vietnamese validation in validation.rs - if invalid Vietnamese, it's foreign.
//!
//! ## Matrices
//! - E1: ONSET_PAIRS - Valid consonant clusters at word start
//! - E2: CODA_PAIRS - Valid consonant clusters at word end
//! - E3: IMPOSSIBLE_AFTER - Impossible bigrams
//! - E4: SUFFIXES/PREFIXES - Common English patterns
//! - E5: english_likelihood - Combined scoring function

use crate::data::keys;

// =============================================================================
// Helper: Convert keycode to letter index (0-25)
// =============================================================================

/// Convert keycode to letter index (0-25 for a-z)
/// Returns 255 for non-letter keys
#[inline]
pub fn key_to_index(key: u16) -> u8 {
    match key {
        k if k == keys::A => 0,
        k if k == keys::B => 1,
        k if k == keys::C => 2,
        k if k == keys::D => 3,
        k if k == keys::E => 4,
        k if k == keys::F => 5,
        k if k == keys::G => 6,
        k if k == keys::H => 7,
        k if k == keys::I => 8,
        k if k == keys::J => 9,
        k if k == keys::K => 10,
        k if k == keys::L => 11,
        k if k == keys::M => 12,
        k if k == keys::N => 13,
        k if k == keys::O => 14,
        k if k == keys::P => 15,
        k if k == keys::Q => 16,
        k if k == keys::R => 17,
        k if k == keys::S => 18,
        k if k == keys::T => 19,
        k if k == keys::U => 20,
        k if k == keys::V => 21,
        k if k == keys::W => 22,
        k if k == keys::X => 23,
        k if k == keys::Y => 24,
        k if k == keys::Z => 25,
        _ => 255,
    }
}

/// Check if letter index is a vowel (a, e, i, o, u, y)
#[inline]
pub fn is_vowel_idx(idx: u8) -> bool {
    matches!(idx, 0 | 4 | 8 | 14 | 20 | 24) // a, e, i, o, u, y
}

// =============================================================================
// E1: Valid Onset Clusters (English word-start consonant clusters)
// =============================================================================

/// Valid English onset (word-start) consonant clusters
const ONSET_PAIRS: &[(u8, u8)] = &[
    (1, 11),
    (1, 17), // bl, br
    (2, 7),
    (2, 11),
    (2, 17),
    (2, 24), // ch, cl, cr, cy
    (3, 17),
    (3, 22), // dr, dw
    (5, 11),
    (5, 17), // fl, fr
    (6, 7),
    (6, 11),
    (6, 13),
    (6, 17), // gh, gl, gn, gr
    (10, 13),
    (10, 22), // kn, kw
    (15, 7),
    (15, 11),
    (15, 17),
    (15, 18),
    (15, 19), // ph, pl, pr, ps, pt
    (16, 20), // qu
    (18, 2),
    (18, 7),
    (18, 10),
    (18, 11),
    (18, 12),
    (18, 13),
    (18, 15),
    (18, 16),
    (18, 19),
    (18, 22), // sc, sh, sk, sl, sm, sn, sp, sq, st, sw
    (19, 7),
    (19, 17),
    (19, 22), // th, tr, tw
    (22, 7),
    (22, 17),
    (22, 24), // wh, wr, wy
    (17, 7),
    (25, 7), // rh, zh
];

/// Check if two-consonant onset cluster is valid in English
#[inline]
pub fn is_valid_onset(c1: u8, c2: u8) -> bool {
    ONSET_PAIRS.iter().any(|&(a, b)| a == c1 && b == c2)
}

/// Check if onset cluster is valid using keycodes
#[inline]
pub fn is_valid_onset_keys(k1: u16, k2: u16) -> bool {
    let c1 = key_to_index(k1);
    let c2 = key_to_index(k2);
    if c1 == 255 || c2 == 255 {
        return false;
    }
    is_valid_onset(c1, c2)
}

// =============================================================================
// E2: Valid Coda Clusters (English word-end consonant clusters)
// =============================================================================

/// Valid English coda (word-end) consonant clusters
const CODA_PAIRS: &[(u8, u8)] = &[
    (2, 10),
    (5, 19), // ck, ft
    (11, 2),
    (11, 3),
    (11, 5),
    (11, 10),
    (11, 11),
    (11, 12),
    (11, 15),
    (11, 18),
    (11, 19), // lc, ld, lf, lk, ll, lm, lp, ls, lt
    (12, 15),
    (12, 18), // mp, ms
    (13, 2),
    (13, 3),
    (13, 6),
    (13, 10),
    (13, 13),
    (13, 18),
    (13, 19), // nc, nd, ng, nk, nn, ns, nt
    (17, 2),
    (17, 3),
    (17, 5),
    (17, 6),
    (17, 10),
    (17, 11),
    (17, 12),
    (17, 13),
    (17, 15),
    (17, 17),
    (17, 18),
    (17, 19), // rc, rd, rf, rg, rk, rl, rm, rn, rp, rr, rs, rt
    (18, 2),
    (18, 7),
    (18, 10),
    (18, 15),
    (18, 18),
    (18, 19), // sch, sh, sk, sp, ss, st
    (2, 19),
    (5, 5),
    (6, 18),
    (6, 19),
    (15, 19),
    (19, 19),
    (23, 19),
    (25, 25), // ct, ff, ghs, ght, pt, tt, xt, zz
];

/// Check if two-consonant coda cluster is valid in English
#[inline]
pub fn is_valid_coda(c1: u8, c2: u8) -> bool {
    CODA_PAIRS.iter().any(|&(a, b)| a == c1 && b == c2)
}

/// Check if coda cluster is valid using keycodes
#[inline]
pub fn is_valid_coda_keys(k1: u16, k2: u16) -> bool {
    let c1 = key_to_index(k1);
    let c2 = key_to_index(k2);
    if c1 == 255 || c2 == 255 {
        return false;
    }
    is_valid_coda(c1, c2)
}

// =============================================================================
// E3: Impossible Bigrams
// =============================================================================

/// For each letter (a=0..z=25), bitmask of letters that NEVER follow it
const IMPOSSIBLE_AFTER: [u32; 26] = [
    0x00000000, // a
    0x02000200, // b: bx, bz
    0x00200200, // c: cj, cv
    0x00010200, // d: dq, dx
    0x00000000, // e
    0x02010200, // f: fq, fx, fz
    0x00010200, // g: gq, gx
    0x02000200, // h: hx, hz
    0x00000000, // i
    0xFFEFBEEE, // j: only vowels valid
    0x02000200, // k: kx, kz
    0x00010200, // l: lq, lx
    0x00010200, // m: mq, mx
    0x00010200, // n: nq, nx
    0x00000000, // o
    0x00210200, // p: pq, pv, px
    0xFFEFFFFF, // q: only qu valid
    0x00010200, // r: rq, rx
    0x00000000, // s
    0x00010200, // t: tq, tx
    0x00000000, // u
    0x02D976E2, // v: many impossible
    0x02010200, // w: wq, wx, wz
    0x02D7FFEE, // x: many impossible
    0x00010200, // y: yq, yx
    0x02D936EE, // z: many impossible
];

/// Check if bigram is impossible in English
#[inline]
pub fn is_impossible_bigram(c1: u8, c2: u8) -> bool {
    if c1 >= 26 || c2 >= 26 {
        return true;
    }
    (IMPOSSIBLE_AFTER[c1 as usize] >> c2) & 1 == 1
}

/// Check if bigram is impossible using keycodes
#[inline]
pub fn is_impossible_bigram_keys(k1: u16, k2: u16) -> bool {
    let c1 = key_to_index(k1);
    let c2 = key_to_index(k2);
    if c1 == 255 || c2 == 255 {
        return true;
    }
    is_impossible_bigram(c1, c2)
}

// =============================================================================
// E4: Common Suffixes and Prefixes
// =============================================================================

/// Common English suffixes (as letter indices, reversed)
const SUFFIXES: &[[u8; 6]] = &[
    [3, 6, 13, 8, 0, 0],   // "ing"
    [2, 3, 4, 0, 0, 0],    // "ed"
    [2, 24, 11, 0, 0, 0],  // "ly"
    [4, 18, 18, 4, 13, 0], // "ness"
    [4, 13, 14, 8, 19, 0], // "tion"
    [4, 19, 13, 4, 12, 0], // "ment"
    [4, 4, 11, 1, 0, 0],   // "able"
    [2, 17, 4, 0, 0, 0],   // "er"
    [3, 19, 18, 4, 0, 0],  // "est"
    [3, 11, 20, 5, 0, 0],  // "ful"
];

/// Common English prefixes
const PREFIXES: &[[u8; 5]] = &[
    [2, 20, 13, 0, 0],  // "un"
    [2, 17, 4, 0, 0],   // "re"
    [3, 15, 17, 4, 0],  // "pre"
    [3, 3, 8, 18, 0],   // "dis"
    [3, 12, 8, 18, 0],  // "mis"
    [4, 14, 21, 4, 17], // "over"
    [2, 4, 23, 0, 0],   // "ex"
];

/// Check if word ends with common English suffix
#[inline]
pub fn has_english_suffix(indices: &[u8]) -> bool {
    if indices.len() < 2 {
        return false;
    }
    for suffix in SUFFIXES {
        let len = suffix[0] as usize;
        if indices.len() >= len {
            let word_end = &indices[indices.len() - len..];
            let suffix_chars = &suffix[1..=len];
            let mut matches = true;
            for i in 0..len {
                if word_end[len - 1 - i] != suffix_chars[i] {
                    matches = false;
                    break;
                }
            }
            if matches {
                return true;
            }
        }
    }
    false
}

/// Check if word starts with common English prefix
#[inline]
pub fn has_english_prefix(indices: &[u8]) -> bool {
    if indices.len() < 2 {
        return false;
    }
    for prefix in PREFIXES {
        let len = prefix[0] as usize;
        if indices.len() >= len {
            let word_start = &indices[..len];
            let prefix_chars = &prefix[1..=len];
            if word_start == prefix_chars {
                return true;
            }
        }
    }
    false
}

// =============================================================================
// E5: English Likelihood Scoring
// =============================================================================

/// English likelihood result
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnglishLikelihood {
    NotEnglish = 0,
    Possible = 1,
    Likely = 2,
    VeryLikely = 3,
}

/// Calculate English likelihood for a word
pub fn english_likelihood(indices: &[u8]) -> EnglishLikelihood {
    if indices.len() < 2 {
        return EnglishLikelihood::NotEnglish;
    }

    // Check for impossible bigrams
    for i in 0..indices.len() - 1 {
        if is_impossible_bigram(indices[i], indices[i + 1]) {
            return EnglishLikelihood::NotEnglish;
        }
    }

    let mut score = 1u8;

    // Check onset cluster
    if indices.len() >= 2 && !is_vowel_idx(indices[0]) && !is_vowel_idx(indices[1]) {
        if is_valid_onset(indices[0], indices[1]) {
            score += 1;
        }
    }

    // Check coda cluster
    let last_idx = indices.len() - 1;
    if indices.len() >= 2
        && !is_vowel_idx(indices[last_idx])
        && !is_vowel_idx(indices[last_idx - 1])
        && is_valid_coda(indices[last_idx - 1], indices[last_idx])
    {
        score += 1;
    }

    // Check patterns
    if has_english_suffix(indices) {
        score += 1;
    }
    if has_english_prefix(indices) {
        score += 1;
    }

    match score {
        0 => EnglishLikelihood::NotEnglish,
        1 => EnglishLikelihood::Possible,
        2 => EnglishLikelihood::Likely,
        _ => EnglishLikelihood::VeryLikely,
    }
}

/// Calculate English likelihood from keycodes
pub fn english_likelihood_keys(keys: &[u16]) -> EnglishLikelihood {
    let mut indices = [0u8; 32];
    let mut len = 0;
    for &key in keys {
        let idx = key_to_index(key);
        if idx == 255 {
            return EnglishLikelihood::NotEnglish;
        }
        if len < 32 {
            indices[len] = idx;
            len += 1;
        }
    }
    english_likelihood(&indices[..len])
}

// =============================================================================
// Bloom Filter for Dictionary
// =============================================================================

/// Simple Bloom filter for English dictionary words
pub struct BloomFilter {
    bits: Vec<u64>,
    num_hashes: u8,
}

impl BloomFilter {
    pub fn new(size_bits: usize, num_hashes: u8) -> Self {
        let num_words = size_bits.div_ceil(64);
        Self {
            bits: vec![0u64; num_words],
            num_hashes,
        }
    }

    pub fn from_bits(bits: Vec<u64>, num_hashes: u8) -> Self {
        Self { bits, num_hashes }
    }

    #[inline]
    fn hash(&self, word: &[u8], seed: u32) -> usize {
        let mut h = 2166136261u32 ^ seed;
        for &b in word {
            h ^= b as u32;
            h = h.wrapping_mul(16777619);
        }
        h as usize % (self.bits.len() * 64)
    }

    pub fn add(&mut self, word: &[u8]) {
        for i in 0..self.num_hashes {
            let bit_idx = self.hash(word, i as u32);
            let word_idx = bit_idx / 64;
            let bit_pos = bit_idx % 64;
            self.bits[word_idx] |= 1u64 << bit_pos;
        }
    }

    pub fn might_contain(&self, word: &[u8]) -> bool {
        for i in 0..self.num_hashes {
            let bit_idx = self.hash(word, i as u32);
            let word_idx = bit_idx / 64;
            let bit_pos = bit_idx % 64;
            if (self.bits[word_idx] >> bit_pos) & 1 == 0 {
                return false;
            }
        }
        true
    }

    pub fn might_contain_keys(&self, keys: &[u16]) -> bool {
        let mut word = [0u8; 32];
        let mut len = 0;
        for &key in keys {
            let idx = key_to_index(key);
            if idx == 255 || len >= 32 {
                return false;
            }
            word[len] = b'a' + idx;
            len += 1;
        }
        self.might_contain(&word[..len])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_onset() {
        assert!(is_valid_onset(1, 11)); // bl
        assert!(is_valid_onset(18, 19)); // st
        assert!(!is_valid_onset(1, 2)); // bc - invalid
    }

    #[test]
    fn test_valid_coda() {
        assert!(is_valid_coda(13, 6)); // ng
        assert!(is_valid_coda(18, 19)); // st
        assert!(!is_valid_coda(1, 3)); // bd - invalid
    }

    #[test]
    fn test_impossible_bigram() {
        assert!(is_impossible_bigram(16, 0)); // qa - impossible
        assert!(!is_impossible_bigram(16, 20)); // qu - valid
    }

    #[test]
    fn test_english_likelihood() {
        let testing = [19, 4, 18, 19, 8, 13, 6]; // testing
        assert!(matches!(
            english_likelihood(&testing),
            EnglishLikelihood::VeryLikely
        ));
    }
}
