//! Exhaustive audit of the phonology against the dictionary (Constitution I1-I6).
//!
//! The dictionary is the oracle: every syllable in `vi.dic` (minus the documented non-syllables)
//! must be Complete, and the tone mark must land where the dictionary puts it. Failures are
//! grouped by cause and listed all at once, so one run shows everything still wrong.

use gonhanh_core::phonology::{tone_index, validate, Opts, Tone, Unit, Validity};
use std::collections::BTreeMap;

const DIC: &str = "src/data/dictionaries";

fn lines(file: &str) -> Vec<String> {
    std::fs::read_to_string(format!("{DIC}/{file}"))
        .unwrap()
        .lines()
        .map(|l| l.split('#').next().unwrap().trim().to_lowercase())
        .filter(|l| !l.is_empty())
        .collect()
}

fn decode(word: &str) -> Option<(Vec<Unit>, Tone, Option<usize>)> {
    let mut tone = Tone::Ngang;
    let mut at = None;
    let mut units = Vec::new();
    for (i, c) in word.chars().enumerate() {
        let (u, t) = Unit::from_char(c)?;
        if t != Tone::Ngang {
            tone = t;
            at = Some(i);
        }
        units.push(u);
    }
    Some((units, tone, at))
}

/// Dictionary entries the regular grammar must reject because the *spelling* is exceptional
/// (loan words / letter names), each with the reason. They are valid words but not valid
/// syllables under I2; the engine handles them via names/keep lexicons, never via the grammar.
const SPELLING_EXCEPTIONS: &[(&str, &str)] = &[
    ("ka", "letter name, k before a"),
    ("gen", "loan: g before e"),
    ("gỵa", "unexplained dictionary entry (g before y)"),
    ("tout", "loan: nucleus ou"),
    ("têt", "loan/slang: stop coda with ngang tone"),
    ("xit", "loan: stop coda with ngang tone"),
    ("gip", "loan (jeep): stop coda with ngang tone"),
    ("ping", "loan: -ing (docs §10.1: -ing takes no tone)"),
];

#[test]
fn every_dictionary_syllable_is_complete() {
    let non: Vec<String> = lines("vi-non-syllables.txt");
    let mut by_cause: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut checked = 0;
    for w in lines("vi.dic").into_iter().skip(1) {
        if non.contains(&w) {
            continue;
        }
        checked += 1;
        let Some((units, tone, _)) = decode(&w) else {
            by_cause.entry("undecodable".into()).or_default().push(w);
            continue;
        };
        let got = validate(
            &units,
            tone,
            &Opts {
                names: false,
                ..Opts::default()
            },
        );
        if got == Validity::Complete {
            continue;
        }
        if let Some((_, why)) = SPELLING_EXCEPTIONS.iter().find(|(x, _)| *x == w) {
            by_cause
                .entry(format!("known exception: {why}"))
                .or_default()
                .push(w);
            continue;
        }
        by_cause.entry(format!("{got:?}")).or_default().push(w);
    }
    report("dictionary recall", checked, &by_cause);
    let unexplained: usize = by_cause
        .iter()
        .filter(|(k, _)| !k.starts_with("known exception"))
        .map(|(_, v)| v.len())
        .sum();
    assert_eq!(
        unexplained, 0,
        "grammar rejects dictionary syllables, see report above"
    );
}

#[test]
fn tone_lands_where_the_dictionary_puts_it() {
    let non = lines("vi-non-syllables.txt");
    let mut by_cause: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut checked = 0;
    for w in lines("vi.dic").into_iter().skip(1) {
        if non.contains(&w) {
            continue;
        }
        let Some((units, tone, Some(at))) = decode(&w) else {
            continue;
        };
        if tone == Tone::Ngang {
            continue;
        }
        checked += 1;
        // oa/oe/uy legitimately appear in both styles in the dictionary: accept either
        let ok = [true, false]
            .iter()
            .any(|&modern| tone_index(&units, modern) == Some(at));
        if !ok {
            by_cause
                .entry(format!(
                    "modern={:?} old={:?}",
                    tone_index(&units, true),
                    tone_index(&units, false)
                ))
                .or_default()
                .push(format!("{w}(dict@{at})"));
        }
    }
    report("tone placement", checked, &by_cause);
    assert!(
        by_cause.is_empty(),
        "tone placement disagrees with the dictionary"
    );
}

/// Over-generation, measured on tone-less skeletons (initial + nucleus + coda): how many skeletons
/// the grammar accepts that no dictionary syllable uses. Accidental lexical gaps are normal (the
/// grammar is not a dictionary); the ceiling keeps the rules from loosening silently.
#[test]
fn over_generation_stays_bounded() {
    let strip = |w: &str| -> String {
        w.chars()
            .map(|c| Unit::from_char(c).map_or(c, |(u, _)| u.to_char()))
            .collect()
    };
    let attested: std::collections::HashSet<String> = lines("vi.dic")
        .into_iter()
        .skip(1)
        .map(|w| strip(&w))
        .collect();
    let initials = [
        "", "b", "c", "ch", "d", "đ", "g", "gh", "gi", "h", "k", "kh", "l", "m", "n", "ng", "ngh",
        "nh", "p", "ph", "qu", "r", "s", "t", "th", "tr", "v", "x",
    ];
    let nuclei: Vec<String> = gonhanh_core::phonology::tables::NUCLEUS_CODAS
        .iter()
        .map(|(pat, _)| {
            pat.iter()
                .map(|&(b, m)| Unit::from_code(b, m).to_char())
                .collect()
        })
        .collect();
    let codas = ["", "c", "ch", "m", "n", "ng", "nh", "p", "t", "k"];
    let (mut accepted, mut unattested) = (0usize, 0usize);
    for ini in initials {
        for nuc in &nuclei {
            for coda in codas {
                let word = format!("{ini}{nuc}{coda}");
                let Some((units, _, _)) = decode(&word) else {
                    continue;
                };
                // a stop coda needs sắc/nặng to be Complete; any tone makes the skeleton comparable
                let tone = if matches!(coda, "c" | "ch" | "p" | "t" | "k") {
                    Tone::Sac
                } else {
                    Tone::Ngang
                };
                if validate(
                    &units,
                    tone,
                    &Opts {
                        names: false,
                        ..Opts::default()
                    },
                ) == Validity::Complete
                {
                    accepted += 1;
                    if !attested.contains(&word) {
                        unattested += 1;
                    }
                }
            }
        }
    }
    println!(
        "\n=== over-generation: {accepted} skeletons accepted, {} attested in vi.dic, {unattested} unattested ({:.0}%) ===",
        attested.len(),
        100.0 * unattested as f64 / accepted as f64
    );
    assert!(
        unattested <= OVERGEN_CEILING,
        "grammar loosened: {unattested} unattested skeletons > {OVERGEN_CEILING}"
    );
}

const OVERGEN_CEILING: usize = 1_800;

fn report(title: &str, checked: usize, by_cause: &BTreeMap<String, Vec<String>>) {
    println!("\n=== {title}: {checked} checked ===");
    for (cause, items) in by_cause {
        let sample: Vec<&str> = items.iter().take(12).map(|s| s.as_str()).collect();
        println!("  {:>5} x {cause}: {}", items.len(), sample.join(" "));
    }
}
