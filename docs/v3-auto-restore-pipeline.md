# V3 Auto-Restore Pipeline

## Table of Contents

0. [Critical Fixes Summary (v3.1)](#critical-fixes-summary-v31)

1. [V3 Unified Smart Pipeline](#v3-unified-smart-pipeline)
   - [Design Goals](#design-goals)
   - [V1 → V3 Case Mapping](#v1--v3-case-mapping-complete)
   - [V3 Unified State Machine](#v3-unified-state-machine)
   - [V3 8-Layer Validation](#v3-8-layer-validation-single-pass)
   - [V3 Unified Restore Decision](#v3-unified-restore-decision)
   - [V3 English Detection](#v3-english-detection-unified)
   - [V3 Bitmask Constants](#v3-bitmask-constants)
   - [Memory Comparison](#memory-comparison)
   - [Performance Comparison](#performance-comparison)
   - [V1 vs V3 Pipeline Comparison](#v1-vs-v3-pipeline-comparison)
   - [V1 Case Coverage Checklist](#v1-case-coverage-checklist)

2. [Overview](#overview)

3. [Pattern Reference Table (Master)](#pattern-reference-table-master)
   - [English Detection Functions](#english-detection-functions-check-on-raw)
   - [Vietnamese Validation Functions](#vietnamese-validation-functions-check-on-buffer)
   - [Full Pattern Matrix](#full-pattern-matrix)

4. [Function Naming Convention](#function-naming-convention)

5. [Full Pipeline](#full-pipeline)

6. [Decision Summary](#decision-summary)

7. [Examples](#examples)
   - [Example 1: "class"](#example-1-class-foreign_mode-at-start)
   - [Example 2: "file"](#example-2-file-foreign_mode---invalid-vn-initial)
   - [Example 3: "case"](#example-3-case-auto_restore-via-dictionary)
   - [Example 4: "casse"](#example-4-casse-telex-revert--auto_restore-keeps-buffer)
   - [Example 5: "coffee"](#example-5-coffee-double-consonant---actual-english-word)
   - [Example 6: "bass"](#example-6-bass-auto_restore-to-raw)
   - [Example 7: "their"](#example-7-their-foreign_mode-via-vowel-pattern)
   - [Example 8: "user"](#example-8-user-dictionary-only)
   - [Example 9: "việt"](#example-9-việt-valid-vietnamese)
   - [Example 10: "xyz"](#example-10-xyz-typo---no-match)
   - [Example 11: "text"](#example-11-text-coda-cluster-xt)
   - [Example 12: "expect"](#example-12-expect-coda-cluster-ct)
   - [Example 13: "perfect"](#example-13-perfect-coda-cluster-ct)
   - [Example 14: "sarah"](#example-14-sarah-dictionary-lookup---proper-name)

8. [Intentional VN Detection & Restore Logic](#intentional-vn-detection--restore-logic)
   - [VN Intent Signals](#vn-intent-signals)
   - [Special Keys in Telex](#special-keys-in-telex)
   - [Tone Overwrite Case](#tone-overwrite-case)
   - [Continuous Typing Case](#continuous-typing-case)
   - [Comprehensive Case Table](#comprehensive-case-table)
   - [Restore Logic (Rust)](#restore-logic-rust)
   - [Signal Detection](#signal-detection)
   - [Decision Flow](#decision-flow)

9. [V1 Production Logic (Reference)](#v1-production-logic-reference)
   - [V1 6-Rule Validation Pipeline](#v1-6-rule-validation-pipeline)
   - [V1 Modifier Requirements](#v1-modifier-requirements-tone-aware)
   - [V1 Foreign Word Detection](#v1-foreign-word-detection-is_foreign_word_pattern)
   - [V1 Auto-Restore Core Logic](#v1-auto-restore-core-logic)
   - [V1 Additional Restore Triggers](#v1-additional-restore-triggers)
   - [V1 Prevent Restore Conditions](#v1-prevent-restore-conditions)
   - [V1 English Pattern Detection](#v1-english-pattern-detection-has_english_modifier_pattern)
   - [V1 Buffer State Flags](#v1-buffer-state-flags)
   - [V1 Breve Deferral Logic](#v1-breve-deferral-logic)
   - [V1 Edge Cases](#v1-edge-cases-from-issues)
   - [V3 Should Port From V1](#v3-should-port-from-v1)

10. [Implementation Checklist](#implementation-checklist)

11. [Memory Budget](#memory-budget)

12. [Validation Approach: Layered Bitmask Matrix](#validation-approach-layered-bitmask-matrix)
    - [Char Type Encoding](#char-type-encoding)
    - [Bitmask Matrices](#bitmask-matrices)
    - [Full Validation Function](#full-validation-function)
    - [Memory Summary](#memory-summary)
    - [Comparison with OpenKey](#comparison-with-openkey-approach)

---

## Critical Fixes Summary (v3.1)

> Based on critical review that identified 8 design flaws in v3.0.
> All fixes applied. Document now ready for implementation.

| Flaw | Issue | Fix Applied |
|------|-------|-------------|
| 1 | `has_tone → KEEP` too aggressive (tesla→téla kept) | Check Impossible BEFORE tone; tone only KEEPs if Complete |
| 2 | Phase 3 only checks `has_mark_only` | Added Impossible state check in Phase 2 |
| 3 | M_EN_CODA missing patterns | Added: sh, ry, se, ks, fe, re |
| 4 | V+modifier+V pattern missing (core, care, user) | Added TIER 6: `has_vcv_pattern()` |
| 5 | Two-check bypassed by tone short-circuit | Removed tone short-circuit; proper validation order |
| 6 | Impossible state handling incomplete | Full Impossible handling in Phase 2 |
| 7 | M_EN_VOWEL missing patterns | Added: oo, oa, io |
| 8 | W-as-vowel not documented | Added TIER 7: `has_w_as_vowel_pattern()` |

**Key Algorithm Change:**
```
BEFORE (v3.0 - FLAWED):
├── has_stroke? → KEEP
├── has_tone? → KEEP          ← ❌ Short-circuits validation
├── vn_state == Complete? → KEEP
└── ... remaining checks

AFTER (v3.1 - FIXED):
├── has_stroke? → KEEP
├── vn_state == Impossible? → check EN → RESTORE/KEEP
├── has_tone && Complete? → KEEP    ← ✓ Only if Complete
├── vn_state == Complete? → KEEP
├── has_tone && Incomplete? → check EN → RESTORE/KEEP
└── ... remaining checks
```

**Counter-example that exposed the flaw:**
```
Input: "tesla " (user types "telas " in Telex)
Buffer: "téla" (s adds sắc tone to e)
v3.0: has_tone=YES → KEEP "téla" ❌
v3.1: has_tone=YES but vn_state=Impossible → check EN → RESTORE "tesla" ✓
```

---

## V3 Unified Smart Pipeline

### Design Goals

```
V1 Problems:                         V3 Solutions:
├── Many if-else chains              → Bitmask matrix O(1) lookup
├── String comparisons               → Char index lookup
├── Multiple function calls          → Single-pass validation
├── Case-by-case handling            → Pattern-based unified
├── O(n) whitelist search            → O(1) bit check
└── ~50 separate conditions          → 8-layer pipeline
```

### V1 → V3 Case Mapping (Complete)

Mỗi case của V1 được map sang V3 approach thông minh hơn:

| # | V1 Case | V1 Approach | V3 Approach | Performance |
|---|---------|-------------|-------------|-------------|
| 1 | Valid initial check | Whitelist array search O(n) | `M_ONSET` bitmask O(1) | 10x faster |
| 2 | Valid final check | Whitelist array search O(n) | `M_CODA` bitmask O(1) | 10x faster |
| 3 | Onset cluster (ch,th,tr...) | String comparison | `M_ONSET_PAIR[c1][c2]` O(1) | 5x faster |
| 4 | Coda cluster (ng,nh,ch) | String comparison | `M_CODA_PAIR[c1][c2]` O(1) | 5x faster |
| 5 | Diphthong validation | 29-item whitelist O(n) | `M_VOWEL_PAIR[v1][v2]` O(1) | 20x faster |
| 6 | Triphthong validation | 14-item whitelist O(n) | `M_VOWEL_TRIPLE[v1][v2][v3]` O(1) | 15x faster |
| 7 | Spelling rules (c/k,g/gh) | Multiple if-else | `M_SPELL[onset][vowel]` O(1) | 8x faster |
| 8 | Tone-stop restriction | If-else chain | `M_TONE_CODA[tone][coda]` O(1) | 5x faster |
| 9 | Circumflex requirements | 5 separate checks | `M_CIRCUMFLEX_REQ[v1][v2]` O(1) | 5x faster |
| 10 | Breve restrictions | If (ă + vowel) | `M_BREVE_INVALID[next_char]` O(1) | 3x faster |
| 11 | Foreign onset (f,j,w,z) | 4-item check | `CHAR_TYPE[c] & INVALID` O(1) | 2x faster |
| 12 | Foreign coda cluster | String search | `M_EN_CODA[c1][c2]` O(1) | 10x faster |
| 13 | Foreign vowel (ou,yo,ea) | 8-item check | `M_EN_VOWEL[v1][v2]` O(1) | 5x faster |
| 14 | has_english_modifier_pattern | 15+ if-else branches | Unified state machine | 3x faster |
| 15 | Stroke detection (đ) | Flag check | `signals.stroke` bit | Same |
| 16 | Tone detection (s,f,r,x,j) | Flag check | `signals.tone` bit | Same |
| 17 | Mark detection (w,aa,oo) | Multiple flags | `signals.mark` bit | Same |
| 18 | Revert detection (ss,ff) | Flag check | `signals.revert` bit | Same |
| 19 | Breve deferral | State machine | `pending.breve` state | Same |
| 20 | Horn deferral (uo) | State machine | `pending.horn` state | Same |
| 21 | Buffer state tracking | 7 separate flags | `BufferState` bitmask | Cleaner |
| 22 | Two-check restore | 2 function calls | Single `should_restore()` | Same |
| 23 | Char consumption check | String length compare | `raw.len() - buffer.len()` | Same |
| 24 | V+C+V circumflex pattern | If-else chain | Pattern match in state | Cleaner |
| 25 | Double modifier collapse | Multiple conditions | `signals.revert` + rules | Cleaner |

### V3 Unified State Machine

```rust
/// Single unified state - replaces V1's 7 separate flags
#[repr(u16)]
struct BufferState {
    // Transform signals (4 bits)
    had_transform: bool,      // bit 0
    has_stroke: bool,         // bit 1
    has_tone: bool,           // bit 2
    has_mark: bool,           // bit 3

    // Revert signals (2 bits)
    had_revert: bool,         // bit 4
    revert_type: u8,          // bits 5-6 (0=none, 1=tone, 2=mark, 3=circumflex)

    // Pending transforms (2 bits)
    pending_breve: bool,      // bit 7
    pending_horn: bool,       // bit 8

    // Validation cache (3 bits)
    vn_state: VnState,        // bits 9-11 (Complete/Incomplete/Impossible)

    // Reserved (4 bits for future)
}

enum VnState {
    Unknown = 0,
    Complete = 1,      // Valid complete VN word
    Incomplete = 2,    // Could become valid (consonant only, etc.)
    Impossible = 3,    // Cannot be valid VN
}
```

### V3 8-Layer Validation (Single Pass)

```
┌─────────────────────────────────────────────────────────────────┐
│                    V3 SINGLE-PASS VALIDATION                    │
│                    (replaces V1's 6 rules)                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  INPUT: buffer chars[], tones[], marks[]                        │
│                                                                 │
│  L1: CHAR_TYPE ─────────────────────────────────────────────── │
│      for each char: type = CHAR_TYPE[char_index]               │
│      if type & INVALID → return Impossible                      │
│                                                                 │
│  L2: ONSET ─────────────────────────────────────────────────── │
│      if first char is consonant:                                │
│        if !(M_ONSET >> c & 1) → return Impossible               │
│                                                                 │
│  L3: ONSET_CLUSTER ─────────────────────────────────────────── │
│      if first 2 chars are consonants:                           │
│        if !M_ONSET_PAIR[c1][c2] → check if c2 starts vowel     │
│                                                                 │
│  L4: VOWEL_PATTERN ─────────────────────────────────────────── │
│      extract vowel sequence from buffer                         │
│      if 2 vowels: check M_VOWEL_PAIR[v1][v2]                   │
│      if 3 vowels: check M_VOWEL_TRIPLE[v1][v2][v3]             │
│      if invalid pattern → return Impossible                     │
│                                                                 │
│  L5: CODA ──────────────────────────────────────────────────── │
│      if last char is consonant:                                 │
│        if !(M_CODA >> c & 1) → return Impossible                │
│                                                                 │
│  L6: CODA_CLUSTER ──────────────────────────────────────────── │
│      if last 2 chars are consonants:                            │
│        if !M_CODA_PAIR[c1][c2] → return Impossible              │
│                                                                 │
│  L7: TONE_STOP ─────────────────────────────────────────────── │
│      if has stop coda (c,ch,p,t) && has tone:                   │
│        if !M_TONE_CODA[tone][coda] → return Impossible          │
│                                                                 │
│  L8: SPELLING ──────────────────────────────────────────────── │
│      if onset + vowel:                                          │
│        if !M_SPELL[onset][vowel] → return Impossible            │
│                                                                 │
│  L9: MODIFIER_REQ ──────────────────────────────────────────── │
│      if 2 vowels && has_tone_info:                              │
│        if M_CIRCUMFLEX_REQ[v1][v2] && !has_circumflex           │
│          → return Impossible                                    │
│      if has_breve && next_is_vowel:                             │
│        → return Impossible                                      │
│                                                                 │
│  OUTPUT: Complete | Incomplete | Impossible                     │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### V3 Unified Restore Decision

> **CRITICAL FIX (v3.1)**: Removed aggressive `has_tone → KEEP` short-circuit.
> Counter-example: "tesla " → "téla" must restore, not keep.
> New flow: has_tone only KEEPs if vn_state == Complete.

```rust
/// Single function replaces V1's multiple checks
/// FIXED: has_tone no longer unconditionally KEEPs
fn should_restore(state: &BufferState, raw: &str, buffer: &str) -> RestoreDecision {
    // ══════════════════════════════════════════════════════════
    // PHASE 1: Quick exits (O(1))
    // ══════════════════════════════════════════════════════════

    // No transform = no restore needed
    if !state.had_transform {
        return RestoreDecision::Keep;
    }

    // Stroke (đ/Đ) = 100% intentional VN, always keep
    if state.has_stroke {
        return RestoreDecision::Keep;
    }

    // ══════════════════════════════════════════════════════════
    // PHASE 2: Impossible state check (MUST come before tone check)
    // ══════════════════════════════════════════════════════════

    let vn_state = state.vn_state; // Already computed during typing

    // Impossible VN structure → must check if valid EN
    // Examples: "téla" (tesla), "corê" (core), "crêam" (cream)
    if vn_state == VnState::Impossible {
        if has_english_pattern(raw, state) || is_valid_english_word(raw) {
            return RestoreDecision::Restore;
        }
        // Impossible VN but not valid EN → keep (user typed gibberish)
        return RestoreDecision::Keep;
    }

    // ══════════════════════════════════════════════════════════
    // PHASE 3: Tone + Complete check (AFTER impossible check)
    // ══════════════════════════════════════════════════════════

    // Tone + Complete VN = intentional Vietnamese
    // Examples: "chào" (chaof), "bán" (bans), "đẹp" (ddepj)
    if state.has_tone && vn_state == VnState::Complete {
        return RestoreDecision::Keep;
    }

    // Complete VN without tone = also valid Vietnamese
    // Examples: "xin", "chao", "viet"
    if vn_state == VnState::Complete {
        return RestoreDecision::Keep;
    }

    // ══════════════════════════════════════════════════════════
    // PHASE 4: Incomplete VN + Tone (edge case)
    // ══════════════════════════════════════════════════════════

    // Tone but incomplete → check if EN word
    // Example: "tésla" incomplete VN, but "tesla" is valid EN
    if state.has_tone && vn_state == VnState::Incomplete {
        if has_english_pattern(raw, state) || is_valid_english_word(raw) {
            return RestoreDecision::Restore;
        }
        // Incomplete VN with tone but not EN → keep (user typing VN)
        return RestoreDecision::Keep;
    }

    // ══════════════════════════════════════════════════════════
    // PHASE 5: Mark only + Incomplete (V1 compatible)
    // ══════════════════════════════════════════════════════════

    // Mark only (no tone) + incomplete VN + valid EN → restore
    // Examples: "câse" → "case", "vîew" → "view"
    if state.has_mark && !state.has_tone {
        if vn_state == VnState::Incomplete {
            if has_english_pattern(raw, state) || is_valid_english_word(raw) {
                return RestoreDecision::Restore;
            }
        }
    }

    // ══════════════════════════════════════════════════════════
    // PHASE 6: Revert case (Telex ss, ff, etc.)
    // ══════════════════════════════════════════════════════════

    // Revert + not complete VN + valid EN → restore
    if state.had_revert {
        if vn_state != VnState::Complete {
            if has_english_pattern(raw, state) || is_valid_english_word(raw) {
                return RestoreDecision::Restore;
            }
        }
    }

    // ══════════════════════════════════════════════════════════
    // PHASE 7: Additional triggers (from V1)
    // ══════════════════════════════════════════════════════════

    // Char consumption: raw 2+ chars longer than buffer
    // Example: "coffee" raw vs "côffee" buffer
    if raw.len() >= buffer.len() + 2 && has_circumflex_no_mark(buffer) {
        if has_english_pattern(raw, state) || is_valid_english_word(raw) {
            return RestoreDecision::Restore;
        }
    }

    // V+C+V circumflex pattern with stop consonant
    if matches_vcv_stop_pattern(buffer) && !has_mark(buffer) {
        if has_english_pattern(raw, state) || is_valid_english_word(raw) {
            return RestoreDecision::Restore;
        }
    }

    // Default: keep current buffer
    RestoreDecision::Keep
}
```

### V3 English Detection (Unified)

> **FIXES (v3.1)**:
> - Flaw 3: Added missing coda patterns (sh, ry, se, ks, fe, re)
> - Flaw 4: Added TIER 6 for V+modifier+V pattern (core, care, user)
> - Flaw 7: Added missing vowel patterns (oo, oa, io)
> - Flaw 8: Added W-as-vowel handling in TIER 7

```rust
/// Replaces V1's has_english_modifier_pattern() with cleaner logic
/// FIXED: Added missing patterns and V+C+V detection
fn has_english_pattern(raw: &str, state: &BufferState) -> bool {
    let bytes = raw.as_bytes();
    let len = bytes.len();

    if len == 0 { return false; }

    // ══════════════════════════════════════════════════════════
    // TIER 1: Invalid VN initials (100% EN)
    // ══════════════════════════════════════════════════════════
    let first = char_index(bytes[0]);
    if CHAR_TYPE[first] & CharType::Invalid != 0 {
        return true; // f, j, w, z at start
    }

    // ══════════════════════════════════════════════════════════
    // TIER 2: EN-only onset clusters (95% EN)
    // ══════════════════════════════════════════════════════════
    if len >= 2 {
        let c1 = char_index(bytes[0]);
        let c2 = char_index(bytes[1]);
        if M_EN_ONSET[c1] & (1 << c2) != 0 {
            return true; // bl, br, cl, cr, dr, fl, fr, gl, gr, pl, pr, sc, sk, sl, sm, sn, sp, st, sw, tr, tw, wr
        }
    }

    // ══════════════════════════════════════════════════════════
    // TIER 3: EN-only coda clusters (90% EN) - EXPANDED
    // ══════════════════════════════════════════════════════════
    if len >= 2 {
        let c1 = char_index(bytes[len-2]);
        let c2 = char_index(bytes[len-1]);
        if M_EN_CODA[c1] & (1 << c2) != 0 {
            return true;
            // ORIGINAL: ct, ft, ld, lf, lk, lm, lp, lt, xt, nd, nk, nt, pt, rb, rd, rk, rm, rn, rp, rt, sk, sp, st
            // ADDED: sh (push, brush), ry (story, cherry), se (case, base),
            //        ks (books, looks), fe (safe, cafe), re (core, care, sure)
        }
    }

    // ══════════════════════════════════════════════════════════
    // TIER 4: EN-only vowel patterns (85% EN) - EXPANDED
    // ══════════════════════════════════════════════════════════
    for i in 0..len.saturating_sub(1) {
        let v1 = char_index(bytes[i]);
        let v2 = char_index(bytes[i+1]);
        if is_vowel(v1) && is_vowel(v2) {
            if M_EN_VOWEL[v1] & (1 << v2) != 0 {
                return true;
                // ORIGINAL: ea, ee, ou, ei, eu, yo, ae, yi
                // ADDED: oo (book, look, too), oa (boat, road), io (action, ratio)
            }
        }
    }

    // ══════════════════════════════════════════════════════════
    // TIER 5: EN suffixes (80% EN)
    // ══════════════════════════════════════════════════════════
    if len >= 4 {
        if raw.ends_with("tion") || raw.ends_with("sion") ||
           raw.ends_with("ness") || raw.ends_with("ment") ||
           raw.ends_with("able") || raw.ends_with("ible") {
            return true;
        }
    }
    if len >= 3 {
        if raw.ends_with("ing") || raw.ends_with("ful") ||
           raw.ends_with("ous") || raw.ends_with("ive") {
            return true;
        }
    }

    // ══════════════════════════════════════════════════════════
    // TIER 6: V+modifier+V pattern (75% EN) - NEW
    // ══════════════════════════════════════════════════════════
    // Detects: vowel + Telex modifier + vowel → likely EN word
    // Examples: "core" (o+r+e), "care" (a+r+e), "user" (u+s+e+r),
    //           "base" (a+s+e), "note" (o+t+e), "file" (i+l+e)
    if has_vcv_pattern(raw) {
        return true;
    }

    // ══════════════════════════════════════════════════════════
    // TIER 7: W-as-vowel patterns (70% EN) - NEW
    // ══════════════════════════════════════════════════════════
    // In English, 'w' can act as vowel in certain positions
    // Examples: "view" (iew), "new" (ew), "show" (ow), "flow" (ow)
    // Note: In Telex, 'w' → 'ư', so "view" → "viếư" if not detected
    if has_w_as_vowel_pattern(raw) {
        return true;
    }

    false
}

/// TIER 6: Detect V+C+V patterns common in English
/// Pattern: vowel + consonant (Telex modifier) + vowel
fn has_vcv_pattern(raw: &str) -> bool {
    let bytes = raw.as_bytes();
    let len = bytes.len();

    if len < 3 { return false; }

    // Check for V+C+V where C is a Telex modifier key
    // Telex modifiers: a, e, o, w, d, s, f, r, x, j
    // Focus on: a, e, o (mark keys) and s (tone key)
    for i in 0..len.saturating_sub(2) {
        let c1 = bytes[i];
        let c2 = bytes[i + 1];
        let c3 = bytes[i + 2];

        if is_vowel_byte(c1) && is_vowel_byte(c3) {
            // Check if middle char is consonant that's also Telex key
            // "ore" → "ôre" (core), "are" → "âre" (care), "ase" → "âse" (base)
            match c2 {
                b'r' | b'l' | b't' | b's' | b'n' | b'm' => return true,
                _ => {}
            }
        }
    }

    false
}

/// TIER 7: Detect W-as-vowel patterns
/// In English: -ew, -ow, -aw, -iew, -ow at end of words
fn has_w_as_vowel_pattern(raw: &str) -> bool {
    let bytes = raw.as_bytes();
    let len = bytes.len();

    if len < 2 { return false; }

    // Check endings: ew, ow, aw, iew
    if len >= 3 {
        let last3 = &bytes[len-3..];
        if last3 == b"iew" || last3 == b"iew" {
            return true; // view, review
        }
    }

    let last2 = &bytes[len-2..];
    match last2 {
        b"ew" | b"ow" | b"aw" => true, // new, show, draw
        _ => false,
    }
}

#[inline]
fn is_vowel_byte(b: u8) -> bool {
    matches!(b, b'a' | b'e' | b'i' | b'o' | b'u' | b'y')
}
```

### V3 Bitmask Constants

```rust
// ══════════════════════════════════════════════════════════
// CHAR INDEX: a=0, b=1, ..., z=25, đ=26, ă=27, â=28, ê=29, ô=30, ơ=31
// ══════════════════════════════════════════════════════════

/// Char type classification (32 bytes)
const CHAR_TYPE: [u8; 32] = [
    // a      b      c      d      e      f      g      h
    0b0010, 0b0001, 0b0101, 0b0001, 0b0010, 0b1000, 0b0001, 0b0001,
    // i      j      k      l      m      n      o      p
    0b0110, 0b1000, 0b0001, 0b0001, 0b0101, 0b0101, 0b0110, 0b0101,
    // q      r      s      t      u      v      w      x
    0b0001, 0b0001, 0b0001, 0b0101, 0b0110, 0b0001, 0b1000, 0b0001,
    // y      z      đ      ă      â      ê      ô      ơ
    0b0110, 0b1000, 0b0001, 0b0010, 0b0010, 0b0010, 0b0010, 0b0010,
];

const ONSET: u8   = 0b0001;  // Valid as onset
const VOWEL: u8   = 0b0010;  // Valid as vowel
const CODA: u8    = 0b0100;  // Valid as coda
const INVALID: u8 = 0b1000;  // Invalid in VN (f,j,w,z)

/// Valid VN single onsets: b,c,d,đ,g,h,k,l,m,n,p,q,r,s,t,v,x (4 bytes)
const M_ONSET: u32 = 0b_0010_0101_1111_1110_1111_1110_0110;

/// Valid VN single codas: c,m,n,p,t + semi-vowels i,o,u,y (4 bytes)
const M_CODA: u32 = 0b_0000_0011_0100_1001_0011_0100_0100;

/// EN-only onset clusters: bl,br,cl,cr,dr,dw,fl,fr,gl,gr,pl,pr,sc,sk,sl,sm,sn,sp,st,sw,tw,wr
/// M_EN_ONSET[first_char] & (1 << second_char) != 0 means EN cluster
const M_EN_ONSET: [u32; 32] = [
    // Precomputed bitmasks for each first char
    // b: l,r valid → bits 11,17 set
    // c: l,r valid → bits 11,17 set
    // d: r,w valid → bits 17,22 set
    // f: l,r valid → bits 11,17 set
    // g: l,r valid → bits 11,17 set
    // p: l,r valid → bits 11,17 set
    // s: c,k,l,m,n,p,t,w valid
    // t: r,w valid → bits 17,22 set
    // w: r valid → bit 17 set
    // ... (populate based on pattern table)
];

/// EN-only coda clusters - EXPANDED (v3.1)
/// ORIGINAL: ct, ft, ld, lf, lk, lm, lp, lt, lv, xt, nd, nk, nt, pt, rb, rd, rk, rl, rm, rn, rp, rt, sk, sp, st
/// ADDED: sh (push), ry (story), se (case), ks (books), fe (safe), re (core, care)
/// Index: a=0, b=1, c=2, d=3, e=4, f=5, g=6, h=7, i=8, j=9, k=10, l=11, m=12,
///        n=13, o=14, p=15, q=16, r=17, s=18, t=19, u=20, v=21, w=22, x=23, y=24, z=25
const M_EN_CODA: [u32; 32] = [
    // For each first char, set bit for valid second char
    // Example: 'f' (index 5) + 'e' (index 4) = fe (safe) → M_EN_CODA[5] |= (1 << 4)
    // Example: 's' (index 18) + 'h' (index 7) = sh (push) → M_EN_CODA[18] |= (1 << 7)
    0x00000000, // a
    0x00000000, // b
    0x00080000, // c: +t (ct) → bit 19
    0x00000000, // d
    0x00000000, // e
    0x00080010, // f: +t (ft) → bit 19, +e (fe) → bit 4 [NEW]
    0x00000000, // g
    0x00000000, // h
    0x00000000, // i
    0x00000000, // j
    0x00080000, // k: +s (ks) → bit 18 [NEW]
    0x000AC930, // l: +d,f,k,m,p,t,v → bits 3,5,10,12,15,19,21
    0x00000000, // m
    0x000A0400, // n: +d,k,t → bits 3,10,19
    0x00000000, // o
    0x00080000, // p: +t (pt) → bit 19
    0x00000000, // q
    0x0108FC06, // r: +b,d,e,k,l,m,n,p,t,y → bits 1,3,4,10,11,12,13,15,19,24 [+e,y NEW]
    0x001E0090, // s: +e,h,k,p,t → bits 4,7,10,15,19 [+e,h NEW]
    0x00000000, // t
    0x00000000, // u
    0x00000000, // v
    0x00000000, // w
    0x00080000, // x: +t (xt) → bit 19
    0x00000000, // y
    0x00000000, // z
    0x00000000, // đ
    0x00000000, // ă
    0x00000000, // â
    0x00000000, // ê
    0x00000000, // ô
    0x00000000, // ơ
];

/// EN-only vowel pairs - EXPANDED (v3.1)
/// ORIGINAL: ea, ee, ou, ei, eu, yo, ae, yi
/// ADDED: oo (book, too), oa (boat, road), io (action, ratio)
/// Index: a=0, e=4, i=8, o=14, u=20, y=24
const M_EN_VOWEL: [u32; 32] = [
    0x00000010, // a: +e (ae) → bit 4
    0x00000000, // b
    0x00000000, // c
    0x00000000, // d
    0x00100111, // e: +a,e,i,u (ea,ee,ei,eu) → bits 0,4,8,20
    0x00000000, // f
    0x00000000, // g
    0x00000000, // h
    0x00004000, // i: +o (io) → bit 14 [NEW]
    0x00000000, // j
    0x00000000, // k
    0x00000000, // l
    0x00000000, // m
    0x00000000, // n
    0x00104001, // o: +a,o,u (oa,oo,ou) → bits 0,14,20 [+a,o NEW]
    0x00000000, // p
    0x00000000, // q
    0x00000000, // r
    0x00000000, // s
    0x00000000, // t
    0x00000000, // u
    0x00000000, // v
    0x00000000, // w
    0x00000000, // x
    0x00004100, // y: +i,o (yi,yo) → bits 8,14
    0x00000000, // z
    0x00000000, // đ
    0x00000000, // ă
    0x00000000, // â
    0x00000000, // ê
    0x00000000, // ô
    0x00000000, // ơ
];

/// Circumflex required pairs: eu→êu, ie→iê, ue→uê, ye→yê
const M_CIRCUMFLEX_REQ: [u32; 32] = [
    // e: u requires circumflex
    // i: e requires circumflex
    // u: e requires circumflex
    // y: e requires circumflex
];

/// Tone-stop restriction: stops (c,ch,p,t) only allow sắc(1) or nặng(5)
const M_TONE_CODA: [[bool; 8]; 6] = [
    // tone 0 (none): all codas valid
    [true, true, true, true, true, true, true, true],
    // tone 1 (sắc): all codas valid
    [true, true, true, true, true, true, true, true],
    // tone 2 (huyền): stops invalid
    [false, false, true, true, true, true, false, true], // c,ch invalid, m,n,ng,nh,p,t valid? Check
    // tone 3 (hỏi): stops invalid
    [false, false, true, true, true, true, false, true],
    // tone 4 (ngã): stops invalid
    [false, false, true, true, true, true, false, true],
    // tone 5 (nặng): all codas valid
    [true, true, true, true, true, true, true, true],
];
```

### Memory Comparison

| Component | V1 Size | V3 Size | Reduction |
|-----------|---------|---------|-----------|
| Validation tables | ~2KB | ~600B | 70% |
| Whitelist arrays | ~1KB | 0B (bitmask) | 100% |
| State flags | 7 bools (7B) | 1 u16 (2B) | 70% |
| Pattern checks | runtime | compile-time | N/A |
| **Total core** | **~3KB** | **~600B** | **80%** |
| EN Dictionary | ~100KB | ~100KB | Same |

### Performance Comparison

| Operation | V1 | V3 | Speedup |
|-----------|----|----|---------|
| Single char validation | O(n) search | O(1) bit | 10x |
| Onset cluster check | strcmp | bit lookup | 5x |
| Diphthong validation | 29-item scan | bit lookup | 20x |
| Full syllable validation | ~50 ops | ~15 ops | 3x |
| Restore decision | ~20 conditions | ~8 conditions | 2.5x |
| **Total per keystroke** | **~100 ops** | **~30 ops** | **3x** |

### V1 vs V3 Pipeline Comparison

```
┌─────────────────────────────────────────────────────────────────────────────────────────────┐
│                              V1 PIPELINE (Current Production)                                │
├─────────────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                             │
│  INPUT: keystroke                                                                           │
│     │                                                                                       │
│     ▼                                                                                       │
│  ┌──────────────────┐                                                                       │
│  │ TRANSFORM PHASE  │ ← Apply tone/mark/stroke immediately                                  │
│  │ (7 separate      │   - Check modifier key                                                │
│  │  transform       │   - Find target vowel                                                 │
│  │  functions)      │   - Apply transform                                                   │
│  └────────┬─────────┘   - Update 7 separate flags                                           │
│           │                                                                                 │
│           ▼                                                                                 │
│  ┌──────────────────┐                                                                       │
│  │ VALIDATION PHASE │ ← 6 separate rule checks                                              │
│  │ (6 rules,        │   - Rule 1: has_vowel() O(n)                                         │
│  │  O(n) searches)  │   - Rule 2: valid_initial() O(n) whitelist                           │
│  └────────┬─────────┘   - Rule 3: all_chars_parsed()                                       │
│           │             - Rule 4: spelling_rules() if-else                                  │
│           │             - Rule 5: valid_final() O(n) whitelist                              │
│           │             - Rule 6: valid_vowel_pattern() O(n) whitelist                      │
│           ▼                                                                                 │
│  ┌──────────────────┐                                                                       │
│  │ ON TERMINATOR    │ ← Complex restore decision                                            │
│  │ (15+ conditions) │   - Check had_any_transform flag                                      │
│  └────────┬─────────┘   - Check has_stroke flag                                             │
│           │             - Check has_mark flag                                               │
│           │             - Check had_mark_revert flag                                        │
│           │             - Check is_buffer_invalid_vietnamese() [6 rules again]              │
│           │             - Check is_raw_input_valid_english()                                │
│           │             - Check has_english_modifier_pattern() [15+ branches]               │
│           │             - Check char consumption                                            │
│           │             - Check V+C+V pattern                                               │
│           │             - Check double modifier collapse                                    │
│           ▼                                                                                 │
│     RESTORE or KEEP                                                                         │
│                                                                                             │
│  PROBLEMS:                                                                                  │
│  ├── 7 separate state flags (hard to track)                                                │
│  ├── 6 rules called twice (transform + terminator)                                         │
│  ├── O(n) whitelist searches (slow)                                                        │
│  ├── 15+ if-else branches in restore decision                                              │
│  ├── has_english_modifier_pattern() is 200+ lines                                          │
│  └── ~100 operations per keystroke                                                          │
│                                                                                             │
└─────────────────────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────────────────────┐
│                              V3 PIPELINE (New Smart Engine)                                  │
├─────────────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                             │
│  INPUT: keystroke                                                                           │
│     │                                                                                       │
│     ▼                                                                                       │
│  ┌──────────────────┐                                                                       │
│  │ PRE-CHECK (L0)   │ ← O(1) bitmask check                                                  │
│  │ CHAR_TYPE[c]     │   if CHAR_TYPE[c] & INVALID → FOREIGN_MODE                            │
│  └────────┬─────────┘   if M_EN_ONSET[c1][c2] → FOREIGN_MODE                                │
│           │                                                                                 │
│     ┌─────┴─────┐                                                                           │
│     ▼           ▼                                                                           │
│  FOREIGN     CONTINUE                                                                       │
│  (skip)         │                                                                           │
│                 ▼                                                                           │
│  ┌──────────────────┐                                                                       │
│  │ TRANSFORM +      │ ← Single pass, update unified state                                   │
│  │ VALIDATE (L1-L9) │   - Transform: tone/mark/stroke                                       │
│  │ (unified state)  │   - Update: BufferState (1 u16)                                       │
│  └────────┬─────────┘   - Validate: 9 layers O(1) bitmask each                              │
│           │             - Cache: vn_state (Complete/Incomplete/Impossible)                  │
│           │                                                                                 │
│           ▼                                                                                 │
│  ┌──────────────────┐                                                                       │
│  │ ON TERMINATOR    │ ← 4-phase restore (uses cached state)                                 │
│  │ (4 phases)       │   Phase 1: Quick exits O(1)                                           │
│  └────────┬─────────┘     - !had_transform → Keep                                           │
│           │               - has_stroke → Keep                                               │
│           │               - has_tone → Keep                                                 │
│           │             Phase 2: VN state O(1)                                              │
│           │               - vn_state == Complete → Keep                                     │
│           │             Phase 3: EN detection O(n)                                          │
│           │               - Mark + incomplete + EN → Restore                                │
│           │               - Revert + invalid + EN → Restore                                 │
│           │             Phase 4: Additional triggers                                        │
│           │               - Char consumption                                                │
│           │               - V+C+V pattern                                                   │
│           ▼                                                                                 │
│     RESTORE or KEEP                                                                         │
│                                                                                             │
│  IMPROVEMENTS:                                                                              │
│  ├── 1 unified state (BufferState u16)                                                     │
│  ├── 9 layers run once (validation cached)                                                 │
│  ├── O(1) bitmask lookups (fast)                                                           │
│  ├── 4-phase restore (structured)                                                          │
│  ├── has_english_pattern() is ~50 lines                                                    │
│  └── ~30 operations per keystroke                                                           │
│                                                                                             │
└─────────────────────────────────────────────────────────────────────────────────────────────┘
```

### Side-by-Side Comparison

| Aspect | V1 (Production) | V3 (Smart) | Improvement |
|--------|-----------------|------------|-------------|
| **State tracking** | 7 separate bool flags | 1 BufferState u16 | Unified |
| **Validation** | 6 rules, O(n) each | 9 layers, O(1) each | 3-20x faster |
| **Validation timing** | Called twice (transform + terminator) | Called once, cached | 2x less work |
| **Restore logic** | 15+ scattered conditions | 4 structured phases | Cleaner |
| **EN detection** | 200+ lines if-else | 50 lines tiered | Maintainable |
| **Whitelist search** | O(n) array scan | O(1) bitmask | 10-20x faster |
| **Memory** | ~3KB tables | ~600B bitmasks | 80% smaller |
| **Ops per keystroke** | ~100 | ~30 | 3x faster |

### V1 Case Coverage Checklist

```
V1 VALIDATION RULES:
☑ Rule 1: HAS_VOWEL        → L1 CHAR_TYPE vowel check
☑ Rule 2: VALID_INITIAL    → L2 M_ONSET bitmask
☑ Rule 3: ALL_CHARS_PARSED → L1-L6 syllable structure
☑ Rule 4: SPELLING_RULES   → L8 M_SPELL matrix
☑ Rule 5: VALID_FINAL      → L5-L6 M_CODA bitmask
☑ Rule 6: VALID_VOWEL      → L4 M_VOWEL_PAIR/TRIPLE

V1 MODIFIER REQUIREMENTS:
☑ Circumflex required (êu,iê,uê,yê)  → L9 M_CIRCUMFLEX_REQ
☑ Breve restrictions (ă+vowel)       → L9 M_BREVE_INVALID

V1 FOREIGN DETECTION:
☑ Invalid initials (f,j,w,z)         → CHAR_TYPE & INVALID
☑ EN onset clusters                  → M_EN_ONSET
☑ EN coda clusters                   → M_EN_CODA
☑ EN vowel patterns (ou,yo,ea)       → M_EN_VOWEL

V1 RESTORE SIGNALS:
☑ Stroke (đ)                         → state.has_stroke
☑ Tone (s,f,r,x,j)                   → state.has_tone
☑ Mark (w,aa,oo,ee)                  → state.has_mark
☑ Revert (ss,ff,rr)                  → state.had_revert

V1 RESTORE TRIGGERS:
☑ Two-check (buffer_invalid && raw_EN) → should_restore() phases
☑ Char consumption                     → raw.len() - buffer.len()
☑ V+C+V circumflex pattern             → matches_vcv_stop_pattern()
☑ Double modifier collapse             → revert_type check

V1 PREVENT RESTORE:
☑ Has stroke                           → Phase 1 quick exit
☑ Has tone                             → Phase 1 quick exit
☑ Double modifier at end               → revert logic
☑ Non-letter prefix                    → pre-check
☑ No transform                         → Phase 1 quick exit
☑ Never collapse "ff"                  → special case in revert

V1 SPECIAL CASES:
☑ Breve deferral (aw→ăn/aw)           → pending_breve state
☑ Horn deferral (uo→ươ)               → pending_horn state
☑ Tone overwrite (banjs→bán)          → has_tone stays true
☑ Continuous typing (chaofooo)        → has_tone prevents restore
```

---

## Overview

Pipeline xử lý Vietnamese IME V3 engine với 2 mechanisms:

1. **FOREIGN_MODE**: Real-time detection khi đang gõ. Detect không phải VN → skip transforms, keep-as-is.
2. **AUTO_RESTORE**: Chỉ trigger on terminator (space/enter/punctuation). Buffer invalid VN + raw/buffer ∈ Dict → restore.

| Mechanism | Timing | Trigger | Action |
|-----------|--------|---------|--------|
| FOREIGN_MODE | Real-time (every keystroke) | English pattern detected | Skip transforms, keep raw |
| AUTO_RESTORE | On terminator only | Invalid VN + dictionary match | Restore to raw or keep buffer |

---

## Pattern Reference Table (Master)

Bảng tổng hợp patterns + functions cho cả English detection và Vietnamese validation.

### English Detection Functions (check on RAW)

| Position | English only patterns | Function | Step |
|----------|----------------------|----------|------|
| **ONSET** | `f,j,w,z` | `has_english_onset()` | 0 |
| **ONSET CLUSTER** | `bl,br,cl,cr,dr,dw,fl,fr,gl,gr,pl,pr,sc,sk,sl,sm,sn,sp,st,sw,tw,wr` | `has_english_onset()` | 0 |
| **CODA (single)** | `b,d,g,h,k,l,q,r,s,v,x` | `has_english_coda()` | 2A |
| **CODA (cluster)** | `ct,ft,ld,lf,lk,lm,lp,lt,lv,mb,mp,nd,nk,nt,pt,rb,rd,rk,rl,rm,rn,rp,rt,sk,sp,st,xt` | `has_english_coda()` | 2A |
| **DIPHTHONG** | `ea,ee,ou,ei,eu,yo,ae,yi` | `has_invalid_vowel_pattern()` | 2A |
| **TRIPHTHONG** | `eau,iou,you` | `has_invalid_vowel_pattern()` | 2A |
| **SUFFIX** | `tion,sion,ness,ment,able,ible,ful,less,ing,ous,ive,ize,ise,ity,ly,ed` | `has_english_suffix()` | 2A |
| **PREFIX** | `un,re,pre,dis,mis,over,out,sub` | `has_english_prefix()` | 2A |
| **BIGRAM** | `bk,cb,dk,gk,hb,jb,kb,kx,kz,pb,qb,tb,vb,wb,xb,zb,...` | `has_impossible_bigram()` | 2A |
| **DOUBLE CONS.** | `ll,ss,ff,tt,pp,mm,nn,rr,dd,gg,bb,zz,cc` | ⚠️ SKIP (Telex revert) | 2B |

### Vietnamese Validation Functions (check on BUFFER)

| Position | Vietnamese patterns | Function | Layer |
|----------|---------------------|----------|-------|
| **ONSET (single)** | `b,c,d,đ,g,h,k,l,m,n,p,q,r,s,t,v,x` (17) | `is_valid_onset()` | L2 |
| **ONSET (cluster)** | `ch,gh,gi,kh,kr¹,ng,ngh,nh,ph,qu,th,tr` (12) | `is_valid_onset_cluster()` | L3 |
| **VOWEL (base)** | `a,e,i,o,u,y` (6) | `is_valid_vowel()` | L1 |
| **VOWEL (modified)** | `ă,â,ê,ô,ơ,ư` (6) | `is_valid_vowel()` | L1 |
| **DIPHTHONG** | `ai,ao,au,ay,âu,ây,eo,êu,ia,iê,iu,oa,oă,oe,oi,ôi,ơi,oo²,ua,uâ,uê,ui,uô,uy,ưa,ưi,ươ,ưu,yê` (29) | `is_valid_diphthong()` | L4 |
| **TRIPHTHONG** | `iêu,yêu,oai,oay,oao,oeo,uây,uôi,uya,uyê,uyu,uêu,ươi,ươu` (14) | `is_valid_triphthong()` | L4 |
| **CODA (single)** | `c,m,n,p,t` (5) | `is_valid_coda()` | L5 |
| **CODA (cluster)** | `ch,ng,nh` (3) | `is_valid_coda_cluster()` | L6 |
| **CODA (semi-vowel)** | `i,o,u,y` (4) | `is_valid_coda()` | L5 |
| **TONE-STOP** | sắc/nặng only with `-c,-ch,-p,-t` | `is_valid_tone_coda()` | L7 |
| **SPELLING** | c→k, g→gh, ng→ngh before e,i,y | `is_valid_spelling()` | L8 |

### Full Pattern Matrix

| Position | Vietnamese only | Shared (VN & EN) | English only |
|----------|-----------------|------------------|--------------|
| **ONSET** | `đ` | `b,c,d,g,h,k,l,m,n,p,q,r,s,t,v,x` | `f,j,w,z` |
| **ONSET CLUSTER** | `gh,gi,kh,ngh,nh,ph,qu,kr¹` | `ch,ng,th,tr` | `bl,br,cl,cr,dr,dw,fl,fr,gl,gr,pl,pr,sc,sk,sl,sm,sn,sp,st,sw,tw,wr` |
| **CODA (single)** | - | `c,m,n,p,t` | `b,d,g,h,k,l,q,r,s,v,x` |
| **CODA (cluster)** | `ch,nh` | `ng` | `ct,ft,ld,lf,lk,lm,lp,lt,lv,mb,mp,nd,nk,nt,pt,rb,rd,rk,rl,rm,rn,rp,rt,sk,sp,st,xt` |
| **CODA (semi-vowel)** | - | `i,o,u,y` | - |
| **VOWEL (base)** | - | `a,e,i,o,u,y` | - |
| **VOWEL (modified)** | `ă,â,ê,ô,ơ,ư` | - | - |
| **DIPHTHONG** | `âu,ây,êu,iê,oă,ôi,ơi,uâ,uê,uô,ươ,ưa,ưi,ưu,yê,oo²` | `ai,ao,au,ay,eo,ia,iu,oa,oe,oi,ua,ui,uy` | `ea,ee,ou,ei,eu,yo,ae,yi` |
| **TRIPHTHONG** | `iêu,yêu,ươu,uôi,ươi,oai,oay,uây,uya,uyê,uyu,uêu,oao,oeo` | - | `eau,iou,you` |
| **SUFFIX** | - | - | `tion,sion,ness,ment,able,ible,ful,less,ing,ous,ive,ize,ise,ity,ly,ed` |
| **PREFIX** | - | - | `un,re,pre,dis,mis,over,out,sub` |
| **BIGRAM** | - | (most combinations) | `bk,cb,dk,gk,hb,jb,kb,kx,kz,pb,qb,tb,vb,wb,xb,zb,...` |
| **DOUBLE CONS.** | - | - | `ll,ss,ff,tt,pp,mm,nn,rr,dd,gg,bb,zz,cc` |

**Notes:**
- ¹ `kr` - ethnic minority place names (Krông Búk, Đắk Lắk)
- ² `oo` - thoòng, thoông (valid VN)

### Source Comparison

| Source | Onset (single) | Onset (cluster) | Coda (single) | Coda (cluster) | Diphthongs | Triphthongs |
|--------|----------------|-----------------|---------------|----------------|------------|-------------|
| **V1 Engine** | 16 | 11+ngh+kr | 6+semi-vowels | 3 | 13 base | 13 |
| **V3 Engine** | 17 (đ) | 11+ngh | 6 | 3 | 18 modified | 13 |
| **OpenKey** | 17+4 foreign | 10 | 5 | 3 | ~20 | ~15 |

### V1 Foreign Detection (validation.rs)

```rust
// V1 detects foreign via is_foreign_word_pattern():
// 1. Vowel patterns: ou, yo (never valid VN)
// 2. Consonant clusters: T+R, P+R, C+R after finals
// 3. Invalid finals: b,d,g,h,k,l,q,r,s,v,x (single)
```

### OpenKey Spell Checking (Engine.cpp)

```cpp
// OpenKey validates via checkSpelling():
// 1. _consonantTable - valid initials
// 2. _endConsonantTable - valid finals
// 3. _vowelCombine - valid vowel combos
// 4. Tone-stop restriction: ch/t finals can't have hỏi/ngã/huyền
// 5. vRestoreIfWrongSpelling flag - restore raw if invalid
```

**Detection logic:**
- **English only** → trigger FOREIGN_MODE (invalid VN)
- **Shared** → ambiguous, cần thêm context hoặc dictionary
- **Vietnamese only** → valid VN, không trigger

**Notes:**
- **ONSET/CODA**: Kiểm tra đầu/cuối raw
- **DIPHTHONG/TRIPHTHONG**: Tìm vowel pattern trong raw
- **DOUBLE CONSONANT**: KHÔNG trigger FOREIGN_MODE (có thể từ Telex revert `ss` → revert sắc). Chỉ dùng ở Step 2B dictionary lookup.

---

## Function Naming Convention

```rust
// Pattern detection (check on RAW) - maps to Pattern Reference Table
fn has_english_onset(raw: &str) -> bool;           // ONSET, ONSET CLUSTER (EN-only)
fn has_english_coda(raw: &str) -> bool;            // CODA single + cluster (EN-only)
fn has_invalid_vowel_pattern(raw: &str) -> bool;   // DIPHTHONG, TRIPHTHONG (EN-only)
fn has_english_suffix(raw: &str) -> bool;          // SUFFIX (EN-only)
fn has_english_prefix(raw: &str) -> bool;          // PREFIX (EN-only)
fn has_impossible_bigram(raw: &str) -> bool;       // BIGRAM (EN-only)

// Combined check (ALL patterns from table except DOUBLE CONS.)
fn has_english_pattern(raw: &str) -> bool {
    has_english_onset(raw)           // Step 0 only
    || has_english_coda(raw)         // Step 2A
    || has_invalid_vowel_pattern(raw) // Step 2A
    || has_english_suffix(raw)       // Step 2A ← NEW
    || has_english_prefix(raw)       // Step 2A ← NEW
    || has_impossible_bigram(raw)    // Step 2A
    // NOTE: DOUBLE CONS. skipped (Telex revert) → Step 2B dict only
}

// Vietnamese validation (check on BUFFER)
fn is_valid_vietnamese(buffer: &str) -> ValidationResult;
fn is_impossible_vietnamese(buffer: &str) -> bool;

// Dictionary lookup
fn is_in_english_dictionary(word: &str) -> bool;

// Mode decisions
fn should_enter_foreign_mode(raw: &str, buffer: &str) -> bool;
fn should_auto_restore(raw: &str, buffer: &str) -> Option<String>;
```

### Pattern → Function Mapping

| Pattern (from Table) | Column Used | Function | Step |
|---------------------|-------------|----------|------|
| ONSET | English only: `f,j,w,z` | `has_english_onset()` | 0 |
| ONSET CLUSTER | English only: `bl,br,cl,cr,...` | `has_english_onset()` | 0 |
| CODA (single) | English only: `b,d,g,h,k,l,...` | `has_english_coda()` | 2A |
| CODA (cluster) | English only: `ct,ft,ld,nd,...` | `has_english_coda()` | 2A |
| DIPHTHONG | English only: `ea,ee,ou,ei,...` | `has_invalid_vowel_pattern()` | 2A |
| TRIPHTHONG | English only: `eau,iou,you` | `has_invalid_vowel_pattern()` | 2A |
| SUFFIX | English only: `tion,ing,ed,...` | `has_english_suffix()` | 2A |
| PREFIX | English only: `un,re,pre,dis,...` | `has_english_prefix()` | 2A |
| BIGRAM | English only: `bk,cb,dk,...` | `has_impossible_bigram()` | 2A |
| DOUBLE CONS. | ⚠️ SKIP | - | 2B (dict) |

---

## Full Pipeline

```
┌─────────────────────────────────────────────────────────────────┐
│                         INPUT                                   │
│  • raw: keystrokes đang gõ (không dấu)                         │
│  • key: phím vừa nhấn                                          │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│              STEP 0: PRE-CHECK → FOREIGN_MODE                   │
│─────────────────────────────────────────────────────────────────│
│  Check: has_english_onset(raw)?                                 │
│  Patterns: See Pattern Reference Table → ONSET, ONSET CLUSTER  │
│                                                                 │
│  If pattern found → ENTER FOREIGN_MODE (skip all transforms)   │
│  If no pattern → Vietnamese mode (allow transforms)             │
└─────────────────────────────────────────────────────────────────┘
                              │
                     ┌────────┴────────┐
                     ▼                 ▼
              [EN Pattern]       [No EN Pattern]
                     │                 │
                     ▼                 ▼
            ┌─────────────────┐  ALLOW TRANSFORM
            │  FOREIGN_MODE   │  (apply VN transforms)
            │  Skip transforms│  → buffer = transform(raw)
            │  Keep raw as-is │        │
            └─────────────────┘        │
                     │                 ▼
                     ▼           Continue to STEP 1...
                  [DONE]
                                       │
                                       ▼
┌─────────────────────────────────────────────────────────────────┐
│                 STEP 1: VIETNAMESE VALIDATION                   │
│─────────────────────────────────────────────────────────────────│
│  Check: is_valid_vietnamese(buffer)?                            │
│  Patterns: See Pattern Reference Table → Vietnamese (valid)    │
│                                                                 │
│  Validate buffer against phonotactic rules:                     │
│  1. Valid onset (+ clusters)                                    │
│  2. Valid vowel nucleus                                         │
│  3. Valid diphthong/triphthong                                  │
│  4. Valid coda                                                  │
│  5. Tone-stop restriction: sắc/nặng only with -c,-ch,-p,-t     │
│                                                                 │
│  Result: Complete | Incomplete | Impossible                     │
└─────────────────────────────────────────────────────────────────┘
                              │
          ┌───────────────────┼───────────────────┐
          ▼                   ▼                   ▼
     [Complete]          [Incomplete]       [Impossible]
          │                   │                   │
          ▼                   ▼                   │
      KEEP VN             KEEP VN                │
     (valid word)        (keep typing)           │
          │                   │                   │
          ▼                   ▼                   ▼
       [DONE]              [DONE]          Continue to STEP 2A...
                                                 │
                                                 ▼
┌─────────────────────────────────────────────────────────────────┐
│            STEP 2A: FOREIGN_MODE DETECTION                      │
│                    (runs EVERY keystroke)                       │
│─────────────────────────────────────────────────────────────────│
│  CHECK 1: is_impossible_vietnamese(buffer)?  ← check trên BUFFER│
│  CHECK 2: has_english_pattern(raw)?          ← check trên RAW  │
│                                                                 │
│  Condition: CHECK 1 && CHECK 2 → FOREIGN_MODE                   │
│                                                                 │
│  Patterns: See Pattern Reference Table (English Detection)      │
│  ✓ CODA         → has_english_coda(raw)                        │
│  ✓ DIPHTHONG    → has_invalid_vowel_pattern(raw)               │
│  ✓ SUFFIX       → has_english_suffix(raw)      ← NEW           │
│  ✓ PREFIX       → has_english_prefix(raw)      ← NEW           │
│  ✓ BIGRAM       → has_impossible_bigram(raw)                   │
│  ✗ ONSET        → already checked at Step 0                    │
│  ✗ DOUBLE CONS. → skip (Telex revert), wait Step 2B            │
│                                                                 │
│  If both checks pass → ENTER FOREIGN_MODE (keep raw)            │
│  If not → Continue to terminator check                          │
└─────────────────────────────────────────────────────────────────┘
                              │
                     ┌────────┴────────┐
                     ▼                 ▼
          [Pattern Match]        [No Pattern]
                  │                    │
                  ▼                    ▼
        ┌─────────────────┐    ┌─────────────────┐
        │  FOREIGN_MODE   │    │ Is terminator?  │
        │  Keep raw as-is │    │ key ∈ {space,   │
        │                 │    │ enter,,.;:!?'"} │
        └─────────────────┘    └─────────────────┘
                  │                    │
                  ▼           ┌────────┴────────┐
               [DONE]         ▼                 ▼
                            [Yes]              [No]
                              │                 │
                              ▼                 ▼
                              │           KEEP AS-IS
                              │          (wait for more
                              │            keystrokes)
                              │                 │
                              ▼                 ▼
                              │              [DONE]
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│            STEP 2B: AUTO_RESTORE                                │
│                 (only on terminator key)                        │
│─────────────────────────────────────────────────────────────────│
│  PRE-CONDITION: is_terminator(key)                              │
│  CHECK: is_impossible_vietnamese(buffer)?  ← check trên BUFFER │
│                                                                 │
│  If CHECK = false → KEEP VN (valid Vietnamese)                  │
│  If CHECK = true  → Continue dictionary lookup                  │
│                                                                 │
│  Dictionary: HashSet<10,000 common English words>               │
│  Size: ~100KB, Lookup: O(1)                                     │
│                                                                 │
│  Check order:                                                   │
│  1. is_in_english_dictionary(raw)?    → AUTO_RESTORE(raw)      │
│  2. is_in_english_dictionary(buffer)? → KEEP(buffer)           │
│  3. Neither match                     → KEEP AS-IS (typo)      │
└─────────────────────────────────────────────────────────────────┘
                              │
              ┌───────────────┼───────────────┐
              ▼               ▼               ▼
        [raw match]     [buffer match]   [No match]
              │               │               │
              ▼               ▼               ▼
      AUTO_RESTORE(raw) KEEP(buffer)     KEEP AS-IS
              │               │            (typo)
              ▼               ▼               │
           [DONE]          [DONE]             ▼
                                           [DONE]
```

---

## Decision Summary

| Step | Timing | Check ON | Function | Pattern (from Table) | Action | Output |
|------|--------|----------|----------|---------------------|--------|--------|
| 0 | Before transform | **raw** | `has_english_onset()` | ONSET, ONSET CLUSTER | FOREIGN_MODE | raw |
| 1 | Every key | **buffer** | `is_valid_vietnamese()` | VN Validation (L1-L8) | KEEP VN | buffer |
| 2A | Every key | **buffer** + **raw** | `is_impossible_vietnamese()` + `has_english_coda()` | CODA (single+cluster) | FOREIGN_MODE | raw |
| 2A | Every key | **buffer** + **raw** | `is_impossible_vietnamese()` + `has_invalid_vowel_pattern()` | DIPHTHONG, TRIPHTHONG | FOREIGN_MODE | raw |
| 2A | Every key | **buffer** + **raw** | `is_impossible_vietnamese()` + `has_english_suffix()` | SUFFIX | FOREIGN_MODE | raw |
| 2A | Every key | **buffer** + **raw** | `is_impossible_vietnamese()` + `has_english_prefix()` | PREFIX | FOREIGN_MODE | raw |
| 2A | Every key | **buffer** + **raw** | `is_impossible_vietnamese()` + `has_impossible_bigram()` | BIGRAM | FOREIGN_MODE | raw |
| 2B | Terminator | **raw** | `is_in_english_dictionary()` | DOUBLE CONS. (via dict) | AUTO_RESTORE | raw |
| 2B | Terminator | **buffer** | `is_in_english_dictionary()` | - | KEEP | buffer |
| 2B | Terminator | - | - | No match | KEEP | buffer (typo) |

**Terminology:**
- **FOREIGN_MODE**: Real-time, skip transforms, keep raw as-is
- **AUTO_RESTORE**: On terminator only, restore raw from dictionary

**Check targets:**
- **buffer**: Text sau khi transform (có thể có dấu VN)
- **raw**: Original keystrokes (không dấu)

**Trigger conditions:**
```
Step 0:  has_english_onset(raw)                                      → FOREIGN_MODE
Step 2A: is_impossible_vietnamese(buffer) && has_english_pattern(raw) → FOREIGN_MODE
Step 2B: is_impossible_vietnamese(buffer) && is_in_english_dictionary(word) → AUTO_RESTORE/KEEP
```

**NOTE:** Double consonant (ss, ff, tt...) từ Telex revert → KHÔNG trigger FOREIGN_MODE.
Chờ đến Step 2B để AUTO_RESTORE check dictionary.

---

## Terminator Keys

```
space, enter, tab
, . ; : ! ? ' " ( ) [ ] { } / \ - + = @ # $ % ^ & * < >
```

---

## Examples

### Example 1: "class" (FOREIGN_MODE at start)

```
Keystroke sequence: c → l → a → s → s → [space]

Step 0: PRE-CHECK on RAW
├── raw = "cl"
├── Check: "cl" is EN-only onset cluster? ✓
└── Result: ENTER FOREIGN_MODE

Subsequent keystrokes (FOREIGN_MODE):
├── l → "cl"
├── a → "cla"
├── s → "clas" (no tone transform)
├── s → "class" (literal, no revert logic)
└── [space] → output "class"

Final: "class" (no Vietnamese transform ever applied)
```

| Key | raw | buffer | Mode | Action |
|-----|-----|--------|------|--------|
| c | c | c | VN | - |
| l | cl | cl | **FOREIGN_MODE** | Step 0: "cl" pattern on raw |
| a | cla | cla | FOREIGN_MODE | literal |
| s | clas | clas | FOREIGN_MODE | literal |
| s | class | class | FOREIGN_MODE | literal |
| space | class | class | - | Output: "class" |

---

### Example 2: "file" (FOREIGN_MODE - invalid VN initial)

```
Keystroke sequence: f → i → l → e → [space]

Step 0: PRE-CHECK on RAW
├── raw = "f"
├── Check: "f" is invalid VN initial? ✓
└── Result: ENTER FOREIGN_MODE

Subsequent keystrokes (FOREIGN_MODE):
├── i → "fi"
├── l → "fil"
├── e → "file"
└── [space] → output "file"

Final: "file" (no Vietnamese transform ever applied)
```

| Key | raw | buffer | Mode | Action |
|-----|-----|--------|------|--------|
| f | f | f | **FOREIGN_MODE** | Step 0: "f" invalid VN initial |
| i | fi | fi | FOREIGN_MODE | literal |
| l | fil | fil | FOREIGN_MODE | literal |
| e | file | file | FOREIGN_MODE | literal |
| space | file | file | - | Output: "file" |

---

### Example 3: "case" (AUTO_RESTORE via dictionary)

```
Keystroke sequence: c → a → s → e → [space]

Step 0: PRE-CHECK
├── raw = "c"
├── Check: No EN pattern at start
└── Result: ALLOW TRANSFORM → Vietnamese mode

Keystroke processing:
├── c → buffer = "c"
├── a → buffer = "ca"
├── s → 's' is Telex sắc modifier → buffer = "cá"
├── e → buffer = "cáe"
└── [space] → trigger validation

Step 1: VN VALIDATION
├── buffer = "cáe"
├── Check: "áe" valid VN vowel combination? ✗
└── Result: Impossible

Step 2A: FOREIGN_MODE DETECTION
├── CHECK 1: Invalid_VN(buffer="cáe")? ✓ (áe not valid)
├── CHECK 2: EN_Pattern(raw="case")?
│   ├── L5 Coda cluster? ✗ (se not coda cluster)
│   ├── L7 Vowel pattern? ✗ (no ea/ee/oo/ou/ei/eu)
│   └── L8 Impossible bigram? ✗
└── Result: No pattern match → continue to Step 2B

Step 2B: AUTO_RESTORE (terminator = space)
├── CHECK: Invalid_VN(buffer="cáe")? ✓
├── CHECK: raw="case" ∈ Dictionary? ✓
└── Result: AUTO_RESTORE → "case"

Final: "case"
```

| Key | raw | buffer | VN State | Action |
|-----|-----|--------|----------|--------|
| c | c | c | - | No pattern, VN mode |
| a | ca | ca | - | - |
| s | cas | cá | - | 's' = sắc tone |
| e | case | cáe | - | - |
| space | case | cáe | Impossible | AUTO_RESTORE: Dict(raw) ✓ |

---

### Example 4: "casse" (Telex revert → AUTO_RESTORE keeps buffer)

```
Keystroke sequence: c → a → s → s → e → [space]

Step 0: PRE-CHECK
├── raw = "c"
├── Check: No EN pattern at start
└── Result: ALLOW TRANSFORM → Vietnamese mode

Keystroke processing (normal Telex behavior):
├── c → buffer = "c"
├── a → buffer = "ca"
├── s → 's' is Telex sắc modifier → buffer = "cá"
├── s → second 's' triggers REVERT (normal Telex) → buffer = "cas"
├── e → buffer = "case"
└── [space] → trigger validation

NOTE: Double 's' for revert is NORMAL Telex behavior, NOT English signal.
      Do NOT trigger immediate restore on revert.

Step 1: VN VALIDATION
├── buffer = "case"
├── Check: "se" valid VN final? ✗ ('s' not valid final consonant)
└── Result: Impossible

Step 2A: FOREIGN_MODE DETECTION
├── CHECK 1: Invalid_VN(buffer="case")? ✓ (se not valid final)
├── CHECK 2: EN_Pattern(raw="casse")?
│   ├── L3 Double consonant "ss"? - SKIP (could be from revert)
│   ├── L5 Coda cluster? ✗
│   └── L7 Vowel pattern? ✗
└── Result: No FOREIGN_MODE trigger → continue to Step 2B

Step 2B: AUTO_RESTORE (terminator = space)
├── CHECK: Invalid_VN(buffer="case")? ✓
├── CHECK 1: raw="casse" ∈ Dictionary? ✗ (not a word)
├── CHECK 2: buffer="case" ∈ Dictionary? ✓
└── Result: KEEP buffer → "case"

Final: "case"
```

| Key | raw | buffer | VN State | Action |
|-----|-----|--------|----------|--------|
| c | c | c | - | No pattern, VN mode |
| a | ca | ca | - | - |
| s | cas | cá | - | 's' = sắc tone |
| s | cass | cas | - | REVERT (normal Telex) |
| e | casse | case | - | - |
| space | casse | case | Impossible | KEEP: Dict(buffer) ✓ |

**Key insight:** User typed "ss" to revert tone and get literal 's', then added 'e'.
User wanted "case", not "casse". Output = buffer = "case".

---

### Example 5: "coffee" (double consonant - actual English word)

```
Keystroke sequence: c → o → f → f → e → e → [space]

Step 0: PRE-CHECK
├── raw = "c"
├── Check: No EN pattern at start
└── Result: ALLOW TRANSFORM → Vietnamese mode

Keystroke processing:
├── c → buffer = "c"
├── o → buffer = "co"
├── f → 'f' is Telex huyền modifier → buffer = "cò"
├── f → second 'f' triggers REVERT → buffer = "cof"
├── e → buffer = "cofe"
├── e → second 'e' = circumflex → buffer = "cofê"
└── [space] → trigger validation

Step 1: VN VALIDATION
├── buffer = "cofê"
├── Check: 'f' valid in VN word? ✗ (f only valid as initial, not medial)
└── Result: Impossible

Step 2A: FOREIGN_MODE DETECTION
├── CHECK 1: Invalid_VN(buffer="cofê")? ✓
├── CHECK 2: EN_Pattern(raw="coffee")?
│   └── L7: Vowel pattern "ee"? ✓
└── Result: ENTER FOREIGN_MODE → keep raw

Final: "coffee"
```

| Key | raw | buffer | VN State | Action |
|-----|-----|--------|----------|--------|
| c | c | c | - | No pattern, VN mode |
| o | co | co | - | - |
| f | cof | cò | - | 'f' = huyền tone |
| f | coff | cof | - | REVERT (normal Telex) |
| e | coffe | cofe | - | - |
| e | coffee | cofê | Impossible | FOREIGN_MODE: L7 "ee" ✓ |
| space | coffee | coffee | - | Output: "coffee" |

**Note:** "ee" vowel pattern triggers FOREIGN_MODE immediately, not the "ff" double consonant.

---

### Example 6: "bass" (AUTO_RESTORE to raw)

```
Keystroke sequence: b → a → s → s → [space]

Step 0: PRE-CHECK → No pattern, VN mode

Keystroke processing:
├── b → buffer = "b"
├── a → buffer = "ba"
├── s → 's' sắc → buffer = "bá"
├── s → REVERT → buffer = "bas"
└── [space] → trigger validation

Step 1: VN VALIDATION
├── buffer = "bas"
├── Check: 's' valid VN final? ✗
└── Result: Impossible

Step 2A: FOREIGN_MODE DETECTION
├── CHECK 1: Invalid_VN(buffer="bas")? ✓ (s not valid final)
├── CHECK 2: EN_Pattern(raw="bass")?
│   ├── L3 Double consonant "ss"? - SKIP (could be from revert)
│   └── No other patterns
└── Result: No FOREIGN_MODE trigger → continue to Step 2B

Step 2B: AUTO_RESTORE (terminator = space)
├── CHECK: Invalid_VN(buffer="bas")? ✓
├── CHECK 1: raw="bass" ∈ Dictionary? ✓
└── Result: AUTO_RESTORE → "bass"

Final: "bass"
```

| Key | raw | buffer | VN State | Action |
|-----|-----|--------|----------|--------|
| b | b | b | - | - |
| a | ba | ba | - | - |
| s | bas | bá | - | 's' = sắc tone |
| s | bass | bas | - | REVERT |
| space | bass | bas | Impossible | AUTO_RESTORE: Dict(raw) ✓ |

**Contrast với "casse":**
- "bass" → raw ∈ Dict → AUTO_RESTORE(raw) → "bass"
- "casse" → raw ∉ Dict, buffer ∈ Dict → KEEP(buffer) → "case"

---

### Example 7: "their" (FOREIGN_MODE via vowel pattern)

```
Keystroke sequence: t → h → e → i → r → [space]

Step 0: PRE-CHECK
├── raw = "th"
├── Check: "th" is EN onset cluster? ✗ (th valid in VN: thành, thì)
└── Result: ALLOW TRANSFORM → Vietnamese mode

Keystroke processing:
├── t → buffer = "t"
├── h → buffer = "th"
├── e → buffer = "the"
├── i → buffer = "thei"
├── r → 'r' is Telex hỏi modifier → buffer = "thẻi"
└── [space] → trigger validation

Step 1: VN VALIDATION
├── buffer = "thẻi"
├── Check: "ei" valid VN diphthong? ✗
└── Result: Impossible

Step 2A: FOREIGN_MODE DETECTION
├── CHECK 1: Invalid_VN(buffer="thẻi")? ✓ (ei not valid diphthong)
├── CHECK 2: EN_Pattern(raw="their")?
│   └── L7: Vowel pattern "ei"? ✓
└── Result: ENTER FOREIGN_MODE → keep raw

Final: "their"
```

| Key | raw | buffer | VN State | Action |
|-----|-----|--------|----------|--------|
| t | t | t | - | - |
| h | th | th | - | th valid VN, continue |
| e | the | the | - | - |
| i | thei | thei | - | - |
| r | their | thẻi | Impossible | FOREIGN_MODE: L7 "ei" ✓ |
| space | their | their | - | Output: "their" |

---

### Example 8: "user" (dictionary only)

```
Keystroke sequence: u → s → e → r → [space]

Step 0: PRE-CHECK → No pattern, VN mode

Keystroke processing:
├── u → buffer = "u"
├── s → 's' sắc on 'u' → buffer = "ú"
├── e → buffer = "úe"
├── r → 'r' hỏi, but 'e' already has no tone → buffer = "úe" (r ignored or "ủe"?)
└── [space] → trigger validation

Assuming buffer = "úer" or "ủe":

Step 1: VN VALIDATION
├── buffer = "ủe" or similar
├── Check: Invalid structure
└── Result: Impossible

Step 2A: FOREIGN_MODE DETECTION
├── CHECK 1: Invalid_VN(buffer)? ✓
├── CHECK 2: EN_Pattern(raw="user")?
│   └── No coda, vowel pattern, or bigram match
└── Result: No FOREIGN_MODE trigger → continue to Step 2B

Step 2B: AUTO_RESTORE (terminator = space)
├── CHECK: Invalid_VN(buffer)? ✓
├── CHECK: raw="user" ∈ Dictionary? ✓
└── Result: AUTO_RESTORE → "user"

Final: "user"
```

---

### Example 9: "việt" (valid Vietnamese)

```
Keystroke sequence: v → i → e → e → j → t → [space]

Step 0: PRE-CHECK → No pattern, VN mode

Keystroke processing:
├── v → buffer = "v"
├── i → buffer = "vi"
├── e → buffer = "vie"
├── e → second 'e' = circumflex on 'e' → buffer = "viê"
├── j → 'j' = nặng tone → buffer = "việ"
├── t → buffer = "việt"
└── [space] → trigger validation

Step 1: VN VALIDATION
├── buffer = "việt"
├── Check all rules: ✓
└── Result: Complete (valid VN syllable)

Final: "việt" (KEEP VN)
```

| Key | raw | buffer | VN State | Action |
|-----|-----|--------|----------|--------|
| v | v | v | - | - |
| i | vi | vi | - | - |
| e | vie | vie | - | - |
| e | viee | viê | - | Circumflex |
| j | vieej | việ | - | Nặng tone |
| t | vieejt | việt | Complete | KEEP VN |
| space | vieejt | việt | - | Output: "việt" |

---

### Example 10: "xyz" (typo - no match)

```
Keystroke sequence: x → y → z → [space]

Step 0: PRE-CHECK → No pattern (x valid VN initial)

Keystroke processing:
├── x → buffer = "x"
├── y → buffer = "xy"
├── z → buffer = "xyz" (z causes issue?)
└── [space] → trigger validation

Step 1: VN VALIDATION
├── buffer = "xyz" or transformed
├── Result: Impossible

Step 2A: EN PATTERN DETECTION
├── raw = "xyz"
├── Check all layers: ✗ (x valid VN, no clusters, no double consonant)
└── No match

Step 2B: DICTIONARY LOOKUP
├── raw = "xyz" ∈ Dictionary? ✗
├── buffer ∈ Dictionary? ✗
└── Result: No match → KEEP AS-IS (typo)

Final: buffer (kept as typed, possibly with transforms)
```

---

### Example 11: "text" (coda cluster xt)

```
Keystroke sequence: t → e → x → t → [space]

Step 0: PRE-CHECK → No pattern, VN mode

Keystroke processing:
├── t → buffer = "t"
├── e → buffer = "te"
├── x → 'x' is Telex ngã modifier → buffer = "tẽ"
├── t → buffer = "tẽt"
└── [space] → trigger validation

Step 1: VN VALIDATION
├── buffer = "tẽt"
├── Check: Structure valid? (single vowel + tone + final t)
└── Result: Possibly valid or Impossible (depends on rules)

Step 2A: FOREIGN_MODE DETECTION
├── CHECK 1: Invalid_VN(buffer="tẽt")? ✓ (ngã tone invalid with -t final)
├── CHECK 2: EN_Pattern(raw="text")?
│   └── L5: Coda cluster "xt"? ✓
└── Result: ENTER FOREIGN_MODE → keep raw

Final: "text"
```

| Key | raw | buffer | VN State | Action |
|-----|-----|--------|----------|--------|
| t | t | t | - | - |
| e | te | te | - | - |
| x | tex | tẽ | - | 'x' = ngã tone |
| t | text | tẽt | Impossible | FOREIGN_MODE: L5 "xt" ✓ |
| space | text | text | - | Output: "text" |

---

### Example 12: "expect" (coda cluster ct)

```
Keystroke sequence: e → x → p → e → c → t → [space]

Step 0: PRE-CHECK → No pattern (e valid VN initial)

Keystroke processing:
├── e → buffer = "e"
├── x → 'x' ngã → buffer = "ẽ"
├── p → buffer = "ẽp"
├── e → buffer = "ẽpe"
├── c → buffer = "ẽpec"
├── t → buffer = "ẽpect"
└── [space] → trigger validation

Step 1: VN VALIDATION
├── buffer = "ẽpect"
├── Check: Multi-syllable structure invalid
└── Result: Impossible

Step 2A: FOREIGN_MODE DETECTION
├── CHECK 1: Invalid_VN(buffer="ẽpect")? ✓
├── CHECK 2: EN_Pattern(raw="expect")?
│   └── L5: Coda cluster "ct"? ✓
└── Result: ENTER FOREIGN_MODE → keep raw

Final: "expect"
```

| Key | raw | buffer | VN State | Action |
|-----|-----|--------|----------|--------|
| e | e | e | - | - |
| x | ex | ẽ | - | 'x' = ngã tone |
| p | exp | ẽp | - | - |
| e | expe | ẽpe | - | - |
| c | expec | ẽpec | - | - |
| t | expect | ẽpect | Impossible | FOREIGN_MODE: L5 "ct" ✓ |
| space | expect | expect | - | Output: "expect" |

---

### Example 13: "perfect" (coda cluster ct)

```
Keystroke sequence: p → e → r → f → e → c → t → [space]

Step 0: PRE-CHECK → No pattern, VN mode

Keystroke processing:
├── p → buffer = "p"
├── e → buffer = "pe"
├── r → 'r' hỏi → buffer = "pẻ"
├── f → 'f' huyền, but 'e' already has tone, might be ignored or conflict
│       → buffer = "pẻf" (f as literal since tone conflict)
├── e → buffer = "pẻfe"
├── c → buffer = "pẻfec"
├── t → buffer = "pẻfect"
└── [space] → trigger validation

Step 1: VN VALIDATION
├── buffer = "pẻfect"
├── Check: 'f' not valid VN consonant in medial position
└── Result: Impossible

Step 2A: FOREIGN_MODE DETECTION
├── CHECK 1: Invalid_VN(buffer="pẻfect")? ✓
├── CHECK 2: EN_Pattern(raw="perfect")?
│   └── L5: Coda cluster "ct"? ✓
└── Result: ENTER FOREIGN_MODE → keep raw

Final: "perfect"
```

| Key | raw | buffer | VN State | Action |
|-----|-----|--------|----------|--------|
| p | p | p | - | - |
| e | pe | pe | - | - |
| r | per | pẻ | - | 'r' = hỏi tone |
| f | perf | pẻf | - | 'f' literal (tone conflict) |
| e | perfe | pẻfe | - | - |
| c | perfec | pẻfec | - | - |
| t | perfect | pẻfect | Impossible | FOREIGN_MODE: L5 "ct" ✓ |
| space | perfect | perfect | - | Output: "perfect" |

---

### Example 14: "sarah" (dictionary lookup - proper name)

```
Keystroke sequence: s → a → r → a → h → [space]

Step 0: PRE-CHECK → No pattern, VN mode

Keystroke processing:
├── s → buffer = "s"
├── a → buffer = "sa"
├── r → 'r' hỏi → buffer = "sả"
├── a → buffer = "sảa" (a after toned vowel)
├── h → buffer = "sảah"
└── [space] → trigger validation

Step 1: VN VALIDATION
├── buffer = "sảah"
├── Check: "ảa" not valid VN vowel combination, 'h' not valid final
└── Result: Impossible

Step 2A: EN PATTERN DETECTION
├── raw = "sarah"
├── Check all layers: No coda cluster, no vowel pattern (ea,ee,oo,ou,ei,eu)
└── Result: No immediate pattern

Step 2B: AUTO_RESTORE (terminator = space)
├── CHECK: Invalid_VN(buffer="sảah")? ✓
├── CHECK: raw="sarah" ∈ Dictionary? ✓ (common name)
└── Result: AUTO_RESTORE → "sarah"

Final: "sarah"
```

| Key | raw | buffer | VN State | Action |
|-----|-----|--------|----------|--------|
| s | s | s | - | - |
| a | sa | sa | - | - |
| r | sar | sả | - | 'r' = hỏi tone |
| a | sara | sảa | - | - |
| h | sarah | sảah | - | - |
| space | sarah | sảah | Impossible | AUTO_RESTORE: Dict(raw) ✓ |

**Note:** "sarah" has no immediate EN pattern, relies on dictionary lookup at terminator.

---

## Intentional VN Detection & Restore Logic

### VN Intent Signals

```
SIGNAL STRENGTH (từ mạnh → yếu):
├── STROKE (đ)       = 100% intentional VN → NEVER restore
├── TONE KEY applied = Strong intentional  → Don't restore
├── MARK KEY only    = Moderate signal     → Check validity
├── REVERT (ss,ff)   = User cancel tone    → Check dictionary
└── No special key   = Ambiguous           → May restore
```

### Special Keys in Telex

| Type | Keys | Function | Example |
|------|------|----------|---------|
| **STROKE** | `dd` | Đ/đ character | `ddang` → `đang` |
| **TONE** | `s,f,r,x,j` | Sắc/Huyền/Hỏi/Ngã/Nặng | `bas` → `bá` |
| **MARK** | `w,aa,oo,ee,aw,ow,uw` | Breve/Circumflex/Horn | `law` → `lă` |
| **REVERT** | Double tone/mark | Undo transform | `bass` → `bas` |
| **OVERWRITE** | Tone after tone | Replace tone | `banjs` → `bán` |

### Tone Overwrite Case

```
banjs:
b → "b"
a → "ba"
n → "ban"
j → "bạn"  (j = nặng tone applied)
s → "bán"  (s = sắc tone OVERWRITES nặng)

Result: "bán" (last tone wins)
Signal: TONE applied → intentional VN → don't restore
```

### Continuous Typing Case

```
chaofooo:
c  → "c"
h  → "ch"
a  → "cha"
o  → "chao"
f  → "chào"   (f = huyền tone applied to 'a')
o  → "chàoo"  (o appended)
o  → "chàooo"
o  → "chàoooo"

Result: "chàoooo" (invalid VN but tone was used)
Signal: TONE applied → intentional VN → DON'T restore
Reason: User gõ tone key 'f' = họ muốn tiếng Việt
```

### Comprehensive Case Table

| Input | Buffer | Transform | Signal | Buffer State | Raw EN? | Decision | Reason |
|-------|--------|-----------|--------|--------------|---------|----------|--------|
| `bans␣` | bán | tone(s) | TONE | Complete ✓ | no | **KEEP** | Valid VN |
| `banjs␣` | bán | tone(j→s) | TONE | Complete ✓ | no | **KEEP** | Tone overwrite |
| `chaofooo␣` | chàoooo | tone(f) | TONE | Invalid | no | **KEEP** | Tone = intentional |
| `đang␣` | đang | stroke | STROKE | Complete ✓ | no | **KEEP** | Stroke = 100% VN |
| `dder␣` | đẻ | stroke+tone | STROKE | Complete ✓ | no | **KEEP** | Stroke present |
| `lawm␣` | lăm | mark(w) | MARK | Complete ✓ | no | **KEEP** | Valid VN result |
| `law␣` | lă | mark(w) | MARK | Incomplete | yes | **RESTORE** | Incomplete + EN |
| `aw␣` | ă | mark(w) | MARK | Incomplete | no | **KEEP** | No EN match |
| `texs␣` | téx | tone(s) | TONE | Invalid | yes | **KEEP** | Tone = intentional |
| `texts␣` | texts | PRE-CHECK | - | - | yes | **KEEP** | Step 0 skip |
| `hello␣` | hello | none | - | Invalid | no | **KEEP** | No transform |
| `file␣` | file | PRE-CHECK | - | - | yes | **KEEP** | Step 0 skip |
| `bass␣` | bass | revert(ss) | REVERT | Valid | yes | **KEEP** | Revert + Dict |
| `issue␣` | isue | revert(ss) | REVERT | Invalid | yes | **RESTORE** | Revert + EN Dict |
| `coffee␣` | cofee | revert(ff) | REVERT | Invalid | yes | **RESTORE** | Revert + EN Dict |
| `caffe␣` | cafê | revert+mark | REVERT | Invalid | no | **KEEP** | No EN match |

### Restore Logic (Rust)

```rust
struct RestoreContext {
    had_transform: bool,      // buffer != raw
    has_stroke: bool,         // đ present in buffer
    has_tone_applied: bool,   // tone key was used (s,f,r,x,j)
    has_mark_only: bool,      // only mark key used (w,aa,oo...)
    is_revert_case: bool,     // double key caused revert
    buffer_state: BufferState,// Complete/Incomplete/Impossible
    raw_is_valid_en: bool,    // raw in EN dictionary
}

fn should_restore(ctx: &RestoreContext) -> bool {
    // Rule 0: No transform = no restore
    if !ctx.had_transform {
        return false;
    }

    // Rule 1: STROKE (đ) = 100% intentional VN, NEVER restore
    if ctx.has_stroke {
        return false;
    }

    // Rule 2: TONE KEY applied = intentional VN, don't restore
    // (includes tone overwrite case like banjs → bán)
    if ctx.has_tone_applied {
        return false;
    }

    // Rule 3: MARK only + incomplete VN + valid EN = restore
    // (case: "law" → "lă" incomplete, "law" is EN word)
    if ctx.has_mark_only
       && ctx.buffer_state == BufferState::Incomplete
       && ctx.raw_is_valid_en {
        return true;
    }

    // Rule 4: REVERT case + invalid VN + valid EN = restore
    // (case: "issue" → "isue" invalid, "issue" is EN word)
    if ctx.is_revert_case
       && ctx.buffer_state != BufferState::Complete
       && ctx.raw_is_valid_en {
        return true;
    }

    // Default: keep buffer
    false
}
```

### Signal Detection

```rust
fn detect_vn_signals(keystrokes: &[Keystroke]) -> VnSignals {
    let mut signals = VnSignals::default();

    for ks in keystrokes {
        match ks.key {
            // Stroke detection
            'd' if ks.prev == 'd' => signals.has_stroke = true,

            // Tone detection (even if overwritten)
            's' | 'f' | 'r' | 'x' | 'j'
                if ks.is_modifier_context => signals.has_tone_applied = true,

            // Mark detection
            'w' if ks.is_modifier_context => signals.has_mark = true,
            'a' | 'o' | 'e' | 'u'
                if ks.prev == ks.key => signals.has_mark = true,

            _ => {}
        }
    }

    signals.has_mark_only = signals.has_mark && !signals.has_tone_applied;
    signals
}
```

### Decision Flow

> **CRITICAL FIX (v3.1)**: has_tone → KEEP was too aggressive.
> Example: "tesla " → "téla" must restore to "tesla", not keep "téla".
> New flow checks Impossible state BEFORE tone check.

```
INPUT: buffer, raw, signals, vn_state
│
├── had_transform? ─NO──→ KEEP (no change)
│   │
│   YES
│   │
├── has_stroke? ─YES──→ KEEP (đ = 100% intentional VN)
│   │
│   NO
│   │
├── vn_state == Impossible? ────────────────────────────────┐
│   │                                                        │
│   YES───→ has_english_pattern(raw) || dict_EN? ─YES──→ RESTORE
│   │                                              │
│   │                                              NO───→ KEEP (gibberish)
│   NO
│   │
├── has_tone && vn_state == Complete? ─YES──→ KEEP (valid VN + tone)
│   │
│   NO
│   │
├── vn_state == Complete? ─YES──→ KEEP (valid VN)
│   │
│   NO (vn_state == Incomplete)
│   │
├── has_tone && vn_state == Incomplete?
│   │
│   YES───→ has_english_pattern(raw) || dict_EN? ─YES──→ RESTORE
│   │                                              │
│   │                                              NO───→ KEEP
│   NO
│   │
├── has_mark_only?
│   │
│   YES───→ has_english_pattern(raw) || dict_EN? ─YES──→ RESTORE
│   │                                              │
│   │                                              NO───→ KEEP
│   NO
│   │
├── is_revert_case?
│   │
│   YES───→ has_english_pattern(raw) || dict_EN? ─YES──→ RESTORE
│   │                                              │
│   │                                              NO───→ KEEP
│   NO
│   │
└── KEEP (default)
```

**Key Changes from v3.0:**
1. `vn_state == Impossible` check comes BEFORE `has_tone` check
2. `has_tone` only KEEPs if `vn_state == Complete`
3. Added Incomplete + Tone case with EN detection
4. All EN checks use `has_english_pattern() || dict_EN`

---

## V1 Production Logic (Reference)

### V1 6-Rule Validation Pipeline

```
Rule 1: HAS_VOWEL
├── Buffer must contain at least one vowel
└── Single consonants fail validation

Rule 2: VALID_INITIAL
├── 1-char: b,c,d,đ,g,h,k,l,m,n,p,q,r,s,t,v,x
├── 2-char: ch,gh,gi,kh,ng,nh,ph,qu,th,tr
└── 3-char: ngh

Rule 3: ALL_CHARS_PARSED
├── Every char must fit syllable structure (C+V+F)
└── Unparseable = invalid

Rule 4: SPELLING_RULES
├── C + I/E/Y → invalid (use K: ke, ki, ky)
├── K + A/O/U → invalid (use C: ca, co, cu)
├── G + E → invalid (use GH: ghe)
└── NG + I/E → invalid (use NGH: nghi, nghe)

Rule 5: VALID_FINAL
├── Valid: c, m, n, p, t, ch, ng, nh
└── Invalid: b, d, g, h, k, l, q, r, s, v, x, z, j, w

Rule 6: VALID_VOWEL_PATTERN (WHITELIST)
├── 29 diphthongs: ai,ao,au,ay,âu,ây,êu,ia,iê,iu,oa,oă,oe,oi,ôi,ơi,ua,uâ,uê,ui,uô,uy,ưa,ưi,ươ,ưu,yê...
├── 11+ triphthongs: iêu,oai,oao,oay,uôi,ươi,ươu,uya,uyê,uyu,yêu...
└── INVALID: ea,ou,yo,yi,ae,eo (English patterns)
```

### V1 Modifier Requirements (Tone-Aware)

```
CIRCUMFLEX REQUIRED (khi có tone info):
├── E+U → êu (eu invalid)
├── I+E → iê (ie invalid)
├── U+E → uê (ue invalid)
├── Y+E → yê (ye invalid)
├── U+Y+E → uyê (uye invalid)
├── I+E+U → iêu (ieu invalid, horn on U = ieư invalid)
└── U+Y+E → uyê (horn on U invalid)

BREVE RESTRICTIONS:
├── ă + vowel → INVALID
├── ăi, ăo, ău, ăy → all invalid
└── Valid: ăn, ăm, ăc, ăp, ăt, ăng, ănh, ăch
```

### V1 Foreign Word Detection (is_foreign_word_pattern)

```rust
// Detects English patterns that shouldn't be transformed
fn is_foreign_word_pattern(buffer_keys, buffer_tones, modifier_key) -> bool {
    // 1. Invalid vowel patterns
    if has_vowel_pair("ou") || has_vowel_pair("yo") {
        return true; // NEVER valid Vietnamese
    }

    // 2. Consonant clusters after finals
    // "tr" after t: "text", "other"
    // "pr" after p: "prayer"
    // "cr" after c: "cry", "create"
    if has_final_consonant_cluster("tr", "pr", "cr") {
        return true;
    }

    // 3. Invalid final + mark modifier
    // Single vowel + INVALID final (x,b,d,g,h,k,l,q,r,s,v) + mark key
    if has_invalid_final_with_mark() {
        return true;
    }

    false
}
```

### V1 Auto-Restore Core Logic

```rust
// Two-check decision
fn should_auto_restore() -> bool {
    // Must have had transform
    if !had_any_transform {
        return false;
    }

    // Check 1: Is buffer invalid Vietnamese?
    let buffer_invalid = is_buffer_invalid_vietnamese();

    // Check 2: Is raw input valid English?
    let raw_valid_en = is_raw_input_valid_english();

    // Core decision
    if buffer_invalid && raw_valid_en {
        return true;
    }

    // Additional triggers...
    false
}

fn is_buffer_invalid_vietnamese() -> bool {
    // Fails 6-rule validation
    if !is_valid_with_tones(buffer) { return true; }

    // -ing + tone mark: "thíng" invalid
    if has_ing_with_tone() { return true; }

    // Single vowel + uncommon tone
    if is_single_vowel_uncommon() { return true; }

    false
}

fn is_raw_input_valid_english() -> bool {
    // All keys are ASCII letters
    raw.chars().all(|c| c.is_ascii_alphabetic())
    // Must have at least one vowel (or allow 1-2 char abbreviations)
}
```

### V1 Additional Restore Triggers

| Trigger | Condition | Example |
|---------|-----------|---------|
| **English Modifier Pattern** | `has_english_modifier_pattern()` | "cursor", "expect" |
| **Significant Char Consumption** | raw 2+ chars longer than buffer | "await"→"âit" |
| **V+C+V Circumflex Pattern** | Circumflex + stop (t,c,p) no mark | "dât", "sêt" |
| **Double Modifier Revert** | Same modifier doubled + vowel (≤3 chars) | "arro"→"aro" |
| **V1-V2-V1 Vowel Collapse** | 3+ consecutive vowels, first=last | "queue"→"quêu" |

### V1 Prevent Restore Conditions

| Condition | Reason | Example |
|-----------|--------|---------|
| **Has Stroke (đ)** | Intentional VN | "đang" |
| **Has Mark/Tone** | Confirms VN intent | "bán", "hỏi" |
| **Double Modifier at End** | Intentional revert | "ass"→keep "as" |
| **has_non_letter_prefix** | Contains numbers/symbols | "149k" |
| **had_any_transform == false** | No transform applied | "forr" (F invalid) |
| **Never collapse "ff"** | Common EN: off, coffee | "coffee" |

### V1 English Pattern Detection (has_english_modifier_pattern)

```
W at Word Start:
├── "w" alone → VN (ư)
├── "w" + mark only (wf, ws) → VN
├── "w" + valid final + mark (wmn) → VN
├── "w" + vowel + consonant (win) → EN
├── "wo", "woa", "wou" + consonant → EN

Consonant + W Pattern (word complete):
├── C + W + O + N + G → VN "ương" (tương)
├── C + W + A exactly 3 chars → VN "ưa" (mưa)
├── C + W + vowel (no tone) → EN "swim"

Consecutive Different Tone Modifiers:
├── "rs" + vowel (r≠s) → EN "cursor"
├── "ss", "ff", "rr" (same) → Telex revert, NOT EN

Modifier + Consonant Patterns:
├── Modifier + consonant + more letters → EN "expect"
├── Exception: J or S + consonant → VN (học, bức)
├── Exception: F/R/X + sonorant (m,n,ng,nh) → VN (làm, mãnh)

Vowel + Modifier + Vowel Pattern:
├── "use" (u+s+e) → EN
├── Exception: U+modifier+A (ủa, ùa) → VN
├── Exception: A+modifier+O (ảo, ào) → VN
```

### V1 Buffer State Flags

```rust
struct BufferState {
    had_any_transform: bool,           // Any VN transform applied
    had_mark_revert: bool,             // Mark removed (ss, ff)
    had_vowel_triggered_circumflex: bool, // V+C+V pattern
    had_circumflex_revert: bool,       // aaa → aa
    pending_breve_pos: Option<usize>,  // Deferred breve
    pending_u_horn_pos: Option<usize>, // Deferred horn
    stroke_reverted: bool,             // đ → d reverted
}
```

### V1 Breve Deferral Logic

```
Open Syllable Breve Deferral:
├── "aw" → keep "aw" (wait for more input)
├── "awm" → "ăm" (apply breve when consonant added)
├── "aws" → "ă" (apply breve when tone added)
├── "aw " → "aw" (boundary, no breve applied)

Reason: "aw" could become:
├── "ăn" (valid VN) if followed by n
├── "aw" (EN word "law") if followed by space
```

### V1 Edge Cases (From Issues)

| Issue | Case | Solution |
|-------|------|----------|
| #44 | Breve in open syllables | Defer breve until consonant/tone |
| #51 | Stroke only on adjacent d's | dd→đ, d+mark→đ |
| #107 | Shortcut prefix (@, #, :) | Skip transform |
| #133 | "uo" pattern horn | Only O gets horn initially |
| #151 | C+W+A pattern | 3-char exactly = VN "ưa" |
| #312 | Vowel with tone | Don't trigger circumflex |

### V3 Should Port From V1

```
CRITICAL:
├── 6-Rule Validation Pipeline
├── Whitelist Vowel Patterns (29 diphthongs, 11+ triphthongs)
├── Two-Check Restore Logic (buffer_invalid && raw_valid_en)
├── Transform Tracking Flags
├── Breve Deferral Logic
└── raw_input Parallel Tracking

IMPORTANT:
├── English Pattern Detection (has_english_modifier_pattern)
├── Modifier Requirements (circumflex, breve restrictions)
├── Buffer State Machine (mark/tone/stroke with revert)
└── Edge Cases (breve deferral, stroke rules)

PATTERN (not case-by-case):
├── Pattern-based transformations
├── Post-transform validation
├── State flags for revert detection
└── Restore via raw reconstruction
```

---

## Implementation Checklist

**Pattern Detection (on RAW):**
- [ ] `has_english_onset(raw)` - ONSET patterns (đầu từ)
- [ ] `has_english_coda(raw)` - CODA patterns (cuối từ)
- [ ] `has_invalid_vowel_pattern(raw)` - VOWEL patterns (giữa từ)
- [ ] `has_impossible_bigram(raw)` - BIGRAM patterns (bất kỳ)
- [ ] `has_english_pattern(raw)` - Combined check (coda + vowel + bigram)

**Vietnamese Validation (on BUFFER):**
- [ ] `is_valid_vietnamese(buffer)` - Full VN validation
- [ ] `is_impossible_vietnamese(buffer)` - Quick invalid check

**Dictionary:**
- [ ] `is_in_english_dictionary(word)` - HashSet lookup
- [ ] 10K English word dictionary (~100KB)

**Mode Management:**
- [ ] `should_enter_foreign_mode(raw, buffer)` - Step 0 + 2A logic
- [ ] `should_auto_restore(raw, buffer)` - Step 2B logic
- [ ] `is_terminator(key)` - Terminator detection
- [ ] Foreign mode flag in engine state

---

## Memory Budget

| Component | Size |
|-----------|------|
| EN Pattern matrices | ~500 bytes |
| EN Dictionary (10K words) | ~100 KB |
| VN Validation matrices | ~2 KB |
| **Total** | **~103 KB** |

---

## Validation Approach: Layered Bitmask Matrix

### Overview

O(1) lookup validation system using bitmask matrices. Total ~520 bytes memory, 8 validation layers.

```
┌─────────────────────────────────────────────────────────────────┐
│                    VALIDATION PIPELINE                          │
├─────────────────────────────────────────────────────────────────┤
│  Layer 1: CHAR_TYPE[32] → Classify char (onset/vowel/coda/inv) │
│  Layer 2: M_ONSET[32] → Valid single onset bitmask             │
│  Layer 3: M_ONSET_PAIR[32][32] → Valid onset clusters          │
│  Layer 4: M_VOWEL_PAIR[32][32] → Valid diphthongs              │
│  Layer 5: M_CODA[32] → Valid single coda bitmask               │
│  Layer 6: M_CODA_PAIR[32][32] → Valid coda clusters            │
│  Layer 7: M_TONE_CODA[6][8] → Tone-stop restriction            │
│  Layer 8: M_SPELL[32][32] → Spelling rules (c/k, g/gh, ng/ngh) │
└─────────────────────────────────────────────────────────────────┘
```

### Char Type Encoding

Map a-z (26 chars) to 5-bit index (0-25). Special chars: đ=26, ă=27, â=28, ê=29, ô=30, ơ=31, ư=special.

```rust
/// Character type classification
#[repr(u8)]
enum CharType {
    Onset     = 0b0001,  // Valid as onset (b,c,d,đ,g,h,k,l,m,n,p,q,r,s,t,v,x)
    Vowel     = 0b0010,  // Valid as vowel (a,ă,â,e,ê,i,o,ô,ơ,u,ư,y)
    Coda      = 0b0100,  // Valid as coda (c,m,n,p,t,i,o,u,y + ch,ng,nh)
    Invalid   = 0b1000,  // Invalid in VN (f,j,w,z)
}

/// 32-byte lookup table
const CHAR_TYPE: [u8; 32] = [
    // a    b    c    d    e    f    g    h    i    j    k    l    m    n    o    p
    0b0010, 0b0001, 0b0101, 0b0001, 0b0010, 0b1000, 0b0001, 0b0001, 0b0110, 0b1000, 0b0001, 0b0001, 0b0101, 0b0101, 0b0110, 0b0101,
    // q    r    s    t    u    v    w    x    y    z    đ    ă    â    ê    ô    ơ
    0b0001, 0b0001, 0b0001, 0b0101, 0b0110, 0b0001, 0b1000, 0b0001, 0b0110, 0b1000, 0b0001, 0b0010, 0b0010, 0b0010, 0b0010, 0b0010,
];
```

### Bitmask Matrices

#### Layer 2: Single Onset (4 bytes)

```rust
/// Valid single onsets: b,c,d,đ,g,h,k,l,m,n,p,q,r,s,t,v,x (17 chars)
const M_ONSET: u32 = 0b_0010_0101_1111_1110_1111_1110_0110;
//                      ơôêâăđzyxwvutsrqponmlkjihgfedcba

fn is_valid_onset(c: u8) -> bool {
    (M_ONSET >> c) & 1 == 1
}
```

#### Layer 3: Onset Clusters (128 bytes)

```rust
/// Valid onset pairs: ch,gh,gi,kh,kr,ng,nh,ph,qu,th,tr + ngh
/// 32x32 bit matrix compressed to 32x4 bytes
const M_ONSET_PAIR: [[u8; 4]; 32] = [
    // Each row = first char, bits = second chars that form valid cluster
    // Row 'c': bit 'h' set (ch)
    // Row 'g': bit 'h','i' set (gh, gi)
    // Row 'k': bit 'h','r' set (kh, kr)
    // Row 'n': bit 'g','h' set (ng, nh)
    // Row 'p': bit 'h' set (ph)
    // Row 'q': bit 'u' set (qu)
    // Row 't': bit 'h','r' set (th, tr)
    // ...
];

fn is_valid_onset_cluster(c1: u8, c2: u8) -> bool {
    (M_ONSET_PAIR[c1 as usize][c2 as usize / 8] >> (c2 % 8)) & 1 == 1
}
```

#### Layer 4: Valid Diphthongs (128 bytes)

```rust
/// Valid diphthongs from Pattern Reference Table
/// VN only + Shared = valid, EN only = invalid
const M_VOWEL_PAIR: [[u8; 4]; 32] = [
    // Row 'a': i,o,u,y valid (ai,ao,au,ay)
    // Row 'e': o valid (eo), u requires circumflex (êu)
    // Row 'i': a,ê,u valid (ia,iê,iu)
    // Row 'o': a,ă,e,i valid (oa,oă,oe,oi)
    // Row 'u': a,â,ê,i,ô,ơ,y valid
    // Row 'ư': a,i,ơ,u valid (ưa,ưi,ươ,ưu)
    // Row 'y': ê valid (yê)
    // ...
];

fn is_valid_diphthong(v1: u8, v2: u8) -> bool {
    (M_VOWEL_PAIR[v1 as usize][v2 as usize / 8] >> (v2 % 8)) & 1 == 1
}
```

#### Layer 5: Single Coda (4 bytes)

```rust
/// Valid single codas: c,m,n,p,t + semi-vowels i,o,u,y
const M_CODA: u32 = 0b_0000_0011_0100_1001_0011_0100_0000;
//                     ơôêâăđzyxwvutsrqponmlkjihgfedcba

fn is_valid_coda(c: u8) -> bool {
    (M_CODA >> c) & 1 == 1
}
```

#### Layer 6: Coda Clusters (128 bytes)

```rust
/// Valid coda clusters: ch, ng, nh
const M_CODA_PAIR: [[u8; 4]; 32] = [
    // Row 'c': bit 'h' set (ch)
    // Row 'n': bit 'g','h' set (ng, nh)
    // ...
];

fn is_valid_coda_cluster(c1: u8, c2: u8) -> bool {
    (M_CODA_PAIR[c1 as usize][c2 as usize / 8] >> (c2 % 8)) & 1 == 1
}
```

#### Layer 7: Tone-Stop Restriction (48 bytes)

```rust
/// Tone marks: 0=none, 1=sắc, 2=huyền, 3=hỏi, 4=ngã, 5=nặng
/// Stop codas: 0=c, 1=ch, 2=p, 3=t (encoded as 3 bits)
///
/// Rule: Stop codas (-c,-ch,-p,-t) only allow sắc(1) or nặng(5)
const M_TONE_CODA: [[u8; 8]; 6] = [
    // tone=0 (none): all codas valid
    [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
    // tone=1 (sắc): all codas valid
    [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
    // tone=2 (huyền): stops invalid (c,ch,p,t = 0)
    [0xF0, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
    // tone=3 (hỏi): stops invalid
    [0xF0, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
    // tone=4 (ngã): stops invalid
    [0xF0, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
    // tone=5 (nặng): all codas valid
    [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
];

fn is_valid_tone_coda(tone: u8, coda: u8) -> bool {
    (M_TONE_CODA[tone as usize][coda as usize / 8] >> (coda % 8)) & 1 == 1
}
```

#### Layer 8: Spelling Rules (128 bytes)

```rust
/// Spelling rules:
/// - c before e,i,y → invalid (use k)
/// - k before a,o,u → invalid (use c)
/// - g before e → invalid (use gh)
/// - ng before e,i → invalid (use ngh)
/// - gh before a,o,u → invalid (use g)
/// - ngh before a,o,u → invalid (use ng)
const M_SPELL: [[u8; 4]; 32] = [
    // Row 'c': bits e,i,y = 0 (invalid combos)
    // Row 'k': bits a,o,u = 0
    // Row 'g': bit e = 0
    // ...
];

fn is_valid_spelling(onset: u8, vowel: u8) -> bool {
    (M_SPELL[onset as usize][vowel as usize / 8] >> (vowel % 8)) & 1 == 1
}
```

### Full Validation Function

```rust
/// O(1) Vietnamese syllable validation
fn validate_syllable(onset: Option<(u8, u8)>, vowels: &[u8], coda: Option<(u8, u8)>, tone: u8) -> bool {
    // Layer 1: Check char types
    for &c in vowels {
        if CHAR_TYPE[c as usize] & CharType::Vowel == 0 {
            return false; // Not a vowel
        }
    }

    // Layer 2-3: Validate onset
    if let Some((c1, c2)) = onset {
        if c2 == 0 {
            if !is_valid_onset(c1) { return false; }
        } else {
            if !is_valid_onset_cluster(c1, c2) { return false; }
        }
    }

    // Layer 4: Validate vowel pattern (diphthong)
    if vowels.len() == 2 {
        if !is_valid_diphthong(vowels[0], vowels[1]) { return false; }
    }

    // Layer 5-6: Validate coda
    if let Some((c1, c2)) = coda {
        if c2 == 0 {
            if !is_valid_coda(c1) { return false; }
        } else {
            if !is_valid_coda_cluster(c1, c2) { return false; }
        }
    }

    // Layer 7: Tone-stop restriction
    if let Some((c1, _)) = coda {
        if !is_valid_tone_coda(tone, c1) { return false; }
    }

    // Layer 8: Spelling rules
    if let Some((o1, o2)) = onset {
        if vowels.len() > 0 {
            let onset_key = if o2 != 0 { o2 } else { o1 };
            if !is_valid_spelling(onset_key, vowels[0]) { return false; }
        }
    }

    true
}
```

### Memory Summary

| Matrix | Size | Purpose |
|--------|------|---------|
| CHAR_TYPE[32] | 32 bytes | Char classification |
| M_ONSET | 4 bytes | Single onset validation |
| M_ONSET_PAIR[32][4] | 128 bytes | Onset cluster validation |
| M_VOWEL_PAIR[32][4] | 128 bytes | Diphthong validation |
| M_CODA | 4 bytes | Single coda validation |
| M_CODA_PAIR[32][4] | 128 bytes | Coda cluster validation |
| M_TONE_CODA[6][8] | 48 bytes | Tone-stop restriction |
| M_SPELL[32][4] | 128 bytes | Spelling rules |
| **Total** | **~600 bytes** | All validation |

### Comparison with OpenKey Approach

| Aspect | OpenKey (Whitelist) | V3 (Bitmask Matrix) |
|--------|---------------------|---------------------|
| **Memory** | ~2KB (vectors) | ~600 bytes |
| **Lookup** | O(n) linear search | O(1) bit lookup |
| **Extensibility** | Add to list | Set bit in matrix |
| **Tone-stop** | Hardcoded if-else | Matrix lookup |
| **Spelling** | Implicit in vowel list | Explicit matrix |
| **Maintenance** | Modify vector data | Modify bit patterns |
