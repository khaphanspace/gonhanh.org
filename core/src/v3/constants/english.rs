//! English detection patterns (Matrix-based, no Bloom filter)
//!
//! 8-layer detection system (~500 bytes total):
//! 1. Invalid Vietnamese initials (F, J, W, Z, pr, cl, str, etc.) - Certain (100%)
//! 2. Onset clusters (bl, br, cl, cr, dr, fl, fr, gr, pl, pr, st, tr, etc.) - 98%
//! 3. Double consonants (ll, ss, ff, tt, pp, mm, nn, rr, etc.) - 95%
//! 4. English suffixes (-tion, -ing, -ed, -ly, -ness, -ment, etc.) - 90%
//! 5. Coda clusters (st, nd, nt, ld, nk, mp, lt, ft, pt, ct, xt, etc.) - 90%
//! 6. English prefixes (un-, re-, pre-, dis-, mis-, etc.) - 75%
//! 7. Invalid vowel patterns (ea, ou, yo, oo, etc.) - 85%
//! 8. Impossible bigrams in Vietnamese - 80%
//!
//! Memory: ~500 bytes with O(1) matrix lookups
//! Target: 98% coverage on top 20,000 English words

// ============================================================================
// TIER 1: Invalid Vietnamese Initials
// ============================================================================

/// Invalid Vietnamese initials - always English
/// F, J, W, Z are not used in native Vietnamese words
pub const INVALID_VN_INITIALS: [char; 4] = ['f', 'j', 'w', 'z'];

/// Quick lookup: is this char an invalid VN initial?
/// Indices for f(5), j(9), w(22), z(25) in a-z
#[inline]
pub fn is_invalid_vn_initial(c: char) -> bool {
    matches!(c.to_ascii_lowercase(), 'f' | 'j' | 'w' | 'z')
}

// ============================================================================
// TIER 2: English-only Onset Clusters (96 bytes)
// ============================================================================

/// Onset cluster matrix indices for first consonant
pub mod onset_c1 {
    pub const B: usize = 0;
    pub const C: usize = 1;
    pub const D: usize = 2;
    pub const F: usize = 3;
    pub const G: usize = 4;
    pub const P: usize = 5;
    pub const S: usize = 6;
    pub const T: usize = 7;
    pub const W: usize = 8;
    pub const COUNT: usize = 9;
}

/// Onset cluster matrix indices for second consonant
pub mod onset_c2 {
    pub const L: usize = 0;
    pub const R: usize = 1;
    pub const W: usize = 2;
    pub const C: usize = 3;
    pub const K: usize = 4;
    pub const M: usize = 5;
    pub const N: usize = 6;
    pub const P: usize = 7;
    pub const T: usize = 8;
    pub const H: usize = 9;
    pub const COUNT: usize = 10;
}

/// English-only onset consonant clusters
/// Matrix: first_consonant x second_consonant -> is_en_onset
/// Covers: bl, br, cl, cr, dr, dw, fl, fr, gl, gr, pl, pr,
///         sc, sk, sl, sm, sn, sp, st, sw, tr, tw, wr
/// Note: th is NOT included (valid Vietnamese: thành, thanh, thì)
/// Size: 9 x 10 = 90 bytes (rounded to 96)
pub static E_ONSET_PAIRS: [[u8; 10]; 9] = [
    //          L  R  W  C  K  M  N  P  T  H
    /* B */
    [1, 1, 0, 0, 0, 0, 0, 0, 0, 0], // bl, br
    /* C */ [1, 1, 0, 0, 0, 0, 0, 0, 0, 0], // cl, cr
    /* D */ [0, 1, 1, 0, 0, 0, 0, 0, 0, 0], // dr, dw
    /* F */ [1, 1, 0, 0, 0, 0, 0, 0, 0, 0], // fl, fr
    /* G */ [1, 1, 0, 0, 0, 0, 0, 0, 0, 0], // gl, gr
    /* P */ [1, 1, 0, 0, 0, 0, 0, 0, 0, 0], // pl, pr
    /* S */ [1, 0, 1, 1, 1, 1, 1, 1, 1, 0], // sl,sw,sc,sk,sm,sn,sp,st
    /* T */ [0, 1, 1, 0, 0, 0, 0, 0, 0, 0], // tr, tw (th removed - valid VN)
    /* W */ [0, 1, 0, 0, 0, 0, 0, 0, 0, 0], // wr
];

/// Map first consonant to onset_c1 index
#[inline]
fn onset_c1_index(c: u8) -> Option<usize> {
    match c {
        b'b' | b'B' => Some(onset_c1::B),
        b'c' | b'C' => Some(onset_c1::C),
        b'd' | b'D' => Some(onset_c1::D),
        b'f' | b'F' => Some(onset_c1::F),
        b'g' | b'G' => Some(onset_c1::G),
        b'p' | b'P' => Some(onset_c1::P),
        b's' | b'S' => Some(onset_c1::S),
        b't' | b'T' => Some(onset_c1::T),
        b'w' | b'W' => Some(onset_c1::W),
        _ => None,
    }
}

/// Map second consonant to onset_c2 index
#[inline]
fn onset_c2_index(c: u8) -> Option<usize> {
    match c {
        b'l' | b'L' => Some(onset_c2::L),
        b'r' | b'R' => Some(onset_c2::R),
        b'w' | b'W' => Some(onset_c2::W),
        b'c' | b'C' => Some(onset_c2::C),
        b'k' | b'K' => Some(onset_c2::K),
        b'm' | b'M' => Some(onset_c2::M),
        b'n' | b'N' => Some(onset_c2::N),
        b'p' | b'P' => Some(onset_c2::P),
        b't' | b'T' => Some(onset_c2::T),
        b'h' | b'H' => Some(onset_c2::H),
        _ => None,
    }
}

/// Check if two consonants form English-only onset cluster (O(1) lookup)
#[inline]
pub fn is_en_onset_pair(c1: u8, c2: u8) -> bool {
    if let (Some(i1), Some(i2)) = (onset_c1_index(c1), onset_c2_index(c2)) {
        E_ONSET_PAIRS[i1][i2] != 0
    } else {
        false
    }
}

// ============================================================================
// LAYER 3: Double Consonants (13 bytes)
// ============================================================================

/// Double consonants - Vietnamese NEVER uses doubled consonants
/// (uses digraphs like ng, nh, ch instead)
pub const E_DOUBLE_CONSONANTS: [u8; 13] = [
    b'l', // ll - tell, all, will
    b's', // ss - class, glass, pass
    b'f', // ff - off, staff, coffee
    b't', // tt - letter, better
    b'p', // pp - happy, apple
    b'm', // mm - summer, hammer
    b'n', // nn - dinner, funny
    b'r', // rr - sorry, worry
    b'd', // dd - add, ladder
    b'g', // gg - egg, bigger
    b'b', // bb - rabbit, hobby
    b'z', // zz - buzz, pizza
    b'c', // cc - access, success
];

/// Check if word contains doubled consonant (O(n) scan)
#[inline]
pub fn has_double_consonant(word: &[u8]) -> bool {
    if word.len() < 2 {
        return false;
    }

    for i in 0..word.len() - 1 {
        let c1 = word[i].to_ascii_lowercase();
        let c2 = word[i + 1].to_ascii_lowercase();

        if c1 == c2 && E_DOUBLE_CONSONANTS.contains(&c1) {
            return true;
        }
    }
    false
}

// ============================================================================
// LAYER 5: English-only Coda Clusters (90 bytes)
// ============================================================================

/// Coda cluster matrix indices for first consonant
pub mod coda_c1 {
    pub const C: usize = 0;
    pub const F: usize = 1;
    pub const L: usize = 2;
    pub const M: usize = 3;
    pub const N: usize = 4;
    pub const P: usize = 5;
    pub const R: usize = 6;
    pub const S: usize = 7;
    pub const X: usize = 8;
    pub const COUNT: usize = 9;
}

/// Coda cluster matrix indices for second consonant
pub mod coda_c2 {
    pub const B: usize = 0;
    pub const D: usize = 1;
    pub const F: usize = 2;
    pub const K: usize = 3;
    pub const L: usize = 4;
    pub const M: usize = 5;
    pub const N: usize = 6;
    pub const P: usize = 7;
    pub const T: usize = 8;
    pub const V: usize = 9;
    pub const COUNT: usize = 10;
}

/// English-only coda consonant clusters
/// Matrix: first_consonant x second_consonant -> is_en_coda
/// Covers: ct, ft, ld, lf, lk, lm, lp, lt, lv, mp, nd, nk, nt, pt,
///         rd, rk, rm, rn, rp, rt, sk, sp, st, xt
/// Size: 9 x 10 = 90 bytes
pub static E_CODA_PAIRS: [[u8; 10]; 9] = [
    //          B  D  F  K  L  M  N  P  T  V
    /* C */
    [0, 0, 0, 0, 0, 0, 0, 0, 1, 0], // ct (fact, act)
    /* F */ [0, 0, 0, 0, 0, 0, 0, 0, 1, 0], // ft (left, soft)
    /* L */ [0, 1, 1, 1, 0, 1, 0, 1, 1, 1], // ld,lf,lk,lm,lp,lt,lv
    /* M */ [1, 0, 0, 0, 0, 0, 0, 1, 0, 0], // mb, mp
    /* N */ [0, 1, 0, 1, 0, 0, 0, 0, 1, 0], // nd, nk, nt
    /* P */ [0, 0, 0, 0, 0, 0, 0, 0, 1, 0], // pt (script)
    /* R */ [1, 1, 0, 1, 1, 1, 1, 1, 1, 0], // rb,rd,rk,rl,rm,rn,rp,rt
    /* S */ [0, 0, 0, 1, 0, 0, 0, 1, 1, 0], // sk, sp, st
    /* X */ [0, 0, 0, 0, 0, 0, 0, 0, 1, 0], // xt (text, next)
];

/// Map first consonant to coda_c1 index
#[inline]
fn coda_c1_index(c: u8) -> Option<usize> {
    match c {
        b'c' | b'C' => Some(coda_c1::C),
        b'f' | b'F' => Some(coda_c1::F),
        b'l' | b'L' => Some(coda_c1::L),
        b'm' | b'M' => Some(coda_c1::M),
        b'n' | b'N' => Some(coda_c1::N),
        b'p' | b'P' => Some(coda_c1::P),
        b'r' | b'R' => Some(coda_c1::R),
        b's' | b'S' => Some(coda_c1::S),
        b'x' | b'X' => Some(coda_c1::X),
        _ => None,
    }
}

/// Map second consonant to coda_c2 index
#[inline]
fn coda_c2_index(c: u8) -> Option<usize> {
    match c {
        b'b' | b'B' => Some(coda_c2::B),
        b'd' | b'D' => Some(coda_c2::D),
        b'f' | b'F' => Some(coda_c2::F),
        b'k' | b'K' => Some(coda_c2::K),
        b'l' | b'L' => Some(coda_c2::L),
        b'm' | b'M' => Some(coda_c2::M),
        b'n' | b'N' => Some(coda_c2::N),
        b'p' | b'P' => Some(coda_c2::P),
        b't' | b'T' => Some(coda_c2::T),
        b'v' | b'V' => Some(coda_c2::V),
        _ => None,
    }
}

/// Check if two consonants form English-only coda cluster (O(1) lookup)
#[inline]
pub fn is_en_coda_pair(c1: u8, c2: u8) -> bool {
    if let (Some(i1), Some(i2)) = (coda_c1_index(c1), coda_c2_index(c2)) {
        E_CODA_PAIRS[i1][i2] != 0
    } else {
        false
    }
}

// ============================================================================
// LAYER 8: Impossible Bigrams in Vietnamese (104 bytes)
// ============================================================================

/// Impossible bigrams: any occurrence = strong English signal
/// Matrix: char1 (a-z) x char2 (b,k,x,z) -> impossible_in_vn
/// Size: 26 x 4 = 104 bytes
pub static E_IMPOSSIBLE_BIGRAM: [[u8; 4]; 26] = [
    //      B  K  X  Z
    /* A */ [0, 0, 0, 0],
    /* B */ [0, 1, 0, 0], // bk impossible
    /* C */ [1, 0, 0, 0], // cb impossible
    /* D */ [0, 1, 0, 0], // dk impossible
    /* E */ [0, 0, 0, 0],
    /* F */ [0, 0, 0, 0], // f is rare anyway
    /* G */ [0, 1, 0, 0], // gk impossible
    /* H */ [1, 0, 0, 0], // hb impossible
    /* I */ [0, 0, 0, 0],
    /* J */ [1, 1, 1, 1], // j* all impossible in VN
    /* K */ [1, 0, 1, 1], // kb, kx, kz impossible
    /* L */ [0, 0, 0, 0],
    /* M */ [0, 0, 0, 0],
    /* N */ [0, 0, 0, 0],
    /* O */ [0, 0, 0, 0],
    /* P */ [1, 0, 0, 0], // pb impossible
    /* Q */ [1, 1, 1, 1], // q* without u impossible
    /* R */ [0, 0, 0, 0],
    /* S */ [0, 0, 0, 0],
    /* T */ [1, 0, 0, 0], // tb impossible
    /* U */ [0, 0, 0, 0],
    /* V */ [1, 0, 0, 0], // vb impossible
    /* W */ [1, 1, 1, 1], // w* all foreign in VN
    /* X */ [1, 0, 0, 0], // xb impossible
    /* Y */ [0, 0, 0, 0],
    /* Z */ [1, 1, 1, 1], // z* all foreign in VN
];

/// Check if bigram is impossible in Vietnamese (O(1) lookup)
#[inline]
pub fn is_impossible_bigram(c1: u8, c2: u8) -> bool {
    let c1_lower = c1.to_ascii_lowercase();
    let c2_lower = c2.to_ascii_lowercase();

    if c1_lower < b'a' || c1_lower > b'z' {
        return false;
    }

    let c2_idx = match c2_lower {
        b'b' => 0,
        b'k' => 1,
        b'x' => 2,
        b'z' => 3,
        _ => return false,
    };

    E_IMPOSSIBLE_BIGRAM[(c1_lower - b'a') as usize][c2_idx] != 0
}

// ============================================================================
// LAYER 4: English Suffixes (80 bytes)
// ============================================================================

/// English suffix patterns: [length, b1, b2, b3, b4]
/// Max 4 chars per suffix
/// Size: 16 x 5 = 80 bytes
pub static E_SUFFIXES: [[u8; 5]; 16] = [
    [4, b't', b'i', b'o', b'n'], // tion
    [4, b's', b'i', b'o', b'n'], // sion
    [4, b'n', b'e', b's', b's'], // ness
    [4, b'm', b'e', b'n', b't'], // ment
    [4, b'a', b'b', b'l', b'e'], // able
    [4, b'i', b'b', b'l', b'e'], // ible
    [3, b'f', b'u', b'l', 0],    // ful
    [4, b'l', b'e', b's', b's'], // less
    [3, b'i', b'n', b'g', 0],    // ing
    [3, b'o', b'u', b's', 0],    // ous
    [3, b'i', b'v', b'e', 0],    // ive
    [3, b'i', b'z', b'e', 0],    // ize
    [3, b'i', b's', b'e', 0],    // ise
    [3, b'i', b't', b'y', 0],    // ity
    [2, b'l', b'y', 0, 0],       // ly
    [2, b'e', b'd', 0, 0],       // ed
];

/// Check if word ends with English suffix
pub fn has_en_suffix(word: &[u8]) -> bool {
    let len = word.len();
    if len < 3 {
        return false;
    }

    for suffix in &E_SUFFIXES {
        let slen = suffix[0] as usize;
        if len > slen {
            let start = len - slen;
            let mut matches = true;
            for i in 0..slen {
                if word[start + i].to_ascii_lowercase() != suffix[1 + i] {
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

// ============================================================================
// LAYER 6: English Prefixes (40 bytes)
// ============================================================================

/// English prefix patterns: [length, b1, b2, b3, b4]
/// Max 4 chars per prefix
/// Size: 8 x 5 = 40 bytes
pub static E_PREFIXES: [[u8; 5]; 8] = [
    [2, b'u', b'n', 0, 0],       // un-
    [2, b'r', b'e', 0, 0],       // re-
    [3, b'p', b'r', b'e', 0],    // pre-
    [3, b'd', b'i', b's', 0],    // dis-
    [3, b'm', b'i', b's', 0],    // mis-
    [4, b'o', b'v', b'e', b'r'], // over-
    [3, b'o', b'u', b't', 0],    // out-
    [3, b's', b'u', b'b', 0],    // sub-
];

/// Check if word starts with English prefix
pub fn has_en_prefix(word: &[u8]) -> bool {
    let len = word.len();
    if len < 4 {
        // Prefix + at least 2 chars for word
        return false;
    }

    for prefix in &E_PREFIXES {
        let plen = prefix[0] as usize;
        if len > plen + 1 {
            // Need more than just prefix
            let mut matches = true;
            for i in 0..plen {
                if word[i].to_ascii_lowercase() != prefix[1 + i] {
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

// ============================================================================
// LAYER 7: Invalid Vietnamese Vowel Patterns
// ============================================================================

/// Vowel patterns impossible/rare in Vietnamese
/// ea, ee, oo, ou, ei, eu (removed ambiguous: ie, ue, io)
/// Note: ie/ue/io excluded - ambiguous with Vietnamese iê/uê/iô
pub static E_VOWEL_PATTERNS: [[u8; 2]; 6] = [
    [b'e', b'a'], // ea (search, beach)
    [b'e', b'e'], // ee (see, tree)
    [b'o', b'o'], // oo (good, food)
    [b'o', b'u'], // ou (you, out)
    [b'e', b'i'], // ei (receive)
    [b'e', b'u'], // eu (neutral)
];

/// Check if word has impossible Vietnamese vowel pattern
pub fn has_invalid_vn_vowel_pattern(word: &[u8]) -> bool {
    if word.len() < 2 {
        return false;
    }

    for i in 0..word.len() - 1 {
        let c1 = word[i].to_ascii_lowercase();
        let c2 = word[i + 1].to_ascii_lowercase();

        // Skip if next char has Vietnamese diacritic
        // (would be multi-byte UTF-8, safe to check ASCII here)
        for pattern in &E_VOWEL_PATTERNS {
            if c1 == pattern[0] && c2 == pattern[1] {
                return true;
            }
        }
    }
    false
}

// ============================================================================
// CONFIDENCE SCORING
// ============================================================================

/// English detection confidence level (8-layer system)
/// Matches validation-algorithm.md exactly
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum EnglishConfidence {
    /// Definitely Vietnamese (0%)
    None = 0,
    /// Layer 8: Has impossible bigram for VN (80%)
    ImpossibleBigram = 80,
    /// Layer 7: Invalid VN vowel pattern (85%)
    VowelPattern = 85,
    /// Layer 6: Has common EN prefix (75%)
    HasPrefix = 75,
    /// Layer 4: Has common EN suffix (90%)
    HasSuffix = 90,
    /// Layer 5: Has EN-only coda cluster (91%)
    CodaCluster = 91,
    /// Layer 3: Has double consonant (95%)
    DoubleConsonant = 95,
    /// Layer 2: Has EN-only onset cluster (98%)
    OnsetCluster = 98,
    /// Layer 1: Invalid VN initial (100%)
    Certain = 100,
}

// Legacy compatibility aliases
#[allow(non_upper_case_globals)]
impl EnglishConfidence {
    pub const Low: Self = Self::ImpossibleBigram;
    pub const Medium: Self = Self::HasSuffix;
    pub const High: Self = Self::DoubleConsonant;
}

/// Calculate English confidence for a word (8-layer system)
/// Returns highest confidence layer matched
/// Order: Layer 1 > 2 > 3 > 4 > 5 > 6 > 7 > 8
pub fn english_confidence(word: &str) -> EnglishConfidence {
    let bytes = word.as_bytes();
    if bytes.is_empty() {
        return EnglishConfidence::None;
    }

    // Layer 1: Invalid VN initials (F, J, W, Z) = Certain (100%)
    if let Some(&first) = bytes.first() {
        if is_invalid_vn_initial(first as char) {
            return EnglishConfidence::Certain;
        }
    }

    // Layer 2: English-only onset clusters = OnsetCluster (98%)
    if bytes.len() >= 2 && is_en_onset_pair(bytes[0], bytes[1]) {
        return EnglishConfidence::OnsetCluster;
    }

    // Layer 3: Double consonants = DoubleConsonant (95%)
    if has_double_consonant(bytes) {
        return EnglishConfidence::DoubleConsonant;
    }

    // Layer 4: English suffixes = HasSuffix (90%)
    if has_en_suffix(bytes) {
        return EnglishConfidence::HasSuffix;
    }

    // Layer 5: English-only coda clusters = CodaCluster (90%)
    if bytes.len() >= 2 {
        let last2 = &bytes[bytes.len() - 2..];
        if is_en_coda_pair(last2[0], last2[1]) {
            return EnglishConfidence::CodaCluster;
        }
    }

    // Layer 6: English prefixes = HasPrefix (75%)
    if has_en_prefix(bytes) {
        return EnglishConfidence::HasPrefix;
    }

    // Layer 7: Invalid VN vowel patterns = VowelPattern (85%)
    if has_invalid_vn_vowel_pattern(bytes) {
        return EnglishConfidence::VowelPattern;
    }

    // Layer 8: Impossible bigrams = ImpossibleBigram (80%)
    for i in 0..bytes.len().saturating_sub(1) {
        if is_impossible_bigram(bytes[i], bytes[i + 1]) {
            return EnglishConfidence::ImpossibleBigram;
        }
    }

    EnglishConfidence::None
}

/// Check if confidence is high enough to trigger restore
/// Threshold: CodaCluster (90%) or higher
#[inline]
pub fn should_restore_english(confidence: EnglishConfidence) -> bool {
    confidence >= EnglishConfidence::HasSuffix // 90% threshold
}

// ============================================================================
// LEGACY COMPATIBILITY - String-based API
// ============================================================================

/// Check if word starts with invalid Vietnamese initial
#[inline]
pub fn has_invalid_initial(word: &str) -> bool {
    word.chars()
        .next()
        .map(|c| is_invalid_vn_initial(c))
        .unwrap_or(false)
}

/// Check if word contains impossible onset cluster
pub fn has_impossible_onset(word: &str) -> bool {
    let bytes = word.as_bytes();
    bytes.len() >= 2 && is_en_onset_pair(bytes[0], bytes[1])
}

/// Check if word contains impossible coda cluster
pub fn has_impossible_coda(word: &str) -> bool {
    let bytes = word.as_bytes();
    if bytes.len() >= 2 {
        let last2 = &bytes[bytes.len() - 2..];
        is_en_coda_pair(last2[0], last2[1])
    } else {
        false
    }
}

/// Check if word has English suffix
pub fn has_english_suffix(word: &str) -> bool {
    has_en_suffix(word.as_bytes())
}

/// Check if word has English prefix
pub fn has_english_prefix(word: &str) -> bool {
    has_en_prefix(word.as_bytes())
}

/// Check if word has doubled consonant (String API)
pub fn has_doubled_consonant(word: &str) -> bool {
    has_double_consonant(word.as_bytes())
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_initials() {
        assert!(has_invalid_initial("file"));
        assert!(has_invalid_initial("jazz"));
        assert!(has_invalid_initial("web"));
        assert!(has_invalid_initial("zone"));
        assert!(!has_invalid_initial("text"));
        assert!(!has_invalid_initial("ban"));
    }

    #[test]
    fn test_onset_matrix() {
        // Valid EN-only onsets
        assert!(is_en_onset_pair(b'b', b'l')); // bl
        assert!(is_en_onset_pair(b'b', b'r')); // br
        assert!(is_en_onset_pair(b'c', b'l')); // cl
        assert!(is_en_onset_pair(b'c', b'r')); // cr
        assert!(is_en_onset_pair(b's', b't')); // st
        assert!(is_en_onset_pair(b't', b'r')); // tr
        assert!(is_en_onset_pair(b'w', b'r')); // wr

        // Invalid (valid in VN or doesn't exist)
        assert!(!is_en_onset_pair(b'n', b'g')); // ng valid in VN
        assert!(!is_en_onset_pair(b't', b'h')); // th valid in VN (thành, thanh)
        assert!(!is_en_onset_pair(b't', b'a')); // not a cluster
    }

    #[test]
    fn test_coda_matrix() {
        // Valid EN-only codas
        assert!(is_en_coda_pair(b'x', b't')); // xt (text)
        assert!(is_en_coda_pair(b'l', b'd')); // ld (world)
        assert!(is_en_coda_pair(b'n', b't')); // nt (point)
        assert!(is_en_coda_pair(b's', b't')); // st (test)
        assert!(is_en_coda_pair(b'f', b't')); // ft (left)
        assert!(is_en_coda_pair(b'l', b'k')); // lk (milk)

        // Invalid (valid in VN or doesn't exist)
        assert!(!is_en_coda_pair(b'n', b'g')); // ng valid in VN
        assert!(!is_en_coda_pair(b'c', b'h')); // ch valid in VN
    }

    #[test]
    fn test_impossible_bigrams() {
        assert!(is_impossible_bigram(b'b', b'k')); // bk
        assert!(is_impossible_bigram(b'c', b'b')); // cb
        assert!(is_impossible_bigram(b'j', b'b')); // jb
        assert!(is_impossible_bigram(b'w', b'z')); // wz
        assert!(!is_impossible_bigram(b'a', b'n')); // an - valid
    }

    #[test]
    fn test_impossible_onsets() {
        assert!(has_impossible_onset("black"));
        assert!(has_impossible_onset("class"));
        assert!(has_impossible_onset("string"));
        assert!(has_impossible_onset("tray")); // tr is EN-only
        assert!(!has_impossible_onset("throw")); // th is valid VN
        assert!(!has_impossible_onset("text")); // te not an onset cluster
    }

    #[test]
    fn test_impossible_codas() {
        assert!(has_impossible_coda("text")); // xt
        assert!(has_impossible_coda("world")); // ld
        assert!(has_impossible_coda("point")); // nt
        assert!(has_impossible_coda("test")); // st
        assert!(!has_impossible_coda("ban")); // n - valid VN final
        assert!(!has_impossible_coda("cam")); // m - valid VN final
    }

    #[test]
    fn test_english_suffixes() {
        assert!(has_english_suffix("action")); // tion
        assert!(has_english_suffix("running")); // ing
        assert!(has_english_suffix("beautiful")); // ful
        assert!(has_english_suffix("happiness")); // ness
        assert!(!has_english_suffix("ban"));
        assert!(!has_english_suffix("ing")); // too short
    }

    #[test]
    fn test_english_prefixes() {
        assert!(has_english_prefix("undo"));
        assert!(has_english_prefix("return"));
        assert!(has_english_prefix("preview"));
        assert!(has_english_prefix("disconnect"));
        assert!(!has_english_prefix("ban"));
        assert!(!has_english_prefix("un")); // too short
    }

    #[test]
    fn test_double_consonants() {
        assert!(has_doubled_consonant("coffee")); // ff
        assert!(has_doubled_consonant("class")); // ss
        assert!(has_doubled_consonant("letter")); // tt
        assert!(has_doubled_consonant("happy")); // pp
        assert!(has_doubled_consonant("all")); // ll
        assert!(has_doubled_consonant("add")); // dd
        assert!(has_doubled_consonant("egg")); // gg
        assert!(has_doubled_consonant("buzz")); // zz
        assert!(has_doubled_consonant("access")); // cc
        assert!(!has_doubled_consonant("ban"));
        assert!(!has_doubled_consonant("text")); // no double
    }

    #[test]
    fn test_vowel_patterns() {
        assert!(has_invalid_vn_vowel_pattern(b"search")); // ea
        assert!(has_invalid_vn_vowel_pattern(b"tree")); // ee
        assert!(has_invalid_vn_vowel_pattern(b"good")); // oo
        assert!(has_invalid_vn_vowel_pattern(b"you")); // ou
        assert!(has_invalid_vn_vowel_pattern(b"receive")); // ei
        assert!(!has_invalid_vn_vowel_pattern(b"ban"));
        assert!(!has_invalid_vn_vowel_pattern(b"viet")); // ie ambiguous with iê
        assert!(!has_invalid_vn_vowel_pattern(b"blue")); // ue ambiguous with uê
    }

    #[test]
    fn test_confidence_8_layers() {
        // Layer 1: Invalid initial = Certain (100%)
        assert_eq!(english_confidence("file"), EnglishConfidence::Certain);
        assert_eq!(english_confidence("jazz"), EnglishConfidence::Certain);
        assert_eq!(english_confidence("web"), EnglishConfidence::Certain);

        // Layer 2: Onset cluster = OnsetCluster (98%)
        assert_eq!(english_confidence("class"), EnglishConfidence::OnsetCluster);
        assert_eq!(
            english_confidence("string"),
            EnglishConfidence::OnsetCluster
        );
        assert_eq!(english_confidence("black"), EnglishConfidence::OnsetCluster);

        // Layer 3: Double consonant = DoubleConsonant (95%)
        assert_eq!(
            english_confidence("coffee"),
            EnglishConfidence::DoubleConsonant
        );
        assert_eq!(
            english_confidence("letter"),
            EnglishConfidence::DoubleConsonant
        );
        assert_eq!(
            english_confidence("happy"),
            EnglishConfidence::DoubleConsonant
        );

        // Layer 4: Suffix = HasSuffix (90%)
        assert_eq!(english_confidence("action"), EnglishConfidence::HasSuffix);
        assert_eq!(english_confidence("nation"), EnglishConfidence::HasSuffix);

        // Layer 5: Coda cluster = CodaCluster (90%)
        assert_eq!(english_confidence("text"), EnglishConfidence::CodaCluster);
        assert_eq!(english_confidence("test"), EnglishConfidence::CodaCluster);

        // Layer 6: Prefix = HasPrefix (75%)
        assert_eq!(english_confidence("undo"), EnglishConfidence::HasPrefix);

        // Layer 7: Vowel pattern = VowelPattern (85%)
        assert_eq!(
            english_confidence("search"),
            EnglishConfidence::VowelPattern
        );

        // No match = None
        assert_eq!(english_confidence("ban"), EnglishConfidence::None);
        assert_eq!(english_confidence("viet"), EnglishConfidence::None);
    }

    #[test]
    fn test_legacy_confidence_aliases() {
        // Ensure legacy aliases work
        assert!(EnglishConfidence::High >= EnglishConfidence::DoubleConsonant);
        assert!(EnglishConfidence::Medium >= EnglishConfidence::HasSuffix);
        assert!(EnglishConfidence::Low >= EnglishConfidence::ImpossibleBigram);
    }

    #[test]
    fn test_should_restore() {
        assert!(should_restore_english(EnglishConfidence::Certain));
        assert!(should_restore_english(EnglishConfidence::OnsetCluster));
        assert!(should_restore_english(EnglishConfidence::DoubleConsonant));
        assert!(should_restore_english(EnglishConfidence::HasSuffix)); // 90% threshold
        assert!(should_restore_english(EnglishConfidence::CodaCluster));
        assert!(!should_restore_english(EnglishConfidence::HasPrefix));
        assert!(!should_restore_english(EnglishConfidence::None));
    }

    #[test]
    fn test_memory_size() {
        // Verify memory budget ~500 bytes
        let onset_size = core::mem::size_of_val(&E_ONSET_PAIRS); // 90 bytes
        let coda_size = core::mem::size_of_val(&E_CODA_PAIRS); // 90 bytes
        let bigram_size = core::mem::size_of_val(&E_IMPOSSIBLE_BIGRAM); // 104 bytes
        let suffix_size = core::mem::size_of_val(&E_SUFFIXES); // 80 bytes
        let prefix_size = core::mem::size_of_val(&E_PREFIXES); // 40 bytes
        let vowel_size = core::mem::size_of_val(&E_VOWEL_PATTERNS); // 12 bytes
        let double_size = core::mem::size_of_val(&E_DOUBLE_CONSONANTS); // 13 bytes

        let total = onset_size
            + coda_size
            + bigram_size
            + suffix_size
            + prefix_size
            + vowel_size
            + double_size;
        assert!(total <= 500, "Memory budget exceeded: {} bytes", total);
        println!("English detection memory: {} bytes", total);
    }
}
