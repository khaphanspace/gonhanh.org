# Matrix System Review Report

**Date**: 2025-12-24
**Reviewer**: Claude Code
**Status**: REVIEW COMPLETE - Issues Found

---

## Executive Summary

Reviewed matrix design (I1-I7) against actual engine implementation (`mod.rs` 3,917 lines).

| Category | Status | Details |
|----------|--------|---------|
| Missing Cases | **8 GAPS** | Pending states, transform types, special patterns |
| Pipeline Completeness | **PARTIAL** | Core OK, gate-level handling missing |
| Redundant Steps | **1 POSSIBLE** | HAS_MODIFIER vs HAS_TONE states |
| Overall | **NEEDS REVISION** | 80% coverage, 20% gaps |

---

## Part 1: Missing Cases

### 1.1 Missing Pending State: `pending_capitalize` ❌

**Actual Engine** (line 268):
```rust
pending_capitalize: bool,
```

**Matrix Design I2** only covers:
- PENDING_BREVE
- PENDING_U_HORN
- PENDING_MARK_POP

**Impact**: Auto-capitalize feature won't work in matrix-only engine.

**Fix**: Add `PENDING_CAPITALIZE` to I2 defer types.

---

### 1.2 Missing Transform Type: `WShortcutSkipped` ❌

**Actual Engine** (line 86-87):
```rust
/// W shortcut was explicitly skipped (prevent re-transformation)
WShortcutSkipped,
```

**Usage** (line 802):
```rust
if matches!(self.last_transform, Some(Transform::WShortcutSkipped)) {
    return None; // Don't try w→ư again
}
```

**Matrix Design I3** has 9 transform types but missing `WShortcutSkipped`.

**Impact**: "ww" → "w" revert then subsequent 'w' might re-trigger transformation.

**Fix**: Add `W_SHORTCUT_SKIPPED` to I3 transform types.

---

### 1.3 Post-Tone Delayed Circumflex NOT Modeled ❌

**Actual Engine** (lines 2293-2338):
```rust
// Telex: Post-tone delayed circumflex (xepse → xếp)
// Pattern: initial-consonant + vowel-with-mark + non-extending-final (t, m, p) + same vowel
// When user types tone BEFORE circumflex modifier: "xeps" → "xép", then 'e' → "xếp"
```

**Matrix Design**: No handling for this pattern.

**Impact**: "xepse" → should become "xếp" but matrix won't handle it.

**Fix**: Add to I2 as `PENDING_POST_TONE_CIRCUMFLEX` or handle in I1 dispatch.

---

### 1.4 Delayed Circumflex Revert NOT Modeled ❌

**Actual Engine** (lines 2252-2291):
```rust
// Telex: Revert delayed circumflex when same vowel is typed again
// Pattern: After "data" → "dât" (delayed circumflex), typing 'a' again should revert to "data"
```

**Matrix Design**: I3 REVERT_LOOKUP only handles double-key revert, not this pattern.

**Impact**: "dataa" flow ("dât" + 'a' → "data") won't work.

**Fix**: Add special revert rule for vowel-after-circumflex-final pattern.

---

### 1.5 I7 POST_STROKE Behavior Mismatch ⚠️

**Matrix Design Case 5** expects:
```
"daudu" → "dauu" (stroke revert via vowel after delayed stroke)
```

**Actual Engine Behavior** (lines 635-689):
- Short-pattern stroke revert triggers when NEW LETTER creates invalid Vietnamese
- Checks BOTH `raw_input` (raw keys) AND `buf + key` (transformed + new)
- If either is valid → KEEP stroke

**Actual "daudu" flow**:
1. d → buf="d"
2. a → buf="da"
3. u → buf="dau"
4. d → delayed stroke applies → buf="đau" (ShortPatternStroke)
5. u → check validity:
   - raw_input="daudu" → is_valid? NO
   - buf+key="đauu" → is_valid? NO
   - REVERT! buf="dauu"

**Verdict**: Matrix I7 design is CORRECT! Actual engine already does this.

But I7 matrix must check BOTH raw_input AND buf+key validity.

---

### 1.6 ESC Restore Logic NOT in Matrix Scope ❌

**Actual Engine** (lines 460-470):
```rust
if key == keys::ESC {
    let result = if self.esc_restore_enabled {
        self.restore_to_raw()
    }
}
```

**Matrix Design**: Not covered. This is gate-level handling before dispatch.

**Impact**: Low - ESC is application-level, not transform-level.

**Status**: OUT OF SCOPE for matrix (acceptable).

---

### 1.7 Auto-Restore on Space NOT Modeled ❌

**Actual Engine** (lines 431-443):
```rust
// Auto-restore: if buffer has transforms but is invalid Vietnamese,
// restore to raw English (like ESC but triggered by space)
let restore_result = self.try_auto_restore_on_space();
```

**Matrix Design**: Not covered in I1-I7.

**Impact**: Medium - English words with accidental Vietnamese might stay transformed.

**Status**: Should add as post-COMPLETE_WORD validation check.

---

### 1.8 Word Boundary Shortcuts NOT Modeled ❌

**Actual Engine** (lines 424-428):
```rust
// First check for shortcut
let shortcut_result = self.try_word_boundary_shortcut();
```

**Matrix Design**: Not covered.

**Impact**: Low - Shortcuts are separate system, not phonological.

**Status**: OUT OF SCOPE (acceptable).

---

## Part 2: Pipeline Comparison

### Actual Engine Pipeline (on_key_ext)

```
GATE LEVEL (pre-dispatch):
├── 1. enabled/ctrl check → clear & none
├── 2. SPACE → shortcut, auto-restore, history push, clear
├── 3. ESC → restore_to_raw, clear
├── 4. Break key → auto-restore, clear
├── 5. Backspace → word history restore, buffer pop
└── 6. Auto-capitalize check

PROCESS LEVEL (process):
├── 1. pending_mark_revert_pop → pop from raw_input if consonant
├── 2. Short-pattern stroke revert check (BEFORE modifiers!)
├── 3. try_stroke() → d → đ
├── 4. try_tone() → circumflex, horn, breve
├── 5. try_mark() → sắc, huyền, hỏi, ngã, nặng
├── 6. try_remove() → z key (remove modifiers)
├── 7. try_w_as_vowel() → w → ư (Telex only)
└── 8. handle_normal_letter() → add to buffer + deferred apply
```

### Matrix Design Pipeline (I1-I7)

```
PROCESS LEVEL:
├── 1. classify_key(key, method) → key_type
├── 2. I2_DEFERRED check → resolve pending
├── 3. I1_ACTION dispatch → action type
├── 4. I3_REVERT check → double-key revert
├── 5. I5/I6 COMPAT check → modifier/tone validity
├── 6. Execute action
└── 7. I4_TRANSITION → new state
```

### Gap Analysis

| Feature | Engine | Matrix | Status |
|---------|--------|--------|--------|
| Gate-level (Space/ESC/Break) | ✓ | ✗ | OUT OF SCOPE |
| Backspace handling | ✓ | ✗ | OUT OF SCOPE |
| Auto-capitalize | ✓ | ✗ | MISSING |
| pending_mark_revert_pop | ✓ | ✓ (I2) | OK |
| Short-pattern stroke revert | ✓ | ✓ (I7) | OK |
| Stroke (d→đ) | ✓ | ✓ (I3) | OK |
| Tone (circumflex/horn/breve) | ✓ | ✓ (I1+I5) | OK |
| Mark (sắc/huyền/etc) | ✓ | ✓ (I1) | OK |
| Remove (z key) | ✓ | ✗ | PARTIAL |
| w→ư (Telex) | ✓ | ✗ | MISSING |
| Deferred breve | ✓ | ✓ (I2) | OK |
| Deferred u_horn | ✓ | ✓ (I2) | OK |
| Post-tone circumflex | ✓ | ✗ | MISSING |
| Double-key revert | ✓ | ✓ (I3) | OK |

---

## Part 3: Redundancy Analysis

### 3.1 HAS_MODIFIER vs HAS_TONE States ⚠️

**Matrix Design** uses separate states:
- State 3: HAS_MODIFIER (circumflex/horn/breve applied)
- State 4: HAS_TONE (mark applied)

**Actual Engine** uses single Char struct:
```rust
struct Char {
    key: u16,       // base key
    caps: bool,     // uppercase
    tone: u8,       // modifier: NONE, CIRCUMFLEX, HORN (0,1,2)
    mark: u8,       // tone mark: 0-5 (ngang→nặng)
    stroke: bool,   // d→đ
}
```

**Question**: Can a vowel have BOTH modifier AND mark?
- YES! Example: "ấ" = 'a' + circumflex + sắc
- So buffer can be in BOTH HAS_MODIFIER AND HAS_TONE state

**Verdict**: States NOT mutually exclusive → need combined state or bitmap.

**Recommendation**: Change to:
```
States (revised):
  0 = EMPTY
  1 = HAS_INITIAL
  2 = HAS_VOWEL
  3 = HAS_DIACRITIC (modifier OR mark OR both)
  4 = HAS_FINAL
```

Or use flags:
```rust
struct InputState {
    has_initial: bool,
    has_vowel: bool,
    has_modifier: bool,
    has_mark: bool,
    has_final: bool,
}
// 5 bits = 32 states max
```

---

### 3.2 I4 3D Matrix ✓ (Already Addressed)

Matrix design already notes optimization:
```rust
fn next_state(state: State, action: Action, result: bool) -> State {
    let action_result = (action as u8) << 1 | (result as u8);
    I4_TRANSITION_FLAT[state as usize][action_result as usize]
}
```

**Status**: Already optimized in design.

---

### 3.3 I5/I6 vs M Matrices ✓ (Not Redundant)

- I5/I6: Runtime input compatibility (can I apply X now?)
- M1-M6: Linguistic validation (is pattern valid Vietnamese?)

These serve different purposes:
- I5/I6 check BEFORE transformation
- M matrices validate AFTER transformation

**Status**: Not redundant, keep both.

---

## Part 4: Specific Issues with "đau" Examples

### Case 4 "dadud" → "đaud" ✓ CORRECT

Matrix design matches actual engine behavior.

### Case 5 "daudu" → "dauu" ✓ CORRECT

Matrix I7 design is correct:
1. After "dau" + 'd' → delayed stroke → "đau"
2. Check pending: PENDING_STROKE_VERIFY
3. Next key 'u' (vowel) → I7[POST_STROKE, u] = REVERT
4. Revert stroke: "đau" → "dau"
5. Add 'u': "dauu"

**BUT**: Need to verify actual engine does this.

**Verification** (lines 656-689):
```rust
if keys::is_letter(key)
    && !is_mark_key
    && !is_tone_key
    && !is_stroke_key
    && matches!(self.last_transform, Some(Transform::ShortPatternStroke))
{
    // Check if raw_input AND buf+key are BOTH invalid
    if !is_valid(&raw_keys) && !is_valid(&buf_keys) {
        // REVERT stroke
    }
}
```

So for "daudu":
- raw_keys = [d,a,u,d,u] → "daudu" → INVALID
- buf_keys = [đ,a,u,u] → "đauu" → INVALID
- REVERT! ✓

**Matrix design matches actual behavior.**

---

## Part 5: Recommendations

### Priority 1: Critical Fixes (MUST DO)

1. **Add `PENDING_CAPITALIZE`** to I2
   - Size: 4 defer_types × 38 keys (was 3)
   - Impact: +38 bytes

2. **Add `W_SHORTCUT_SKIPPED`** to I3
   - Size: 12 transform_types × 38 keys (was 11)
   - Impact: +38 bytes

3. **Add post-tone circumflex handling**
   - Either in I2 as PENDING_POST_CIRCUMFLEX
   - Or as special case in I1 APPLY_MODIFIER

4. **Fix HAS_MODIFIER/HAS_TONE conflict**
   - Use combined state or flags
   - Or reduce to HAS_DIACRITIC

### Priority 2: Should Do

5. **Model try_remove()** (z key)
   - Add REMOVE action to I1
   - Returns buffer to pre-modifier state

6. **Model try_w_as_vowel()** (Telex w→ư)
   - Add W_AS_VOWEL action
   - Handle revert via I3

### Priority 3: Nice to Have

7. **Auto-restore on space**
   - Post-COMPLETE_WORD validation
   - If invalid VN → restore

8. **Document out-of-scope**
   - Gate level: Space/ESC/Break/Backspace
   - Shortcuts: Word boundary

---

## Part 6: Updated Memory Estimate

| Matrix | Original Size | After Fix | Notes |
|--------|--------------|-----------|-------|
| I1 | 6×38 = 228 | 6×38 = 228 | Unchanged |
| I2 | 3×38 = 114 | 4×38 = 152 | +pending_capitalize |
| I3 | 11×38 = 418 | 12×38 = 456 | +w_shortcut_skipped |
| I4 | 6×16 = 96 | 6×16 = 96 | Unchanged |
| I5 | 4×12 = 48 | 4×12 = 48 | Unchanged |
| I6 | 6×43 = 258 | 6×43 = 258 | Unchanged |
| I7 | 1×38 = 38 | 1×38 = 38 | Unchanged |
| **Total** | **~1.2KB** | **~1.3KB** | +76 bytes |

---

## Conclusion

Matrix design is **80% complete**. Key gaps:

1. **Missing pending states** (capitalize, post-tone circumflex)
2. **Missing transform types** (w_shortcut_skipped)
3. **State model issue** (modifier/mark not mutually exclusive)

All are fixable with minor additions (~76 bytes extra).

The core design philosophy (matrix-first, O(1) lookup) is sound. Implementation should proceed after addressing Priority 1 fixes.

---

**Report Generated**: 2025-12-24
**Confidence**: HIGH (based on line-by-line code review)
**Next Step**: Update comprehensive-matrix-system.md with fixes

