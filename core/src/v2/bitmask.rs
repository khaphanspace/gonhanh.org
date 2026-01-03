//! Bitmask constants for O(1) Vietnamese validation
//!
//! Uses compact bitmask representations for:
//! - Character type classification (33 entries)
//! - Valid onset/coda combinations
//! - Onset/coda clusters
//! - Tone-stop restrictions

/// Character index mapping for bitmask operations.
/// a=0, b=1, ..., z=25, đ=26, ă=27, â=28, ê=29, ô=30, ơ=31, ư=32
#[inline]
pub fn char_idx(c: char) -> usize {
    match c.to_ascii_lowercase() {
        'a'..='z' => (c.to_ascii_lowercase() as usize) - ('a' as usize),
        'đ' => 26,
        'ă' => 27,
        'â' => 28,
        'ê' => 29,
        'ô' => 30,
        'ơ' => 31,
        'ư' => 32,
        _ => 33, // Invalid
    }
}

/// Get base vowel (strip tones) for validation
#[inline]
pub fn get_base_vowel(c: char) -> char {
    match c {
        // A variants
        'á' | 'à' | 'ả' | 'ã' | 'ạ' => 'a',
        'ắ' | 'ằ' | 'ẳ' | 'ẵ' | 'ặ' => 'ă',
        'ấ' | 'ầ' | 'ẩ' | 'ẫ' | 'ậ' => 'â',
        // E variants
        'é' | 'è' | 'ẻ' | 'ẽ' | 'ẹ' => 'e',
        'ế' | 'ề' | 'ể' | 'ễ' | 'ệ' => 'ê',
        // I variants
        'í' | 'ì' | 'ỉ' | 'ĩ' | 'ị' => 'i',
        // O variants
        'ó' | 'ò' | 'ỏ' | 'õ' | 'ọ' => 'o',
        'ố' | 'ồ' | 'ổ' | 'ỗ' | 'ộ' => 'ô',
        'ớ' | 'ờ' | 'ở' | 'ỡ' | 'ợ' => 'ơ',
        // U variants
        'ú' | 'ù' | 'ủ' | 'ũ' | 'ụ' => 'u',
        'ứ' | 'ừ' | 'ử' | 'ữ' | 'ự' => 'ư',
        // Y variants
        'ý' | 'ỳ' | 'ỷ' | 'ỹ' | 'ỵ' => 'y',
        // Base characters
        _ => c.to_ascii_lowercase(),
    }
}

/// Character type flags
pub const ONSET: u8 = 0b0001;
pub const VOWEL: u8 = 0b0010;
pub const CODA: u8 = 0b0100;
pub const INVALID: u8 = 0b1000;

/// Character type table (33 entries)
/// Index: a=0, b=1, ..., z=25, đ=26, ă=27, â=28, ê=29, ô=30, ơ=31, ư=32
pub const CHAR_TYPE: [u8; 33] = [
    // a      b      c           d      e      f        g      h
    VOWEL, ONSET, ONSET | CODA, ONSET, VOWEL, INVALID, ONSET, ONSET,
    // i           j        k      l      m           n           o           p
    VOWEL | CODA, INVALID, ONSET, ONSET, ONSET | CODA, ONSET | CODA, VOWEL | CODA, ONSET | CODA,
    // q      r      s      t           u           v      w        x
    ONSET, ONSET, ONSET, ONSET | CODA, VOWEL | CODA, ONSET, INVALID, ONSET,
    // y           z        đ      ă      â      ê      ô      ơ      ư
    VOWEL | CODA, INVALID, ONSET, VOWEL, VOWEL, VOWEL, VOWEL, VOWEL, VOWEL,
];

/// Valid VN single onsets bitmask (17 consonants)
/// b,c,d,đ,g,h,k,l,m,n,p,q,r,s,t,v,x
/// bit positions: a=0, b=1, c=2, d=3, ... đ=26
pub const M_ONSET: u32 = 0b_0000_0100_0010_0101_1111_1110_1111_1110 | (1 << 26);
// x=23, v=21, t=19, s=18, r=17, q=16, p=15, n=13, m=12, l=11, k=10, h=7, g=6, d=3, c=2, b=1, đ=26

/// Valid VN single codas bitmask (5 + 4 semi-vowels)
/// c,m,n,p,t (true consonant codas)
/// i,o,u,y (semi-vowel codas)
/// Bit positions: c=2, i=8, m=12, n=13, o=14, p=15, t=19, u=20, y=24
pub const M_CODA: u32 = (1 << 2) | (1 << 8) | (1 << 12) | (1 << 13) | (1 << 14) | (1 << 15) | (1 << 19) | (1 << 20) | (1 << 24);

/// Valid VN onset clusters (10 two-char clusters)
/// Note: "ngh" is a 3-char cluster handled separately in validate.rs check_onset()
pub const VN_ONSET_CLUSTERS: &[[u8; 2]] = &[
    *b"ch", *b"gh", *b"gi", *b"kh", *b"ng",
    *b"nh", *b"ph", *b"qu", *b"th", *b"tr",
];

/// Valid VN coda clusters (3 clusters)
pub const VN_CODA_CLUSTERS: &[[u8; 2]] = &[*b"ch", *b"ng", *b"nh"];

/// Valid VN diphthongs (29 patterns)
/// From spec Section 2.2
pub const VALID_DIPHTHONGS: &[[char; 2]] = &[
    ['a', 'i'], ['a', 'o'], ['a', 'u'], ['a', 'y'],
    ['â', 'u'], ['â', 'y'],
    ['e', 'o'], ['ê', 'u'],
    ['i', 'a'], ['i', 'ê'], ['i', 'u'],
    ['o', 'a'], ['o', 'ă'], ['o', 'e'], ['o', 'i'],
    ['ô', 'i'], ['ơ', 'i'],
    ['u', 'a'], ['u', 'â'], ['u', 'ê'], ['u', 'i'], ['u', 'ô'], ['u', 'y'],
    ['ư', 'a'], ['ư', 'i'], ['ư', 'ơ'], ['ư', 'u'],
    ['y', 'ê'],
];

/// Valid VN triphthongs (14 patterns)
/// From spec Section 2.2
pub const VALID_TRIPHTHONGS: &[[char; 3]] = &[
    ['i', 'ê', 'u'], ['y', 'ê', 'u'],
    ['o', 'a', 'i'], ['o', 'a', 'y'], ['o', 'a', 'o'], ['o', 'e', 'o'],
    ['u', 'â', 'y'], ['u', 'ô', 'i'], ['u', 'y', 'a'], ['u', 'y', 'ê'], ['u', 'y', 'u'], ['u', 'ê', 'u'],
    ['ư', 'ơ', 'i'], ['ư', 'ơ', 'u'],
];

/// Tone-stop restriction check
/// Stops (c, ch, p, t) only allow sắc (1) and nặng (5)
#[inline]
pub fn is_valid_tone_for_coda(tone: u8, coda: &str) -> bool {
    let is_stop = matches!(coda, "c" | "ch" | "p" | "t");
    if is_stop {
        // Only sắc (1) and nặng (5) allowed with stops
        // 0 = no tone, also allowed
        matches!(tone, 0 | 1 | 5)
    } else {
        true
    }
}

/// Get tone value from toned vowel character
#[inline]
pub fn get_tone(c: char) -> u8 {
    match c {
        // Sắc (1)
        'á' | 'ắ' | 'ấ' | 'é' | 'ế' | 'í' | 'ó' | 'ố' | 'ớ' | 'ú' | 'ứ' | 'ý' => 1,
        // Huyền (2)
        'à' | 'ằ' | 'ầ' | 'è' | 'ề' | 'ì' | 'ò' | 'ồ' | 'ờ' | 'ù' | 'ừ' | 'ỳ' => 2,
        // Hỏi (3)
        'ả' | 'ẳ' | 'ẩ' | 'ẻ' | 'ể' | 'ỉ' | 'ỏ' | 'ổ' | 'ở' | 'ủ' | 'ử' | 'ỷ' => 3,
        // Ngã (4)
        'ã' | 'ẵ' | 'ẫ' | 'ẽ' | 'ễ' | 'ĩ' | 'õ' | 'ỗ' | 'ỡ' | 'ũ' | 'ữ' | 'ỹ' => 4,
        // Nặng (5)
        'ạ' | 'ặ' | 'ậ' | 'ẹ' | 'ệ' | 'ị' | 'ọ' | 'ộ' | 'ợ' | 'ụ' | 'ự' | 'ỵ' => 5,
        // No tone (0)
        _ => 0,
    }
}

/// Check if character is a Vietnamese vowel (including modified)
#[inline]
pub fn is_vn_vowel(c: char) -> bool {
    let base = get_base_vowel(c);
    matches!(
        base,
        'a' | 'ă' | 'â' | 'e' | 'ê' | 'i' | 'o' | 'ô' | 'ơ' | 'u' | 'ư' | 'y'
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_idx() {
        assert_eq!(char_idx('a'), 0);
        assert_eq!(char_idx('z'), 25);
        assert_eq!(char_idx('đ'), 26);
        assert_eq!(char_idx('ă'), 27);
        assert_eq!(char_idx('ư'), 32);
        assert_eq!(char_idx('@'), 33); // Invalid
    }

    #[test]
    fn test_char_type_onset() {
        // Valid onsets
        assert!(CHAR_TYPE[char_idx('b')] & ONSET != 0);
        assert!(CHAR_TYPE[char_idx('c')] & ONSET != 0);
        assert!(CHAR_TYPE[char_idx('đ')] & ONSET != 0);
        // Invalid onset
        assert!(CHAR_TYPE[char_idx('a')] & ONSET == 0);
    }

    #[test]
    fn test_char_type_vowel() {
        assert!(CHAR_TYPE[char_idx('a')] & VOWEL != 0);
        assert!(CHAR_TYPE[char_idx('e')] & VOWEL != 0);
        assert!(CHAR_TYPE[char_idx('ă')] & VOWEL != 0);
        assert!(CHAR_TYPE[char_idx('ư')] & VOWEL != 0);
    }

    #[test]
    fn test_char_type_coda() {
        assert!(CHAR_TYPE[char_idx('c')] & CODA != 0);
        assert!(CHAR_TYPE[char_idx('m')] & CODA != 0);
        assert!(CHAR_TYPE[char_idx('n')] & CODA != 0);
        assert!(CHAR_TYPE[char_idx('p')] & CODA != 0);
        assert!(CHAR_TYPE[char_idx('t')] & CODA != 0);
    }

    #[test]
    fn test_char_type_invalid() {
        assert!(CHAR_TYPE[char_idx('f')] & INVALID != 0);
        assert!(CHAR_TYPE[char_idx('j')] & INVALID != 0);
        assert!(CHAR_TYPE[char_idx('w')] & INVALID != 0);
        assert!(CHAR_TYPE[char_idx('z')] & INVALID != 0);
    }

    #[test]
    fn test_get_tone() {
        assert_eq!(get_tone('á'), 1);
        assert_eq!(get_tone('à'), 2);
        assert_eq!(get_tone('ả'), 3);
        assert_eq!(get_tone('ã'), 4);
        assert_eq!(get_tone('ạ'), 5);
        assert_eq!(get_tone('a'), 0);
    }

    #[test]
    fn test_tone_stop_restriction() {
        // Stops only allow sắc (1) and nặng (5)
        assert!(is_valid_tone_for_coda(0, "c")); // No tone OK
        assert!(is_valid_tone_for_coda(1, "c")); // sắc OK
        assert!(!is_valid_tone_for_coda(2, "c")); // huyền NOT OK
        assert!(!is_valid_tone_for_coda(3, "ch")); // hỏi NOT OK
        assert!(!is_valid_tone_for_coda(4, "p")); // ngã NOT OK
        assert!(is_valid_tone_for_coda(5, "t")); // nặng OK

        // Non-stops allow all tones
        assert!(is_valid_tone_for_coda(2, "m"));
        assert!(is_valid_tone_for_coda(3, "n"));
        assert!(is_valid_tone_for_coda(4, "ng"));
    }

    #[test]
    fn test_get_base_vowel() {
        assert_eq!(get_base_vowel('á'), 'a');
        assert_eq!(get_base_vowel('ắ'), 'ă');
        assert_eq!(get_base_vowel('ấ'), 'â');
        assert_eq!(get_base_vowel('ế'), 'ê');
        assert_eq!(get_base_vowel('ớ'), 'ơ');
        assert_eq!(get_base_vowel('ự'), 'ư');
    }

    #[test]
    fn test_is_vn_vowel() {
        assert!(is_vn_vowel('a'));
        assert!(is_vn_vowel('á'));
        assert!(is_vn_vowel('ă'));
        assert!(is_vn_vowel('â'));
        assert!(is_vn_vowel('ư'));
        assert!(is_vn_vowel('ự'));
        assert!(!is_vn_vowel('b'));
        assert!(!is_vn_vowel('c'));
    }

    #[test]
    fn test_valid_diphthongs() {
        assert!(VALID_DIPHTHONGS.contains(&['a', 'i']));
        assert!(VALID_DIPHTHONGS.contains(&['o', 'a']));
        assert!(VALID_DIPHTHONGS.contains(&['ư', 'ơ']));
        assert!(!VALID_DIPHTHONGS.contains(&['b', 'c']));
    }

    #[test]
    fn test_valid_triphthongs() {
        assert!(VALID_TRIPHTHONGS.contains(&['o', 'a', 'i']));
        assert!(VALID_TRIPHTHONGS.contains(&['ư', 'ơ', 'i']));
        assert!(!VALID_TRIPHTHONGS.contains(&['a', 'b', 'c']));
    }
}
