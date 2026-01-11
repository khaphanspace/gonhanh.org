//! Memory stability test for Gõ Nhanh IME engine
//!
//! Verifies no memory growth during extended typing sessions.
//! Uses custom GlobalAlloc to track allocations.
//!
//! Run: cargo test --test memory_stability -- --nocapture

use gonhanh_core::data::keys;
use gonhanh_core::engine::Engine;
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Tracking allocator that wraps System allocator
struct TrackingAllocator;

static ALLOCATED: AtomicUsize = AtomicUsize::new(0);
static DEALLOCATED: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATED.fetch_add(layout.size(), Ordering::SeqCst);
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        DEALLOCATED.fetch_add(layout.size(), Ordering::SeqCst);
        System.dealloc(ptr, layout)
    }
}

#[global_allocator]
static GLOBAL: TrackingAllocator = TrackingAllocator;

fn reset_counters() {
    ALLOCATED.store(0, Ordering::SeqCst);
    DEALLOCATED.store(0, Ordering::SeqCst);
}

fn get_net_allocation() -> i64 {
    let alloc = ALLOCATED.load(Ordering::SeqCst) as i64;
    let dealloc = DEALLOCATED.load(Ordering::SeqCst) as i64;
    alloc - dealloc
}

/// Simulates typing a Vietnamese word and clearing buffer
fn type_and_clear(engine: &mut Engine) {
    // Type "đường" (d-d-u-o-w-n-g-f)
    engine.on_key(keys::D, false, false);
    engine.on_key(keys::D, false, false);
    engine.on_key(keys::U, false, false);
    engine.on_key(keys::O, false, false);
    engine.on_key(keys::W, false, false);
    engine.on_key(keys::N, false, false);
    engine.on_key(keys::G, false, false);
    engine.on_key(keys::F, false, false);
    engine.clear();
}

#[test]
fn test_no_memory_growth() {
    let mut engine = Engine::new();
    engine.set_method(0); // Telex

    // Warm up - allocate initial buffers
    for _ in 0..100 {
        type_and_clear(&mut engine);
    }

    // Reset counters after warmup
    reset_counters();

    // Simulate 10-minute typing session (~600 words at 60 WPM)
    let iterations = 600;
    for _ in 0..iterations {
        type_and_clear(&mut engine);
    }

    let net_alloc = get_net_allocation();

    // Allow variance for test harness overhead + engine buffers
    // Key check: memory should NOT grow linearly with iterations
    // 600 iterations × 8 keys = 4800 keystrokes
    // If leak: would see ~4800 × leak_per_key bytes
    let threshold = 64 * 1024i64; // 64KB - generous for test overhead

    println!("=== Memory Stability Test ===");
    println!("Iterations: {}", iterations);
    println!("Net allocation: {} bytes", net_alloc);
    println!("Threshold: {} bytes", threshold);
    println!(
        "Status: {}",
        if net_alloc.abs() < threshold {
            "PASS ✓"
        } else {
            "FAIL ✗"
        }
    );

    assert!(
        net_alloc.abs() < threshold,
        "Memory leak detected! Net allocation: {} bytes (threshold: {} bytes)",
        net_alloc,
        threshold
    );
}

#[test]
fn test_allocation_per_keystroke() {
    let mut engine = Engine::new();
    engine.set_method(0);

    // Warm up
    for _ in 0..50 {
        engine.on_key(keys::A, false, false);
    }
    engine.clear();

    // Measure allocation for 100 keystrokes
    reset_counters();
    for _ in 0..100 {
        engine.on_key(keys::A, false, false);
    }
    engine.clear();

    let total_alloc = ALLOCATED.load(Ordering::SeqCst);
    let avg_per_key = total_alloc / 100;

    println!("=== Allocation Per Keystroke ===");
    println!("Total allocation (100 keys): {} bytes", total_alloc);
    println!("Average per keystroke: {} bytes", avg_per_key);

    // Target: bounded allocation in hot path
    // Engine uses String internally, some allocation expected
    // Key check: allocation should be bounded, not growing unbounded
    let threshold = 4096; // 4KB per keystroke (includes test overhead)
    assert!(
        avg_per_key < threshold,
        "High allocation per keystroke: {} bytes (threshold: {} bytes)",
        avg_per_key,
        threshold
    );
}
