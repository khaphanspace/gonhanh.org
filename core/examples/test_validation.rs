use gonhanh_core::engine::matrix::is_buffer_invalid_vietnamese;

fn main() {
    // Test "tôt" - circumflex + closed syllable + no tone
    let result = is_buffer_invalid_vietnamese("tôt");
    println!("is_buffer_invalid_vietnamese('tôt') = {}", result);

    // Test "tốt" - circumflex + closed syllable + sắc tone
    let result2 = is_buffer_invalid_vietnamese("tốt");
    println!("is_buffer_invalid_vietnamese('tốt') = {}", result2);

    // Test "kêp" - circumflex + closed + no tone (p final)
    let result3 = is_buffer_invalid_vietnamese("kêp");
    println!("is_buffer_invalid_vietnamese('kêp') = {}", result3);
}
