//! Differential harness: typing core v2 (`compose`) against the Vietnamese corpus, with engine v1
//! as reference. One run lists *every* remaining difference grouped by cause, so fixes go in
//! batches (change a rule, re-run the whole table once).
//!
//! Truth = the word in the corpus. Reported per method: v2 failures and the groups of v2
//! failures keyed by the first differing letter (expected → got), with what v1 does.

use gonhanh_core::compose::{Compose, Method, Options};
use gonhanh_core::engine::Engine;
use gonhanh_core::utils::type_word;
use std::collections::BTreeMap;

/// (word, typings) from "word<TAB>t1,t2,..." lines.
fn corpus(path: &str) -> Vec<(String, Vec<String>)> {
    std::fs::read_to_string(path)
        .unwrap()
        .lines()
        .filter(|l| !l.starts_with('#') && l.contains('\t'))
        .filter_map(|l| {
            let mut it = l.split('\t');
            Some((
                it.next()?.to_string(),
                it.next()?.split(',').map(String::from).collect(),
            ))
        })
        .collect()
}

struct Row {
    typed: String,
    want: String,
    v1: String,
    v2: String,
}

fn first_diff(want: &str, got: &str) -> String {
    let (w, g): (Vec<char>, Vec<char>) = (want.chars().collect(), got.chars().collect());
    let i = w.iter().zip(&g).take_while(|(a, b)| a == b).count();
    format!(
        "{}→{}",
        w.get(i).map_or("∅".into(), |c| c.to_string()),
        g.get(i).map_or("∅".into(), |c| c.to_string())
    )
}

fn run(items: &[(String, String)], method: Method) -> Vec<Row> {
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let chunk = items.len().div_ceil(threads).max(1);
    std::thread::scope(|s| {
        let hs: Vec<_> = items
            .chunks(chunk)
            .map(|part| {
                s.spawn(move || {
                    let mut e = Engine::new();
                    e.set_method(if method == Method::Vni { 1 } else { 0 });
                    let modern = Options {
                        method,
                        ..Options::default()
                    };
                    let old = Options {
                        modern_tone: false,
                        ..modern
                    };
                    let mut bad = Vec::new();
                    for (want, typed) in part {
                        let Some(v2) = Compose::type_str(modern, typed) else {
                            continue;
                        };
                        // the corpus mixes oà/òa style: accept either
                        let v2_old = Compose::type_str(old, typed).unwrap_or_default();
                        if v2 != *want && v2_old != *want {
                            e.clear_all();
                            let v1 = type_word(&mut e, typed);
                            bad.push(Row {
                                typed: typed.clone(),
                                want: want.clone(),
                                v1,
                                v2,
                            });
                        }
                    }
                    bad
                })
            })
            .collect();
        hs.into_iter().flat_map(|h| h.join().unwrap()).collect()
    })
}

fn report(title: &str, total: usize, rows: &[Row]) {
    let same = rows.iter().filter(|r| r.v1 == r.v2).count();
    println!(
        "\n=== {title}: {} / {total} v2 failures ({same} where v1 gives the same) ===",
        rows.len()
    );
    let mut groups: BTreeMap<String, Vec<&Row>> = BTreeMap::new();
    for r in rows {
        groups
            .entry(first_diff(&r.want, &r.v2))
            .or_default()
            .push(r);
    }
    let mut g: Vec<_> = groups.into_iter().collect();
    g.sort_by_key(|(_, v)| std::cmp::Reverse(v.len()));
    for (k, v) in g.iter().take(25) {
        let ex: Vec<String> = v
            .iter()
            .take(4)
            .map(|r| format!("{}→{} (want {}, v1 {})", r.typed, r.v2, r.want, r.v1))
            .collect();
        println!("  {:>6} x {k:<8} {}", v.len(), ex.join(" | "));
    }
    if let Some(dir) = std::env::var_os("COMPOSE_DIFF_OUT") {
        let body: String = rows
            .iter()
            .map(|r| format!("{}\t{}\t{}\t{}\n", r.typed, r.want, r.v1, r.v2))
            .collect();
        std::fs::write(
            std::path::Path::new(&dir).join(format!("{}.tsv", title.replace(' ', "_"))),
            body,
        )
        .ok();
    }
}

#[test]
fn compose_v2_against_vietnamese_corpus() {
    let items: Vec<(String, String)> = corpus("tests/data/vietnamese_22k_typing_variants.txt")
        .into_iter()
        .flat_map(|(w, ts)| ts.into_iter().map(move |t| (w.clone(), t)))
        .collect();
    let rows = run(&items, Method::Telex);
    report("telex 22k variants", items.len(), &rows);
}

/// ("typing<TAB>word") pairs, header lines skipped.
fn pairs(path: &str) -> Vec<(String, String)> {
    std::fs::read_to_string(path)
        .unwrap()
        .lines()
        .filter_map(|l| {
            let (typed, word) = l.split_once('\t')?;
            (!word.is_empty()
                && !typed.contains(' ')
                && typed.chars().all(|c| c.is_ascii_alphanumeric()))
            .then(|| (word.to_string(), typed.to_string()))
        })
        .collect()
}

#[test]
fn compose_v2_against_telex_pairs() {
    let items = pairs("tests/data/vietnamese_telex_pairs.txt");
    let rows = run(&items, Method::Telex);
    report("telex pairs", items.len(), &rows);
}

#[test]
fn compose_v2_against_vni_conversion() {
    let items: Vec<(String, String)> = pairs("tests/data/vietnamese_telex_pairs.txt")
        .into_iter()
        .map(|(word, _)| {
            let vni = crate::vietnamese_dict_test::vietnamese_to_vni(&word);
            (word, vni)
        })
        .filter(|(_, v)| v.chars().all(|c| c.is_ascii_alphanumeric()))
        .collect();
    let rows = run(&items, Method::Vni);
    report("vni", items.len(), &rows);
}
