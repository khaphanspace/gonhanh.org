use gonhanh_core::data::keys;
use gonhanh_core::engine::matrix::Processor;

fn main() {
    let mut p = Processor::new();

    println!("=== Typing 'o' ===");
    let r1 = p.process(keys::O, false, false);
    println!(
        "  state={}, buffer={}, result={:?}",
        p.state(),
        p.buffer().to_full_string(),
        r1
    );

    println!("\n=== Typing 'i' ===");
    let r2 = p.process(keys::I, false, false);
    println!(
        "  state={}, buffer={}, result={:?}",
        p.state(),
        p.buffer().to_full_string(),
        r2
    );

    println!("\n=== Typing second 'o' ===");
    let r3 = p.process(keys::O, false, false);
    println!(
        "  state={}, buffer={}, result={:?}",
        p.state(),
        p.buffer().to_full_string(),
        r3
    );

    println!("\nFinal buffer: {}", p.buffer().to_full_string());
}
