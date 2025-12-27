use gonhanh_core::data::keys;
use gonhanh_core::engine::matrix::Processor;

fn main() {
    let mut p = Processor::new();

    // Type "nhanaj" - should be nhận (â with nặng tone)
    println!("=== Typing 'nhanaj' ===");
    for (label, key) in [
        ("n", keys::N),
        ("h", keys::H),
        ("a", keys::A),
        ("n", keys::N),
        ("a", keys::A),
        ("j", keys::J),
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
