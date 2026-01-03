//! Vietnamese validation matrices
//!
//! Implements 8 phonotactic validation rules:
//! 1. Valid initial consonants
//! 2. Valid initial + vowel combinations (spelling rules)
//! 3. Valid vowel pairs (diphthongs)
//! 4. Valid vowel + final combinations
//! 5. Valid tone + final combinations
//! 6. Valid consonant clusters
//! 7. Syllable structure constraints
//! 8. Tone placement rules
//!
//! Total: ~700 bytes

// ============================================================================
// INITIAL CONSONANT CONSTANTS
// ============================================================================

/// Initial consonant indices for matrix lookup
pub mod initial {
    pub const B: u8 = 0;
    pub const C: u8 = 1;
    pub const CH: u8 = 2;
    pub const D: u8 = 3;
    pub const G: u8 = 4;
    pub const GH: u8 = 5;
    pub const GI: u8 = 6;
    pub const H: u8 = 7;
    pub const K: u8 = 8;
    pub const KH: u8 = 9;
    pub const L: u8 = 10;
    pub const M: u8 = 11;
    pub const N: u8 = 12;
    pub const NG: u8 = 13;
    pub const NGH: u8 = 14;
    pub const NH: u8 = 15;
    pub const P: u8 = 16;
    pub const PH: u8 = 17;
    pub const Q: u8 = 18;
    pub const QU: u8 = 19;
    pub const R: u8 = 20;
    pub const S: u8 = 21;
    pub const T: u8 = 22;
    pub const TH: u8 = 23;
    pub const TR: u8 = 24;
    pub const V: u8 = 25;
    pub const X: u8 = 26;
    pub const DD: u8 = 27; // đ
    pub const NONE: u8 = 28; // vowel-initial syllables
    pub const COUNT: usize = 29;
    pub const INVALID: u8 = 255;
}

/// String representations for initial consonants (for parsing)
pub const INITIAL_STRINGS: [&str; 29] = [
    "b", "c", "ch", "d", "g", "gh", "gi", "h", "k", "kh", "l", "m", "n", "ng", "ngh", "nh", "p",
    "ph", "q", "qu", "r", "s", "t", "th", "tr", "v", "x", "đ", "",
];

/// Lookup initial type from string
#[inline]
pub fn lookup_initial_type(s: &str) -> u8 {
    let lower = s.to_lowercase();
    match lower.as_str() {
        "" => initial::NONE,
        "b" => initial::B,
        "c" => initial::C,
        "ch" => initial::CH,
        "d" => initial::D,
        "g" => initial::G,
        "gh" => initial::GH,
        "gi" => initial::GI,
        "h" => initial::H,
        "k" => initial::K,
        "kh" => initial::KH,
        "l" => initial::L,
        "m" => initial::M,
        "n" => initial::N,
        "ng" => initial::NG,
        "ngh" => initial::NGH,
        "nh" => initial::NH,
        "p" => initial::P,
        "ph" => initial::PH,
        "q" => initial::Q,
        "qu" => initial::QU,
        "r" => initial::R,
        "s" => initial::S,
        "t" => initial::T,
        "th" => initial::TH,
        "tr" => initial::TR,
        "v" => initial::V,
        "x" => initial::X,
        "đ" => initial::DD,
        _ => initial::INVALID,
    }
}

// ============================================================================
// VOWEL TYPE CONSTANTS
// ============================================================================

/// Vowel type indices for matrix lookup
pub mod vowel_type {
    pub const A: u8 = 0; // a, à, á, ả, ã, ạ
    pub const AW: u8 = 1; // ă, ằ, ắ, ẳ, ẵ, ặ
    pub const AA: u8 = 2; // â, ầ, ấ, ẩ, ẫ, ậ
    pub const E: u8 = 3; // e, è, é, ẻ, ẽ, ẹ
    pub const EE: u8 = 4; // ê, ề, ế, ể, ễ, ệ
    pub const I: u8 = 5; // i, ì, í, ỉ, ĩ, ị
    pub const O: u8 = 6; // o, ò, ó, ỏ, õ, ọ
    pub const OO: u8 = 7; // ô, ồ, ố, ổ, ỗ, ộ
    pub const OW: u8 = 8; // ơ, ờ, ớ, ở, ỡ, ợ
    pub const U: u8 = 9; // u, ù, ú, ủ, ũ, ụ
    pub const UW: u8 = 10; // ư, ừ, ứ, ử, ữ, ự
    pub const Y: u8 = 11; // y, ỳ, ý, ỷ, ỹ, ỵ
    pub const COUNT: usize = 12;
    pub const INVALID: u8 = 255;
}

/// Get vowel type from character
#[inline]
pub fn char_to_vowel_type(c: char) -> u8 {
    match c {
        'a' | 'A' | 'à' | 'á' | 'ả' | 'ã' | 'ạ' | 'À' | 'Á' | 'Ả' | 'Ã' | 'Ạ' => {
            vowel_type::A
        }
        'ă' | 'Ă' | 'ằ' | 'ắ' | 'ẳ' | 'ẵ' | 'ặ' | 'Ằ' | 'Ắ' | 'Ẳ' | 'Ẵ' | 'Ặ' => {
            vowel_type::AW
        }
        'â' | 'Â' | 'ầ' | 'ấ' | 'ẩ' | 'ẫ' | 'ậ' | 'Ầ' | 'Ấ' | 'Ẩ' | 'Ẫ' | 'Ậ' => {
            vowel_type::AA
        }
        'e' | 'E' | 'è' | 'é' | 'ẻ' | 'ẽ' | 'ẹ' | 'È' | 'É' | 'Ẻ' | 'Ẽ' | 'Ẹ' => {
            vowel_type::E
        }
        'ê' | 'Ê' | 'ề' | 'ế' | 'ể' | 'ễ' | 'ệ' | 'Ề' | 'Ế' | 'Ể' | 'Ễ' | 'Ệ' => {
            vowel_type::EE
        }
        'i' | 'I' | 'ì' | 'í' | 'ỉ' | 'ĩ' | 'ị' | 'Ì' | 'Í' | 'Ỉ' | 'Ĩ' | 'Ị' => {
            vowel_type::I
        }
        'o' | 'O' | 'ò' | 'ó' | 'ỏ' | 'õ' | 'ọ' | 'Ò' | 'Ó' | 'Ỏ' | 'Õ' | 'Ọ' => {
            vowel_type::O
        }
        'ô' | 'Ô' | 'ồ' | 'ố' | 'ổ' | 'ỗ' | 'ộ' | 'Ồ' | 'Ố' | 'Ổ' | 'Ỗ' | 'Ộ' => {
            vowel_type::OO
        }
        'ơ' | 'Ơ' | 'ờ' | 'ớ' | 'ở' | 'ỡ' | 'ợ' | 'Ờ' | 'Ớ' | 'Ở' | 'Ỡ' | 'Ợ' => {
            vowel_type::OW
        }
        'u' | 'U' | 'ù' | 'ú' | 'ủ' | 'ũ' | 'ụ' | 'Ù' | 'Ú' | 'Ủ' | 'Ũ' | 'Ụ' => {
            vowel_type::U
        }
        'ư' | 'Ư' | 'ừ' | 'ứ' | 'ử' | 'ữ' | 'ự' | 'Ừ' | 'Ứ' | 'Ử' | 'Ữ' | 'Ự' => {
            vowel_type::UW
        }
        'y' | 'Y' | 'ỳ' | 'ý' | 'ỷ' | 'ỹ' | 'ỵ' | 'Ỳ' | 'Ý' | 'Ỷ' | 'Ỹ' | 'Ỵ' => {
            vowel_type::Y
        }
        _ => vowel_type::INVALID,
    }
}

/// Check if character is Vietnamese vowel (including toned)
#[inline]
pub fn is_vn_vowel(c: char) -> bool {
    char_to_vowel_type(c) != vowel_type::INVALID
}

/// Check if character is vowel with tone mark
#[inline]
pub fn is_toned_vowel(c: char) -> bool {
    matches!(
        c,
        'á' | 'à'
            | 'ả'
            | 'ã'
            | 'ạ'
            | 'ắ'
            | 'ằ'
            | 'ẳ'
            | 'ẵ'
            | 'ặ'
            | 'ấ'
            | 'ầ'
            | 'ẩ'
            | 'ẫ'
            | 'ậ'
            | 'é'
            | 'è'
            | 'ẻ'
            | 'ẽ'
            | 'ẹ'
            | 'ế'
            | 'ề'
            | 'ể'
            | 'ễ'
            | 'ệ'
            | 'í'
            | 'ì'
            | 'ỉ'
            | 'ĩ'
            | 'ị'
            | 'ó'
            | 'ò'
            | 'ỏ'
            | 'õ'
            | 'ọ'
            | 'ố'
            | 'ồ'
            | 'ổ'
            | 'ỗ'
            | 'ộ'
            | 'ớ'
            | 'ờ'
            | 'ở'
            | 'ỡ'
            | 'ợ'
            | 'ú'
            | 'ù'
            | 'ủ'
            | 'ũ'
            | 'ụ'
            | 'ứ'
            | 'ừ'
            | 'ử'
            | 'ữ'
            | 'ự'
            | 'ý'
            | 'ỳ'
            | 'ỷ'
            | 'ỹ'
            | 'ỵ'
    )
}

// ============================================================================
// FINAL CONSONANT CONSTANTS
// ============================================================================

/// Final consonant type indices
pub mod final_type {
    pub const NONE: u8 = 0;
    pub const M: u8 = 1;
    pub const N: u8 = 2;
    pub const NG: u8 = 3;
    pub const NH: u8 = 4;
    pub const P: u8 = 5;
    pub const T: u8 = 6;
    pub const C: u8 = 7; // includes k
    pub const CH: u8 = 8;
    pub const COUNT: usize = 9;
    pub const INVALID: u8 = 255;
}

/// Final consonant category (for tone restriction)
pub mod final_cat {
    pub const OPEN: u8 = 0; // no final
    pub const NASAL: u8 = 1; // m, n, ng, nh
    pub const SEMI: u8 = 2; // i, y, o, u (semivowel finals)
    pub const STOP: u8 = 3; // p, t, c, ch
    pub const COUNT: usize = 4;
}

/// Lookup final type from string
#[inline]
pub fn lookup_final_type(s: &str) -> u8 {
    let lower = s.to_lowercase();
    match lower.as_str() {
        "" => final_type::NONE,
        "m" => final_type::M,
        "n" => final_type::N,
        "ng" => final_type::NG,
        "nh" => final_type::NH,
        "p" => final_type::P,
        "t" => final_type::T,
        "c" | "k" => final_type::C,
        "ch" => final_type::CH,
        _ => final_type::INVALID,
    }
}

/// Get final category from final type
#[inline]
pub fn final_type_to_category(ft: u8) -> u8 {
    match ft {
        final_type::NONE => final_cat::OPEN,
        final_type::M | final_type::N | final_type::NG | final_type::NH => final_cat::NASAL,
        final_type::P | final_type::T | final_type::C | final_type::CH => final_cat::STOP,
        _ => final_cat::OPEN,
    }
}

// ============================================================================
// TONE CONSTANTS
// ============================================================================

/// Tone values
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Tone {
    None = 0,  // ngang (no mark)
    Sac = 1,   // sắc (acute)
    Huyen = 2, // huyền (grave)
    Hoi = 3,   // hỏi (hook)
    Nga = 4,   // ngã (tilde)
    Nang = 5,  // nặng (dot)
}

pub mod tone {
    pub const NONE: u8 = 0;
    pub const SAC: u8 = 1;
    pub const HUY: u8 = 2;
    pub const HOI: u8 = 3;
    pub const NGA: u8 = 4;
    pub const NANG: u8 = 5;
    pub const COUNT: usize = 6;
}

/// Get tone from character
#[inline]
pub fn char_to_tone(c: char) -> u8 {
    match c {
        'á' | 'ắ' | 'ấ' | 'é' | 'ế' | 'í' | 'ó' | 'ố' | 'ớ' | 'ú' | 'ứ' | 'ý' => {
            tone::SAC
        }
        'à' | 'ằ' | 'ầ' | 'è' | 'ề' | 'ì' | 'ò' | 'ồ' | 'ờ' | 'ù' | 'ừ' | 'ỳ' => {
            tone::HUY
        }
        'ả' | 'ẳ' | 'ẩ' | 'ẻ' | 'ể' | 'ỉ' | 'ỏ' | 'ổ' | 'ở' | 'ủ' | 'ử' | 'ỷ' => {
            tone::HOI
        }
        'ã' | 'ẵ' | 'ẫ' | 'ẽ' | 'ễ' | 'ĩ' | 'õ' | 'ỗ' | 'ỡ' | 'ũ' | 'ữ' | 'ỹ' => {
            tone::NGA
        }
        'ạ' | 'ặ' | 'ậ' | 'ẹ' | 'ệ' | 'ị' | 'ọ' | 'ộ' | 'ợ' | 'ụ' | 'ự' | 'ỵ' => {
            tone::NANG
        }
        _ => tone::NONE,
    }
}

// ============================================================================
// M_INITIAL_VALID (29 bytes)
// ============================================================================

/// Bitmap: 1 = valid initial consonant
pub static M_INITIAL_VALID: [u8; initial::COUNT] = [1; initial::COUNT];

/// Check if initial type is valid
#[inline]
pub fn is_valid_initial(init_type: u8) -> bool {
    (init_type as usize) < initial::COUNT && M_INITIAL_VALID[init_type as usize] != 0
}

// ============================================================================
// M_INITIAL_VOWEL (348 bytes) - Spelling Rules Matrix
// ============================================================================

/// Spelling rules: M_INITIAL_VOWEL[initial][vowel_type] = valid(1) or invalid(0)
/// 29 initials x 12 vowel types = 348 entries
///
/// Rules:
/// - c + e,ê,i,y → INVALID (use k)
/// - k + a,ă,â,o,ô,ơ,u,ư → INVALID (use c)
/// - g + e,ê → INVALID (use gh)
/// - gh + NOT(e,ê) → INVALID (gh only valid before e,ê)
/// - ng + e,ê,i,y → INVALID (use ngh)
/// - ngh + NOT(e,ê,i) → INVALID (ngh only valid before e,ê,i)
pub static M_INITIAL_VOWEL: [[u8; vowel_type::COUNT]; initial::COUNT] = {
    let mut m = [[1u8; vowel_type::COUNT]; initial::COUNT];

    // c + e,ê,i,y invalid (use k instead)
    m[initial::C as usize][vowel_type::E as usize] = 0;
    m[initial::C as usize][vowel_type::EE as usize] = 0;
    m[initial::C as usize][vowel_type::I as usize] = 0;
    m[initial::C as usize][vowel_type::Y as usize] = 0;

    // k + a,ă,â,o,ô,ơ,u,ư invalid (use c instead)
    m[initial::K as usize][vowel_type::A as usize] = 0;
    m[initial::K as usize][vowel_type::AW as usize] = 0;
    m[initial::K as usize][vowel_type::AA as usize] = 0;
    m[initial::K as usize][vowel_type::O as usize] = 0;
    m[initial::K as usize][vowel_type::OO as usize] = 0;
    m[initial::K as usize][vowel_type::OW as usize] = 0;
    m[initial::K as usize][vowel_type::U as usize] = 0;
    m[initial::K as usize][vowel_type::UW as usize] = 0;

    // g + e,ê invalid (use gh instead)
    m[initial::G as usize][vowel_type::E as usize] = 0;
    m[initial::G as usize][vowel_type::EE as usize] = 0;

    // gh + NOT(e,ê) invalid (gh only valid before e,ê)
    m[initial::GH as usize][vowel_type::A as usize] = 0;
    m[initial::GH as usize][vowel_type::AW as usize] = 0;
    m[initial::GH as usize][vowel_type::AA as usize] = 0;
    m[initial::GH as usize][vowel_type::I as usize] = 0;
    m[initial::GH as usize][vowel_type::O as usize] = 0;
    m[initial::GH as usize][vowel_type::OO as usize] = 0;
    m[initial::GH as usize][vowel_type::OW as usize] = 0;
    m[initial::GH as usize][vowel_type::U as usize] = 0;
    m[initial::GH as usize][vowel_type::UW as usize] = 0;
    m[initial::GH as usize][vowel_type::Y as usize] = 0;

    // ng + e,ê,i,y invalid (use ngh instead)
    m[initial::NG as usize][vowel_type::E as usize] = 0;
    m[initial::NG as usize][vowel_type::EE as usize] = 0;
    m[initial::NG as usize][vowel_type::I as usize] = 0;
    m[initial::NG as usize][vowel_type::Y as usize] = 0;

    // ngh + NOT(e,ê,i) invalid (ngh only valid before e,ê,i)
    m[initial::NGH as usize][vowel_type::A as usize] = 0;
    m[initial::NGH as usize][vowel_type::AW as usize] = 0;
    m[initial::NGH as usize][vowel_type::AA as usize] = 0;
    m[initial::NGH as usize][vowel_type::O as usize] = 0;
    m[initial::NGH as usize][vowel_type::OO as usize] = 0;
    m[initial::NGH as usize][vowel_type::OW as usize] = 0;
    m[initial::NGH as usize][vowel_type::U as usize] = 0;
    m[initial::NGH as usize][vowel_type::UW as usize] = 0;
    m[initial::NGH as usize][vowel_type::Y as usize] = 0;

    m
};

/// Check spelling rule validity
#[inline]
pub fn is_valid_spelling(init_type: u8, vowel_t: u8) -> bool {
    if init_type as usize >= initial::COUNT || vowel_t as usize >= vowel_type::COUNT {
        return false;
    }
    M_INITIAL_VOWEL[init_type as usize][vowel_t as usize] != 0
}

// ============================================================================
// M_VOWEL_PAIR (144 bytes) - Valid Diphthong/Triphthong Matrix
// ============================================================================

/// Valid vowel pair combinations
/// M_VOWEL_PAIR[v1][v2] = valid(1) or invalid(0)
/// 12 x 12 = 144 entries
///
/// Inclusion approach: only list valid Vietnamese diphthongs
pub static M_VOWEL_PAIR: [[u8; vowel_type::COUNT]; vowel_type::COUNT] = {
    let mut m = [[0u8; vowel_type::COUNT]; vowel_type::COUNT];

    // Standard Vietnamese diphthongs
    m[vowel_type::A as usize][vowel_type::I as usize] = 1; // ai
    m[vowel_type::A as usize][vowel_type::O as usize] = 1; // ao
    m[vowel_type::A as usize][vowel_type::U as usize] = 1; // au
    m[vowel_type::A as usize][vowel_type::Y as usize] = 1; // ay

    m[vowel_type::AW as usize][vowel_type::U as usize] = 1; // ău (rare but valid)
    m[vowel_type::AW as usize][vowel_type::Y as usize] = 1; // ăy (rare but valid)

    m[vowel_type::AA as usize][vowel_type::U as usize] = 1; // âu
    m[vowel_type::AA as usize][vowel_type::Y as usize] = 1; // ây

    m[vowel_type::E as usize][vowel_type::O as usize] = 1; // eo

    m[vowel_type::EE as usize][vowel_type::U as usize] = 1; // êu

    m[vowel_type::I as usize][vowel_type::A as usize] = 1; // ia
    m[vowel_type::I as usize][vowel_type::EE as usize] = 1; // iê (in iên, iếu)
    m[vowel_type::I as usize][vowel_type::U as usize] = 1; // iu

    m[vowel_type::O as usize][vowel_type::A as usize] = 1; // oa
    m[vowel_type::O as usize][vowel_type::AW as usize] = 1; // oă
    m[vowel_type::O as usize][vowel_type::E as usize] = 1; // oe
    m[vowel_type::O as usize][vowel_type::I as usize] = 1; // oi

    m[vowel_type::OO as usize][vowel_type::I as usize] = 1; // ôi

    m[vowel_type::OW as usize][vowel_type::I as usize] = 1; // ơi

    m[vowel_type::U as usize][vowel_type::A as usize] = 1; // ua
    m[vowel_type::U as usize][vowel_type::AA as usize] = 1; // uâ
    m[vowel_type::U as usize][vowel_type::EE as usize] = 1; // uê
    m[vowel_type::U as usize][vowel_type::I as usize] = 1; // ui
    m[vowel_type::U as usize][vowel_type::O as usize] = 1; // uo (intermediate for uô)
    m[vowel_type::U as usize][vowel_type::OO as usize] = 1; // uô
    m[vowel_type::U as usize][vowel_type::OW as usize] = 1; // uơ
    m[vowel_type::U as usize][vowel_type::Y as usize] = 1; // uy

    m[vowel_type::UW as usize][vowel_type::A as usize] = 1; // ưa
    m[vowel_type::UW as usize][vowel_type::I as usize] = 1; // ưi
    m[vowel_type::UW as usize][vowel_type::O as usize] = 1; // ươ (intermediate)
    m[vowel_type::UW as usize][vowel_type::OW as usize] = 1; // ươ
    m[vowel_type::UW as usize][vowel_type::U as usize] = 1; // ưu

    m[vowel_type::Y as usize][vowel_type::EE as usize] = 1; // yê (in yên, yếu)

    // Telex intermediate states (toggle patterns)
    m[vowel_type::A as usize][vowel_type::A as usize] = 1; // aa → â
    m[vowel_type::E as usize][vowel_type::E as usize] = 1; // ee → ê
    m[vowel_type::O as usize][vowel_type::O as usize] = 1; // oo → ô

    // Telex intermediate for ê toggle
    m[vowel_type::E as usize][vowel_type::I as usize] = 1; // ei (intermediate)

    m
};

/// Check if vowel pair is valid
#[inline]
pub fn is_valid_vowel_pair(v1: u8, v2: u8) -> bool {
    if v1 as usize >= vowel_type::COUNT || v2 as usize >= vowel_type::COUNT {
        return false;
    }
    M_VOWEL_PAIR[v1 as usize][v2 as usize] != 0
}

// ============================================================================
// M_VOWEL_FINAL (108 bytes) - Vowel + Final Compatibility
// ============================================================================

/// Vowel + Final compatibility matrix
/// M_VOWEL_FINAL[vowel_type][final_type] = valid(1) or invalid(0)
/// 12 x 9 = 108 entries
///
/// Most combinations are valid. Specific restrictions handled in validation logic.
// Note: Complex restrictions like "ưu + any final = INVALID"
// are handled in the validation function because ưu is a digraph
pub static M_VOWEL_FINAL: [[u8; final_type::COUNT]; vowel_type::COUNT] =
    [[1u8; final_type::COUNT]; vowel_type::COUNT];

/// Check vowel-final compatibility
#[inline]
pub fn is_valid_vowel_final(vowel_t: u8, final_t: u8) -> bool {
    if vowel_t as usize >= vowel_type::COUNT || final_t as usize >= final_type::COUNT {
        return false;
    }
    M_VOWEL_FINAL[vowel_t as usize][final_t as usize] != 0
}

// ============================================================================
// M_TONE_FINAL (24 bytes) - Tone + Final Restriction
// ============================================================================

/// Tone + Final category restriction matrix
/// M_TONE_FINAL[tone][final_cat] = valid(1) or invalid(0)
/// 6 x 4 = 24 entries
///
/// Stop finals (p,t,c,ch) ONLY allow ngang, sắc or nặng
/// (huyền, hỏi, ngã are invalid with stop finals)
pub static M_TONE_FINAL: [[u8; final_cat::COUNT]; tone::COUNT] = [
    //         OPEN  NASAL SEMI  STOP
    /* NONE */
    [1, 1, 1, 1], // ngang OK everywhere (including stops)
    /* SAC  */ [1, 1, 1, 1], // sắc OK everywhere
    /* HUY  */ [1, 1, 1, 0], // huyền not with stops
    /* HOI  */ [1, 1, 1, 0], // hỏi not with stops
    /* NGA  */ [1, 1, 1, 0], // ngã not with stops
    /* NANG */ [1, 1, 1, 1], // nặng OK everywhere
];

/// Check tone-final compatibility
#[inline]
pub fn is_valid_tone_final(t: u8, fc: u8) -> bool {
    if t as usize >= tone::COUNT || fc as usize >= final_cat::COUNT {
        return false;
    }
    M_TONE_FINAL[t as usize][fc as usize] != 0
}

/// Check if tone is valid with stop final (legacy function)
#[inline]
pub fn is_valid_tone_with_stop(t: Tone) -> bool {
    matches!(t, Tone::None | Tone::Sac | Tone::Nang)
}

// ============================================================================
// VALID VOWEL COMBINATIONS (for reference/fallback)
// ============================================================================

/// Valid vowel combinations (diphthongs and triphthongs) - string version
pub const VALID_VOWEL_COMBOS: [&str; 33] = [
    // Diphthongs
    "ai", "ao", "au", "ay", "âu", "ây", "eo", "êu", "ia", "iê", "iu", "oa", "oă", "oe", "oi", "ôi",
    "ơi", "ua", "uâ", "uê", "ui", "uo", "uô", "uơ", "uy", "ưa", "ưi", "ươ", "ưu", "yê",
    // Telex intermediates
    "aa", "ee", "oo",
];

/// Vietnamese vowels (base forms)
pub const VOWELS: [char; 12] = ['a', 'ă', 'â', 'e', 'ê', 'i', 'o', 'ô', 'ơ', 'u', 'ư', 'y'];

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Initial lookup tests
    #[test]
    fn test_lookup_initial_type() {
        assert_eq!(lookup_initial_type("b"), initial::B);
        assert_eq!(lookup_initial_type("ch"), initial::CH);
        assert_eq!(lookup_initial_type("ngh"), initial::NGH);
        assert_eq!(lookup_initial_type("qu"), initial::QU);
        assert_eq!(lookup_initial_type("đ"), initial::DD);
        assert_eq!(lookup_initial_type(""), initial::NONE);
        assert_eq!(lookup_initial_type("xyz"), initial::INVALID);
    }

    // Vowel type tests
    #[test]
    fn test_char_to_vowel_type() {
        assert_eq!(char_to_vowel_type('a'), vowel_type::A);
        assert_eq!(char_to_vowel_type('á'), vowel_type::A);
        assert_eq!(char_to_vowel_type('ă'), vowel_type::AW);
        assert_eq!(char_to_vowel_type('ắ'), vowel_type::AW);
        assert_eq!(char_to_vowel_type('â'), vowel_type::AA);
        assert_eq!(char_to_vowel_type('ư'), vowel_type::UW);
        assert_eq!(char_to_vowel_type('ứ'), vowel_type::UW);
        assert_eq!(char_to_vowel_type('b'), vowel_type::INVALID);
    }

    #[test]
    fn test_is_vn_vowel() {
        assert!(is_vn_vowel('a'));
        assert!(is_vn_vowel('á'));
        assert!(is_vn_vowel('ă'));
        assert!(is_vn_vowel('â'));
        assert!(is_vn_vowel('ư'));
        assert!(!is_vn_vowel('b'));
        assert!(!is_vn_vowel('c'));
    }

    // Spelling rules tests
    #[test]
    fn test_spelling_rules_c_k() {
        // c before a,o,u is valid
        assert!(is_valid_spelling(initial::C, vowel_type::A));
        assert!(is_valid_spelling(initial::C, vowel_type::O));
        assert!(is_valid_spelling(initial::C, vowel_type::U));

        // c before e,i,y is invalid
        assert!(!is_valid_spelling(initial::C, vowel_type::E));
        assert!(!is_valid_spelling(initial::C, vowel_type::EE));
        assert!(!is_valid_spelling(initial::C, vowel_type::I));
        assert!(!is_valid_spelling(initial::C, vowel_type::Y));

        // k before e,i,y is valid
        assert!(is_valid_spelling(initial::K, vowel_type::E));
        assert!(is_valid_spelling(initial::K, vowel_type::EE));
        assert!(is_valid_spelling(initial::K, vowel_type::I));
        assert!(is_valid_spelling(initial::K, vowel_type::Y));

        // k before a,o,u is invalid
        assert!(!is_valid_spelling(initial::K, vowel_type::A));
        assert!(!is_valid_spelling(initial::K, vowel_type::O));
        assert!(!is_valid_spelling(initial::K, vowel_type::U));
    }

    #[test]
    fn test_spelling_rules_g_gh() {
        // g before a,o,u is valid
        assert!(is_valid_spelling(initial::G, vowel_type::A));
        assert!(is_valid_spelling(initial::G, vowel_type::O));
        assert!(is_valid_spelling(initial::G, vowel_type::U));

        // g before e,ê is invalid
        assert!(!is_valid_spelling(initial::G, vowel_type::E));
        assert!(!is_valid_spelling(initial::G, vowel_type::EE));

        // gh before e,ê is valid
        assert!(is_valid_spelling(initial::GH, vowel_type::E));
        assert!(is_valid_spelling(initial::GH, vowel_type::EE));

        // gh before a,o,u is invalid
        assert!(!is_valid_spelling(initial::GH, vowel_type::A));
        assert!(!is_valid_spelling(initial::GH, vowel_type::O));
        assert!(!is_valid_spelling(initial::GH, vowel_type::U));
    }

    #[test]
    fn test_spelling_rules_ng_ngh() {
        // ng before a,o,u is valid
        assert!(is_valid_spelling(initial::NG, vowel_type::A));
        assert!(is_valid_spelling(initial::NG, vowel_type::O));
        assert!(is_valid_spelling(initial::NG, vowel_type::U));

        // ng before e,i,y is invalid
        assert!(!is_valid_spelling(initial::NG, vowel_type::E));
        assert!(!is_valid_spelling(initial::NG, vowel_type::EE));
        assert!(!is_valid_spelling(initial::NG, vowel_type::I));
        assert!(!is_valid_spelling(initial::NG, vowel_type::Y));

        // ngh before e,ê,i is valid
        assert!(is_valid_spelling(initial::NGH, vowel_type::E));
        assert!(is_valid_spelling(initial::NGH, vowel_type::EE));
        assert!(is_valid_spelling(initial::NGH, vowel_type::I));

        // ngh before a,o,u is invalid
        assert!(!is_valid_spelling(initial::NGH, vowel_type::A));
        assert!(!is_valid_spelling(initial::NGH, vowel_type::O));
        assert!(!is_valid_spelling(initial::NGH, vowel_type::U));
    }

    // Vowel pair tests
    #[test]
    fn test_valid_vowel_pairs() {
        // Standard diphthongs
        assert!(is_valid_vowel_pair(vowel_type::A, vowel_type::I)); // ai
        assert!(is_valid_vowel_pair(vowel_type::A, vowel_type::O)); // ao
        assert!(is_valid_vowel_pair(vowel_type::A, vowel_type::U)); // au
        assert!(is_valid_vowel_pair(vowel_type::EE, vowel_type::U)); // êu
        assert!(is_valid_vowel_pair(vowel_type::I, vowel_type::A)); // ia
        assert!(is_valid_vowel_pair(vowel_type::U, vowel_type::A)); // ua
        assert!(is_valid_vowel_pair(vowel_type::UW, vowel_type::A)); // ưa
    }

    #[test]
    fn test_invalid_vowel_pairs() {
        // These are NOT valid Vietnamese diphthongs
        assert!(!is_valid_vowel_pair(vowel_type::E, vowel_type::A)); // ea
        assert!(!is_valid_vowel_pair(vowel_type::O, vowel_type::U)); // ou
        assert!(!is_valid_vowel_pair(vowel_type::Y, vowel_type::O)); // yo
    }

    #[test]
    fn test_telex_intermediate_vowel_pairs() {
        // Telex toggle patterns
        assert!(is_valid_vowel_pair(vowel_type::A, vowel_type::A)); // aa → â
        assert!(is_valid_vowel_pair(vowel_type::E, vowel_type::E)); // ee → ê
        assert!(is_valid_vowel_pair(vowel_type::O, vowel_type::O)); // oo → ô
    }

    // Tone-final tests
    #[test]
    fn test_tone_final_stop_restriction() {
        // Stop finals allow ngang, sắc, or nặng
        assert!(is_valid_tone_final(tone::NONE, final_cat::STOP)); // ngang OK with stops
        assert!(is_valid_tone_final(tone::SAC, final_cat::STOP)); // sắc OK
        assert!(!is_valid_tone_final(tone::HUY, final_cat::STOP)); // huyền NOT with stops
        assert!(!is_valid_tone_final(tone::HOI, final_cat::STOP)); // hỏi NOT with stops
        assert!(!is_valid_tone_final(tone::NGA, final_cat::STOP)); // ngã NOT with stops
        assert!(is_valid_tone_final(tone::NANG, final_cat::STOP)); // nặng OK
    }

    #[test]
    fn test_tone_final_non_stop() {
        // Non-stop finals allow all tones
        for t in 0..tone::COUNT as u8 {
            assert!(is_valid_tone_final(t, final_cat::OPEN));
            assert!(is_valid_tone_final(t, final_cat::NASAL));
            assert!(is_valid_tone_final(t, final_cat::SEMI));
        }
    }

    #[test]
    fn test_is_valid_tone_with_stop_legacy() {
        assert!(is_valid_tone_with_stop(Tone::None));
        assert!(is_valid_tone_with_stop(Tone::Sac));
        assert!(is_valid_tone_with_stop(Tone::Nang));
        assert!(!is_valid_tone_with_stop(Tone::Huyen));
        assert!(!is_valid_tone_with_stop(Tone::Hoi));
        assert!(!is_valid_tone_with_stop(Tone::Nga));
    }

    // Final type tests
    #[test]
    fn test_lookup_final_type() {
        assert_eq!(lookup_final_type(""), final_type::NONE);
        assert_eq!(lookup_final_type("m"), final_type::M);
        assert_eq!(lookup_final_type("n"), final_type::N);
        assert_eq!(lookup_final_type("ng"), final_type::NG);
        assert_eq!(lookup_final_type("nh"), final_type::NH);
        assert_eq!(lookup_final_type("p"), final_type::P);
        assert_eq!(lookup_final_type("t"), final_type::T);
        assert_eq!(lookup_final_type("c"), final_type::C);
        assert_eq!(lookup_final_type("ch"), final_type::CH);
        assert_eq!(lookup_final_type("xyz"), final_type::INVALID);
    }

    #[test]
    fn test_final_type_to_category() {
        assert_eq!(final_type_to_category(final_type::NONE), final_cat::OPEN);
        assert_eq!(final_type_to_category(final_type::M), final_cat::NASAL);
        assert_eq!(final_type_to_category(final_type::N), final_cat::NASAL);
        assert_eq!(final_type_to_category(final_type::NG), final_cat::NASAL);
        assert_eq!(final_type_to_category(final_type::P), final_cat::STOP);
        assert_eq!(final_type_to_category(final_type::T), final_cat::STOP);
        assert_eq!(final_type_to_category(final_type::C), final_cat::STOP);
        assert_eq!(final_type_to_category(final_type::CH), final_cat::STOP);
    }

    // Tone from character tests
    #[test]
    fn test_char_to_tone() {
        assert_eq!(char_to_tone('a'), tone::NONE);
        assert_eq!(char_to_tone('á'), tone::SAC);
        assert_eq!(char_to_tone('à'), tone::HUY);
        assert_eq!(char_to_tone('ả'), tone::HOI);
        assert_eq!(char_to_tone('ã'), tone::NGA);
        assert_eq!(char_to_tone('ạ'), tone::NANG);
    }
}
