# Matrix-Based Validation System

**Status**: REDESIGNED v2 - Pure Dynamic
**Memory**: ~141 bytes input processing (was ~1KB), ~3.5KB total
**Key Change**: Zero case-by-case logic, 87% memory reduction

---

## Overview

All validation, placement, and input processing uses matrix lookup. **Zero if-else in hot path.**

```
Input → Classify (matrix) → Dispatch (matrix) → Execute → Done

OLD: if is_vowel(c) { ... } else if is_final(c) { ... }
NEW: DISPATCH[state][KEY_CAT[key]] → action|next_state
```

---

## Design Principles (v2)

1. **Single lookup per decision** - No chained conditions
2. **Packed data** - Bit flags instead of separate tables
3. **5 states** - Simplified from 6 (merged HAS_MODIFIER/HAS_TONE)
4. **Action + State combined** - One byte encodes both

---

## Matrix Categories

### 1. Vietnamese Validation (M1-M6)

| Matrix | Size | Purpose |
|--------|------|---------|
| M1: INITIAL_VALID | 29×1 | Valid initial consonants |
| M2: INITIAL_VOWEL | 29×12 | Initial + Vowel spelling rules |
| M3: VOWEL_PAIR | 12×12 | Diphthong validity |
| M4: VOWEL_TRIPLE | 8×12 | Triphthong extensions |
| M5: VOWEL_FINAL | 12×9 | Vowel + Final compatibility |
| M6: TONE_FINAL | 6×4 | Rule 7: Tone + Stop Final |

### 2. Placement (M7-M8)

| Matrix | Size | Purpose |
|--------|------|---------|
| M7: TONE_PLACEMENT | 43×4 | Which vowel gets tone (43 patterns) |
| M8: MODIFIER_PLACEMENT | 43×1 | Which vowel(s) get modifier |

### 3. English Validation (E1-E3)

| Matrix | Size | Purpose |
|--------|------|---------|
| E1: ONSET_CC | 26×26 | Valid onset clusters |
| E2: CODA_CC | 26×26 | Valid coda clusters |
| E3: IMPOSSIBLE | 26×26 | Never-occurring bigrams |

### 4. Input Processing (U1-U7) - REDESIGNED

*v2 Unified Design - 87% smaller*

| Matrix | Size | Purpose |
|--------|------|---------|
| U1: LETTER_CLASS | 26 bytes | Vowel/consonant/final/stop bits |
| U2: KEY_CAT | 38 bytes | Key → category mapping |
| U3: DISPATCH | 40 bytes | State × Category → Action|State |
| U4: DEFER | 8 bytes | Pending × is_final → resolution |
| U5: REVERT_KEY | 11 bytes | Transform → revert trigger key |
| U6: TONE_STOP_VALID | 6 bytes | Tone validity with stop finals |
| U7: MOD_VALID | 12 bytes | Modifier validity per vowel |

---

## What Was Eliminated (v2)

| Old Code | Problem | Solution |
|----------|---------|----------|
| `transition_add_letter()` | if-else logic | Encoded in DISPATCH |
| `is_tone_compat()` | if-else logic | TONE_STOP_VALID direct |
| `classify_key()` match | per-key branch | KEY_CAT at init only |
| IS_VOWEL + IS_FINAL + IS_STOP | 3 separate tables | LETTER_CLASS bits |
| I3_REVERT 532 bytes | 14×38 sparse | REVERT_KEY 11 bytes |

---

## Key Rules Encoded

### Rule 7: Stop Finals + Tone Restriction

```
Stop finals (p, t, c, ch) only allow sắc/nặng:

TONE_STOP_VALID = [false, true, false, false, false, true]
                   ngang  sắc   huyền  hỏi    ngã    nặng
```

### 43 Vowel Patterns

| Pattern | Default | After Q | With Final |
|---------|---------|---------|------------|
| ua | V1 (u) | V2 (a) | V2 (a) |
| ươ | V2 (ơ) | V2 | V2 |
| iê | V2 (ê) | V2 | V2 |

---

## Input Processing Flow (v2)

```rust
pub fn process(&mut self, key: u8) -> u8 {
    // Step 1: Resolve pending (matrix)
    if self.pending != 0 {
        let res = DEFER[self.pending][is_final(key)];
        // 0=keep, 1=apply, 2=cancel
    }

    // Step 2: Dispatch (matrix) - ONE LOOKUP gets action + new state
    let packed = DISPATCH[self.state][KEY_CAT[key]];
    let action = packed >> 4;
    let new_state = packed & 0x0F;

    // Step 3: Revert check (matrix)
    if action >= 4 && REVERT_KEY[self.last_transform] == key {
        return self.do_revert();
    }

    // Step 4: Execute action
    // Step 5: Update state
    self.state = new_state;
}
```

---

## Random Order Typing Examples

### "đau" - 5 Ways to Type

| Input | Process | Result |
|-------|---------|--------|
| ddau | d→d, d→đ (stroke), a, u | đau |
| dadu | d→d, a→da, d→đa (delayed), u | đau |
| daud | d→d, a, u→dau, d→đau (delayed) | đau |
| dadud | ...→đau, d→đaud (stroke locked) | đaud |
| daudu | ...→đau (pending), u→revert→dauu | dauu |

### Deferred Breve: "trắm"

```
t → r → a → w (defer breve, open syllable)
              ↓
         m (is_final=true) → DEFER[BREVE][1] = APPLY
              ↓
         "trắm" (breve applied)
```

---

## Memory Summary (v2)

| Category | v1 Size | v2 Size | Reduction |
|----------|---------|---------|-----------|
| Vietnamese validation | ~800B | ~800B | - |
| Placement (43 patterns) | ~220B | ~220B | - |
| Transform tables | ~400B | ~400B | - |
| English validation | ~2KB | ~2KB | - |
| **Input processing** | **~1,050B** | **~141B** | **87%** |
| **Total** | **~4.5KB** | **~3.5KB** | **22%** |

All lookups O(1), cache-friendly, zero if-else.

---

## Files

| Location | Content |
|----------|---------|
| `plans/.../research/comprehensive-matrix-system.md` | Full v1 design |
| `plans/.../research/matrix-data-UNIFIED.md` | **v2 unified design** |
| `plans/.../research/matrix-data-input-processing-v2.md` | **v2 Rust constants** |
| `plans/.../research/MATRIX_REVIEW_REPORT.md` | Review findings |

---

## Related

- [Engine Architecture V2](./engine-architecture-v2.md) - Overall architecture
- [Vietnamese Language System](./vietnamese-language-system.md) - Linguistic rules source
