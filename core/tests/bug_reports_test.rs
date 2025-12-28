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
// FIXED: "Wf" → "Ừ", "wmf" → "ừm", "Wmf " → "Ừm "
// W shortcut converts to ư, then mark 'f' applies to ư correctly
// Tests added to unit_test.rs TELEX_WORDS section
// =============================================================================

#[test]
fn fixed_w_shortcut_with_mark() {
    telex(&[
        ("Wf", "Ừ"),
        ("wf", "ừ"),
        ("wmf", "ừm"),
        ("Wmf ", "Ừm "),
        ("wmf ", "ừm "),
    ]);
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

// =============================================================================
// BUG 8: "taifii" -> "taìi", expected "tàii"
// When extra vowels are typed after a valid diphthong with mark, the mark
// should stay on the correct vowel for the original diphthong, not move to
// a new position based on invalid triphthong rules.
// =============================================================================

#[test]
fn bug8_extra_vowel_after_diphthong_mark() {
    let mut e = Engine::new();
    // taif → tài (mark on 'a' for "ai" diphthong)
    // taifi → should be tàii (mark stays on 'a', not moved to 'i')
    // The issue was: typing "taifi" produced "taìi" (mark wrongly on first 'i')
    // Fixed: "taifi" now correctly produces "tàii" (mark stays on 'a')
    let result = type_word(&mut e, "taifi");
    println!("'taifi' -> '{}' (expected: 'tàii')", result);
    assert_eq!(
        result, "tàii",
        "'taifi' should produce 'tàii' (mark on 'a')"
    );

    // Also verify the 6-key input produces 3 i's
    let mut e2 = Engine::new();
    let result2 = type_word(&mut e2, "taifii");
    println!("'taifii' -> '{}' (expected: 'tàiii')", result2);
    assert_eq!(
        result2, "tàiii",
        "'taifii' should produce 'tàiii' (mark on 'a')"
    );
}

// =============================================================================
// BUG 9: Delayed circumflex with post-tone 'd' for stroke
// "ddoong " -> "đông " (dd=đ, oo=ô, ng=final)
// "doodng " -> "đông " (d, oo=ô, d=stroke on initial d, ng=final)
// "duod" -> "đuo" (d, uo, d=stroke on initial d)
// =============================================================================

#[test]
fn bug9_delayed_circumflex_stroke() {
    let mut e = Engine::new();
    let result = type_word(&mut e, "ddoong ");
    println!("'ddoong ' -> '{}' (expected: 'đông ')", result);
    assert_eq!(result, "đông ", "'ddoong ' should produce 'đông '");

    let mut e2 = Engine::new();
    let result2 = type_word(&mut e2, "doodng ");
    println!("'doodng ' -> '{}' (expected: 'đông ')", result2);
    assert_eq!(result2, "đông ", "'doodng ' should produce 'đông '");

    // Test without space
    let mut e3 = Engine::new();
    let result3 = type_word(&mut e3, "duod");
    println!("'duod' -> '{}' (expected: 'đuo')", result3);
    assert_eq!(result3, "đuo", "'duod' should produce 'đuo'");

    // Test with space
    let mut e4 = Engine::new();
    let result4 = type_word(&mut e4, "duod ");
    println!("'duod ' -> '{}' (expected: 'đuo ')", result4);
    assert_eq!(result4, "đuo ", "'duod ' should produce 'đuo '");
}

// =============================================================================
// BUG 10: "raisse " should restore to "raise ", "raise " should stay "raise "
// With auto_restore enabled, English words should be detected and restored
// =============================================================================

#[test]
fn bug10_raisse_restore() {
    // First check without auto_restore
    let mut e = Engine::new();
    let result = type_word(&mut e, "raisse ");
    println!("[no auto_restore] 'raisse ' -> '{}'", result);

    let mut e2 = Engine::new();
    let result2 = type_word(&mut e2, "raise ");
    println!("[no auto_restore] 'raise ' -> '{}'", result2);

    // Then with auto_restore
    let mut e3 = Engine::new();
    e3.set_english_auto_restore(true);
    let result3 = type_word(&mut e3, "raisse ");
    println!("[auto_restore] 'raisse ' -> '{}'", result3);

    let mut e4 = Engine::new();
    e4.set_english_auto_restore(true);
    let result4 = type_word(&mut e4, "raise ");
    println!("[auto_restore] 'raise ' -> '{}'", result4);

    // Assert expected behavior with auto_restore
    assert_eq!(result3, "raise ", "'raisse ' should produce 'raise '");
    assert_eq!(result4, "raise ", "'raise ' should produce 'raise '");

    // Check what "theme " produces (without and with auto_restore)
    let mut e5a = Engine::new();
    let result5a = type_word(&mut e5a, "theme ");
    println!("[no auto_restore] 'theme ' -> '{}'", result5a);

    let mut e5b = Engine::new();
    e5b.set_english_auto_restore(true);
    let result5b = type_word(&mut e5b, "theme ");
    println!("[auto_restore] 'theme ' -> '{}'", result5b);

    // "theme " should produce "thêm " (valid Vietnamese, NOT restored)
    // In Telex: delayed circumflex - 'e' after consonant applies to previous 'e'
    assert_eq!(
        result5b, "thêm ",
        "'theme ' should produce 'thêm ' (valid Vietnamese)"
    );

    // "sorry " should stay as "sorry " (not "sory ")
    // This verifies we excluded 'y' from the double-s + vowel pattern
    let mut e6 = Engine::new();
    e6.set_english_auto_restore(true);
    let result6 = type_word(&mut e6, "sorry ");
    println!("[auto_restore] 'sorry ' -> '{}'", result6);
    assert_eq!(
        result6, "sorry ",
        "'sorry ' should produce 'sorry ' (not 'sory ')"
    );

    // "dayda " and "daday " should produce "đây " (valid Vietnamese)
    let mut e7 = Engine::new();
    let result7 = type_word(&mut e7, "dayda ");
    println!("[no auto_restore] 'dayda ' -> '{}'", result7);

    let mut e8 = Engine::new();
    e8.set_english_auto_restore(true);
    let result8 = type_word(&mut e8, "dayda ");
    println!("[auto_restore] 'dayda ' -> '{}'", result8);

    let mut e9 = Engine::new();
    e9.set_english_auto_restore(true);
    let result9 = type_word(&mut e9, "daday ");
    println!("[auto_restore] 'daday ' -> '{}'", result9);

    assert_eq!(result8, "đây ", "'dayda ' should produce 'đây '");
    assert_eq!(result9, "đây ", "'daday ' should produce 'đây '");
}

// =============================================================================
// BUG 11: Shortcut "->" → "→" not working
// Break characters like '-' and '>' are not accumulated in buffer
// =============================================================================

#[test]
fn bug11_arrow_shortcut() {
    use gonhanh_core::engine::shortcut::Shortcut;

    let mut e = Engine::new();
    // Add shortcut "->" → "→"
    e.shortcuts_mut().add(Shortcut::immediate("->", "→"));

    let result = type_word(&mut e, "->");
    println!("'->' -> '{}' (expected: '→')", result);
    assert_eq!(result, "→", "'->' should produce '→'");
}

// =============================================================================
// ISSUE #128: Gõ tắt không hoạt động sau khi xoá
// After typing shortcut "->", deleting "→", then typing "->" again doesn't work
// =============================================================================

#[test]
fn issue128_shortcut_after_delete() {
    use gonhanh_core::engine::shortcut::Shortcut;

    let mut e = Engine::new();
    e.shortcuts_mut().add(Shortcut::immediate("->", "→"));

    // Step 1: Type "->" → "→"
    let result1 = type_word(&mut e, "->");
    println!("Step 1: '->' -> '{}' (expected: '→')", result1);
    assert_eq!(result1, "→", "First '->' should produce '→'");

    // Step 2: Delete "→" (simulated by '<' which maps to DELETE)
    let result2 = type_word(&mut e, "<");
    println!("Step 2: after delete -> '{}' (expected: '')", result2);

    // Step 3: Type "->" again → should still produce "→"
    let result3 = type_word(&mut e, "->");
    println!("Step 3: '->' again -> '{}' (expected: '→')", result3);
    assert_eq!(result3, "→", "Second '->' after delete should produce '→'");
}

#[test]
fn issue128_shortcut_after_multiple_deletes() {
    use gonhanh_core::data::keys;
    use gonhanh_core::engine::shortcut::Shortcut;
    use gonhanh_core::engine::Action;

    let mut e = Engine::new();
    e.shortcuts_mut().add(Shortcut::immediate("->", "→"));

    // Step 1: Type "-" (first char of shortcut)
    let r1 = e.on_key_ext(keys::MINUS, false, false, false);
    println!("Step 1: '-' action={}", r1.action);

    // Step 2: Type ">" (should trigger shortcut)
    let r2 = e.on_key_ext(keys::DOT, false, false, true); // Shift+DOT = '>'
    println!("Step 2: '>' action={}, count={}", r2.action, r2.count);
    assert_eq!(r2.action, Action::Send as u8, "Shortcut should fire");

    // Step 3: Delete (backspace)
    let r3 = e.on_key_ext(keys::DELETE, false, false, false);
    println!("Step 3: DELETE action={}", r3.action);

    // Step 4: Type "-" again
    let r4 = e.on_key_ext(keys::MINUS, false, false, false);
    println!("Step 4: '-' action={}", r4.action);

    // Step 5: Type ">" again (should trigger shortcut)
    let r5 = e.on_key_ext(keys::DOT, false, false, true);
    println!("Step 5: '>' action={}, count={}", r5.action, r5.count);
    assert_eq!(
        r5.action,
        Action::Send as u8,
        "Second shortcut should fire after delete"
    );
}

#[test]
fn issue128_detailed_debug() {
    use gonhanh_core::data::keys;
    use gonhanh_core::engine::shortcut::Shortcut;
    use gonhanh_core::engine::Action;

    let mut e = Engine::new();
    e.shortcuts_mut().add(Shortcut::immediate("->", "→"));

    println!("=== Issue #128 Detailed Debug ===");

    // Type "->" first time
    println!("\n[1] Type '-'");
    let r = e.on_key_ext(keys::MINUS, false, false, false);
    println!("    Result: action={}", r.action);

    println!("\n[2] Type '>' (Shift+DOT)");
    let r = e.on_key_ext(keys::DOT, false, false, true);
    println!(
        "    Result: action={}, backspace={}, count={}",
        r.action, r.backspace, r.count
    );
    if r.action == Action::Send as u8 {
        let chars: String = (0..r.count as usize)
            .filter_map(|i| char::from_u32(r.chars[i]))
            .collect();
        println!("    Output: '{}'", chars);
    }

    // Now delete
    println!("\n[3] Press DELETE");
    let r = e.on_key_ext(keys::DELETE, false, false, false);
    println!("    Result: action={}", r.action);

    // Type "->" second time
    println!("\n[4] Type '-' again");
    let r = e.on_key_ext(keys::MINUS, false, false, false);
    println!("    Result: action={}", r.action);

    println!("\n[5] Type '>' again (Shift+DOT)");
    let r = e.on_key_ext(keys::DOT, false, false, true);
    println!(
        "    Result: action={}, backspace={}, count={}",
        r.action, r.backspace, r.count
    );
    if r.action == Action::Send as u8 {
        let chars: String = (0..r.count as usize)
            .filter_map(|i| char::from_u32(r.chars[i]))
            .collect();
        println!("    Output: '{}' ✓", chars);
    } else {
        println!("    ✗ Shortcut did NOT fire!");
    }

    assert_eq!(r.action, Action::Send as u8, "Second shortcut should fire");
}

// =============================================================================
// ISSUE #129: Gõ tắt không hoạt động khi chuyển sang tiếng Anh
// Shortcuts don't work after switching to English mode
// Note: This is a design decision - shortcuts should work in English mode too
// =============================================================================

#[test]
fn issue129_shortcut_in_english_mode() {
    use gonhanh_core::engine::shortcut::Shortcut;

    let mut e = Engine::new();
    e.shortcuts_mut().add(Shortcut::immediate("->", "→"));

    // Step 1: Type "->" in Vietnamese mode → "→"
    let result1 = type_word(&mut e, "->");
    println!("Step 1 [VI]: '->' -> '{}' (expected: '→')", result1);
    assert_eq!(result1, "→", "'->' in Vietnamese mode should produce '→'");

    // Step 2: Switch to English mode (disable IME)
    e.set_enabled(false);

    // Step 3: Type "->" in English mode → should still produce "→"
    // Currently this fails because disabled IME bypasses all processing
    let result2 = type_word(&mut e, "->");
    println!("Step 2 [EN]: '->' -> '{}' (expected: '→')", result2);
    assert_eq!(
        result2, "→",
        "'->' in English mode should still produce '→'"
    );
}

// =============================================================================
// ISSUE #130: Gõ tắt không hoạt động khi gõ tắt nhiều lần
// Shortcut doesn't work when used multiple times with text in between
// =============================================================================

#[test]
fn issue130_multiple_shortcuts() {
    use gonhanh_core::engine::shortcut::Shortcut;

    let mut e = Engine::new();
    e.shortcuts_mut().add(Shortcut::immediate("->", "→"));

    // Step 1: Type "->" → "→"
    let result1 = type_word(&mut e, "->");
    println!("Step 1: '->' -> '{}' (expected: '→')", result1);
    assert_eq!(result1, "→", "First '->' should produce '→'");

    // Step 2: Type "abc"
    let result2 = type_word(&mut e, "abc");
    println!("Step 2: 'abc' -> '{}' (expected: 'abc')", result2);
    assert_eq!(result2, "abc", "'abc' should produce 'abc'");

    // Step 3: Type "->" again → should produce "→"
    let result3 = type_word(&mut e, "->");
    println!("Step 3: '->' again -> '{}' (expected: '→')", result3);
    assert_eq!(result3, "→", "Second '->' should produce '→'");
}

#[test]
fn issue130_shortcut_after_word_with_space() {
    use gonhanh_core::engine::shortcut::Shortcut;

    let mut e = Engine::new();
    e.shortcuts_mut().add(Shortcut::immediate("->", "→"));

    // Type "->" + "abc " + "->"
    // The space after "abc" should clear state properly
    let result1 = type_word(&mut e, "->");
    assert_eq!(result1, "→", "First '->'");

    let result2 = type_word(&mut e, "abc ");
    assert_eq!(result2, "abc ", "'abc ' should pass through");

    let result3 = type_word(&mut e, "->");
    println!("After '→abc ': '->' -> '{}' (expected: '→')", result3);
    assert_eq!(result3, "→", "'->' after 'abc ' should produce '→'");
}

// =============================================================================
// BUG 145: "view" → "vieư", expected "view"
// The triphthong iêu requires circumflex on E. When typing "view":
// - "iew" has no circumflex on E and horn on U → invalid Vietnamese
// - Should NOT transform w→ư when result is invalid
// =============================================================================

#[test]
fn bug145_view_should_not_transform() {
    // Without auto_restore: "view" should stay as "view" (w not transformed)
    let mut e = Engine::new();
    let result = type_word(&mut e, "view");
    println!("'view' -> '{}' (expected: 'view')", result);
    assert_eq!(result, "view", "'view' should stay as 'view', not 'vieư'");

    // With auto_restore and space: should also be "view "
    let mut e2 = Engine::new();
    e2.set_english_auto_restore(true);
    let result2 = type_word(&mut e2, "view ");
    println!(
        "[auto_restore] 'view ' -> '{}' (expected: 'view ')",
        result2
    );
    assert_eq!(
        result2, "view ",
        "'view ' with auto_restore should be 'view '"
    );
}

// =============================================================================
// BUG: "derde " → "để " (circumflex + hỏi combined)
// In Telex: d=initial, e=vowel, r=hỏi, d=stroke, e=circumflex
// The second 'e' should add circumflex to existing ẻ → ể
// =============================================================================

#[test]
fn bug_derde_to_de_hoi() {
    // Debug: step by step
    use gonhanh_core::engine::Action;

    let mut e = Engine::new();
    e.set_english_auto_restore(true);

    let mut screen = String::new();
    let inputs = ['d', 'e', 'r', 'd', 'e', ' '];

    for c in inputs {
        let key = gonhanh_core::utils::char_to_key(c);
        let r = e.on_key(key, false, false);

        if r.action == Action::Send as u8 {
            for _ in 0..r.backspace {
                screen.pop();
            }
            for i in 0..r.count as usize {
                if let Some(ch) = char::from_u32(r.chars[i]) {
                    screen.push(ch);
                }
            }
            println!(
                "Key '{}': backspace={}, output='{}', screen='{}'",
                c,
                r.backspace,
                (0..r.count as usize)
                    .filter_map(|i| char::from_u32(r.chars[i]))
                    .collect::<String>(),
                screen
            );
        } else {
            screen.push(c);
            println!("Key '{}': passthrough, screen='{}'", c, screen);
        }
    }

    println!("\nFinal: 'derde ' -> '{}' (expected: 'để ')", screen);
    assert_eq!(
        screen, "để ",
        "'derde ' with auto_restore should produce 'để '"
    );
}

// =============================================================================
// ISSUE #146: "tóm" → "toms" (tone mark not applied)
// In Telex: "toms" should produce "tóm" (s = sắc tone on 'o')
// =============================================================================

#[test]
fn issue146_tom_s_should_produce_tom_sac() {
    // "toms" in Telex should produce "tóm" (sắc tone on 'o')
    let mut e = Engine::new();
    let result = type_word(&mut e, "toms");
    println!("'toms' -> '{}' (expected: 'tóm')", result);
    assert_eq!(result, "tóm", "'toms' should produce 'tóm', not 'toms'");

    // Also test with space
    let mut e2 = Engine::new();
    let result2 = type_word(&mut e2, "toms ");
    println!("'toms ' -> '{}' (expected: 'tóm ')", result2);
    assert_eq!(
        result2, "tóm ",
        "'toms ' should produce 'tóm ', not 'toms '"
    );

    // Test similar patterns
    telex(&[
        ("toms", "tóm"), // Issue #146
        ("moms", "móm"), // Similar pattern
        ("boms", "bóm"), // Similar pattern
        ("coms", "cóm"), // Similar pattern
        ("doms", "dóm"), // Similar pattern
        ("noms", "nóm"), // Similar pattern
    ]);
}
