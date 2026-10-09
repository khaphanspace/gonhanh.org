//! Packs the word lists into static blobs so the engine pays nothing at startup.
//!
//! Each list becomes `<name>.blob` (sorted words, each followed by '\n') and `<name>.idx`
//! (little-endian u32 offsets, n+1 entries). Lookups binary-search the blob in place:
//! no HashSet, no heap, no first-keystroke initialisation (src/data/lexicon.rs). `<name>.bkt`
//! holds, for each pair of first letters (aa..zz), the first entry that is not smaller than the
//! pair: a lookup starts in a slice of about 25 entries instead of the whole list.
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
        // bucket table: first entry >= each pair of lower-case letters, then the list length
        let mut bkt = Vec::new();
        for pair in 0..26 * 26 {
            let key = [b'a' + (pair / 26) as u8, b'a' + (pair % 26) as u8];
            let at = words.partition_point(|w| w.as_bytes() < &key[..]);
            bkt.extend((at as u32).to_le_bytes());
        }
        bkt.extend((words.len() as u32).to_le_bytes());
        fs::write(out.join(format!("{name}.bkt")), bkt).unwrap();
        fs::write(out.join(format!("{name}.blob")), blob).unwrap();
        fs::write(out.join(format!("{name}.idx")), idx).unwrap();
    }
    println!("cargo:rerun-if-changed=build.rs");
}
