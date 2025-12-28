use gonhanh_core::data::keys;
use gonhanh_core::engine::matrix::Processor;

#[test]
fn debug_thiss_class() {
    // Test "thiss"
    eprintln!("--- Testing thiss ---");
    let mut processor = Processor::new();
    let key_seq = [keys::T, keys::H, keys::I, keys::S, keys::S];
    for (i, &key) in key_seq.iter().enumerate() {
        processor.process(key, false, false);
        let buffer = processor.buffer().to_full_string();
        let raw_chars: Vec<char> = processor.raw().restore_all();
        let raw_str: String = raw_chars.iter().collect();
        eprintln!(
            "[{}] Key {}: buffer='{}', raw='{}'",
            i, key, buffer, raw_str
        );
    }
    let raw = processor.raw().restore_all();
    let raw_str: String = raw.iter().collect();
    let buffer = processor.buffer().to_full_string();
    eprintln!(
        "Final: raw='{}' ({} chars), buffer='{}' ({} chars)",
        raw_str,
        raw_str.len(),
        buffer,
        buffer.len()
    );

    // Test "class"
    eprintln!("\n--- Testing class ---");
    let mut processor2 = Processor::new();
    let key_seq2 = [keys::C, keys::L, keys::A, keys::S, keys::S];
    for (i, &key) in key_seq2.iter().enumerate() {
        processor2.process(key, false, false);
        let buffer = processor2.buffer().to_full_string();
        let raw_chars: Vec<char> = processor2.raw().restore_all();
        let raw_str: String = raw_chars.iter().collect();
        eprintln!(
            "[{}] Key {}: buffer='{}', raw='{}'",
            i, key, buffer, raw_str
        );
    }
    let raw2 = processor2.raw().restore_all();
    let raw_str2: String = raw2.iter().collect();
    let buffer2 = processor2.buffer().to_full_string();
    eprintln!(
        "Final: raw='{}' ({} chars), buffer='{}' ({} chars)",
        raw_str2,
        raw_str2.len(),
        buffer2,
        buffer2.len()
    );
}
