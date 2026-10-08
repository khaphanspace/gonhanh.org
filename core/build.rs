//! Packs the word lists into static blobs so the engine pays nothing at startup.
//!
//! Each list becomes `<name>.blob` (sorted words, each followed by '\n') and `<name>.idx`
//! (little-endian u32 offsets, n+1 entries). Lookups binary-search the blob in place:
//! no HashSet, no heap, no first-keystroke initialisation (src/data/lexicon.rs).
//! Entries are kept exactly as written in the source files (case included).

use std::{env, fs, path::PathBuf};

fn main() {
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    // (name, source file, header lines to skip)
    for (name, path, skip) in [
        ("en", "src/data/english_dict_merged.txt", 0),
        ("vi", "src/data/dictionaries/vi.dic", 1),
        ("keep", "src/data/dictionaries/keep.dic", 1),
        ("doubles", "src/data/telex_doubles.txt", 0),
    ] {
        println!("cargo:rerun-if-changed={path}");
        let text = fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
        let mut words: Vec<&str> = text.lines().skip(skip).filter(|l| !l.is_empty()).collect();
        words.sort_unstable();
        words.dedup();
        let (mut blob, mut idx) = (Vec::new(), Vec::new());
        for w in &words {
            idx.extend((blob.len() as u32).to_le_bytes());
            blob.extend(w.as_bytes());
            blob.push(b'\n');
        }
        idx.extend((blob.len() as u32).to_le_bytes());
        fs::write(out.join(format!("{name}.blob")), blob).unwrap();
        fs::write(out.join(format!("{name}.idx")), idx).unwrap();
    }
    println!("cargo:rerun-if-changed=build.rs");
}
