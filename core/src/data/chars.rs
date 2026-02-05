//! Vietnamese Unicode Character System
//!
//! Provides bidirectional conversion between:
//! - Forward: base vowels + modifiers + marks → Vietnamese Unicode
//! - Reverse: Vietnamese Unicode → components (for buffer restoration)
//!
//! ## Design Principles
//! - Single lookup table for all vowel combinations (12 bases × 6 marks = 72)
//! - O(1) reverse lookup via match (compiler-optimized jump table)
//! - Uses Rust's built-in `to_uppercase()` for case conversion
//!
//! ## Character Components
//! - Base vowel: a, ă, â, e, ê, i, o, ô, ơ, u, ư, y
//! - Mark (dấu thanh): none, sắc, huyền, hỏi, ngã, nặng
//! - Case: lowercase, uppercase

use super::keys;

/// Tone modifiers (dấu phụ) - changes base vowel form
pub mod tone {
    pub const NONE: u8 = 0;
    pub const CIRCUMFLEX: u8 = 1; // ^ (mũ): a→â, e→ê, o→ô
    pub const HORN: u8 = 2; // ơ, ư or breve ă
}

/// Marks (dấu thanh) - Vietnamese tone marks
pub mod mark {
    pub const NONE: u8 = 0;
    pub const SAC: u8 = 1; // sắc (á)
    pub const HUYEN: u8 = 2; // huyền (à)
    pub const HOI: u8 = 3; // hỏi (ả)
    pub const NGA: u8 = 4; // ngã (ã)
    pub const NANG: u8 = 5; // nặng (ạ)
}

/// Vietnamese vowel lookup table
/// Each entry: (base_char, [sắc, huyền, hỏi, ngã, nặng])
const VOWEL_TABLE: [(char, [char; 5]); 12] = [
    ('a', ['á', 'à', 'ả', 'ã', 'ạ']),
    ('ă', ['ắ', 'ằ', 'ẳ', 'ẵ', 'ặ']),
    ('â', ['ấ', 'ầ', 'ẩ', 'ẫ', 'ậ']),
    ('e', ['é', 'è', 'ẻ', 'ẽ', 'ẹ']),
    ('ê', ['ế', 'ề', 'ể', 'ễ', 'ệ']),
    ('i', ['í', 'ì', 'ỉ', 'ĩ', 'ị']),
    ('o', ['ó', 'ò', 'ỏ', 'õ', 'ọ']),
    ('ô', ['ố', 'ồ', 'ổ', 'ỗ', 'ộ']),
    ('ơ', ['ớ', 'ờ', 'ở', 'ỡ', 'ợ']),
    ('u', ['ú', 'ù', 'ủ', 'ũ', 'ụ']),
    ('ư', ['ứ', 'ừ', 'ử', 'ữ', 'ự']),
    ('y', ['ý', 'ỳ', 'ỷ', 'ỹ', 'ỵ']),
];

/// Get base character from key + tone modifier
///
/// # Arguments
/// * `key` - Virtual keycode (a, e, i, o, u, y)
/// * `tone` - Tone modifier: 0=none, 1=circumflex(^), 2=horn/breve
///
/// # Returns
/// Base vowel character: a, ă, â, e, ê, i, o, ô, ơ, u, ư, y
fn get_base_char(key: u16, t: u8) -> Option<char> {
    match key {
        keys::A => Some(match t {
            tone::CIRCUMFLEX => 'â',
            tone::HORN => 'ă', // breve for 'a'
            _ => 'a',
        }),
        keys::E => Some(match t {
            tone::CIRCUMFLEX => 'ê',
            _ => 'e',
        }),
        keys::I => Some('i'),
        keys::O => Some(match t {
            tone::CIRCUMFLEX => 'ô',
            tone::HORN => 'ơ',
            _ => 'o',
        }),
        keys::U => Some(match t {
            tone::HORN => 'ư',
            _ => 'u',
        }),
        keys::Y => Some('y'),
        _ => None,
    }
}

/// Apply mark to base vowel character
///
/// Uses lookup table to find the marked variant.
///
/// # Arguments
/// * `base` - Base vowel character (a, ă, â, e, ê, i, o, ô, ơ, u, ư, y)
/// * `mark` - Mark: 0=none, 1=sắc, 2=huyền, 3=hỏi, 4=ngã, 5=nặng
fn apply_mark(base: char, m: u8) -> char {
    if m == mark::NONE || m > mark::NANG {
        return base;
    }

    VOWEL_TABLE
        .iter()
        .find(|(b, _)| *b == base)
        .map(|(_, marks)| marks[(m - 1) as usize])
        .unwrap_or(base)
}

/// Convert to uppercase using Rust's Unicode-aware method
///
/// This handles all Vietnamese characters correctly without
/// explicit character mapping.
fn to_upper(ch: char) -> char {
    ch.to_uppercase().next().unwrap_or(ch)
}

/// Convert key + modifiers to Vietnamese character
///
/// # Arguments
/// * `key` - Virtual keycode
/// * `caps` - Uppercase flag
/// * `tone` - Tone modifier: 0=none, 1=circumflex(^), 2=horn/breve
/// * `mark` - Mark: 0=none, 1=sắc, 2=huyền, 3=hỏi, 4=ngã, 5=nặng
pub fn to_char(key: u16, caps: bool, tone: u8, mark: u8) -> Option<char> {
    // Handle D specially (not a vowel but needs conversion)
    if key == keys::D {
        return Some(if caps { 'D' } else { 'd' });
    }

    let base = get_base_char(key, tone)?;
    let marked = apply_mark(base, mark);
    Some(if caps { to_upper(marked) } else { marked })
}

/// Get đ/Đ character
pub fn get_d(caps: bool) -> char {
    if caps {
        'Đ'
    } else {
        'đ'
    }
}

// REVERSE PARSING: Vietnamese char → buffer components

/// Parsed character components for buffer restoration
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParsedChar {
    pub key: u16,
    pub caps: bool,
    pub tone: u8,
    pub mark: u8,
    pub stroke: bool,
}

impl ParsedChar {
    const fn new(key: u16, caps: bool, t: u8, m: u8) -> Self {
        Self {
            key,
            caps,
            tone: t,
            mark: m,
            stroke: false,
        }
    }

    const fn stroke(key: u16, caps: bool) -> Self {
        Self {
            key,
            caps,
            tone: 0,
            mark: 0,
            stroke: true,
        }
    }
}

/// Parse Vietnamese character back to buffer components
///
/// Returns None for unknown characters (symbols, numbers handled separately).
/// O(1) via compiler-optimized match.
pub fn parse_char(c: char) -> Option<ParsedChar> {
    use keys::*;
    use mark::{HOI, HUYEN, NANG, NGA, SAC};
    use tone::{CIRCUMFLEX, HORN};
    // 0 for no tone/mark
    const T0: u8 = 0;
    const M0: u8 = 0;

    // Macro to reduce boilerplate for vowel parsing
    macro_rules! vowel {
        ($key:expr, $caps:expr, $tone:expr, $mark:expr) => {
            Some(ParsedChar::new($key, $caps, $tone, $mark))
        };
    }

    match c {
        // ===== A variants =====
        'a' => vowel!(A, false, T0, M0),
        'A' => vowel!(A, true, T0, M0),
        'á' => vowel!(A, false, T0, SAC),
        'Á' => vowel!(A, true, T0, SAC),
        'à' => vowel!(A, false, T0, HUYEN),
        'À' => vowel!(A, true, T0, HUYEN),
        'ả' => vowel!(A, false, T0, HOI),
        'Ả' => vowel!(A, true, T0, HOI),
        'ã' => vowel!(A, false, T0, NGA),
        'Ã' => vowel!(A, true, T0, NGA),
        'ạ' => vowel!(A, false, T0, NANG),
        'Ạ' => vowel!(A, true, T0, NANG),
        // ă (breve)
        'ă' => vowel!(A, false, HORN, M0),
        'Ă' => vowel!(A, true, HORN, M0),
        'ắ' => vowel!(A, false, HORN, SAC),
        'Ắ' => vowel!(A, true, HORN, SAC),
        'ằ' => vowel!(A, false, HORN, HUYEN),
        'Ằ' => vowel!(A, true, HORN, HUYEN),
        'ẳ' => vowel!(A, false, HORN, HOI),
        'Ẳ' => vowel!(A, true, HORN, HOI),
        'ẵ' => vowel!(A, false, HORN, NGA),
        'Ẵ' => vowel!(A, true, HORN, NGA),
        'ặ' => vowel!(A, false, HORN, NANG),
        'Ặ' => vowel!(A, true, HORN, NANG),
        // â (circumflex)
        'â' => vowel!(A, false, CIRCUMFLEX, M0),
        'Â' => vowel!(A, true, CIRCUMFLEX, M0),
        'ấ' => vowel!(A, false, CIRCUMFLEX, SAC),
        'Ấ' => vowel!(A, true, CIRCUMFLEX, SAC),
        'ầ' => vowel!(A, false, CIRCUMFLEX, HUYEN),
        'Ầ' => vowel!(A, true, CIRCUMFLEX, HUYEN),
        'ẩ' => vowel!(A, false, CIRCUMFLEX, HOI),
        'Ẩ' => vowel!(A, true, CIRCUMFLEX, HOI),
        'ẫ' => vowel!(A, false, CIRCUMFLEX, NGA),
        'Ẫ' => vowel!(A, true, CIRCUMFLEX, NGA),
        'ậ' => vowel!(A, false, CIRCUMFLEX, NANG),
        'Ậ' => vowel!(A, true, CIRCUMFLEX, NANG),

        // ===== E variants =====
        'e' => vowel!(E, false, T0, M0),
        'E' => vowel!(E, true, T0, M0),
        'é' => vowel!(E, false, T0, SAC),
        'É' => vowel!(E, true, T0, SAC),
        'è' => vowel!(E, false, T0, HUYEN),
        'È' => vowel!(E, true, T0, HUYEN),
        'ẻ' => vowel!(E, false, T0, HOI),
        'Ẻ' => vowel!(E, true, T0, HOI),
        'ẽ' => vowel!(E, false, T0, NGA),
        'Ẽ' => vowel!(E, true, T0, NGA),
        'ẹ' => vowel!(E, false, T0, NANG),
        'Ẹ' => vowel!(E, true, T0, NANG),
        // ê (circumflex)
        'ê' => vowel!(E, false, CIRCUMFLEX, M0),
        'Ê' => vowel!(E, true, CIRCUMFLEX, M0),
        'ế' => vowel!(E, false, CIRCUMFLEX, SAC),
        'Ế' => vowel!(E, true, CIRCUMFLEX, SAC),
        'ề' => vowel!(E, false, CIRCUMFLEX, HUYEN),
        'Ề' => vowel!(E, true, CIRCUMFLEX, HUYEN),
        'ể' => vowel!(E, false, CIRCUMFLEX, HOI),
        'Ể' => vowel!(E, true, CIRCUMFLEX, HOI),
        'ễ' => vowel!(E, false, CIRCUMFLEX, NGA),
        'Ễ' => vowel!(E, true, CIRCUMFLEX, NGA),
        'ệ' => vowel!(E, false, CIRCUMFLEX, NANG),
        'Ệ' => vowel!(E, true, CIRCUMFLEX, NANG),

        // ===== I variants =====
        'i' => vowel!(I, false, T0, M0),
        'I' => vowel!(I, true, T0, M0),
        'í' => vowel!(I, false, T0, SAC),
        'Í' => vowel!(I, true, T0, SAC),
        'ì' => vowel!(I, false, T0, HUYEN),
        'Ì' => vowel!(I, true, T0, HUYEN),
        'ỉ' => vowel!(I, false, T0, HOI),
        'Ỉ' => vowel!(I, true, T0, HOI),
        'ĩ' => vowel!(I, false, T0, NGA),
        'Ĩ' => vowel!(I, true, T0, NGA),
        'ị' => vowel!(I, false, T0, NANG),
        'Ị' => vowel!(I, true, T0, NANG),

        // ===== O variants =====
        'o' => vowel!(O, false, T0, M0),
        'O' => vowel!(O, true, T0, M0),
        'ó' => vowel!(O, false, T0, SAC),
        'Ó' => vowel!(O, true, T0, SAC),
        'ò' => vowel!(O, false, T0, HUYEN),
        'Ò' => vowel!(O, true, T0, HUYEN),
        'ỏ' => vowel!(O, false, T0, HOI),
        'Ỏ' => vowel!(O, true, T0, HOI),
        'õ' => vowel!(O, false, T0, NGA),
        'Õ' => vowel!(O, true, T0, NGA),
        'ọ' => vowel!(O, false, T0, NANG),
        'Ọ' => vowel!(O, true, T0, NANG),
        // ô (circumflex)
        'ô' => vowel!(O, false, CIRCUMFLEX, M0),
        'Ô' => vowel!(O, true, CIRCUMFLEX, M0),
        'ố' => vowel!(O, false, CIRCUMFLEX, SAC),
        'Ố' => vowel!(O, true, CIRCUMFLEX, SAC),
        'ồ' => vowel!(O, false, CIRCUMFLEX, HUYEN),
        'Ồ' => vowel!(O, true, CIRCUMFLEX, HUYEN),
        'ổ' => vowel!(O, false, CIRCUMFLEX, HOI),
        'Ổ' => vowel!(O, true, CIRCUMFLEX, HOI),
        'ỗ' => vowel!(O, false, CIRCUMFLEX, NGA),
        'Ỗ' => vowel!(O, true, CIRCUMFLEX, NGA),
        'ộ' => vowel!(O, false, CIRCUMFLEX, NANG),
        'Ộ' => vowel!(O, true, CIRCUMFLEX, NANG),
        // ơ (horn)
        'ơ' => vowel!(O, false, HORN, M0),
        'Ơ' => vowel!(O, true, HORN, M0),
        'ớ' => vowel!(O, false, HORN, SAC),
        'Ớ' => vowel!(O, true, HORN, SAC),
        'ờ' => vowel!(O, false, HORN, HUYEN),
        'Ờ' => vowel!(O, true, HORN, HUYEN),
        'ở' => vowel!(O, false, HORN, HOI),
        'Ở' => vowel!(O, true, HORN, HOI),
        'ỡ' => vowel!(O, false, HORN, NGA),
        'Ỡ' => vowel!(O, true, HORN, NGA),
        'ợ' => vowel!(O, false, HORN, NANG),
        'Ợ' => vowel!(O, true, HORN, NANG),

        // ===== U variants =====
        'u' => vowel!(U, false, T0, M0),
        'U' => vowel!(U, true, T0, M0),
        'ú' => vowel!(U, false, T0, SAC),
        'Ú' => vowel!(U, true, T0, SAC),
        'ù' => vowel!(U, false, T0, HUYEN),
        'Ù' => vowel!(U, true, T0, HUYEN),
        'ủ' => vowel!(U, false, T0, HOI),
        'Ủ' => vowel!(U, true, T0, HOI),
        'ũ' => vowel!(U, false, T0, NGA),
        'Ũ' => vowel!(U, true, T0, NGA),
        'ụ' => vowel!(U, false, T0, NANG),
        'Ụ' => vowel!(U, true, T0, NANG),
        // ư (horn)
        'ư' => vowel!(U, false, HORN, M0),
        'Ư' => vowel!(U, true, HORN, M0),
        'ứ' => vowel!(U, false, HORN, SAC),
        'Ứ' => vowel!(U, true, HORN, SAC),
        'ừ' => vowel!(U, false, HORN, HUYEN),
        'Ừ' => vowel!(U, true, HORN, HUYEN),
        'ử' => vowel!(U, false, HORN, HOI),
        'Ử' => vowel!(U, true, HORN, HOI),
        'ữ' => vowel!(U, false, HORN, NGA),
        'Ữ' => vowel!(U, true, HORN, NGA),
        'ự' => vowel!(U, false, HORN, NANG),
        'Ự' => vowel!(U, true, HORN, NANG),

        // ===== Y variants =====
        'y' => vowel!(Y, false, T0, M0),
        'Y' => vowel!(Y, true, T0, M0),
        'ý' => vowel!(Y, false, T0, SAC),
        'Ý' => vowel!(Y, true, T0, SAC),
        'ỳ' => vowel!(Y, false, T0, HUYEN),
        'Ỳ' => vowel!(Y, true, T0, HUYEN),
        'ỷ' => vowel!(Y, false, T0, HOI),
        'Ỷ' => vowel!(Y, true, T0, HOI),
        'ỹ' => vowel!(Y, false, T0, NGA),
        'Ỹ' => vowel!(Y, true, T0, NGA),
        'ỵ' => vowel!(Y, false, T0, NANG),
        'Ỵ' => vowel!(Y, true, T0, NANG),

        // ===== Consonants =====
        'đ' => Some(ParsedChar::stroke(D, false)),
        'Đ' => Some(ParsedChar::stroke(D, true)),
        'd' => Some(ParsedChar::new(D, false, 0, 0)),
        'D' => Some(ParsedChar::new(D, true, 0, 0)),
        'b' => Some(ParsedChar::new(B, false, 0, 0)),
        'B' => Some(ParsedChar::new(B, true, 0, 0)),
        'c' => Some(ParsedChar::new(C, false, 0, 0)),
        'C' => Some(ParsedChar::new(C, true, 0, 0)),
        'f' => Some(ParsedChar::new(F, false, 0, 0)),
        'F' => Some(ParsedChar::new(F, true, 0, 0)),
        'g' => Some(ParsedChar::new(G, false, 0, 0)),
        'G' => Some(ParsedChar::new(G, true, 0, 0)),
        'h' => Some(ParsedChar::new(H, false, 0, 0)),
        'H' => Some(ParsedChar::new(H, true, 0, 0)),
        'j' => Some(ParsedChar::new(J, false, 0, 0)),
        'J' => Some(ParsedChar::new(J, true, 0, 0)),
        'k' => Some(ParsedChar::new(K, false, 0, 0)),
        'K' => Some(ParsedChar::new(K, true, 0, 0)),
        'l' => Some(ParsedChar::new(L, false, 0, 0)),
        'L' => Some(ParsedChar::new(L, true, 0, 0)),
        'm' => Some(ParsedChar::new(M, false, 0, 0)),
        'M' => Some(ParsedChar::new(M, true, 0, 0)),
        'n' => Some(ParsedChar::new(N, false, 0, 0)),
        'N' => Some(ParsedChar::new(N, true, 0, 0)),
        'p' => Some(ParsedChar::new(P, false, 0, 0)),
        'P' => Some(ParsedChar::new(P, true, 0, 0)),
        'q' => Some(ParsedChar::new(Q, false, 0, 0)),
        'Q' => Some(ParsedChar::new(Q, true, 0, 0)),
        'r' => Some(ParsedChar::new(R, false, 0, 0)),
        'R' => Some(ParsedChar::new(R, true, 0, 0)),
        's' => Some(ParsedChar::new(S, false, 0, 0)),
        'S' => Some(ParsedChar::new(S, true, 0, 0)),
        't' => Some(ParsedChar::new(T, false, 0, 0)),
        'T' => Some(ParsedChar::new(T, true, 0, 0)),
        'v' => Some(ParsedChar::new(V, false, 0, 0)),
        'V' => Some(ParsedChar::new(V, true, 0, 0)),
        'w' => Some(ParsedChar::new(W, false, 0, 0)),
        'W' => Some(ParsedChar::new(W, true, 0, 0)),
        'x' => Some(ParsedChar::new(X, false, 0, 0)),
        'X' => Some(ParsedChar::new(X, true, 0, 0)),
        'z' => Some(ParsedChar::new(Z, false, 0, 0)),
        'Z' => Some(ParsedChar::new(Z, true, 0, 0)),

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_vowels() {
        // Basic vowels without modifiers
        assert_eq!(to_char(keys::A, false, 0, 0), Some('a'));
        assert_eq!(to_char(keys::E, false, 0, 0), Some('e'));
        assert_eq!(to_char(keys::I, false, 0, 0), Some('i'));
        assert_eq!(to_char(keys::O, false, 0, 0), Some('o'));
        assert_eq!(to_char(keys::U, false, 0, 0), Some('u'));
        assert_eq!(to_char(keys::Y, false, 0, 0), Some('y'));
    }

    #[test]
    fn test_tone_modifiers() {
        // Circumflex (^)
        assert_eq!(to_char(keys::A, false, 1, 0), Some('â'));
        assert_eq!(to_char(keys::E, false, 1, 0), Some('ê'));
        assert_eq!(to_char(keys::O, false, 1, 0), Some('ô'));

        // Horn/Breve
        assert_eq!(to_char(keys::A, false, 2, 0), Some('ă'));
        assert_eq!(to_char(keys::O, false, 2, 0), Some('ơ'));
        assert_eq!(to_char(keys::U, false, 2, 0), Some('ư'));
    }

    #[test]
    fn test_marks() {
        // All 5 marks on 'a'
        assert_eq!(to_char(keys::A, false, 0, 1), Some('á')); // sắc
        assert_eq!(to_char(keys::A, false, 0, 2), Some('à')); // huyền
        assert_eq!(to_char(keys::A, false, 0, 3), Some('ả')); // hỏi
        assert_eq!(to_char(keys::A, false, 0, 4), Some('ã')); // ngã
        assert_eq!(to_char(keys::A, false, 0, 5), Some('ạ')); // nặng
    }

    #[test]
    fn test_combined_tone_and_mark() {
        // â + sắc = ấ
        assert_eq!(to_char(keys::A, false, 1, 1), Some('ấ'));
        // ơ + huyền = ờ
        assert_eq!(to_char(keys::O, false, 2, 2), Some('ờ'));
        // ư + nặng = ự
        assert_eq!(to_char(keys::U, false, 2, 5), Some('ự'));
    }

    #[test]
    fn test_uppercase() {
        assert_eq!(to_char(keys::A, true, 0, 0), Some('A'));
        assert_eq!(to_char(keys::A, true, 0, 1), Some('Á'));
        assert_eq!(to_char(keys::A, true, 1, 1), Some('Ấ'));
        assert_eq!(to_char(keys::O, true, 2, 2), Some('Ờ'));
        assert_eq!(to_char(keys::U, true, 2, 5), Some('Ự'));
    }

    #[test]
    fn test_d() {
        assert_eq!(get_d(false), 'đ');
        assert_eq!(get_d(true), 'Đ');
    }

    // ===== Reverse parsing tests =====

    #[test]
    fn test_parse_basic_vowels() {
        let p = parse_char('a').unwrap();
        assert_eq!((p.key, p.caps, p.tone, p.mark), (keys::A, false, 0, 0));

        let p = parse_char('E').unwrap();
        assert_eq!((p.key, p.caps, p.tone, p.mark), (keys::E, true, 0, 0));
    }

    #[test]
    fn test_parse_vowels_with_marks() {
        let p = parse_char('á').unwrap();
        assert_eq!((p.key, p.tone, p.mark), (keys::A, 0, mark::SAC));

        let p = parse_char('ụ').unwrap();
        assert_eq!((p.key, p.tone, p.mark), (keys::U, 0, mark::NANG));

        let p = parse_char('Ề').unwrap();
        assert_eq!(
            (p.key, p.caps, p.tone, p.mark),
            (keys::E, true, tone::CIRCUMFLEX, mark::HUYEN)
        );
    }

    #[test]
    fn test_parse_complex_vowels() {
        // ự = ư + nặng
        let p = parse_char('ự').unwrap();
        assert_eq!((p.key, p.tone, p.mark), (keys::U, tone::HORN, mark::NANG));

        // ấ = â + sắc
        let p = parse_char('ấ').unwrap();
        assert_eq!(
            (p.key, p.tone, p.mark),
            (keys::A, tone::CIRCUMFLEX, mark::SAC)
        );
    }

    #[test]
    fn test_parse_d_stroke() {
        let p = parse_char('đ').unwrap();
        assert!(p.stroke);
        assert_eq!(p.key, keys::D);
        assert!(!p.caps);

        let p = parse_char('Đ').unwrap();
        assert!(p.stroke);
        assert!(p.caps);
    }

    #[test]
    fn test_parse_consonants() {
        let p = parse_char('n').unwrap();
        assert_eq!(p.key, keys::N);
        assert!(!p.stroke);

        let p = parse_char('T').unwrap();
        assert_eq!(p.key, keys::T);
        assert!(p.caps);
    }

    #[test]
    fn test_parse_roundtrip() {
        // Test that parse_char is inverse of to_char for all vowels
        let test_cases = [
            ('a', keys::A, 0, 0),
            ('á', keys::A, 0, 1),
            ('ả', keys::A, 0, 3),
            ('â', keys::A, 1, 0),
            ('ấ', keys::A, 1, 1),
            ('ậ', keys::A, 1, 5),
            ('ă', keys::A, 2, 0),
            ('ắ', keys::A, 2, 1),
            ('ư', keys::U, 2, 0),
            ('ự', keys::U, 2, 5),
            ('ơ', keys::O, 2, 0),
            ('ợ', keys::O, 2, 5),
        ];
        for (ch, key, t, m) in test_cases {
            let p = parse_char(ch).unwrap();
            assert_eq!((p.key, p.tone, p.mark), (key, t, m), "Failed for '{}'", ch);
        }
    }
}
