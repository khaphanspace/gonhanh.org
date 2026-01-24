//! Vietnamese Vowel System
//!
//! Implements phonological classification of Vietnamese vowels based on:
//! - docs/vietnamese-language-system.md
//! - https://vi.wikipedia.org/wiki/Quy_tắc_đặt_dấu_thanh_của_chữ_Quốc_ngữ
//!
//! ## Vowel Classification
//!
//! Vietnamese has 12 vowels with 3 modifier types:
//! - Simple: a, e, i, o, u, y
//! - Circumflex (^): â, ê, ô
//! - Horn (móc): ơ, ư
//! - Breve (trăng): ă
//!
//! ## Phonological Roles
//!
//! In Vietnamese syllable structure (C)(G)V(C):
//! - **Medial (âm đệm)**: o, u when followed by main vowel (oa, oe, uy, ua, uê)
//! - **Main (âm chính)**: The primary vowel carrying tone
//! - **Glide (bán nguyên âm)**: i/y, u/o at syllable end (ai, ao, iu, oi)

use super::keys;

/// Vowel modifier type (dấu phụ)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Modifier {
    None = 0,       // a, e, i, o, u, y
    Circumflex = 1, // â, ê, ô (^)
    Horn = 2,       // ơ, ư (móc) / ă (trăng)
}

/// Phonological role in syllable
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Main,   // Primary vowel (carries tone)
    Medial, // Glide before main vowel (o in oa, u in uy)
    Final,  // Glide at syllable end (i in ai, u in au)
}

/// Vowel information
#[derive(Clone, Copy, Debug)]
pub struct Vowel {
    pub key: u16,
    pub modifier: Modifier,
    pub pos: usize,
}

impl Vowel {
    pub fn new(key: u16, modifier: Modifier, pos: usize) -> Self {
        Self { key, modifier, pos }
    }

    /// Check if this vowel has a diacritic modifier (^, ơ, ư, ă)
    pub fn has_diacritic(&self) -> bool {
        self.modifier != Modifier::None
    }
}

// =============================================================================
// VOWEL PATTERN ENUMS - Shared across tone and horn placement
// =============================================================================

/// Position for tone mark placement
///
/// Based on docs/vietnamese-language-system.md section 7.3
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TonePosition {
    /// Position 1 - First vowel (âm chính + glide: ai, ao, ia, ưu...)
    First,
    /// Position 2 - Second vowel (âm đệm + chính: oa, uy, compound: iê, uô, ươ)
    Second,
    /// Position 3 - Last vowel (only uyê triphthong)
    Last,
}

/// Horn placement rule for a vowel pair
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HornPlacement {
    /// Apply horn to BOTH vowels (compound ươ)
    Both,
    /// Apply horn to FIRST vowel only
    First,
    /// Apply horn/breve to SECOND vowel only
    Second,
}

/// Vowel pair pattern with horn placement rule
///
/// Based on docs/vietnamese-language-system.md section 3.2 (Nguyên âm đôi)
pub struct VowelPairPattern {
    /// First vowel key
    pub v1: u16,
    /// Second vowel key
    pub v2: u16,
    /// Where to place horn modifier
    pub placement: HornPlacement,
    /// Description for documentation
    pub desc: &'static str,
}

/// All vowel pair patterns for horn/breve placement
///
/// Order matters: first match wins
pub const HORN_PATTERNS: &[VowelPairPattern] = &[
    // Compound ươ - both vowels get horn
    VowelPairPattern {
        v1: keys::U,
        v2: keys::O,
        placement: HornPlacement::Both,
        desc: "ươ compound (được, ướt)",
    },
    VowelPairPattern {
        v1: keys::O,
        v2: keys::U,
        placement: HornPlacement::Both,
        desc: "ươ compound reversed",
    },
    // ưu cluster - first u gets horn
    VowelPairPattern {
        v1: keys::U,
        v2: keys::U,
        placement: HornPlacement::First,
        desc: "ưu cluster (lưu, hưu, ngưu)",
    },
    // Breve patterns - second vowel gets breve
    VowelPairPattern {
        v1: keys::O,
        v2: keys::A,
        placement: HornPlacement::Second,
        desc: "oă pattern (hoặc, xoắn)",
    },
    // ua pattern - context-dependent (handled specially)
    // Default: second gets breve, but with consonant prefix: first gets horn
];

// =============================================================================
// TONE POSITION PATTERNS - Based on docs/vietnamese-language-system.md 7.3
// =============================================================================

/// Triphthong pattern for tone placement
pub struct TriphthongTonePattern {
    pub v1: u16,
    pub v2: u16,
    pub v3: u16,
    pub position: TonePosition,
}

/// Diphthongs with tone on FIRST vowel (âm chính + glide)
///
/// Section 7.3.1: ai, ao, au, ay, âu, ây, eo, êu, ia, iu, oi, ôi, ơi, ui, ưi, ưu, ua*, ưa
/// *ua is First only when NOT preceded by 'q'
pub const TONE_FIRST_PATTERNS: &[[u16; 2]] = &[
    [keys::A, keys::I], // ai: mái, hài
    [keys::A, keys::O], // ao: cáo, sào
    [keys::A, keys::U], // au: sáu, màu
    [keys::A, keys::Y], // ay: máy, tày
    [keys::E, keys::O], // eo: kéo, trèo
    [keys::E, keys::U], // êu: nếu, kêu (circumflex on e, tone on e)
    [keys::I, keys::A], // ia: kìa, mía (not after gi)
    [keys::I, keys::U], // iu: dịu, kíu
    [keys::O, keys::I], // oi: đói, còi
    [keys::U, keys::I], // ui: túi, mùi
    // Note: "ua" removed - context-dependent, handled specially in find_diphthong_position()
    // Open syllable: tone on u (mùa), Closed syllable: tone on a (chuẩn)
    [keys::U, keys::U], // ưu: lưu, hưu (when first has horn)
];

/// Diphthongs with tone on SECOND vowel (âm đệm + chính, compound)
///
/// Section 7.3.1: oa, oă, oe, uê, uy, ua (after q), iê, uô, ươ
pub const TONE_SECOND_PATTERNS: &[[u16; 2]] = &[
    [keys::O, keys::A], // oa: hoà, toá
    [keys::O, keys::E], // oe: khoẻ, xoè
    [keys::U, keys::E], // uê: huế, tuệ
    [keys::U, keys::Y], // uy: quý, thuỳ
    [keys::I, keys::E], // iê: tiên (compound)
    [keys::U, keys::O], // uô/ươ: (compound - when both have horn)
];

/// Triphthongs - all use middle (position 2) except uyê
///
/// Section 7.3.3
pub const TRIPHTHONG_PATTERNS: &[TriphthongTonePattern] = &[
    TriphthongTonePattern {
        v1: keys::I,
        v2: keys::E,
        v3: keys::U,
        position: TonePosition::Second,
    }, // iêu: tiếu
    TriphthongTonePattern {
        v1: keys::Y,
        v2: keys::E,
        v3: keys::U,
        position: TonePosition::Second,
    }, // yêu: yếu
    TriphthongTonePattern {
        v1: keys::O,
        v2: keys::A,
        v3: keys::I,
        position: TonePosition::Second,
    }, // oai: ngoài
    TriphthongTonePattern {
        v1: keys::O,
        v2: keys::A,
        v3: keys::Y,
        position: TonePosition::Second,
    }, // oay: xoáy
    TriphthongTonePattern {
        v1: keys::O,
        v2: keys::E,
        v3: keys::O,
        position: TonePosition::Second,
    }, // oeo: khoèo
    TriphthongTonePattern {
        v1: keys::U,
        v2: keys::A,
        v3: keys::Y,
        position: TonePosition::Second,
    }, // uây: khuấy (â in middle)
    TriphthongTonePattern {
        v1: keys::U,
        v2: keys::O,
        v3: keys::I,
        position: TonePosition::Second,
    }, // uôi: cuối
    TriphthongTonePattern {
        v1: keys::U,
        v2: keys::O,
        v3: keys::I,
        position: TonePosition::Second,
    }, // ươi: mười (ư+ơ+i, both have horn)
    TriphthongTonePattern {
        v1: keys::U,
        v2: keys::O,
        v3: keys::U,
        position: TonePosition::Second,
    }, // ươu: rượu
    TriphthongTonePattern {
        v1: keys::I,
        v2: keys::U,
        v3: keys::O,
        position: TonePosition::Last,
    }, // iươ: giường (gi + ươ, tone on ơ)
    // Special: uyê uses Last position
    TriphthongTonePattern {
        v1: keys::U,
        v2: keys::Y,
        v3: keys::E,
        position: TonePosition::Last,
    }, // uyê: khuyến, quyền
];

/// Vietnamese vowel phonology analyzer
pub struct Phonology;

impl Phonology {
    /// Find the position where tone mark should be placed
    ///
    /// Uses data-driven pattern lookup from TONE_*_PATTERNS and TRIPHTHONG_PATTERNS.
    /// See docs/vietnamese-language-system.md section 7.3 for the complete matrix.
    ///
    /// ## Rules Summary
    /// 1. Single vowel: mark on it
    /// 2. Diacritic priority: vowel with diacritic (ă,â,ê,ô,ơ,ư) gets the mark
    /// 3. With final consonant: mark on 2nd vowel
    /// 4. Open syllable: use pattern tables
    /// 5. Triphthong: use TRIPHTHONG_PATTERNS (middle, except uyê → last)
    pub fn find_tone_position(
        vowels: &[Vowel],
        has_final_consonant: bool,
        modern: bool,
        has_qu_initial: bool,
        has_gi_initial: bool,
    ) -> usize {
        // Handle gi-initial: first vowel 'i' is part of consonant, use remaining vowels
        // Example: "giàu" → vowels [i, a, u], but with gi-initial, treat as [a, u] diphthong
        if has_gi_initial && vowels.len() >= 2 && vowels[0].key == keys::I {
            let remaining = &vowels[1..];
            return match remaining.len() {
                0 => vowels[0].pos, // Shouldn't happen, but fallback
                1 => remaining[0].pos,
                2 => Self::find_diphthong_position(
                    remaining,
                    has_final_consonant,
                    modern,
                    false,
                    false,
                ),
                _ => Self::find_default_position(remaining),
            };
        }

        // Handle qu-initial: first vowel 'u' is part of consonant, use remaining vowels
        // Example: "quào" → vowels [u, a, o], but with qu-initial, treat as [a, o] diphthong
        if has_qu_initial && vowels.len() >= 2 && vowels[0].key == keys::U {
            let remaining = &vowels[1..];
            return match remaining.len() {
                0 => vowels[0].pos, // Shouldn't happen, but fallback
                1 => remaining[0].pos,
                2 => Self::find_diphthong_position(
                    remaining,
                    has_final_consonant,
                    modern,
                    false, // No longer qu-initial for remaining vowels
                    false,
                ),
                _ => Self::find_default_position(remaining),
            };
        }

        match vowels.len() {
            0 => 0,
            1 => vowels[0].pos,
            2 => Self::find_diphthong_position(
                vowels,
                has_final_consonant,
                modern,
                has_qu_initial,
                has_gi_initial,
            ),
            3 => Self::find_triphthong_position(vowels),
            _ => {
                // For 4+ vowels, check if first 3 form a triphthong pattern
                // This handles cases like "cuoiwsi" → "cướii" (ươi + i)
                // where the triphthong "ươi" should get the tone on ơ (position 2)
                if vowels.len() >= 3 {
                    let first_three = &vowels[0..3];
                    let triphthong_pos = Self::find_triphthong_position(first_three);
                    // triphthong_pos is already a buffer position from vowels[i].pos
                    // Check if it's within the first 3 vowels (valid triphthong match)
                    if triphthong_pos >= vowels[0].pos && triphthong_pos <= vowels[2].pos {
                        return triphthong_pos;
                    }
                }
                // Fall back to default position
                Self::find_default_position(vowels)
            }
        }
    }

    /// Find tone position for diphthongs (2 vowels)
    fn find_diphthong_position(
        vowels: &[Vowel],
        has_final_consonant: bool,
        modern: bool,
        has_qu_initial: bool,
        has_gi_initial: bool,
    ) -> usize {
        let (v1, v2) = (&vowels[0], &vowels[1]);
        let pair = [v1.key, v2.key];

        // Rule 0: TONE_FIRST_PATTERNS (main+glide) always get mark on first vowel
        // This MUST come before final consonant rule because main+glide patterns
        // have mark on main vowel regardless of syllable structure
        // Example: "au" → mark on 'a' whether open (màu) or closed (aum → àum)
        if TONE_FIRST_PATTERNS
            .iter()
            .any(|p| p[0] == pair[0] && p[1] == pair[1])
        {
            return v1.pos;
        }

        // Rule 1: With final consonant → 2nd vowel (for medial+main patterns)
        // Only applies to patterns NOT in TONE_FIRST_PATTERNS
        if has_final_consonant {
            return v2.pos;
        }

        // Rule 2: Diacritic priority (Section 7.2.5)
        // - If 1st has diacritic and 2nd doesn't → 1st (ưa → ư)
        // - If 2nd has diacritic → 2nd (iê → ê)
        if v1.has_diacritic() && !v2.has_diacritic() {
            return v1.pos;
        }
        if v2.has_diacritic() {
            return v2.pos;
        }

        // Rule 3: Context-dependent patterns
        // ia: 1st unless gi-initial (gia → a, kìa → i)
        if v1.key == keys::I && v2.key == keys::A {
            return if has_gi_initial { v2.pos } else { v1.pos };
        }

        // ua: context-dependent based on syllable structure
        // - Open syllable (mùa): u is main, a is glide → tone on u (1st)
        // - Closed syllable (chuẩn): u is medial, a is main → tone on a (2nd)
        // - qu-initial (quà): u is part of consonant → tone on a (2nd)
        if v1.key == keys::U && v2.key == keys::A {
            return if has_final_consonant || has_qu_initial {
                v2.pos
            } else {
                v1.pos
            };
        }

        // uy with qu-initial: always on y (quý - 'u' is part of qu consonant)
        // Not affected by modern setting
        if v1.key == keys::U && v2.key == keys::Y && has_qu_initial {
            return v2.pos;
        }

        // Rule 4: TONE_SECOND_PATTERNS (medial + main, compound)
        // Modern setting only affects: oa, oe, uy (without qu-initial)
        if TONE_SECOND_PATTERNS
            .iter()
            .any(|p| p[0] == pair[0] && p[1] == pair[1])
        {
            // Only oa, oe, uy are affected by modern/traditional debate
            let is_modern_pattern = matches!(
                (v1.key, v2.key),
                (keys::O, keys::A) | (keys::O, keys::E) | (keys::U, keys::Y)
            );
            if is_modern_pattern {
                return if modern { v2.pos } else { v1.pos };
            }
            // Other patterns (uê, iê, uô): always 2nd vowel
            return v2.pos;
        }

        // Default: 2nd vowel (unknown patterns)
        v2.pos
    }

    /// Find tone position for triphthongs (3 vowels)
    fn find_triphthong_position(vowels: &[Vowel]) -> usize {
        let (k0, k1, k2) = (vowels[0].key, vowels[1].key, vowels[2].key);

        // Rule 1: Pattern table lookup (takes priority)
        // Patterns define exact tone positions for known triphthongs
        for pattern in TRIPHTHONG_PATTERNS {
            if k0 == pattern.v1 && k1 == pattern.v2 && k2 == pattern.v3 {
                return match pattern.position {
                    TonePosition::First => vowels[0].pos,
                    TonePosition::Second => vowels[1].pos,
                    TonePosition::Last => vowels[2].pos,
                };
            }
        }

        // Rule 2: Diacritic priority (for unmatched patterns)
        // First with diacritic → first (ơi: ơ has diacritic, tone on ơ)
        if vowels[0].has_diacritic() {
            return vowels[0].pos;
        }
        // Middle with diacritic → middle (ươi: ơ has diacritic)
        if vowels[1].has_diacritic() {
            return vowels[1].pos;
        }
        // Last with diacritic → last (uyê: ê has diacritic)
        if vowels[2].has_diacritic() {
            return vowels[2].pos;
        }

        // Rule 3: For unrecognized triphthongs (like "aii", "oee"), fall back to
        // diphthong rules for the first two vowels. This handles cases where user
        // types extra vowels after a valid diphthong - the mark should stay on
        // the correct vowel for the original diphthong.
        // Example: "tài" + "i" → "tàii" (mark stays on 'a' because "ai" → first)
        let pair = [k0, k1];
        if TONE_FIRST_PATTERNS.contains(&pair) {
            return vowels[0].pos;
        }
        if TONE_SECOND_PATTERNS.contains(&pair) {
            return vowels[1].pos;
        }

        // Default: middle vowel (for truly unknown patterns)
        vowels[1].pos
    }

    /// Find tone position for 4+ vowels (rare cases)
    fn find_default_position(vowels: &[Vowel]) -> usize {
        let mid = vowels.len() / 2;

        // Priority: middle with diacritic
        if vowels[mid].has_diacritic() {
            return vowels[mid].pos;
        }

        // Then any vowel with diacritic
        for v in vowels {
            if v.has_diacritic() {
                return v.pos;
            }
        }

        // Fall back to diphthong rules for first two vowels
        // This handles cases like "tàiii" where the original diphthong "ai" should keep mark on 'a'
        if vowels.len() >= 2 {
            let pair = [vowels[0].key, vowels[1].key];
            if TONE_FIRST_PATTERNS.contains(&pair) {
                return vowels[0].pos;
            }
            if TONE_SECOND_PATTERNS.contains(&pair) {
                return vowels[1].pos;
            }
        }

        // Default: middle
        vowels[mid].pos
    }

    /// Find position(s) for horn modifier based on vowel patterns
    ///
    /// Uses HORN_PATTERNS array to match Vietnamese vowel pair patterns.
    /// Pattern matching is order-dependent (first match wins).
    ///
    /// Special "ua" handling (inferred from buffer context):
    /// - C+ua (mua, chua): horn on u → "mưa"
    /// - ua, qua: breve on a → "uă", "quă"
    pub fn find_horn_positions(buffer_keys: &[u16], vowel_positions: &[usize]) -> Vec<usize> {
        let mut result = Vec::new();
        let len = vowel_positions.len();

        if len == 0 {
            return result;
        }

        // Check adjacent vowel pairs against pattern table
        if len >= 2 {
            for i in 0..len - 1 {
                let pos1 = vowel_positions[i];
                let pos2 = vowel_positions[i + 1];

                // Must be adjacent buffer positions
                if pos2 != pos1 + 1 {
                    continue;
                }

                let k1 = buffer_keys.get(pos1).copied().unwrap_or(0);
                let k2 = buffer_keys.get(pos2).copied().unwrap_or(0);

                // Special case: "ua" pattern
                // - "qua" → Q is part of initial, so target A for breve → "quă"
                // - "ua" standalone → target U for horn → "ưa"
                // - "mua", "chua" → target U for horn → "mưa", "chưa"
                if k1 == keys::U && k2 == keys::A {
                    let preceded_by_q =
                        pos1 > 0 && buffer_keys.get(pos1 - 1).copied() == Some(keys::Q);

                    // Only apply breve to A when preceded by Q (qu-initial)
                    // Otherwise apply horn to U
                    result.push(if preceded_by_q { pos2 } else { pos1 });
                    return result;
                }

                // Match against pattern table
                for pattern in HORN_PATTERNS {
                    if k1 == pattern.v1 && k2 == pattern.v2 {
                        match pattern.placement {
                            HornPlacement::Both => {
                                // Check if 'u' is preceded by 'Q' (qu-initial consonant cluster)
                                // In "Qu-", the 'u' is part of the initial and should not get horn
                                // Examples: "Quoiws" → "Quới" (not "Qưới"), "quốc" (not "qước")
                                let preceded_by_q =
                                    pos1 > 0 && buffer_keys.get(pos1 - 1).copied() == Some(keys::Q);
                                if preceded_by_q {
                                    // Only apply horn to the second vowel
                                    result.push(pos2);
                                } else {
                                    // Issue #133: Check if "uo" pattern is at end of syllable (no final)
                                    // If no final consonant/vowel after "uo", only apply horn to 'o'
                                    // Examples: "huow" → "huơ", "khuow" → "khuơ"
                                    // But: "duowc" → "dược", "muowif" → "mười" (both get horn)
                                    let has_final = buffer_keys.get(pos2 + 1).is_some();
                                    if k1 == keys::U && k2 == keys::O && !has_final {
                                        // "uơ" pattern - only 'o' gets horn
                                        result.push(pos2);
                                    } else {
                                        // "ươ" pattern - both get horn
                                        result.push(pos1);
                                        result.push(pos2);
                                    }
                                }
                            }
                            HornPlacement::First => {
                                result.push(pos1);
                            }
                            HornPlacement::Second => {
                                result.push(pos2);
                            }
                        }
                        return result;
                    }
                }
            }
        }

        // Default: find last u or o (single vowel case)
        for &pos in vowel_positions.iter().rev() {
            let k = buffer_keys.get(pos).copied().unwrap_or(0);
            if k == keys::U || k == keys::O {
                // Check if this creates invalid vowel combination
                // "oe" pattern: horn on 'o' creates invalid "ơe" - skip this position
                if k == keys::O {
                    let next_key = buffer_keys.get(pos + 1).copied();
                    if next_key == Some(keys::E) {
                        continue; // Skip - "oe" doesn't accept horn on 'o'
                    }
                }
                result.push(pos);
                return result;
            }
        }

        // If no u/o, apply to last a (breve case in Telex)
        if let Some(&pos) = vowel_positions.last() {
            let k = buffer_keys.get(pos).copied().unwrap_or(0);
            if k == keys::A {
                result.push(pos);
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(key: u16, modifier: Modifier, pos: usize) -> Vowel {
        Vowel::new(key, modifier, pos)
    }

    #[test]
    fn test_single_vowel() {
        let vowels = vec![v(keys::A, Modifier::None, 0)];
        assert_eq!(
            Phonology::find_tone_position(&vowels, false, true, false, false),
            0
        );
    }

    #[test]
    fn test_medial_pairs() {
        // oa → mark on a (pos 1)
        let vowels = vec![v(keys::O, Modifier::None, 0), v(keys::A, Modifier::None, 1)];
        assert_eq!(
            Phonology::find_tone_position(&vowels, false, true, false, false),
            1
        );

        // uy → mark on y (pos 1)
        let vowels = vec![v(keys::U, Modifier::None, 0), v(keys::Y, Modifier::None, 1)];
        assert_eq!(
            Phonology::find_tone_position(&vowels, false, true, false, false),
            1
        );
    }

    #[test]
    fn test_ua_patterns() {
        // ua open syllable (mùa) → mark on u (pos 0)
        // u is main vowel, a is glide
        let vowels = vec![v(keys::U, Modifier::None, 0), v(keys::A, Modifier::None, 1)];
        assert_eq!(
            Phonology::find_tone_position(&vowels, false, true, false, false),
            0
        );

        // ua with final consonant (chuẩn) → mark on a (pos 1)
        // u is medial, a is main vowel
        let vowels = vec![v(keys::U, Modifier::None, 0), v(keys::A, Modifier::None, 1)];
        assert_eq!(
            Phonology::find_tone_position(&vowels, true, true, false, false),
            1,
            "ua with final consonant should mark on a (chuẩn)"
        );

        // ua with q (quà) → mark on a (pos 1)
        // u is part of qu consonant
        let vowels = vec![v(keys::U, Modifier::None, 0), v(keys::A, Modifier::None, 1)];
        assert_eq!(
            Phonology::find_tone_position(&vowels, false, true, true, false),
            1
        );
    }

    #[test]
    fn test_ia_pattern() {
        // ia (kìa, mía, lìa) → mark on i (pos 0)
        // Descending diphthong: i is main vowel, a is off-glide
        let vowels = vec![v(keys::I, Modifier::None, 0), v(keys::A, Modifier::None, 1)];
        assert_eq!(
            Phonology::find_tone_position(&vowels, false, true, false, false),
            0
        );

        // ia with gi initial (gia, giau) → mark on a (pos 1)
        // When gi is initial, 'i' is part of initial, 'a' is the only vowel
        let vowels = vec![v(keys::I, Modifier::None, 0), v(keys::A, Modifier::None, 1)];
        assert_eq!(
            Phonology::find_tone_position(&vowels, false, true, false, true),
            1
        );
    }

    #[test]
    fn test_main_glide_pairs() {
        // ai → mark on a (pos 0)
        let vowels = vec![v(keys::A, Modifier::None, 0), v(keys::I, Modifier::None, 1)];
        assert_eq!(
            Phonology::find_tone_position(&vowels, false, true, false, false),
            0
        );

        // ao → mark on a (pos 0)
        let vowels = vec![v(keys::A, Modifier::None, 0), v(keys::O, Modifier::None, 1)];
        assert_eq!(
            Phonology::find_tone_position(&vowels, false, true, false, false),
            0
        );
    }

    #[test]
    fn test_with_final_consonant() {
        // oan → mark on a (pos 1)
        let vowels = vec![v(keys::O, Modifier::None, 0), v(keys::A, Modifier::None, 1)];
        assert_eq!(
            Phonology::find_tone_position(&vowels, true, true, false, false),
            1
        );
    }

    #[test]
    fn test_compound_vowels() {
        // ươ → mark on ơ (pos 1)
        let vowels = vec![v(keys::U, Modifier::Horn, 0), v(keys::O, Modifier::Horn, 1)];
        assert_eq!(
            Phonology::find_tone_position(&vowels, false, true, false, false),
            1
        );

        // iê → mark on ê (pos 1)
        let vowels = vec![
            v(keys::I, Modifier::None, 0),
            v(keys::E, Modifier::Circumflex, 1),
        ];
        assert_eq!(
            Phonology::find_tone_position(&vowels, false, true, false, false),
            1
        );
    }

    #[test]
    fn test_three_vowels() {
        // ươi → mark on ơ (pos 1, middle with diacritic)
        let vowels = vec![
            v(keys::U, Modifier::Horn, 0),
            v(keys::O, Modifier::Horn, 1),
            v(keys::I, Modifier::None, 2),
        ];
        assert_eq!(
            Phonology::find_tone_position(&vowels, false, true, false, false),
            1
        );

        // oai → mark on a (pos 1, middle)
        let vowels = vec![
            v(keys::O, Modifier::None, 0),
            v(keys::A, Modifier::None, 1),
            v(keys::I, Modifier::None, 2),
        ];
        assert_eq!(
            Phonology::find_tone_position(&vowels, false, true, false, false),
            1
        );
    }

    #[test]
    fn test_diacritic_priority() {
        // ưa → mark on ư (pos 0, has diacritic)
        let vowels = vec![v(keys::U, Modifier::Horn, 0), v(keys::A, Modifier::None, 1)];
        // ưa is NOT a compound vowel (compound is ươ, not ưa)
        // ư has diacritic, a doesn't → mark on ư
        assert_eq!(
            Phonology::find_tone_position(&vowels, false, true, false, false),
            0
        );
    }
}
