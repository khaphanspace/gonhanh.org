#!/usr/bin/env python3
"""Generate core/src/phonology/tables.rs from the dictionaries (single source of truth = data).

  nucleus x coda matrix  <- core/src/data/dictionaries/vi.dic  minus  vi-non-syllables.txt
  names lexicon          <- core/src/data/dictionaries/names.dic

Usage: python3 scripts/gen/phonology_tables.py [--check | --list-non-syllables]
Edit the dictionaries or this script, never tables.rs.
"""
import sys, unicodedata as ud, collections as C, pathlib

ROOT = pathlib.Path(__file__).resolve().parents[2]
DIC = ROOT / "core/src/data/dictionaries"
OUT = ROOT / "core/src/phonology/tables.rs"
TONES = {"̀", "́", "̉", "̃", "̣"}
V = set("aăâeêioôơuưy")
INIT = ["ngh", "ng", "nh", "ch", "gh", "gi", "kh", "ph", "qu", "th", "tr"] + list("bcdđghklmnpqrstvx")
CODA = ["ch", "ng", "nh", "c", "m", "n", "p", "t", "k"]
BIT = {"-": "OPEN", "c": "C", "ch": "CH", "m": "M", "n": "N", "ng": "NG", "nh": "NH", "p": "P", "t": "T", "k": "K"}
# Nuclei documented in docs/vietnamese-language-system.md §7.6 that the dictionary may not attest.
DOC_ONLY = {"uêu": "-"}
# Dictionary artefacts that are not Vietnamese nuclei (loan/typo): never become grammar.
JUNK_NUCLEI = {"ou", "ya"}
# (nucleus, coda) pairs the dictionary has only as loans; the docs forbid them (§10.1: -ing takes no tone).
FORBIDDEN = {("i", "ng")}
# Multi-vowel nuclei attested with few codas: typing time also accepts the other plain codas (lenient mode).
WIDE_MAX_CODAS = 4


MD = {"â": ("a", 1), "ê": ("e", 1), "ô": ("o", 1), "ă": ("a", 3), "ơ": ("o", 2), "ư": ("u", 2), "đ": ("d", 4)}


def enc(word):
    """word -> Rust slice of (base letter, modifier code). 0 none, 1 circ, 2 horn, 3 breve, 4 stroke."""
    out = []
    for ch in base_keep(word):
        b, m = MD.get(ch, (ch, 0))
        out.append(f"(b'{b}', {m})")
    return "&[" + ", ".join(out) + "]"


MASK = {"OPEN": 1, "C": 2, "CH": 4, "M": 8, "N": 16, "NG": 32, "NH": 64, "P": 128, "T": 256, "K": 512, "WIDE": 1024}
LETTER = {c: i for i, c in enumerate("aeiouy")}


def nucleus_key(units):
    """[(letter, modifier)] -> u16: base-25 digits (letter*4 + modifier + 1). Mirrors validity.rs."""
    k = 0
    for b, m in units:
        k = k * 25 + LETTER[b] * 4 + m + 1
    return k


def nucleus_index(nuclei):
    """Every way a typed run of vowels can relate to a whitelisted nucleus, as one sorted table:
    key -> (exact codas, pending codas, is a proper prefix). `pending` = the same letters without
    the diacritics still to be typed (ie -> iê), so typing time does one lookup."""
    import itertools
    idx = {}
    for pat, mask in nuclei:
        for l in range(1, len(pat) + 1):
            head = pat[:l]
            modded = [i for i, (_, m) in enumerate(head) if m]
            for drop in itertools.chain.from_iterable(itertools.combinations(modded, r) for r in range(len(modded) + 1)):
                units = [(b, 0 if i in drop else m) for i, (b, m) in enumerate(head)]
                e = idx.setdefault(nucleus_key(units), [0, 0, False])
                if l < len(pat):
                    e[2] = True
                elif drop:
                    e[1] |= mask
                else:
                    e[0] |= mask
    return sorted(idx.items())


def tone_of(word):
    names = {"\u0301": 1, "\u0300": 2, "\u0309": 3, "\u0303": 4, "\u0323": 5}
    for c in ud.normalize("NFD", word):
        if c in names:
            return names[c]
    return 0


def base_keep(word):
    """tone stripped, modifiers kept (NFC)"""
    return ud.normalize("NFC", "".join(c for c in ud.normalize("NFD", word) if c not in TONES))


def base(s):
    return ud.normalize("NFC", "".join(c for c in ud.normalize("NFD", s) if c not in TONES))


def parse(b):
    """base letters (tone stripped) -> (initial, nucleus, coda) or None"""
    ini = next((i for i in INIT if b.startswith(i)), "")
    rest = b[len(ini):]
    if ini == "g" and rest[:1] == "i" and len(rest) > 1 and rest[1] in V:
        ini, rest = "gi", rest[1:]
    if ini == "gi" and not any(c in V for c in rest):
        ini, rest = "g", "i" + rest  # "gì", "gìn", "gíp": the i is both the initial's and the nucleus
    if ini == "qu" and rest == "":
        return None
    coda = next((c for c in CODA if rest.endswith(c) and len(rest) > len(c) and rest[: -len(c)][-1] in V), "")
    nuc = rest[: len(rest) - len(coda)] if coda else rest
    if not nuc or any(c not in V for c in nuc):
        return None
    return ini, nuc, coda


def words(path):
    return [l.strip().lower() for l in path.read_text(encoding="utf8").splitlines()[1:] if l.strip()]


def main():
    vi = words(DIC / "vi.dic")
    if "--list-non-syllables" in sys.argv:
        print("\n".join(w for w in vi if parse(base(w)) is None))
        return
    non = {l.strip() for l in (DIC / "vi-non-syllables.txt").read_text(encoding="utf8").splitlines()
           if l.strip() and not l.startswith("#")}
    names = [l.split("#")[0].strip().lower() for l in (DIC / "names.dic").read_text(encoding="utf8").splitlines()
             if l.split("#")[0].strip()]
    M = C.defaultdict(C.Counter)
    used = 0
    for w in vi:
        if w in non:
            continue
        p = parse(base(w))
        if p is None:
            print(f"unparsed and not listed in vi-non-syllables.txt: {w}", file=sys.stderr)
            continue
        _, nuc, coda = p
        if nuc in JUNK_NUCLEI:
            continue
        if (nuc, coda) in FORBIDDEN:
            continue
        M[nuc][coda or "-"] += 1
        used += 1
    for nuc, c in DOC_ONLY.items():
        M[nuc][c] += 0
    rows = []
    index_src = []
    for nuc in sorted(M, key=lambda n: (len(n), n)):
        cs = sorted(M[nuc], key=lambda c: list(BIT).index(c))
        mask = "|".join(BIT[c] for c in cs)
        if len(nuc) >= 2 and 1 <= len([c for c in cs if c != "-"]) <= WIDE_MAX_CODAS:
            mask += "|WIDE"
        note = " ".join(f"{c}:{M[nuc][c]}" for c in cs)
        rows.append(f'    ({enc(nuc)}, {mask}), // {nuc} {note}')
        index_src.append(([MD.get(ch, (ch, 0)) for ch in base_keep(nuc)], sum(MASK[m] for m in mask.split("|"))))
    out = [
        "// GENERATED by scripts/gen/phonology_tables.py: edit the dictionaries or the script, not this file.",
        f"// Source: vi.dic ({used} syllables used, {len(non)} non-syllables excluded), names.dic ({len(names)} names).",
        "",
        "pub const OPEN: u16 = 1 << 0;",
        "pub const C: u16 = 1 << 1;",
        "pub const CH: u16 = 1 << 2;",
        "pub const M: u16 = 1 << 3;",
        "pub const N: u16 = 1 << 4;",
        "pub const NG: u16 = 1 << 5;",
        "pub const NH: u16 = 1 << 6;",
        "pub const P: u16 = 1 << 7;",
        "pub const T: u16 = 1 << 8;",
        "pub const K: u16 = 1 << 9;",
        "/// Typing-time (lenient) mode also accepts C|M|N|NG|P|T after this nucleus.",
        "pub const WIDE: u16 = 1 << 10;",
        "pub const STOPS: u16 = C | CH | P | T | K;",
        "",
        "/// Vowel nucleus -> allowed codas (OPEN = no consonant coda). Nucleus list = docs §7.6 plus attested forms;",
        "/// a nucleus without OPEN must be followed by a coda (iê, uô, ươ...). Letters are pre-decoded:",
        "/// (base letter, modifier) with modifier 0 none, 1 circumflex, 2 horn, 3 breve, 4 stroke.",
        "pub const NUCLEUS_CODAS: &[(&[(u8, u8)], u16)] = &[",
        *rows,
        "];",
        "",
        "/// `NUCLEUS_CODAS` flattened for typing time: key = base-25 digits of (letter a e i o u y)*4 + modifier + 1;",
        "/// value = (codas of an exact match, codas when only diacritics are missing, is a proper prefix).",
        "pub const NUCLEUS_INDEX: &[(u16, u16, u16, bool)] = &[",
        *[f"    ({k}, {e}, {pe}, {str(pr).lower()})," for k, (e, pe, pr) in nucleus_index(index_src)],
        "];",
        "",
        "/// Proper-name syllables whose spelling breaks the regular rules (data, prefix-closed at match time).",
        "/// (letters as above, tone 0 ngang 1 sắc 2 huyền 3 hỏi 4 ngã 5 nặng)",
        "pub const NAMES: &[(&[(u8, u8)], u8)] = &[",
        *[f'    ({enc(n)}, {tone_of(n)}), // {n}' for n in names],
        "];",
        "",
    ]
    text = "\n".join(out)
    if "--check" in sys.argv:  # CI: tables.rs must be what the dictionaries generate
        if not OUT.exists() or OUT.read_text(encoding="utf8") != text:
            sys.exit(f"{OUT.relative_to(ROOT)} is out of date: run python3 scripts/gen/phonology_tables.py")
        print(f"{OUT.relative_to(ROOT)} is up to date")
        return
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(text, encoding="utf8")
    print(f"wrote {OUT.relative_to(ROOT)}: {len(rows)} nuclei, {len(names)} names, {used} syllables")


main()
