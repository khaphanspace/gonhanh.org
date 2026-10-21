//! English Words Protection Test
//!
//! Tests that English words with INVALID Vietnamese structure are NOT transformed.
//!
//! ## Limitations (words that CANNOT be protected)
//! - Words matching valid Vietnamese patterns: "as"→"á", "or"→"ỏ", "an"→"ăn"
//! - Words with sw-, tw- clusters: "swap"→"sưap" (vì "sư" hợp lệ)
//! - Words starting with 'w': "without"→"ưithout" (vì "ư" hợp lệ)
//!
//! ## What CAN be protected
//! - Invalid initials: bl-, cl-, str-, chr-, etc. (không tồn tại trong tiếng Việt)
//! - Invalid vowel patterns: ou, yo
//! - Invalid final clusters: T+R, P+R, C+R patterns

use gonhanh_core::engine::Engine;

fn assert_no_transform(words: &[&str]) {
    let mut telex = Engine::new();
    telex.set_method(0);

    for word in words {
        telex.clear();
        let mut output = String::new();

        for ch in word.chars() {
            let key = char_to_key(ch);
            let result = telex.on_key(key, ch.is_uppercase(), false);

            if result.action == 1 {
                let bs = result.backspace as usize;
                for _ in 0..bs.min(output.len()) {
                    output.pop();
                }
                for i in 0..result.count as usize {
                    if let Some(c) = char::from_u32(result.chars[i]) {
                        output.push(c);
                    }
                }
            } else {
                output.push(ch);
            }
        }

        assert_eq!(output, *word, "'{}' → '{}'", word, output);
    }
}

/// Test with auto-restore enabled - types word + space, expects "word " output
fn assert_auto_restore(words: &[&str]) {
    let mut telex = Engine::new();
    telex.set_method(0);
    telex.set_english_auto_restore(true); // Enable auto-restore

    for word in words {
        telex.clear();
        let mut output = String::new();

        // Type the word
        for ch in word.chars() {
            let key = char_to_key(ch);
            let result = telex.on_key(key, ch.is_uppercase(), false);

            if result.action == 1 {
                let bs = result.backspace as usize;
                for _ in 0..bs.min(output.len()) {
                    output.pop();
                }
                for i in 0..result.count as usize {
                    if let Some(c) = char::from_u32(result.chars[i]) {
                        output.push(c);
                    }
                }
            } else {
                output.push(ch);
            }
        }

        // Type space to trigger auto-restore
        let result = telex.on_key(49, false, false); // 49 = SPACE key
        if result.action == 1 {
            let bs = result.backspace as usize;
            for _ in 0..bs.min(output.len()) {
                output.pop();
            }
            for i in 0..result.count as usize {
                if let Some(c) = char::from_u32(result.chars[i]) {
                    output.push(c);
                }
            }
        } else {
            output.push(' ');
        }

        let expected = format!("{} ", word);
        assert_eq!(output, expected, "'{}' → '{}'", word, output);
    }
}

fn char_to_key(c: char) -> u16 {
    match c.to_ascii_lowercase() {
        'a' => 0,
        's' => 1,
        'd' => 2,
        'f' => 3,
        'h' => 4,
        'g' => 5,
        'z' => 6,
        'x' => 7,
        'c' => 8,
        'v' => 9,
        'b' => 11,
        'q' => 12,
        'w' => 13,
        'e' => 14,
        'r' => 15,
        'y' => 16,
        't' => 17,
        '1' => 18,
        '2' => 19,
        '3' => 20,
        '4' => 21,
        '6' => 22,
        '5' => 23,
        '9' => 25,
        '7' => 26,
        '8' => 28,
        '0' => 29,
        'o' => 31,
        'u' => 32,
        'i' => 34,
        'p' => 35,
        'l' => 37,
        'j' => 38,
        'k' => 40,
        'n' => 45,
        'm' => 46,
        _ => 255,
    }
}

// =============================================================================
// INVALID INITIALS - bl, br, cl, cr, dr, fl, fr, gl, gr, pl, pr, sc, sk, sl, sm, sn, sp, st
// (Excludes sw-, tw-, wr- vì bị transform thành sư-, tư-, ư-)
// =============================================================================

const INVALID_INITIALS: &[&str] = &[
    // bl-
    "black", "blue", "blank", "blast", "blend", "blind", "block", "blood", "blow", "blog",
    // br-
    "brain", "branch", "brand", "break", "bring", "broad", "brief", // cl-
    "class", "clean", "clear", "click", "client", "climb", "clone", "close", "club",
    // cr-
    "crash", "create", "credit", "cross", "crypto", "crystal", // dr-
    "draft", "dragon", "drain", "dream", "dress", "drink", "drive", "drop", "drug",
    // fl-
    "flag", "flash", "flat", "flex", "flight", "float", "floor", "flow", "fluid",
    // fr-
    "frame", "free", "fresh", "friend", "from", "front", "frozen", "fruit", // gl-
    "glass", "global", "glory", "glue", "gmail", // gr-
    "grade", "grand", "grant", "graph", "grass", "great", "green", "grid", "group", "grow",
    // pl-
    "place", "plan", "plant", "plate", "play", "please", "plot", "plug", "plus",
    // pr-
    "practice", "press", "price", "print", "private", "problem", "process", "product", "program",
    "project", // sc-
    "scale", "scan", "scene", "school", "science", "scope", "score", "screen", "script", "scroll",
    // sk-
    "sketch", "skill", "skip", "sky", // sl-
    "slack", "sleep", "slide", "slim", "slot", "slow", // sm-
    "small", "smart", "smile", "smtp", // sn-
    "snake", "snap", "snow", // sp-
    "space", "spam", "span", "spark", "speak", "special", "speed", "spell", "spend", "split",
    "sport", "spot", "spread", "spring", "sql", // st-
    "stack", "staff", "stage", "stand", "star", "start", "state", "static", "status", "stay",
    "step", "stick", "still", "stock", "stop", "store", "story", "strategy", "stream", "street",
    "stress", "strict", "string", "strip", "struct", "student", "study", "style", "submit",
];

// =============================================================================
// INVALID VOWEL PATTERNS - ou, yo (không tồn tại trong tiếng Việt)
// =============================================================================

const INVALID_OU_PATTERN: &[&str] = &[
    "you", "your", "out", "our", "hour", "four", "pour", "tour", "soup", "soul", "loud", "proud",
    "sound", "round", "found", "bound", "pound", "ground", "about", "count", "mount", "amount",
    "house", "mouse", "south", "mouth", "route", "could", "should", "through", "enough", "though",
    "thought", "brought", "touch", "couch", "source", "course", "account",
];

const INVALID_YO_PATTERN: &[&str] = &[
    "you", "your", "york", "yoga", "young", "youth", "beyond", "anyone",
];

// =============================================================================
// INVALID FINAL CLUSTERS - T+R, C+R patterns (detected by is_foreign_word_pattern)
// NOTE: "describe" removed - now handled via auto-restore (D+E pattern)
// =============================================================================

const INVALID_FINAL_CLUSTERS: &[&str] = &[
    "metric", "matrix", "electric", "spectrum", "control", "central", "abstract", "contract",
];

// =============================================================================
// DE + S pattern (describe, design...)
// These words are auto-restored when space is typed
// =============================================================================

const AUTO_RESTORE_DE_S: &[&str] = &[
    "describe",
    "design",
    "desk",
    "desktop",
    "destroy",
    "desperate",
];

// =============================================================================
// TECH TERMS - chỉ những từ có invalid structure
// =============================================================================

const TECH_TERMS: &[&str] = &[
    // Invalid initials - protected
    "string",
    "struct",
    "stream",
    "script",
    "scroll",
    "spring",
    "sprite",
    "blockchain",
    "bluetooth",
    "broadcast",
    "browser",
    "chrome",
    "chromium",
    "crypto",
    "crystal",
    "flask",
    "flutter",
    "framework",
    "frontend",
    "github",
    "gitlab",
    "gradle",
    "graphql",
    "grpc",
    "playwright",
    "prisma",
    "prometheus",
    "sklearn",
    "slack",
    "smtp",
    "spark",
    "splunk",
    "sql",
    "sqlite",
    "stack",
    "stripe",
    "trello",
    "typescript",
    // NOTE: Removed - starts with valid Vietnamese pattern:
    // "postgres" (po+s→pó), "terraform" (te+r→tẻ), "travis" (tra+v→valid)
];

// =============================================================================
// TESTS
// =============================================================================

#[test]
fn protect_invalid_initials() {
    assert_no_transform(INVALID_INITIALS);
}

#[test]
fn protect_ou_pattern() {
    assert_no_transform(INVALID_OU_PATTERN);
}

#[test]
fn protect_yo_pattern() {
    assert_no_transform(INVALID_YO_PATTERN);
}

#[test]
fn protect_final_clusters() {
    assert_no_transform(INVALID_FINAL_CLUSTERS);
}

#[test]
fn protect_de_s_pattern() {
    // DE + S pattern uses auto-restore when space is typed
    assert_auto_restore(AUTO_RESTORE_DE_S);
}

#[test]
fn protect_tech_terms() {
    assert_no_transform(TECH_TERMS);
}

#[test]
fn all_protected_words() {
    // Words that are protected without transformation (invalid Vietnamese structure)
    let no_transform: Vec<&str> = [
        INVALID_INITIALS,
        INVALID_OU_PATTERN,
        INVALID_YO_PATTERN,
        INVALID_FINAL_CLUSTERS,
        TECH_TERMS,
    ]
    .concat();

    let mut unique: Vec<&str> = no_transform.clone();
    unique.sort();
    unique.dedup();

    println!("Testing {} unique words (no transform)", unique.len());
    assert_no_transform(&unique);

    // Words that are restored via auto-restore (D+E pattern)
    println!("Testing {} words (auto-restore)", AUTO_RESTORE_DE_S.len());
    assert_auto_restore(AUTO_RESTORE_DE_S);
}
