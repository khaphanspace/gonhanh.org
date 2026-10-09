//! Two English words typed with no space (helloworld): how often does free typing change them,
//! beyond what normal typing already changes? `... --example en_compound`
use gonhanh_core::engine::Engine;
use gonhanh_core::utils::type_word;
use std::collections::HashSet;

fn main() {
    let words: Vec<String> = std::fs::read_to_string("src/data/english_dict_merged.txt")
        .unwrap()
        .lines()
        .filter(|w| w.len() >= 3 && w.chars().all(|c| c.is_ascii_lowercase()))
        .map(String::from)
        .collect();
    let mut seed = 99u64;
    let mut next = || {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (seed >> 33) as usize
    };
    let pairs: Vec<String> = (0..4000)
        .map(|_| {
            format!(
                "{}{}",
                words[next() % words.len()],
                words[next() % words.len()]
            )
        })
        .collect();
    let run = |free: bool, p: &str| {
        let mut e = Engine::new();
        e.set_english_auto_restore(true);
        if free {
            e.set_free_tone(true);
            e.set_allow_foreign_consonants(true);
        }
        type_word(&mut e, &format!("{p} ")).trim_end().to_string()
    };
    let normal: HashSet<&String> = pairs.iter().filter(|p| run(false, p) != **p).collect();
    let extra: Vec<String> = pairs
        .iter()
        .filter(|p| !normal.contains(p) && run(true, p) != **p)
        .map(|p| format!("{p}→{}", run(true, p)))
        .collect();
    println!(
        "changed by normal typing: {} of {}",
        normal.len(),
        pairs.len()
    );
    println!("changed only by free typing: {}", extra.len());
    for e in extra.iter().take(70) {
        println!("{e}");
    }
}
