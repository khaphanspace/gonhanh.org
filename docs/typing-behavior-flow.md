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
| 4 | `w` | HORN | `[d,u,o,w]` | `[d,ư,ơ]` | `dươ` | ✗ | ✓ | | | |
| 5 | `c` | PASS | `[d,u,o,w,c]` | `[d,ư,ơ,c]` | `dươc` | ✗ | ✓ | | | |
| 6 | `j` | TONE | `[d,u,o,w,c,j]` | `[d,ư,ợ,c]` | `dược` | ✗ | ✓ | | | |
| 7 | `d` | STROKE | `[d,u,o,w,c,j,d]` | `[đ,ư,ợ,c]` | `được` | ✗ | ✓ | | | |
| 8 | `space` | COMMIT | `[]` | `[]` | `được ` | | ✓ | | | |

> **Note:** Raw contains 'w','j' (modifiers) → VN(R)=✗ → VN(B)=✓ (buffer valid) → VN mode.

---

## Example 2: `d u w o w c d j` + space → "được "

| # | Key | Action | Raw | Buffer | Output | VN(R) | VN(B) | EN(R) | EN(B) | Restore? |
|---|-----|--------|-----|--------|--------|-------|-------|-------|-------|----------|
| 1 | `d` | PASS | `[d]` | `[d]` | `d` | ✓ | | | | |
| 2 | `u` | PASS | `[d,u]` | `[d,u]` | `du` | ✓ | | | | |
| 3 | `w` | HORN | `[d,u,w]` | `[d,ư]` | `dư` | ✗ | ✓ | | | |
| 4 | `o` | PASS | `[d,u,w,o]` | `[d,ư,o]` | `dưo` | ✗ | ✓ | | | |
| 5 | `w` | HORN | `[d,u,w,o,w]` | `[d,ư,ơ]` | `dươ` | ✗ | ✓ | | | |
| 6 | `c` | PASS | `[d,u,w,o,w,c]` | `[d,ư,ơ,c]` | `dươc` | ✗ | ✓ | | | |
| 7 | `d` | STROKE | `[d,u,w,o,w,c,d]` | `[đ,ư,ơ,c]` | `đươc` | ✗ | ✓ | | | |
| 8 | `j` | TONE | `[d,u,w,o,w,c,d,j]` | `[đ,ư,ợ,c]` | `được` | ✗ | ✓ | | | |
| 9 | `space` | COMMIT | `[]` | `[]` | `được ` | | ✓ | | | |

> **Note:** Raw contains 'w','j' (modifiers) → VN(R)=✗ → VN(B)=✓ → VN mode.

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
| 3 | `x` | TONE | `[t,e,x]` | `[t,ẽ]` | `tẽ` | ✗ | ✓ | | | |
| 4 | `t` | RESTORE | `[t,e,x,t]` | `[t,e,x,t]` | `text` | ✗ | ✗ | ✓ | | **Triggered** |
| 5 | `space` | COMMIT | `[]` | `[]` | `text ` | | | ✓ | | |

> **Note:** Step 3: 'x' modifier → VN(R)=✗ → VN(B)=✓. Step 4: 'xt' impossible → VN(B)=✗ → EN(R)=✓ → RESTORE.

---

## Example 5: `e x p e c t` + space → "expect "

| # | Key | Action | Raw | Buffer | Output | VN(R) | VN(B) | EN(R) | EN(B) | Restore? |
|---|-----|--------|-----|--------|--------|-------|-------|-------|-------|----------|
| 1 | `e` | PASS | `[e]` | `[e]` | `e` | ✓ | | | | |
| 2 | `x` | TONE | `[e,x]` | `[ẽ]` | `ẽ` | ✗ | ✓ | | | |
| 3 | `p` | RESTORE | `[e,x,p]` | `[e,x,p]` | `exp` | ✗ | ✗ | ✓ | | **Triggered** |
| 4 | `e` | PASS | `[e,x,p,e]` | `[e,x,p,e]` | `expe` | | | ✓ | | |
| 5 | `c` | PASS | `[e,x,p,e,c]` | `[e,x,p,e,c]` | `expec` | | | ✓ | | |
| 6 | `t` | PASS | `[e,x,p,e,c,t]` | `[e,x,p,e,c,t]` | `expect` | | | ✓ | | |
| 7 | `space` | COMMIT | `[]` | `[]` | `expect ` | | | ✓ | | |

> **Note:** Step 2: 'x' modifier → VN(R)=✗ → VN(B)=✓. Step 3: 'xp' → VN(B)=✗ → EN(R)=✓ → RESTORE.

---

## Example 6: `v a r` + space → "vả " (VN wins)

| # | Key | Action | Raw | Buffer | Output | VN(R) | VN(B) | EN(R) | EN(B) | Restore? |
|---|-----|--------|-----|--------|--------|-------|-------|-------|-------|----------|
| 1 | `v` | PASS | `[v]` | `[v]` | `v` | ✓ | | | | |
| 2 | `a` | PASS | `[v,a]` | `[v,a]` | `va` | ✓ | | | | |
| 3 | `r` | TONE | `[v,a,r]` | `[v,ả]` | `vả` | ✗ | ✓ | | | |
| 4 | `space` | COMMIT | `[]` | `[]` | `vả ` | | ✓ | | | |

> **Note:** 'r' modifier → VN(R)=✗ → VN(B)=✓ ("vả" valid) → VN wins.

---

## Example 7: `v a r r` + space → "var " (double-key revert)

| # | Key | Action | Raw | Buffer | Output | VN(R) | VN(B) | EN(R) | EN(B) | Restore? |
|---|-----|--------|-----|--------|--------|-------|-------|-------|-------|----------|
| 1 | `v` | PASS | `[v]` | `[v]` | `v` | ✓ | | | | |
| 2 | `a` | PASS | `[v,a]` | `[v,a]` | `va` | ✓ | | | | |
| 3 | `r` | TONE | `[v,a,r]` | `[v,ả]` | `vả` | ✗ | ✓ | | | |
| 4 | `r` | REVERT | `[v,a,r]` | `[v,a,r]` | `var` | ✗ | ✗ | ✓ | | |
| 5 | `space` | COMMIT | `[]` | `[]` | `var ` | | | ✓ | | |

> **Note:** Step 3: 'r' modifier → VN(B)=✓. Step 4: 'rr' double-key → VN(B)=✗ → REVERT → EN mode.

---

## Example 8: `d e n d e s n n n n` + space → "đếnnnnn "

| # | Key | Action | Raw | Buffer | Output | VN(R) | VN(B) | EN(R) | EN(B) | Restore? |
|---|-----|--------|-----|--------|--------|-------|-------|-------|-------|----------|
| 1 | `d` | PASS | `[d]` | `[d]` | `d` | ✓ | | | | |
| 2 | `e` | PASS | `[d,e]` | `[d,e]` | `de` | ✓ | | | | |
| 3 | `n` | PASS | `[d,e,n]` | `[d,e,n]` | `den` | ✓ | | | | |
| 4 | `d` | STROKE | `[d,e,n,d]` | `[đ,e,n]` | `đen` | ✓ | | | | |
| 5 | `e` | CIRCUM | `[d,e,n,d,e]` | `[đ,ê,n]` | `đên` | ✓ | | | | |
| 6 | `s` | TONE | `[d,e,n,d,e,s]` | `[đ,ế,n]` | `đến` | ✗ | ✓ | | | |
| 7 | `n` | PASS | `[d,e,n,d,e,s,n]` | `[đ,ế,n,n]` | `đếnn` | ✗ | ✓ | | | |
| 8 | `n` | PASS | `[...,n,n]` | `[đ,ế,n,n,n]` | `đếnnn` | ✗ | ✓ | | | |
| 9 | `n` | PASS | `[...,n,n,n]` | `[...,n,n,n,n]` | `đếnnnn` | ✗ | ✓ | | | |
| 10 | `n` | PASS | `[...,n,n,n,n]` | `[...,n,n,n,n,n]` | `đếnnnnn` | ✗ | ✓ | | | |
| 11 | `space` | COMMIT | `[]` | `[]` | `đếnnnnn ` | | ✓ | | | |

> **Note:**
> - `dd` stroke, `ee` circumflex work non-consecutively
> - Step 6+: 's' modifier → VN(R)=✗ → VN(B)=✓ (buffer valid) → VN mode

---

## Example 9: `t o t o s` + space → "tốt "

| # | Key | Action | Raw | Buffer | Output | VN(R) | VN(B) | EN(R) | EN(B) | Restore? |
|---|-----|--------|-----|--------|--------|-------|-------|-------|-------|----------|
| 1 | `t` | PASS | `[t]` | `[t]` | `t` | ✓ | | | | |
| 2 | `o` | PASS | `[t,o]` | `[t,o]` | `to` | ✓ | | | | |
| 3 | `t` | PASS | `[t,o,t]` | `[t,o,t]` | `tot` | ✓ | | | | |
| 4 | `o` | CIRCUM | `[t,o,t,o]` | `[t,ô,t]` | `tôt` | ✓ | | | | |
| 5 | `s` | TONE | `[t,o,t,o,s]` | `[t,ố,t]` | `tốt` | ✗ | ✓ | | | |
| 6 | `space` | COMMIT | `[]` | `[]` | `tốt ` | | ✓ | | | |

> **Note:** 'oo' forms circumflex (ô) non-consecutively. Step 5: 's' modifier → VN(R)=✗ → VN(B)=✓.

---

## Example 10: `c o n s o l e` + space → "console "

| # | Key | Action | Raw | Buffer | Output | VN(R) | VN(B) | EN(R) | EN(B) | Restore? |
|---|-----|--------|-----|--------|--------|-------|-------|-------|-------|----------|
| 1 | `c` | PASS | `[c]` | `[c]` | `c` | ✓ | | | | |
| 2 | `o` | PASS | `[c,o]` | `[c,o]` | `co` | ✓ | | | | |
| 3 | `n` | PASS | `[c,o,n]` | `[c,o,n]` | `con` | ✓ | | | | |
| 4 | `s` | TONE | `[c,o,n,s]` | `[c,ó,n]` | `cón` | ✗ | ✓ | | | |
| 5 | `o` | PASS | `[c,o,n,s,o]` | `[c,ó,n,o]` | `cóno` | ✗ | ✓ | | | |
| 6 | `l` | RESTORE | `[c,o,n,s,o,l]` | `[c,o,n,s,o,l]` | `consol` | ✗ | ✗ | ✓ | | **Triggered** |
| 7 | `e` | PASS | `[c,o,n,s,o,l,e]` | `[c,o,n,s,o,l,e]` | `console` | | | ✓ | | |
| 8 | `space` | COMMIT | `[]` | `[]` | `console ` | | | ✓ | | |

> **Note:** Step 4+: 's' modifier → VN(R)=✗. Step 6: 'l' invalid final → VN(B)=✗ → RESTORE.

### Example 10a: `c o n s s o l e` + space → "console " (double-key revert)

| # | Key | Action | Raw | Buffer | Output | VN(R) | VN(B) | EN(R) | EN(B) | Restore? |
|---|-----|--------|-----|--------|--------|-------|-------|-------|-------|----------|
| 1 | `c` | PASS | `[c]` | `[c]` | `c` | ✓ | | | | |
| 2 | `o` | PASS | `[c,o]` | `[c,o]` | `co` | ✓ | | | | |
| 3 | `n` | PASS | `[c,o,n]` | `[c,o,n]` | `con` | ✓ | | | | |
| 4 | `s` | TONE | `[c,o,n,s]` | `[c,ó,n]` | `cón` | ✗ | ✓ | | | |
| 5 | `s` | REVERT | `[c,o,n,s]` | `[c,o,n,s]` | `cons` | ✗ | ✗ | ✓ | | |
| 6 | `o` | PASS | `[c,o,n,s,o]` | `[c,o,n,s,o]` | `conso` | | | ✓ | | |
| 7 | `l` | PASS | `[c,o,n,s,o,l]` | `[c,o,n,s,o,l]` | `consol` | | | ✓ | | |
| 8 | `e` | PASS | `[c,o,n,s,o,l,e]` | `[c,o,n,s,o,l,e]` | `console` | | | ✓ | | |
| 9 | `space` | COMMIT | `[]` | `[]` | `console ` | | | ✓ | | |

> **Note:** Step 4: 's' modifier → VN(R)=✗ → VN(B)=✓ ("cón" valid). Step 5: 'ss' double-key → REVERT → EN mode. Steps 6-8 continue in EN mode.

---

## Example 11: `d i s s t` + space → "dist "

| # | Key | Action | Raw | Buffer | Output | VN(R) | VN(B) | EN(R) | EN(B) | Restore? |
|---|-----|--------|-----|--------|--------|-------|-------|-------|-------|----------|
| 1 | `d` | PASS | `[d]` | `[d]` | `d` | ✓ | | | | |
| 2 | `i` | PASS | `[d,i]` | `[d,i]` | `di` | ✓ | | | | |
| 3 | `s` | TONE | `[d,i,s]` | `[d,í]` | `dí` | ✗ | ✓ | | | |
| 4 | `s` | REVERT | `[d,i,s]` | `[d,i,s]` | `dis` | ✗ | ✗ | ✓ | | |
| 5 | `t` | PASS | `[d,i,s,t]` | `[d,i,s,t]` | `dist` | | | ✓ | | |
| 6 | `space` | COMMIT | `[]` | `[]` | `dist ` | | | ✓ | | |

> **Note:** Step 3: 's' modifier → VN(R)=✗ → VN(B)=✓. Step 4: 'ss' double-key → REVERT → EN mode.

---

## Example 11a: `m i s s s` + space → "miss " (triple-key for double letter)

| # | Key | Action | Raw | Buffer | Output | VN(R) | VN(B) | EN(R) | EN(B) | Restore? |
|---|-----|--------|-----|--------|--------|-------|-------|-------|-------|----------|
| 1 | `m` | PASS | `[m]` | `[m]` | `m` | ✓ | | | | |
| 2 | `i` | PASS | `[m,i]` | `[m,i]` | `mi` | ✓ | | | | |
| 3 | `s` | TONE | `[m,i,s]` | `[m,í]` | `mí` | ✗ | ✓ | | | |
| 4 | `s` | REVERT | `[m,i,s]` | `[m,i,s]` | `mis` | ✗ | ✗ | ✓ | | |
| 5 | `s` | PASS | `[m,i,s,s]` | `[m,i,s,s]` | `miss` | | | ✓ | | |
| 6 | `space` | COMMIT | `[]` | `[]` | `miss ` | | | ✓ | | |

> **Note:**
> - Step 4: 'ss' REVERT → Raw synced với Buffer (bỏ trigger key)
> - Step 5: Third 's' = literal trong EN mode
> - Pattern: `ss` = revert + 1 literal, `sss` = revert + 2 literal

---

## Example 11b: `k e e p` + space → "keep " (commit-time validation)

| # | Key | Action | Raw | Buffer | Output | VN(R) | VN(B) | EN(R) | EN(B) | Restore? |
|---|-----|--------|-----|--------|--------|-------|-------|-------|-------|----------|
| 1 | `k` | PASS | `[k]` | `[k]` | `k` | ✓ | | | | |
| 2 | `e` | PASS | `[k,e]` | `[k,e]` | `ke` | ✓ | | | | |
| 3 | `e` | CIRCUM | `[k,e,e]` | `[k,ê]` | `kê` | ✓ | | | | |
| 4 | `p` | PASS | `[k,e,e,p]` | `[k,ê,p]` | `kêp` | ✓ | | | | |
| 5 | `space` | VALIDATE | | | | | ✗ | | | "kêp" invalid |
| 5 | `space` | RESTORE+COMMIT | `[]` | `[]` | `keep ` | | | ✓ | | **Triggered** |

> **Note:**
> - Step 5: VALIDATE tại commit time → "kêp" không valid trong Vietnamese
> - Buffer fallback về Raw → "keep"
> - Commit "keep " (EN mode)

---

## Example 12: `d u o w n g o d` + space → "đuông "

| # | Key | Action | Raw | Buffer | Output | VN(R) | VN(B) | EN(R) | EN(B) | Restore? |
|---|-----|--------|-----|--------|--------|-------|-------|-------|-------|----------|
| 1 | `d` | PASS | `[d]` | `[d]` | `d` | ✓ | | | | |
| 2 | `u` | PASS | `[d,u]` | `[d,u]` | `du` | ✓ | | | | |
| 3 | `o` | PASS | `[d,u,o]` | `[d,u,o]` | `duo` | ✓ | | | | |
| 4 | `w` | HORN | `[d,u,o,w]` | `[d,ư,ơ]` | `dươ` | ✗ | ✓ | | | |
| 5 | `n` | PASS | `[d,u,o,w,n]` | `[d,ư,ơ,n]` | `dươn` | ✗ | ✓ | | | |
| 6 | `g` | PASS | `[d,u,o,w,n,g]` | `[d,ư,ơ,n,g]` | `dương` | ✗ | ✓ | | | |
| 7 | `o` | CIRCUM | `[d,u,o,w,n,g,o]` | `[d,u,ô,n,g]` | `duông` | ✗ | ✓ | | | |
| 8 | `d` | STROKE | `[d,u,o,w,n,g,o,d]` | `[đ,u,ô,n,g]` | `đuông` | ✗ | ✓ | | | |
| 9 | `space` | COMMIT | `[]` | `[]` | `đuông ` | | ✓ | | | |

> **Note:**
> - 'w' applies horn to both u→ư and o→ơ
> - 'oo' circumflex overrides horn (ơ→ô, ư→u)
> - 'dd' stroke applies non-consecutively
> - Step 4+: 'w' modifier → VN(R)=✗ → VN(B)=✓ → VN mode

---

## Column Legend

| Column | Description |
|--------|-------------|
| Raw | `RawBuffer` - keystroke gốc (chưa transform) |
| Buffer | `Buffer` - ký tự đã transform (có dấu) |
| VN(R) | Raw cluster validation: ✓=valid raw, ✗=invalid (has modifier keys: w,j,s,f,r,x) |
| VN(B) | Buffer validation (only when VN(R)=✗): ✓=valid syllable, ✗=invalid |
| EN(R) | English mode via Raw |
| EN(B) | English mode via Buffer |
| Restore? | (empty)=no, **Triggered**=executed immediately |

### Validation Flow

```
VN(R) → if ✗ → VN(B) → if ✗ → EN(R) → if ✓ → RESTORE/FOREIGN → EN mode
  ↓                        ↓            ↓
  ✓                        ✓            ✗
  ↓                        ↓            ↓
  continue VN              continue VN  continue VN (keep buffer)
```

### Validation Checks

| Check Type | Column | Examples |
|------------|--------|----------|
| Modifier key in raw | VN(R)=✗ | `w`, `j`, `s`, `f`, `r`, `x` present |
| Impossible cluster | VN(R)=✗ | `cl`, `xp`, `xt` |
| Double-key revert | VN(R)=✗ | `ss`, `rr`, `ff` |
| Invalid final consonant | VN(B)=✗ | `l`, `r` in final |
| Invalid syllable structure | VN(B)=✗ | vowel-consonant patterns |

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

| Condition | VN(R) | VN(B) | Result |
|-----------|-------|-------|--------|
| Valid raw (no modifiers) | ✓ | | VN mode |
| Modifier + valid buffer | ✗ | ✓ | VN mode |
| Impossible cluster (`xt`, `xp`) | ✗ | ✗ | RESTORE |
| Foreign cluster early (`cl`) | ✗ | ✗ | FOREIGN |
| Invalid final (`l`, `r`) | ✗ | ✗ | RESTORE |
| Double-key (`ss`, `rr`) | ✗ | ✗ | REVERT → EN |

---

## Commit-time Validation

Khi user nhấn SPACE (commit), engine thực hiện final validation:

```
COMMIT Flow:
1. Kiểm tra Buffer có phải Vietnamese word valid không
2. Nếu invalid → RESTORE về Raw → commit Raw
3. Nếu valid → commit Buffer
```

### Circumflex Syllable Rules

| Pattern | Example | Valid? | Reason |
|---------|---------|--------|--------|
| Open syllable (no final) | `kê`, `bê` | ✓ | Circumflex alone OK |
| Closed + tone | `kép`, `bếp`, `kếp` | ✓ | Has tone mark |
| Closed + no tone | `kêp`, `bêp` | ✗ | Missing tone → invalid |

### Commit-time Restore Examples

| Input | Buffer | Valid? | Output |
|-------|--------|--------|--------|
| `k e e p` | `kêp` | ✗ | `keep` (restore) |
| `k e e s p` | `kếp` | ✓ | `kếp` (commit buffer) |
| `b e e p` | `bêp` | ✗ | `beep` (restore) |
| `b e e s p` | `bếp` | ✓ | `bếp` (commit buffer) |

> **Note:** Commit-time validation chỉ apply khi VN(R)=✓ (no modifier keys).
> Nếu VN(R)=✗, validation đã xảy ra real-time qua VN(B) check.

---

## Comparison

| Sequence | Keys | Result | VN(R) | VN(B) | EN |
|----------|------|--------|-------|-------|-----|
| `d u o w c j d` + space | 8 | `được ` | ✗ | ✓ | |
| `d u w o w c d j` + space | 9 | `được ` | ✗ | ✓ | |
| `c l a r` + space | 5 | `clar ` | ✗ | ✗ | ✓ |
| `t e x t` + space | 5 | `text ` | ✗ | ✗ | ✓ |
| `e x p e c t` + space | 7 | `expect ` | ✗ | ✗ | ✓ |
| `v a r` + space | 4 | `vả ` | ✗ | ✓ | |
| `v a r r` + space | 5 | `var ` | ✗ | ✗ | ✓ |
| `d e n d e s n n n n` + space | 11 | `đếnnnnn ` | ✗ | ✓ | |
| `t o t o s` + space | 6 | `tốt ` | ✗ | ✓ | |
| `c o n s o l e` + space | 8 | `console ` | ✗ | ✗ | ✓ |
| `c o n s s o l e` + space | 9 | `console ` | ✗ | ✗ | ✓ |
| `d i s s t` + space | 6 | `dist ` | ✗ | ✗ | ✓ |
| `m i s s s` + space | 6 | `miss ` | ✗ | ✗ | ✓ |
| `k e e p` + space | 5 | `keep ` | ✓ | ✗ | ✓ |
| `d u o w n g o d` + space | 9 | `đuông ` | ✗ | ✓ | |
