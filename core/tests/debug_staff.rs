use gonhanh_core::data::keys;
use gonhanh_core::engine::matrix::Processor;

#[test]
fn debug_staff() {
    // Test what happens when typing "staff"
    let key_seq = [keys::S, keys::T, keys::A, keys::F, keys::F];

    let mut processor = Processor::new();
    for (i, &key) in key_seq.iter().enumerate() {
        let result = processor.process(key, false, false);
        let buffer = processor.buffer().to_full_string();
        let raw_chars: Vec<char> = processor.raw().restore_all();
        let raw_str: String = raw_chars.iter().collect();
        eprintln!(
            "[{}] Key {}: result={:?}, buffer='{}', raw='{}'",
            i, key, result, buffer, raw_str
        );
    }

    let raw = processor.raw().restore_all();
    let raw_str: String = raw.iter().collect();
    let buffer = processor.buffer().to_full_string();
    eprintln!(
        "\nFinal: raw='{}' ({} chars), buffer='{}' ({} chars)",
        raw_str,
        raw_str.len(),
        buffer,
        buffer.len()
    );

    // Now test "soffa"
    eprintln!("\n--- Testing soffa ---");
    let mut processor2 = Processor::new();
    let key_seq2 = [keys::S, keys::O, keys::F, keys::F, keys::A];
    for (i, &key) in key_seq2.iter().enumerate() {
        let result = processor2.process(key, false, false);
        let buffer = processor2.buffer().to_full_string();
        let raw_chars: Vec<char> = processor2.raw().restore_all();
        let raw_str: String = raw_chars.iter().collect();
        eprintln!(
            "[{}] Key {}: result={:?}, buffer='{}', raw='{}'",
            i, key, result, buffer, raw_str
        );
    }

    let raw2 = processor2.raw().restore_all();
    let raw_str2: String = raw2.iter().collect();
    let buffer2 = processor2.buffer().to_full_string();
    eprintln!(
        "\nFinal: raw='{}' ({} chars), buffer='{}' ({} chars)",
        raw_str2,
        raw_str2.len(),
        buffer2,
        buffer2.len()
    );
}
