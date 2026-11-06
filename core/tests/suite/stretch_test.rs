//! A stretched ending (nhỉiiii, nháaaa) over every open Vietnamese syllable of the pairs list:
//! the tone typed first or last gives the same word, the tone is never lost, and what is shown while
//! typing is what stays at the space. The length of the stretch is not compared: the first repeat is a
//! circumflex try that eats a key when the tone comes last.

use gonhanh_core::data::lexicon::{EN, REF};
use gonhanh_core::engine::Engine;
use gonhanh_core::utils::type_word;

const TONED: &str = "áàảãạắằẳẵặấầẩẫậéèẻẽẹếềểễệíìỉĩịóòỏõọốồổỗộớờởỡợúùủũụứừửữựýỳỷỹỵ";
const CIRC: &str = "âấầẩẫậêếềểễệôốồổỗộ";
const VOWELS: &str = "aeiouyăâêôơưáàảãạắằẳẵặấầẩẫậéèẻẽẹếềểễệíìỉĩịóòỏõọốồổỗộớờởỡợúùủũụứừửữựýỳỷỹỵ";

fn plain(c: char) -> char {
    for (base, set) in [
        ('a', "aăâáàảãạắằẳẵặấầẩẫậ"),
        ('e', "eêéèẻẽẹếềểễệ"),
        ('i', "iíìỉĩị"),
        ('o', "oôơóòỏõọốồổỗộớờởỡợ"),
        ('u', "uưúùủũụứừửữự"),
        ('y', "yýỳỷỹỵ"),
    ] {
        if set.contains(c) {
            return base;
        }
    }
    c
}

/// The word with its trailing run of one letter cut to one.
fn squash(s: &str) -> String {
    let mut cs: Vec<char> = s.chars().collect();
    while cs.len() >= 2 && cs[cs.len() - 1] == cs[cs.len() - 2] {
        cs.pop();
    }
    cs.into_iter().collect()
}

/// `s` with every run of 3+ equal letters cut to `to` letters.
fn cut_runs(s: &str, to: usize) -> String {
    let cs: Vec<char> = s.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < cs.len() {
        let run = cs[i..].iter().take_while(|c| **c == cs[i]).count();
        out.extend(std::iter::repeat(cs[i]).take(if run >= 3 { to } else { run }));
        i += run;
    }
    out
}

fn screen(typed: &str, free: bool) -> String {
    let mut e = Engine::new();
    e.set_english_auto_restore(true);
    if free {
        e.set_free_tone(true);
        e.set_allow_foreign_consonants(true);
    }
    type_word(&mut e, typed)
}

#[test]
fn a_stretched_ending_is_the_same_whichever_key_comes_last() {
    let pairs = std::fs::read_to_string("tests/data/vietnamese_telex_pairs.txt").unwrap();
    let (mut n, mut lost, mut space, mut order) = (0, Vec::new(), Vec::new(), Vec::new());
    for line in pairs.lines() {
        let Some((typed, word)) = line.split_once('\t') else {
            continue;
        };
        // tone key last, an open syllable ending in a single vowel (a doubled one is a circumflex)
        let Some(tone) = typed.chars().last().filter(|c| "sfrxj".contains(*c)) else {
            continue;
        };
        let base = &typed[..typed.len() - 1];
        let mut tail = base.chars().rev();
        let (Some(v), second) = (tail.next().filter(|c| "aeiouy".contains(*c)), tail.next()) else {
            continue;
        };
        if second == Some(v)
            || !word.chars().any(|c| TONED.contains(c))
            || !word.chars().last().is_some_and(|c| VOWELS.contains(c))
            || word.chars().last().map(plain) != Some(v)
            // a circumflex typed after other letters (`dede` → `đê`): the last key is a modifier
            || word.chars().last().is_some_and(|c| CIRC.contains(c))
        {
            continue;
        }
        for k in 2..=4 {
            let run = v.to_string().repeat(k);
            let last = format!("{base}{run}{tone}");
            let first = format!("{typed}{run}");
            // letters that start an English word (viiis: the numeral viii) stay English in free typing
            if EN.begins_inflected(&last) || REF.begins_inflected(&last) {
                continue;
            }
            // a stretched English word (beeef: beef) is given back as English at the space
            if EN.contains_lower(&cut_runs(&last, 2)) {
                continue;
            }
            for free in [false, true] {
                n += 1;
                let live = screen(&last, free);
                let tone_first = screen(&first, free);
                if !live.chars().any(|c| TONED.contains(c)) {
                    lost.push(format!("{last} free={free}: {live}"));
                }
                // free typing reads `biri…` as two syllables, so the tone-first form is raw there
                if free
                    && live != last
                    && tone_first != first
                    && squash(&live) != squash(&tone_first)
                {
                    order.push(format!("{last}: {live} / {first}: {tone_first}"));
                }
                // with English restore on, some letters are an English word (beer, roof) and win at the space
                if free && format!("{live} ") != screen(&format!("{last} "), true) {
                    space.push(format!("{last}: {live}"));
                }
            }
        }
    }
    assert!(n > 5000, "only {n} stretched words");
    let report =
        |what: &str, v: &[String]| format!("{} {what}: {:?}", v.len(), &v[..v.len().min(8)]);
    assert!(lost.is_empty(), "{}", report("lost the tone", &lost));
    assert!(
        space.is_empty(),
        "{}",
        report("changed at the space", &space)
    );
    assert!(
        order.is_empty(),
        "{}",
        report("depend on the key order", &order)
    );
}
