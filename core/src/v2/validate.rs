//! Step 5: 9-Layer Vietnamese Validation
//!
//! Validates Vietnamese syllable structure using O(1) bitmask operations.
//!
//! ## 9 Validation Layers
//!
//! | Layer | Check | Method |
//! |-------|-------|--------|
//! | L1 | CHAR_TYPE | CHAR_TYPE[33] lookup |
//! | L2 | ONSET (single) | M_ONSET bitmask |
//! | L3 | ONSET_CLUSTER | Hardcoded list |
//! | L4 | VOWEL_PATTERN | Diphthong/Triphthong lists |
//! | L5 | CODA (single) | M_CODA bitmask |
//! | L6 | CODA_CLUSTER | Hardcoded list |
//! | L7 | TONE_STOP | Stop coda restriction |
//! | L8 | SPELLING | c/k, g/gh, ng/ngh rules |
//! | L9 | MODIFIER_REQ | Circumflex requirements |

use super::bitmask::*;
use super::state::VnState;

/// Parsed syllable structure
#[derive(Debug, Default)]
pub struct Syllable {
    pub onset: Vec<char>,
    pub vowels: Vec<char>,
    pub coda: Vec<char>,
    pub tone: u8,
}

impl Syllable {
    /// Get onset as lowercase string
    pub fn onset_str(&self) -> String {
        self.onset.iter().collect::<String>().to_ascii_lowercase()
    }

    /// Get coda as lowercase string
    pub fn coda_str(&self) -> String {
        self.coda.iter().collect::<String>().to_ascii_lowercase()
    }

    /// Get vowels as base characters (strip tones)
    pub fn vowels_base(&self) -> Vec<char> {
        self.vowels.iter().map(|&c| get_base_vowel(c)).collect()
    }
}

/// Validate buffer as Vietnamese syllable
///
/// Returns VnState indicating validation result:
/// - Complete: Valid complete syllable
/// - Incomplete: Could become valid (consonant-only)
/// - Impossible: Cannot be valid Vietnamese
#[inline]
pub fn validate_vn(buffer: &str) -> VnState {
    if buffer.is_empty() {
        return VnState::Incomplete;
    }

    let syllable = parse_syllable(buffer);

    // L1: Character type check
    if !check_char_types(buffer) {
        return VnState::Impossible;
    }

    // L2-L3: Onset validation
    if !check_onset(&syllable) {
        return VnState::Impossible;
    }

    // L4: Vowel pattern validation
    if syllable.vowels.is_empty() {
        // Consonant-only is incomplete (could add vowel)
        return VnState::Incomplete;
    }
    if !check_vowel_pattern(&syllable) {
        return VnState::Impossible;
    }

    // L5-L6: Coda validation
    if !check_coda(&syllable) {
        return VnState::Impossible;
    }

    // L7: Tone-stop restriction
    if !check_tone_stop(&syllable) {
        return VnState::Impossible;
    }

    // L8: Spelling rules
    if !check_spelling(&syllable) {
        return VnState::Impossible;
    }

    // L9: Modifier requirements (circumflex for êu, iê, etc.)
    if !check_modifier_req(&syllable) {
        return VnState::Impossible;
    }

    VnState::Complete
}

/// Parse buffer into syllable structure (onset, vowels, coda)
pub fn parse_syllable(buffer: &str) -> Syllable {
    let mut syllable = Syllable::default();
    let chars: Vec<char> = buffer.chars().collect();
    let len = chars.len();
    let mut idx = 0;
    let mut tone: u8 = 0;

    // Parse onset (consonant cluster at start)
    // Special handling for "gi" cluster - 'i' is part of onset, not vowel
    while idx < len {
        let c = chars[idx];
        let is_vowel = is_vn_vowel(c);

        // Check for "gi" cluster: g + i + vowel
        if syllable.onset.len() == 1
            && syllable.onset[0].to_ascii_lowercase() == 'g'
            && c.to_ascii_lowercase() == 'i'
        {
            // Check if there's another vowel after 'i'
            if idx + 1 < len && is_vn_vowel(chars[idx + 1]) {
                // "gi" is onset cluster (gia, giê, etc.)
                syllable.onset.push(c);
                idx += 1;
                continue;
            }
        }

        // Check for "qu" cluster: q + u
        if syllable.onset.len() == 1
            && syllable.onset[0].to_ascii_lowercase() == 'q'
            && c.to_ascii_lowercase() == 'u'
        {
            syllable.onset.push(c);
            idx += 1;
            continue;
        }

        if is_vowel {
            // Start of vowel nucleus
            break;
        }

        syllable.onset.push(c);
        idx += 1;
    }

    // Parse vowel nucleus
    while idx < len {
        let c = chars[idx];
        let is_vowel = is_vn_vowel(c);
        let base = get_base_vowel(c);

        // Track tone from any toned vowel
        let char_tone = get_tone(c);
        if char_tone > 0 {
            tone = char_tone;
        }

        if is_vowel {
            syllable.vowels.push(base);
            idx += 1;
        } else {
            // Start of coda
            break;
        }
    }

    // Parse coda (remaining consonants)
    while idx < len {
        let c = chars[idx];
        syllable.coda.push(c);
        idx += 1;
    }

    syllable.tone = tone;
    syllable
}

/// L1: Check all characters are valid Vietnamese characters
#[inline]
fn check_char_types(buffer: &str) -> bool {
    for c in buffer.chars() {
        // Allow Vietnamese vowels (base and toned)
        if is_vn_vowel(c) {
            continue;
        }

        // Check against CHAR_TYPE table
        let idx = char_idx(c);
        if idx >= CHAR_TYPE.len() {
            return false; // Unknown character
        }
        if CHAR_TYPE[idx] & INVALID != 0 {
            return false; // f, j, w, z are invalid
        }
    }
    true
}

/// L2-L3: Check onset validity (single consonant or cluster)
#[inline]
fn check_onset(syllable: &Syllable) -> bool {
    match syllable.onset.len() {
        0 => true, // No onset is valid (vowel-initial)
        1 => {
            let c = syllable.onset[0];
            let idx = char_idx(c);
            // Allow đ (index 26) as valid single onset
            if idx > 26 {
                return false;
            }
            // Check if it's a valid single onset using bitmask
            (M_ONSET >> idx) & 1 == 1
        }
        2 => {
            // Check against onset clusters
            let pair = [
                syllable.onset[0].to_ascii_lowercase() as u8,
                syllable.onset[1].to_ascii_lowercase() as u8,
            ];
            VN_ONSET_CLUSTERS.contains(&pair)
        }
        3 => {
            // Only "ngh" is valid 3-char onset
            syllable.onset_str() == "ngh"
        }
        _ => false, // 4+ consonant onset is invalid
    }
}

/// L4: Check vowel pattern (diphthong/triphthong validity)
#[inline]
fn check_vowel_pattern(syllable: &Syllable) -> bool {
    let vowels = &syllable.vowels;
    match vowels.len() {
        0 => true, // Handled separately (incomplete)
        1 => true, // Single vowel always valid
        2 => {
            // Check against valid diphthongs
            let pair = [vowels[0], vowels[1]];
            VALID_DIPHTHONGS.contains(&pair)
        }
        3 => {
            // Check against valid triphthongs
            let triple = [vowels[0], vowels[1], vowels[2]];
            VALID_TRIPHTHONGS.contains(&triple)
        }
        _ => false, // 4+ vowels invalid
    }
}

/// L5-L6: Check coda validity (single consonant or cluster)
#[inline]
fn check_coda(syllable: &Syllable) -> bool {
    match syllable.coda.len() {
        0 => true, // Open syllable (no coda)
        1 => {
            let c = syllable.coda[0].to_ascii_lowercase();
            let idx = char_idx(c);
            if idx >= 33 {
                return false; // Invalid character index
            }
            // Check if it's a valid single coda
            (M_CODA >> idx) & 1 == 1
        }
        2 => {
            // Check against coda clusters
            let pair = [
                syllable.coda[0].to_ascii_lowercase() as u8,
                syllable.coda[1].to_ascii_lowercase() as u8,
            ];
            VN_CODA_CLUSTERS.contains(&pair)
        }
        _ => false, // 3+ coda is invalid
    }
}

/// L7: Check tone-stop restriction
/// Stops (c, ch, p, t) only allow sắc (1) and nặng (5)
#[inline]
fn check_tone_stop(syllable: &Syllable) -> bool {
    if syllable.coda.is_empty() {
        return true;
    }
    let coda = syllable.coda_str();
    is_valid_tone_for_coda(syllable.tone, &coda)
}

/// L8: Check spelling rules (c/k, g/gh, ng/ngh)
///
/// - c → k before e/i/y (ce, ci, cy invalid)
/// - g → gh before e/i (ge, gi invalid without h)
/// - ng → ngh before e/i (nge, ngi invalid)
#[inline]
fn check_spelling(syllable: &Syllable) -> bool {
    if syllable.onset.is_empty() || syllable.vowels.is_empty() {
        return true;
    }

    let onset = syllable.onset_str();
    let first_vowel = syllable.vowels[0];

    // c → k before e/i/y
    if onset == "c" && matches!(first_vowel, 'e' | 'ê' | 'i' | 'y') {
        return false;
    }

    // g → gh before e/i
    // Exception: "g" + single vowel 'i' is valid (gì, gỉ, gí, gĩ, gị)
    // These are standalone gi-words where 'i' is the only vowel
    if onset == "g" && matches!(first_vowel, 'e' | 'ê') {
        return false;
    }
    // "gi" with just one vowel 'i' is valid (gì = what)
    // "gi" before another vowel needs "gi" cluster handling in parse_syllable
    if onset == "g" && first_vowel == 'i' && syllable.vowels.len() > 1 {
        return false;
    }

    // ng → ngh before e/i
    if onset == "ng" && matches!(first_vowel, 'e' | 'ê' | 'i') {
        return false;
    }

    true
}

/// L9: Check modifier requirements
/// Some vowel combinations require specific modifiers
#[inline]
fn check_modifier_req(syllable: &Syllable) -> bool {
    let vowels = &syllable.vowels;
    if vowels.len() < 2 {
        return true;
    }

    // iê requires circumflex on ê (but base form ie is also typed)
    // This is more of a transformation rule than validation
    // For now, accept both ie and iê

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_syllable_simple() {
        let s = parse_syllable("ba");
        assert_eq!(s.onset, vec!['b']);
        assert_eq!(s.vowels, vec!['a']);
        assert!(s.coda.is_empty());
    }

    #[test]
    fn test_parse_syllable_with_coda() {
        let s = parse_syllable("ban");
        assert_eq!(s.onset, vec!['b']);
        assert_eq!(s.vowels, vec!['a']);
        assert_eq!(s.coda, vec!['n']);
    }

    #[test]
    fn test_parse_syllable_cluster_onset() {
        let s = parse_syllable("cha");
        assert_eq!(s.onset, vec!['c', 'h']);
        assert_eq!(s.vowels, vec!['a']);
    }

    #[test]
    fn test_parse_syllable_diphthong() {
        let s = parse_syllable("bai");
        assert_eq!(s.onset, vec!['b']);
        assert_eq!(s.vowels, vec!['a', 'i']);
    }

    #[test]
    fn test_parse_syllable_triphthong() {
        let s = parse_syllable("oai");
        assert!(s.onset.is_empty());
        assert_eq!(s.vowels, vec!['o', 'a', 'i']);
    }

    #[test]
    fn test_parse_syllable_with_tone() {
        let s = parse_syllable("bán");
        assert_eq!(s.tone, 1); // sắc
        assert_eq!(s.vowels, vec!['a']); // Base vowel stored
    }

    #[test]
    fn test_validate_valid_syllables() {
        assert_eq!(validate_vn("ba"), VnState::Complete);
        assert_eq!(validate_vn("an"), VnState::Complete);
        assert_eq!(validate_vn("cha"), VnState::Complete);
        assert_eq!(validate_vn("bai"), VnState::Complete);
        assert_eq!(validate_vn("oai"), VnState::Complete);
    }

    #[test]
    fn test_validate_incomplete() {
        assert_eq!(validate_vn("b"), VnState::Incomplete);
        assert_eq!(validate_vn("ch"), VnState::Incomplete);
        assert_eq!(validate_vn(""), VnState::Incomplete);
    }

    #[test]
    fn test_validate_invalid_onset() {
        // "cl" is English-only cluster
        assert_eq!(validate_vn("cla"), VnState::Impossible);
        // "bl" is English-only
        assert_eq!(validate_vn("bla"), VnState::Impossible);
    }

    #[test]
    fn test_validate_spelling_rules() {
        // "ce" invalid (should be "ke")
        assert_eq!(validate_vn("ce"), VnState::Impossible);
        // "ge" invalid (should be "ghe")
        assert_eq!(validate_vn("ge"), VnState::Impossible);
        // "nge" invalid (should be "nghe")
        assert_eq!(validate_vn("nge"), VnState::Impossible);
        // Valid forms
        assert_eq!(validate_vn("ke"), VnState::Complete);
        assert_eq!(validate_vn("ghe"), VnState::Complete);
        assert_eq!(validate_vn("nghe"), VnState::Complete);
    }

    #[test]
    fn test_validate_tone_stop() {
        // Stop codas only allow sắc (1) and nặng (5)
        assert_eq!(validate_vn("bác"), VnState::Complete); // sắc + c OK
        assert_eq!(validate_vn("bạc"), VnState::Complete); // nặng + c OK
                                                           // Invalid: huyền with stop
                                                           // Note: "bàc" would have tone=2 (huyền)
        let s = parse_syllable("bàc");
        assert_eq!(s.tone, 2);
        assert_eq!(validate_vn("bàc"), VnState::Impossible);
    }

    #[test]
    fn test_validate_invalid_vowel_pattern() {
        // "eu" is not a valid Vietnamese diphthong (ê+u is, but not e+u)
        // Actually "eo" is valid, but "eu" is not
        assert_eq!(validate_vn("beu"), VnState::Impossible);
    }

    #[test]
    fn test_validate_invalid_coda() {
        // "bb" is not a valid coda
        assert_eq!(validate_vn("abb"), VnState::Impossible);
        // "ngh" is not a valid coda (it's an onset)
        // Actually single-char check: "h" alone is not valid coda
        let s = parse_syllable("bah");
        assert_eq!(s.coda, vec!['h']);
        assert_eq!(validate_vn("bah"), VnState::Impossible);
    }

    #[test]
    fn test_validate_with_modified_vowels() {
        assert_eq!(validate_vn("đi"), VnState::Complete);
        assert_eq!(validate_vn("ăn"), VnState::Complete);
        assert_eq!(validate_vn("ướt"), VnState::Complete);
    }

    #[test]
    fn test_check_char_types_invalid() {
        // 'f', 'j', 'w', 'z' are invalid in Vietnamese
        assert!(!check_char_types("fa"));
        assert!(!check_char_types("ja"));
        assert!(!check_char_types("wa"));
        assert!(!check_char_types("za"));
    }

    #[test]
    fn test_valid_onset_clusters() {
        // All VN onset clusters should be valid
        for cluster in VN_ONSET_CLUSTERS {
            let s = format!("{}a", std::str::from_utf8(cluster).unwrap());
            assert_eq!(
                validate_vn(&s),
                VnState::Complete,
                "Failed for cluster: {:?}",
                cluster
            );
        }
    }

    #[test]
    fn test_valid_coda_clusters() {
        // All VN coda clusters should be valid
        for cluster in VN_CODA_CLUSTERS {
            let s = format!("a{}", std::str::from_utf8(cluster).unwrap());
            assert_eq!(
                validate_vn(&s),
                VnState::Complete,
                "Failed for coda: {:?}",
                cluster
            );
        }
    }

    #[test]
    fn test_validate_tieng() {
        let s = "tiếng";
        println!("Testing: '{}'", s);

        let syllable = parse_syllable(s);
        println!(
            "Parsed: onset={:?}, vowels={:?}, coda={:?}, tone={}",
            syllable.onset, syllable.vowels, syllable.coda, syllable.tone
        );

        // Check each layer
        println!("L1 check_char_types: {}", check_char_types(s));
        println!("L2-L3 check_onset: {}", check_onset(&syllable));
        println!("L4 check_vowel_pattern: {}", check_vowel_pattern(&syllable));
        println!("L5-L6 check_coda: {}", check_coda(&syllable));
        println!("L7 check_tone_stop: {}", check_tone_stop(&syllable));
        println!("L8 check_spelling: {}", check_spelling(&syllable));
        println!("L9 check_modifier_req: {}", check_modifier_req(&syllable));

        let result = validate_vn(s);
        println!("validate_vn result: {:?}", result);

        assert_eq!(result, VnState::Complete);
    }
}
