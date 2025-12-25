# Typing Behavior Flow - Vietnamese IME Engine V2

## TELEX Key Mappings

| Key | Function | Value |
|-----|----------|-------|
| `s` | sắc (á) | mark=1 |
| `f` | huyền (à) | mark=2 |
| `r` | hỏi (ả) | mark=3 |
| `x` | ngã (ã) | mark=4 |
| `j` | nặng (ạ) | mark=5 |
| `w` | horn (ư,ơ) / breve (ă) | tone=2 |
| `aa/ee/oo` | circumflex (â,ê,ô) | tone=1 |
| `dd` | stroke (đ) | stroke=true |

---

## Example 1: `d u o w c j d` + space → "được "

| # | Key | Action | Raw | Buffer | Output | VN(R) | VN(B) | EN(R) | EN(B) | Restore? |
|---|-----|--------|-----|--------|--------|-------|-------|-------|-------|----------|
| 1 | `d` | PASS | `[d]` | `[d]` | `d` | ✓ | | | | |
| 2 | `u` | PASS | `[d,u]` | `[d,u]` | `du` | ✓ | | | | |
| 3 | `o` | PASS | `[d,u,o]` | `[d,u,o]` | `duo` | ✓ | | | | |
| 4 | `w` | HORN | `[d,u,o,w]` | `[d,ư,ơ]` | `dươ` | ✓ | | | | |
| 5 | `c` | PASS | `[d,u,o,w,c]` | `[d,ư,ơ,c]` | `dươc` | ✓ | | | | |
| 6 | `j` | TONE | `[d,u,o,w,c,j]` | `[d,ư,ợ,c]` | `dược` | ✓ | | | | |
| 7 | `d` | STROKE | `[d,u,o,w,c,j,d]` | `[đ,ư,ợ,c]` | `được` | ✓ | | | | |
| 8 | `space` | COMMIT | `[]` | `[]` | `được ` | ✓ | | | | |

---

## Example 2: `d u w o w c d j` + space → "được "

| # | Key | Action | Raw | Buffer | Output | VN(R) | VN(B) | EN(R) | EN(B) | Restore? |
|---|-----|--------|-----|--------|--------|-------|-------|-------|-------|----------|
| 1 | `d` | PASS | `[d]` | `[d]` | `d` | ✓ | | | | |
| 2 | `u` | PASS | `[d,u]` | `[d,u]` | `du` | ✓ | | | | |
| 3 | `w` | HORN | `[d,u,w]` | `[d,ư]` | `dư` | ✓ | | | | |
| 4 | `o` | PASS | `[d,u,w,o]` | `[d,ư,o]` | `dưo` | ✓ | | | | |
| 5 | `w` | HORN | `[d,u,w,o,w]` | `[d,ư,ơ]` | `dươ` | ✓ | | | | |
| 6 | `c` | PASS | `[d,u,w,o,w,c]` | `[d,ư,ơ,c]` | `dươc` | ✓ | | | | |
| 7 | `d` | STROKE | `[d,u,w,o,w,c,d]` | `[đ,ư,ơ,c]` | `đươc` | ✓ | | | | |
| 8 | `j` | TONE | `[d,u,w,o,w,c,d,j]` | `[đ,ư,ợ,c]` | `được` | ✓ | | | | |
| 9 | `space` | COMMIT | `[]` | `[]` | `được ` | ✓ | | | | |

---

## Example 3: `c l a r` + space → "clar "

| # | Key | Action | Raw | Buffer | Output | VN(R) | VN(B) | EN(R) | EN(B) | Restore? |
|---|-----|--------|-----|--------|--------|-------|-------|-------|-------|----------|
| 1 | `c` | PASS | `[c]` | `[c]` | `c` | ✓ | | | | |
| 2 | `l` | PASS | `[c,l]` | `[c,l]` | `cl` | ✗ | ✗ | ✓ | | Foreign |
| 3 | `a` | PASS | `[c,l,a]` | `[c,l,a]` | `cla` | | | ✓ | | |
| 4 | `r` | REJECT | `[c,l,a,r]` | `[c,l,a,r]` | `clar` | | | ✓ | | |
| 5 | `space` | COMMIT | `[]` | `[]` | `clar ` | | | ✓ | | |

> **Note:** 'cl' impossible → VN(R)=✗ → VN(B)=✗ (no transform) → Foreign mode.

---

## Example 4: `t e x t` + space → "text "

| # | Key | Action | Raw | Buffer | Output | VN(R) | VN(B) | EN(R) | EN(B) | Restore? |
|---|-----|--------|-----|--------|--------|-------|-------|-------|-------|----------|
| 1 | `t` | PASS | `[t]` | `[t]` | `t` | ✓ | | | | |
| 2 | `e` | PASS | `[t,e]` | `[t,e]` | `te` | ✓ | | | | |
| 3 | `x` | TONE | `[t,e,x]` | `[t,ẽ]` | `tẽ` | ✓ | | | | |
| 4 | `t` | RESTORE | `[t,e,x,t]` | `[t,e,x,t]` | `text` | ✗ | ✗ | ✓ | | **Triggered** |
| 5 | `space` | COMMIT | `[]` | `[]` | `text ` | | | ✓ | | |

> **Note:** 'xt' impossible → VN(R)=✗ → VN(B)=✗ → RESTORE triggered (undo ẽ→e).

---

## Example 5: `e x p e c t` + space → "expect "

| # | Key | Action | Raw | Buffer | Output | VN(R) | VN(B) | EN(R) | EN(B) | Restore? |
|---|-----|--------|-----|--------|--------|-------|-------|-------|-------|----------|
| 1 | `e` | PASS | `[e]` | `[e]` | `e` | ✓ | | | | |
| 2 | `x` | TONE | `[e,x]` | `[ẽ]` | `ẽ` | ✓ | | | | |
| 3 | `p` | RESTORE | `[e,x,p]` | `[e,x,p]` | `exp` | ✗ | ✗ | ✓ | | **Triggered** |
| 4 | `e` | PASS | `[e,x,p,e]` | `[e,x,p,e]` | `expe` | | | ✓ | | |
| 5 | `c` | PASS | `[e,x,p,e,c]` | `[e,x,p,e,c]` | `expec` | | | ✓ | | |
| 6 | `t` | PASS | `[e,x,p,e,c,t]` | `[e,x,p,e,c,t]` | `expect` | | | ✓ | | |
| 7 | `space` | COMMIT | `[]` | `[]` | `expect ` | | | ✓ | | |

> **Note:** 'xp' impossible → VN(R)=✗ → VN(B)=✗ → RESTORE triggered (undo ẽ→e).

---

## Example 6: `v a r` + space → "vả " (VN wins)

| # | Key | Action | Raw | Buffer | Output | VN(R) | VN(B) | EN(R) | EN(B) | Restore? |
|---|-----|--------|-----|--------|--------|-------|-------|-------|-------|----------|
| 1 | `v` | PASS | `[v]` | `[v]` | `v` | ✓ | | | | |
| 2 | `a` | PASS | `[v,a]` | `[v,a]` | `va` | ✓ | | | | |
| 3 | `r` | TONE | `[v,a,r]` | `[v,ả]` | `vả` | ✓ | | | | |
| 4 | `space` | COMMIT | `[]` | `[]` | `vả ` | ✓ | | | | |

> **Note:** "vả" is valid Vietnamese → VN(R)=✓ all steps → VN wins.

---

## Example 7: `v a r r` + space → "var " (double-key revert)

| # | Key | Action | Raw | Buffer | Output | VN(R) | VN(B) | EN(R) | EN(B) | Restore? |
|---|-----|--------|-----|--------|--------|-------|-------|-------|-------|----------|
| 1 | `v` | PASS | `[v]` | `[v]` | `v` | ✓ | | | | |
| 2 | `a` | PASS | `[v,a]` | `[v,a]` | `va` | ✓ | | | | |
| 3 | `r` | TONE | `[v,a,r]` | `[v,ả]` | `vả` | ✓ | | | | |
| 4 | `r` | REVERT | `[v,a,r,r]` | `[v,a,r]` | `var` | ✗ | | ✓ | | |
| 5 | `space` | COMMIT | `[]` | `[]` | `var ` | | | ✓ | | |

> **Note:** Double 'rr' detected → VN(R)=✗ → REVERT (undo hỏi) → EN mode.

---

## Example 8: `d e n d e s n n n n` + space → "đếnnnnn "

| # | Key | Action | Raw | Buffer | Output | VN(R) | VN(B) | EN(R) | EN(B) | Restore? |
|---|-----|--------|-----|--------|--------|-------|-------|-------|-------|----------|
| 1 | `d` | PASS | `[d]` | `[d]` | `d` | ✓ | | | | |
| 2 | `e` | PASS | `[d,e]` | `[d,e]` | `de` | ✓ | | | | |
| 3 | `n` | PASS | `[d,e,n]` | `[d,e,n]` | `den` | ✓ | | | | |
| 4 | `d` | STROKE | `[d,e,n,d]` | `[đ,e,n]` | `đen` | ✓ | | | | |
| 5 | `e` | CIRCUM | `[d,e,n,d,e]` | `[đ,ê,n]` | `đên` | ✓ | | | | |
| 6 | `s` | TONE | `[d,e,n,d,e,s]` | `[đ,ế,n]` | `đến` | ✓ | | | | |
| 7 | `n` | PASS | `[d,e,n,d,e,s,n]` | `[đ,ế,n,n]` | `đếnn` | ✓ | | | | |
| 8 | `n` | PASS | `[...,n,n]` | `[đ,ế,n,n,n]` | `đếnnn` | ✓ | | | | |
| 9 | `n` | PASS | `[...,n,n,n]` | `[...,n,n,n,n]` | `đếnnnn` | ✓ | | | | |
| 10 | `n` | PASS | `[...,n,n,n,n]` | `[...,n,n,n,n,n]` | `đếnnnnn` | ✓ | | | | |
| 11 | `space` | COMMIT | `[]` | `[]` | `đếnnnnn ` | ✓ | | | | |

> **Note:**
> - `dd` stroke, `ee` circumflex work non-consecutively
> - VN(R)=✓ throughout (repeated 'n' not impossible) → keep VN transform

---

## Example 9: `t o t o s` + space → "tốt "

| # | Key | Action | Raw | Buffer | Output | VN(R) | VN(B) | EN(R) | EN(B) | Restore? |
|---|-----|--------|-----|--------|--------|-------|-------|-------|-------|----------|
| 1 | `t` | PASS | `[t]` | `[t]` | `t` | ✓ | | | | |
| 2 | `o` | PASS | `[t,o]` | `[t,o]` | `to` | ✓ | | | | |
| 3 | `t` | PASS | `[t,o,t]` | `[t,o,t]` | `tot` | ✓ | | | | |
| 4 | `o` | CIRCUM | `[t,o,t,o]` | `[t,ô,t]` | `tôt` | ✓ | | | | |
| 5 | `s` | TONE | `[t,o,t,o,s]` | `[t,ố,t]` | `tốt` | ✓ | | | | |
| 6 | `space` | COMMIT | `[]` | `[]` | `tốt ` | ✓ | | | | |

> **Note:** 'oo' forms circumflex (ô) non-consecutively. Valid VN word "tốt" (good).

---

## Example 10: `c o n s o l e` + space → "console "

| # | Key | Action | Raw | Buffer | Output | VN(R) | VN(B) | EN(R) | EN(B) | Restore? |
|---|-----|--------|-----|--------|--------|-------|-------|-------|-------|----------|
| 1 | `c` | PASS | `[c]` | `[c]` | `c` | ✓ | | | | |
| 2 | `o` | PASS | `[c,o]` | `[c,o]` | `co` | ✓ | | | | |
| 3 | `n` | PASS | `[c,o,n]` | `[c,o,n]` | `con` | ✓ | | | | |
| 4 | `s` | TONE | `[c,o,n,s]` | `[c,ó,n]` | `cón` | ✓ | | | | |
| 5 | `o` | PASS | `[c,o,n,s,o]` | `[c,ó,n,o]` | `cóno` | ✓ | | | | |
| 6 | `l` | RESTORE | `[c,o,n,s,o,l]` | `[c,o,n,s,o,l]` | `consol` | ✓ | ✗ | ✓ | | **Triggered** |
| 7 | `e` | PASS | `[c,o,n,s,o,l,e]` | `[c,o,n,s,o,l,e]` | `console` | | | ✓ | | |
| 8 | `space` | COMMIT | `[]` | `[]` | `console ` | | | ✓ | | |

> **Note:** 'l' invalid final → VN(R)=✓ but VN(B)=✗ → RESTORE triggered.

---

## Example 11: `d i s s t` + space → "dist "

| # | Key | Action | Raw | Buffer | Output | VN(R) | VN(B) | EN(R) | EN(B) | Restore? |
|---|-----|--------|-----|--------|--------|-------|-------|-------|-------|----------|
| 1 | `d` | PASS | `[d]` | `[d]` | `d` | ✓ | | | | |
| 2 | `i` | PASS | `[d,i]` | `[d,i]` | `di` | ✓ | | | | |
| 3 | `s` | TONE | `[d,i,s]` | `[d,í]` | `dí` | ✓ | | | | |
| 4 | `s` | REVERT | `[d,i,s,s]` | `[d,i,s]` | `dis` | ✗ | | ✓ | | |
| 5 | `t` | PASS | `[d,i,s,s,t]` | `[d,i,s,t]` | `dist` | | | ✓ | | |
| 6 | `space` | COMMIT | `[]` | `[]` | `dist ` | | | ✓ | | |

> **Note:** Double 'ss' detected → VN(R)=✗ → REVERT (undo sắc) → EN mode.

---

## Example 12: `d u o w n g o d` + space → "đuông "

| # | Key | Action | Raw | Buffer | Output | VN | EN | Restore? |
|---|-----|--------|-----|--------|--------|----|----|----------|
| 1 | `d` | PASS | `[d]` | `[d]` | `d` | ✓ | | |
| 2 | `u` | PASS | `[d,u]` | `[d,u]` | `du` | ✓ | | |
| 3 | `o` | PASS | `[d,u,o]` | `[d,u,o]` | `duo` | ✓ | | |
| 4 | `w` | HORN | `[d,u,o,w]` | `[d,ư,ơ]` | `dươ` | ✓ | | |
| 5 | `n` | PASS | `[d,u,o,w,n]` | `[d,ư,ơ,n]` | `dươn` | ✓ | | |
| 6 | `g` | PASS | `[d,u,o,w,n,g]` | `[d,ư,ơ,n,g]` | `dương` | ✓ | | |
| 7 | `o` | CIRCUM | `[d,u,o,w,n,g,o]` | `[d,u,ô,n,g]` | `duông` | ✓ | | |
| 8 | `d` | STROKE | `[d,u,o,w,n,g,o,d]` | `[đ,u,ô,n,g]` | `đuông` | ✓ | | |
| 9 | `space` | COMMIT | `[]` | `[]` | `đuông ` | ✓ | | |

> **Note:**
> - 'w' applies horn to both u→ư and o→ơ
> - Second 'o' (step 7): 'oo' circumflex overrides horn, ơ→ô and ư→u (horn removed from both)
> - 'dd' stroke applies non-consecutively to first 'd'

---

## Column Legend

| Column | Description |
|--------|-------------|
| Raw | `RawBuffer` - keystroke gốc (chưa transform) |
| Buffer | `Buffer` - ký tự đã transform (có dấu) |
| VN | Validation result with source: ✓=valid, ✗=impossible, ~=invalid word but possible |
| | `(R)`=validated via Raw, `(B)`=validated via Buffer, `(R+B)`=both |
| EN | ✓=English/Foreign mode |
| Restore? | (empty)=no, **Triggered**=executed immediately (once), **Foreign**=early foreign |

### Validation Source per Check Type

| Check Type | Source | When |
|------------|--------|------|
| Cluster validation (cl, xp, xt) | **Raw** | After consonant added |
| Invalid final (l, r in final) | **Buffer** | When consonant follows vowel |
| Double-key revert (ss, rr, ff) | **Raw** | When same key repeated |
| Syllable structure | **Buffer** | Ongoing pattern check |

---

## Validation Logic (Option 3: Both Raw + Buffer)

| Check | Source | Purpose | Result |
|-------|--------|---------|--------|
| Impossible clusters | **Raw** | Detect `cl`, `xp`, `xt` early | FOREIGN or RESTORE |
| Invalid final consonant | **Buffer + State** | 'l', 'r' in final position | RESTORE |
| Double-key revert | **Raw** | `ss`, `rr`, `ff` patterns | → EN mode |
| Syllable structure | **Buffer** | Vowel-consonant patterns | VN=~ (keep transform) |

---

## Restore Logic

| Condition | VN | Restore? |
|-----------|-------|----------|
| Valid Vietnamese word | ✓ | - |
| Impossible cluster in Raw (`xt`, `xp`) | ✗ | **Triggered** immediately |
| Foreign cluster early (`cl`) | ✗ | **Foreign** (stay raw) |
| Invalid final in Buffer (`l`, `r`) | ✗ | **Triggered** immediately |
| Double-key revert (`ss`, `rr`) | | → EN mode (no restore needed) |
| Invalid word, possible pattern (`nnn`) | ~ | - (keep VN transform) |

---

## Comparison

| Sequence | Keys | Result | Restore? |
|----------|------|--------|----------|
| `d u o w c j d` + space | 8 | `được ` | - |
| `d u w o w c d j` + space | 9 | `được ` | - |
| `c l a r` + space | 5 | `clar ` | Foreign (step 2) |
| `t e x t` + space | 5 | `text ` | Triggered (step 4) |
| `e x p e c t` + space | 7 | `expect ` | Triggered (step 3) |
| `v a r` + space | 4 | `vả ` | - (VN wins) |
| `v a r r` + space | 5 | `var ` | Revert → EN mode |
| `d e n d e s n n n n` + space | 11 | `đếnnnnn ` | VN=~ (step 7+), no restore |
| `t o t o s` + space | 6 | `tốt ` | - ('oo' → ô, valid VN) |
| `c o n s o l e` + space | 8 | `console ` | Triggered (step 6, 'l' invalid final) |
| `d i s s t` + space | 6 | `dist ` | Revert → EN mode |
