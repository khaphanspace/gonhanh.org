# Auto-Restore Behavior - Vietnamese IME Engine

## Decision Flow

```
On Word Boundary (Space/Break):

  1. had_any_transform?
     ├── NO ──→ KEEP (no validation needed)
     └── YES ──→ 2. VN(B) Valid?
                    ├── YES ──→ KEEP VN
                    └── NO ──→ 3. EN(R) Valid?
                                  ├── YES ──→ RESTORE to raw
                                  └── NO ──→ KEEP AS-IS
```

## Decision Matrix

| Condition | VN(B) | EN(R) | Action |
|-----------|-------|-------|--------|
| No transform | - | - | KEEP |
| Transform + VN OK | ✓ | - | KEEP VN |
| Transform + EN OK | ✗ | ✓ | RESTORE |
| Both invalid | ✗ | ✗ | KEEP AS-IS |

---

## Test Cases by Category

### 1. Double Tone Modifier at End (ss, ff, rr, xx, jj)

| Word | Input | Buffer | Raw | VN(B) | EN(R) | Output | Action |
|------|-------|--------|-----|-------|-------|--------|--------|
| bass | b-a-s-s | bas | bass | ✗ | ✓ | bass | RESTORE |
| off | o-f-f | ò | off | ✗ | ✓ | off | RESTORE |
| buff | b-u-f-f | bù | buff | ✗ | ✓ | buff | RESTORE |
| miss | m-i-s-s | mí | miss | ✗ | ✓ | miss | RESTORE |
| lass | l-a-s-s | las | lass | ✗ | ✓ | lass | RESTORE |
| mass | m-a-s-s | mas | mass | ✗ | ✓ | mass | RESTORE |
| pass | p-a-s-s | pas | pass | ✗ | ✓ | pass | RESTORE |

**Notes:**
- Buffer after double modifier shows reverted state (no tone)
- "bas" invalid VN (S not valid final) → check EN → "bass" valid → RESTORE

### 2. Double Tone Modifier - Multiple Occurrences

| Word | Input | Buffer | Raw | VN(B) | EN(R) | Output | Action |
|------|-------|--------|-----|-------|-------|--------|--------|
| assess | a-s-s-e-s-s | ásess | assess | ✗ | ✓ | assess | RESTORE |
| access | a-c-c-e-s-s | accés | access | ✗ | ✓ | access | RESTORE |
| possess | p-o-s-s-e-s-s | pósess | possess | ✗ | ✓ | possess | RESTORE |
| success | s-u-c-c-e-s-s | succés | success | ✗ | ✓ | success | RESTORE |

**Notes:**
- First `ss` triggers revert and foreign mode
- Subsequent `ss` are literal in foreign mode
- Buffer shows transformed state before revert

### 3. Standalone Earlier + Double End

| Word | Input | Buffer | Raw | VN(B) | EN(R) | Output | Action |
|------|-------|--------|-----|-------|-------|--------|--------|
| sims | s-i-m-s | sím | sims | ✗ | ✓ | sims | RESTORE |
| simss | s-i-m-s-s | síms | simss | ✗ | ✗ | sims | KEEP_AS_IS |

**Notes:**
- "sims": `s` at start is literal, `s` at end is tone, buffer "sím" → invalid VN → restore
- "simss": `ss` at end triggers revert, buffer becomes "sims" (literal), "simss" NOT valid EN

### 4. Short Words (buf.len <= 3)

| Word | Input | Buffer | Raw | VN(B) | EN(R) | Output | Action |
|------|-------|--------|-----|-------|-------|--------|--------|
| aro | a-r-r-o | ảo | arro | ✗ | ✗ | aro | KEEP_AS_IS |
| ass | a-s-s | as | ass | ✗ | ✓ | ass | RESTORE |
| err | e-r-r | ẻ | err | ✗ | ✓ | err | RESTORE |

**Notes:**
- "arro" not valid EN → keep buffer "ảo" but output normalized

### 5. Pattern: U + Doubled Modifier

| Word | Input | Buffer | Raw | VN(B) | EN(R) | Output | Action |
|------|-------|--------|-----|-------|-------|--------|--------|
| users | u-s-s-e-r-s | úsers | ussers | ✗ | ✗ | users | KEEP_AS_IS |
| user | u-s-e-r | úer | user | ✗ | ✓ | user | RESTORE |

**Notes:**
- "ussers" raw NOT valid EN → keep buffer form
- "user" raw valid EN → restore

### 6. Vietnamese Words (Should NOT Restore)

| Word | Input | Buffer | Raw | VN(B) | EN(R) | Output | Action |
|------|-------|--------|-----|-------|-------|--------|--------|
| bạn | b-a-j-n | bạn | bajn | ✓ | ✗ | bạn | KEEP_VN |
| việt | v-i-e-e-j-t | việt | vieejt | ✓ | ✗ | việt | KEEP_VN |
| được | d-d-u-o-w-j-c | được | dduowjc | ✓ | ✗ | được | KEEP_VN |

**Notes:**
- Valid VN buffer → KEEP VN regardless of EN validity
- VN validation takes priority

### 7. AW Pattern (Breve Trigger)

| Word | Input | Buffer | Raw | VN(B) | EN(R) | Output | Action |
|------|-------|--------|-----|-------|-------|--------|--------|
| law | l-a-w | lă | law | ✗ | ✓ | law | RESTORE |
| raw | r-a-w | ră | raw | ✗ | ✓ | raw | RESTORE |
| saw | s-a-w | să | saw | ✗ | ✓ | saw | RESTORE |
| draw | d-r-a-w | dră | draw | ✗ | ✓ | draw | RESTORE |

**Notes:**
- "lă" invalid VN (no valid final consonant)
- "law" valid EN → RESTORE
- Pattern: aw → ă in VN, but standalone ă is invalid syllable

### 8. OW Pattern (Horn Trigger)

| Word | Input | Buffer | Raw | VN(B) | EN(R) | Output | Action |
|------|-------|--------|-----|-------|-------|--------|--------|
| low | l-o-w | lơ | low | ✗ | ✓ | low | RESTORE |
| row | r-o-w | rơ | row | ✗ | ✓ | row | RESTORE |
| show | s-h-o-w | shơ | show | ✗ | ✓ | show | RESTORE |
| flow | f-l-o-w | flơ | flow | ✗ | ✓ | flow | RESTORE |

**Notes:**
- "lơ" invalid VN (no valid final)
- "shơ" invalid VN ("sh" not valid VN initial)
- All valid EN words → RESTORE

### 9. Circumflex Pattern (aa, ee, oo)

| Word | Input | Buffer | Raw | VN(B) | EN(R) | Output | Action |
|------|-------|--------|-----|-------|-------|--------|--------|
| data | d-a-t-a | dât | data | ✓ | ✓ | dât | KEEP_VN |
| dataa | d-a-t-a-a | data | dataa | ✓ | ✗ | data | KEEP_VN |
| saas | s-a-a-s | sâs | saas | ✗ | ✓ | saas | RESTORE |

**Notes:**
- "dât" valid VN → KEEP even though "data" also valid EN (VN priority)
- "dataa" → revert circumflex → "data" valid VN
- "sâs" invalid VN (S not valid final) → "saas" valid EN pattern → RESTORE

### 10. English Tech Terms (X modifier)

| Word | Input | Buffer | Raw | VN(B) | EN(R) | Output | Action |
|------|-------|--------|-----|-------|-------|--------|--------|
| text | t-e-x-t | tẽt | text | ✗ | ✓ | text | RESTORE |
| next | n-e-x-t | nẽt | next | ✗ | ✓ | next | RESTORE |
| expect | e-x-p-e-c-t | ẽpect | expect | ✗ | ✓ | expect | RESTORE |
| context | c-o-n-t-e-x-t | contẽt | context | ✗ | ✓ | context | RESTORE |

**Notes:**
- "xt" impossible cluster in VN → triggers restore
- X as tone modifier (ngã) creates invalid combinations

### 11. VIEW/IEW Pattern

| Word | Input | Buffer | Raw | VN(B) | EN(R) | Output | Action |
|------|-------|--------|-----|-------|-------|--------|--------|
| view | v-i-e-w | vieư | view | ✗ | ✓ | view | RESTORE |
| review | r-e-v-i-e-w | revieư | review | ✗ | ✓ | review | RESTORE |

**Notes:**
- "vieư" invalid VN triphthong
- "view" valid EN → RESTORE

### 12. Triple Modifier (User Types 3 Same Keys)

| Word | Input | Buffer | Raw | VN(B) | EN(R) | Output | Action |
|------|-------|--------|-----|-------|-------|--------|--------|
| basss | b-a-s-s-s | bass | basss | ✗ | ✓ | bass | RESTORE |
| isssue | i-s-s-s-u-e | issue | isssue | ✗ | ✓ | issue | RESTORE |

**Notes:**
- First `ss` triggers revert + foreign mode
- Third `s` is literal in foreign mode
- Buffer becomes "bass", "issue" respectively
- Both patterns produce correct EN output

### 13. Multi-syllable Foreign Words

| Word | Input | Buffer | Raw | VN(B) | EN(R) | Output | Action |
|------|-------|--------|-----|-------|-------|--------|--------|
| datas | d-a-t-a-s | dất | datas | ✓ | ✓ | datas | RESTORE |
| makes | m-a-k-e-s | makés | makes | ✗ | ✓ | makes | RESTORE |
| wikis | w-i-k-i-s | wikís | wikis | ✗ | ✓ | wikis | RESTORE |

**Notes:**
- "datas" EN plural, should restore despite VN buffer valid
- "K" not valid VN initial before A → invalid VN → restore
- "W" not typical VN initial → foreign detection

### 14. FF in Middle (Coffee Pattern)

| Word | Input | Buffer | Raw | VN(B) | EN(R) | Output | Action |
|------|-------|--------|-----|-------|-------|--------|--------|
| coffee | c-o-f-f-e-e | coffê | coffee | ✗ | ✓ | coffee | RESTORE |
| offer | o-f-f-e-r | òfer | offer | ✗ | ✓ | offer | RESTORE |
| office | o-f-f-i-c-e | òfice | office | ✗ | ✓ | office | RESTORE |
| effect | e-f-f-e-c-t | èfect | effect | ✗ | ✓ | effect | RESTORE |

**Notes:**
- Double consonant in middle common EN pattern
- First `f` is tone modifier (huyền), second triggers revert
- Foreign mode continues after revert

### 15. SS in Middle (Issue Pattern)

| Word | Input | Buffer | Raw | VN(B) | EN(R) | Output | Action |
|------|-------|--------|-----|-------|-------|--------|--------|
| issue | i-s-s-u-e | ísue | issue | ✗ | ✓ | issue | RESTORE |
| tissue | t-i-s-s-u-e | tísue | tissue | ✗ | ✓ | tissue | RESTORE |
| mission | m-i-s-s-i-o-n | mísion | mission | ✗ | ✓ | mission | RESTORE |

**Notes:**
- `ss` in middle triggers revert + foreign mode
- Remaining characters are literal

### 16. Ambiguous (Both VN and EN Valid)

| Word | Input | Buffer | Raw | VN(B) | EN(R) | Output | Action |
|------|-------|--------|-----|-------|-------|--------|--------|
| con | c-o-n | con | con | ✓ | ✓ | con | KEEP_VN |
| an | a-n | an | an | ✓ | ✓ | an | KEEP_VN |

**Notes:**
- When both valid, prefer VN (user intent assumption)
- No transform occurred → no validation needed

---

## Column Legend

| Column | Description |
|--------|-------------|
| Word | Target word (lowercase) |
| Input | Keystroke sequence (hyphen-separated) |
| Buffer | Buffer state after all keystrokes (transformed) |
| Raw | Raw input (original keystrokes without modifiers consumed) |
| VN(B) | Buffer validation: ✓=valid VN syllable, ✗=invalid |
| EN(R) | English validation: ✓=valid EN word, ✗=invalid |
| Output | Final output after space |
| Action | KEEP_VN / RESTORE / KEEP_AS_IS |

---

## Validation Rules

### VN Buffer Validation
1. Valid initial consonant (or none)
2. Valid vowel nucleus
3. Valid final consonant (or none)
4. Valid tone+final combination
5. Valid syllable structure

### EN Pattern Detection
1. Common EN onset clusters (bl, cl, dr, fl, gr, pl, pr, sc, sk, sl, sm, sn, sp, st, sw, tr, tw)
2. Common EN coda clusters (ct, ft, ld, lf, lk, lm, lp, lt, mp, nd, nk, nt, pt, rd, rf, rk, rm, rn, rp, rt, sk, sp, st)
3. Impossible VN bigrams (xt, xp, bf, bv, etc.)
4. Common EN suffixes (-tion, -ing, -ness, -ment, -able)
5. Common EN prefixes (un-, re-, pre-, dis-)
6. Invalid VN vowel patterns
7. Word dictionary lookup (optional)

---

## Implementation Notes

**CRITICAL: NEVER fix case-by-case**

All decisions MUST go through unified validation flow:
1. VN(B) - Validate transformed buffer
2. EN(R) - English pattern detection
3. Decision based on validation results

Do NOT add special handling for specific words like "bass", "issue", "data", etc.
