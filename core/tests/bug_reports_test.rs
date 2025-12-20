//! Bug reports test cases
//! These tests document expected behavior from user bug reports.

mod common;
use common::telex;
use gonhanh_core::engine::Engine;
use gonhanh_core::utils::type_word;

// =============================================================================
// BUG 1: "did" -> expect "đi"
// Current: ?
// Expected: "đi"
// =============================================================================

#[test]
fn bug1_did_to_di() {
    let mut e = Engine::new();
    let result = type_word(&mut e, "did");
    println!("'did' -> '{}' (expected: 'đi')", result);
    // TODO: Verify expected behavior
    // telex(&[("did", "đi")]);
}

// =============================================================================
// BUG 2: "thowifi" -> "thơìi", expected "thờii"
// Current: thơìi (horn on o, huyền on second i)
// Expected: thờii (horn+huyền on o, plain ii)
// =============================================================================

#[test]
fn bug2_thowifi() {
    // Test with huyền tone mark (f) - the actual input sequence
    // "thowifi" should produce "thờii" (tone on ơ, not on i)
    let mut e = Engine::new();
    let result = type_word(&mut e, "thowifi");
    println!("'thowifi' -> '{}' (expected: 'thờii')", result);
    // TODO: Verify expected behavior
    // telex(&[("thowifi", "thờii")]);
}

// =============================================================================
// BUG 3: "uawf"
// GoNhanh: uằ (w applies breve to a)
// OS built-in: ừa (w applies horn to u, creating ưa pattern)
// =============================================================================

#[test]
fn bug3_uawf() {
    let mut e = Engine::new();
    let result = type_word(&mut e, "uawf");
    println!("'uawf' -> '{}' (OS built-in gives: 'ừa')", result);
    // TODO: Decide on expected behavior
    // If following OS built-in: telex(&[("uawf", "ừa")]);
}

// =============================================================================
// BUG 4: "cuoiwsi" -> "cươii", expected "cướii"
// Current: cươii (ươ without tone, or tone on wrong position)
// Expected: cướii (ươ + sắc tone on ươ)
// =============================================================================

#[test]
fn bug4_thuoiwfi() {
    // Test with compound vowel ươ + sắc tone mark (s)
    // "cuoiwsi" should produce "cướii" (tone on ươ, not on i)
    let mut e = Engine::new();
    let result = type_word(&mut e, "cuoiwsi");
    println!("'cuoiwsi' -> '{}' (expected: 'cướii')", result);
    // TODO: Verify expected behavior
    // telex(&[("cuoiwsi", "cướii")]);
}

// =============================================================================
// BUG 5: "ddd" -> "đd", expected "dd"
// Current: đd (đ + d because third d is just added)
// Expected: dd (third d reverts stroke, returning to raw)
// =============================================================================

#[test]
fn bug5_ddd_revert() {
    let mut e = Engine::new();
    let result = type_word(&mut e, "ddd");
    println!("'ddd' -> '{}' (expected: 'dd')", result);
    // TODO: Change behavior
    // telex(&[("ddd", "dd")]);
}

// =============================================================================
// Additional test: Current expected behaviors
// =============================================================================

#[test]
fn current_dd_makes_stroke() {
    // dd → đ (correct, should not change)
    telex(&[("dd", "đ")]);
}

#[test]
fn current_thowi() {
    // Check what thowi produces
    let mut e = Engine::new();
    let result = type_word(&mut e, "thowi");
    println!("'thowi' -> '{}'", result);
}

#[test]
fn current_uaw() {
    // Check what uaw produces (without f)
    let mut e = Engine::new();
    let result = type_word(&mut e, "uaw");
    println!("'uaw' -> '{}'", result);
}

// =============================================================================
// BUG 6: " ddddd" (space + ddddd) -> deletes the space
// Current: space is deleted
// Expected: " dddd" (space preserved)
// =============================================================================

#[test]
fn bug6_ddddd_deletes_space() {
    let mut e = Engine::new();
    let result = type_word(&mut e, " ddddd");
    println!("' ddddd' -> '{}' (expected: ' dddd')", result);
    assert_eq!(
        result, " dddd",
        "Space should be preserved when typing ' ddddd'"
    );
}

#[test]
fn ddddd_behavior() {
    let mut e = Engine::new();

    // Debug step by step
    use gonhanh_core::engine::Action;

    let mut screen = String::new();
    let inputs = ['d', 'd', 'd', 'd', 'd'];

    for c in inputs {
        let key = gonhanh_core::utils::char_to_key(c);
        let r = e.on_key(key, false, false);

        if r.action == Action::Send as u8 {
            println!(
                "Key '{}': backspace={}, output='{}' (screen before: '{}')",
                c,
                r.backspace,
                (0..r.count as usize)
                    .filter_map(|i| char::from_u32(r.chars[i]))
                    .collect::<String>(),
                screen
            );
            for _ in 0..r.backspace {
                screen.pop();
            }
            for i in 0..r.count as usize {
                if let Some(ch) = char::from_u32(r.chars[i]) {
                    screen.push(ch);
                }
            }
        } else {
            println!("Key '{}': passthrough (screen before: '{}')", c, screen);
            screen.push(c);
        }
        println!("  -> screen after: '{}'", screen);
    }

    println!("\nFinal: 'ddddd' -> '{}' (expected: 'dddd')", screen);
    assert_eq!(screen, "dddd", "'ddddd' should produce 'dddd'");
}

// =============================================================================
// BUG 7: After "ddddd" → "dddd", backspace to "d", then "d" should produce "đ"
// The stroke_reverted flag should be reset on backspace
// =============================================================================

#[test]
fn bug7_backspace_resets_stroke_reverted() {
    // Type "ddddd" → "dddd", then backspace 3 times → "d", then type "d" → should be "đ"
    // Note: '<' is mapped to DELETE key in char_to_key
    let mut e = Engine::new();
    let result = type_word(&mut e, "ddddd<<<d");
    println!(
        "'ddddd' + backspace×3 + 'd' -> '{}' (expected: 'đ')",
        result
    );
    assert_eq!(result, "đ", "After backspace, dd should produce đ again");
}
