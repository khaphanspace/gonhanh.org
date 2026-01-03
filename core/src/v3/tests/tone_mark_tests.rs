//! Tone and mark combination tests
//!
//! Tests for Vietnamese tone marks and vowel marks.

use crate::v3::processor::Processor;
use crate::v3::tests::type_sequence;

#[test]
fn test_tone_sac() {
    let mut p = Processor::new();

    // Sắc tone (s key)
    // Note: actual transform behavior TBD in Phase 05
    // These tests verify the infrastructure works
    let _ = type_sequence(&mut p, "as");
}

#[test]
fn test_tone_huyen() {
    let mut p = Processor::new();

    // Huyền tone (f key)
    let _ = type_sequence(&mut p, "af");
}

#[test]
fn test_tone_hoi() {
    let mut p = Processor::new();

    // Hỏi tone (r key)
    let _ = type_sequence(&mut p, "ar");
}

#[test]
fn test_tone_nga() {
    let mut p = Processor::new();

    // Ngã tone (x key)
    let _ = type_sequence(&mut p, "ax");
}

#[test]
fn test_tone_nang() {
    let mut p = Processor::new();

    // Nặng tone (j key)
    let _ = type_sequence(&mut p, "aj");
}

#[test]
fn test_circumflex() {
    let mut p = Processor::new();

    // Circumflex (aa -> â, ee -> ê, oo -> ô)
    // Note: actual transform behavior TBD in Phase 05
    let _ = type_sequence(&mut p, "aa");
    p.clear();
    let _ = type_sequence(&mut p, "ee");
    p.clear();
    let _ = type_sequence(&mut p, "oo");
}

#[test]
fn test_horn() {
    let mut p = Processor::new();

    // Horn (ow -> ơ, uw -> ư)
    let _ = type_sequence(&mut p, "ow");
    p.clear();
    let _ = type_sequence(&mut p, "uw");
}

#[test]
fn test_breve() {
    let mut p = Processor::new();

    // Breve (aw -> ă)
    let _ = type_sequence(&mut p, "aw");
}

#[test]
fn test_stroke() {
    let mut p = Processor::new();

    // Stroke (dd -> đ)
    let _ = type_sequence(&mut p, "dd");
}
