// Issue #387: In English mode (IME disabled), a word shortcut must still expand
// after the user corrects a typo with Backspace.
//
// Repro: shortcut "#hcm" -> "Thành phố Hồ Chí Minh"
//   type # h c n  -> Backspace (delete n) -> type m  -> Space
//   expected: replacement triggered (was: nothing, because Backspace cleared
//   the internal shortcut_prefix in the disabled-mode handler)

use gonhanh_core::data::keys;
use gonhanh_core::engine::shortcut::Shortcut;
use gonhanh_core::engine::Engine;

fn setup() -> Engine {
    let mut e = Engine::new();
    e.set_enabled(false); // English mode (IME off)
    e.shortcuts_mut()
        .add(Shortcut::new("#hcm", "Thành phố Hồ Chí Minh"));
    e
}

fn result_string(r: &gonhanh_core::engine::Result) -> String {
    (0..r.count as usize)
        .filter_map(|i| char::from_u32(r.chars[i]))
        .collect()
}

#[test]
fn issue387_shortcut_after_backspace_correction() {
    let mut e = setup();

    e.on_key_ext(keys::N3, false, false, true); // '#'
    e.on_key(keys::H, false, false);
    e.on_key(keys::C, false, false);
    e.on_key(keys::N, false, false); // typo: "#hcn"
    e.on_key(keys::DELETE, false, false); // Backspace -> "#hc"
    e.on_key(keys::M, false, false); // -> "#hcm"
    let r = e.on_key(keys::SPACE, false, false);

    assert_eq!(
        r.action, 1,
        "Space should trigger shortcut after backspace correction"
    );
    assert_eq!(r.backspace, 4, "Should backspace 4 chars (#hcm)");
    assert_eq!(result_string(&r), "Thành phố Hồ Chí Minh ");
}

#[test]
fn issue387_multiple_backspaces() {
    let mut e = setup();

    // Type "#hcXYZ" then delete 3 wrong chars, retype "m"
    e.on_key_ext(keys::N3, false, false, true); // '#'
    e.on_key(keys::H, false, false);
    e.on_key(keys::C, false, false);
    e.on_key(keys::X, false, false);
    e.on_key(keys::Y, false, false);
    e.on_key(keys::Z, false, false); // "#hcXYZ"
    e.on_key(keys::DELETE, false, false);
    e.on_key(keys::DELETE, false, false);
    e.on_key(keys::DELETE, false, false); // -> "#hc"
    e.on_key(keys::M, false, false); // -> "#hcm"
    let r = e.on_key(keys::SPACE, false, false);

    assert_eq!(
        r.action, 1,
        "Shortcut should expand after multiple backspaces"
    );
    assert_eq!(result_string(&r), "Thành phố Hồ Chí Minh ");
}

#[test]
fn issue387_backspace_does_not_overshoot() {
    // Backspacing more than typed must not panic / underflow, and prefix stays empty
    let mut e = setup();
    e.on_key(keys::H, false, false);
    e.on_key(keys::DELETE, false, false);
    e.on_key(keys::DELETE, false, false); // extra backspace on empty prefix
                                          // Now type a clean shortcut, should still work
    e.on_key_ext(keys::N3, false, false, true); // '#'
    e.on_key(keys::H, false, false);
    e.on_key(keys::C, false, false);
    e.on_key(keys::M, false, false);
    let r = e.on_key(keys::SPACE, false, false);
    assert_eq!(
        r.action, 1,
        "Clean shortcut works after overshoot backspaces"
    );
}

// Control: clean typing (no backspace) keeps working
#[test]
fn issue387_control_clean_typing() {
    let mut e = setup();
    e.on_key_ext(keys::N3, false, false, true); // '#'
    e.on_key(keys::H, false, false);
    e.on_key(keys::C, false, false);
    e.on_key(keys::M, false, false);
    let r = e.on_key(keys::SPACE, false, false);
    assert_eq!(r.action, 1);
    assert_eq!(result_string(&r), "Thành phố Hồ Chí Minh ");
}
