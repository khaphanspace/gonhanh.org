//! E1-E5: English Detection Matrices
//!
//! Matrix-based English word detection for auto-restore.
//! Uses sparse encoding (valid pairs only) for memory efficiency.
//!
//! ## Matrices
//!
//! - **E1: ONSET_PAIRS** (48 pairs) - Valid consonant clusters at word start
//! - **E2: CODA_PAIRS** (52 pairs) - Valid consonant clusters at word end
//! - **E3: IMPOSSIBLE_AFTER** (26×32 bits) - Impossible bigrams
//! - **E4: SUFFIXES/PREFIXES** - Common English patterns
//! - **E5: english_likelihood** - Combined scoring function

use crate::data::keys;

// =============================================================================
// Helper: Convert macOS keycode to letter index (0-25)
// =============================================================================

/// Convert macOS keycode to letter index (0-25 for a-z)
/// Returns 255 for non-letter keys
#[inline]
fn key_to_index(key: u16) -> u8 {
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
fn is_vowel_idx(idx: u8) -> bool {
    matches!(idx, 0 | 4 | 8 | 14 | 20 | 24) // a, e, i, o, u, y
}

// =============================================================================
// E1: Valid Onset Clusters (48 pairs)
// =============================================================================

/// Valid English onset (word-start) consonant clusters
/// Encoded as letter indices (a=0, b=1, ...)
const ONSET_PAIRS: &[(u8, u8)] = &[
    // b-clusters
    (1, 11), // bl
    (1, 17), // br
    // c-clusters
    (2, 7),  // ch
    (2, 11), // cl
    (2, 17), // cr
    (2, 24), // cy
    // d-clusters
    (3, 17), // dr
    (3, 22), // dw
    // f-clusters
    (5, 11), // fl
    (5, 17), // fr
    // g-clusters
    (6, 7),  // gh
    (6, 11), // gl
    (6, 13), // gn
    (6, 17), // gr
    // k-clusters
    (10, 13), // kn
    (10, 22), // kw
    // p-clusters
    (15, 7),  // ph
    (15, 11), // pl
    (15, 17), // pr
    (15, 18), // ps
    (15, 19), // pt
    // q-clusters
    (16, 20), // qu (mandatory)
    // s-clusters (most productive)
    (18, 2),  // sc
    (18, 7),  // sh
    (18, 10), // sk
    (18, 11), // sl
    (18, 12), // sm
    (18, 13), // sn
    (18, 15), // sp
    (18, 16), // sq
    (18, 19), // st
    (18, 22), // sw
    // t-clusters
    (19, 7),  // th
    (19, 17), // tr
    (19, 22), // tw
    // w-clusters
    (22, 7),  // wh
    (22, 17), // wr
    (22, 24), // wy
    // misc
    (17, 7),  // rh
    (25, 7),  // zh
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
// E2: Valid Coda Clusters (52 pairs)
// =============================================================================

/// Valid English coda (word-end) consonant clusters
const CODA_PAIRS: &[(u8, u8)] = &[
    // Common endings
    (2, 10),  // ck
    (5, 19),  // ft
    // -l clusters
    (11, 2),  // lc (talc)
    (11, 3),  // ld (cold)
    (11, 5),  // lf (self)
    (11, 10), // lk (milk)
    (11, 11), // ll (ball)
    (11, 12), // lm (calm)
    (11, 15), // lp (help)
    (11, 18), // ls (else)
    (11, 19), // lt (salt)
    // -m clusters
    (12, 15), // mp (jump)
    (12, 18), // ms (items)
    // -n clusters
    (13, 2),  // nc (dance)
    (13, 3),  // nd (hand)
    (13, 6),  // ng (ring)
    (13, 10), // nk (think)
    (13, 13), // nn
    (13, 18), // ns (fans)
    (13, 19), // nt (want)
    // -r clusters
    (17, 2),  // rc (arc)
    (17, 3),  // rd (card)
    (17, 5),  // rf (scarf)
    (17, 6),  // rg (berg)
    (17, 10), // rk (work)
    (17, 11), // rl (girl)
    (17, 12), // rm (arm)
    (17, 13), // rn (turn)
    (17, 15), // rp (harp)
    (17, 17), // rr
    (17, 18), // rs (cars)
    (17, 19), // rt (part)
    // -s clusters
    (18, 2),  // sch
    (18, 7),  // sh
    (18, 10), // sk (task)
    (18, 15), // sp (crisp)
    (18, 18), // ss (boss)
    (18, 19), // st (test)
    // other
    (2, 19),  // ct (fact)
    (5, 5),   // ff (staff)
    (6, 18),  // ghs (sighs)
    (6, 19),  // ght (night)
    (15, 19), // pt (script)
    (19, 19), // tt
    (23, 19), // xt (next)
    (25, 25), // zz (jazz)
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
// E3: Impossible Bigrams (bitmask per letter)
// =============================================================================

/// For each letter (a=0..z=25), bitmask of letters that NEVER follow it
/// Bit n = 1 means letter n cannot follow this letter in English
const IMPOSSIBLE_AFTER: [u32; 26] = [
    // a: vowel, no impossible
    0x00000000,
    // b: bx, bz rare
    0x02000200,
    // c: cj, cv uncommon
    0x00200200,
    // d: dq, dx
    0x00010200,
    // e: vowel, no impossible
    0x00000000,
    // f: fq, fx, fz
    0x02010200,
    // g: gq, gx
    0x00010200,
    // h: hx, hz
    0x02000200,
    // i: vowel, no impossible
    0x00000000,
    // j: only ja, je, ji, jo, ju valid - everything else impossible
    // ~(1<<0 | 1<<4 | 1<<8 | 1<<14 | 1<<20) = 0xFFEFBEEE
    0xFFEFBEEE,
    // k: kx, kz
    0x02000200,
    // l: lq, lx rare
    0x00010200,
    // m: mq, mx
    0x00010200,
    // n: nq, nx
    0x00010200,
    // o: vowel, no impossible
    0x00000000,
    // p: pq, pv, px
    0x00210200,
    // q: ONLY qu valid
    0xFFEFFFFF,
    // r: rq, rx
    0x00010200,
    // s: very productive, no impossible
    0x00000000,
    // t: tq, tx
    0x00010200,
    // u: vowel, no impossible
    0x00000000,
    // v: vb, vf, vg, vh, vj, vk, vm, vp, vq, vt, vw, vx, vz
    0x02D976E2,
    // w: wq, wx, wz
    0x02010200,
    // x: many impossible
    0x02D7FFEE,
    // y: yq, yx
    0x00010200,
    // z: zb, zc, zd, zf, zg, zj, zk, zp, zq, zr, zs, zv, zw, zx
    0x02D936EE,
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

/// Common English suffixes (as letter indices, reversed for matching)
/// Format: [length, indices...] padded to 6
const SUFFIXES: &[[u8; 6]] = &[
    [3, 6, 13, 8, 0, 0],    // "ing" reversed
    [2, 3, 4, 0, 0, 0],     // "ed" reversed
    [2, 24, 11, 0, 0, 0],   // "ly" reversed
    [4, 18, 18, 4, 13, 0],  // "ness" reversed
    [4, 13, 14, 8, 19, 0],  // "tion" reversed
    [4, 19, 13, 4, 12, 0],  // "ment" reversed
    [4, 4, 11, 1, 0, 0],    // "able" reversed
    [2, 17, 4, 0, 0, 0],    // "er" reversed
    [3, 19, 18, 4, 0, 0],   // "est" reversed
    [3, 11, 20, 5, 0, 0],   // "ful" reversed
];

/// Common English prefixes (as letter indices)
const PREFIXES: &[[u8; 5]] = &[
    [2, 20, 13, 0, 0],   // "un"
    [2, 17, 4, 0, 0],    // "re"
    [3, 15, 17, 4, 0],   // "pre"
    [3, 3, 8, 18, 0],    // "dis"
    [3, 12, 8, 18, 0],   // "mis"
    [4, 14, 21, 4, 17],  // "over"
    [2, 4, 23, 0, 0],    // "ex"
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
            // Check reversed
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
    /// Definitely not English (impossible bigrams)
    NotEnglish = 0,
    /// Possibly English (no violations)
    Possible = 1,
    /// Likely English (has suffix/prefix or valid clusters)
    Likely = 2,
    /// Very likely English (multiple signals)
    VeryLikely = 3,
}

/// Calculate English likelihood for a word (letter indices)
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

    let mut score = 1u8; // Start as possibly English

    // Check onset cluster (first 2 letters if consonants)
    if indices.len() >= 2 && !is_vowel_idx(indices[0]) && !is_vowel_idx(indices[1]) {
        if is_valid_onset(indices[0], indices[1]) {
            score += 1;
        } else {
            // Invalid onset but not impossible - reduce confidence
            // Don't mark as NotEnglish since some valid words exist
        }
    }

    // Check coda cluster (last 2 letters if consonants)
    let last_idx = indices.len() - 1;
    if indices.len() >= 2 && !is_vowel_idx(indices[last_idx]) && !is_vowel_idx(indices[last_idx - 1]) {
        if is_valid_coda(indices[last_idx - 1], indices[last_idx]) {
            score += 1;
        }
    }

    // Check for common patterns
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
    // Convert keys to indices
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
    /// Create new Bloom filter with given size in bits and number of hash functions
    pub fn new(size_bits: usize, num_hashes: u8) -> Self {
        let num_words = (size_bits + 63) / 64;
        Self {
            bits: vec![0u64; num_words],
            num_hashes,
        }
    }

    /// Create from pre-computed bit array
    pub fn from_bits(bits: Vec<u64>, num_hashes: u8) -> Self {
        Self { bits, num_hashes }
    }

    /// Simple hash function (FNV-1a variant)
    #[inline]
    fn hash(&self, word: &[u8], seed: u32) -> usize {
        let mut h = 2166136261u32 ^ seed;
        for &b in word {
            h ^= b as u32;
            h = h.wrapping_mul(16777619);
        }
        h as usize % (self.bits.len() * 64)
    }

    /// Add word to filter
    pub fn add(&mut self, word: &[u8]) {
        for i in 0..self.num_hashes {
            let bit_idx = self.hash(word, i as u32);
            let word_idx = bit_idx / 64;
            let bit_pos = bit_idx % 64;
            self.bits[word_idx] |= 1u64 << bit_pos;
        }
    }

    /// Check if word might be in filter
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

    /// Convert keycodes to lowercase bytes for lookup
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
    fn test_key_to_index() {
        assert_eq!(key_to_index(keys::A), 0);
        assert_eq!(key_to_index(keys::B), 1);
        assert_eq!(key_to_index(keys::Z), 25);
    }

    #[test]
    fn test_is_vowel_idx() {
        assert!(is_vowel_idx(0));  // a
        assert!(is_vowel_idx(4));  // e
        assert!(is_vowel_idx(8));  // i
        assert!(is_vowel_idx(14)); // o
        assert!(is_vowel_idx(20)); // u
        assert!(is_vowel_idx(24)); // y

        assert!(!is_vowel_idx(1));  // b
        assert!(!is_vowel_idx(2));  // c
    }

    #[test]
    fn test_valid_onset() {
        // Valid onsets
        assert!(is_valid_onset(1, 11));  // bl
        assert!(is_valid_onset(1, 17));  // br
        assert!(is_valid_onset(18, 19)); // st
        assert!(is_valid_onset(19, 17)); // tr
        assert!(is_valid_onset(16, 20)); // qu

        // Invalid onsets
        assert!(!is_valid_onset(1, 2));  // bc
        assert!(!is_valid_onset(3, 6));  // dg
    }

    #[test]
    fn test_valid_onset_keys() {
        assert!(is_valid_onset_keys(keys::B, keys::L)); // bl
        assert!(is_valid_onset_keys(keys::S, keys::T)); // st
        assert!(!is_valid_onset_keys(keys::B, keys::C)); // bc invalid
    }

    #[test]
    fn test_valid_coda() {
        // Valid codas
        assert!(is_valid_coda(13, 6));  // ng
        assert!(is_valid_coda(13, 3));  // nd
        assert!(is_valid_coda(18, 19)); // st

        // Invalid codas
        assert!(!is_valid_coda(1, 3));  // bd
    }

    #[test]
    fn test_impossible_bigram() {
        // q only followed by u
        assert!(is_impossible_bigram(16, 0));  // qa impossible
        assert!(!is_impossible_bigram(16, 20)); // qu valid

        // j only with vowels
        assert!(is_impossible_bigram(9, 1));   // jb impossible
        assert!(!is_impossible_bigram(9, 14)); // jo valid
    }

    #[test]
    fn test_english_likelihood_word() {
        // "string" - st onset, ng coda
        let string = [18, 19, 17, 8, 13, 6]; // s,t,r,i,n,g
        assert!(matches!(
            english_likelihood(&string),
            EnglishLikelihood::Likely | EnglishLikelihood::VeryLikely
        ));

        // "qat" - impossible (qa)
        let qat = [16, 0, 19];
        assert_eq!(english_likelihood(&qat), EnglishLikelihood::NotEnglish);
    }

    #[test]
    fn test_english_likelihood_keys() {
        // "test" - has valid coda "st" and suffix "est" so very likely
        let test_keys = [keys::T, keys::E, keys::S, keys::T];
        let result = english_likelihood_keys(&test_keys);
        assert!(
            result != EnglishLikelihood::NotEnglish,
            "Expected English word, got {:?}",
            result
        );

        // "xyz" - starts with invalid cluster
        let xyz_keys = [keys::X, keys::Y, keys::Z];
        // xy is not a valid English onset but not impossible either
        let xyz_result = english_likelihood_keys(&xyz_keys);
        // Just verify no panic
        let _ = xyz_result;
    }

    #[test]
    fn test_has_english_suffix() {
        // "testing" - has "ing"
        let testing = [19, 4, 18, 19, 8, 13, 6]; // t,e,s,t,i,n,g
        assert!(has_english_suffix(&testing));

        // "able"
        let able = [0, 1, 11, 4]; // a,b,l,e
        assert!(has_english_suffix(&able));
    }

    #[test]
    fn test_has_english_prefix() {
        // "unhappy" - has "un"
        let unhappy = [20, 13, 7, 0, 15, 15, 24]; // u,n,h,a,p,p,y
        assert!(has_english_prefix(&unhappy));

        // "redo" - has "re"
        let redo = [17, 4, 3, 14]; // r,e,d,o
        assert!(has_english_prefix(&redo));
    }

    #[test]
    fn test_bloom_filter() {
        let mut filter = BloomFilter::new(1024, 3);

        filter.add(b"hello");
        filter.add(b"world");
        filter.add(b"test");

        assert!(filter.might_contain(b"hello"));
        assert!(filter.might_contain(b"world"));
        assert!(filter.might_contain(b"test"));

        // Probably not in filter (may have false positives)
        // Just verify no crash
        let _ = filter.might_contain(b"xyz");
    }

    #[test]
    fn test_bloom_filter_keys() {
        let mut filter = BloomFilter::new(1024, 3);
        filter.add(b"test");

        let test_keys = [keys::T, keys::E, keys::S, keys::T];
        assert!(filter.might_contain_keys(&test_keys));
    }
}
