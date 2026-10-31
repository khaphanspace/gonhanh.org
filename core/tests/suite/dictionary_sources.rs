//! The English dictionary is merged from every file in `src/data/dictionaries/en/` (general
//! words, tech terms, brand names, chat words...). This checks that each source file is well
//! formed and that the engine recognises what it lists: a word of any file, typed in Telex with
//! auto-restore on, comes out as typed. A word that cannot (it is also a Vietnamese syllable of
//! `vi.dic`, like "test" → "tét") is reported, and the share of such words is bounded per file.

use crate::common::type_word;
use gonhanh_core::engine::Engine;
use std::collections::HashMap;
use std::fs;

const DIR: &str = "src/data/dictionaries/en";

/// (file name, words) of every `*.txt` in the directory.
fn sources() -> Vec<(String, Vec<String>)> {
    let mut files: Vec<_> = fs::read_dir(DIR)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "txt"))
        .collect();
    files.sort();
    files
        .into_iter()
        .map(|p| {
            let words = fs::read_to_string(&p)
                .unwrap()
                .lines()
                .map(|l| l.split('#').next().unwrap_or("").trim().to_lowercase())
                .filter(|w| !w.is_empty())
                .collect();
            (p.file_name().unwrap().to_string_lossy().into_owned(), words)
        })
        .collect()
}

fn typed(word: &str) -> String {
    let mut e = Engine::new();
    e.set_english_auto_restore(true);
    type_word(&mut e, &format!("{word} "))
        .trim_end()
        .to_string()
}

#[test]
fn every_source_file_is_well_formed() {
    let all = sources();
    assert!(all.len() >= 2, "expected several dictionary files in {DIR}");
    for (name, words) in all.iter().filter(|(n, _)| n != "general.txt") {
        for w in words {
            assert!(
                w.chars().all(|c| c.is_ascii_lowercase()),
                "{name}: {w:?} is not lower-case letters only"
            );
            assert!(
                w.len() >= 2,
                "{name}: {w:?} is too short to be worth a line"
            );
        }
    }
    // strings of one repeated letter (ww, ss, aaaa) are not words, and as dictionary entries they
    // win over the cancel the user typed on purpose
    for (name, words) in &all {
        for w in words {
            let b = w.as_bytes();
            assert!(
                !(b.len() >= 2 && b.iter().all(|&c| c == b[0])),
                "{name}: {w:?} is no word"
            );
        }
    }
    // a word listed in two files hides which kind it is: keep each word in one file
    let mut seen: HashMap<&str, &str> = HashMap::new();
    let mut dups = Vec::new();
    for (name, words) in &all {
        for w in words {
            if let Some(first) = seen.insert(w, name) {
                if first != name {
                    dups.push(format!("{w} ({first} and {name})"));
                }
            }
        }
    }
    assert!(dups.is_empty(), "words in more than one file: {dups:?}");
}

/// Curated files are checked word by word; reference lists (general, base) are too large and too
/// full of rare words that are also Vietnamese syllables for that.
const CURATED_MAX: usize = 2000;

#[test]
fn the_engine_recognises_what_each_source_lists() {
    for (name, words) in sources()
        .into_iter()
        .filter(|(_, w)| w.len() <= CURATED_MAX)
    {
        let lost: Vec<String> = words
            .iter()
            .filter_map(|w| {
                let got = typed(w);
                (got != *w).then(|| format!("{w}→{got}"))
            })
            .collect();
        let share = lost.len() as f64 / words.len() as f64;
        println!(
            "{name}: {} words, {} not kept as typed: {}",
            words.len(),
            lost.len(),
            lost.join(" ")
        );
        assert!(
            share <= 0.10,
            "{name}: {:.0}% of the words come out changed: {lost:?}",
            share * 100.0
        );
    }
}
