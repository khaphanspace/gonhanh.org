//! Toggling a diacritic must still work after a word is committed with Space and
//! then restored with Backspace.
//!
//! Repro (Telex): type "sow" -> "sơ", Space -> "sơ ", Backspace -> "sơ"
//!   (word restored from history), then type "w" again.
//! Expected: the second "w" toggles the horn off, exactly as "soww" -> "sow"
//!   would without the Space/Backspace round-trip.
//!
//! Before the fix, tone keys (Telex w / a / e / o, VNI 6 / 7 / 8) did NOT toggle
//! after restore because `last_transform` was not reconstructed when the
//! committed word was pulled back from history. Tone-mark keys (s/f/r/x/j, VNI
//! 1-5) already toggled correctly via a separate "vowel-at-end" revert path, so
//! the bug was tone-only and silent.
//!
//! `<` is mapped to Backspace (DELETE) by the test harness; a literal space is a
//! Space keystroke.

use gonhanh_core::engine::Engine;
use gonhanh_core::utils::type_word;

fn telex(input: &str) -> String {
    let mut e = Engine::new();
    type_word(&mut e, input)
}

fn vni(input: &str) -> String {
    let mut e = Engine::new();
    e.set_method(1);
    type_word(&mut e, input)
}

// ============================================================
// Telex: tone toggle after restore must equal the no-restore baseline
// ============================================================

#[test]
fn telex_horn_o_toggle_after_restore() {
    // Baseline: "soww" -> "sow"
    assert_eq!(telex("soww"), "sow");
    // After restore: same outcome
    assert_eq!(telex("sow <w"), "sow");
}

#[test]
fn telex_horn_u_toggle_after_restore() {
    assert_eq!(telex("uww"), "uw");
    assert_eq!(telex("uw <w"), "uw");
}

#[test]
fn telex_breve_a_toggle_after_restore() {
    assert_eq!(telex("aww"), "aw");
    assert_eq!(telex("aw <w"), "aw");
}

#[test]
fn telex_circumflex_a_toggle_after_restore() {
    assert_eq!(telex("aaa"), "aa");
    assert_eq!(telex("aa <a"), "aa");
}

#[test]
fn telex_circumflex_e_toggle_after_restore() {
    assert_eq!(telex("eee"), "ee");
    assert_eq!(telex("ee <e"), "ee");
}

#[test]
fn telex_circumflex_o_toggle_after_restore() {
    assert_eq!(telex("ooo"), "oo");
    assert_eq!(telex("oo <o"), "oo");
}

#[test]
fn telex_horn_compound_toggle_after_restore() {
    // "uoww" -> "uow" (compound horn reverted)
    assert_eq!(telex("uoww"), "uow");
    assert_eq!(telex("uow <w"), "uow");
}

#[test]
fn telex_horn_consonant_initial_compound_after_restore() {
    assert_eq!(telex("huoww"), "huow");
    assert_eq!(telex("huow <w"), "huow");
}

// ============================================================
// Telex: tone-mark keys must KEEP working after restore (regression guard)
// ============================================================

#[test]
fn telex_mark_sac_toggle_after_restore_regression() {
    assert_eq!(telex("ass"), "as");
    assert_eq!(telex("as <s"), "as");
}

#[test]
fn telex_mark_huyen_toggle_after_restore_regression() {
    assert_eq!(telex("aff"), "af");
    assert_eq!(telex("af <f"), "af");
}

// ============================================================
// VNI: tone toggle after restore
// ============================================================

#[test]
fn vni_horn_o_toggle_after_restore() {
    assert_eq!(vni("o77"), "o7");
    assert_eq!(vni("o7 <7"), "o7");
}

#[test]
fn vni_horn_u_toggle_after_restore() {
    assert_eq!(vni("u77"), "u7");
    assert_eq!(vni("u7 <7"), "u7");
}

#[test]
fn vni_breve_a_toggle_after_restore() {
    assert_eq!(vni("a88"), "a8");
    assert_eq!(vni("a8 <8"), "a8");
}

#[test]
fn vni_circumflex_a_toggle_after_restore() {
    assert_eq!(vni("a66"), "a6");
    assert_eq!(vni("a6 <6"), "a6");
}

#[test]
fn vni_mark_sac_toggle_after_restore_regression() {
    assert_eq!(vni("a11"), "a1");
    assert_eq!(vni("a1 <1"), "a1");
}

// ============================================================
// Guard: a real Vietnamese word restored should still ADD a new tone normally,
// not be mistaken for a toggle/revert.
// ============================================================

#[test]
fn telex_add_mark_after_restore_still_works() {
    // "sow" -> "sơ", commit, restore, then add sắc -> "sớ"
    assert_eq!(telex("sow <s"), "sớ");
}

#[test]
fn telex_continue_typing_after_restore_still_works() {
    // "sow" -> "sơ", commit, restore, then type "n" -> "sơn"
    assert_eq!(telex("sow <n"), "sơn");
}

// ============================================================
// Mark on a NON-FINAL vowel after restore.
//
// "bias" -> "bía" carries the sắc on 'í', not the trailing 'a'. Reconstructing
// the last transform only from the final buffer char missed it, so the second
// mark key after a Space/Backspace round-trip did not revert (and re-arm the
// English-whitelist auto-restore) the way continuous typing does. Each restore
// case must equal its no-restore baseline.
// ============================================================

#[test]
fn telex_mark_revert_after_restore_mid_word_bias() {
    // Baseline: "biass" double-s reverts sắc and restores English "bias".
    assert_eq!(telex("biass"), "bias");
    // After Space/Backspace restore of "bía", the second "s" must do the same.
    assert_eq!(telex("bias <s"), "bias");
}

#[test]
fn telex_mark_revert_after_restore_mid_word_hoas() {
    assert_eq!(telex("hoass"), "hoas");
    assert_eq!(telex("hoas <s"), "hoas");
}

// Guard: a vowel tone (circumflex/horn/breve) on a NON-final char must NOT be
// treated as the last transform after restore. A trailing consonant means the
// consonant was the last keystroke, so a following vowel appends (matching
// continuous typing) instead of falsely reverting the tone.
#[test]
fn telex_closed_syllable_vowel_tone_not_reverted_after_restore() {
    // "tuân" + 'a': continuous appends -> "tuâna"; restore must match (not "tuana").
    assert_eq!(telex("tuaana"), "tuâna");
    assert_eq!(telex("tuaan <a"), "tuâna");
    // "tiêng" + 'e' likewise.
    assert_eq!(telex("tieenge"), "tiênge");
    assert_eq!(telex("tieeng <e"), "tiênge");
}
