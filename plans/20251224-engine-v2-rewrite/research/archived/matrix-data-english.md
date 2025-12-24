# English Matrix Data - Complete Implementation Reference

**Date**: 2025-12-24
**Sources**: English phonotactics research, linguistic databases

---

## E1: ONSET_SINGLE Matrix (26×1)

Which single letters can start English words.

```rust
/// Single consonant onset validity
/// 0=INVALID, 1=VALID, 2=RARE
pub static E_ONSET_SINGLE: [u8; 26] = [
//  a  b  c  d  e  f  g  h  i  j  k  l  m  n  o  p  q  r  s  t  u  v  w  x  y  z
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 2, 1, 1
];
// Notes: q=rare (always qu), x=rare (xylophone)
```

---

## E2: ONSET_CC Matrix (26×26)

Valid two-letter onset consonant clusters.

```rust
/// Onset consonant cluster validity
/// Rows: C1 (first consonant), Cols: C2 (second consonant)
/// 0=INVALID, 1=VALID
///          a  b  c  d  e  f  g  h  i  j  k  l  m  n  o  p  q  r  s  t  u  v  w  x  y  z
pub static E_ONSET_CC: [[u8; 26]; 26] = [
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // a (vowel)
    [0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,1,0,0,0,0,0,0,0,0], // b: bl,br
    [0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,1,0,0,0,0,0,0,0,0], // c: cl,cr
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,1,0,0,0], // d: dr,dw
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // e (vowel)
    [0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,1,0,0,0,0,0,0,0,0], // f: fl,fr
    [0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,1,0,0,0,0,1,0,0,0], // g: gl,gr,gw (rare)
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // h: (no clusters)
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // i (vowel)
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // j: (no clusters)
    [0,0,0,0,0,0,0,0,0,0,0,1,0,1,0,0,0,0,0,0,0,0,1,0,0,0], // k: kl(rare),kn,kw
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // l: (no onset clusters)
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // m: (no onset clusters)
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // n: (no onset clusters)
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // o (vowel)
    [0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,1,0,0,0,0,0,0,0,0], // p: pl,pr
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0], // q: qu
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // r: (no onset clusters)
    [0,0,1,0,0,0,0,1,0,0,1,1,1,1,0,1,0,0,0,1,0,0,1,0,0,0], // s: sc,sh,sk,sl,sm,sn,sp,st,sw
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,1,0,0,0], // t: tr,tw
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // u (vowel)
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // v: (no onset clusters)
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0], // w: wr
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // x: (no onset clusters)
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // y: (no onset clusters)
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // z: (no onset clusters)
];

/// Summary of valid onset clusters:
/// bl, br, cl, cr, dr, dw, fl, fr, gl, gr, gw (rare)
/// kl (rare), kn, kw, pl, pr, qu
/// sc, sh, sk, sl, sm, sn, sp, st, sw
/// tr, tw, wr
```

---

## E3: ONSET_CCC - Valid Triple Onset Clusters

Only s-clusters are valid.

```rust
/// Triple onset cluster patterns (only s-initial)
pub static E_ONSET_CCC: &[&[u8; 3]] = &[
    b"scr", // scream, script
    b"spl", // split, splash
    b"spr", // spring, spread
    b"squ", // square, squat
    b"str", // string, strong
];

/// Check if triple onset is valid
pub fn is_valid_onset_ccc(c1: u8, c2: u8, c3: u8) -> bool {
    if c1 != b's' { return false; }
    match (c2, c3) {
        (b'c', b'r') => true, // scr
        (b'p', b'l') => true, // spl
        (b'p', b'r') => true, // spr
        (b'q', b'u') => true, // squ
        (b't', b'r') => true, // str
        _ => false
    }
}
```

---

## E4: CODA_SINGLE Matrix (26×1)

Which single letters can end English words.

```rust
/// Single consonant coda validity
/// 0=INVALID, 1=VALID
pub static E_CODA_SINGLE: [u8; 26] = [
//  a  b  c  d  e  f  g  h  i  j  k  l  m  n  o  p  q  r  s  t  u  v  w  x  y  z
    0, 1, 1, 1, 0, 1, 1, 1, 0, 0, 1, 1, 1, 1, 0, 1, 0, 1, 1, 1, 0, 0, 1, 1, 0, 1
];
// Valid endings: b,c,d,f,g,h,k,l,m,n,p,r,s,t,w,x,z
// Invalid endings: a,e,i,j,o,q,u,v,y
```

---

## E5: CODA_CC Matrix (26×26)

Valid two-letter coda consonant clusters.

```rust
/// Coda consonant cluster validity
/// Rows: C1 (first consonant), Cols: C2 (second consonant)
/// 0=INVALID, 1=VALID
///          a  b  c  d  e  f  g  h  i  j  k  l  m  n  o  p  q  r  s  t  u  v  w  x  y  z
pub static E_CODA_CC: [[u8; 26]; 26] = [
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // a (vowel)
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0], // b: bs (ribs)
    [0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0], // c: ck,ct
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,1,0,0,0,0,0,0], // d: ds,dt (rare)
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // e (vowel)
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0], // f: ft
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0], // g: gs (dogs)
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // h: rarely in clusters
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // i (vowel)
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // j: no codas
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0], // k: ks (books)
    [0,0,0,1,0,1,0,0,0,0,1,0,1,0,0,1,0,0,1,1,0,0,0,0,0,0], // l: ld,lf,lk,lm,lp,ls,lt
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,1,0,0,0,0,0,0,0], // m: mp,ms
    [0,0,1,1,0,0,1,0,0,0,1,0,0,0,0,0,0,0,1,1,0,0,0,0,0,0], // n: nc,nd,ng,nk,ns,nt
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // o (vowel)
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,1,0,0,0,0,0,0], // p: ps,pt
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // q: no codas
    [0,0,1,1,0,1,1,0,0,0,1,1,1,1,0,1,0,0,1,1,0,0,0,0,0,0], // r: rc,rd,rf,rg,rk,rl,rm,rn,rp,rs,rt
    [0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,1,0,0,0,1,0,0,0,0,0,0], // s: sk,sp,st
    [0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0], // t: th,ts
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // u (vowel)
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // v: no codas
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // w: rarely in clusters
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0], // x: xt (next)
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // y: no codas
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // z: rare
];

/// Summary of valid coda clusters:
/// bs, ck, ct, ds, ft, gs, ks
/// ld, lf, lk, lm, lp, ls, lt
/// mp, ms
/// nc, nd, ng, nk, ns, nt
/// ps, pt
/// rc, rd, rf, rg, rk, rl, rm, rn, rp, rs, rt
/// sk, sp, st, th, ts, xt
```

---

## E6: CODA_CCC - Valid Triple Coda Clusters

```rust
/// Triple coda cluster patterns
pub static E_CODA_CCC: &[&[u8; 3]] = &[
    b"nds", // hands, lands
    b"ngs", // rings, songs
    b"nks", // banks, links
    b"nts", // ants, pants
    b"rks", // works, parks
    b"rms", // arms, forms
    b"rns", // burns, turns
    b"rps", // warps, carps
    b"rts", // arts, parts
    b"sts", // tests, lists
    b"lfs", // elfs, golfs
    b"lks", // walks, talks
    b"lms", // films, helms
    b"lps", // helps, yelps
    b"lts", // melts, belts
    b"mps", // jumps, bumps
    b"fts", // lifts, gifts
    b"sks", // asks, masks
    b"sps", // gasps, clasps
];

/// Check if triple coda is valid
pub fn is_valid_coda_ccc(c1: u8, c2: u8, c3: u8) -> bool {
    // Most end in 's' for plurals/conjugation
    if c3 != b's' && c3 != b't' { return false; }

    // Check known patterns
    for pattern in E_CODA_CCC.iter() {
        if pattern[0] == c1 && pattern[1] == c2 && pattern[2] == c3 {
            return true;
        }
    }
    false
}
```

---

## E7: IMPOSSIBLE_BIGRAM Matrix (26×26)

Letter pairs that never occur in English.

```rust
/// Impossible bigram matrix
/// 1 = IMPOSSIBLE (never occurs), 0 = POSSIBLE
///          a  b  c  d  e  f  g  h  i  j  k  l  m  n  o  p  q  r  s  t  u  v  w  x  y  z
pub static E_IMPOSSIBLE_BIGRAM: [[u8; 26]; 26] = [
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0], // a: aq impossible
    [0,1,0,1,0,1,1,0,0,1,1,0,1,0,0,1,1,0,1,0,0,1,1,1,0,1], // b: bb,bd,bf,bg,bj,bk,bm,bp,bq,bs,bv,bw,bx,bz
    [0,1,0,1,0,1,1,0,0,1,0,0,1,0,0,1,1,0,1,0,0,1,1,1,0,1], // c: cb,cd,cf,cg,cj,cm,cp,cq,cs,cv,cw,cx,cz
    [0,1,1,1,0,1,0,0,0,1,1,0,1,0,0,1,1,0,0,1,0,1,1,1,0,1], // d: db,dc,dd,df,dj,dk,dm,dp,dq,dt,dv,dw,dx,dz
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0], // e: eq impossible
    [0,1,1,1,0,0,1,0,0,1,1,0,1,0,0,1,1,0,1,0,0,1,1,1,0,1], // f: fb,fc,fd,fg,fj,fk,fm,fp,fq,fs,fv,fw,fx,fz
    [0,1,1,1,0,1,0,0,0,1,1,0,1,0,0,1,1,0,1,1,0,1,1,1,0,1], // g: gb,gc,gd,gf,gj,gk,gm,gp,gq,gs,gt,gv,gw,gx,gz
    [0,1,1,1,0,1,1,0,0,1,1,0,1,0,0,1,1,0,1,1,0,1,1,1,0,1], // h: hb,hc,hd,hf,hg,hj,hk,hm,hp,hq,hs,ht,hv,hw,hx,hz
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0], // i: iq impossible
    [0,1,1,1,0,1,1,0,0,0,1,1,1,1,0,1,1,1,1,1,0,1,1,1,1,1], // j: most combos impossible
    [0,1,1,1,0,1,1,0,0,1,0,0,1,0,0,1,1,0,0,1,0,1,0,1,0,1], // k: kb,kc,kd,kf,kg,kj,km,kp,kq,kt,kv,kx,kz
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0], // l: lq impossible
    [0,0,0,1,0,0,1,0,0,1,1,0,0,0,0,0,1,0,0,0,0,1,1,1,0,1], // m: md,mg,mj,mk,mq,mv,mw,mx,mz
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,1,0,0,0,0,1,0,1,0,0], // n: np,nq,nv,nx
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0], // o: oq impossible
    [0,1,1,1,0,1,1,0,0,1,1,0,1,0,0,0,1,0,0,0,0,1,1,1,0,1], // p: pb,pc,pd,pf,pg,pj,pk,pm,pq,pv,pw,px,pz
    [1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,0,1,1,1,1,1], // q: only qu valid
    [0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0], // r: rj,rq impossible
    [0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,1,0,0,0,0,1,0,1,0,0], // s: sj,sq,sv,sx
    [0,1,1,1,0,1,1,0,0,1,1,0,1,0,0,1,1,0,0,0,0,1,0,1,0,1], // t: tb,tc,td,tf,tg,tj,tk,tm,tp,tq,tv,tx,tz
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0], // u: uq impossible
    [0,1,1,1,0,1,1,0,0,1,1,1,1,0,0,1,1,0,1,1,0,0,1,1,0,1], // v: vb,vc,vd,vf,vg,vj,vk,vl,vm,vp,vq,vs,vt,vw,vx,vz
    [0,1,1,1,0,1,1,0,0,1,1,1,1,0,0,1,1,0,1,1,0,1,1,1,0,1], // w: most combos impossible
    [0,1,1,1,0,1,1,0,0,1,1,1,1,1,0,1,1,1,1,1,0,1,1,0,0,1], // x: most combos impossible
    [0,1,1,1,0,1,1,1,0,1,1,1,1,1,0,1,1,1,1,1,0,1,1,1,0,1], // y: many combos impossible
    [0,1,1,1,0,1,1,0,0,1,1,1,1,1,0,1,1,1,1,1,0,1,1,1,0,0], // z: most combos impossible
];

/// Common impossible patterns (quick check):
pub const IMPOSSIBLE_PATTERNS: &[&str] = &[
    "bx", "cj", "cx", "dq", "dx", "fq", "fx", "gq", "gx",
    "hq", "hx", "jb", "jc", "jd", "jf", "jg", "jk", "jl",
    "jm", "jn", "jp", "jq", "jr", "js", "jt", "jv", "jw",
    "jx", "jy", "jz", "kq", "kx", "mq", "mx", "pq", "px",
    "qa", "qb", "qc", "qd", "qe", "qf", "qg", "qh", "qi",
    "qj", "qk", "ql", "qm", "qn", "qo", "qp", "qq", "qr",
    "qs", "qt", "qv", "qw", "qx", "qy", "qz", "rq", "sq",
    "vq", "vx", "wq", "wx", "xq", "yq", "zq", "zx"
];
```

---

## E8: VOWEL_DIGRAPH Matrix (5×5)

Valid English vowel pairs.

```rust
/// Vowel digraph validity
/// Rows: V1 (first vowel), Cols: V2 (second vowel)
/// 0=INVALID, 1=VALID, 2=RARE
///          a  e  i  o  u
pub static E_VOWEL_DIGRAPH: [[u8; 5]; 5] = [
    [0, 1, 1, 0, 1], // a: ae(rare),ai,au
    [1, 1, 1, 0, 1], // e: ea,ee,ei,eu
    [0, 1, 0, 0, 0], // i: ie
    [1, 1, 1, 1, 1], // o: oa,oe,oi,oo,ou
    [0, 1, 1, 0, 0], // u: ue,ui
];

/// Common English vowel digraphs
pub const VOWEL_DIGRAPHS: &[&str] = &[
    "ai", "au", "ay",  // rain, cause, day
    "ea", "ee", "ei", "eo", "eu", "ey",  // team, see, vein, people, feud, key
    "ie",  // field
    "oa", "oe", "oi", "oo", "ou", "ow", "oy",  // boat, toe, oil, food, out, now, boy
    "ue", "ui",  // blue, fruit
];
```

---

## P1: WORD_START Matrix (26×1)

Which letters can start English words.

```rust
/// Word start validity
/// 0=NEVER, 1=COMMON, 2=RARE
pub static E_WORD_START: [u8; 26] = [
//  a  b  c  d  e  f  g  h  i  j  k  l  m  n  o  p  q  r  s  t  u  v  w  x  y  z
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1
];
// x is rare as word start (xylophone, x-ray)
```

---

## P2: WORD_END Matrix (26×1)

Which letters can end English words.

```rust
/// Word end validity
/// 0=NEVER, 1=COMMON, 2=RARE
pub static E_WORD_END: [u8; 26] = [
//  a  b  c  d  e  f  g  h  i  j  k  l  m  n  o  p  q  r  s  t  u  v  w  x  y  z
    1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 0, 1, 1, 1, 1
];
// j, q, v rarely end words
// Some exceptions: Raj, Iraq (proper nouns)
```

---

## MORPHOLOGY: Common Suffix Patterns

```rust
/// English morphological patterns for validation bonus
pub const SUFFIXES: &[(&str, &str)] = &[
    // Derivational
    ("tion", "noun"),    // nation, action
    ("sion", "noun"),    // vision, tension
    ("ness", "noun"),    // happiness, darkness
    ("ment", "noun"),    // movement, agreement
    ("able", "adj"),     // capable, readable
    ("ible", "adj"),     // possible, visible
    ("ful", "adj"),      // beautiful, careful
    ("less", "adj"),     // careless, hopeless
    ("ous", "adj"),      // famous, dangerous
    ("ive", "adj"),      // active, creative
    ("ly", "adv"),       // quickly, slowly
    // Inflectional
    ("ing", "verb"),     // running, swimming
    ("ed", "verb"),      // walked, talked
    ("er", "comp"),      // bigger, faster
    ("est", "super"),    // biggest, fastest
    ("'s", "poss"),      // John's, dog's
    ("s", "plural"),     // cats, dogs
];

/// Check if word ends with valid suffix
pub fn has_valid_suffix(word: &str) -> Option<&'static str> {
    for (suffix, category) in SUFFIXES {
        if word.ends_with(suffix) {
            return Some(category);
        }
    }
    None
}
```

---

## Usage Example

```rust
pub fn validate_english_word(word: &str) -> EnglishValidation {
    let bytes = word.as_bytes();
    let len = bytes.len();

    if len == 0 { return EnglishValidation::Empty; }

    // Step 1: Check impossible bigrams via matrix
    for i in 0..len-1 {
        let c1 = (bytes[i] - b'a') as usize;
        let c2 = (bytes[i+1] - b'a') as usize;
        if c1 < 26 && c2 < 26 && E_IMPOSSIBLE_BIGRAM[c1][c2] == 1 {
            return EnglishValidation::ImpossibleBigram(
                bytes[i] as char,
                bytes[i+1] as char
            );
        }
    }

    // Step 2: Check triple letters (never valid)
    for i in 0..len-2 {
        if bytes[i] == bytes[i+1] && bytes[i+1] == bytes[i+2] {
            return EnglishValidation::TripleLetter(bytes[i] as char);
        }
    }

    // Step 3: Check word start via matrix
    let first = (bytes[0] - b'a') as usize;
    if first < 26 && E_WORD_START[first] == 0 {
        return EnglishValidation::InvalidStart(bytes[0] as char);
    }

    // Step 4: Check word end via matrix
    let last = (bytes[len-1] - b'a') as usize;
    if last < 26 && E_WORD_END[last] == 0 {
        return EnglishValidation::InvalidEnd(bytes[len-1] as char);
    }

    // Step 5: Check onset cluster (first consonants)
    if let Some((onset_end, _)) = find_first_vowel(bytes) {
        if onset_end >= 2 {
            let c1 = (bytes[0] - b'a') as usize;
            let c2 = (bytes[1] - b'a') as usize;
            if c1 < 26 && c2 < 26 && E_ONSET_CC[c1][c2] == 0 {
                // Check for triple onset
                if onset_end >= 3 {
                    if !is_valid_onset_ccc(bytes[0], bytes[1], bytes[2]) {
                        return EnglishValidation::InvalidOnset(
                            String::from_utf8_lossy(&bytes[0..3]).to_string()
                        );
                    }
                } else {
                    return EnglishValidation::InvalidOnset(
                        String::from_utf8_lossy(&bytes[0..2]).to_string()
                    );
                }
            }
        }
    }

    // Step 6: Check coda cluster (final consonants)
    if let Some((_, vowel_end)) = find_last_vowel(bytes) {
        let coda_start = vowel_end + 1;
        let coda_len = len - coda_start;
        if coda_len >= 2 {
            let c1 = (bytes[coda_start] - b'a') as usize;
            let c2 = (bytes[coda_start+1] - b'a') as usize;
            if c1 < 26 && c2 < 26 && E_CODA_CC[c1][c2] == 0 {
                return EnglishValidation::InvalidCoda(
                    String::from_utf8_lossy(&bytes[coda_start..]).to_string()
                );
            }
        }
    }

    // Step 7: Must have at least one vowel
    if !bytes.iter().any(|&b| matches!(b, b'a'|b'e'|b'i'|b'o'|b'u'|b'y')) {
        return EnglishValidation::NoVowel;
    }

    // Step 8: Bonus - check morphology
    if has_valid_suffix(word).is_some() {
        return EnglishValidation::ValidWithMorphology;
    }

    EnglishValidation::PossiblyValid
}
```

---

## Performance Analysis

| Matrix | Size | Memory | Lookup |
|--------|------|--------|--------|
| E_ONSET_SINGLE | 26×1 | 26 bytes | O(1) |
| E_ONSET_CC | 26×26 | 676 bytes | O(1) |
| E_CODA_SINGLE | 26×1 | 26 bytes | O(1) |
| E_CODA_CC | 26×26 | 676 bytes | O(1) |
| E_IMPOSSIBLE_BIGRAM | 26×26 | 676 bytes | O(1) |
| E_VOWEL_DIGRAPH | 5×5 | 25 bytes | O(1) |
| E_WORD_START | 26×1 | 26 bytes | O(1) |
| E_WORD_END | 26×1 | 26 bytes | O(1) |
| **Total** | - | **~2.1KB** | O(1) |

All operations are O(1) matrix lookups. Full word validation is O(n) where n = word length.
