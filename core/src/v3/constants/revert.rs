//! Revert triggers for double-key patterns
//!
//! Double-key patterns revert transformations:
//! - ss: Remove sắc tone
//! - ff: Remove huyền tone
//! - rr: Remove hỏi tone
//! - xx: Remove ngã tone
//! - jj: Remove nặng tone
//! - aa: Remove circumflex (â -> a)
//! - ee: Remove circumflex (ê -> e)
//! - oo: Remove circumflex (ô -> o)
//! - dd: Remove stroke (đ -> d)
//! - ww: Remove horn/breve

/// Revert key lookup (11 bytes)
/// Maps key to revert type (0 = no revert)
pub const REVERT_KEY: [u8; 26] = [
    1,  // a - circumflex revert
    0,  // b
    0,  // c
    2,  // d - stroke revert
    3,  // e - circumflex revert
    4,  // f - tone revert (huyền)
    0,  // g
    0,  // h
    0,  // i
    5,  // j - tone revert (nặng)
    0,  // k
    0,  // l
    0,  // m
    0,  // n
    6,  // o - circumflex revert
    0,  // p
    0,  // q
    7,  // r - tone revert (hỏi)
    8,  // s - tone revert (sắc)
    0,  // t
    0,  // u
    0,  // v
    9,  // w - horn/breve revert
    10, // x - tone revert (ngã)
    0,  // y
    0,  // z
];

/// Revert types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RevertType {
    /// No revert
    None = 0,
    /// Circumflex revert (â -> a)
    CircumflexA = 1,
    /// Stroke revert (đ -> d)
    Stroke = 2,
    /// Circumflex revert (ê -> e)
    CircumflexE = 3,
    /// Tone huyền revert
    ToneHuyen = 4,
    /// Tone nặng revert
    ToneNang = 5,
    /// Circumflex revert (ô -> o)
    CircumflexO = 6,
    /// Tone hỏi revert
    ToneHoi = 7,
    /// Tone sắc revert
    ToneSac = 8,
    /// Horn/breve revert
    Horn = 9,
    /// Tone ngã revert
    ToneNga = 10,
}

/// Check if double-key should trigger revert
#[inline]
pub fn should_revert(last_key: char, new_key: char) -> Option<RevertType> {
    let last = last_key.to_ascii_lowercase();
    let new = new_key.to_ascii_lowercase();

    // Must be same key
    if last != new {
        return None;
    }

    // Check if key has revert type
    if ('a'..='z').contains(&last) {
        let idx = (last as u8 - b'a') as usize;
        let revert = REVERT_KEY[idx];
        if revert > 0 {
            return Some(match revert {
                1 => RevertType::CircumflexA,
                2 => RevertType::Stroke,
                3 => RevertType::CircumflexE,
                4 => RevertType::ToneHuyen,
                5 => RevertType::ToneNang,
                6 => RevertType::CircumflexO,
                7 => RevertType::ToneHoi,
                8 => RevertType::ToneSac,
                9 => RevertType::Horn,
                10 => RevertType::ToneNga,
                _ => return None,
            });
        }
    }

    None
}

/// Check if revert is tone-related
#[inline]
pub fn is_tone_revert(rt: RevertType) -> bool {
    matches!(
        rt,
        RevertType::ToneSac
            | RevertType::ToneHuyen
            | RevertType::ToneHoi
            | RevertType::ToneNga
            | RevertType::ToneNang
    )
}

/// Check if revert is mark-related (circumflex, stroke, horn)
#[inline]
pub fn is_mark_revert(rt: RevertType) -> bool {
    matches!(
        rt,
        RevertType::CircumflexA
            | RevertType::CircumflexE
            | RevertType::CircumflexO
            | RevertType::Stroke
            | RevertType::Horn
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tone_reverts() {
        assert_eq!(should_revert('s', 's'), Some(RevertType::ToneSac));
        assert_eq!(should_revert('f', 'f'), Some(RevertType::ToneHuyen));
        assert_eq!(should_revert('r', 'r'), Some(RevertType::ToneHoi));
        assert_eq!(should_revert('x', 'x'), Some(RevertType::ToneNga));
        assert_eq!(should_revert('j', 'j'), Some(RevertType::ToneNang));
    }

    #[test]
    fn test_circumflex_reverts() {
        assert_eq!(should_revert('a', 'a'), Some(RevertType::CircumflexA));
        assert_eq!(should_revert('e', 'e'), Some(RevertType::CircumflexE));
        assert_eq!(should_revert('o', 'o'), Some(RevertType::CircumflexO));
    }

    #[test]
    fn test_stroke_revert() {
        assert_eq!(should_revert('d', 'd'), Some(RevertType::Stroke));
    }

    #[test]
    fn test_no_revert() {
        assert_eq!(should_revert('a', 'b'), None);
        assert_eq!(should_revert('b', 'b'), None); // b has no revert
    }

    #[test]
    fn test_revert_classification() {
        assert!(is_tone_revert(RevertType::ToneSac));
        assert!(is_mark_revert(RevertType::CircumflexA));
        assert!(!is_tone_revert(RevertType::CircumflexA));
        assert!(!is_mark_revert(RevertType::ToneSac));
    }
}
