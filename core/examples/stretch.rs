//! Typing-experience audit: stretch the last vowel of every toned Vietnamese syllable
//! (mùa → mùaaaa) and check every key: the word grows by one letter, the tone stays on its vowel
//! and no mark appears that the syllable did not have. `... --example stretch -- [ar] < pairs`
use gonhanh_core::engine::Engine;
use gonhanh_core::utils::type_word;
use std::io::BufRead;

const TONED: &str = "áàảãạắằẳẵặấầẩẫậéèẻẽẹếềểễệíìỉĩịóòỏõọốồổỗộớờởỡợúùủũụứừửữựýỳỷỹỵ";
const MARKED: &str = "ăắằẳẵặâấầẩẫậêếềểễệôốồổỗộơớờởỡợưứừửữựđ";

fn main() {
    let ar = std::env::args().any(|a| a == "ar");
    let (mut n, mut bad, mut shown) = (0, 0, Vec::new());
    for line in std::io::stdin().lock().lines().map_while(Result::ok) {
        let Some((typed, _)) = line.split_once('\t') else { continue };
        let Some(v) = typed.chars().rev().find(|c| "aeiouy".contains(*c)) else { continue };
        let screen = |t: &str| {
            let mut e = Engine::new();
            e.set_english_auto_restore(ar);
            type_word(&mut e, t)
        };
        let base = screen(typed);
        let Some(at) = base.chars().position(|c| TONED.contains(c)) else { continue };
        n += 1;
        let base_marks = base.chars().filter(|c| MARKED.contains(*c)).count();
        let mut why = None;
        for k in 1..=5 {
            let s = screen(&format!("{typed}{}", v.to_string().repeat(k)));
            let marks = s.chars().filter(|c| MARKED.contains(*c)).count();
            if s.chars().position(|c| TONED.contains(c)) != Some(at) {
                why = Some(format!("tone moved at +{k}: {s}"));
            } else if marks > base_marks {
                why = Some(format!("new mark at +{k}: {s}"));
            } else if s.chars().count() != base.chars().count() + k {
                why = Some(format!("length at +{k}: {s}"));
            }
            if why.is_some() {
                break;
            }
        }
        if let Some(w) = why {
            bad += 1;
            shown.push(format!("{typed}+{v}: {base} → {w}"));
        }
    }
    println!("{n} toned syllables, {bad} with a jump while the last vowel is stretched");
    println!("{}", shown.join("\n"));
}
