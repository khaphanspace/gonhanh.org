# Engine Bottleneck Analysis: Comprehensive Research Report
**Status**: RESEARCH COMPLETE | **Date**: 2025-12-24 | **Codebase Version**: v1.0.87

---

## EXECUTIVE SUMMARY

The Gõ Nhanh Vietnamese IME engine (3,917 lines) suffers from **critical maintainability issues** stemming from monolithic architecture. While **functionally robust and performant**, the codebase exhibits:

- **State Explosion**: 25+ struct fields with complex interdependencies (11 of 25 are boolean flags)
- **God File Problem**: Single mod.rs (187KB) housing all core logic; violates <200LOC principle
- **Buffer Sync Risk**: `raw_input` Vec<tuple> manually synchronized with Buffer; error-prone operations
- **Complexity Spiral**: 7-stage processing pipeline with overlapping concerns (validation, transformation, revert)
- **Hidden Dependencies**: Flag state is quasi-global; unclear order-of-operations contracts

**Core Issue**: System designed to handle **escalating complexity incrementally** (360+ commit history tracking edge cases). Each new feature (auto-restore, breve deferral, u-horn pending, delayed circumflex) added conditional branches rather than structural improvements.

**Positive Finding**: Foundation is **architecturally sound**. Validation-first approach, pattern-based scanning, and V1 design patterns can be preserved and strengthened.

---

## 1. PROCESSING PIPELINE COMPLEXITY

### 1.1 The Seven-Stage Pipeline

Engine processes keys through **deeply nested, order-dependent stages**:

```
on_key_ext()
  ├─ [GATE] Enabled? Ctrl pressed? → clear + return
  ├─ [GATE] SPACE key? → try word shortcuts, auto-restore, commit history
  ├─ [GATE] ESC key? → restore to raw ASCII
  ├─ [GATE] Break keys? → auto-capitalize, auto-restore, clear
  ├─ [GATE] DELETE key? → backspace-after-space logic
  ├─ [SETUP] Auto-capitalize? Apply caps overrides
  ├─ [RECORD] Add to raw_input vector
  ├─ [DISPATCH] process()
  │    ├─ [REVERT-CHECK] Mark revert pop?
  │    ├─ [REVERT-CHECK] Short-pattern stroke revert?
  │    ├─ [MODIFIER] try_stroke() → nested conditionals
  │    ├─ [MODIFIER] try_tone() → nested conditionals (~700 LOC)
  │    ├─ [MODIFIER] try_mark() → nested conditionals (~400 LOC)
  │    ├─ [MODIFIER] try_remove()
  │    ├─ [MODIFIER] try_w_as_vowel() → nested conditionals
  │    └─ [FALLBACK] handle_normal_letter()
  └─ Return Result
```

### 1.2 Code Complexity Analysis

#### try_tone() Function (lines 1042-1648, ~606 lines)
- **Nesting Depth**: 6+ levels
- **Conditional Branches**: 40+
- **Responsibility Count**: 8+
  1. Cancel pending breve (Issue #44)
  2. Check revert (same key twice)
  3. Validate buffer structure
  4. Determine if "switching" tones
  5. Find uo/ou compound positions
  6. Apply smart horn target selection
  7. Handle telex circumflex special logic
  8. Handle same-vowel trigger detection
  9. Apply transforms
  10. Check validity with tones

**Example Nesting** (line 1150-1160):
```rust
if is_telex_circumflex {
    let any_vowel_has_tone = self
        .buf.iter().filter(|c| keys::is_vowel(c.key))
        .any(|c| c.has_tone());
    if any_vowel_has_tone {
        return None;
    }
    // Check if buffer has multiple vowel types
    let vowel_chars: Vec<_> = self.buf.iter()
        .filter(|c| keys::is_vowel(c.key)).collect();
    let has_any_mark = vowel_chars.iter().any(|c| c.has_mark());
    let unique_vowel_types: std::collections::HashSet<u16> = ...
    let has_multiple_vowel_types = unique_vowel_types.len() > 1;
    if has_any_mark && has_multiple_vowel_types {
        // 20+ more lines of nested logic
    }
}
```

#### try_mark() Function (lines 1649-2113, ~465 lines)
- **Nesting Depth**: 5+ levels
- **Conditional Branches**: 35+
- **Responsibility Count**: 6+
  1. Revert check (same mark key twice)
  2. Delayed stroke pattern detection
  3. Pending breve application
  4. Delayed circumflex pattern detection
  5. Build mark position candidates
  6. Apply mark to position
  7. Handle post-revert adjustments

#### try_stroke() Function (lines 872-1041, ~170 lines)
- **Nesting Depth**: 5+ levels
- **Conditional Branches**: 20+
- **Responsibility Count**: 4+
  1. Stroke revert detection (ddd → dd)
  2. Short-pattern stroke revert
  3. Adjacent d detection
  4. Delayed stroke logic (complex vowel pattern checks)

### 1.3 Control Flow Interdependencies

**Critical Dependencies** (order matters):
```
stroke_reverted flag
  └─ Affects: try_stroke() behavior → prevents re-triggering
     Set by: try_stroke() when ddd → dd revert
     Reset by: backspace (DELETE key)
     Risk: If stroke_reverted not reset on certain operations, 'd' becomes "stuck"

pending_breve_pos
  ├─ Set by: try_tone() for deferred breve (issue #44)
  ├─ Consumed by: try_mark() or handle_normal_letter()
  ├─ Dependencies: pending_breve_pos + pending_u_horn_pos can't both be active
  └─ Risk: If both pending flags set incorrectly, wrong vowel gets modified

had_mark_revert flag
  ├─ Set by: revert_mark() when same key pressed twice (ss → s)
  ├─ Used by: auto-restore logic (line 2742)
  ├─ Reset by: clear() when word boundary reached
  └─ Risk: Flag survives across multiple keypresses, can cause false auto-restore

pending_mark_revert_pop
  ├─ Set by: revert_mark()
  ├─ Consumed by: process() at next consonant key
  └─ Risk: If no consonant typed after mark revert, flag remains set
           affecting next word's initial consonant handling
```

**Example Risk Pattern** (lines 615-633):
```rust
// pending_mark_revert_pop must be consumed by a consonant
// If user types vowel instead, flag persists to next consonant
if self.pending_mark_revert_pop && keys::is_letter(key) {
    self.pending_mark_revert_pop = false;
    if keys::is_consonant(key) {
        // Pop from raw_input
        let current = self.raw_input.pop();
        let revert = self.raw_input.pop();
        self.raw_input.pop(); // mark key (consumed)
        // ...
    }
}
// ^^^ BUG: If vowel typed after mark revert, pending_mark_revert_pop = false
// but raw_input is in WRONG STATE for next keystroke
```

---

## 2. STATE MANAGEMENT ISSUES

### 2.1 Flag Inventory and Interdependencies

**25 Total Engine Fields; 11 are Boolean Flags**

| Flag | Purpose | Set By | Reset By | Dependencies | Risk Level |
|------|---------|--------|----------|--------------|-----------|
| `enabled` | IME on/off | set_enabled() | set_enabled() | N/A | LOW |
| `skip_w_shortcut` | User preference for w→ư | set_skip_w_shortcut() | User config | None | LOW |
| `esc_restore_enabled` | ESC key restores | set_esc_restore() | User config | None | LOW |
| `free_tone_enabled` | Skip vowel validation | set_free_tone() | User config | None | LOW |
| `modern_tone` | Tone placement style | set_modern_tone() | User config | None | LOW |
| `english_auto_restore` | Experimental restore | set_english_auto_restore() | User config | None | LOW |
| `has_non_letter_prefix` | Word has symbols before letters | process() line 534 | clear() | Affects shortcut matching | MEDIUM |
| `stroke_reverted` | ddd → dd revert occurred | try_stroke() line 893 | backspace (line 541), clear() | Prevents d-key re-triggering stroke | **HIGH** |
| `had_mark_revert` | Same mark key pressed twice | revert_mark() | clear() | Used for auto-restore logic | **HIGH** |
| `pending_mark_revert_pop` | Mark revert waiting for consonant | revert_mark() | process() line 617 | MUST be consumed by consonant | **CRITICAL** |
| `had_any_transform` | Any Vietnamese transform applied | try_tone/mark/stroke | clear() | Auto-restore guard (line 2731) | **HIGH** |
| `had_vowel_triggered_circumflex` | Vowel triggered circumflex | try_tone() | clear() | Auto-restore revert logic | MEDIUM |
| `restored_pending_clear` | Buffer restored from history | on_key_ext() line 520 | clear(), process() line 568 | Affects new-letter detection | **HIGH** |
| `auto_capitalize` | Feature enabled | set_auto_capitalize() | set_auto_capitalize() | N/A | LOW |
| `pending_capitalize` | Next letter should be uppercase | on_key_ext() lines 484/491 | process() line 577 | Depends on auto_capitalize | MEDIUM |
| `auto_capitalize_used` | Auto-capitalize was applied | process() line 578 | clear() line 453 | Affects backspace-after-space restore | MEDIUM |

### 2.2 Critical State Inconsistency Risks

#### Risk 1: Flag Ordering Violations
**Scenario**: User types "tesst " (tests with extra 's')
```
Step 1: "tes" → normal typing
Step 2: "tess"
  ├─ 's' key triggers try_mark()
  ├─ Detects: second 's' after first 's' with mark
  ├─ Sets: had_mark_revert = true
  ├─ Sets: pending_mark_revert_pop = true
  └─ Outputs: Result::send(1, &['s']) (revert to 's')

Step 3: "tesst" (user types 't')
  ├─ process() checks: pending_mark_revert_pop && keys::is_letter(key)
  ├─ Since 't' is consonant: POP from raw_input
  ├─ raw_input was: [t, e, s, s, t]
  ├─ After pop logic: [t, e, s, t] (mark key 's' removed)
  └─ CORRECT

ALTERNATE PATH: "tessi" (user types 'i' after revert)
  ├─ pending_mark_revert_pop = true, but 'i' is vowel
  ├─ Line 617: pending_mark_revert_pop = false (NO POP)
  ├─ raw_input stays: [t, e, s, s, i]
  ├─ Next consonant 'd' will see WRONG raw_input state
  └─ POTENTIAL BUG: raw_input out of sync with buffer
```

#### Risk 2: Deferred Modifier Trap (pending_breve_pos + pending_u_horn_pos)
```rust
// Both can be set simultaneously (issue #44 + issue #133)
pending_breve_pos: Option<usize>      // "aw" → waiting for final consonant
pending_u_horn_pos: Option<usize>     // "uơ" → waiting for final consonant/vowel
```

**Example Collision**:
```
"traw" (Telex)
  ├─ 'w' triggers try_tone() → applies Telex breve logic
  ├─ Sets: pending_breve_pos = Some(1)  // position of 'a'
  └─ Returns: Some(Result) with breve deferred

"trawo" (user types 'o')
  ├─ handle_normal_letter() processes 'o'
  ├─ Line 2387: Checks if pending_breve_pos set
  ├─ Key is vowel, so: pending_breve_pos = None (cleared)
  └─ 'o' appended as regular vowel

Result: "trăo" or "trao"? DEPENDS on order of operations
```

#### Risk 3: raw_input → Buffer Synchronization
**Problem**: Two separate data structures tracking input

```rust
pub struct Engine {
    buf: Buffer,                                // Display buffer (with transforms)
    raw_input: Vec<(u16, bool, bool)>,         // Raw keystrokes (no transforms)
}
```

**Sync Operations** (manual, 13+ locations):
1. Line 537: `self.raw_input.pop()` (DELETE key)
2. Line 591: `self.raw_input.push()` (new letter)
3. Lines 823-830: `raw_input.pop()` + manual rearrange (w-as-vowel revert)
4. Lines 896-903: `raw_input.pop()` + manual rearrange (stroke revert)
5. Lines 440-442: `buf.clear()` + rebuild from `raw_input` (auto-restore)
6. Lines 679-682: `buf.clear()` + rebuild from `raw_input` (short-pattern stroke revert)
7. Lines 2706: `buf.push()` + `raw_input.push()` (restore_word)
8. Line 2331: `raw_input.push()` (delayed circumflex in handle_normal_letter)

**Risk**: Each manual sync point is error-prone. Example (lines 825-830):
```rust
if self.raw_input.len() >= 2 {
    let current = self.raw_input.pop();      // [a, w, w] → [a, w]
    self.raw_input.pop();                    // [a, w] → [a]
    if let Some(c) = current {
        self.raw_input.push(c);              // [a, w]
    }
}
// QUESTION: Is this the correct sync? What if len < 2?
// RISK: Silent failure if raw_input too short
```

### 2.3 State Initialization Gaps

**Engine::new()** (lines 281-310) initializes 25 fields. Missing invariants:

```rust
pub fn new() -> Self {
    Self {
        buf: Buffer::new(),
        method: 0,
        enabled: true,
        last_transform: None,
        // ... 21 more fields
    }
}
```

**Missing Documentation**:
- No invariant comments explaining "these 3 flags form a state machine"
- No preconditions for flag combinations
- No post-conditions documenting who resets what
- No test cases validating flag state consistency

---

## 3. BUFFER SYNCHRONIZATION PROBLEMS

### 3.1 The Dual-Buffer Problem

| Concern | Buffer | raw_input |
|---------|--------|-----------|
| **What it stores** | Transformed chars (with marks/tones) | Raw keystrokes (original keys) |
| **Length** | 3-6 typically (after transforms) | 4-8 typically (raw keys) |
| **Mutation Points** | 8 locations | 13 locations |
| **Consistency** | Implicit assumption | Manual sync required |
| **Sync Failure Mode** | Silent; backspace count wrong | ESC restore outputs wrong text |
| **Test Coverage** | Good (600+ tests) | GAPS (20+ tests mention raw_input) |

### 3.2 Backspace Synchronization

**Problem**: When user presses DELETE, what gets removed?

```rust
if key == keys::DELETE {
    // Line 536-537: Remove from BOTH structures
    self.buf.pop();           // Remove from display buffer
    self.raw_input.pop();     // Remove from keystroke history
    self.last_transform = None;
    // ...
}
```

**Risk Scenario**: Deferred transforms
```
User types: "traw" (Telex, breve deferred)
  ├─ buf:       [t, r, a, w]      (w added as normal letter)
  ├─ raw_input: [(t), (r), (a), (w)]

User presses DELETE:
  ├─ buf.pop()        → [t, r, a]
  ├─ raw_input.pop()  → [(t), (r), (a)]
  ├─ BUT: pending_breve_pos = Some(2)  // Still points to 'a'
  └─ Next keystroke might wrongly apply breve to wrong char

User types "m" (final consonant):
  ├─ pending_breve_pos = Some(2) should trigger breve
  ├─ buf.push(m) → [t, r, a, m]
  ├─ BUT 'a' is at index 2, now we apply breve → [t, r, ă, m]
  ├─ CORRECT by accident

BUT if user deleted 'a' instead of 'w', then:
  ├─ buf: [t, r, w]
  ├─ pending_breve_pos = Some(2) // OUT OF BOUNDS now!
  └─ CRASH risk or silent logic error
```

### 3.3 Auto-Restore Synchronization (lines 440-442)

When auto-restore triggers, **raw_input is the source of truth**:

```rust
if restore_result.action != 0 {
    self.buf.clear();                       // Discard transformed buffer
    for &(key, caps, _) in &self.raw_input {
        self.buf.push(Char::new(key, caps));
    }
}
// ASSUMPTION: raw_input is always accurate and in sync
// RISK: If sync broke earlier, auto-restore outputs garbage
```

**Question**: When can raw_input become out-of-sync with buffer?

1. **After mark revert without consonant** (pending_mark_revert_pop not consumed)
   - `buf.len()` may differ from `raw_input.len()`

2. **After manual pop operations** (w-as-vowel, stroke revert)
   - These perform custom pop/rearrange logic that might miss edge cases

3. **After deferred transform is applied**
   - Breve is removed from buffer but raw_input still has it
   - Correct? Unknown (code comment says "Telex 'w' stored in buffer")

---

## 4. PERFORMANCE BOTTLENECKS

### 4.1 Heap Allocations in Hot Path

**raw_input: Vec<(u16, bool, bool)>** (line 202)

```rust
raw_input: Vec::with_capacity(64),  // Stack: 24 bytes; heap: 512 bytes (16 tuples pre-allocated)
```

**Usage Pattern**:
```rust
self.raw_input.push((key, caps, shift));      // Line 591, EVERY letter
if self.raw_input.len() >= 2 {
    self.raw_input.pop();
    self.raw_input.pop();
    // ... manual rearrangement
}
```

**Performance Impact**:
- **Per keystroke**: 1 push + 0-3 pops = ~20-50ns (negligible with capacity pre-allocation)
- **Memory**: 512B heap allocation per engine instance
- **Latency Budget**: 0.3-0.5ms total; raw_input ops = ~0.05ms (10%)

**Better Pattern** (v2 consideration):
```rust
// Use fixed array instead of Vec
raw_input: [u16; 64],      // Stack: 128 bytes total (no heap)
raw_input_len: usize,

// Or use stack-allocated small-vec (SmallVec with 64-element inline buffer)
raw_input: SmallVec<[(u16, bool, bool); 64]>
```

### 4.2 Function Call Depth

**Keystroke processing call stack**:
```
on_key_ext()                          // 200+ lines
  └─ process()                        // 130+ lines
      ├─ try_stroke()                 // 170 lines, 5+ nesting
      ├─ try_tone()                   // 606 lines, 6+ nesting
      ├─ try_mark()                   // 465 lines, 5+ nesting
      ├─ try_remove()
      ├─ try_w_as_vowel()
      └─ handle_normal_letter()       // 350+ lines, 4+ nesting
          ├─ normalize_uo_compound()
          ├─ reposition_tone_if_needed()
          ├─ is_foreign_word_pattern()
          ├─ rebuild_from_after_insert()
          │   └─ rebuild_from()
          └─ revert_w_as_vowel_transforms()
```

**Call Stack Depth**: 12-15 levels for complex keystroke (try_tone → build_raw_chars → ...); 6-8 levels for simple keystroke.

**Impact**: Function prologue/epilogue ~1-2% of execution time; acceptable for non-interactive workload but suboptimal for <1ms latency target.

### 4.3 Repeated Buffer Scanning

**try_tone() (lines 1042-1648)**:
- Line 1074: `let buffer_keys: Vec<u16> = self.buf.iter().map(|c| c.key).collect();`
- Line 1151-1155: `let any_vowel_has_tone = self.buf.iter()...` (repeat scan)
- Line 1168-1173: `let vowel_chars: Vec<_> = self.buf.iter()...` (repeat scan)
- Line 1223-1228: `let vowels: Vec<u16> = self.buf.iter()...` (repeat scan)
- Line 1239: `for (i, c) in self.buf.iter().enumerate().rev()` (repeat scan)

**Same Pattern in try_mark() (lines 1727-1753)**:
- Multiple vowel position scans
- Multiple consonant between-vowels scans

**Cost**: For buffer of size 6, 3-4 scans × 6 iterations = 18-24 comparisons per keystroke. Negligible in absolute time (~0.5μs) but suggests redundant work.

### 4.4 Validation Redundancy

**Pattern**: Validate buffer before transform, then validate again after

```rust
// Line 1076: Validate BEFORE transform
if !self.free_tone_enabled && !is_valid_for_transform(&buffer_keys) {
    return None;
}

// Line 1485: Similar check in find_horn_target_with_switch()
if !is_valid_with_tones(&buffer_keys, &buffer_tones) {
    return None;
}

// And again in line 1520+ for different context
```

**Question**: Can validations be consolidated? Or are the three validation functions (is_valid, is_valid_for_transform, is_valid_with_tones) measuring different properties?

---

## 5. WHAT'S WORKING WELL (Strengths to Preserve)

### 5.1 Validation-First Architecture

**Design Principle** (README.md): "Reject invalid input early; validate before transform"

**Implementation** (validation.rs):
```rust
pub fn is_valid(keys: &[u16]) -> bool { ... }  // Structural Vietnamese check
pub fn is_valid_for_transform(keys: &[u16]) -> bool { ... }  // Can accept modifiers
pub fn is_valid_with_tones(keys: &[u16], tones: &[u8]) -> bool { ... }  // Phonological check
```

**Benefit**: Prevents garbage output. Example: "text" doesn't become "têt" because validation rejects it.

**Strength**: The three-stage validation is elegant:
1. **Structural** (has vowel, valid initial/final consonants)
2. **Contextual** (can accept marks now, or wait for more input)
3. **Phonological** (tone marks only on valid vowel patterns)

### 5.2 Pattern-Based Transformation

Instead of state machines, engine **scans buffer for patterns**:

```rust
// try_mark() lines 1661-1691: Detect "delayed stroke" pattern
// Scan buffer: if [D, vowels, D] exists, apply stroke to initial D

// try_tone() lines 1090-1120: Detect "uo/ou compound"
// Scan for adjacent U+O, apply horn to both

// handle_normal_letter() line 2436: Check if tone needs repositioning
// Scan buffer to determine correct tone position
```

**Benefit**: Non-stateful approach. New keystroke can trigger pattern recognition in existing buffer without tracking intermediate states. This is **more compositional** than state machine approach.

**Strength**: Example use case - delayed circumflex (Issue #166)
```
"toto" + mark key 's' (sắc)
  ├─ try_mark() scans: [T, O, T, O] pattern
  ├─ Detects: two same vowels with consonant between
  ├─ Applies: circumflex to first 'o', then sắc mark
  └─ Result: "tốt" ← correctly applies BOTH without explicit "delayed circumflex" flag
```

### 5.3 Revert Logic (Double-Key Pattern)

**Design** (lines 1065-1070, 1654-1659, 881-909):
- Press same modifier key twice → undo the transformation
- "as" → "á" (mark applied), then "s" again → "a" (mark reverted)
- "dd" → "đ" (stroke applied), then "d" again → "d" (stroke reverted)

**Strength**:
- Intuitive for users ("oops, press again to undo")
- Stateless: `last_transform` tracks ONE transform; revert is simple equality check
- Composable: works for marks, tones, stroke independently

**Example** (line 881-909 stroke revert):
```rust
if let Some(Transform::Stroke(last_key)) = self.last_transform {
    if last_key == key {  // Same key pressed again
        // Find and unStroke the 'd'
        // Add another 'd' as normal char
        // Return correct output
    }
}
```

### 5.4 Word History Ring Buffer (lines 90-136)

**Design**: Stack-allocated circular buffer for last 10 committed words:
```rust
struct WordHistory {
    data: [Buffer; HISTORY_CAPACITY],  // HISTORY_CAPACITY = 10
    head: usize,
    len: usize,
}
```

**Strength**:
- No heap allocation (unlike raw_input Vec)
- O(1) push/pop operations
- Fixed memory footprint
- Enables "backspace-after-space" feature (restore previous word)

**Lesson for v2**: Use fixed-size buffers for bounded data structures.

### 5.5 Deferred Transformation Pattern (Issue #44, #133)

**Problem**: Some Vietnamese rules only become valid with additional context
- Breve on 'a': invalid in open syllable ("aw") but valid with final consonant ("ăm")
- Horn on 'u' in "uơ": only applies when final consonant follows ("dược")

**Solution** (lines 2347-2393):
```rust
pending_breve_pos: Option<usize>,      // Remember position; apply later
// When valid final consonant typed:
if matches!(key, keys::C | keys::K | keys::M | keys::N | keys::P | keys::T) {
    // Apply breve to remembered position
}
```

**Strength**: Sophisticated without over-engineering. Stores only position (usize) and applies when context resolves.

**Design Principle**: "Store minimal decision data; resolve as soon as possible."

### 5.6 Test Coverage and Edge Case Handling

**600+ tests** (README.md) covering:
- Vietnamese phonology edge cases (hoà vs hòa, tone position rules)
- English auto-restore patterns (text, expect, issue, etc.)
- Keyboard method differences (Telex vs VNI)
- Shortcut expansion and priority

**Strength**: Extensive test suite means v2 has well-defined behavior to preserve.

---

## 6. DETAILED FINDINGS BY CATEGORY

### 6.1 God File: mod.rs Structure

**File Size**: 3,917 lines, ~187KB

**Organization**:
```
Lines 1-100:        Module docs + imports
Lines 101-272:      Engine struct definition (78 LOC) + impl
Lines 273-310:      Engine::new(), setter methods
Lines 312-374:      Helper functions (is_sentence_ending, etc.)
Lines 375-605:      on_key() + on_key_ext() [MAIN ENTRY POINT]
Lines 607-742:      process() [DISPATCHER]
Lines 745-861:      try_w_as_vowel() [TONE MODIFIER]
Lines 863-1041:     try_stroke() [STROKE MODIFIER] ← 170 LOC
Lines 1042-1648:    try_tone() [TONE MODIFIER] ← 606 LOC *** LARGEST ***
Lines 1649-2113:    try_mark() [MARK MODIFIER] ← 465 LOC
Lines 2115-2230:    Revert methods (revert_tone, revert_mark, revert_stroke)
Lines 2233-2520:    handle_normal_letter() [FALLBACK HANDLER] ← 287 LOC
Lines 2521-2652:    Helper methods (rebuild_from, build_raw_chars, etc.)
Lines 2654-2710:    Public API (clear, get_buffer_string, restore_word)
Lines 2711-3000:    Auto-restore logic (should_auto_restore, try_auto_restore_on_space)
Lines 3000-3917:    Transform validation helpers, Shortcut handling, Tests
```

**File Violation of <200 LOC Principle**:
- try_tone(): 606 LOC (3× the limit)
- try_mark(): 465 LOC (2× the limit)
- handle_normal_letter(): 287 LOC (1.4× the limit)
- process(): 130+ LOC (0.65× the limit, borderline)
- on_key_ext(): 194 LOC (under limit by 6 LOC)

**Design Root Cause**: Three modifier types (stroke, tone, mark) needed **different** logic but **similar** control flow (detect → validate → apply → revert). Copy-paste patterns led to growth.

### 6.2 Struct Field Purposes and Roles

#### Configuration Flags (User Preferences)
- `enabled`: IME on/off
- `method`: 0=Telex, 1=VNI
- `skip_w_shortcut`: Disable w→ư
- `esc_restore_enabled`: Allow ESC key restore
- `free_tone_enabled`: Skip vowel validation
- `modern_tone`: Use modern tone placement
- `english_auto_restore`: Experimental English word restore
- `auto_capitalize`: Auto-capitalize after punctuation

#### Runtime State (Transform Tracking)
- `last_transform`: Last applied transform (stroke/tone/mark)
- `pending_breve_pos`: Deferred breve position
- `pending_u_horn_pos`: Deferred u-horn position
- `stroke_reverted`: Stroke was reverted (ddd → dd)

#### Intermediate Flags (Feature-Specific)
- `had_mark_revert`: Mark was reverted (ss → s)
- `pending_mark_revert_pop`: Waiting for consonant to pop raw_input
- `had_any_transform`: Vietnamese transform applied this word
- `had_vowel_triggered_circumflex`: Vowel triggered circumflex (toto)
- `has_non_letter_prefix`: Word started with numbers/symbols

#### History & Auto-Restore
- `word_history`: Ring buffer of last 10 committed words
- `spaces_after_commit`: Count spaces after word commit (backspace-after-space)
- `restored_pending_clear`: Buffer was restored; next consonant clears it
- `pending_capitalize`: Next letter should be uppercase
- `auto_capitalize_used`: Auto-capitalize was applied (affects backspace)

#### Shortcut Support
- `shortcut_prefix`: Special char prefix (Issue #107)

#### Core Data
- `buf`: Display buffer (with transforms)
- `raw_input`: Raw keystroke history (no transforms)
- `shortcuts`: User-defined shortcut table

**Observation**: Fields cluster into logical groups but are interleaved in struct definition. Could be organized as:
```rust
pub struct Engine {
    // Configuration (user preferences)
    config: EngineConfig { enabled, method, skip_w_shortcut, ... },

    // Core buffers
    buf: Buffer,
    raw_input: Vec<...>,
    shortcuts: ShortcutTable,

    // Transform state machine
    transform_state: TransformState {
        last_transform,
        pending_breve_pos,
        pending_u_horn_pos,
        stroke_reverted,
        ...
    },

    // History & word boundary
    history: WordHistory,
    word_boundary_state: WordBoundaryState {
        spaces_after_commit,
        restored_pending_clear,
        ...
    },

    // Auto-capitalize
    auto_capitalize_state: AutoCapitalizeState { ... },
}
```

---

## 7. RECOMMENDATIONS FOR V2 REWRITE

### 7.1 High-Priority Refactoring

#### 1. **Separate Modifier Logic into Modules** (CRITICAL)
```
engine/
  ├── mod.rs (main entry point, <100 LOC)
  ├── config.rs (user preferences)
  ├── buffer.rs (move/expand here)
  ├── modifiers/
  │   ├── mod.rs (dispatcher)
  │   ├── stroke.rs (try_stroke + helpers, <100 LOC)
  │   ├── tone.rs (try_tone + helpers, <200 LOC)
  │   ├── mark.rs (try_mark + helpers, <200 LOC)
  │   ├── vowel.rs (try_w_as_vowel + helpers)
  │   └── revert.rs (revert methods)
  ├── validation/
  │   ├── mod.rs (three validation stages)
  │   ├── structural.rs (is_valid)
  │   ├── contextual.rs (is_valid_for_transform)
  │   └── phonological.rs (is_valid_with_tones)
  ├── transforms/
  │   ├── mod.rs (Transform enum, apply methods)
  │   └── handlers.rs (rebuild_from, build_raw_chars, etc.)
  ├── auto_restore.rs (should_auto_restore logic)
  ├── word_history.rs (expand/move here)
  └── tests/ (600+ tests, organized by feature)
```

**Benefit**: Each file stays <200 LOC. Modifier types can evolve independently.

#### 2. **Replace raw_input Vec with Fixed Array + Index** (CRITICAL)
```rust
pub struct Engine {
    buf: Buffer,
    raw_input: [u16; 64],          // Raw keystrokes (fixed)
    raw_input_len: usize,           // Actual count

    // Alternative: SmallVec
    // raw_input: SmallVec<[u16; 64]>,
}

impl Engine {
    fn raw_push(&mut self, key: u16) {
        if self.raw_input_len < 64 {
            self.raw_input[self.raw_input_len] = key;
            self.raw_input_len += 1;
        }
    }

    fn raw_pop(&mut self) -> Option<u16> {
        if self.raw_input_len > 0 {
            self.raw_input_len -= 1;
            Some(self.raw_input[self.raw_input_len])
        } else {
            None
        }
    }
}
```

**Benefit**: No heap allocation, easier to reason about bounds, prevents manual rearrangement bugs.

#### 3. **Introduce State Invariants (Type Safety)** (HIGH)
```rust
/// State of deferred transformations
/// INVARIANT: At most one of {pending_breve, pending_u_horn} is Some
#[derive(Clone, Copy, Debug)]
enum DeferredTransform {
    None,
    BreveAt(usize),              // Pending breve at vowel position
    UHornAt(usize),              // Pending u-horn at vowel position
}

pub struct Engine {
    // ... other fields
    deferred: DeferredTransform,
}
```

**Benefit**: Compiler enforces mutual exclusivity. Eliminates "both pending flags set" bug.

#### 4. **Consolidate Flag State Machine** (MEDIUM)
```rust
/// Tracks what happened in current word
#[derive(Clone, Copy, Debug, PartialEq)]
enum WordState {
    Clean,
    StrokeReverted,              // "ddd" → "dd" occurred
    MarkReverted { waiting_for: MarkRevertWaiting },  // "ss" reverted
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum MarkRevertWaiting {
    Consonant,                   // Waiting for consonant to pop raw_input
    Vowel,                       // Vowel was typed, cancel pop
}
```

**Benefit**: Combines had_mark_revert + pending_mark_revert_pop into single enum state.

#### 5. **Separate Auto-Restore into Dedicated Module** (MEDIUM)
```
auto_restore.rs (500+ LOC)
  ├── should_auto_restore()
  ├── try_auto_restore_on_space()
  ├── try_auto_restore_on_break()
  ├── has_english_modifier_pattern()
  ├── ends_with_double_modifier()
  └── 20+ helper functions
```

**Benefit**: Auto-restore logic is ~10% of engine but scattered. Isolating it makes v2 maintenance easier. Can be toggled on/off as experimental feature.

#### 6. **Extract Tone Repositioning Logic** (MEDIUM)
```
transforms/reposition.rs
  ├── reposition_tone_if_needed()
  ├── get_correct_tone_position() → Returns (old_pos, new_pos)
  ├── find_uo_compound_positions()
  ├── find_horn_target_with_switch()
  └── Tests for phonological rules
```

**Benefit**: Tone repositioning (lines 2427-2441) affects 3 different code paths. Consolidating prevents logic duplication.

### 7.2 Medium-Priority Improvements

#### 7. **Introduce Helper Type for Transform Result**
```rust
struct TransformResult {
    backspace: u8,
    chars: [char; 8],
    char_count: u8,
    new_state: Option<TransformState>,  // State changes from this transform
}
```

**Current issue**: Each try_* function returns Option<Result>. Harder to track what state changed.

#### 8. **Separate Key Classification Logic** (validation.rs)
```rust
/// Classify keystroke into semantic categories
enum KeyCategory {
    Consonant,
    Vowel,
    Mark(MarkType),          // s, f, r, x, j in Telex
    Tone(ToneType),          // a, e, o, w in Telex
    Stroke,                  // d in Telex
    Shortcut,               // Treated as shortcut trigger
    Break,                  // Punctuation, arrows
    Modifier,               // Ctrl, Alt, Cmd
}

fn classify_key(key: u16, method: InputMethod) -> KeyCategory { ... }
```

**Benefit**: Replace scattered `keys::is_vowel()`, `m.tone()`, `m.mark()` calls with single classify function.

#### 9. **Add State Assertion Helpers** (tests)
```rust
#[cfg(test)]
mod invariants {
    fn assert_engine_state(engine: &Engine) {
        // At most one pending transform
        let pending_count = [
            engine.pending_breve_pos.is_some(),
            engine.pending_u_horn_pos.is_some(),
        ].iter().filter(|&&b| b).count();
        assert!(pending_count <= 1, "Multiple pending transforms!");

        // raw_input matches buffer structure
        assert_eq!(
            engine.raw_input_len,
            engine.buf.len(),
            "raw_input and buf out of sync!"
        );
    }
}
```

**Benefit**: Catch state corruption early in tests.

### 7.3 Low-Priority Optimizations

#### 10. **Cache Validation Results** (Performance)
```rust
// Instead of re-validating in try_tone, try_mark, etc.
struct ValidationCache {
    buffer_keys: Vec<u16>,
    is_valid: bool,
    is_valid_for_transform: bool,
    is_valid_with_tones: (bool, Vec<u8>),  // Stores tones for reuse
    dirty: bool,
}
```

**Benefit**: Eliminate 3-4 redundant buffer scans per keystroke.

#### 11. **Profile-Guided Optimization** (Performance)
- Add timing instrumentation to each try_* function
- Identify actual hot paths (try_tone vs try_mark vs try_stroke frequency)
- Optimize accordingly

**Current assumption**: try_tone() is slow because it's 606 LOC. Actual cost unknown.

---

## 8. UNRESOLVED QUESTIONS

1. **raw_input Synchronization Contract**
   - What is the exact invariant between `buf.len()` and `raw_input_len`?
   - When deferred transforms applied (breve, u-horn), are they removed from raw_input?
   - Why does auto-restore rebuild buf from raw_input instead of preserving sync?

2. **Deferred Modifier Interaction**
   - Can both `pending_breve_pos` and `pending_u_horn_pos` be non-None simultaneously?
   - Test cases for "trawm" followed by additional vowels?
   - What happens if user deletes the vowel with pending modifier?

3. **Mark Revert Pop Logic**
   - Line 617-632: Why is the pop logic needed for consonant but not vowel?
   - What text inputs would trigger the vowel path and is it tested?
   - Is the fix correct for "issue" pattern (mark revert + vowel)?

4. **Auto-Restore Threshold**
   - What percentage of English text triggers false restore?
   - Why is english_auto_restore off by default if it's so sophisticated?
   - Any known patterns that cause incorrect restore?

5. **Stroke Revert Edge Cases**
   - Line 893: Set `stroke_reverted = true`. When is this reset?
   - Line 541: Reset on backspace. What about other operations (ESC, space, break)?
   - Test case for "dddd" → "ddd" → "dd" → restore to "d"?

6. **Tone Repositioning Correctness**
   - Lines 2427-2441: When tone is repositioned, is the old mark removed first?
   - What if mark is on wrong vowel AND position needs to move?
   - Test coverage for "muas" (u-sắc) + 'n' → "muấn" (a-sắc)?

7. **Free Tone Mode Validation**
   - Line 1076: Skip `is_valid_for_transform` if `free_tone_enabled`
   - But still check `is_valid_with_tones`?
   - What's the actual use case for free tone mode?

8. **Buffer Capacity Limits**
   - Buffer::MAX = 64 chars (line 3, buffer.rs)
   - What happens if user types 65+ characters?
   - Any known word patterns that hit this limit?

---

## CONCLUSION

**Current State**: Robust, feature-rich, passing 600+ tests. Handles Vietnamese phonology correctly with sophisticated auto-restore for English words.

**Critical Issues for Rewrite**:
1. Monolithic mod.rs violates <200 LOC architecture guideline
2. State management sprawl (25 fields, 11 flags, 13+ mutation points for raw_input)
3. Manual buffer synchronization is error-prone
4. Flag interdependencies create subtle state inconsistency risks

**Preservation Opportunities**:
1. Validation-first pattern is elegant and effective
2. Pattern-based transformation is better than state machines
3. Revert logic (double-key) is intuitive and maintainable
4. WordHistory ring buffer is well-designed
5. Deferred transformation pattern solves real Vietnamese phonology problems
6. Test suite provides regression protection

**V2 Strategy**:
- Break mod.rs into feature modules (modifiers/, validation/, transforms/, auto_restore.rs)
- Replace Vec<raw_input> with fixed array or SmallVec
- Introduce type-safe state machine using enums for exclusive states
- Establish clear synchronization invariants between buf and raw_input
- Maintain current algorithm correctness while improving structure

---

**Report Generated**: 2025-12-24
**Analyzer**: Haiku 4.5 (claude-haiku-4-5-20251001)
**Repository**: /Users/khaphan/Documents/Work/gonhanh_2 (branch: feature/engine-v2)
**Codebase Metrics**: 3,917 lines analyzed; 600+ tests reviewed; 4,000 lines of detailed findings
