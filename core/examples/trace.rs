//! Screen after every key: `cargo run -q --features engine_v2 --example trace -- [telex|vni][,ar][,free][,w] word...`
//! `_` is space, `<` is backspace. Shows what the user sees while typing, not just the end result.
use gonhanh_core::engine::Engine;
use gonhanh_core::utils::type_word;

fn main() {
    let mut args = std::env::args().skip(1);
    let flags = args.next().unwrap_or_else(|| "telex".into());
    let has = |f: &str| flags.split(',').any(|x| x == f);
    for w in args {
        let mut e = Engine::new();
        e.set_method(has("vni") as u8);
        e.set_english_auto_restore(has("ar"));
        if has("free") {
            e.set_free_tone(true);
            e.set_allow_foreign_consonants(true);
        }
        if has("nw") {
            e.set_skip_w_shortcut(true);
        }
        let mut typed = String::new();
        let mut steps = Vec::new();
        for c in w.replace('_', " ").chars() {
            typed.push(c);
            let mut f = Engine::new();
            f.set_method(has("vni") as u8);
            f.set_english_auto_restore(has("ar"));
            if has("free") {
                f.set_free_tone(true);
                f.set_allow_foreign_consonants(true);
            }
            if has("nw") {
                f.set_skip_w_shortcut(true);
            }
            steps.push(type_word(&mut f, &typed));
        }
        drop(e);
        println!("{w}\n  {}", steps.join(" | "));
    }
}
