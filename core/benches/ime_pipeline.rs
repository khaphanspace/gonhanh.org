//! Criterion benchmarks for Gõ Nhanh IME engine
//!
//! Run: cargo bench --bench ime_pipeline
//! Output: target/criterion/ (HTML reports)

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use gonhanh_core::data::keys;
use gonhanh_core::engine::Engine;

/// Benchmark single key processing (hot path)
fn bench_single_key(c: &mut Criterion) {
    let mut engine = Engine::new();
    engine.set_method(0); // Telex

    c.bench_function("ime_key_single", |b| {
        b.iter(|| engine.on_key(black_box(keys::A), false, false))
    });
}

/// Benchmark typing Vietnamese word "việt" (v-i-e-j-t)
/// Tests tone mark application (j = nặng)
fn bench_vietnamese_word(c: &mut Criterion) {
    let mut engine = Engine::new();
    engine.set_method(0); // Telex

    c.bench_function("ime_key_viet", |b| {
        b.iter(|| {
            engine.clear();
            engine.on_key(black_box(keys::V), false, false);
            engine.on_key(black_box(keys::I), false, false);
            engine.on_key(black_box(keys::E), false, false);
            engine.on_key(black_box(keys::J), false, false); // nặng tone
            engine.on_key(black_box(keys::T), false, false);
        })
    });
}

/// Benchmark tone mark application (a + s → á)
/// This is one of the hottest paths
fn bench_tone_mark_processing(c: &mut Criterion) {
    let mut engine = Engine::new();
    engine.set_method(0);

    c.bench_function("tone_mark_apply", |b| {
        b.iter(|| {
            engine.clear();
            engine.on_key(black_box(keys::A), false, false);
            engine.on_key(black_box(keys::S), false, false); // sắc tone
        })
    });
}

/// Benchmark validation pipeline with multi-vowel word
/// "thoan" requires vowel sequence validation
fn bench_validation(c: &mut Criterion) {
    let mut engine = Engine::new();
    engine.set_method(0);

    c.bench_function("validation_pipeline", |b| {
        b.iter(|| {
            engine.clear();
            engine.on_key(black_box(keys::T), false, false);
            engine.on_key(black_box(keys::H), false, false);
            engine.on_key(black_box(keys::O), false, false);
            engine.on_key(black_box(keys::A), false, false);
            engine.on_key(black_box(keys::N), false, false);
        })
    });
}

/// Benchmark circumflex mark (aa → â)
fn bench_circumflex(c: &mut Criterion) {
    let mut engine = Engine::new();
    engine.set_method(0);

    c.bench_function("circumflex_apply", |b| {
        b.iter(|| {
            engine.clear();
            engine.on_key(black_box(keys::A), false, false);
            engine.on_key(black_box(keys::A), false, false); // â
        })
    });
}

/// Benchmark stroke (dd → đ)
fn bench_stroke(c: &mut Criterion) {
    let mut engine = Engine::new();
    engine.set_method(0);

    c.bench_function("stroke_apply", |b| {
        b.iter(|| {
            engine.clear();
            engine.on_key(black_box(keys::D), false, false);
            engine.on_key(black_box(keys::D), false, false); // đ
        })
    });
}

/// Benchmark complex word "đường" (d-d-u-o-w-n-g-f)
/// Tests multiple transforms: stroke + horn + tone
fn bench_complex_word(c: &mut Criterion) {
    let mut engine = Engine::new();
    engine.set_method(0);

    c.bench_function("ime_key_duong", |b| {
        b.iter(|| {
            engine.clear();
            engine.on_key(black_box(keys::D), false, false);
            engine.on_key(black_box(keys::D), false, false); // đ
            engine.on_key(black_box(keys::U), false, false);
            engine.on_key(black_box(keys::O), false, false);
            engine.on_key(black_box(keys::W), false, false); // ươ
            engine.on_key(black_box(keys::N), false, false);
            engine.on_key(black_box(keys::G), false, false);
            engine.on_key(black_box(keys::F), false, false); // huyền tone
        })
    });
}

/// Benchmark buffer clear operation
fn bench_clear(c: &mut Criterion) {
    let mut engine = Engine::new();
    engine.set_method(0);

    // Pre-fill buffer
    engine.on_key(keys::T, false, false);
    engine.on_key(keys::E, false, false);
    engine.on_key(keys::S, false, false);
    engine.on_key(keys::T, false, false);

    c.bench_function("buffer_clear", |b| {
        b.iter(|| {
            engine.clear();
        })
    });
}

criterion_group!(
    benches,
    bench_single_key,
    bench_vietnamese_word,
    bench_tone_mark_processing,
    bench_validation,
    bench_circumflex,
    bench_stroke,
    bench_complex_word,
    bench_clear,
);
criterion_main!(benches);
