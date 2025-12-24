# Engine V2 Rewrite: Implementation Plan

**Date:** 2025-12-24
**Branch:** `feature/engine-v2`
**Status:** READY FOR IMPLEMENTATION

---

## References

### Documentation
| Doc | Purpose |
|-----|---------|
| `docs/engine-architecture-v2.md` | V2 architecture design |
| `docs/matrix-validation-system.md` | Matrix system overview |
| `docs/vietnamese-language-system.md` | Linguistic rules source |
| `docs/validation-algorithm.md` | Validation flow |
| `docs/core-engine-algorithm.md` | Current algorithm reference |

### Technical Spec
| File | Purpose |
|------|---------|
| `research/matrix-system.md` | Complete matrix tables (U1-U7, M1-M8, E1-E5) |

---

## Design Summary

### Matrix-First Approach
```
Input → Classify (matrix) → Dispatch (matrix) → Execute → Done

Zero if-else in hot path. Every decision = matrix lookup.
```

### Memory Budget
| Category | Size |
|----------|------|
| Input Processing (U1-U7) | 141 bytes |
| Vietnamese Validation (M1-M8) | ~950 bytes |
| English Validation (E1-E5) | ~384 bytes |
| **Total** | **~1.5KB** |

### State Machine (5 states)
```rust
pub mod st {
    pub const EMPTY: u8 = 0;
    pub const INIT: u8 = 1;   // initial consonant
    pub const VOW: u8 = 2;    // has vowel
    pub const DIA: u8 = 3;    // has diacritic
    pub const FIN: u8 = 4;    // has final
}
```

### Key Tables
- **U1: LETTER_CLASS** (26B) - Bit flags: V|I|F|S
- **U2: KEY_CAT** (38B) - Key → category
- **U3: DISPATCH** (40B) - State × Category → Action|State
- **U4: DEFER** (8B) - Pending resolution
- **U5: REVERT_KEY** (11B) - Transform → revert trigger

---

## Implementation Phases

### Phase 1: Core Matrix Infrastructure
**Goal:** Implement matrix tables and dispatch logic

**Tasks:**
- [ ] Create `core/src/engine/matrix/` module
- [ ] Implement U1-U7 tables from `research/matrix-system.md`
- [ ] Implement dispatch function (single lookup)
- [ ] Unit tests for all matrix lookups

**Key Code:**
```rust
pub fn dispatch(state: u8, key: u8, cat: &[u8; 38]) -> (u8, u8) {
    let c = cat[key as usize];
    let packed = DISPATCH[state as usize][c as usize];
    (packed >> 4, packed & 0x0F)  // action, next_state
}
```

### Phase 2: Processor Implementation
**Goal:** Replace current Engine with matrix-based Processor

**Tasks:**
- [ ] Create `Processor` struct with 5-state machine
- [ ] Implement defer resolution (U4)
- [ ] Implement revert check (U5)
- [ ] Implement tone/modifier validity (U6, U7)
- [ ] Integration tests

**Key Code:**
```rust
pub struct Processor {
    state: u8,
    pending: u8,
    last_transform: u8,
    reverted: bool,
    key_cat: &'static [u8; 38],
}
```

### Phase 3: Vietnamese Validation
**Goal:** Implement M1-M8 validation matrices

**Tasks:**
- [ ] M2: Initial + Vowel compatibility (348B)
- [ ] M5: Vowel + Final compatibility (108B)
- [ ] M6: Tone + Stop Final (Rule 7) (24B)
- [ ] M7: Tone placement (172B)
- [ ] M8: Modifier placement (43B)
- [ ] Validation tests (8 rules)

### Phase 4: English Detection
**Goal:** Implement E1-E5 English matrices

**Tasks:**
- [ ] E2: Onset clusters (676B)
- [ ] E5: Coda clusters (676B)
- [ ] E7: Impossible bigrams (676B)
- [ ] Auto-restore integration
- [ ] English detection tests

### Phase 5: Integration & Migration
**Goal:** Replace old engine, preserve behavior

**Tasks:**
- [ ] Migrate DualBuffer to fixed arrays
- [ ] Remove old if-else logic
- [ ] Run all 561+ existing tests
- [ ] Performance benchmark (<1ms latency)
- [ ] Memory benchmark (0 heap alloc in hot path)

---

## Success Criteria

| Metric | Current | Target |
|--------|---------|--------|
| mod.rs lines | 3,917 | <500 |
| Largest function | 606 LOC | <200 LOC |
| State count | 7+ | 5 |
| If-else in hot path | Many | Zero |
| Matrix memory | N/A | ~3.1KB |
| Test count | 561 | 650+ |
| Keystroke latency | <1ms | <1ms |
| Heap allocations | Yes | Zero |

---

## Archive

Old v1 designs in `research/archived/`.
