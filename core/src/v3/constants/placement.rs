//! Tone and modifier placement rules
//!
//! Vietnamese tone marks must be placed on specific vowels:
//! - Single vowel: place on that vowel
//! - Diphthong: follow phonological rules
//! - Triphthong: place on middle vowel
//!
//! Modifier marks (circumflex, horn, breve) target specific vowels.

/// Vowel priority for tone placement
/// Higher number = higher priority for receiving tone
pub const VOWEL_PRIORITY: [u8; 12] = [
    5, // a - highest priority
    5, // ă
    5, // â
    4, // e
    4, // ê
    2, // i
    3, // o
    3, // ô
    3, // ơ
    2, // u
    2, // ư
    1, // y - lowest priority
];

/// Get vowel index (0-11) from character
#[inline]
pub fn vowel_index(c: char) -> Option<usize> {
    match c.to_ascii_lowercase() {
        'a' => Some(0),
        'ă' => Some(1),
        'â' => Some(2),
        'e' => Some(3),
        'ê' => Some(4),
        'i' => Some(5),
        'o' => Some(6),
        'ô' => Some(7),
        'ơ' => Some(8),
        'u' => Some(9),
        'ư' => Some(10),
        'y' => Some(11),
        _ => None,
    }
}

/// Find which vowel should receive the tone in a vowel sequence
/// Returns the index (0-based) of the vowel to receive tone
pub fn find_tone_position(vowels: &[char]) -> usize {
    match vowels.len() {
        0 => 0,
        1 => 0,
        2 => {
            // Diphthong rules:
            // - If second vowel is i/y, tone goes on first
            // - If first vowel is o/u followed by a/ă/â, tone goes on second
            // - Otherwise, default to first
            let v1 = vowels[0].to_ascii_lowercase();
            let v2 = vowels[1].to_ascii_lowercase();

            // iê, uô, ươ: tone on ê, ô, ơ (second)
            if matches!((v1, v2), ('i', 'ê') | ('u', 'ô') | ('ư', 'ơ')) {
                return 1;
            }

            // oa, oă, oe, ua, uâ, uê: tone on second
            if matches!(v1, 'o' | 'u') && matches!(v2, 'a' | 'ă' | 'â' | 'e' | 'ê') {
                return 1;
            }

            // ai, ao, au, ay, âu, ây, eo, êu, iu, oi, ôi, ơi, ui, ưi, ưu: tone on first
            0
        }
        _ => {
            // Triphthong: tone goes on middle vowel
            1
        }
    }
}

/// Find which vowel should receive modifier (circumflex, horn, breve)
/// Returns the index of the vowel to modify
pub fn find_modifier_position(vowels: &[char], modifier: char) -> Option<usize> {
    match modifier {
        'a' | 'e' | 'o' => {
            // Circumflex: find matching base vowel
            vowels.iter().position(|&v| {
                let lower = v.to_ascii_lowercase();
                lower == modifier
            })
        }
        'w' => {
            // Horn (ơ, ư) or breve (ă)
            // Find o -> ơ, u -> ư, a -> ă
            vowels.iter().position(|&v| {
                matches!(v.to_ascii_lowercase(), 'o' | 'u' | 'a')
            })
        }
        _ => None,
    }
}

/// Apply circumflex to vowel
pub fn apply_circumflex(c: char) -> char {
    match c {
        'a' => 'â', 'A' => 'Â',
        'e' => 'ê', 'E' => 'Ê',
        'o' => 'ô', 'O' => 'Ô',
        _ => c,
    }
}

/// Apply horn to vowel
pub fn apply_horn(c: char) -> char {
    match c {
        'o' => 'ơ', 'O' => 'Ơ',
        'u' => 'ư', 'U' => 'Ư',
        _ => c,
    }
}

/// Apply breve to vowel
pub fn apply_breve(c: char) -> char {
    match c {
        'a' => 'ă', 'A' => 'Ă',
        _ => c,
    }
}

/// Remove tone from vowel (return base vowel with mark preserved)
pub fn remove_tone(c: char) -> char {
    match c {
        'á'|'à'|'ả'|'ã'|'ạ' => 'a',
        'ắ'|'ằ'|'ẳ'|'ẵ'|'ặ' => 'ă',
        'ấ'|'ầ'|'ẩ'|'ẫ'|'ậ' => 'â',
        'é'|'è'|'ẻ'|'ẽ'|'ẹ' => 'e',
        'ế'|'ề'|'ể'|'ễ'|'ệ' => 'ê',
        'í'|'ì'|'ỉ'|'ĩ'|'ị' => 'i',
        'ó'|'ò'|'ỏ'|'õ'|'ọ' => 'o',
        'ố'|'ồ'|'ổ'|'ỗ'|'ộ' => 'ô',
        'ớ'|'ờ'|'ở'|'ỡ'|'ợ' => 'ơ',
        'ú'|'ù'|'ủ'|'ũ'|'ụ' => 'u',
        'ứ'|'ừ'|'ử'|'ữ'|'ự' => 'ư',
        'ý'|'ỳ'|'ỷ'|'ỹ'|'ỵ' => 'y',
        // Uppercase
        'Á'|'À'|'Ả'|'Ã'|'Ạ' => 'A',
        'Ắ'|'Ằ'|'Ẳ'|'Ẵ'|'Ặ' => 'Ă',
        'Ấ'|'Ầ'|'Ẩ'|'Ẫ'|'Ậ' => 'Â',
        'É'|'È'|'Ẻ'|'Ẽ'|'Ẹ' => 'E',
        'Ế'|'Ề'|'Ể'|'Ễ'|'Ệ' => 'Ê',
        'Í'|'Ì'|'Ỉ'|'Ĩ'|'Ị' => 'I',
        'Ó'|'Ò'|'Ỏ'|'Õ'|'Ọ' => 'O',
        'Ố'|'Ồ'|'Ổ'|'Ỗ'|'Ộ' => 'Ô',
        'Ớ'|'Ờ'|'Ở'|'Ỡ'|'Ợ' => 'Ơ',
        'Ú'|'Ù'|'Ủ'|'Ũ'|'Ụ' => 'U',
        'Ứ'|'Ừ'|'Ử'|'Ữ'|'Ự' => 'Ư',
        'Ý'|'Ỳ'|'Ỷ'|'Ỹ'|'Ỵ' => 'Y',
        _ => c,
    }
}

/// Remove circumflex from vowel (preserves tone)
pub fn remove_circumflex(c: char) -> char {
    match c {
        'â' => 'a', 'Â' => 'A',
        'ê' => 'e', 'Ê' => 'E',
        'ô' => 'o', 'Ô' => 'O',
        // Toned versions: â+tone → a+tone
        'ấ' => 'á', 'ầ' => 'à', 'ẩ' => 'ả', 'ẫ' => 'ã', 'ậ' => 'ạ',
        'ế' => 'é', 'ề' => 'è', 'ể' => 'ẻ', 'ễ' => 'ẽ', 'ệ' => 'ẹ',
        'ố' => 'ó', 'ồ' => 'ò', 'ổ' => 'ỏ', 'ỗ' => 'õ', 'ộ' => 'ọ',
        'Ấ' => 'Á', 'Ầ' => 'À', 'Ẩ' => 'Ả', 'Ẫ' => 'Ã', 'Ậ' => 'Ạ',
        'Ế' => 'É', 'Ề' => 'È', 'Ể' => 'Ẻ', 'Ễ' => 'Ẽ', 'Ệ' => 'Ẹ',
        'Ố' => 'Ó', 'Ồ' => 'Ò', 'Ổ' => 'Ỏ', 'Ỗ' => 'Õ', 'Ộ' => 'Ọ',
        _ => c,
    }
}

/// Remove horn from vowel (preserves tone)
pub fn remove_horn(c: char) -> char {
    match c {
        'ơ' => 'o', 'Ơ' => 'O',
        'ư' => 'u', 'Ư' => 'U',
        // Toned versions: ơ+tone → o+tone, ư+tone → u+tone
        'ớ' => 'ó', 'ờ' => 'ò', 'ở' => 'ỏ', 'ỡ' => 'õ', 'ợ' => 'ọ',
        'ứ' => 'ú', 'ừ' => 'ù', 'ử' => 'ủ', 'ữ' => 'ũ', 'ự' => 'ụ',
        'Ớ' => 'Ó', 'Ờ' => 'Ò', 'Ở' => 'Ỏ', 'Ỡ' => 'Õ', 'Ợ' => 'Ọ',
        'Ứ' => 'Ú', 'Ừ' => 'Ù', 'Ử' => 'Ủ', 'Ữ' => 'Ũ', 'Ự' => 'Ụ',
        _ => c,
    }
}

/// Remove breve from vowel (preserves tone)
pub fn remove_breve(c: char) -> char {
    match c {
        'ă' => 'a', 'Ă' => 'A',
        // Toned versions: ă+tone → a+tone
        'ắ' => 'á', 'ằ' => 'à', 'ẳ' => 'ả', 'ẵ' => 'ã', 'ặ' => 'ạ',
        'Ắ' => 'Á', 'Ằ' => 'À', 'Ẳ' => 'Ả', 'Ẵ' => 'Ã', 'Ặ' => 'Ạ',
        _ => c,
    }
}

/// Remove stroke from đ
pub fn remove_stroke(c: char) -> char {
    match c {
        'đ' => 'd', 'Đ' => 'D',
        _ => c,
    }
}

/// Apply stroke to d
pub fn apply_stroke(c: char) -> char {
    match c {
        'd' => 'đ', 'D' => 'Đ',
        _ => c,
    }
}

/// Tone value constants
pub mod tone {
    pub const NONE: u8 = 0;
    pub const SAC: u8 = 1;   // sắc (acute)
    pub const HUY: u8 = 2;   // huyền (grave)
    pub const HOI: u8 = 3;   // hỏi (hook)
    pub const NGA: u8 = 4;   // ngã (tilde)
    pub const NANG: u8 = 5;  // nặng (dot below)
}

/// Mark value constants
pub mod mark {
    pub const NONE: u8 = 0;
    pub const CIRCUM: u8 = 1;  // circumflex (â, ê, ô)
    pub const HORN: u8 = 2;    // horn (ơ, ư)
    pub const BREVE: u8 = 3;   // breve (ă)
}

/// Convert key to tone value (Telex)
pub fn key_to_tone_telex(key: char) -> u8 {
    match key.to_ascii_lowercase() {
        's' => tone::SAC,
        'f' => tone::HUY,
        'r' => tone::HOI,
        'x' => tone::NGA,
        'j' => tone::NANG,
        _ => tone::NONE,
    }
}

/// Convert key to tone value (VNI)
pub fn key_to_tone_vni(key: char) -> u8 {
    match key {
        '1' => tone::SAC,
        '2' => tone::HUY,
        '3' => tone::HOI,
        '4' => tone::NGA,
        '5' => tone::NANG,
        _ => tone::NONE,
    }
}

/// Check if character is a vowel
pub fn is_vowel(c: char) -> bool {
    matches!(c.to_ascii_lowercase(),
        'a' | 'ă' | 'â' | 'e' | 'ê' | 'i' | 'o' | 'ô' | 'ơ' | 'u' | 'ư' | 'y' |
        'á' | 'à' | 'ả' | 'ã' | 'ạ' |
        'ắ' | 'ằ' | 'ẳ' | 'ẵ' | 'ặ' |
        'ấ' | 'ầ' | 'ẩ' | 'ẫ' | 'ậ' |
        'é' | 'è' | 'ẻ' | 'ẽ' | 'ẹ' |
        'ế' | 'ề' | 'ể' | 'ễ' | 'ệ' |
        'í' | 'ì' | 'ỉ' | 'ĩ' | 'ị' |
        'ó' | 'ò' | 'ỏ' | 'õ' | 'ọ' |
        'ố' | 'ồ' | 'ổ' | 'ỗ' | 'ộ' |
        'ớ' | 'ờ' | 'ở' | 'ỡ' | 'ợ' |
        'ú' | 'ù' | 'ủ' | 'ũ' | 'ụ' |
        'ứ' | 'ừ' | 'ử' | 'ữ' | 'ự' |
        'ý' | 'ỳ' | 'ỷ' | 'ỹ' | 'ỵ'
    )
}

/// Apply tone to vowel
pub fn apply_tone(c: char, tone: u8) -> char {
    // Tone: 0=none, 1=sắc, 2=huyền, 3=hỏi, 4=ngã, 5=nặng
    match c {
        'a' => ['a', 'á', 'à', 'ả', 'ã', 'ạ'][tone as usize],
        'ă' => ['ă', 'ắ', 'ằ', 'ẳ', 'ẵ', 'ặ'][tone as usize],
        'â' => ['â', 'ấ', 'ầ', 'ẩ', 'ẫ', 'ậ'][tone as usize],
        'e' => ['e', 'é', 'è', 'ẻ', 'ẽ', 'ẹ'][tone as usize],
        'ê' => ['ê', 'ế', 'ề', 'ể', 'ễ', 'ệ'][tone as usize],
        'i' => ['i', 'í', 'ì', 'ỉ', 'ĩ', 'ị'][tone as usize],
        'o' => ['o', 'ó', 'ò', 'ỏ', 'õ', 'ọ'][tone as usize],
        'ô' => ['ô', 'ố', 'ồ', 'ổ', 'ỗ', 'ộ'][tone as usize],
        'ơ' => ['ơ', 'ớ', 'ờ', 'ở', 'ỡ', 'ợ'][tone as usize],
        'u' => ['u', 'ú', 'ù', 'ủ', 'ũ', 'ụ'][tone as usize],
        'ư' => ['ư', 'ứ', 'ừ', 'ử', 'ữ', 'ự'][tone as usize],
        'y' => ['y', 'ý', 'ỳ', 'ỷ', 'ỹ', 'ỵ'][tone as usize],
        // Uppercase
        'A' => ['A', 'Á', 'À', 'Ả', 'Ã', 'Ạ'][tone as usize],
        'Ă' => ['Ă', 'Ắ', 'Ằ', 'Ẳ', 'Ẵ', 'Ặ'][tone as usize],
        'Â' => ['Â', 'Ấ', 'Ầ', 'Ẩ', 'Ẫ', 'Ậ'][tone as usize],
        'E' => ['E', 'É', 'È', 'Ẻ', 'Ẽ', 'Ẹ'][tone as usize],
        'Ê' => ['Ê', 'Ế', 'Ề', 'Ể', 'Ễ', 'Ệ'][tone as usize],
        'I' => ['I', 'Í', 'Ì', 'Ỉ', 'Ĩ', 'Ị'][tone as usize],
        'O' => ['O', 'Ó', 'Ò', 'Ỏ', 'Õ', 'Ọ'][tone as usize],
        'Ô' => ['Ô', 'Ố', 'Ồ', 'Ổ', 'Ỗ', 'Ộ'][tone as usize],
        'Ơ' => ['Ơ', 'Ớ', 'Ờ', 'Ở', 'Ỡ', 'Ợ'][tone as usize],
        'U' => ['U', 'Ú', 'Ù', 'Ủ', 'Ũ', 'Ụ'][tone as usize],
        'Ư' => ['Ư', 'Ứ', 'Ừ', 'Ử', 'Ữ', 'Ự'][tone as usize],
        'Y' => ['Y', 'Ý', 'Ỳ', 'Ỷ', 'Ỹ', 'Ỵ'][tone as usize],
        _ => c,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tone_position_single() {
        assert_eq!(find_tone_position(&['a']), 0);
    }

    #[test]
    fn test_tone_position_diphthong() {
        assert_eq!(find_tone_position(&['a', 'i']), 0); // ai -> ái
        assert_eq!(find_tone_position(&['o', 'a']), 1); // oa -> oá
        assert_eq!(find_tone_position(&['i', 'ê']), 1); // iê -> iế
    }

    #[test]
    fn test_tone_position_triphthong() {
        assert_eq!(find_tone_position(&['u', 'y', 'ê']), 1); // uyê -> uyế
    }

    #[test]
    fn test_apply_circumflex() {
        assert_eq!(apply_circumflex('a'), 'â');
        assert_eq!(apply_circumflex('e'), 'ê');
        assert_eq!(apply_circumflex('o'), 'ô');
        assert_eq!(apply_circumflex('A'), 'Â');
    }

    #[test]
    fn test_apply_tone() {
        assert_eq!(apply_tone('a', 1), 'á'); // sắc
        assert_eq!(apply_tone('a', 2), 'à'); // huyền
        assert_eq!(apply_tone('ê', 1), 'ế'); // sắc on ê
    }
}
