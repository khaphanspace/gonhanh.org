//! Quick probe: `cargo run -q --example try -- telex|vni|telex_ar|vni_ar word...`
use gonhanh_core::engine::Engine;
use gonhanh_core::utils::type_word;

fn main() {
    let mut args = std::env::args().skip(1);
    let mode = args.next().unwrap_or_else(|| "telex".into());
    for w in args {
        let mut e = Engine::new();
        if mode.starts_with("vni") {
            e.set_method(1);
        }
        if mode.ends_with("_ar") {
            e.set_english_auto_restore(true);
        }
        let w = w.replace('_', " ");
        println!("{w:?} -> {:?}", type_word(&mut e, &w));
    }
}
