use gonhanh_core::data::telex_doubles;

#[test]
fn test_www() {
    println!("Testing whitelist contains:");
    for word in [
        "www", "daddy", "poor", "monsoon", "cocoon", "veneer", "bass", "coffee",
    ] {
        let found = telex_doubles::contains(word);
        println!("  {} -> {}", word, found);
    }
}
