//! Words from tests/data/english_100k.txt that change when typed with auto-restore: `word<TAB>actual`.
use gonhanh_core::engine::Engine;
use gonhanh_core::utils::type_word;

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "tests/data/english_100k.txt".into());
    let content = std::fs::read_to_string(path).unwrap();
    let mut e = Engine::new();
    e.set_english_auto_restore(true);
    let mut fails = 0;
    for w in content
        .lines()
        .map(str::trim)
        .filter(|w| !w.is_empty() && w.chars().all(|c| c.is_ascii_alphabetic()))
    {
        let got = type_word(&mut e, &format!("{w} "));
        if got != format!("{w} ") {
            fails += 1;
            println!("{w}\t{}", got.trim_end());
        }
    }
    eprintln!("fails: {fails}");
}
