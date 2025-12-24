# Phase 3: State Machine & Modularization

**Duration:** Week 5-8
**Branch:** `feature/engine-v2-phase3`
**Risk Level:** HIGH
**Prerequisite:** Phase 1 + Phase 2 complete

---

## Objectives

1. Implement EngineState enum with explicit transitions
2. Split try_tone() (606 LOC) into modular components
3. Split try_mark() (465 LOC) into modular components
4. Consolidate 11 boolean flags into typed state
5. Reduce mod.rs from 3,917 to <500 LOC

---

## Task 3.1: Implement EngineState

### Design

Vietnamese syllable follows pattern: **(C₁)(G)V(C₂)+T**

Map this to state machine:

```
Empty → Initial → VowelStart → VowelCompound → Final → Marked → Foreign
```

### Implementation

**File:** `core/src/engine/state.rs` (NEW)

```rust
//! Engine state machine for Vietnamese syllable processing
//!
//! States map to syllable structure: (C₁)(G)V(C₂)+T
//! - Empty: No input yet
//! - Initial: Initial consonant (C₁) or cluster
//! - VowelStart: First vowel (V)
//! - VowelCompound: Diphthong/triphthong (VV, VVV)
//! - Final: Final consonant (C₂)
//! - Marked: Tone or vowel mark applied
//! - Foreign: Invalid Vietnamese pattern

/// Input key classification
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyType {
    /// Initial consonants: b, c, ch, d, đ, g, gh, h, k, kh, l, m, n, nh, p, ph, q, r, s, t, th, tr, v, x
    InitialConsonant,

    /// Vowels: a, ă, â, e, ê, i, o, ô, ơ, u, ư, y
    Vowel,

    /// Glide: Semi-vowels that can follow initial (u in "qu", o in "ho")
    Glide,

    /// Final consonants: c, ch, m, n, ng, nh, p, t
    FinalConsonant,

    /// Tone modifiers: s, f, r, x, j (Telex) or 1-5 (VNI)
    ToneMark,

    /// Vowel marks: a→ă (w), a→â (aa/^), etc.
    VowelMark,

    /// Stroke: d→đ
    Stroke,

    /// Invalid for Vietnamese
    Invalid,
}

/// Engine processing state
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum EngineState {
    /// No input (initial state)
    #[default]
    Empty,

    /// Initial consonant typed (b, c, d, etc.)
    Initial,

    /// First vowel typed (a, e, i, o, u, y)
    VowelStart,

    /// Multiple vowels (diphthong: ai, uo; triphthong: uoi)
    VowelCompound,

    /// Final consonant typed (m, n, ng, c, t, p, ch, nh)
    Final,

    /// Diacritic applied (tone or vowel mark)
    Marked,

    /// Invalid Vietnamese pattern detected
    Foreign,
}

impl EngineState {
    /// Transition to next state based on key type
    ///
    /// Returns (new_state, is_valid_transition)
    pub fn transition(self, key_type: KeyType) -> (Self, bool) {
        use EngineState::*;
        use KeyType::*;

        match (self, key_type) {
            // From Empty
            (Empty, InitialConsonant) => (Initial, true),
            (Empty, Vowel) => (VowelStart, true),
            (Empty, Stroke) => (Initial, true), // đ

            // From Initial (consonant)
            (Initial, Vowel) => (VowelStart, true),
            (Initial, Glide) => (Initial, true), // qu, gi
            (Initial, InitialConsonant) => (Initial, true), // ch, ng, etc.
            (Initial, Stroke) => (Initial, true), // dd→đ

            // From VowelStart
            (VowelStart, Vowel) => (VowelCompound, true),
            (VowelStart, FinalConsonant) => (Final, true),
            (VowelStart, ToneMark) => (Marked, true),
            (VowelStart, VowelMark) => (Marked, true),
            (VowelStart, InitialConsonant) => {
                // Could be final (m, n) or new word
                // Need context to determine
                (Final, true)
            }

            // From VowelCompound
            (VowelCompound, Vowel) => (VowelCompound, true), // triphthong
            (VowelCompound, FinalConsonant) => (Final, true),
            (VowelCompound, ToneMark) => (Marked, true),
            (VowelCompound, VowelMark) => (Marked, true),

            // From Final
            (Final, ToneMark) => (Marked, true),
            (Final, VowelMark) => (Marked, true),
            (Final, FinalConsonant) => {
                // Could be ng, nh, ch
                (Final, true)
            }

            // From Marked
            (Marked, ToneMark) => (Marked, true), // Tone change
            (Marked, VowelMark) => (Marked, true), // Mark change

            // Invalid transitions → Foreign
            (_, Invalid) => (Foreign, false),
            (Final, Vowel) => (Foreign, false), // Vowel after final
            (Final, InitialConsonant) => (Foreign, false), // Non-final consonant after final

            // Foreign state
            (Foreign, InitialConsonant) => (Initial, true), // Reset on new consonant
            (Foreign, Vowel) => (Foreign, false), // Stay foreign
            (Foreign, _) => (Foreign, false),

            // Catch-all: stay in current state
            _ => (self, true),
        }
    }

    /// Check if current state allows word completion
    pub fn is_completable(&self) -> bool {
        matches!(
            self,
            EngineState::VowelStart
                | EngineState::VowelCompound
                | EngineState::Final
                | EngineState::Marked
        )
    }

    /// Check if we're in an error state
    pub fn is_error(&self) -> bool {
        matches!(self, EngineState::Foreign)
    }

    /// Reset to empty
    pub fn reset(&mut self) {
        *self = EngineState::Empty;
    }
}

/// Classify a key into KeyType
pub fn classify_key(key: u16, method: InputMethod) -> KeyType {
    use KeyType::*;

    // Check for tone modifiers first (method-specific)
    if is_tone_key(key, method) {
        return ToneMark;
    }

    // Check for vowel marks
    if is_vowel_mark_key(key, method) {
        return VowelMark;
    }

    // Check for stroke
    if is_stroke_key(key) {
        return Stroke;
    }

    // Check vowels
    if is_vowel(key) {
        return Vowel;
    }

    // Check final consonants
    if is_final_consonant(key) {
        return FinalConsonant;
    }

    // Check initial consonants
    if is_initial_consonant(key) {
        return InitialConsonant;
    }

    Invalid
}

fn is_tone_key(key: u16, method: InputMethod) -> bool {
    match method {
        InputMethod::Telex => {
            matches!(key, 0x73 | 0x66 | 0x72 | 0x78 | 0x6A | 0x7A)
            // s, f, r, x, j, z
        }
        InputMethod::VNI => {
            matches!(key, 0x31..=0x35 | 0x36..=0x39)
            // 1-5, 6-9
        }
    }
}

fn is_vowel_mark_key(key: u16, method: InputMethod) -> bool {
    match method {
        InputMethod::Telex => {
            matches!(key, 0x77 | 0x5E)
            // w, ^
        }
        InputMethod::VNI => {
            matches!(key, 0x36..=0x38)
            // 6, 7, 8
        }
    }
}

fn is_stroke_key(key: u16) -> bool {
    key == 0x64 // d
}

fn is_vowel(key: u16) -> bool {
    matches!(
        key,
        0x61 | 0x65 | 0x69 | 0x6F | 0x75 | 0x79 // a, e, i, o, u, y
    )
}

fn is_final_consonant(key: u16) -> bool {
    matches!(
        key,
        0x63 | 0x6D | 0x6E | 0x70 | 0x74 // c, m, n, p, t
    )
}

fn is_initial_consonant(key: u16) -> bool {
    matches!(
        key,
        0x62 | 0x63 | 0x64 | 0x67 | 0x68 | 0x6B | 0x6C | 0x6D | 0x6E
            | 0x70 | 0x71 | 0x72 | 0x73 | 0x74 | 0x76 | 0x78
        // b, c, d, g, h, k, l, m, n, p, q, r, s, t, v, x
    )
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputMethod {
    Telex,
    VNI,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_word_transitions() {
        // "việt" = v + i + ê + t
        let mut state = EngineState::Empty;

        let (next, valid) = state.transition(KeyType::InitialConsonant); // v
        assert!(valid);
        state = next;
        assert_eq!(state, EngineState::Initial);

        let (next, valid) = state.transition(KeyType::Vowel); // i
        assert!(valid);
        state = next;
        assert_eq!(state, EngineState::VowelStart);

        let (next, valid) = state.transition(KeyType::VowelMark); // ê (circumflex)
        assert!(valid);
        state = next;
        assert_eq!(state, EngineState::Marked);

        let (next, valid) = state.transition(KeyType::FinalConsonant); // t
        // After mark, final should keep Marked state
        // (This might need adjustment based on exact semantics)
        state = next;
        assert!(state.is_completable());
    }

    #[test]
    fn test_invalid_becomes_foreign() {
        let mut state = EngineState::Final;

        // Vowel after final = invalid
        let (next, valid) = state.transition(KeyType::Vowel);
        assert!(!valid);
        assert_eq!(next, EngineState::Foreign);
    }

    #[test]
    fn test_foreign_resets_on_consonant() {
        let state = EngineState::Foreign;

        let (next, valid) = state.transition(KeyType::InitialConsonant);
        assert!(valid);
        assert_eq!(next, EngineState::Initial);
    }
}
```

### Acceptance Criteria

- [ ] EngineState enum with 7 states
- [ ] KeyType classification
- [ ] Transition function with valid/invalid tracking
- [ ] Input method awareness (Telex/VNI)
- [ ] 10+ unit tests
- [ ] Documentation of transition table

---

## Task 3.2: Split try_tone() (606 LOC)

### Current Structure Analysis

```
try_tone() responsibilities:
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
```

### New Module Structure

```
core/src/engine/modifiers/
├── mod.rs              # Modifier trait + dispatcher
├── tone.rs             # Core tone logic (<200 LOC)
├── tone_telex.rs       # Telex-specific (<100 LOC)
├── tone_vni.rs         # VNI-specific (<50 LOC)
└── tone_placement.rs   # Vowel target selection (<100 LOC)
```

### Implementation

**File:** `core/src/engine/modifiers/mod.rs`

```rust
//! Modifier system for Vietnamese diacritics
//!
//! Handles: tones, vowel marks (circumflex, breve, horn), stroke

pub mod tone;
pub mod tone_telex;
pub mod tone_vni;
pub mod tone_placement;
pub mod mark;
pub mod stroke;

use crate::engine::{DualBuffer, EngineState, ImeResult};

/// Result of applying a modifier
#[derive(Debug)]
pub enum ModifyResult {
    /// Modifier applied successfully
    Applied(ImeResult),

    /// Same key pressed twice - revert
    Reverted(ImeResult),

    /// Modifier not applicable (passthrough)
    NotApplicable,

    /// Invalid state for modifier
    Invalid,
}

/// Trait for modifier implementations
pub trait Modifier {
    /// Try to apply this modifier
    fn apply(
        &self,
        buffer: &mut DualBuffer,
        state: &mut EngineState,
        key: u16,
        shift: bool,
    ) -> ModifyResult;

    /// Check if this modifier handles the given key
    fn handles(&self, key: u16) -> bool;
}
```

**File:** `core/src/engine/modifiers/tone.rs`

```rust
//! Core tone application logic
//!
//! Tones: sắc (´), huyền (`), hỏi (?), ngã (~), nặng (.)

use super::{Modifier, ModifyResult};
use crate::engine::{DualBuffer, EngineState};
use crate::engine::validation;

/// Tone types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tone {
    Sac,    // ´ (rising)
    Huyen,  // ` (falling)
    Hoi,    // ? (dipping)
    Nga,    // ~ (creaky rising)
    Nang,   // . (heavy)
}

impl Tone {
    /// Get tone from Telex key
    pub fn from_telex(key: u16) -> Option<Self> {
        match key {
            0x73 => Some(Tone::Sac),   // s
            0x66 => Some(Tone::Huyen), // f
            0x72 => Some(Tone::Hoi),   // r
            0x78 => Some(Tone::Nga),   // x
            0x6A => Some(Tone::Nang),  // j
            _ => None,
        }
    }

    /// Get tone from VNI key
    pub fn from_vni(key: u16) -> Option<Self> {
        match key {
            0x31 => Some(Tone::Sac),   // 1
            0x32 => Some(Tone::Huyen), // 2
            0x33 => Some(Tone::Hoi),   // 3
            0x34 => Some(Tone::Nga),   // 4
            0x35 => Some(Tone::Nang),  // 5
            _ => None,
        }
    }
}

/// Find the best vowel position for tone placement
///
/// Vietnamese tone placement rules:
/// 1. If single vowel: apply to that vowel
/// 2. If diphthong/triphthong: apply to main vowel
/// 3. Main vowel rules:
///    - oa, oe, uy: apply to second vowel
///    - ai, ao, au, ay, eo, eu, iu, oi, ou, ui: apply to first vowel
///    - Special: ua→ưa, uo→ươ consider the marked vowel
pub fn find_tone_target(buffer: &DualBuffer) -> Option<usize> {
    let chars = buffer.transformed();

    // Find all vowel positions
    let vowel_positions: Vec<usize> = chars
        .iter()
        .enumerate()
        .filter(|(_, c)| c.is_vowel())
        .map(|(i, _)| i)
        .collect();

    match vowel_positions.len() {
        0 => None,
        1 => Some(vowel_positions[0]),
        _ => {
            // Multiple vowels: use placement rules
            super::tone_placement::select_main_vowel(buffer, &vowel_positions)
        }
    }
}

/// Apply tone to buffer at position
pub fn apply_tone(buffer: &mut DualBuffer, pos: usize, tone: Tone) -> bool {
    let char = match buffer.transformed().get(pos) {
        Some(c) => c,
        None => return false,
    };

    // Check if same tone already applied
    if char.tone() == Some(tone) {
        return false; // Will trigger revert
    }

    buffer.apply_transform(pos, Transform::ToneAdd(tone));
    true
}

/// Check if applying tone would be valid
pub fn validate_tone_application(buffer: &DualBuffer, pos: usize, tone: Tone) -> bool {
    // Check Rule 7: Stop finals only allow sắc/nặng
    if let Some(final_c) = buffer.transformed().final_consonant() {
        if !validation::rule_tone_stop_final_compatibility(Some(&final_c), Some(tone)) {
            return false;
        }
    }

    true
}

pub struct ToneModifier {
    method: InputMethod,
}

impl ToneModifier {
    pub fn new(method: InputMethod) -> Self {
        Self { method }
    }

    fn get_tone(&self, key: u16) -> Option<Tone> {
        match self.method {
            InputMethod::Telex => Tone::from_telex(key),
            InputMethod::VNI => Tone::from_vni(key),
        }
    }
}

impl Modifier for ToneModifier {
    fn apply(
        &self,
        buffer: &mut DualBuffer,
        state: &mut EngineState,
        key: u16,
        shift: bool,
    ) -> ModifyResult {
        let tone = match self.get_tone(key) {
            Some(t) => t,
            None => return ModifyResult::NotApplicable,
        };

        // Find target vowel
        let pos = match find_tone_target(buffer) {
            Some(p) => p,
            None => return ModifyResult::Invalid,
        };

        // Check for revert (same tone twice)
        if buffer.transformed().get(pos).map(|c| c.tone()) == Some(Some(tone)) {
            // Revert logic
            buffer.apply_transform(pos, Transform::ToneRemove);
            buffer.pop_raw(); // Remove modifier key
            return ModifyResult::Reverted(/* build result */);
        }

        // Validate
        if !validate_tone_application(buffer, pos, tone) {
            return ModifyResult::Invalid;
        }

        // Apply
        apply_tone(buffer, pos, tone);
        buffer.push_raw_consumed(key, shift);
        *state = EngineState::Marked;

        ModifyResult::Applied(/* build result */)
    }

    fn handles(&self, key: u16) -> bool {
        self.get_tone(key).is_some()
    }
}
```

**File:** `core/src/engine/modifiers/tone_placement.rs`

```rust
//! Vowel selection for tone placement in diphthongs/triphthongs

use crate::engine::DualBuffer;

/// Select main vowel from multiple vowels
///
/// Rules based on Vietnamese phonology:
/// - oa, oe, uy → second vowel (oá, oè, uý)
/// - ai, ao, au, ay, eo, eu, iu, oi, ou, ui → first vowel
/// - uoi, uai → middle vowel (uổi, uải)
pub fn select_main_vowel(buffer: &DualBuffer, positions: &[usize]) -> Option<usize> {
    if positions.len() < 2 {
        return positions.first().copied();
    }

    let chars = buffer.transformed();

    // Get the vowel keys
    let vowels: Vec<u16> = positions
        .iter()
        .filter_map(|&i| chars.get(i).map(|c| c.base_key()))
        .collect();

    // Check patterns
    match vowels.as_slice() {
        // Second vowel patterns
        [0x6F, 0x61] => Some(positions[1]), // oa
        [0x6F, 0x65] => Some(positions[1]), // oe
        [0x75, 0x79] => Some(positions[1]), // uy

        // First vowel patterns
        [0x61, 0x69] => Some(positions[0]), // ai
        [0x61, 0x6F] => Some(positions[0]), // ao
        [0x61, 0x75] => Some(positions[0]), // au
        [0x61, 0x79] => Some(positions[0]), // ay
        [0x65, 0x6F] => Some(positions[0]), // eo
        [0x65, 0x75] => Some(positions[0]), // eu
        [0x69, 0x75] => Some(positions[0]), // iu
        [0x6F, 0x69] => Some(positions[0]), // oi
        [0x6F, 0x75] => Some(positions[0]), // ou
        [0x75, 0x69] => Some(positions[0]), // ui

        // Triphthong patterns (middle vowel)
        [_, v, _] if positions.len() == 3 => Some(positions[1]), // uoi, uai → middle

        // Default: first vowel with mark, or last vowel
        _ => {
            // Find first vowel with existing mark
            for &pos in positions {
                if let Some(c) = chars.get(pos) {
                    if c.has_mark() {
                        return Some(pos);
                    }
                }
            }
            // Fallback to last vowel
            positions.last().copied()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests would use mock DualBuffer with specific vowel patterns
}
```

### Acceptance Criteria

- [ ] tone.rs < 200 LOC
- [ ] tone_telex.rs < 100 LOC
- [ ] tone_vni.rs < 50 LOC
- [ ] tone_placement.rs < 100 LOC
- [ ] Total < 450 LOC (vs 606 original)
- [ ] All tone tests passing
- [ ] Clear separation of concerns

---

## Task 3.3: Split try_mark() (465 LOC)

### New Structure

```
core/src/engine/modifiers/
├── mark.rs             # Core mark logic (<150 LOC)
├── mark_circumflex.rs  # Circumflex (â, ê, ô) (<80 LOC)
├── mark_breve.rs       # Breve (ă) (<50 LOC)
├── mark_horn.rs        # Horn (ơ, ư) (<80 LOC)
└── mark_delayed.rs     # Delayed application (<100 LOC)
```

### Similar pattern to Task 3.2

Each mark type gets its own module with clear responsibility.

---

## Task 3.4: Consolidate Boolean Flags

### Current Flags (11 total)

```rust
struct Engine {
    // Config flags (5)
    enabled: bool,
    skip_w_shortcut: bool,
    esc_restore_enabled: bool,
    free_tone_enabled: bool,
    modern_tone: bool,

    // Runtime flags (6)
    stroke_reverted: bool,
    had_mark_revert: bool,
    pending_mark_revert_pop: bool,
    had_any_transform: bool,
    had_vowel_triggered_circumflex: bool,
    pending_capitalize: bool,
}
```

### New Typed State

```rust
/// Engine configuration (immutable during typing)
#[derive(Clone)]
pub struct EngineConfig {
    pub enabled: bool,
    pub method: InputMethod,
    pub skip_w_shortcut: bool,
    pub esc_restore_enabled: bool,
    pub free_tone_enabled: bool,
    pub modern_tone: bool,
    pub english_auto_restore: bool,
    pub auto_capitalize: bool,
}

/// Pending transformation state
#[derive(Debug, Clone, Copy, Default)]
pub enum PendingTransform {
    #[default]
    None,
    Breve(usize),           // Position for deferred breve
    UHorn(usize),           // Position for deferred u-horn
    MarkRevertPop,          // Waiting for consonant after mark revert
}

/// Transform history for auto-restore
#[derive(Debug, Default)]
pub struct TransformHistory {
    pub had_any_transform: bool,
    pub had_vowel_triggered_circumflex: bool,
    pub stroke_reverted: bool,
    pub had_mark_revert: bool,
}

impl TransformHistory {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn record_transform(&mut self) {
        self.had_any_transform = true;
    }

    pub fn record_circumflex_trigger(&mut self) {
        self.had_vowel_triggered_circumflex = true;
        self.had_any_transform = true;
    }

    pub fn record_stroke_revert(&mut self) {
        self.stroke_reverted = true;
    }

    pub fn record_mark_revert(&mut self) {
        self.had_mark_revert = true;
    }
}

/// New Engine structure with typed state
pub struct Engine {
    // Core data
    buffer: DualBuffer,
    state: EngineState,

    // Typed state (replaces boolean flags)
    config: EngineConfig,
    pending: PendingTransform,
    history: TransformHistory,

    // Other
    word_history: WordHistory,
}
```

### Migration

| Old Flag | New Location | Type |
|----------|--------------|------|
| enabled | config.enabled | bool |
| skip_w_shortcut | config.skip_w_shortcut | bool |
| esc_restore_enabled | config.esc_restore_enabled | bool |
| free_tone_enabled | config.free_tone_enabled | bool |
| modern_tone | config.modern_tone | bool |
| stroke_reverted | history.stroke_reverted | bool |
| had_mark_revert | history.had_mark_revert | bool |
| pending_mark_revert_pop | pending == MarkRevertPop | enum |
| had_any_transform | history.had_any_transform | bool |
| had_vowel_triggered_circumflex | history.had_vowel_triggered_circumflex | bool |
| pending_capitalize | separate field | bool |

### Benefits

1. **Clear grouping**: Config vs Runtime vs Pending
2. **Mutual exclusion**: PendingTransform enum prevents multiple pending states
3. **Easy reset**: history.reset() clears all transform flags
4. **Type safety**: Can't accidentally check wrong flag

### Acceptance Criteria

- [ ] EngineConfig struct extracted
- [ ] PendingTransform enum implemented
- [ ] TransformHistory struct implemented
- [ ] Engine migrated to new structures
- [ ] No bare boolean flags in Engine
- [ ] All tests passing

---

## Task 3.5: Reduce mod.rs to <500 LOC

### Target Structure

```rust
// mod.rs (~300 LOC)
pub struct Engine { ... }

impl Engine {
    pub fn new() -> Self { ... }
    pub fn on_key_ext(&mut self, ...) -> ImeResult { ... }

    // Delegate to modules
    fn process(&mut self, key: u16, shift: bool) -> Option<ImeResult> {
        // Try modifiers in order
        if let Some(result) = self.modifiers.stroke.try_apply(...) { ... }
        if let Some(result) = self.modifiers.tone.try_apply(...) { ... }
        if let Some(result) = self.modifiers.mark.try_apply(...) { ... }
        // ...
    }
}
```

### Line Count Target

| Module | LOC | vs Original |
|--------|-----|-------------|
| mod.rs | 300 | -3,617 |
| state.rs | 150 | new |
| dual_buffer.rs | 200 | new |
| modifiers/mod.rs | 50 | new |
| modifiers/tone.rs | 200 | from try_tone |
| modifiers/tone_*.rs | 150 | from try_tone |
| modifiers/mark.rs | 150 | from try_mark |
| modifiers/mark_*.rs | 150 | from try_mark |
| modifiers/stroke.rs | 100 | from try_stroke |
| auto_restore.rs | 150 | extracted |
| word_history.rs | 100 | existing |
| **Total** | ~1,700 | -2,217 (56% reduction) |

### Acceptance Criteria

- [ ] mod.rs < 500 LOC
- [ ] No function > 200 LOC
- [ ] Clear module boundaries
- [ ] All 561+ tests passing
- [ ] Code review completed

---

## Phase 3 Completion Checklist

- [ ] **Task 3.1:** EngineState with 10+ tests
- [ ] **Task 3.2:** try_tone() split into 4 modules
- [ ] **Task 3.3:** try_mark() split into 5 modules
- [ ] **Task 3.4:** Boolean flags consolidated
- [ ] **Task 3.5:** mod.rs < 500 LOC
- [ ] All 650+ tests passing (including Phase 1-2 additions)
- [ ] Performance regression test passed
- [ ] Code reviewed and merged

**Estimated Effort:** 3-4 weeks
**Risk Level:** HIGH (major refactoring)

---

## Rollback Plan

1. Keep original mod.rs as `mod_legacy.rs`
2. Feature flag: `--features legacy-engine`
3. Parallel testing during transition
4. Gradual migration with compatibility layer

---

## Next Phase

After Phase 3 completion, proceed to **Phase 4: Performance & Polish**.

See main `plan.md` for Phase 4 details.
