//! Packs the word lists into static blobs so the engine pays nothing at startup. The English list
//! is merged from every file in `src/data/dictionaries/en/` (general words, tech terms, brand
//! names...): to support a new kind of word, add a file there.
//!
//! Each list becomes `<name>.blob` (sorted words, each followed by '\n') and `<name>.idx`
//! (little-endian u32 offsets, n+1 entries). Lookups binary-search the blob in place:
//! no HashSet, no heap, no first-keystroke initialisation (src/data/lexicon.rs). `<name>.bkt`
//! holds, for each pair of first letters (aa..zz), the first entry that is not smaller than the
//! pair: a lookup starts in a slice of about 25 entries instead of the whole list.
//! Entries of single-file lists are kept as written (case included); merged ones are lowercased.

use std::{env, fs, path::PathBuf};

/// Where a list comes from.
enum Source {
    /// One file; the first `skip` lines are a header.
    File(&'static str, usize),
    /// Every `*.txt` in a directory, merged: one file per kind of word (general, tech, brands...),
    /// so a new kind is a new file and no code changes. `#` starts a comment.
    Dir(&'static str),
}

/// (file the word came from, word); single files have no origin name.
fn read_words(source: &Source) -> Vec<(String, String)> {
    match source {
        Source::File(path, skip) => {
            println!("cargo:rerun-if-changed={path}");
            let text = fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
            text.lines()
                .skip(*skip)
                .filter(|l| !l.is_empty())
                .map(|l| (String::new(), l.to_string()))
                .collect()
        }
        Source::Dir(dir) => {
            println!("cargo:rerun-if-changed={dir}");
            let mut files: Vec<PathBuf> = fs::read_dir(dir)
                .unwrap_or_else(|e| panic!("{dir}: {e}"))
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| p.extension().is_some_and(|x| x == "txt"))
                .collect();
            files.sort();
            let mut words = Vec::new();
            for f in files {
                println!("cargo:rerun-if-changed={}", f.display());
                let text =
                    fs::read_to_string(&f).unwrap_or_else(|e| panic!("{}: {e}", f.display()));
                for line in text.lines() {
                    let w = line.split('#').next().unwrap_or("").trim().to_lowercase();
                    if !w.is_empty() {
                        let origin = f
                            .file_stem()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .into_owned();
                        words.push((origin, w));
                    }
                }
            }
            words
        }
    }
}

fn has_double_letter(w: &str) -> bool {
    w.as_bytes()
        .windows(2)
        .any(|p| p[0] == p[1] && p[0].is_ascii_alphabetic())
}

fn main() {
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let mut curated_doubles: Vec<String> = Vec::new();
    for (name, source) in [
        ("en", Source::Dir("src/data/dictionaries/en")),
        ("vi", Source::File("src/data/dictionaries/vi.dic", 1)),
        ("keep", Source::File("src/data/dictionaries/keep.dic", 1)),
        ("doubles", Source::File("src/data/telex_doubles.txt", 0)),
    ] {
        let mut all = read_words(&source);
        if name == "doubles" {
            // Curated English words (tech, brands, chat...) spelled with a doubled letter keep it
            // (terraform, github's ss...): they join the Telex-doubles list without a second edit.
            all.extend(curated_doubles.iter().map(|w| (String::new(), w.clone())));
        }
        if name == "en" {
            curated_doubles = all
                .iter()
                .filter(|(origin, w)| origin != "general" && has_double_letter(w))
                .map(|(_, w)| w.clone())
                .collect();
        }
        let all: Vec<String> = all.into_iter().map(|(_, w)| w).collect();
        let mut words: Vec<&str> = all.iter().map(String::as_str).collect();
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
