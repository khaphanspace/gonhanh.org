use gonhanh_core::data::keys;
use gonhanh_core::engine::matrix::Processor;

fn main() {
    let mut p = Processor::new();

    // Type "tiengs"
    println!("=== Typing 'tiengs' ===");
    for (label, key) in [
        ("t", keys::T),
        ("i", keys::I),
        ("e", keys::E),
        ("n", keys::N),
        ("g", keys::G),
        ("s", keys::S),
    ] {
        let r = p.process(key, false, false);
        println!(
            "  {} → state={}, buffer='{}', result={:?}",
            label,
            p.state(),
            p.buffer().to_full_string(),
            r
        );
    }
    println!("\nFinal buffer: {}", p.buffer().to_full_string());
}
