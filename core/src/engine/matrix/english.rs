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
    (17, 7), // rh
    (25, 7), // zh
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
    (2, 10), // ck
    (5, 19), // ft
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
    0x00000000, // b: bx, bz rare
    0x02000200, // c: cj, cv uncommon
    0x00200200, // d: dq, dx
    0x00010200, // e: vowel, no impossible
    0x00000000, // f: fq, fx, fz
    0x02010200, // g: gq, gx
    0x00010200, // h: hx, hz
    0x02000200, // i: vowel, no impossible
    0x00000000,
    // j: only ja, je, ji, jo, ju valid - everything else impossible
    // ~(1<<0 | 1<<4 | 1<<8 | 1<<14 | 1<<20) = 0xFFEFBEEE
    0xFFEFBEEE, // k: kx, kz
    0x02000200, // l: lq, lx rare
    0x00010200, // m: mq, mx
    0x00010200, // n: nq, nx
    0x00010200, // o: vowel, no impossible
    0x00000000, // p: pq, pv, px
    0x00210200, // q: ONLY qu valid
    0xFFEFFFFF, // r: rq, rx
    0x00010200, // s: very productive, no impossible
    0x00000000, // t: tq, tx
    0x00010200, // u: vowel, no impossible
    0x00000000, // v: vb, vf, vg, vh, vj, vk, vm, vp, vq, vt, vw, vx, vz
    0x02D976E2, // w: wq, wx, wz
    0x02010200, // x: many impossible
    0x02D7FFEE, // y: yq, yx
    0x00010200, // z: zb, zc, zd, zf, zg, zj, zk, zp, zq, zr, zs, zv, zw, zx
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
    [3, 6, 13, 8, 0, 0],   // "ing" reversed
    [2, 3, 4, 0, 0, 0],    // "ed" reversed
    [2, 24, 11, 0, 0, 0],  // "ly" reversed
    [4, 18, 18, 4, 13, 0], // "ness" reversed
    [4, 13, 14, 8, 19, 0], // "tion" reversed
    [4, 19, 13, 4, 12, 0], // "ment" reversed
    [4, 4, 11, 1, 0, 0],   // "able" reversed
    [2, 17, 4, 0, 0, 0],   // "er" reversed
    [3, 19, 18, 4, 0, 0],  // "est" reversed
    [3, 11, 20, 5, 0, 0],  // "ful" reversed
];

/// Common English prefixes (as letter indices)
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
    if indices.len() >= 2
        && !is_vowel_idx(indices[last_idx])
        && !is_vowel_idx(indices[last_idx - 1])
        && is_valid_coda(indices[last_idx - 1], indices[last_idx])
    {
        score += 1;
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
// Invalid Vietnamese Detection (for auto-restore)
// =============================================================================

/// Check if word has patterns that are impossible in Vietnamese
/// This is more reliable than detecting English - we detect non-Vietnamese instead
#[inline]
pub fn has_invalid_vietnamese_pattern(keys: &[u16]) -> bool {
    if keys.is_empty() {
        return false;
    }

    let first = key_to_index(keys[0]);

    // Z-initial words are always English (z is not a Vietnamese letter)
    if first == 25 {
        // z=25
        return true;
    }

    // F at word start is English (f is not a Vietnamese initial consonant)
    // But 'f' after a vowel is a Telex modifier (huyền tone), so don't flag that
    if first == 5 {
        // f=5 at start
        return true;
    }
    // Check for 'f' NOT after a vowel or vowel modifier (which would be invalid)
    // In Telex, 'w' is a vowel modifier (ow→ơ, aw→ă, uw→ư), so 'f' after 'w' is valid
    for i in 1..keys.len() {
        if key_to_index(keys[i]) == 5 {
            // f=5
            let prev = key_to_index(keys[i - 1]);
            // Allow 'f' after vowel or after 'w' (vowel modifier)
            let is_after_vowel_like = is_vowel_idx(prev) || prev == 22; // w=22
            if !is_after_vowel_like {
                return true;
            }
        }
    }

    // J-initial words are always English (j is not a Vietnamese letter)
    if first == 9 {
        // j=9
        return true;
    }

    // Check for English-only onset clusters at word start
    // These are consonant clusters that don't exist in Vietnamese
    if keys.len() >= 2 && !is_vowel_idx(first) {
        let second = key_to_index(keys[1]);
        if !is_vowel_idx(second) {
            // Two consonants at start - check if it's English-only cluster
            // Vietnamese valid: ch, gh, gi, kh, ng, nh, ph, qu, th, tr, ngh
            // English-only: bl, br, cl, cr, dr, dw, fl, fr, gl, gn, gr, kn, kw, pl, pr,
            //               sc, sh, sk, sl, sm, sn, sp, sq, st, sw, tw, wh, wr
            let is_vn_onset = matches!(
                (first, second),
                (2, 7)   // ch
                | (6, 7)   // gh
                | (6, 8)   // gi
                | (10, 7)  // kh
                | (13, 6)  // ng
                | (13, 7)  // nh
                | (15, 7)  // ph
                | (16, 20) // qu
                | (19, 7)  // th
                | (19, 17) // tr
            );
            // If it's a valid English onset but NOT a valid Vietnamese onset, it's English
            if !is_vn_onset && is_valid_onset(first, second) {
                return true;
            }
        }
    }

    // P-initial without H (ph-) is rare in Vietnamese
    // BUT "p + vowel + modifier" can produce valid Vietnamese (post → pót)
    // Only flag as English if word is longer and has English-specific patterns
    if first == 15 && keys.len() >= 4 {
        // p=15
        let second = key_to_index(keys[1]);
        // If second letter is vowel (not 'h'), check for English suffixes
        if second != 7 && is_vowel_idx(second) {
            // Check for English-specific endings that can't be Vietnamese
            let len = keys.len();
            if len >= 3 {
                let last_two: [u8; 2] = [key_to_index(keys[len - 2]), key_to_index(keys[len - 1])];
                // "_ck", "_ng" at end are English for p-initial
                if last_two == [2, 10] || (last_two == [13, 6] && len > 4) {
                    return true;
                }
            }
        }
    }

    // W-initial words: check if it forms valid Vietnamese pattern
    // W → ư, so "wa" → "ưa" is valid, "water" → "ưater" is not
    if first == 22 {
        // w=22
        return is_w_word_english(keys);
    }

    // Check for 'w' in non-initial position
    for (pos, &key) in keys[1..].iter().enumerate() {
        let idx = pos + 1; // actual position in keys
        if key_to_index(key) == 22 {
            // w=22 in non-initial position
            // Check if it's at the END of word (trailing w)
            if idx == keys.len() - 1 {
                // Trailing w: "bow" → "bơ", "saw" → "să"
                return is_trailing_w_english(keys);
            } else {
                // W in middle position (not at start, not at end)
                // Check if preceded by valid English onset cluster (br, cr, dr, etc.)
                if idx >= 1 {
                    let prev = key_to_index(keys[idx - 1]);
                    // Common English consonant + w patterns in middle: "ow" preceded by consonant cluster
                    // Examples: "brown" (br+ow+n), "crown" (cr+ow+n), "grown" (gr+ow+n)
                    if !is_vowel_idx(prev) && idx >= 2 {
                        let prev2 = key_to_index(keys[idx - 2]);
                        // Check if prev2+prev forms English onset cluster (br, cr, dr, fr, gr, etc.)
                        if is_valid_onset(prev2, prev) {
                            return true; // English word with onset cluster + w
                        }
                    }
                    // Also check for s+w pattern (sweet, swim, swing, etc.)
                    if prev == 18 && idx == 1 {
                        // s=18, w at position 1
                        return true; // sw- is English pattern
                    }
                }
            }
        }
    }

    // Check for 'z' anywhere (z is not a Vietnamese letter)
    for &key in &keys[1..] {
        if key_to_index(key) == 25 {
            // z=25
            return true;
        }
    }

    // Pattern 4: Vowel-initial + modifier + vowel (no consonant initial)
    // Example: "use" = u + s(modifier) + e → "úe" is invalid Vietnamese
    // But "air" = a + i + r(modifier) → "ải" is valid (ai is valid diphthong)
    if is_vowel_idx(first) && keys.len() >= 3 {
        // Check for modifier between vowels
        let modifiers = [18u8, 5, 17, 23, 9]; // s, f, r, x, j (tone modifiers)
        for i in 1..keys.len() - 1 {
            let mid = key_to_index(keys[i]);
            if modifiers.contains(&mid) {
                // Check if next letter is vowel
                let next = key_to_index(keys[i + 1]);
                if is_vowel_idx(next) {
                    // Check if preceding vowels form valid Vietnamese diphthong
                    let prev_vowels: Vec<u8> = keys[..i]
                        .iter()
                        .map(|&k| key_to_index(k))
                        .filter(|&idx| is_vowel_idx(idx))
                        .collect();

                    // Single vowel + modifier + different vowel is usually English
                    // Exception: valid diphthongs like "ai", "oi", "ua", etc.
                    if prev_vowels.len() == 1 {
                        let v1 = prev_vowels[0];
                        let v2 = next;
                        // Check if v1+v2 forms valid Vietnamese diphthong
                        if !is_valid_vietnamese_diphthong(v1, v2) {
                            return true;
                        }
                    }
                }
            }
        }
    }

    // Check for double vowels that indicate English (ee, oo patterns)
    // Vietnamese uses circumflex for these (ê, ô), but in Telex double typing reverts
    // For words like "keep", "looks", "saas", the double vowel is English
    // EXCEPTION: Vietnamese Telex patterns like "soos" → "số" (double vowel + tone key)
    for i in 0..keys.len().saturating_sub(1) {
        let c1 = key_to_index(keys[i]);
        let c2 = key_to_index(keys[i + 1]);

        // Double vowels (not caused by circumflex typing)
        // ee, oo, aa patterns in the MIDDLE or at the END of a word
        if c1 == c2 && is_vowel_idx(c1) {
            // Check if there's a consonant before or after (not just isolated double vowel)
            let has_context = (i > 0 && !is_vowel_idx(key_to_index(keys[i - 1])))
                || (i + 2 < keys.len() && !is_vowel_idx(key_to_index(keys[i + 2])));

            if has_context {
                // Check if immediately followed by tone key (Vietnamese Telex pattern)
                // Tone keys: s(18), f(5), r(17), x(23), j(9)
                // Examples: "soos" → "số", "toof" → "tồ"
                let followed_by_tone_key = if i + 2 < keys.len() {
                    let next = key_to_index(keys[i + 2]);
                    matches!(next, 18 | 5 | 17 | 23 | 9)
                } else {
                    false
                };

                // Also check for tone key at end of word (double vowel at end-1)
                let at_end_with_tone = if i + 2 == keys.len() - 1 {
                    let last = key_to_index(keys[keys.len() - 1]);
                    matches!(last, 18 | 5 | 17 | 23 | 9)
                } else {
                    false
                };

                if !followed_by_tone_key && !at_end_with_tone {
                    // Double vowel with consonant context but no tone key: English
                    return true;
                }
            }
        }

        // "ou" sequence (common in English: around, sound, ground, out, our, etc.)
        // NOT a valid Vietnamese diphthong - Vietnamese has ô, ơ but not raw "ou"
        if c1 == 14 && c2 == 20 {
            // o=14, u=20
            return true;
        }

        // "ei" sequence (common in English: receive, ceiling, weird, etc.)
        // NOT a valid Vietnamese diphthong - Vietnamese has ê but not raw "ei"
        if c1 == 4 && c2 == 8 {
            // e=4, i=8
            return true;
        }
    }

    // Check for double consonants that are invalid in Vietnamese
    // e.g., "ll", "ss", "tt", "ff", "rr", "pp", "ck", "ght", etc.
    for i in 0..keys.len().saturating_sub(1) {
        let c1 = key_to_index(keys[i]);
        let c2 = key_to_index(keys[i + 1]);

        // Skip if either is a vowel
        if is_vowel_idx(c1) || is_vowel_idx(c2) {
            continue;
        }

        // Double consonants (ll, tt, pp, etc.)
        // These are common in English but invalid in Vietnamese
        // Exceptions for Telex patterns (only at word END for revert):
        // - s, f, r, x, j can double to revert tone marks
        // - 'd' can double for stroke (dd → đ) - this is valid anywhere
        if c1 == c2 {
            let is_telex_modifier = matches!(c1, 18 | 5 | 17 | 23 | 9); // s, f, r, x, j
            let is_dd_stroke = c1 == 3; // d=3 (dd → đ)
            let at_end = i + 2 == keys.len(); // double is at the end of word

            // Skip dd (stroke) anywhere, but only skip tone modifier doubles at END
            // "bass" → 'ss' at end → skip (revert pattern)
            // "issue" → 'ss' in middle → flag as English
            if is_dd_stroke || (is_telex_modifier && at_end) {
                // Valid Telex pattern, skip
            } else {
                // Double consonant detected - English pattern
                return true;
            }
        }

        // "ck" pattern (common in English, invalid in Vietnamese)
        if c1 == 2 && c2 == 10 {
            // c=2, k=10
            return true;
        }

        // "gh" at end (Vietnamese "gh" only at start)
        if i > 0 && c1 == 6 && c2 == 7 {
            // g=6, h=7
            return true;
        }

        // "ght" pattern
        if c1 == 6 && c2 == 7 && i + 2 < keys.len() {
            let c3 = key_to_index(keys[i + 2]);
            if c3 == 19 {
                // t=19
                return true;
            }
        }
    }

    // Check for common English suffixes that produce invalid Vietnamese
    // "_ure" pattern (pure, sure, cure, etc.) - "ure" is not valid Vietnamese
    // "_ire" pattern (hire, wire, fire, etc.) - "ire" is not valid Vietnamese
    // "_ore" pattern (more, store, etc.) - "ore" is not valid Vietnamese
    // "_are" pattern (care, share, etc.) - "are" is not valid Vietnamese
    let len = keys.len();
    if len >= 3 {
        let last_three: [u8; 3] = [
            key_to_index(keys[len - 3]),
            key_to_index(keys[len - 2]),
            key_to_index(keys[len - 1]),
        ];

        // "_ure" pattern: u(20) + r(17) + e(4)
        if last_three == [20, 17, 4] {
            return true;
        }

        // "_ire" pattern: i(8) + r(17) + e(4)
        if last_three == [8, 17, 4] {
            return true;
        }

        // "_ore" pattern: o(14) + r(17) + e(4)
        if last_three == [14, 17, 4] {
            return true;
        }

        // "_are" pattern: a(0) + r(17) + e(4)
        if last_three == [0, 17, 4] {
            return true;
        }

        // "_isk" pattern: i(8) + s(18) + k(10) - "risk", "disk"
        if last_three == [8, 18, 10] {
            return true;
        }

        // "_ask" pattern: a(0) + s(18) + k(10) - "task", "mask"
        if last_three == [0, 18, 10] {
            return true;
        }

        // "_esk" pattern: e(4) + s(18) + k(10) - "desk"
        if last_three == [4, 18, 10] {
            return true;
        }

        // "_usk" pattern: u(20) + s(18) + k(10) - "dusk", "tusk", "husk"
        if last_three == [20, 18, 10] {
            return true;
        }

        // "_est" pattern: e(4) + s(18) + t(19) - "test", "best", "request"
        // Only for longer words (5+ chars) to avoid false positives with Vietnamese
        if len >= 5 && last_three == [4, 18, 19] {
            return true;
        }

        // "_ata" pattern: a(0) + t(19) + a(0) - "data", "metadata"
        // Same vowel separated by complete final 't' - English pattern
        // This would trigger circumflex creating "dât" which is unusual
        if last_three == [0, 19, 0] {
            return true;
        }
    }

    // Check for same vowel separated by complete final consonant
    // Pattern: vowel + complete_final + same_vowel (without tone key after)
    // Examples: "data" (a-t-a), "mama" (a-m-a), "papa" (a-p-a)
    // This would trigger circumflex but creates unusual Vietnamese
    // Exception: extendable finals (n, c) - "ieneg" → "iêng" is valid
    // Exception: 3+ same vowels indicate Telex revert (aaa → circumflex reverted)
    if len >= 3 {
        for i in 0..len - 2 {
            let c1 = key_to_index(keys[i]);
            let c2 = key_to_index(keys[i + 1]);
            let c3 = key_to_index(keys[i + 2]);

            // Check: vowel + complete_final + same_vowel
            if is_vowel_idx(c1) && !is_vowel_idx(c2) && c1 == c3 {
                // Count how many times this vowel appears in the word
                // If 3+ occurrences, a Telex revert happened (aaa → aa + a)
                let vowel_count = keys.iter().filter(|&&k| key_to_index(k) == c1).count();
                if vowel_count >= 3 {
                    continue; // Skip - revert happened, not English pattern
                }

                // Complete finals: t(19), m(12), p(15)
                // These cannot extend, so VCV pattern is English-like
                // Extendable finals: n(13) can become ng/nh, c(2) can become ch
                let is_complete_final = matches!(c2, 19 | 12 | 15); // t, m, p
                if is_complete_final {
                    // Check if there's a tone key after (s, f, r, x, j)
                    // If there's a tone key, user intends Vietnamese
                    let has_tone_key = if i + 3 < len {
                        let next = key_to_index(keys[i + 3]);
                        matches!(next, 18 | 5 | 17 | 23 | 9) // s, f, r, x, j
                    } else {
                        false
                    };
                    if !has_tone_key {
                        return true;
                    }
                }
            }
        }
    }

    false
}

/// Check if two vowels form a valid Vietnamese diphthong
#[inline]
fn is_valid_vietnamese_diphthong(v1: u8, v2: u8) -> bool {
    // Valid Vietnamese diphthongs (as pairs of letter indices)
    // a=0, e=4, i=8, o=14, u=20, y=24
    matches!(
        (v1, v2),
        // ai, ao, au, ay
        (0, 8) | (0, 14) | (0, 20) | (0, 24) |
        // eo, êu (e+u patterns)
        (4, 14) | (4, 20) |
        // ia, iê, iu
        (8, 0) | (8, 4) | (8, 20) |
        // oa, oe, oi
        (14, 0) | (14, 4) | (14, 8) |
        // ua, ưa, ui, ưi, uo, uô, ươ, ưu
        (20, 0) | (20, 8) | (20, 14) | (20, 20) |
        // ye, yê
        (24, 4)
    )
}

/// Check if W-initial word is English (not valid Vietnamese)
/// W → ư in Vietnamese, so valid patterns are limited:
/// - wa → ưa (valid)
/// - wo → ươ (valid)
/// - wu → ưu (valid)
/// - wng → ưng (valid: ư + ng final)
/// - water → ưater (INVALID - "ater" is not valid Vietnamese)
#[inline]
fn is_w_word_english(keys: &[u16]) -> bool {
    if keys.len() <= 1 {
        return false;
    }

    // Check for English onset clusters (wr-, wh-, wy-) which are common in English
    // These are always English, not Vietnamese
    let second = key_to_index(keys[1]);
    if second == 17 {
        // wr- cluster: wrong, write, wrap, wrist
        return true;
    }
    if second == 7 {
        // wh- cluster: what, when, where, which, while, white
        return true;
    }

    // First check for English suffix patterns that would be invalid Vietnamese
    // These patterns take priority over other W-initial checks
    let len = keys.len();
    if len >= 3 {
        let last_three: [u8; 3] = [
            key_to_index(keys[len - 3]),
            key_to_index(keys[len - 2]),
            key_to_index(keys[len - 1]),
        ];

        // "_ore" pattern: o(14) + r(17) + e(4) - "wore", "more", etc.
        if last_three == [14, 17, 4] {
            return true;
        }

        // "_ire" pattern: i(8) + r(17) + e(4) - "wire", "fire", etc.
        if last_three == [8, 17, 4] {
            return true;
        }

        // "_are" pattern: a(0) + r(17) + e(4) - "ware", "care", etc.
        if last_three == [0, 17, 4] {
            return true;
        }

        // "_ure" pattern: u(20) + r(17) + e(4) - "pure", etc.
        if last_three == [20, 17, 4] {
            return true;
        }

        // "_ide" pattern: i(8) + d(3) + e(4) - "wide", "side", etc.
        if last_three == [8, 3, 4] {
            return true;
        }

        // "_ise" pattern: i(8) + s(18) + e(4) - "wise", etc.
        if last_three == [8, 18, 4] {
            return true;
        }
    }

    // W + single vowel = valid Vietnamese (except we, wi, wy)
    if keys.len() == 2 {
        // Valid: wa(ưa), wo(ươ), wu(ưu)
        // Invalid: we(ưe), wi(ưi), wy(ưy)
        return matches!(second, 4 | 8 | 24); // e=4, i=8, y=24 are invalid
    }

    // PRIORITY CHECK: W + final consonant(s) pattern (no explicit vowel)
    // "wng" → "ưng" is valid (ư is implicit vowel)
    // This MUST come before the generic "w + consonant" check
    // Valid finals: ng, nh, n, m, c, t, p, ch
    let rest = &keys[1..];
    if rest.len() <= 2 && rest.iter().all(|&k| !is_vowel_idx(key_to_index(k))) {
        // All remaining are consonants - check if valid final
        if rest.len() == 2 {
            let f1 = key_to_index(rest[0]);
            let f2 = key_to_index(rest[1]);
            // Valid final clusters: ng, nh, ch
            if (f1 == 13 && matches!(f2, 6 | 7)) || (f1 == 2 && f2 == 7) {
                return false; // Valid Vietnamese (wng → ưng, wnh → ưnh, wch → ưch)
            }
        } else if rest.len() == 1 {
            let f = key_to_index(rest[0]);
            // Valid single finals: c, m, n, p, t
            if matches!(f, 2 | 12 | 13 | 15 | 19) {
                return false; // Valid Vietnamese (wn → ưn, wm → ưm, etc.)
            }
        }
        // Invalid single consonant or cluster (e.g., "wb", "wk")
        return true;
    }

    // W + vowel that forms invalid diphthong with ư
    // "wi", "we", "wy" are invalid Vietnamese (ưi, ưe, ưy not valid diphthongs)
    // So "win", "web", "why" should be English
    if is_vowel_idx(second) && matches!(second, 4 | 8 | 24) {
        // e=4, i=8, y=24 form invalid diphthongs with ư
        return true;
    }

    // W + vowel + consonant + vowel pattern is usually English
    // Example: "wide" = w + i + d + e → consonant between vowels
    if rest.len() >= 3 {
        let vowel_positions: Vec<usize> = rest
            .iter()
            .enumerate()
            .filter(|(_, &k)| is_vowel_idx(key_to_index(k)))
            .map(|(i, _)| i)
            .collect();

        if vowel_positions.len() >= 2 {
            // Check for consonant between first two vowels
            let first_vowel = vowel_positions[0];
            let second_vowel = vowel_positions[1];
            if second_vowel > first_vowel + 1 {
                // There's a consonant gap between vowels
                // Check if this forms valid Vietnamese structure
                // In Vietnamese, vowel clusters are adjacent or with specific patterns
                // "wide" = i + d + e → consonant 'd' separates vowels → English
                return true;
            }
        }
    }

    // W + vowel + optional finals
    if keys.len() <= 5 {
        // Check if pattern like: w + vowel(s) + valid_final
        let has_vowel = rest.iter().any(|&k| is_vowel_idx(key_to_index(k)));
        if has_vowel {
            // Find last vowel position
            let last_vowel_pos = rest
                .iter()
                .rposition(|&k| is_vowel_idx(key_to_index(k)))
                .unwrap_or(0);

            // Check finals after last vowel
            let finals = &rest[last_vowel_pos + 1..];

            if finals.is_empty() {
                // W + vowel(s) only - valid Vietnamese
                // Check vowel pattern validity
                let vowels: Vec<u8> = rest
                    .iter()
                    .filter(|&&k| is_vowel_idx(key_to_index(k)))
                    .map(|&k| key_to_index(k))
                    .collect();

                // Single vowel or valid diphthong
                if vowels.len() <= 2 {
                    // Check for invalid patterns
                    if vowels.len() == 1 {
                        // w + single vowel: valid except we, wi, wy
                        return matches!(vowels[0], 4 | 8 | 24);
                    }
                    // w + two vowels: check common patterns
                    // wa/wo/wu + vowel could be valid
                    return false;
                }
            }

            // Check valid Vietnamese finals
            if finals.len() == 1 {
                let f = key_to_index(finals[0]);
                // Valid single finals: c, m, n, p, t
                if matches!(f, 2 | 12 | 13 | 15 | 19) {
                    return false; // Valid Vietnamese
                }
            } else if finals.len() == 2 {
                let f1 = key_to_index(finals[0]);
                let f2 = key_to_index(finals[1]);
                // Valid final clusters: ng, nh, ch
                if (f1 == 13 && matches!(f2, 6 | 7)) || (f1 == 2 && f2 == 7) {
                    return false; // Valid Vietnamese
                }
            }
        }
    }

    // Longer patterns or patterns with invalid finals are English
    true
}

/// Check if word with trailing W is English
/// In Vietnamese, W after vowels can create horn (ơ, ư)
/// But English words like "saw", "draw" should be restored
///
/// Pattern rules:
/// - `_ow` with valid Vietnamese initial → keep Vietnamese (bow → bơ)
/// - `_ow` with invalid initial (brow, crow) → restore English
/// - `_aw` → restore English (saw, law, draw)
/// - `_ew` → restore English (new, few, drew)
#[inline]
fn is_trailing_w_english(keys: &[u16]) -> bool {
    let len = keys.len();
    if len < 2 {
        return false;
    }

    // Check if ends with 'w'
    let last_idx = key_to_index(keys[len - 1]);
    if last_idx != 22 {
        // w=22
        return false;
    }

    // Check vowel before 'w'
    if len < 2 {
        return false;
    }
    let prev = key_to_index(keys[len - 2]);

    // Pattern: "ow" - can be valid Vietnamese if initial is valid single consonant
    if prev == 14 {
        // o=14
        // Check if initial consonant(s) are valid Vietnamese
        if len == 3 {
            // Single consonant + ow (e.g., "bow", "cow")
            let initial = key_to_index(keys[0]);
            // Valid single Vietnamese initials that make sense with ơ
            // b, c, d, g, h, k, l, m, n, p, r, s, t, v, x
            if !is_vowel_idx(initial) {
                // Single consonant initial with ow - valid Vietnamese (bơ, cơ, etc.)
                return false;
            }
        }
        // Multi-consonant initial + ow (e.g., "brow", "crow") - English
        // or vowel initial - English
        return true;
    }

    // Pattern: "aw" - always English (saw, law, draw)
    if prev == 0 {
        // a=0
        return true;
    }

    // Pattern: "ew" - always English (new, few, drew)
    if prev == 4 {
        // e=4
        return true;
    }

    // Pattern: "iw" - English (rare but possible)
    if prev == 8 {
        // i=8
        return true;
    }

    // Pattern: "uw" - could be Vietnamese (like in "huw" for horn)
    // But at word end it's likely English
    if prev == 20 {
        // u=20
        return true;
    }

    false
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
        let num_words = size_bits.div_ceil(64);
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
        assert!(is_vowel_idx(0)); // a
        assert!(is_vowel_idx(4)); // e
        assert!(is_vowel_idx(8)); // i
        assert!(is_vowel_idx(14)); // o
        assert!(is_vowel_idx(20)); // u
        assert!(is_vowel_idx(24)); // y

        assert!(!is_vowel_idx(1)); // b
        assert!(!is_vowel_idx(2)); // c
    }

    #[test]
    fn test_valid_onset() {
        // Valid onsets
        assert!(is_valid_onset(1, 11)); // bl
        assert!(is_valid_onset(1, 17)); // br
        assert!(is_valid_onset(18, 19)); // st
        assert!(is_valid_onset(19, 17)); // tr
        assert!(is_valid_onset(16, 20)); // qu

        // Invalid onsets
        assert!(!is_valid_onset(1, 2)); // bc
        assert!(!is_valid_onset(3, 6)); // dg
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
        assert!(is_valid_coda(13, 6)); // ng
        assert!(is_valid_coda(13, 3)); // nd
        assert!(is_valid_coda(18, 19)); // st

        // Invalid codas
        assert!(!is_valid_coda(1, 3)); // bd
    }

    #[test]
    fn test_impossible_bigram() {
        // q only followed by u
        assert!(is_impossible_bigram(16, 0)); // qa impossible
        assert!(!is_impossible_bigram(16, 20)); // qu valid

        // j only with vowels
        assert!(is_impossible_bigram(9, 1)); // jb impossible
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
