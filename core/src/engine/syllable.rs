//! Vietnamese Syllable Parser
//!
//! Parses buffer into syllable structure: (C₁)(G)V(C₂)
//! - C₁: Initial consonant (phụ âm đầu)
//! - G: Glide/Medial (âm đệm)
//! - V: Vowel nucleus (nguyên âm chính) - REQUIRED
//! - C₂: Final consonant (âm cuối)

use crate::data::constants;
use crate::data::keys;

/// Parsed syllable structure
#[derive(Debug, Clone, Default)]
pub struct Syllable {
    /// Initial consonant indices in buffer
    pub initial: Vec<usize>,
    /// Glide/medial index (o in "hoa", u in "qua")
    pub glide: Option<usize>,
    /// Vowel nucleus indices
    pub vowel: Vec<usize>,
    /// Final consonant indices
    pub final_c: Vec<usize>,
}

impl Syllable {
    pub fn is_empty(&self) -> bool {
        self.vowel.is_empty()
    }

    pub fn has_initial(&self) -> bool {
        !self.initial.is_empty()
    }

    pub fn has_final(&self) -> bool {
        !self.final_c.is_empty()
    }

    pub fn vowel_count(&self) -> usize {
        self.vowel.len()
    }
}

/// Valid final consonants (âm cuối)
const FINALS_2: &[[u16; 2]] = &[
    [keys::C, keys::H], // ch
    [keys::N, keys::G], // ng
    [keys::N, keys::H], // nh
];

/// Parse buffer keys into syllable structure
///
/// Uses longest-match-first algorithm:
/// 1. Match initial consonant (3 → 2 → 1 chars)
/// 2. Check for glide (o/u before main vowel)
/// 3. Match vowel nucleus
/// 4. Remainder is final consonant
///
/// Note: This parser is lenient - it will parse invalid initials
/// and let validation reject them later.
pub fn parse(buffer_keys: &[u16]) -> Syllable {
    let mut syllable = Syllable::default();
    let len = buffer_keys.len();

    if len == 0 {
        return syllable;
    }

    // Step 1: Find first vowel position, with special handling for "gi", "qu"
    let first_vowel_pos = buffer_keys.iter().position(|&k| keys::is_vowel(k));

    let vowel_start = match first_vowel_pos {
        Some(pos) => {
            // Special case: "gi" + vowel → gi is initial, not g alone
            // Check if we have g + i + another_vowel pattern
            if pos > 0 && pos + 1 < len {
                let prev = buffer_keys[pos - 1];
                let curr = buffer_keys[pos];
                let next = buffer_keys[pos + 1];

                // gi + vowel (giàu, giếng, etc.)
                if prev == keys::G && curr == keys::I && keys::is_vowel(next) {
                    // Include 'i' in initial, not as vowel
                    for i in 0..=pos {
                        syllable.initial.push(i);
                    }
                    pos + 1 // vowels start after 'i'
                }
                // qu + vowel (qua, quê, etc.) - qu is initial
                else if prev == keys::Q && curr == keys::U && keys::is_vowel(next) {
                    // Include 'u' in initial, not as vowel
                    for i in 0..=pos {
                        syllable.initial.push(i);
                    }
                    pos + 1
                } else {
                    // Normal: everything before first vowel is initial
                    for i in 0..pos {
                        syllable.initial.push(i);
                    }
                    pos
                }
            } else {
                // Normal: everything before first vowel is initial
                for i in 0..pos {
                    syllable.initial.push(i);
                }
                pos
            }
        }
        None => {
            // No vowel found - invalid syllable
            return syllable;
        }
    };

    // Step 2: Find vowels and glide
    let mut vowel_end = vowel_start;

    // Find all consecutive vowels
    while vowel_end < len && keys::is_vowel(buffer_keys[vowel_end]) {
        vowel_end += 1;
    }

    if vowel_end == vowel_start {
        // No vowel found - invalid syllable (shouldn't happen here)
        return syllable;
    }

    // Check for glide pattern
    let vowel_count = vowel_end - vowel_start;
    if vowel_count >= 2 {
        let first_vowel = buffer_keys[vowel_start];
        let second_vowel = buffer_keys[vowel_start + 1];

        // Check if it's a glide pattern
        let is_glide = is_glide_pattern(first_vowel, second_vowel, &syllable);

        if is_glide {
            syllable.glide = Some(vowel_start);
            for i in (vowel_start + 1)..vowel_end {
                syllable.vowel.push(i);
            }
        } else {
            for i in vowel_start..vowel_end {
                syllable.vowel.push(i);
            }
        }
    } else {
        // Single vowel
        syllable.vowel.push(vowel_start);
    }

    // Step 3: Match final consonant
    if vowel_end < len {
        match_final(buffer_keys, vowel_end, &mut syllable);
    }

    syllable
}

/// Match final consonant
fn match_final(keys: &[u16], start: usize, syllable: &mut Syllable) {
    let len = keys.len();
    let remaining = len - start;

    // Try 2-char finals
    if remaining >= 2 {
        for pattern in FINALS_2 {
            if keys[start] == pattern[0] && keys[start + 1] == pattern[1] {
                syllable.final_c = vec![start, start + 1];
                return;
            }
        }
    }

    // Try 1-char finals
    if remaining >= 1 && constants::VALID_FINALS_1.contains(&keys[start]) {
        syllable.final_c = vec![start];
    }
}

/// Check if first vowel is a glide (âm đệm)
///
/// Glide patterns:
/// - o + (a, ă, e) → oa, oă, oe
/// - u + (a, â, ê, y) after "qu" → qua, quâ, quê, quy
fn is_glide_pattern(first: u16, second: u16, syllable: &Syllable) -> bool {
    // Check if initial is "qu" - then u is part of initial, not glide
    let is_qu = syllable.initial.len() == 2;
    if is_qu {
        // qu already includes u, no separate glide
        return false;
    }

    match first {
        keys::O => {
            // o + (a, e) → glide
            matches!(second, keys::A | keys::E)
        }
        keys::U => {
            // u + (a, â, ê, y) when NOT after qu
            // Actually u is glide in: uy, ua (non-qu context handled differently)
            matches!(second, keys::Y | keys::E)
        }
        _ => false,
    }
}

/// Check if buffer represents a potentially valid Vietnamese syllable structure
///
/// This is a quick structural check, not full phonological validation
pub fn is_valid_structure(buffer_keys: &[u16]) -> bool {
    if buffer_keys.is_empty() {
        return false;
    }

    let syllable = parse(buffer_keys);

    // Must have at least one vowel
    if syllable.is_empty() {
        return false;
    }

    // Basic structure check passed
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::keys_from_str;

    #[test]
    fn parse_simple_syllable() {
        let s = parse(&keys_from_str("ba"));
        assert_eq!(s.initial.len(), 1);
        assert_eq!(s.vowel.len(), 1);
        assert!(s.final_c.is_empty());
    }

    #[test]
    fn parse_ngh_initial() {
        let s = parse(&keys_from_str("nghieng"));
        assert_eq!(s.initial.len(), 3);
        assert_eq!(s.vowel.len(), 2);
        assert_eq!(s.final_c.len(), 2);
    }

    #[test]
    fn parse_qu_initial() {
        let s = parse(&keys_from_str("qua"));
        assert_eq!(s.initial.len(), 2);
        assert_eq!(s.vowel.len(), 1);
        assert!(s.glide.is_none());
    }

    #[test]
    fn parse_hoa_with_glide() {
        let s = parse(&keys_from_str("hoa"));
        assert_eq!(s.initial.len(), 1);
        assert!(s.glide.is_some());
        assert_eq!(s.vowel.len(), 1);
    }

    #[test]
    fn parse_gi_initial() {
        let s = parse(&keys_from_str("giau"));
        assert_eq!(s.initial.len(), 2);
        assert_eq!(s.vowel.len(), 2);
    }

    #[test]
    fn parse_duoc() {
        let s = parse(&keys_from_str("duoc"));
        assert_eq!(s.initial.len(), 1);
        assert_eq!(s.vowel.len(), 2);
        assert_eq!(s.final_c.len(), 1);
    }

    #[test]
    fn parse_vowel_only() {
        let s = parse(&keys_from_str("a"));
        assert!(s.initial.is_empty());
        assert_eq!(s.vowel.len(), 1);
    }

    #[test]
    fn invalid_no_vowel() {
        let s = parse(&keys_from_str("bcd"));
        assert!(s.is_empty());
    }

    #[test]
    fn test_is_valid_structure() {
        assert!(is_valid_structure(&keys_from_str("ba")));
        assert!(is_valid_structure(&keys_from_str("nghieng")));
        assert!(is_valid_structure(&keys_from_str("a")));
        assert!(!is_valid_structure(&keys_from_str("bcd")));
        assert!(!is_valid_structure(&keys_from_str("")));
    }
}
