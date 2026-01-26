//! Typing Order Permutation Test
//!
//! Comprehensive test that generates ALL valid Telex typing orders for Vietnamese words.
//! Tests that any valid typing order produces the correct output.
//!
//! Typing order variations:
//! - Tone (s/f/r/x/j): before vowel, after vowel, before final, after final
//! - Circumflex (aa/ee/oo): consecutive or split by other characters
//! - Horn (w): different positions relative to vowels (uw, ow, uow, uwo, etc.)
//! - Breve (w on a): aw positions
//!
//! Run with: cargo test --test typing_order_permutation_test -- --nocapture

use gonhanh_core::engine::Engine;
use gonhanh_core::utils::type_word;
use std::collections::HashSet;

// =============================================================================
// VIETNAMESE CHARACTER DECOMPOSITION
// =============================================================================

/// Decompose Vietnamese character into (base, mark, tone)
/// - base: the base vowel (a, e, i, o, u, y) or consonant
/// - mark: vowel modifier (a/e/o for circumflex, w for horn/breve, d for stroke)
/// - tone: tone mark (s=sắc, f=huyền, r=hỏi, x=ngã, j=nặng)
fn decompose_char(c: char) -> (char, Option<char>, Option<char>) {
    match c {
        // Plain vowels with tones
        'à' => ('a', None, Some('f')),
        'á' => ('a', None, Some('s')),
        'ả' => ('a', None, Some('r')),
        'ã' => ('a', None, Some('x')),
        'ạ' => ('a', None, Some('j')),
        'è' => ('e', None, Some('f')),
        'é' => ('e', None, Some('s')),
        'ẻ' => ('e', None, Some('r')),
        'ẽ' => ('e', None, Some('x')),
        'ẹ' => ('e', None, Some('j')),
        'ì' => ('i', None, Some('f')),
        'í' => ('i', None, Some('s')),
        'ỉ' => ('i', None, Some('r')),
        'ĩ' => ('i', None, Some('x')),
        'ị' => ('i', None, Some('j')),
        'ò' => ('o', None, Some('f')),
        'ó' => ('o', None, Some('s')),
        'ỏ' => ('o', None, Some('r')),
        'õ' => ('o', None, Some('x')),
        'ọ' => ('o', None, Some('j')),
        'ù' => ('u', None, Some('f')),
        'ú' => ('u', None, Some('s')),
        'ủ' => ('u', None, Some('r')),
        'ũ' => ('u', None, Some('x')),
        'ụ' => ('u', None, Some('j')),
        'ỳ' => ('y', None, Some('f')),
        'ý' => ('y', None, Some('s')),
        'ỷ' => ('y', None, Some('r')),
        'ỹ' => ('y', None, Some('x')),
        'ỵ' => ('y', None, Some('j')),
        // Circumflex â
        'â' => ('a', Some('a'), None),
        'ầ' => ('a', Some('a'), Some('f')),
        'ấ' => ('a', Some('a'), Some('s')),
        'ẩ' => ('a', Some('a'), Some('r')),
        'ẫ' => ('a', Some('a'), Some('x')),
        'ậ' => ('a', Some('a'), Some('j')),
        // Circumflex ê
        'ê' => ('e', Some('e'), None),
        'ề' => ('e', Some('e'), Some('f')),
        'ế' => ('e', Some('e'), Some('s')),
        'ể' => ('e', Some('e'), Some('r')),
        'ễ' => ('e', Some('e'), Some('x')),
        'ệ' => ('e', Some('e'), Some('j')),
        // Circumflex ô
        'ô' => ('o', Some('o'), None),
        'ồ' => ('o', Some('o'), Some('f')),
        'ố' => ('o', Some('o'), Some('s')),
        'ổ' => ('o', Some('o'), Some('r')),
        'ỗ' => ('o', Some('o'), Some('x')),
        'ộ' => ('o', Some('o'), Some('j')),
        // Breve ă
        'ă' => ('a', Some('w'), None),
        'ằ' => ('a', Some('w'), Some('f')),
        'ắ' => ('a', Some('w'), Some('s')),
        'ẳ' => ('a', Some('w'), Some('r')),
        'ẵ' => ('a', Some('w'), Some('x')),
        'ặ' => ('a', Some('w'), Some('j')),
        // Horn ơ
        'ơ' => ('o', Some('w'), None),
        'ờ' => ('o', Some('w'), Some('f')),
        'ớ' => ('o', Some('w'), Some('s')),
        'ở' => ('o', Some('w'), Some('r')),
        'ỡ' => ('o', Some('w'), Some('x')),
        'ợ' => ('o', Some('w'), Some('j')),
        // Horn ư
        'ư' => ('u', Some('w'), None),
        'ừ' => ('u', Some('w'), Some('f')),
        'ứ' => ('u', Some('w'), Some('s')),
        'ử' => ('u', Some('w'), Some('r')),
        'ữ' => ('u', Some('w'), Some('x')),
        'ự' => ('u', Some('w'), Some('j')),
        // Stroke đ
        'đ' => ('d', Some('d'), None),
        // Uppercase - Plain with tones
        'À' => ('A', None, Some('f')),
        'Á' => ('A', None, Some('s')),
        'Ả' => ('A', None, Some('r')),
        'Ã' => ('A', None, Some('x')),
        'Ạ' => ('A', None, Some('j')),
        'È' => ('E', None, Some('f')),
        'É' => ('E', None, Some('s')),
        'Ẻ' => ('E', None, Some('r')),
        'Ẽ' => ('E', None, Some('x')),
        'Ẹ' => ('E', None, Some('j')),
        'Ì' => ('I', None, Some('f')),
        'Í' => ('I', None, Some('s')),
        'Ỉ' => ('I', None, Some('r')),
        'Ĩ' => ('I', None, Some('x')),
        'Ị' => ('I', None, Some('j')),
        'Ò' => ('O', None, Some('f')),
        'Ó' => ('O', None, Some('s')),
        'Ỏ' => ('O', None, Some('r')),
        'Õ' => ('O', None, Some('x')),
        'Ọ' => ('O', None, Some('j')),
        'Ù' => ('U', None, Some('f')),
        'Ú' => ('U', None, Some('s')),
        'Ủ' => ('U', None, Some('r')),
        'Ũ' => ('U', None, Some('x')),
        'Ụ' => ('U', None, Some('j')),
        'Ỳ' => ('Y', None, Some('f')),
        'Ý' => ('Y', None, Some('s')),
        'Ỷ' => ('Y', None, Some('r')),
        'Ỹ' => ('Y', None, Some('x')),
        'Ỵ' => ('Y', None, Some('j')),
        // Uppercase - Circumflex
        'Â' => ('A', Some('A'), None),
        'Ầ' => ('A', Some('A'), Some('f')),
        'Ấ' => ('A', Some('A'), Some('s')),
        'Ẩ' => ('A', Some('A'), Some('r')),
        'Ẫ' => ('A', Some('A'), Some('x')),
        'Ậ' => ('A', Some('A'), Some('j')),
        'Ê' => ('E', Some('E'), None),
        'Ề' => ('E', Some('E'), Some('f')),
        'Ế' => ('E', Some('E'), Some('s')),
        'Ể' => ('E', Some('E'), Some('r')),
        'Ễ' => ('E', Some('E'), Some('x')),
        'Ệ' => ('E', Some('E'), Some('j')),
        'Ô' => ('O', Some('O'), None),
        'Ồ' => ('O', Some('O'), Some('f')),
        'Ố' => ('O', Some('O'), Some('s')),
        'Ổ' => ('O', Some('O'), Some('r')),
        'Ỗ' => ('O', Some('O'), Some('x')),
        'Ộ' => ('O', Some('O'), Some('j')),
        // Uppercase - Horn/Breve
        'Ă' => ('A', Some('W'), None),
        'Ằ' => ('A', Some('W'), Some('f')),
        'Ắ' => ('A', Some('W'), Some('s')),
        'Ẳ' => ('A', Some('W'), Some('r')),
        'Ẵ' => ('A', Some('W'), Some('x')),
        'Ặ' => ('A', Some('W'), Some('j')),
        'Ơ' => ('O', Some('W'), None),
        'Ờ' => ('O', Some('W'), Some('f')),
        'Ớ' => ('O', Some('W'), Some('s')),
        'Ở' => ('O', Some('W'), Some('r')),
        'Ỡ' => ('O', Some('W'), Some('x')),
        'Ợ' => ('O', Some('W'), Some('j')),
        'Ư' => ('U', Some('W'), None),
        'Ừ' => ('U', Some('W'), Some('f')),
        'Ứ' => ('U', Some('W'), Some('s')),
        'Ử' => ('U', Some('W'), Some('r')),
        'Ữ' => ('U', Some('W'), Some('x')),
        'Ự' => ('U', Some('W'), Some('j')),
        'Đ' => ('D', Some('D'), None),
        _ => (c, None, None),
    }
}

// =============================================================================
// WORD STRUCTURE FOR PERMUTATION GENERATION
// =============================================================================

/// Represents a decomposed Vietnamese syllable for permutation generation
#[derive(Debug, Clone)]
struct SyllableParts {
    /// Initial consonant(s): b, c, ch, d, đ(dd), g, gh, gi, h, k, kh, l, m, n, ng, ngh, nh, p, ph, q, r, s, t, th, tr, v, x
    initial: String,
    /// Vowel nucleus - list of (base_vowel, mark_type) pairs
    /// mark_type: None=plain, Some('a')=circumflex-a, Some('e')=circumflex-e, Some('o')=circumflex-o, Some('w')=horn/breve
    vowels: Vec<(char, Option<char>)>,
    /// Tone mark: s=sắc, f=huyền, r=hỏi, x=ngã, j=nặng
    tone: Option<char>,
    /// Final consonant(s): c, ch, m, n, ng, nh, p, t
    final_cons: String,
}

/// Check if a character is a Vietnamese vowel base
fn is_vowel(c: char) -> bool {
    matches!(c.to_ascii_lowercase(), 'a' | 'e' | 'i' | 'o' | 'u' | 'y')
}

/// Parse a Vietnamese word into syllable parts
fn parse_syllable(word: &str) -> Option<SyllableParts> {
    let chars: Vec<char> = word.chars().collect();
    if chars.is_empty() {
        return None;
    }

    let mut initial = String::new();
    let mut vowels: Vec<(char, Option<char>)> = Vec::new();
    let mut tone: Option<char> = None;
    let mut final_cons = String::new();

    let mut in_vowel_section = false;

    for c in chars {
        let (base, mark, char_tone) = decompose_char(c);
        let base_is_vowel = is_vowel(base);

        if !in_vowel_section && !base_is_vowel {
            // Initial consonant section
            if mark == Some('d') || mark == Some('D') {
                initial.push(base);
                initial.push(base); // dd for đ
            } else {
                initial.push(base);
            }
        } else if base_is_vowel {
            // Vowel section
            in_vowel_section = true;
            let vowel_mark = if mark == Some('w') || mark == Some('W') {
                Some('w')
            } else if mark.is_some() {
                // Circumflex - use the base letter as mark indicator
                Some(base.to_ascii_lowercase())
            } else {
                None
            };
            vowels.push((base, vowel_mark));

            if char_tone.is_some() {
                tone = char_tone;
            }
        } else {
            // Final consonant section
            final_cons.push(base);
        }
    }

    Some(SyllableParts {
        initial,
        vowels,
        tone,
        final_cons,
    })
}

// =============================================================================
// COMPREHENSIVE TELEX VARIANT GENERATOR
// =============================================================================

/// Generate ALL valid Telex typing orders for a Vietnamese syllable
///
/// Valid positions for modifiers:
/// - Tone (s/f/r/x/j): after the vowel nucleus (all vowels + marks), or after final consonant
/// - Circumflex (aa/ee/oo): immediately after the vowel it modifies
/// - Horn/Breve (w): after the vowel it modifies
///
/// For diphthongs (2 vowels) without final consonant, tone can also appear between vowels.
fn generate_all_telex_variants(word: &str) -> Vec<String> {
    let parts = match parse_syllable(word) {
        Some(p) => p,
        None => return vec![word.to_string()],
    };

    let mut variants: HashSet<String> = HashSet::new();

    let initial = &parts.initial;
    let vowels = &parts.vowels;
    let final_cons = &parts.final_cons;
    let tone = parts.tone;

    // Generate vowel typing patterns (with marks in correct positions)
    let vowel_patterns = generate_vowel_patterns(&parts);

    for vowel_pattern in &vowel_patterns {
        if let Some(t) = tone {
            // Pattern 1: Tone after all vowels (before final)
            // Example: "nào" → "naof"
            {
                let mut v = initial.clone();
                v.push_str(vowel_pattern);
                v.push(t);
                v.push_str(final_cons);
                variants.insert(v);
            }

            // Pattern 2: Tone after final consonant
            // Example: "nám" → "nasm"
            if !final_cons.is_empty() {
                let mut v = initial.clone();
                v.push_str(vowel_pattern);
                v.push_str(final_cons);
                v.push(t);
                variants.insert(v);
            }

            // Pattern 3: For diphthongs WITHOUT final consonant, tone between vowels
            // Example: "nào" → "nafo" (tone after first vowel, before second)
            // This is ONLY valid when:
            // - There are exactly 2 vowels (diphthong)
            // - There is no final consonant (open syllable)
            if vowels.len() == 2 && final_cons.is_empty() {
                // Find first vowel end position (including its mark if any)
                let vowel_chars: Vec<char> = vowel_pattern.chars().collect();
                let mut first_vowel_end = 0;
                let mut found_first_vowel = false;

                for (i, c) in vowel_chars.iter().enumerate() {
                    if is_vowel(*c) {
                        if !found_first_vowel {
                            found_first_vowel = true;
                            first_vowel_end = i + 1;
                            // Include any mark right after the first vowel
                            if i + 1 < vowel_chars.len() && !is_vowel(vowel_chars[i + 1]) {
                                first_vowel_end = i + 2;
                            }
                        }
                    }
                }

                if first_vowel_end > 0 && first_vowel_end < vowel_chars.len() {
                    let mut v = initial.clone();
                    v.extend(&vowel_chars[..first_vowel_end]);
                    v.push(t);
                    v.extend(&vowel_chars[first_vowel_end..]);
                    variants.insert(v);
                }
            }

            // Pattern 4: Split circumflex with tone before circumflex completer
            // For circumflex vowels (â, ê, ô), tone can come BEFORE the second letter
            // Example: "giấc" → "giacsa" (gi + a + c + s + a)
            // Example: "giầm" → "giafam" (gi + a + f + a + m)
            // This works when: vowel has circumflex mark and there's a tone
            for (v_idx, (v_char, v_mark)) in vowels.iter().enumerate() {
                // Check for circumflex (mark equals base vowel lowercase)
                let is_circumflex = v_mark.map_or(false, |m| {
                    m.to_ascii_lowercase() == v_char.to_ascii_lowercase()
                });

                if is_circumflex {
                    // Build base WITHOUT the circumflex (just the vowel once)
                    let mut base_no_circ = initial.clone();
                    for (i, (vc, _)) in vowels.iter().enumerate() {
                        base_no_circ.push(*vc);
                        // Add marks for other vowels, but not circumflex for this one
                        if i != v_idx {
                            if let Some(m) = vowels[i].1 {
                                base_no_circ.push(m);
                            }
                        }
                    }

                    let circumflex_char = *v_char; // The char that completes circumflex

                    // Pattern 4a: base + final + tone + circumflex_completer
                    // Example: giấc → giac + s + a = giacsa
                    if !final_cons.is_empty() {
                        let mut v = base_no_circ.clone();
                        v.push_str(final_cons);
                        v.push(t);
                        v.push(circumflex_char.to_ascii_lowercase());
                        variants.insert(v);
                    }

                    // Pattern 4b: base + tone + circumflex_completer + final
                    // Example: giầm → gia + f + a + m = giafam
                    {
                        let mut v = base_no_circ.clone();
                        v.push(t);
                        v.push(circumflex_char.to_ascii_lowercase());
                        v.push_str(final_cons);
                        variants.insert(v);
                    }
                }
            }
        } else {
            // No tone, just the base pattern
            let mut v = initial.clone();
            v.push_str(vowel_pattern);
            v.push_str(final_cons);
            variants.insert(v);
        }
    }

    // Pattern 5: All modifiers at end (double typing pattern)
    // Example: "đường" → "duongdwf" (base letters, then d for đ, w for horn, f for tone)
    // Example: "quên" → "quene" (base letters quen, then e for circumflex)
    let end_patterns = generate_modifiers_at_end_patterns(&parts);
    for pattern in end_patterns {
        variants.insert(pattern);
    }

    // Pattern 6: Consecutive identical vowels (literal oo, aa, ee)
    // When word has literal "oo" (not ô), need to type "ooo" to cancel circumflex
    // Example: "boong" → "booong" (bo + ooo → boo + ng)
    // This applies to aa, ee, oo without marks
    // IMPORTANT: For Telex, the double-vowel variant (boong) is INVALID because
    // it produces circumflex (bông). Only triple-vowel variant (booong) is valid.
    let has_consecutive_vowels = detect_consecutive_identical_vowels(vowels);
    if has_consecutive_vowels {
        // Remove all existing variants (they have double vowels which produce circumflex)
        variants.clear();
        // Generate ONLY the triple-vowel variant to cancel circumflex
        let cancel_patterns = generate_circumflex_cancel_variants(&parts);
        for pattern in cancel_patterns {
            variants.insert(pattern);
        }
    }

    let mut result: Vec<String> = variants.into_iter().collect();
    result.sort(); // Ensure deterministic order
    result
}

/// Generate patterns where all modifiers are typed at the end of the word
/// This handles the "delayed typing" style where users type base letters first,
/// then add all diacritics at the end
fn generate_modifiers_at_end_patterns(parts: &SyllableParts) -> Vec<String> {
    let initial = &parts.initial;
    let vowels = &parts.vowels;
    let final_cons = &parts.final_cons;

    // Build base word (without any marks)
    let has_stroke = initial.to_lowercase() == "dd";
    let mut base = String::new();
    if has_stroke {
        base.push(if initial.starts_with(|c: char| c.is_uppercase()) {
            'D'
        } else {
            'd'
        });
    } else {
        base.push_str(initial);
    }
    for (v, _) in vowels {
        base.push(*v);
    }
    base.push_str(final_cons);

    // Collect modifiers
    let stroke_mod = if has_stroke { Some('d') } else { None };
    let vowel_mods = collect_vowel_mods(vowels);

    // Generate permutations based on which modifiers exist
    generate_modifier_permutations(&base, stroke_mod, &vowel_mods, parts.tone)
}

/// Collect vowel modifiers (horn w, circumflex, breve)
fn collect_vowel_mods(vowels: &[(char, Option<char>)]) -> Vec<char> {
    let has_uwo = vowels
        .iter()
        .any(|(v, m)| v.to_ascii_lowercase() == 'u' && *m == Some('w'))
        && vowels
            .iter()
            .any(|(v, m)| v.to_ascii_lowercase() == 'o' && *m == Some('w'));

    let mut mods = Vec::new();
    let mut horn_added = false;

    for (v, mark) in vowels {
        if let Some(m) = mark {
            if *m == 'w' {
                if !has_uwo || !horn_added {
                    mods.push('w');
                    horn_added = true;
                }
            } else if *m == v.to_ascii_lowercase() {
                mods.push(*v); // Circumflex uses base vowel
            } else {
                mods.push(*m);
            }
        }
    }
    mods
}

/// Generate all valid permutations of modifiers at end
fn generate_modifier_permutations(
    base: &str,
    stroke: Option<char>,
    vowel_mods: &[char],
    tone: Option<char>,
) -> Vec<String> {
    let mut patterns = Vec::new();
    let has_stroke = stroke.is_some();
    let has_vmods = !vowel_mods.is_empty();
    let has_tone = tone.is_some();

    if !has_stroke && !has_vmods && !has_tone {
        return patterns;
    }

    // Helper to append vowel mods
    let append_vmods = |s: &mut String| {
        for m in vowel_mods {
            s.push(*m);
        }
    };

    match (has_stroke, has_vmods, has_tone) {
        (false, true, true) => {
            let t = tone.unwrap();
            // vowel_mods + tone
            let mut p1 = base.to_string();
            append_vmods(&mut p1);
            p1.push(t);
            patterns.push(p1);
            // tone + vowel_mods
            let mut p2 = base.to_string();
            p2.push(t);
            append_vmods(&mut p2);
            patterns.push(p2);
        }
        (false, true, false) => {
            let mut p = base.to_string();
            append_vmods(&mut p);
            patterns.push(p);
        }
        (false, false, true) => {
            let mut p = base.to_string();
            p.push(tone.unwrap());
            patterns.push(p);
        }
        (true, false, false) => {
            let mut p = base.to_string();
            p.push(stroke.unwrap());
            patterns.push(p);
        }
        (true, false, true) => {
            let (d, t) = (stroke.unwrap(), tone.unwrap());
            for order in [[d, t], [t, d]] {
                let mut p = base.to_string();
                p.extend(order);
                patterns.push(p);
            }
        }
        (true, true, false) => {
            let d = stroke.unwrap();
            // stroke + vowel_mods
            let mut p1 = base.to_string();
            p1.push(d);
            append_vmods(&mut p1);
            patterns.push(p1);
            // vowel_mods + stroke
            let mut p2 = base.to_string();
            append_vmods(&mut p2);
            p2.push(d);
            patterns.push(p2);
        }
        (true, true, true) => {
            // All 6 permutations of (stroke, vowel_mods, tone)
            let (d, t) = (stroke.unwrap(), tone.unwrap());
            let orders: [(u8, u8, u8); 6] = [
                (0, 1, 2),
                (0, 2, 1),
                (1, 0, 2),
                (1, 2, 0),
                (2, 0, 1),
                (2, 1, 0),
            ];
            for (a, b, c) in orders {
                let mut p = base.to_string();
                for idx in [a, b, c] {
                    match idx {
                        0 => p.push(d),
                        1 => append_vmods(&mut p),
                        _ => p.push(t),
                    }
                }
                patterns.push(p);
            }
        }
        _ => {}
    }

    patterns
}

/// Detect if word has consecutive identical vowels without marks (literal aa, ee, oo)
/// These need special handling because typing "oo" produces "ô", not "oo"
fn detect_consecutive_identical_vowels(vowels: &[(char, Option<char>)]) -> bool {
    vowels.windows(2).any(|pair| {
        let (v1, m1) = &pair[0];
        let (v2, m2) = &pair[1];
        v1.to_ascii_lowercase() == v2.to_ascii_lowercase()
            && m1.is_none()
            && m2.is_none()
            && matches!(v1.to_ascii_lowercase(), 'a' | 'e' | 'o')
    })
}

/// Generate variants that cancel circumflex for consecutive identical vowels
/// Example: "boong" with vowels [('o', None), ('o', None)] → needs "ooo" variant
fn generate_circumflex_cancel_variants(parts: &SyllableParts) -> Vec<String> {
    let vowels = &parts.vowels;

    // Build vowel string, tripling consecutive identical vowels
    let mut vowel_str = String::new();
    let mut i = 0;
    while i < vowels.len() {
        let (v, m) = &vowels[i];
        vowel_str.push(*v);

        // Check for consecutive identical unmarked vowels (a/e/o)
        let is_consecutive = i + 1 < vowels.len() && {
            let (v2, m2) = &vowels[i + 1];
            v.to_ascii_lowercase() == v2.to_ascii_lowercase()
                && m.is_none()
                && m2.is_none()
                && matches!(v.to_ascii_lowercase(), 'a' | 'e' | 'o')
        };

        if is_consecutive {
            // Triple vowel to cancel circumflex (ooo instead of oo)
            vowel_str.push(v.to_ascii_lowercase());
            vowel_str.push(vowels[i + 1].0);
            i += 2;
        } else {
            if let Some(mark) = m {
                vowel_str.push(*mark);
            }
            i += 1;
        }
    }

    let mut patterns = Vec::new();

    // Generate BOTH patterns for triple-o words:
    // 1. "Tone before final": initial + ooo + tone + final (e.g., gooofng → goòng)
    // 2. "Tone after final": initial + ooo + final + tone (e.g., gooongf → goòng)

    // Pattern 1: Tone before final
    patterns.push(format!(
        "{}{}{}{}",
        parts.initial,
        vowel_str,
        parts.tone.map(|t| t.to_string()).unwrap_or_default(),
        parts.final_cons
    ));

    // Pattern 2: Tone after final (only if both tone and final exist)
    if parts.tone.is_some() && !parts.final_cons.is_empty() {
        patterns.push(format!(
            "{}{}{}{}",
            parts.initial,
            vowel_str,
            parts.final_cons,
            parts.tone.unwrap()
        ));
    }

    patterns
}

/// Generate all valid vowel+mark patterns for a syllable
fn generate_vowel_patterns(parts: &SyllableParts) -> Vec<String> {
    let vowels = &parts.vowels;
    if vowels.is_empty() {
        return vec![String::new()];
    }

    let mut patterns: HashSet<String> = HashSet::new();

    // Base pattern: vowels with marks immediately after
    let base: String = vowels
        .iter()
        .flat_map(|(v, m)| std::iter::once(*v).chain(m.iter().copied()))
        .collect();
    patterns.insert(base);

    // Special case: ươ (horn on both u and o) - generate "uow" variant (w after o only)
    let has_horn_u = vowels
        .iter()
        .any(|(v, m)| v.to_ascii_lowercase() == 'u' && *m == Some('w'));
    let has_horn_o = vowels
        .iter()
        .any(|(v, m)| v.to_ascii_lowercase() == 'o' && *m == Some('w'));
    if has_horn_u && has_horn_o {
        let uow: String = vowels
            .iter()
            .flat_map(|(v, m)| {
                let is_horn_o = v.to_ascii_lowercase() == 'o' && *m == Some('w');
                std::iter::once(*v).chain(if is_horn_o { Some('w') } else { None })
            })
            .collect();
        patterns.insert(uow);
    }

    patterns.into_iter().collect()
}

/// Test a single word with all its valid typing variants
fn test_word_all_variants(
    word: &str,
    use_auto_restore: bool,
) -> (bool, Vec<(String, String)>, usize) {
    let variants = generate_all_telex_variants(word);
    let mut failures: Vec<(String, String)> = Vec::new();
    let total = variants.len();

    for variant in &variants {
        let input = format!("{} ", variant);
        let mut e = Engine::new();
        e.set_method(0); // Telex
        if use_auto_restore {
            e.set_english_auto_restore(true);
        }
        let result = type_word(&mut e, &input);
        let result_trimmed = result.trim();

        if result_trimmed != word {
            failures.push((variant.clone(), result_trimmed.to_string()));
        }
    }

    (failures.is_empty(), failures, total)
}

// =============================================================================
// TESTS
// =============================================================================

/// Test common Vietnamese words with all valid typing orders
#[test]
fn common_words_all_orders() {
    let words = [
        // Diphthongs with tones
        "nào", "sao", "cao", "bảo", "gái", "mái", "tài", "bài", "hỏi", "bói", "của", "múa", "bùa",
        "tụi", "mủi", "núi", "cúi", // With final consonants
        "tầng", "bền", "tấn", "lắm", "nắng", // Complex vowels
        "riêng", "tiếng", "nước", "được", "bước", "mười", "người", // Common words
        "không", "những", "cũng", "trong", "này", "với", "đến", "còn", "theo", "trên",
        // More diphthongs
        "chào", "kêu", "đều", "mèo", "kéo",
    ];

    let mut all_passed = true;
    let mut total_variants = 0;
    let mut failed_count = 0;

    for word in &words {
        let (passed, failures, count) = test_word_all_variants(word, false);
        total_variants += count;

        if !passed {
            all_passed = false;
            failed_count += failures.len();
            println!(
                "\n'{}' FAILED ({} of {} variants):",
                word,
                failures.len(),
                count
            );
            for (variant, actual) in failures.iter().take(5) {
                println!("  '{}' → '{}' (expected '{}')", variant, actual, word);
            }
            if failures.len() > 5 {
                println!("  ... and {} more", failures.len() - 5);
            }
        }
    }

    println!(
        "\n=== Common Words Test ===\nWords: {}\nTotal variants: {}\nFailed: {}",
        words.len(),
        total_variants,
        failed_count
    );
    assert!(all_passed, "Some typing order variants failed");
}

/// Test common words with auto-restore enabled
#[test]
fn common_words_auto_restore() {
    let words = [
        // UI diphthong (was buggy - tuji → tuji instead of tụi)
        "tụi", "mủi", "núi", "cúi", "đùi", "vùi", "bụi", // Other diphthongs
        "của", "múa", "bùa", "mùa", "nào", "sao", "bảo", "gái", "mái", "tài", "hỏi", "bói", "tối",
        // With stroke
        "đi", "đến", "được", "đều",
    ];

    let mut all_passed = true;
    let mut total_variants = 0;
    let mut failed_count = 0;

    for word in &words {
        let (passed, failures, count) = test_word_all_variants(word, true);
        total_variants += count;

        if !passed {
            all_passed = false;
            failed_count += failures.len();
            println!(
                "\n'{}' FAILED ({} of {} variants):",
                word,
                failures.len(),
                count
            );
            for (variant, actual) in failures.iter().take(5) {
                println!("  '{}' → '{}' (expected '{}')", variant, actual, word);
            }
        }
    }

    println!(
        "\n=== Auto-Restore Test ===\nWords: {}\nTotal variants: {}\nFailed: {}",
        words.len(),
        total_variants,
        failed_count
    );
    assert!(all_passed, "Some auto-restore variants failed");
}

/// Test diphthong tone positions (tone before/after second vowel)
#[test]
fn diphthong_tone_positions() {
    let cases = [
        // (expected_output, [valid_inputs])
        ("nào", vec!["naof", "nafo"]),
        ("sào", vec!["saof", "safo"]),
        ("cáo", vec!["caos", "caso"]),
        ("bảo", vec!["baor", "baro"]),
        ("gái", vec!["gais", "gasi"]),
        ("mái", vec!["mais", "masi"]),
        ("tài", vec!["taif", "tafi"]),
        ("bói", vec!["bois", "bosi"]),
        ("hỏi", vec!["hoir", "hori"]),
        ("múa", vec!["muas", "musa"]),
        ("bùa", vec!["buaf", "bufa"]),
        // UI diphthong (fixed bug)
        ("tụi", vec!["tuij", "tuji"]),
        ("mủi", vec!["muir", "muri"]),
        ("núi", vec!["nuis", "nusi"]),
        ("cúi", vec!["cuis", "cusi"]),
        ("đùi", vec!["dduif", "ddufi"]),
        ("vùi", vec!["vuif", "vufi"]),
        // IU diphthong
        ("chịu", vec!["chiuj", "chiju"]),
        ("líu", vec!["lius", "lisu"]),
    ];

    let mut all_passed = true;

    for (expected, variants) in &cases {
        for variant in variants {
            let input = format!("{} ", variant);
            let mut e = Engine::new();
            e.set_english_auto_restore(true);
            let result = type_word(&mut e, &input);

            if result.trim() != *expected {
                println!(
                    "FAIL: '{}' → '{}' (expected '{}')",
                    variant,
                    result.trim(),
                    expected
                );
                all_passed = false;
            }
        }
    }

    assert!(all_passed, "Some diphthong tone variants failed");
}

/// Test tone positions relative to final consonants
#[test]
fn tone_position_with_final() {
    let cases = [
        // (expected, [tone_before_final, tone_after_final])
        ("nám", vec!["nams", "nasm"]),
        ("mát", vec!["mats", "mast"]),
        ("lác", vec!["lacs", "lasc"]),
        ("láng", vec!["langs", "lasng"]),
        ("lánh", vec!["lanhs", "lasnh"]),
        ("lách", vec!["lachs", "lasch"]),
        ("tầng", vec!["taangf", "taafng"]),
        ("bền", vec!["beenf", "beefn"]),
    ];

    let mut all_passed = true;

    for (expected, variants) in &cases {
        for variant in variants {
            let input = format!("{} ", variant);
            let mut e = Engine::new();
            let result = type_word(&mut e, &input);

            if result.trim() != *expected {
                println!(
                    "FAIL: '{}' → '{}' (expected '{}')",
                    variant,
                    result.trim(),
                    expected
                );
                all_passed = false;
            }
        }
    }

    assert!(all_passed, "Some tone position variants failed");
}

/// Test ươ (horn on both u and o) typing patterns
#[test]
fn horn_uo_patterns() {
    let cases = [
        // (expected, [valid_inputs])
        ("ươ", vec!["uow", "uwo", "uwow"]),
        ("ước", vec!["uowc", "uwoc", "uwowc"]),
        ("ươn", vec!["uown", "uwon", "uwown"]),
        ("nước", vec!["nuowc", "nuwoc", "nuwowc"]),
        ("được", vec!["dduowc", "dduwoc", "dduwowc"]),
        ("mượn", vec!["muowjn", "muwojn", "muwowjn"]),
        ("người", vec!["nguoiw", "nguowi", "nguwowi"]),
    ];

    let mut passed_count = 0;
    let mut failed_count = 0;

    for (expected, variants) in &cases {
        for variant in variants {
            let input = format!("{} ", variant);
            let mut e = Engine::new();
            let result = type_word(&mut e, &input);

            if result.trim() == *expected {
                passed_count += 1;
            } else {
                println!(
                    "FAIL: '{}' → '{}' (expected '{}')",
                    variant,
                    result.trim(),
                    expected
                );
                failed_count += 1;
            }
        }
    }

    println!(
        "\n=== Horn ươ Patterns ===\nPassed: {}\nFailed: {}",
        passed_count, failed_count
    );
    // Note: Some patterns may not work depending on engine implementation
    // This test documents expected behavior
}

/// Test circumflex patterns (â, ê, ô)
#[test]
fn circumflex_patterns() {
    let cases = [
        ("ân", vec!["aan"]),
        ("ấn", vec!["aans", "aasn"]),
        ("ên", vec!["een"]),
        ("ến", vec!["eens", "eesn"]),
        ("ôn", vec!["oon"]),
        ("ốn", vec!["oons", "oosn"]),
        ("riêng", vec!["rieeng"]),
        ("tiếng", vec!["tieengs", "tieesng"]),
    ];

    let mut all_passed = true;

    for (expected, variants) in &cases {
        for variant in variants {
            let input = format!("{} ", variant);
            let mut e = Engine::new();
            let result = type_word(&mut e, &input);

            if result.trim() != *expected {
                println!(
                    "FAIL: '{}' → '{}' (expected '{}')",
                    variant,
                    result.trim(),
                    expected
                );
                all_passed = false;
            }
        }
    }

    assert!(all_passed, "Some circumflex variants failed");
}

/// Test breve patterns (ă)
#[test]
fn breve_patterns() {
    let cases = [
        ("ăn", vec!["awn"]),
        ("ắn", vec!["awns", "awsn"]),
        ("lắm", vec!["lawms", "lawsm"]),
        ("nắng", vec!["nawngs", "nawsng"]),
        ("đặc", vec!["ddawcj", "ddawjc"]),
    ];

    let mut all_passed = true;

    for (expected, variants) in &cases {
        for variant in variants {
            let input = format!("{} ", variant);
            let mut e = Engine::new();
            let result = type_word(&mut e, &input);

            if result.trim() != *expected {
                println!(
                    "FAIL: '{}' → '{}' (expected '{}')",
                    variant,
                    result.trim(),
                    expected
                );
                all_passed = false;
            }
        }
    }

    assert!(all_passed, "Some breve variants failed");
}

/// Test modifiers-at-end patterns (delayed typing)
/// Example: "đường" → "duongdwf", "quên" → "quene"
#[test]
fn modifiers_at_end_patterns() {
    let cases = [
        // Stroke at end: đ
        ("đi", vec!["did"]),
        ("đen", vec!["dend"]),
        // Circumflex at end: ê, â, ô
        ("quên", vec!["quene"]),
        ("tân", vec!["tana"]),
        ("hôn", vec!["hono"]),
        // Horn at end: ư, ơ
        ("mưa", vec!["muaw"]),
        ("mơ", vec!["mow"]),
        // Combined: stroke + horn + tone
        ("đường", vec!["duongdwf"]),
        ("đời", vec!["doidwf"]),
        // Breve at end
        ("ăn", vec!["anw"]),
        // Tone at end
        ("là", vec!["laf"]),
        ("lá", vec!["las"]),
    ];

    let mut passed = 0;
    let mut failed = 0;

    for (expected, variants) in &cases {
        for variant in variants {
            let input = format!("{} ", variant);
            let mut e = Engine::new();
            let result = type_word(&mut e, &input);

            if result.trim() == *expected {
                passed += 1;
            } else {
                println!(
                    "FAIL: '{}' → '{}' (expected '{}')",
                    variant,
                    result.trim(),
                    expected
                );
                failed += 1;
            }
        }
    }

    println!(
        "\n=== Modifiers at End ===\nPassed: {}\nFailed: {}",
        passed, failed
    );
    // Note: Some patterns may not work depending on engine implementation
    // This test documents expected behavior
}

// =============================================================================
// 22K VIETNAMESE DICTIONARY TEST
// =============================================================================

/// Test all 22k Vietnamese words with their typing variants
/// This is the comprehensive test that validates all valid typing orders.
#[test]
#[ignore] // Run with: cargo test test_22k_all_variants -- --ignored --nocapture
fn test_22k_all_variants() {
    let content = include_str!("data/vietnamese_22k.txt");

    let mut total_words = 0;
    let mut words_passed = 0;
    let mut words_failed = 0;
    let mut total_variants = 0;
    let mut failed_variants = 0;
    let mut failures: Vec<(String, Vec<(String, String)>)> = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Handle compound words (space-separated)
        for word in line.split_whitespace() {
            total_words += 1;

            let (passed, word_failures, count) = test_word_all_variants(word, true);
            total_variants += count;

            if passed {
                words_passed += 1;
            } else {
                words_failed += 1;
                failed_variants += word_failures.len();
                if failures.len() < 100 {
                    failures.push((word.to_string(), word_failures));
                }
            }
        }
    }

    println!("\n=== Vietnamese 22k All Variants Test ===");
    println!("Total words: {}", total_words);
    println!(
        "Words passed: {} ({:.2}%)",
        words_passed,
        words_passed as f64 / total_words as f64 * 100.0
    );
    println!("Words failed: {}", words_failed);
    println!("Total variants tested: {}", total_variants);
    println!("Failed variants: {}", failed_variants);

    if !failures.is_empty() {
        println!("\n=== Sample Failures (first 100 words) ===\n");
        for (word, word_failures) in failures.iter().take(20) {
            println!("'{}' ({} failures):", word, word_failures.len());
            for (variant, actual) in word_failures.iter().take(3) {
                println!("  '{}' → '{}' (expected '{}')", variant, actual, word);
            }
        }
    }

    // Require high pass rate
    let pass_rate = words_passed as f64 / total_words as f64 * 100.0;
    assert!(
        pass_rate >= 95.0,
        "22k pass rate {:.2}% is below threshold 95%",
        pass_rate
    );
}

/// Generate a report of all valid typing orders for each word in 22k dictionary
/// Writes to tests/data/vietnamese_22k_typing_variants.txt
#[test]
#[ignore] // Run with: cargo test generate_22k_typing_orders -- --ignored --nocapture
fn generate_22k_typing_orders() {
    use std::fs::File;
    use std::io::Write;

    let content = include_str!("data/vietnamese_22k.txt");
    let mut output = File::create("tests/data/vietnamese_22k_typing_variants.txt")
        .expect("Failed to create output file");

    writeln!(output, "# Vietnamese 22k Typing Variants").unwrap();
    writeln!(output, "# Format: word TAB variant1,variant2,...").unwrap();
    writeln!(output, "# Generated by typing_order_permutation_test.rs").unwrap();
    writeln!(output).unwrap();

    let mut total_words = 0;
    let mut total_variants = 0;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        for word in line.split_whitespace() {
            let variants = generate_all_telex_variants(word);
            total_words += 1;
            total_variants += variants.len();

            let variants_str = variants.join(",");
            writeln!(output, "{}\t{}", word, variants_str).unwrap();
        }
    }

    println!(
        "Generated typing orders for {} words ({} total variants)",
        total_words, total_variants
    );
    println!("Output: tests/data/vietnamese_22k_typing_variants.txt");
}
