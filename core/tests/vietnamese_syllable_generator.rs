//! Vietnamese Syllable Generator
//!
//! Generates all valid Vietnamese syllables from phonotactic rules in constants.rs
//! This ensures the syllable list is consistent with the engine's validation rules.

use std::collections::HashSet;
use std::fs::File;
use std::io::Write;

// Phonotactic components based on constants.rs and vietnamese-language-system.md

/// Single initial consonants (16)
const INITIALS_1: &[&str] = &[
    "b", "c", "d", "g", "h", "k", "l", "m", "n", "p", "q", "r", "s", "t", "v", "x",
];

/// Double initial consonants (11)
const INITIALS_2: &[&str] = &[
    "ch", "gh", "gi", "kh", "kr", "ng", "nh", "ph", "qu", "th", "tr",
];

/// Triple initial consonant (1)
const INITIALS_3: &[&str] = &["ngh"];

/// Single vowels (12)
const SINGLE_VOWELS: &[&str] = &["a", "ă", "â", "e", "ê", "i", "o", "ô", "ơ", "u", "ư", "y"];

/// Valid diphthongs (27) - from VALID_DIPHTHONGS in constants.rs
/// Including modifier variants (âu, ây, ôi, ơi, ưa, ưi, ươ, ưu)
const DIPHTHONGS: &[&str] = &[
    // A combinations
    "ai", "ao", "au", "ay", "âu", "ây", // E combinations
    "eo", "êu", // I combinations
    "ia", "iê", "iu", // O combinations
    "oa", "oă", "oe", "oi", "ôi", "ơi", // U combinations
    "ua", "uâ", "uê", "ui", "uô", "uy", "ưa", "ưi", "ươ", "ưu", // Y combinations
    "yê", // Special
    "uo",
];

/// Valid triphthongs (13) - from VALID_TRIPHTHONGS in constants.rs
const TRIPHTHONGS: &[&str] = &[
    "iêu", "yêu", "oai", "oay", "oeo", "uây", "uôi", "ươi", "uya", "ươu", "uyê", "uyu", "uêu",
    "oao",
];

/// Single final consonants (10)
const FINALS_1: &[&str] = &["c", "k", "m", "n", "p", "t", "i", "y", "o", "u"];

/// Double final consonants (3)
const FINALS_2: &[&str] = &["ch", "ng", "nh"];

/// Spelling rules: (initial, invalid_first_vowels)
/// If initial + first_vowel matches, combination is INVALID
const SPELLING_RULES: &[(&str, &[char])] = &[
    ("c", &['e', 'ê', 'i', 'y']), // c before e/ê/i/y → use k
    ("k", &['a', 'ă', 'â', 'o', 'ô', 'ơ', 'u', 'ư']), // k before back vowels → use c
    ("g", &['e', 'ê', 'i']),      // g before e/ê/i → use gh
    ("ng", &['e', 'ê', 'i']),     // ng before e/ê/i → use ngh
    ("gh", &['a', 'ă', 'â', 'o', 'ô', 'ơ', 'u', 'ư']), // gh before back vowels → use g
    ("ngh", &['a', 'ă', 'â', 'o', 'ô', 'ơ', 'u', 'ư']), // ngh before back vowels → use ng
];

/// Check if initial + vowel combination violates spelling rules
fn violates_spelling_rule(initial: &str, vowel: &str) -> bool {
    let first_char = vowel.chars().next().unwrap_or(' ');

    for (rule_initial, invalid_vowels) in SPELLING_RULES {
        if initial == *rule_initial && invalid_vowels.contains(&first_char) {
            return true;
        }
    }
    false
}

/// Check if vowel + final combination is valid
/// Rules:
/// 1. If vowel ends in semi-vowel glide (i, y, o, u), no final allowed
/// 2. Semivowel finals (i, y, o, u) can only follow single vowels, not diphthongs
fn is_valid_vowel_final(vowel: &str, final_c: &str) -> bool {
    let vowel_len = vowel.chars().count();
    let vowel_last = vowel.chars().last().unwrap_or(' ');

    // Rule 1: Vowels ending in semi-vowel glides can't take any finals
    let ends_in_glide = ['i', 'y', 'o', 'u'].contains(&vowel_last);
    if ends_in_glide {
        return final_c.is_empty();
    }

    // Rule 2: Semivowel finals can only follow single vowels
    let is_semivowel_final = ["i", "y", "o", "u"].contains(&final_c);
    if is_semivowel_final && vowel_len > 1 {
        return false;
    }

    // For consonant finals or empty final, all valid
    true
}

/// Generate all valid Vietnamese syllables
fn generate_syllables() -> Vec<String> {
    let mut syllables = HashSet::new();

    // Collect all initials including empty (no initial)
    let mut all_initials: Vec<&str> = vec![""];
    all_initials.extend(INITIALS_1);
    all_initials.extend(INITIALS_2);
    all_initials.extend(INITIALS_3);

    // Collect all vowels
    let mut all_vowels: Vec<&str> = Vec::new();
    all_vowels.extend(SINGLE_VOWELS);
    all_vowels.extend(DIPHTHONGS);
    all_vowels.extend(TRIPHTHONGS);

    // Collect all finals including empty (no final)
    let mut all_finals: Vec<&str> = vec![""];
    all_finals.extend(FINALS_1);
    all_finals.extend(FINALS_2);

    // Generate combinations
    for initial in &all_initials {
        for vowel in &all_vowels {
            // Skip if violates spelling rules
            if !initial.is_empty() && violates_spelling_rule(initial, vowel) {
                continue;
            }

            for final_c in &all_finals {
                // Skip invalid vowel+final combinations
                if !final_c.is_empty() && !is_valid_vowel_final(vowel, final_c) {
                    continue;
                }

                let syllable = format!("{}{}{}", initial, vowel, final_c);
                syllables.insert(syllable);
            }
        }
    }

    let mut result: Vec<String> = syllables.into_iter().collect();
    result.sort();
    result
}

#[test]
fn test_generate_and_save_syllables() {
    let syllables = generate_syllables();

    println!("Generated {} valid Vietnamese syllables", syllables.len());

    // Save to file
    let path = "tests/data/vietnamese-syllables.txt";
    let mut file = File::create(path).expect("Failed to create file");

    for syllable in &syllables {
        writeln!(file, "{}", syllable).expect("Failed to write");
    }

    println!("Saved to {}", path);

    // Basic assertions
    // ~8000-9000 syllables after applying vowel+final constraints
    assert!(syllables.len() > 7000, "Should have >7k syllables");
    assert!(syllables.contains(&"ba".to_string()), "Should contain 'ba'");
    assert!(
        syllables.contains(&"ngươi".to_string()),
        "Should contain 'ngươi' (no tone marks)"
    );

    // Check spelling rules are applied
    assert!(
        !syllables.contains(&"ce".to_string()),
        "ce is invalid (should be ke)"
    );
    assert!(
        !syllables.contains(&"ka".to_string()),
        "ka is invalid (should be ca)"
    );
    assert!(
        !syllables.contains(&"ge".to_string()),
        "ge is invalid (should be ghe)"
    );
    assert!(
        !syllables.contains(&"nge".to_string()),
        "nge is invalid (should be nghe)"
    );
}

#[test]
fn test_syllable_statistics() {
    let syllables = generate_syllables();

    let no_initial: Vec<_> = syllables
        .iter()
        .filter(|s| {
            let first = s.chars().next().unwrap_or(' ');
            "aăâeêioôơuưy".contains(first)
        })
        .collect();

    let with_final: Vec<_> = syllables
        .iter()
        .filter(|s| {
            let last = s.chars().last().unwrap_or(' ');
            "ckmntpyou".contains(last)
                || s.ends_with("ch")
                || s.ends_with("ng")
                || s.ends_with("nh")
        })
        .collect();

    println!("Statistics:");
    println!("  Total syllables: {}", syllables.len());
    println!("  No initial (V pattern): {}", no_initial.len());
    println!("  With final consonant: {}", with_final.len());
    println!(
        "  Open syllables (no final): {}",
        syllables.len() - with_final.len()
    );
}

/// Print sample syllables for verification
#[test]
fn test_print_samples() {
    let syllables = generate_syllables();

    println!("\n=== Sample Syllables ===\n");

    // First 50
    println!("First 50:");
    for s in syllables.iter().take(50) {
        print!("{} ", s);
    }
    println!("\n");

    // Some specific patterns
    println!("With 'ng' initial:");
    for s in syllables
        .iter()
        .filter(|s| s.starts_with("ng") && !s.starts_with("ngh"))
        .take(20)
    {
        print!("{} ", s);
    }
    println!("\n");

    println!("With 'ngh' initial:");
    for s in syllables.iter().filter(|s| s.starts_with("ngh")).take(20) {
        print!("{} ", s);
    }
    println!("\n");

    println!("Triphthongs:");
    for s in syllables
        .iter()
        .filter(|s| TRIPHTHONGS.iter().any(|t| s.contains(t)))
        .take(30)
    {
        print!("{} ", s);
    }
    println!();
}
