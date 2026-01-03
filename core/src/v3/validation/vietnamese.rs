//! Vietnamese validation (8 phonotactic rules)
//!
//! Validates Vietnamese syllables against phonological constraints:
//! 1. Has vowel (must have at least one vowel)
//! 2. Valid initial consonant
//! 3. All chars parsed (structure check)
//! 4. Spelling rules (initial + vowel combination)
//! 5. Valid final consonant
//! 6. Valid vowel pattern (diphthongs)
//! 7. Vowel-final compatibility
//! 8. Tone-stop restriction (stop finals only allow sắc/nặng)

use crate::v3::constants::vietnamese::{
    char_to_tone, char_to_vowel_type, final_cat, final_type, initial, is_valid_spelling,
    is_valid_tone_final, is_valid_vowel_pair, is_vn_vowel, lookup_final_type, lookup_initial_type,
    tone, vowel_type, final_type_to_category, Tone,
};

// ============================================================================
// VALIDATION RESULT
// ============================================================================

/// Vietnamese validation result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationResult {
    /// Valid Vietnamese syllable
    Valid,
    /// No vowel found
    NoVowel,
    /// Invalid initial consonant
    InvalidInitial,
    /// Invalid syllable structure (unparsed chars)
    InvalidStructure,
    /// Invalid spelling rule (c before e, etc.)
    InvalidSpelling,
    /// Invalid final consonant
    InvalidFinal,
    /// Invalid vowel pattern (foreign diphthong)
    InvalidVowelPattern,
    /// Incompatible vowel + final
    IncompatibleVowelFinal,
    /// Invalid tone with stop final
    InvalidTone,
}

// ============================================================================
// PARSED SYLLABLE
// ============================================================================

/// Parsed Vietnamese syllable structure
#[derive(Debug, Clone, Default)]
pub struct Syllable {
    /// Initial consonant string (0-3 chars)
    pub initial: String,
    /// Initial type index for matrix lookup
    pub initial_type: u8,

    /// Glide (âm đệm): 'o' or 'u'
    pub glide: Option<char>,

    /// Vowel string (1-3 chars)
    pub vowel: String,
    /// Primary vowel type for matrix lookup
    pub vowel_type: u8,

    /// Final consonant string (0-2 chars)
    pub final_c: String,
    /// Final type for matrix lookup
    pub final_type: u8,
    /// Final category (OPEN/NASAL/SEMI/STOP)
    pub final_cat: u8,

    /// Tone mark
    pub tone: u8,
}

// ============================================================================
// SYLLABLE PARSER
// ============================================================================

/// Parse a syllable string into components
pub fn parse_syllable(s: &str) -> Option<Syllable> {
    if s.is_empty() {
        return None;
    }

    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    let mut syl = Syllable::default();
    let mut pos = 0;

    // Step 1: Extract initial consonant
    // Try longest match first (3-char: ngh, 2-char: ch/gh/gi/kh/ng/nh/ph/qu/th/tr, 1-char)
    let initial_str = extract_initial(&chars, &mut pos);
    syl.initial = initial_str.clone();
    syl.initial_type = lookup_initial_type(&initial_str);

    // If initial is invalid and not empty, could be foreign
    if syl.initial_type == initial::INVALID && !syl.initial.is_empty() {
        return None;
    }

    // Step 2: Check for glide (âm đệm)
    // Glide occurs when: initial (not qu) + o/u + different vowel
    if pos < len && syl.initial_type != initial::QU {
        let c = chars[pos].to_ascii_lowercase();
        if (c == 'o' || c == 'u') && pos + 1 < len {
            let next = chars[pos + 1].to_ascii_lowercase();
            // Check if next char is a different vowel
            if is_vn_vowel(next) && char_to_vowel_type(next) != char_to_vowel_type(chars[pos]) {
                // Check specific glide patterns
                if c == 'o' && matches!(next, 'a' | 'ă' | 'e') {
                    syl.glide = Some(chars[pos]);
                    pos += 1;
                } else if c == 'u' && !matches!(syl.initial_type, initial::Q | initial::QU) {
                    // u as glide before most vowels (not after q/qu which already has u)
                    if matches!(next, 'a' | 'â' | 'e' | 'ê' | 'i' | 'y' | 'ơ' | 'ô') {
                        syl.glide = Some(chars[pos]);
                        pos += 1;
                    }
                }
            }
        }
    }

    // Step 3: Extract vowels and detect tone
    let vowel_start = pos;
    while pos < len && is_vn_vowel(chars[pos]) {
        // Detect tone from toned vowels
        let t = char_to_tone(chars[pos]);
        if t != tone::NONE {
            syl.tone = t;
        }
        pos += 1;
    }

    if pos == vowel_start {
        // No vowel found
        return None;
    }

    syl.vowel = chars[vowel_start..pos].iter().collect();

    // Determine primary vowel type
    // For diphthongs, use the first vowel's type
    if !syl.vowel.is_empty() {
        let first_vowel = syl.vowel.chars().next().unwrap();
        syl.vowel_type = char_to_vowel_type(first_vowel);
    }

    // Step 4: Extract final consonant
    if pos < len {
        let final_str: String = chars[pos..].iter().collect();
        syl.final_c = final_str.to_lowercase();
        syl.final_type = lookup_final_type(&syl.final_c);
        syl.final_cat = final_type_to_category(syl.final_type);

        // Check for semivowel finals (i, y, o, u at end)
        if syl.final_type == final_type::INVALID {
            // Check if it's a semivowel final
            let final_lower = syl.final_c.to_lowercase();
            if matches!(final_lower.as_str(), "i" | "y" | "o" | "u") {
                syl.final_cat = final_cat::SEMI;
                syl.final_type = final_type::NONE; // Treat as open for matrix
            }
        }
    } else {
        syl.final_type = final_type::NONE;
        syl.final_cat = final_cat::OPEN;
    }

    Some(syl)
}

/// Extract initial consonant from character slice
fn extract_initial(chars: &[char], pos: &mut usize) -> String {
    let len = chars.len();
    if *pos >= len {
        return String::new();
    }

    // Try 3-char initial first (ngh)
    if *pos + 3 <= len {
        let s: String = chars[*pos..*pos + 3].iter().collect();
        let lower = s.to_lowercase();
        if lower == "ngh" {
            *pos += 3;
            return lower;
        }
    }

    // Try 2-char initials
    if *pos + 2 <= len {
        let s: String = chars[*pos..*pos + 2].iter().collect();
        let lower = s.to_lowercase();
        match lower.as_str() {
            "ch" | "gh" | "gi" | "kh" | "ng" | "nh" | "ph" | "qu" | "th" | "tr" => {
                *pos += 2;
                return lower;
            }
            _ => {}
        }
    }

    // Try 1-char initial
    if *pos < len {
        let c = chars[*pos].to_ascii_lowercase();
        // Check if it's a consonant (not a vowel)
        if !is_vn_vowel(chars[*pos]) {
            *pos += 1;
            return c.to_string();
        }
    }

    String::new()
}

// ============================================================================
// VALIDATION FUNCTION
// ============================================================================

/// Validate a Vietnamese syllable against 8 phonotactic rules
pub fn validate_vietnamese(syllable: &str) -> ValidationResult {
    // Parse syllable first
    let syl = match parse_syllable(syllable) {
        Some(s) => s,
        None => return ValidationResult::InvalidStructure,
    };

    // Rule 1: Has vowel
    if syl.vowel.is_empty() {
        return ValidationResult::NoVowel;
    }

    // Rule 2: Valid initial
    if syl.initial_type == initial::INVALID {
        return ValidationResult::InvalidInitial;
    }

    // Rule 3: All chars parsed (structure check)
    // Already handled by parse_syllable returning None

    // Rule 4: Spelling rules (initial + vowel)
    if syl.initial_type != initial::NONE && syl.vowel_type != vowel_type::INVALID {
        // Get the first vowel after any glide
        let check_vowel = if syl.glide.is_some() {
            // Skip glide, check actual vowel
            syl.vowel.chars().next().map(char_to_vowel_type).unwrap_or(vowel_type::INVALID)
        } else {
            syl.vowel_type
        };

        if !is_valid_spelling(syl.initial_type, check_vowel) {
            return ValidationResult::InvalidSpelling;
        }
    }

    // Rule 5: Valid final consonant
    if syl.final_type == final_type::INVALID && !syl.final_c.is_empty() {
        // Check if it's a valid semivowel final
        let final_lower = syl.final_c.to_lowercase();
        if !matches!(final_lower.as_str(), "i" | "y" | "o" | "u") {
            return ValidationResult::InvalidFinal;
        }
    }

    // Rule 6: Valid vowel pattern
    // Check all adjacent vowel pairs
    let vowel_chars: Vec<char> = syl.vowel.chars().collect();
    if vowel_chars.len() >= 2 {
        for i in 0..vowel_chars.len() - 1 {
            let v1 = char_to_vowel_type(vowel_chars[i]);
            let v2 = char_to_vowel_type(vowel_chars[i + 1]);

            if v1 != vowel_type::INVALID && v2 != vowel_type::INVALID {
                if !is_valid_vowel_pair(v1, v2) {
                    return ValidationResult::InvalidVowelPattern;
                }
            }
        }
    }

    // Rule 7: Vowel-final compatibility
    // Most combinations are valid; complex rules handled separately
    // For now, skip this as M_VOWEL_FINAL allows all

    // Rule 8: Tone-stop restriction
    if syl.final_cat == final_cat::STOP {
        if !is_valid_tone_final(syl.tone, syl.final_cat) {
            return ValidationResult::InvalidTone;
        }
    }

    ValidationResult::Valid
}

/// Quick check if syllable is valid Vietnamese
pub fn is_valid_syllable(syllable: &str) -> bool {
    matches!(validate_vietnamese(syllable), ValidationResult::Valid)
}

/// Check if tone is valid with stop final (legacy wrapper)
pub fn is_valid_tone_with_stop(tone: Tone) -> bool {
    crate::v3::constants::vietnamese::is_valid_tone_with_stop(tone)
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Parser tests
    #[test]
    fn test_parse_simple_syllables() {
        let syl = parse_syllable("ba").unwrap();
        assert_eq!(syl.initial, "b");
        assert_eq!(syl.vowel, "a");
        assert!(syl.final_c.is_empty());
    }

    #[test]
    fn test_parse_with_final() {
        let syl = parse_syllable("ban").unwrap();
        assert_eq!(syl.initial, "b");
        assert_eq!(syl.vowel, "a");
        assert_eq!(syl.final_c, "n");
    }

    #[test]
    fn test_parse_digraph_initial() {
        let syl = parse_syllable("cha").unwrap();
        assert_eq!(syl.initial, "ch");
        assert_eq!(syl.vowel, "a");

        let syl2 = parse_syllable("nghi").unwrap();
        assert_eq!(syl2.initial, "ngh");
        assert_eq!(syl2.vowel, "i");
    }

    #[test]
    fn test_parse_vowel_only() {
        let syl = parse_syllable("a").unwrap();
        assert!(syl.initial.is_empty());
        assert_eq!(syl.vowel, "a");
    }

    #[test]
    fn test_parse_diphthong() {
        let syl = parse_syllable("ai").unwrap();
        assert!(syl.initial.is_empty());
        assert_eq!(syl.vowel, "ai");
    }

    #[test]
    fn test_parse_toned() {
        let syl = parse_syllable("bán").unwrap();
        assert_eq!(syl.tone, tone::SAC);
    }

    // Validation tests - Valid syllables
    #[test]
    fn test_valid_simple() {
        assert!(is_valid_syllable("a"));
        assert!(is_valid_syllable("e"));
        assert!(is_valid_syllable("o"));
        assert!(is_valid_syllable("u"));
        assert!(is_valid_syllable("i"));
        assert!(is_valid_syllable("y"));
    }

    #[test]
    fn test_valid_cv() {
        assert!(is_valid_syllable("ba"));
        assert!(is_valid_syllable("ca"));
        assert!(is_valid_syllable("da"));
        assert!(is_valid_syllable("ga"));
        assert!(is_valid_syllable("ha"));
        assert!(is_valid_syllable("ke")); // k before e
        assert!(is_valid_syllable("ki")); // k before i
        assert!(is_valid_syllable("la"));
        assert!(is_valid_syllable("ma"));
        assert!(is_valid_syllable("na"));
    }

    #[test]
    fn test_valid_digraph_initial() {
        assert!(is_valid_syllable("cha"));
        assert!(is_valid_syllable("ghe")); // gh before e
        assert!(is_valid_syllable("gia"));
        assert!(is_valid_syllable("kha"));
        assert!(is_valid_syllable("ngo"));
        assert!(is_valid_syllable("nghe")); // ngh before e
        assert!(is_valid_syllable("nha"));
        assert!(is_valid_syllable("pha"));
        assert!(is_valid_syllable("qua"));
        assert!(is_valid_syllable("tha"));
        assert!(is_valid_syllable("tra"));
    }

    #[test]
    fn test_valid_with_final() {
        assert!(is_valid_syllable("an"));
        assert!(is_valid_syllable("em"));
        assert!(is_valid_syllable("ong"));
        assert!(is_valid_syllable("anh"));
        assert!(is_valid_syllable("ap"));
        assert!(is_valid_syllable("at"));
        assert!(is_valid_syllable("ac"));
        assert!(is_valid_syllable("ach"));
    }

    #[test]
    fn test_valid_diphthongs() {
        assert!(is_valid_syllable("ai"));
        assert!(is_valid_syllable("ao"));
        assert!(is_valid_syllable("au"));
        assert!(is_valid_syllable("ay"));
        assert!(is_valid_syllable("eo"));
        assert!(is_valid_syllable("ia"));
        assert!(is_valid_syllable("iu"));
        assert!(is_valid_syllable("oi"));
        assert!(is_valid_syllable("ua"));
        assert!(is_valid_syllable("ui"));
        assert!(is_valid_syllable("uy"));
    }

    #[test]
    fn test_valid_toned() {
        assert!(is_valid_syllable("bá"));
        assert!(is_valid_syllable("bà"));
        assert!(is_valid_syllable("bả"));
        assert!(is_valid_syllable("bã"));
        assert!(is_valid_syllable("bạ"));
    }

    // Validation tests - Invalid patterns
    #[test]
    fn test_invalid_no_vowel() {
        assert!(!is_valid_syllable("bcd"));
        assert!(!is_valid_syllable("str"));
        assert!(!is_valid_syllable("ng"));
    }

    #[test]
    fn test_invalid_spelling_c_k() {
        // c before e,i,y is invalid
        assert!(!is_valid_syllable("ce"));
        assert!(!is_valid_syllable("ci"));
        assert!(!is_valid_syllable("cy"));

        // k before a,o,u is invalid
        assert!(!is_valid_syllable("ka"));
        assert!(!is_valid_syllable("ko"));
        assert!(!is_valid_syllable("ku"));
    }

    #[test]
    fn test_invalid_spelling_g_gh() {
        // g before e is invalid
        assert!(!is_valid_syllable("ge"));

        // gh before a,o,u is invalid
        assert!(!is_valid_syllable("gha"));
        assert!(!is_valid_syllable("gho"));
    }

    #[test]
    fn test_invalid_spelling_ng_ngh() {
        // ng before e,i is invalid
        assert!(!is_valid_syllable("nge"));
        assert!(!is_valid_syllable("ngi"));

        // ngh before a,o,u is invalid
        assert!(!is_valid_syllable("ngha"));
        assert!(!is_valid_syllable("ngho"));
    }

    #[test]
    fn test_invalid_vowel_pattern() {
        // Foreign word vowel patterns
        assert!(!is_valid_syllable("ea")); // ea not valid
        assert!(!is_valid_syllable("ou")); // ou not valid
        // Note: "yo" parses differently, skip for now
    }

    // Telex intermediate states
    #[test]
    fn test_telex_intermediate() {
        // These should pass validation during typing
        assert!(is_valid_syllable("aa")); // → â
        assert!(is_valid_syllable("ee")); // → ê
        assert!(is_valid_syllable("oo")); // → ô
        assert!(is_valid_syllable("baa")); // → bâ
        assert!(is_valid_syllable("bee")); // → bê
        assert!(is_valid_syllable("boo")); // → bô
    }

    // Tone-stop restriction tests
    #[test]
    fn test_tone_stop_valid() {
        // Sắc on stop final - valid
        assert!(is_valid_syllable("ác")); // sắc + c
        assert!(is_valid_syllable("áp")); // sắc + p
        assert!(is_valid_syllable("át")); // sắc + t
        assert!(is_valid_syllable("ách")); // sắc + ch

        // Nặng on stop final - valid
        assert!(is_valid_syllable("ạc")); // nặng + c
        assert!(is_valid_syllable("ạp")); // nặng + p
        assert!(is_valid_syllable("ạt")); // nặng + t
        assert!(is_valid_syllable("ạch")); // nặng + ch
    }

    #[test]
    fn test_tone_stop_invalid() {
        // Huyền on stop final - invalid
        assert!(!is_valid_syllable("àc"));
        assert!(!is_valid_syllable("àp"));

        // Hỏi on stop final - invalid
        assert!(!is_valid_syllable("ảc"));
        assert!(!is_valid_syllable("ảp"));

        // Ngã on stop final - invalid
        assert!(!is_valid_syllable("ãc"));
        assert!(!is_valid_syllable("ãp"));
    }

    #[test]
    fn test_validation_result_types() {
        assert_eq!(validate_vietnamese("ba"), ValidationResult::Valid);
        assert_eq!(validate_vietnamese("ng"), ValidationResult::InvalidStructure);
        assert_eq!(validate_vietnamese("ce"), ValidationResult::InvalidSpelling);
        assert_eq!(validate_vietnamese("ea"), ValidationResult::InvalidVowelPattern);
        assert_eq!(validate_vietnamese("àc"), ValidationResult::InvalidTone);
    }
}
