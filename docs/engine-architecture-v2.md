# GõNhanh Engine Architecture - Deep Analysis

Phân tích kiến trúc toàn diện, xác định điểm yếu, điểm hở và đề xuất cải thiện.

---

## 0. Architecture Diagrams

### 0.1 Current Architecture (Problems)

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          mod.rs (3,771 lines)                           │
│                              GOD FILE                                    │
├─────────────────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                    Engine Struct (25+ fields)                    │   │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────────────┐│   │
│  │  │   Buffer    │ │  raw_input  │ │   15+ Boolean Flags         ││   │
│  │  │ (transform) │ │   (manual)  │ │  stroke_reverted            ││   │
│  │  │             │ │             │ │  had_mark_revert            ││   │
│  │  │    ↕ ??     │ │    ↕ ??     │ │  pending_mark_revert_pop    ││   │
│  │  │ OUT OF SYNC │ │ OUT OF SYNC │ │  had_any_transform          ││   │
│  │  └─────────────┘ └─────────────┘ │  pending_capitalize...      ││   │
│  │                                   └─────────────────────────────┘│   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                    60+ Functions (Mixed)                         │   │
│  │  on_key_ext() ─┬→ try_stroke()                                   │   │
│  │                ├→ try_tone()                                     │   │
│  │                ├→ try_mark()         ┌─────────────────────┐     │   │
│  │                ├→ try_remove()       │ Validation calls    │     │   │
│  │                ├→ try_w_as_vowel()   │ SCATTERED across    │     │   │
│  │                └→ handle_normal()    │ all functions       │     │   │
│  │                                      └─────────────────────┘     │   │
│  └─────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
                                    ↓
        ┌───────────────────────────┼───────────────────────────┐
        ↓                           ↓                           ↓
┌───────────────┐         ┌─────────────────┐         ┌─────────────────┐
│  buffer.rs    │         │  syllable.rs    │         │ validation.rs   │
│  (Buffer,Char)│         │  (Parser)       │         │ (6 Rules)       │
│   Clean ✓     │         │   Clean ✓       │         │  Clean ✓        │
└───────────────┘         └─────────────────┘         └─────────────────┘
```

### 0.2 Proposed Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           mod.rs (<500 lines)                           │
│                         ORCHESTRATOR ONLY                               │
├─────────────────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │              Engine Struct (Minimal, State Machine)              │   │
│  │  ┌──────────────────┐  ┌────────────────────────────────────┐   │   │
│  │  │   EngineState    │  │          DualBuffer                │   │   │
│  │  │  ┌────────────┐  │  │  ┌────────────┐  ┌──────────────┐  │   │   │
│  │  │  │   Empty    │  │  │  │ transformed│←→│    raw       │  │   │   │
│  │  │  │   Initial  │  │  │  │  (Buffer)  │  │ (RawHistory) │  │   │   │
│  │  │  │   Vowel*   │  │  │  │            │  │              │  │   │   │
│  │  │  │   Final    │  │  │  │  ALWAYS    │  │   consumed   │  │   │   │
│  │  │  │   Marked   │  │  │  │   SYNCED   │  │   tracking   │  │   │   │
│  │  │  │   Foreign  │  │  │  └────────────┘  └──────────────┘  │   │   │
│  │  │  └────────────┘  │  └────────────────────────────────────┘   │   │
│  │  └──────────────────┘                                           │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  on_key_ext() → transition(Transition) → dispatch to modules           │
└─────────────────────────────────────────────────────────────────────────┘
         │              │               │               │
         ↓              ↓               ↓               ↓
┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────────┐
│  stroke.rs  │ │  tone.rs    │ │  mark.rs    │ │  restore.rs     │
│  try_stroke │ │  try_tone   │ │  try_mark   │ │  try_restore    │
│  revert     │ │  circumflex │ │  sắc/huyền  │ │  bidirectional  │
│  logic      │ │  horn/breve │ │  hỏi/ngã..  │ │  VN+EN validate │
└─────────────┘ └─────────────┘ └─────────────┘ └─────────────────┘
         │              │               │               │
         └──────────────┴───────────────┴───────────────┘
                                │
         ┌──────────────────────┼──────────────────────┐
         ↓                      ↓                      ↓
┌─────────────────┐   ┌─────────────────┐   ┌─────────────────────┐
│  validation.rs  │   │   english.rs    │   │    syllable.rs      │
│  (8 Rules)      │   │   (NEW)         │   │    (Parser)         │
│  + ToneStop     │   │   Pattern-based │   │                     │
│  + VowelFinal   │   │   Morphology    │   │                     │
│                 │   │   Bloom (opt)   │   │                     │
└─────────────────┘   └─────────────────┘   └─────────────────────┘
```

### 0.3 State Machine Flow

```
                              ┌─────────┐
                              │  Empty  │
                              └────┬────┘
                                   │ AddInitial(k)
                                   ↓
                              ┌─────────┐
          AddVowel(k)         │ Initial │
      ┌──────────────────────→└────┬────┘
      │                            │ AddVowel(k)
      │                            ↓
      │                      ┌───────────┐
      │     AddVowel(k)      │VowelStart │←─────────────────────┐
      │   ┌─────────────────→└─────┬─────┘                      │
      │   │                        │ AddVowel(k)                │
      │   │                        ↓                            │
      │   │                  ┌─────────────┐                    │
      │   │                  │VowelCompound│                    │
      │   │                  └──────┬──────┘                    │
      │   │                         │ AddFinal(k)               │ Revert
      │   │                         ↓                           │
      │   │                   ┌───────────┐                     │
      │   │                   │   Final   │                     │
      │   │                   └─────┬─────┘                     │
      │   │                         │                           │
      │   │                         ↓                           │
      │   │    ┌────────────────────────────────────────────┐   │
      │   │    │              ApplyTone(t)                   │   │
      │   │    │              ApplyMark(m)                   │   │
      │   │    │              ApplyStroke                    │   │
      │   │    └────────────────────┬───────────────────────┘   │
      │   │                         ↓                           │
      │   │                   ┌───────────┐                     │
      │   │                   │  Marked   │─────────────────────┘
      │   │                   └───────────┘
      │   │                         │ Detect foreign pattern
      │   │                         ↓
      │   │                   ┌───────────┐
      │   │                   │  Foreign  │──→ (keep as-is OR restore)
      │   │                   └───────────┘
      │   │
      │   └──── Reset ───→ ┌─────────┐
      └────────────────────│  Empty  │
                           └─────────┘
```

### 0.4 Bidirectional Validation Pipeline

```
┌──────────────────────────────────────────────────────────────────────┐
│                    ON WORD BOUNDARY (Space/Break)                     │
└──────────────────────────────────────────────────────────────────────┘
                                   │
                                   ↓
                    ┌──────────────────────────┐
                    │ had_any_transform = true?│
                    └────────────┬─────────────┘
                          No     │     Yes
                          ↓      │      ↓
                    ┌─────────┐  │  ┌───────────────────────┐
                    │  KEEP   │  │  │  Validate Vietnamese  │
                    │ as-is   │  │  │    (8 rules)          │
                    └─────────┘  │  └───────────┬───────────┘
                                 │         Valid│     Invalid
                                 │              ↓           ↓
                                 │        ┌─────────┐  ┌───────────────────┐
                                 │        │  KEEP   │  │ Validate English  │
                                 │        │ VN word │  │ (pattern+morpho)  │
                                 │        └─────────┘  └─────────┬─────────┘
                                 │                          Valid│    Invalid
                                 │                               ↓          ↓
                                 │                         ┌──────────┐ ┌─────────┐
                                 │                         │ RESTORE  │ │  KEEP   │
                                 │                         │ to raw   │ │  as-is  │
                                 │                         └──────────┘ └─────────┘
                                 │                              │
                                 │   "text" (tẽt→text) ────────┘
                                 │   "đườngfffff" ─────────────────────→ KEEP
                                 │
                                 └─── "hello" (no transform) ────────→ KEEP
```

### 0.5 DualBuffer Structure

```
┌─────────────────────────────────────────────────────────────────────────┐
│                             DualBuffer                                   │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌────────────────────────────┐    ┌────────────────────────────────┐   │
│  │     transformed (Buffer)   │    │      raw (Vec<RawKeystroke>)   │   │
│  ├────────────────────────────┤    ├────────────────────────────────┤   │
│  │ [0] Char { key: V, ...}    │    │ [0] { key: V, consumed: false }│   │
│  │ [1] Char { key: I, tone:^} │    │ [1] { key: I, consumed: false }│   │
│  │ [2] Char { key: E, mark:´} │←──→│ [2] { key: E, consumed: false }│   │
│  │ [3] Char { key: T, ...}    │    │ [3] { key: E, consumed: TRUE } │←── modifier key
│  │                            │    │ [4] { key: T, consumed: false }│   │
│  └────────────────────────────┘    └────────────────────────────────┘   │
│         ↓ render()                          ↓ restore()                  │
│      "việt" (4 chars)                    "viet" (4 chars, skip consumed)│
│                                                                          │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                        Invariants                                │    │
│  │  • push() updates BOTH buffers atomically                       │    │
│  │  • pop() removes from BOTH (respecting consumed)                │    │
│  │  • restore() always reconstructs correct raw input              │    │
│  │  • NEVER out of sync                                            │    │
│  └─────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────┘
```

### 0.6 Module Dependency Graph (Proposed)

```
                          ┌──────────────────┐
                          │    FFI Layer     │
                          │   (C bindings)   │
                          └────────┬─────────┘
                                   │
                                   ↓
                          ┌──────────────────┐
                          │    mod.rs        │
                          │  (Orchestrator)  │
                          │   <500 lines     │
                          └────────┬─────────┘
                                   │
           ┌───────────────────────┼───────────────────────┐
           ↓                       ↓                       ↓
   ┌───────────────┐      ┌───────────────┐      ┌───────────────┐
   │  transform/   │      │   validate/   │      │   restore.rs  │
   │  ├─stroke.rs  │      │  ├─rules.rs   │      │               │
   │  ├─tone.rs    │      │  ├─phonotax.rs│      │  bidirectional│
   │  ├─mark.rs    │      │  └─pipeline.rs│      │  validation   │
   │  └─remove.rs  │      │               │      │               │
   └───────┬───────┘      └───────┬───────┘      └───────┬───────┘
           │                      │                      │
           └──────────────────────┼──────────────────────┘
                                  │
                    ┌─────────────┼─────────────┐
                    ↓             ↓             ↓
            ┌─────────────┐ ┌──────────┐ ┌─────────────┐
            │ dual_buffer │ │ syllable │ │  english    │
            │    .rs      │ │   .rs    │ │    .rs      │
            └─────────────┘ └──────────┘ └─────────────┘
                    │             │             │
                    └─────────────┼─────────────┘
                                  ↓
                          ┌──────────────┐
                          │   data/      │
                          │ ├─vowel.rs   │
                          │ ├─keys.rs    │
                          │ ├─chars.rs   │
                          │ └─constants  │
                          └──────────────┘
```

### 0.7 Typing Flow (Complete)

```
User types: "vieets" (intended: việt)

┌─────────────────────────────────────────────────────────────────────────┐
│ Step │ Key │ State    │ DualBuffer.transformed │ DualBuffer.raw        │
├─────────────────────────────────────────────────────────────────────────┤
│  1   │  v  │ Initial  │ [v]                    │ [v]                   │
│  2   │  i  │ Vowel*   │ [v,i]                  │ [v,i]                 │
│  3   │  e  │ Vowel**  │ [v,i,e]                │ [v,i,e]               │
│  4   │  e  │ Marked   │ [v,i,ê] ← circumflex   │ [v,i,e,ê̲] consumed   │
│  5   │  t  │ Final    │ [v,i,ê,t]              │ [v,i,e,ê̲,t]          │
│  6   │  s  │ Marked   │ [v,i,ế,t] ← sắc        │ [v,i,e,ê̲,t,s̲] cons.  │
├─────────────────────────────────────────────────────────────────────────┤
│ SPACE → Validate VN → VALID → Output: "việt "                          │
│         (Don't restore)                                                 │
└─────────────────────────────────────────────────────────────────────────┘

User types: "text" (transforms to "tẽt", should restore)

┌─────────────────────────────────────────────────────────────────────────┐
│ Step │ Key │ State    │ DualBuffer.transformed │ DualBuffer.raw        │
├─────────────────────────────────────────────────────────────────────────┤
│  1   │  t  │ Initial  │ [t]                    │ [t]                   │
│  2   │  e  │ Vowel*   │ [t,e]                  │ [t,e]                 │
│  3   │  x  │ Marked   │ [t,ẽ] ← ngã            │ [t,e,x̲] consumed      │
│  4   │  t  │ Invalid! │ [t,ẽ,t] ← bad final    │ [t,e,x̲,t]             │
├─────────────────────────────────────────────────────────────────────────┤
│ SPACE → Validate VN → INVALID (ẽ+t)                                     │
│       → Validate EN → VALID ("text" is English word)                    │
│       → RESTORE → Output: "text "                                       │
└─────────────────────────────────────────────────────────────────────────┘

User types: "đườngfffff" (invalid both VN and EN)

┌─────────────────────────────────────────────────────────────────────────┐
│ After typing...                                                         │
├─────────────────────────────────────────────────────────────────────────┤
│ SPACE → Validate VN → INVALID                                           │
│       → Validate EN → INVALID (no English pattern matches)              │
│       → KEEP AS-IS → Output: "đườngfffff "                              │
│         (Don't restore vô tội vạ)                                       │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 1. Current State Analysis

### 1.1 God File Problem - `mod.rs`

**Vấn đề nghiêm trọng nhất:** `mod.rs` có ~3,771 lines, gánh toàn bộ engine logic.

```
mod.rs (3,771 lines)
├── Engine struct (25+ fields, 15+ boolean flags)
├── 60+ functions
├── State management (hỗn loạn)
├── Transform pipeline (fragmented)
└── Validation calls (scattered)
```

**25+ Fields trong Engine struct:**
```rust
pub struct Engine {
    buf: Buffer,
    method: u8,
    enabled: bool,
    last_transform: Option<Transform>,
    shortcuts: ShortcutTable,
    raw_input: Vec<(u16, bool, bool)>,
    has_non_letter_prefix: bool,
    skip_w_shortcut: bool,
    esc_restore_enabled: bool,
    free_tone_enabled: bool,
    modern_tone: bool,
    english_auto_restore: bool,
    word_history: WordHistory,
    spaces_after_commit: u8,
    pending_breve_pos: Option<usize>,
    pending_u_horn_pos: Option<usize>,
    stroke_reverted: bool,
    had_mark_revert: bool,
    pending_mark_revert_pop: bool,
    had_any_transform: bool,
    had_vowel_triggered_circumflex: bool,
    shortcut_prefix: Option<char>,
    restored_pending_clear: bool,
    auto_capitalize: bool,
    pending_capitalize: bool,
    auto_capitalize_used: bool,
}
```

**15+ Boolean Flags - State Explosion:**
| Flag | Purpose | Coupling |
|------|---------|----------|
| `stroke_reverted` | Track if stroke was undone | Tight to try_stroke |
| `had_mark_revert` | Track if mark was reverted | Tight to try_mark |
| `pending_mark_revert_pop` | Deferred raw_input cleanup | Cross-function |
| `had_any_transform` | Track if VN transform applied | Auto-restore logic |
| `had_vowel_triggered_circumflex` | V+C+V circumflex tracking | Auto-restore |
| `restored_pending_clear` | Post-DELETE state | on_key_ext |
| `pending_capitalize` | Auto-cap state | on_key_ext |
| `auto_capitalize_used` | Track if auto-cap fired | DELETE handler |
| `has_non_letter_prefix` | Shortcut filtering | try_word_boundary_shortcut |
| ... | ... | ... |

### 1.2 Existing Module Structure

```
core/src/engine/
├── mod.rs          (3,771 lines - GOD FILE)
├── buffer.rs       (Buffer struct, Char struct)
├── syllable.rs     (301 lines - Syllable parser)
├── validation.rs   (575 lines - 6 validation rules)
├── transform.rs    (Transform utilities)
└── shortcut.rs     (Shortcut table)
```

**Điểm mạnh hiện tại:**
- `syllable.rs`: Clean parser với structure rõ ràng
- `validation.rs`: Rule-based approach, extensible
- `vowel.rs`: Data-driven pattern matching cho tone/horn placement

**Điểm yếu hiện tại:**
- Logic chính nằm hết trong `mod.rs`
- Boolean flags thay vì State Machine
- Validation gọi scattered, không centralized
- Raw input tracking manual, error-prone

---

## 2. Critical Architectural Weaknesses

### 2.1 Weak #1: No State Machine

**Vấn đề:** Engine dùng 15+ boolean flags thay vì explicit state machine.

**Hệ quả:**
- State transitions implicit, hard to trace
- Edge case bugs khi flags conflict
- Testing requires setting multiple flags
- New features require adding more flags

**Current Flow (Implicit States):**
```
on_key_ext()
    ├── if SPACE → try_word_boundary_shortcut() → try_auto_restore_on_space()
    ├── if ESC → restore_to_raw()
    ├── if DELETE → complex backspace logic with 5+ conditions
    └── else → process()
            ├── try_stroke() (checks stroke_reverted, last_transform)
            ├── try_tone() (checks pending_breve_pos, pending_u_horn_pos)
            ├── try_mark() (updates had_mark_revert, pending_mark_revert_pop)
            ├── try_remove()
            ├── try_w_as_vowel()
            └── handle_normal_letter()
```

### 2.2 Weak #2: Dual-Track Buffer Not Explicit

**Vấn đề:** Engine maintain `buf` (transformed) và `raw_input` (original) nhưng logic sync thủ công.

```rust
// Current: Manual sync, error-prone
self.raw_input.push((key, effective_caps, shift));
// ... later in try_stroke ...
if self.raw_input.len() >= 2 {
    let current = self.raw_input.pop();
    self.raw_input.pop(); // consumed
    if let Some(c) = current {
        self.raw_input.push(c);
    }
}
```

**Hệ quả:**
- Buffer out-of-sync khi logic complex
- restore_to_raw() có thể trả wrong chars
- Debug extremely difficult

### 2.3 Weak #3: Restore Logic Too Aggressive

**Vấn đề từ user feedback:**
> "Nên có logic tương tự cho english để validate chứ không nên restore vô tội vạ"

**Case Analysis:**

| Input | Current Behavior | Expected | Issue |
|-------|-----------------|----------|-------|
| `đườngfffff` | Restore to `duongfffff` | Keep as-is | Neither VN nor EN valid |
| `text` | `tẽt` → restore | `text` | Correct |
| `texxt` | `text` | `text` | Correct (double-key revert) |
| `vowel` | `vôel` → restore | `vowel` | Need EN validation |
| `issue` | `íue` → restore | `issue` | Correct |
| `bass` | `báss` → restore | `bass` | Need detect double consonant |

**Current Logic Gap:**
```rust
fn try_auto_restore_on_space(&mut self) -> Result {
    // Only checks if VN invalid, doesn't validate if EN valid
    if !is_valid_with_tones(&buffer_keys, &buffer_tones) {
        // Restore blindly - WRONG for "đườngfffff"
        return self.restore_to_raw();
    }
}
```

### 2.4 Weak #4: Missing Phonotactic Constraints

**Từ docs/vietnamese-language-system.md Section 6.5:**

**Tone + Stop Final Rules (MISSING):**
| Final | Valid Tones | Invalid Tones |
|-------|-------------|---------------|
| p, t, c, ch | sắc (´), nặng (.) | huyền, hỏi, ngã, ngang |

**Vowel + Final Compatibility (MISSING):**
| Vowel Pattern | Valid Finals | Invalid Finals |
|---------------|--------------|----------------|
| ă, â | m, n, p, t, c | ng, nh |
| ư | All | (specific exceptions) |

**Current validation.rs chỉ check:**
1. Has vowel
2. Valid initial
3. All chars parsed
4. Spelling rules (c/k, g/gh, ng/ngh)
5. Valid final
6. Valid vowel pattern

**KHÔNG check:**
- Tone + Final compatibility
- Vowel + Final compatibility

### 2.5 Weak #5: English Detection Heuristic-Only

**Current `is_foreign_word_pattern()`:**
```rust
// Check 1: Invalid vowel patterns (ou, yo - not in whitelist)
// Check 2: Consonant clusters (T+R, P+R, C+R after finals)
// Check 3: English prefix (de + s)
// Check 4: REMOVED (too aggressive)
// Check 5: Invalid final consonant + mark modifier
```

**Missing:**
- No morphological analysis
- No dictionary lookup option
- Pattern-only approach has false positives/negatives

---

## 3. Edge Cases Deep Analysis

### 3.1 Typing Behavior Matrix

**User có thể gõ theo nhiều kiểu:**

| Behavior | Example | Engine Challenge |
|----------|---------|-----------------|
| **Dấu sau (Standard)** | `viết` = `viet` + `s` | Normal flow |
| **Dấu trước (Fast)** | `viết` = `vs` + `i` + `e` + `t` | Deferred tone placement |
| **Mixed** | `được` = `d` + `ươ` + `s` + `c` | Partial context |
| **Error correction** | `vieets` → backspace → `viets` | State rollback |
| **Double-key revert** | `vieett` = `viết` | Modifier consumed |

### 3.2 Diacritic Placement Edge Cases

**Case 1: "ua" context-dependent**
```
mua  → dấu trên u (u là âm chính, a là glide) → mùa
qua  → dấu trên a (u là phần của qu)         → quà
chuẩn → dấu trên â (u là âm đệm, â là chính)  → chuẩn
```

**Case 2: "ươ" compound**
```
uo + w (no final) → huơ  (chỉ o có horn)
uo + w + c        → dược (cả u và o có horn)
```

**Case 3: Deferred breve**
```
a + w (open syllable) → aw (defer breve, invalid "ă" alone)
aw + n               → ăn  (apply breve, valid "ăn")
```

### 3.3 Restore Decision Matrix (Refined)

| Condition | VN Valid | EN Valid | Action | Example |
|-----------|----------|----------|--------|---------|
| Transform applied | ✓ | - | Keep | `việt` |
| Transform applied | ✗ | ✓ | Restore | `text` (tẽt → text) |
| Transform applied | ✗ | ✗ | **Keep as-is** | `đườngfffff` |
| No transform | - | - | Keep | `hello` |
| Double-key revert | - | - | Already handled | `texxt` → `text` |

### 3.4 English Validation Levels

**Level 0: Pattern-Only (~2KB)**
```rust
const EN_INVALID_PATTERNS: &[&str] = &[
    "xq", "qx", "zx", "xz", // Impossible combos
];
const EN_COMMON_PATTERNS: &[&str] = &[
    "tion", "ing", "ed", "ly", "ness", // Suffixes
    "str", "chr", "thr", "sch",        // Consonant clusters
];
```

**Level 1: Morphology (~5KB)**
```rust
// Check affixes
fn has_english_affixes(word: &str) -> bool {
    let prefixes = ["un", "re", "pre", "dis", "mis"];
    let suffixes = ["ing", "ed", "ly", "ness", "tion", "ment"];
    // ...
}
```

**Level 2: Bloom Filter (~12KB for 10K words)**
```rust
// Probabilistic dictionary check
struct BloomFilter {
    bits: [u64; 192],  // ~12KB
    k: u8,             // Hash functions
}
impl BloomFilter {
    fn probably_contains(&self, word: &str) -> bool { ... }
}
```

**Level 3: FST Dictionary (~20-25KB)**
```rust
// Exact match with FST compression
// Only if Level 2 insufficient
```

**Recommended: Level 0 + Level 1 (default) + Level 2 (opt-in)**

---

## 4. Proposed Architecture Improvements

### 4.1 State Machine Design

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
enum EngineState {
    /// Buffer empty, awaiting input
    Empty,
    /// Initial consonant(s) entered, no vowel yet
    Initial,
    /// First vowel entered
    VowelStart,
    /// Multiple vowels (diphthong/triphthong potential)
    VowelCompound,
    /// After final consonant
    Final,
    /// Diacritic applied, awaiting more input or commit
    Marked,
    /// Invalid state (foreign word detected)
    Foreign,
}

#[derive(Clone, Copy, Debug)]
enum Transition {
    AddInitial(u16),
    AddVowel(u16),
    AddFinal(u16),
    ApplyTone(u8),
    ApplyMark(u8),
    ApplyStroke,
    Revert,
    Reset,
}

impl Engine {
    fn transition(&mut self, t: Transition) -> Result {
        let (new_state, result) = match (self.state, t) {
            (Empty, AddInitial(k)) => (Initial, self.push_initial(k)),
            (Empty, AddVowel(k)) => (VowelStart, self.push_vowel(k)),
            (Initial, AddVowel(k)) => (VowelStart, self.push_vowel(k)),
            (VowelStart, AddVowel(k)) => (VowelCompound, self.push_vowel(k)),
            (VowelStart, ApplyTone(t)) => (Marked, self.apply_tone(t)),
            (VowelCompound, AddFinal(k)) => (Final, self.push_final(k)),
            // ... explicit transitions
            _ => return self.handle_unexpected(),
        };
        self.state = new_state;
        result
    }
}
```

### 4.2 Dual-Track Buffer Abstraction

```rust
/// Synchronized dual-track buffer
struct DualBuffer {
    /// Transformed Vietnamese buffer
    transformed: Buffer,
    /// Original raw keystrokes
    raw: Vec<RawKeystroke>,
    /// Sync invariant: len(transformed) may differ from len(raw)
    /// but restore() always reconstructs correctly
}

struct RawKeystroke {
    key: u16,
    caps: bool,
    shift: bool,
    consumed: bool,  // True if key was consumed by modifier
}

impl DualBuffer {
    fn push(&mut self, key: u16, caps: bool, shift: bool) {
        self.raw.push(RawKeystroke { key, caps, shift, consumed: false });
        // transformed updated by caller based on modifier logic
    }

    fn mark_consumed(&mut self, idx: usize) {
        if let Some(k) = self.raw.get_mut(idx) {
            k.consumed = true;
        }
    }

    fn restore(&self) -> Vec<char> {
        self.raw.iter()
            .filter(|k| !k.consumed)
            .filter_map(|k| key_to_char(k.key, k.caps))
            .collect()
    }

    fn pop(&mut self) {
        self.transformed.pop();
        // Find last non-consumed raw keystroke
        while let Some(k) = self.raw.last() {
            if k.consumed {
                self.raw.pop();
            } else {
                self.raw.pop();
                break;
            }
        }
    }
}
```

### 4.3 Bidirectional Validation Pipeline

```rust
/// Validation result with language detection
enum LanguageValidation {
    ValidVietnamese,
    InvalidVietnameseValidEnglish,
    InvalidVietnameseInvalidEnglish,
    NoTransformApplied,
}

impl Engine {
    fn validate_for_restore(&self) -> LanguageValidation {
        // Fast path: no transform applied
        if !self.dual_buffer.had_any_transform() {
            return LanguageValidation::NoTransformApplied;
        }

        let vn_valid = self.validate_vietnamese();
        if vn_valid {
            return LanguageValidation::ValidVietnamese;
        }

        let en_valid = self.validate_english();
        if en_valid {
            LanguageValidation::InvalidVietnameseValidEnglish
        } else {
            LanguageValidation::InvalidVietnameseInvalidEnglish
        }
    }

    fn decide_restore(&self) -> bool {
        match self.validate_for_restore() {
            LanguageValidation::InvalidVietnameseValidEnglish => true,
            _ => false,  // Keep as-is for all other cases
        }
    }
}
```

### 4.4 Enhanced Validation Rules

```rust
// Add to validation.rs

/// Rule 7: Tone + Stop Final compatibility
fn rule_tone_stop_final(snap: &BufferSnapshot, syllable: &Syllable) -> Option<ValidationResult> {
    if syllable.final_c.is_empty() {
        return None;
    }

    let final_keys: Vec<u16> = syllable.final_c.iter().map(|&i| snap.keys[i]).collect();

    // Check if final is a stop consonant (p, t, c, ch)
    let is_stop = matches!(
        final_keys.as_slice(),
        [keys::P] | [keys::T] | [keys::C] | [keys::C, keys::H]
    );

    if !is_stop {
        return None;
    }

    // Stop finals only allow sắc (´) or nặng (.)
    let tone_mark = syllable.vowel.iter()
        .find_map(|&i| {
            let mark = snap.marks.get(i).copied().unwrap_or(0);
            if mark > 0 { Some(mark) } else { None }
        });

    if let Some(mark) = tone_mark {
        // Only sắc (1) and nặng (5) are valid with stops
        if mark != mark::SAC && mark != mark::NANG {
            return Some(ValidationResult::InvalidToneFinalCombo);
        }
    }

    None
}

/// Rule 8: Vowel + Final compatibility
fn rule_vowel_final_compat(snap: &BufferSnapshot, syllable: &Syllable) -> Option<ValidationResult> {
    // Short vowels (ă, â) cannot precede ng, nh
    // Implementation based on phonotactic constraints
    // ...
    None
}
```

### 4.5 English Validation Module

```rust
// New file: core/src/engine/english.rs

/// Pattern-based English validation (Level 0 + 1)
pub struct EnglishValidator {
    /// Common English patterns
    patterns: &'static [&'static str],
    /// Invalid character sequences
    invalid_seqs: &'static [&'static str],
}

impl EnglishValidator {
    pub fn is_possibly_english(&self, word: &str) -> bool {
        let lower = word.to_lowercase();

        // Check invalid sequences first (fast reject)
        for seq in self.invalid_seqs {
            if lower.contains(seq) {
                return false;
            }
        }

        // Check for common English patterns
        for pattern in self.patterns {
            if lower.contains(pattern) {
                return true;
            }
        }

        // Check morphology
        self.has_english_morphology(&lower)
    }

    fn has_english_morphology(&self, word: &str) -> bool {
        const PREFIXES: &[&str] = &["un", "re", "pre", "dis", "mis", "over", "under"];
        const SUFFIXES: &[&str] = &["ing", "ed", "ly", "ness", "tion", "ment", "able", "ible"];

        for prefix in PREFIXES {
            if word.starts_with(prefix) && word.len() > prefix.len() + 2 {
                return true;
            }
        }

        for suffix in SUFFIXES {
            if word.ends_with(suffix) && word.len() > suffix.len() + 2 {
                return true;
            }
        }

        false
    }
}

/// Optional: Bloom filter for dictionary check
#[cfg(feature = "english_dictionary")]
pub struct BloomDictionary {
    bits: Vec<u64>,
    hash_count: u8,
}
```

---

## 5. Implementation Phases

### Phase 1: Foundation (Low Risk)
1. Add `rule_tone_stop_final` to validation.rs
2. Add `rule_vowel_final_compat` to validation.rs
3. Create `english.rs` with pattern-based validation
4. Update `try_auto_restore_on_space()` to use bidirectional validation

### Phase 2: Abstraction (Medium Risk)
1. Create `DualBuffer` struct
2. Migrate `buf` + `raw_input` to `DualBuffer`
3. Update all transform functions to use `DualBuffer`

### Phase 3: State Machine (Higher Risk)
1. Define `EngineState` enum
2. Implement `transition()` method
3. Replace boolean flags with state
4. Extensive regression testing

### Phase 4: Modularization (Refactoring)
1. Extract transform logic from `mod.rs`
2. Create `stroke.rs`, `tone.rs`, `mark.rs` modules
3. Reduce `mod.rs` to orchestration only

---

## 6. Memory Budget Analysis

### 6.1 Current Memory Analysis

**GOOD (Already Stack-Allocated):**
```rust
// buffer.rs - Already optimized
pub struct Char {              // ~8 bytes (with alignment)
    key: u16,
    caps: bool,
    tone: u8,
    mark: u8,
    stroke: bool,
}
pub struct Buffer {            // ~520 bytes (stack)
    data: [Char; 64],          // 64 × 8 = 512
    len: usize,                // 8
}
// WordHistory - Ring buffer, stack
struct WordHistory {           // ~5KB (stack)
    data: [Buffer; 10],        // 10 × 520 = 5,200
    head: usize,
    len: usize,
}
```

**BAD (Heap-Allocated - NEEDS FIX):**
```rust
// mod.rs line 202 - HEAP ALLOCATION PER KEYSTROKE
raw_input: Vec<(u16, bool, bool)>,  // Vec = heap!
```

**Memory Impact:**
| Component | Current | Target |
|-----------|---------|--------|
| Buffer | 520B stack ✓ | Keep |
| WordHistory | ~5KB stack ✓ | Keep |
| raw_input | Vec (heap) ✗ | Fixed array |
| ShortcutTable | ~2KB heap | Acceptable (user config) |

### 6.1.1 Static Memory Budget

| Component | Size | Notes |
|-----------|------|-------|
| Engine base | ~500 bytes | Current |
| DualBuffer | ~1KB | Buffer + raw history |
| English patterns | ~2KB | Level 0 + 1 |
| Bloom filter (opt) | ~12KB | 10K words, 0.1% FP |
| Total | ~3.5KB default | ~15KB with dictionary |

### 6.2 Runtime Memory Optimization (CRITICAL: Must be minimal)

**Philosophy: "App có thể nặng, nhưng khi chạy thì phải nhẹ"**

#### 6.2.1 Stack-First Design

```rust
// BAD: Heap allocation every keystroke
struct Engine {
    buffer: Vec<Char>,           // Heap
    raw_input: Vec<RawKeystroke>, // Heap
}

// GOOD: Fixed stack arrays
struct Engine {
    buffer: [Char; 32],          // Stack - max syllable ~12 chars + margin
    buffer_len: u8,              // 1 byte
    raw_input: [RawKeystroke; 48], // Stack - max raw + modifiers
    raw_len: u8,                 // 1 byte
}
// Total: ~200 bytes stack, ZERO heap allocation per keystroke
```

#### 6.2.2 Compact Data Structures

```rust
// Current RawKeystroke (potential)
struct RawKeystroke {
    key: u16,      // 2 bytes
    caps: bool,    // 1 byte (but aligned to 4)
    shift: bool,   // 1 byte (aligned)
    consumed: bool, // 1 byte (aligned)
}
// = 8 bytes after alignment

// Optimized: Pack into 4 bytes
struct RawKeystroke {
    key: u16,      // 2 bytes
    flags: u8,     // 1 byte: bit0=caps, bit1=shift, bit2=consumed
    _pad: u8,      // alignment
}
// = 4 bytes, 50% savings

// Even better: Pack multiple keystrokes
// 48 keystrokes × 4 bytes = 192 bytes (vs 384 bytes unoptimized)
```

#### 6.2.3 Zero-Allocation Hot Path

```rust
impl Engine {
    // CRITICAL: on_key_ext() MUST NOT allocate
    pub fn on_key_ext(&mut self, key: u16, caps: bool, shift: bool, alt: bool) -> Result {
        // No Vec::push(), no String, no Box
        // Only mutate pre-allocated buffers

        if self.buffer_len >= 32 {
            return Result::RejectOverflow;
        }

        self.buffer[self.buffer_len as usize] = Char::from_key(key);
        self.buffer_len += 1;

        // ... transform logic using stack-local variables only
    }
}
```

#### 6.2.4 Lazy Loading for Optional Features

```rust
// English dictionary (opt-in) - load only when needed
static BLOOM_FILTER: OnceLock<BloomDictionary> = OnceLock::new();

impl Engine {
    fn validate_english(&self) -> bool {
        // Pattern check first (always available, zero cost)
        if self.pattern_check_english() {
            return true;
        }

        // Bloom filter only if feature enabled AND pattern failed
        #[cfg(feature = "english_dictionary")]
        {
            let bloom = BLOOM_FILTER.get_or_init(|| {
                // Load once, live forever
                BloomDictionary::from_embedded()
            });
            return bloom.probably_contains(&self.current_word());
        }

        false
    }
}
```

#### 6.2.5 Memory Layout Summary

```
┌─────────────────────────────────────────────────────────────────┐
│                     Engine Memory Layout                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  STACK (per-instance, ~500 bytes)                               │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │ buffer: [Char; 32]     │ 32 × 8 = 256 bytes                │ │
│  │ raw:    [Raw; 48]      │ 48 × 4 = 192 bytes                │ │
│  │ state:  EngineState    │ 1 byte (enum)                     │ │
│  │ flags:  u8             │ 1 byte (packed bools)             │ │
│  │ config: EngineConfig   │ 8 bytes                           │ │
│  │ lens, etc              │ ~40 bytes                         │ │
│  └────────────────────────────────────────────────────────────┘ │
│                                                                  │
│  STATIC (compile-time, shared across instances)                 │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │ VOWEL_PATTERNS        │ ~800 bytes (vowel.rs data)         │ │
│  │ EN_PATTERNS           │ ~200 bytes (english patterns)      │ │
│  │ VALIDATION_TABLES     │ ~500 bytes (lookup tables)         │ │
│  │ KEY_MAPS              │ ~400 bytes (keycode tables)        │ │
│  └────────────────────────────────────────────────────────────┘ │
│                                                                  │
│  HEAP (optional, lazy-loaded)                                   │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │ BloomDictionary       │ ~12KB (opt-in, load once)          │ │
│  │ ShortcutTable         │ ~2KB (user shortcuts)              │ │
│  │ WordHistory           │ ~1KB (recent words)                │ │
│  └────────────────────────────────────────────────────────────┘ │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘

Runtime Memory Per Keystroke:
• Stack temporaries: ~64 bytes (function locals)
• Heap allocations: ZERO
• CPU cache friendly: All hot data fits in L1 cache (32KB)

Total App Memory Target:
• Engine core: ~3KB (stack + static)
• Optional dictionary: ~12KB (lazy)
• macOS UI layer: ~2MB (SwiftUI overhead)
• Total: <5MB RAM at runtime
```

#### 6.2.6 Benchmark Targets

| Metric | Target | Current | Notes |
|--------|--------|---------|-------|
| Keystroke latency | <1ms | ✓ | Hot path |
| Memory per keystroke | 0 bytes heap | TBD | No alloc |
| Engine instance size | <1KB | ~500B | Stack-based |
| Static data | <2KB | ~2KB | Patterns + tables |
| Optional features | lazy-load | ✓ | Bloom filter |
| Total app RAM | <5MB | ~4MB | macOS target |

#### 6.2.7 Forbidden Patterns in Hot Path

```rust
// ❌ NEVER do these in on_key_ext() or process()
String::from("...")           // Heap alloc
vec![...]                     // Heap alloc
Box::new(...)                 // Heap alloc
format!(...)                  // Heap alloc + formatting
.to_string()                  // Heap alloc
.clone() on Vec/String        // Heap alloc
.collect::<Vec<_>>()          // Heap alloc

// ✓ ALLOWED
&str (references only)        // Zero-copy
[T; N] (fixed arrays)         // Stack
&[T] (slices)                 // Zero-copy
Copy types                    // Stack
unsafe { ... } (if needed)    // Manual memory
```

**Target: <5MB RAM total app (engine < 3KB stack, <15KB with lazy features)**

### 6.3 Immediate Fix: raw_input Vec → Fixed Array

**Current Issue (mod.rs:202):**
```rust
raw_input: Vec<(u16, bool, bool)>,  // Heap alloc every push!
```

**Fix:**
```rust
const RAW_MAX: usize = 96;  // Max raw keystrokes per word

struct RawInput {
    data: [(u16, u8); RAW_MAX],  // u8 = packed flags (caps|shift|consumed)
    len: u8,
}

impl RawInput {
    #[inline]
    fn push(&mut self, key: u16, caps: bool, shift: bool) {
        if (self.len as usize) < RAW_MAX {
            let flags = (caps as u8) | ((shift as u8) << 1);
            self.data[self.len as usize] = (key, flags);
            self.len += 1;
        }
    }

    #[inline]
    fn pop(&mut self) -> Option<(u16, bool, bool)> {
        if self.len > 0 {
            self.len -= 1;
            let (key, flags) = self.data[self.len as usize];
            Some((key, flags & 1 != 0, flags & 2 != 0))
        } else {
            None
        }
    }
}
// Memory: 96 × 3 = 288 bytes (stack) vs Vec ~24 bytes + heap allocs
```

**Impact:**
- Before: Vec heap alloc on every push → GC pressure, cache misses
- After: Zero heap allocs in hot path → L1 cache friendly
- Total engine size: ~6KB stack (Buffer + WordHistory + RawInput)

---

## 7. Testing Strategy

### 7.1 State Machine Tests
```rust
#[test]
fn test_state_transitions() {
    let mut engine = Engine::new();
    assert_eq!(engine.state, EngineState::Empty);

    engine.process_key(keys::V, false, false);
    assert_eq!(engine.state, EngineState::Initial);

    engine.process_key(keys::I, false, false);
    assert_eq!(engine.state, EngineState::VowelStart);

    engine.process_key(keys::E, false, false);
    assert_eq!(engine.state, EngineState::VowelCompound);

    engine.process_key(keys::S, false, false); // sắc
    assert_eq!(engine.state, EngineState::Marked);
}
```

### 7.2 Bidirectional Validation Tests
```rust
#[test]
fn test_restore_decision_matrix() {
    // VN valid → keep
    assert!(!should_restore("việt"));

    // VN invalid, EN valid → restore
    assert!(should_restore("tẽt")); // text

    // VN invalid, EN invalid → keep as-is
    assert!(!should_restore("đườngfffff"));

    // No transform → keep
    assert!(!should_restore("hello"));
}
```

### 7.3 Edge Case Tests
```rust
#[test]
fn test_ua_context() {
    // mùa (open syllable, u is main)
    assert_tone_position("mua", 1); // tone on u

    // quà (qu-initial, a is main)
    assert_tone_position("qua", 2); // tone on a

    // chuẩn (closed syllable, â is main)
    assert_tone_position("chuan", 3); // tone on â
}

#[test]
fn test_uo_compound() {
    // huơ (no final) → only o gets horn
    assert_horn_positions("huow", &[2]); // o at pos 2

    // dược (with final) → both get horn
    assert_horn_positions("duowc", &[1, 2]); // u at 1, o at 2
}
```

---

## 8. Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| State machine migration breaks existing | High | Extensive regression tests, gradual rollout |
| DualBuffer sync bugs | Medium | Strong invariants, panic on desync |
| English validation false positives | Low | Pattern-only default, opt-in dictionary |
| Performance regression | Low | Benchmark before/after each phase |

---

## 9. Open Questions

1. **Dictionary size vs accuracy**: Bloom filter 10K words đủ không? Hay cần 50K+?
2. **Restore timing**: Restore on SPACE hay on break key? Hay cả hai?
3. **User preference**: Có cần option "never auto-restore"?
4. **Edge case priority**: Những edge case nào cần fix first?

---

## 10. Conclusion

**3 vấn đề cốt lõi cần giải quyết:**
1. **God File** → Modularize với clear boundaries
2. **State Explosion** → Explicit State Machine
3. **Aggressive Restore** → Bidirectional Validation

**Recommended order:**
1. Phase 1 (validation rules + bidirectional) - Immediate value, low risk
2. Phase 2 (DualBuffer) - Foundation for future
3. Phase 3 (State Machine) - Major refactor
4. Phase 4 (Modularization) - Long-term maintainability

**Success metrics:**
- `mod.rs` < 500 lines
- Boolean flags < 5
- Test coverage > 95%
- No false restore (đườngfffff stays as-is)
- <1ms latency maintained
