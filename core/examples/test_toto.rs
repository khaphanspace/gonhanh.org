use gonhanh_core::data::keys;
use gonhanh_core::engine::matrix::Processor;

fn main() {
    let mut p = Processor::new();

    // Type "toto" - should be tôt (delayed circumflex)
    println!("=== Typing 'toto' (no space) ===");
    for (label, key) in [
        ("t", keys::T),
        ("o", keys::O),
        ("t", keys::T),
        ("o", keys::O),
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

    // Check validation
    let buf = p.buffer().to_full_string();
    let is_invalid = gonhanh_core::engine::matrix::is_buffer_invalid_vietnamese(&buf);
    println!("is_buffer_invalid_vietnamese('{}') = {}", buf, is_invalid);
}
