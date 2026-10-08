use gonhanh_core::engine::Engine;
use gonhanh_core::utils::type_word;
fn main() {
    let cases = [
        "wes ", "wa ", "Wes ", "zij ", "Zias ", "khphas ", "qcaos ", "qcaoz ", "kpas ", "xyzas ",
        "hoas ", "toanf ", "hoaf ", "ddojc ", "djoc ", "dojc ", "ngoaif ", "hfoa ",
    ];
    for (free, foreign) in [(false, false), (true, false), (true, true), (false, true)] {
        let mut out = Vec::new();
        for c in cases {
            let mut e = Engine::new();
            e.set_free_tone(free);
            e.set_allow_foreign_consonants(foreign);
            out.push(format!("{}→{}", c.trim(), type_word(&mut e, c).trim()));
        }
        println!("free={free} foreign={foreign}: {}", out.join("  "));
    }
}
