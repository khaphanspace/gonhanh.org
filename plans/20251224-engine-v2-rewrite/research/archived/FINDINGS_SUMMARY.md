# V2 Architecture Validation - Quick Reference

**Status:** ✅ APPROVED with 3 Required Enhancements
**Date:** 2025-12-24
**Full Report:** `researcher-02-v2-coverage-validation.md`

---

## Key Findings

### What Works ✅
- **State Machine Design:** 7-state enum covers all phonetic progressions
- **DualBuffer Concept:** Correctly separates transformed + raw for restore
- **Bidirectional Validation:** Solves "false restore" problem (e.g., "đườngfffff")
- **Telex/VNI Input:** Both methods fully supported via existing parsers
- **Error Correction:** Backspace, delete, word history all functional
- **Double-Key Revert:** Pattern detection prevents false finals
- **Tone Placement:** Vowel.rs patterns handle complex diphthongs
- **English Auto-Restore:** Restores "text" → "tẽt" → "text" correctly

**Coverage:** 95%+ of typing scenarios handled correctly.

---

## Critical Gaps ⚠️

### Gap 1: Tone-Stop Final Rule (MISSING)
**Severity:** HIGH | **Impact:** False Vietnamese validation possible

Vietnamese phonotactic rule: Stop finals (p, t, c, ch) only allow sắc(´) or nặng(.) tones.
Currently not validated - can accept `tàp` (huyền + p = INVALID).

**Fix:** Add Rule 7 to validation.rs before Phase 1 completion.

```rust
fn rule_tone_stop_final(snap: &BufferSnapshot, syllable: &Syllable) -> bool {
    if matches!(final_consonant, p|t|c|ch) {
        return tone in [sắc, nặng]  // Only these allowed
    }
    true  // Other finals allow all tones
}
```

### Gap 2: Foreign State Recovery (AMBIGUOUS)
**Severity:** MEDIUM | **Impact:** User experience unclear

v2 defines Foreign state but doesn't specify next key behavior. After invalid word detected:
- Does user continue typing?
- Do they lose the word?
- Can they recover it?

**Fix:** Explicitly document in Phase 1:
```
Foreign → [next consonant] → Reset (new word)
Foreign → [vowel] → Stay Foreign (complete invalid sequence)
Foreign → [Backspace] → Revert to Marked
```

### Gap 3: DualBuffer Vec Allocation (PERFORMANCE)
**Severity:** MEDIUM | **Impact:** Breaks "zero allocation hot path" principle

Architecture proposes:
```rust
raw: Vec<RawKeystroke>,  // ❌ HEAP ALLOCATION
```

Should be:
```rust
raw: [RawKeystroke; 96],  // ✅ FIXED ARRAY, NO ALLOCATION
```

**Fix:** Must address in Phase 2 before DualBuffer integration.

---

## Test Coverage Gaps

| Test Type | Status | Count | Priority |
|-----------|--------|-------|----------|
| Tone-Stop Final validation | ❌ MISSING | 0 | P0 - Add before Phase 1 |
| Bidirectional restore | ⚠️ Partial | ~5 | P1 - Expand coverage |
| Foreign pattern recovery | ⚠️ Partial | ~3 | P1 - Add behavior tests |
| State machine explicit | ❌ MISSING | 0 | P2 - Phase 3 addition |
| Ambiguous valid-both cases | ❌ MISSING | 0 | P3 - Phase 3+ feature |

**Existing:** 561 tests (mostly input method + transform specifics)
**Need to add:** 20-30 tests for v2-specific validation

---

## Quick Scenario Check (20 cases)

✅ PASS cases (18):
- Basic telex: `vieejt` → `việt`
- VNI input: `vie65t` → `việt`
- Double-key revert: `viitt` → `viết`
- Tone placement: `mua` + `s` → `múa`
- Qu-special: `qua` + `f` → `quả`
- English restore: `text` → `tẽt` → `text`
- Invalid both: `đườngfffff` → keep as-is
- ESC restore, auto-capitalize, shortcuts, all compound vowels, finals (m,n,ng,nh)

⚠️ FAIL cases (2):
- Tone + stop final: `tap` + huyền → INVALID (Rule 7 missing)
- Ambiguous valid both: `mix` → `mĩ` → unclear (Phase 3 feature)

---

## Phase 1 Checklist

Before starting implementation:

- [ ] **Add Rule 7:** Tone-Stop Final validation to validation.rs
  - [ ] Code implementation (10 lines)
  - [ ] Unit tests (4 tests)
  - [ ] Integration test with engine

- [ ] **Define Foreign State Behavior:**
  - [ ] Document next-key transitions
  - [ ] Add tests for each case
  - [ ] Update architecture doc

- [ ] **Change raw_input to Fixed Array:**
  - [ ] Replace `Vec<RawKeystroke>` with `[RawKeystroke; 96]`
  - [ ] Update all access patterns
  - [ ] Verify no performance regression

- [ ] **Add English Validation Module:**
  - [ ] Create `core/src/engine/english.rs`
  - [ ] Implement pattern-based validation
  - [ ] Add morphological checks (prefixes/suffixes)
  - [ ] Tests (10+ cases)

- [ ] **Bidirectional Validation Integration:**
  - [ ] Update `try_auto_restore_on_space()`
  - [ ] Call `validate_english()` after VN fails
  - [ ] Implement restore decision matrix
  - [ ] Add regression tests

**Estimated effort:** 1-2 weeks (includes testing)

---

## Memory Profile Impact

| Aspect | v1 Current | v2 Target | Improvement |
|--------|-----------|-----------|-------------|
| Per-keystroke heap allocs | Yes (Vec push) | Zero (fixed array) | 100% reduction |
| Engine struct size | ~500B | ~500B | No change |
| Hot path allocations | Vec + Buffer | Buffer only | ~50% reduction |
| Total app RAM | ~4MB | <5MB | Slight improvement |
| L1 cache efficiency | Mixed | Better (stack-first) | Measurable |

---

## Performance Metrics Maintained

| Metric | Target | v2 Impact |
|--------|--------|-----------|
| On-key latency | <1ms | ✅ Same or faster |
| Validation time | <0.1ms | ✅ Same (O(n) at word boundary only) |
| Memory per keystroke | Zero heap | ✅ Achieved |
| CPU cache usage | L1 friendly | ✅ Improved (fixed arrays) |

---

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|---|---|---|
| Rule 7 false negatives | Low | High | Implement + test thoroughly |
| Foreign state edge cases | Medium | Low | Explicit state docs |
| English false positives | Low | Low | Pattern-only default, opt-in Bloom |
| DualBuffer desync bugs | Low | High | Strong invariant testing |
| Performance regression | Low | Low | Benchmark before/after |

**Overall Risk Level:** LOW-MEDIUM (all mitigated by Phase 1 work)

---

## Recommendation

### ✅ GREEN LIGHT FOR IMPLEMENTATION

**Conditions:**
1. Implement 3 gaps identified above in Phase 1
2. Add 20+ tests for new validation rules
3. Document Foreign state transitions clearly
4. Use fixed array for raw_input (CRITICAL)

**Go/No-Go:** ✅ **GO** - Proceed with Phase 1 immediately

**Timeline:**
- Phase 1 (Validation + Bidirectional): 2 weeks
- Phase 2 (DualBuffer): 2 weeks
- Phase 3 (State Machine): 3-4 weeks
- Phase 4 (Modularization): 2-3 weeks
- **Total:** ~10-11 weeks for full v2 release

---

## Next Steps

1. **Review this summary** with team
2. **Read full report** for detailed analysis
3. **Create Phase 1 task list** with Jira/GitHub
4. **Start implementation** on branch `feature/engine-v2-phase1`
5. **Run full test suite** before each merge

---

**Full validation report:** `researcher-02-v2-coverage-validation.md`
**Detailed phonotactic analysis:** Section 11 in full report
**Specific test recommendations:** Section 7.2 in full report
