# Phase 2: DualBuffer Integration

**Duration:** Week 3-4
**Branch:** `feature/engine-v2-phase2`
**Risk Level:** MEDIUM
**Prerequisite:** Phase 1 complete

---

## Objectives

1. Create DualBuffer abstraction with fixed arrays (zero-alloc)
2. Migrate Engine from separate buf + raw_input to unified DualBuffer
3. Remove all 13+ manual synchronization points
4. Verify zero heap allocations in hot path

---

## Background: Current Problems

### Buffer Synchronization Risk

Current implementation maintains two separate data structures:

```rust
pub struct Engine {
    buf: Buffer,                          // Transformed characters
    raw_input: Vec<(u16, bool, bool)>,    // Raw keystrokes
}
```

**Problems:**
1. Manual sync at 13+ locations in mod.rs
2. Silent desync causes wrong ESC restore output
3. Vec causes heap allocation per keystroke

### Manual Sync Points (from bottleneck analysis)

| Location | Operation | Risk |
|----------|-----------|------|
| Line 534 | raw_input.push() | Buffer grows, may desync |
| Line 617 | raw_input.pop() (mark revert) | Pop count mismatch |
| Line 893 | Stroke revert manipulation | Complex pop sequence |
| Line 1150 | Tone revert | Sync after revert |
| Line 1650 | Mark revert | Sync after revert |
| Line 2716 | Auto-restore | Read both for comparison |
| Line 2850 | ESC restore | Read raw_input |
| +6 more | Various | Scattered sync |

---

## Task 2.1: Create DualBuffer Module

### Design Principles

1. **Fixed arrays** - No heap allocation
2. **Atomic operations** - push/pop affect both buffers
3. **Invariant enforcement** - lengths always match (when no transform)
4. **Transform tracking** - Know which raw keys were consumed

### Implementation

**File:** `core/src/engine/dual_buffer.rs` (NEW)

```rust
//! DualBuffer: Synchronized transformed + raw keystroke buffers
//!
//! Invariants:
//! - push() adds to BOTH buffers atomically
//! - pop() removes from BOTH buffers atomically
//! - apply_transform() only affects transformed buffer
//! - Raw buffer preserves original keystrokes for ESC restore

use crate::engine::buffer::{Buffer, BufferChar};

/// Raw keystroke record
#[derive(Clone, Copy, Debug, Default)]
pub struct RawKeystroke {
    /// Original key code
    pub key: u16,
    /// Shift was pressed
    pub shift: bool,
    /// Key was consumed by a transform (e.g., tone modifier)
    pub consumed: bool,
}

impl RawKeystroke {
    pub const EMPTY: Self = Self {
        key: 0,
        shift: false,
        consumed: false,
    };

    pub fn new(key: u16, shift: bool) -> Self {
        Self {
            key,
            shift,
            consumed: false,
        }
    }
}

/// Maximum buffer sizes (based on longest Vietnamese word)
pub const MAX_TRANSFORMED: usize = 64;
pub const MAX_RAW: usize = 96;  // 64 + space for consumed modifiers

/// DualBuffer: Synchronized transformed + raw buffers
pub struct DualBuffer {
    /// Transformed characters (what user sees)
    transformed: Buffer,

    /// Raw keystrokes (for ESC restore)
    raw: [RawKeystroke; MAX_RAW],
    raw_len: usize,
}

impl DualBuffer {
    /// Create empty DualBuffer
    pub const fn new() -> Self {
        Self {
            transformed: Buffer::new(),
            raw: [RawKeystroke::EMPTY; MAX_RAW],
            raw_len: 0,
        }
    }

    /// Get length of transformed buffer
    #[inline]
    pub fn len(&self) -> usize {
        self.transformed.len()
    }

    /// Check if empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.transformed.is_empty()
    }

    /// Get raw keystroke count
    #[inline]
    pub fn raw_len(&self) -> usize {
        self.raw_len
    }

    /// Push keystroke to both buffers atomically
    ///
    /// Returns false if buffers are full
    pub fn push(&mut self, key: u16, shift: bool) -> bool {
        if self.transformed.len() >= MAX_TRANSFORMED || self.raw_len >= MAX_RAW {
            return false;
        }

        // Add to transformed buffer
        self.transformed.push(BufferChar::new(key));

        // Add to raw buffer
        self.raw[self.raw_len] = RawKeystroke::new(key, shift);
        self.raw_len += 1;

        true
    }

    /// Push only to raw buffer (for consumed modifiers)
    ///
    /// Use when a key is consumed by transform (e.g., tone/mark key)
    pub fn push_raw_consumed(&mut self, key: u16, shift: bool) {
        if self.raw_len < MAX_RAW {
            self.raw[self.raw_len] = RawKeystroke {
                key,
                shift,
                consumed: true,
            };
            self.raw_len += 1;
        }
    }

    /// Pop from both buffers atomically
    ///
    /// Handles consumed modifiers: pops raw until non-consumed found
    pub fn pop(&mut self) -> Option<BufferChar> {
        let char = self.transformed.pop()?;

        // Pop raw keystrokes (including any consumed modifiers after this char)
        while self.raw_len > 0 {
            self.raw_len -= 1;
            if !self.raw[self.raw_len].consumed {
                break;
            }
        }

        Some(char)
    }

    /// Pop only from raw buffer (for mark/tone revert scenarios)
    pub fn pop_raw(&mut self) -> Option<RawKeystroke> {
        if self.raw_len > 0 {
            self.raw_len -= 1;
            Some(self.raw[self.raw_len])
        } else {
            None
        }
    }

    /// Access transformed buffer for reading
    #[inline]
    pub fn transformed(&self) -> &Buffer {
        &self.transformed
    }

    /// Access transformed buffer for modification
    #[inline]
    pub fn transformed_mut(&mut self) -> &mut Buffer {
        &mut self.transformed
    }

    /// Get raw keystrokes slice
    #[inline]
    pub fn raw_slice(&self) -> &[RawKeystroke] {
        &self.raw[..self.raw_len]
    }

    /// Apply transform to character at position
    ///
    /// Only affects transformed buffer; raw buffer unchanged
    pub fn apply_transform(&mut self, pos: usize, transform: Transform) {
        if pos < self.transformed.len() {
            self.transformed.get_mut(pos).unwrap().apply(transform);
        }
    }

    /// Get raw input as string (for English validation)
    pub fn raw_to_string(&self) -> String {
        self.raw[..self.raw_len]
            .iter()
            .filter(|r| !r.consumed)
            .filter_map(|r| char::from_u32(r.key as u32))
            .collect()
    }

    /// Get raw input as chars (for ESC restore)
    pub fn raw_chars(&self) -> Vec<char> {
        self.raw[..self.raw_len]
            .iter()
            .filter_map(|r| char::from_u32(r.key as u32))
            .collect()
    }

    /// Clear both buffers
    pub fn clear(&mut self) {
        self.transformed.clear();
        self.raw_len = 0;
    }

    /// Debug: Check if buffers are in sync
    #[cfg(debug_assertions)]
    pub fn assert_sync(&self) {
        let non_consumed = self.raw[..self.raw_len]
            .iter()
            .filter(|r| !r.consumed)
            .count();
        debug_assert_eq!(
            self.transformed.len(),
            non_consumed,
            "DualBuffer desync: transformed={}, non-consumed raw={}",
            self.transformed.len(),
            non_consumed
        );
    }
}

impl Default for DualBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_pop_sync() {
        let mut buf = DualBuffer::new();

        buf.push(b'a' as u16, false);
        buf.push(b'b' as u16, false);
        buf.push(b'c' as u16, false);

        assert_eq!(buf.len(), 3);
        assert_eq!(buf.raw_len(), 3);

        buf.pop();
        assert_eq!(buf.len(), 2);
        assert_eq!(buf.raw_len(), 2);

        buf.assert_sync();
    }

    #[test]
    fn test_consumed_modifier() {
        let mut buf = DualBuffer::new();

        // Type "t", "e", "s" (tone modifier consumed)
        buf.push(b't' as u16, false);
        buf.push(b'e' as u16, false);
        buf.push_raw_consumed(b's' as u16, false); // Consumed by tone

        assert_eq!(buf.len(), 2); // t, e (tone applied)
        assert_eq!(buf.raw_len(), 3); // t, e, s

        // Raw string excludes consumed
        assert_eq!(buf.raw_to_string(), "te");

        // Raw chars includes consumed (for ESC restore)
        assert_eq!(buf.raw_chars(), vec!['t', 'e', 's']);
    }

    #[test]
    fn test_clear() {
        let mut buf = DualBuffer::new();
        buf.push(b'a' as u16, false);
        buf.push(b'b' as u16, false);

        buf.clear();

        assert!(buf.is_empty());
        assert_eq!(buf.raw_len(), 0);
    }

    #[test]
    fn test_transform_doesnt_affect_raw() {
        let mut buf = DualBuffer::new();
        buf.push(b'a' as u16, false);

        // Apply transform to position 0
        // (Actual Transform implementation depends on Buffer)
        // buf.apply_transform(0, Transform::CircumflexAdd);

        // Raw should still have original 'a'
        assert_eq!(buf.raw[0].key, b'a' as u16);
    }
}
```

### Transform Enum (if not exists)

```rust
/// Character transformation types
#[derive(Clone, Copy, Debug)]
pub enum Transform {
    // Tones
    ToneAdd(Tone),
    ToneRemove,

    // Vowel marks
    CircumflexAdd,
    CircumflexRemove,
    BreveAdd,
    BreveRemove,
    HornAdd,
    HornRemove,

    // Stroke
    StrokeAdd,
    StrokeRemove,
}
```

### Acceptance Criteria

- [ ] DualBuffer struct implemented
- [ ] Fixed arrays (no Vec)
- [ ] push/pop atomically sync both buffers
- [ ] push_raw_consumed for modifier keys
- [ ] raw_to_string() and raw_chars() helpers
- [ ] 10+ unit tests passing
- [ ] assert_sync() debug helper

---

## Task 2.2: Migrate Engine to DualBuffer

### Current Structure

```rust
pub struct Engine {
    buf: Buffer,                          // ~500 bytes
    raw_input: Vec<(u16, bool, bool)>,    // Heap allocated
    // ... 23 other fields
}
```

### New Structure

```rust
pub struct Engine {
    buffer: DualBuffer,                   // Unified, stack-allocated
    // ... reduced fields
}
```

### Migration Steps

#### Step 1: Add DualBuffer, keep old fields

```rust
pub struct Engine {
    // NEW
    buffer: DualBuffer,

    // OLD (temporary, for gradual migration)
    #[deprecated]
    buf: Buffer,
    #[deprecated]
    raw_input: Vec<(u16, bool, bool)>,
}
```

#### Step 2: Update push operations

**Before:**
```rust
self.buf.push(BufferChar::new(key));
self.raw_input.push((key, shift, ctrl));
```

**After:**
```rust
self.buffer.push(key, shift);
```

#### Step 3: Update pop operations

**Before:**
```rust
self.buf.pop();
if let Some(last) = self.raw_input.last() {
    // Check if consumed
    self.raw_input.pop();
}
```

**After:**
```rust
self.buffer.pop();
```

#### Step 4: Update modifier handling

**Before:**
```rust
// Tone key consumed, only add to raw_input
self.raw_input.push((key, shift, false));
// Apply tone to buf
self.buf.apply_tone(pos, tone);
```

**After:**
```rust
// Mark key as consumed in raw
self.buffer.push_raw_consumed(key, shift);
// Apply tone to transformed
self.buffer.apply_transform(pos, Transform::ToneAdd(tone));
```

#### Step 5: Remove deprecated fields

After all usages migrated:
```rust
pub struct Engine {
    buffer: DualBuffer,
    // ... other non-buffer fields
}
```

### Acceptance Criteria

- [ ] DualBuffer integrated into Engine
- [ ] All push operations use buffer.push()
- [ ] All pop operations use buffer.pop()
- [ ] Modifier keys use push_raw_consumed()
- [ ] Old buf and raw_input fields removed
- [ ] All 561 tests passing

---

## Task 2.3: Remove Manual Sync Points

### Sync Point Inventory

| Line | Current Code | Replacement |
|------|-------------|-------------|
| 534 | `raw_input.push(...)` | `buffer.push(key, shift)` |
| 541 | `raw_input.pop()` (backspace) | `buffer.pop()` |
| 617 | Pop sequence (mark revert) | `buffer.pop_raw()` |
| 893 | Stroke revert manipulation | `buffer.pop()` + `buffer.push()` |
| 1150 | Tone revert sync | Handled by Transform |
| 1650 | Mark revert sync | `buffer.pop_raw()` |
| 2716 | Auto-restore read | `buffer.raw_to_string()` |
| 2850 | ESC restore read | `buffer.raw_chars()` |

### Specific Fixes

#### Mark Revert (line 617)

**Before:**
```rust
if self.pending_mark_revert_pop && keys::is_letter(key) {
    self.pending_mark_revert_pop = false;
    if keys::is_consonant(key) {
        let current = self.raw_input.pop();
        let revert = self.raw_input.pop();
        self.raw_input.pop(); // mark key
        // ...
    }
}
```

**After:**
```rust
if self.pending_mark_revert_pop && keys::is_letter(key) {
    self.pending_mark_revert_pop = false;
    if keys::is_consonant(key) {
        // Pop consumed mark key from raw
        self.buffer.pop_raw();
        // ...
    }
}
```

#### Auto-restore (line 2716)

**Before:**
```rust
let raw_string: String = self.raw_input
    .iter()
    .map(|(k, _, _)| char::from_u32(*k as u32).unwrap())
    .collect();
```

**After:**
```rust
let raw_string = self.buffer.raw_to_string();
```

### Verification

Add debug assertions:

```rust
#[cfg(debug_assertions)]
fn verify_buffer_sync(&self) {
    self.buffer.assert_sync();
}

// Call after every operation in debug mode
pub fn on_key_ext(&mut self, key: u16, shift: bool, ctrl: bool) -> ImeResult {
    let result = self.on_key_ext_impl(key, shift, ctrl);

    #[cfg(debug_assertions)]
    self.verify_buffer_sync();

    result
}
```

### Acceptance Criteria

- [ ] All 13 sync points replaced with DualBuffer methods
- [ ] No direct access to old buf or raw_input
- [ ] Debug assertions verify sync after each operation
- [ ] All 561 tests passing in debug mode

---

## Task 2.4: Verify Zero Heap Allocations

### Benchmark Setup

**File:** `core/benches/allocation_bench.rs` (NEW)

```rust
use criterion::{criterion_group, criterion_main, Criterion};
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

struct CountingAllocator;

static ALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOC_COUNT.fetch_add(1, Ordering::SeqCst);
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout)
    }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

fn reset_alloc_count() {
    ALLOC_COUNT.store(0, Ordering::SeqCst);
}

fn get_alloc_count() -> usize {
    ALLOC_COUNT.load(Ordering::SeqCst)
}

fn bench_keystroke_allocations(c: &mut Criterion) {
    c.bench_function("keystroke_hot_path", |b| {
        b.iter(|| {
            let mut engine = Engine::new();

            reset_alloc_count();

            // Type a typical Vietnamese word: "việt"
            engine.on_key('v' as u16, false, false);
            engine.on_key('i' as u16, false, false);
            engine.on_key('e' as u16, false, false);
            engine.on_key('e' as u16, false, false); // circumflex
            engine.on_key('j' as u16, false, false); // nặng

            let allocs = get_alloc_count();

            // Assert zero allocations in hot path
            assert_eq!(allocs, 0, "Hot path allocated {} times", allocs);
        });
    });
}

criterion_group!(benches, bench_keystroke_allocations);
criterion_main!(benches);
```

### Memory Layout Verification

```rust
#[test]
fn test_dual_buffer_stack_size() {
    use std::mem::size_of;

    // DualBuffer should be stack-allocated
    let size = size_of::<DualBuffer>();

    // Expected: Buffer (~500) + raw array (96 * 5) + len (8)
    // Approximately 1KB
    assert!(size < 2048, "DualBuffer too large: {} bytes", size);

    // Verify no heap pointers
    // (This is compile-time guaranteed by using fixed arrays)
}
```

### Acceptance Criteria

- [ ] Allocation benchmark created
- [ ] Zero allocations in keystroke hot path
- [ ] DualBuffer size < 2KB
- [ ] Memory profile documented

---

## Phase 2 Completion Checklist

- [ ] **Task 2.1:** DualBuffer module with 10+ tests
- [ ] **Task 2.2:** Engine migrated to DualBuffer
- [ ] **Task 2.3:** All 13 sync points removed
- [ ] **Task 2.4:** Zero-alloc verified with benchmarks
- [ ] All 561 existing tests passing
- [ ] Code reviewed and merged
- [ ] Performance benchmark baseline established

**Estimated Effort:** 2 weeks
**Risk Level:** MEDIUM (refactoring core data structure)

---

## Rollback Plan

If issues discovered:
1. Keep old buf/raw_input as fallback
2. Feature flag: `use_dual_buffer: bool`
3. Gradual rollout with A/B testing

```rust
impl Engine {
    fn push_key(&mut self, key: u16, shift: bool) {
        if cfg!(feature = "dual_buffer") {
            self.buffer.push(key, shift);
        } else {
            self.buf.push(BufferChar::new(key));
            self.raw_input.push((key, shift, false));
        }
    }
}
```

---

## Next Phase

After Phase 2 completion, proceed to **Phase 3: State Machine & Modularization**.

See `phase-03-state-machine.md` for details.
