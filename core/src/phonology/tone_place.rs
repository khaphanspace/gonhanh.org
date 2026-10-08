//! Where does the tone mark go? (Constitution I6, docs §7.2–7.6.)
//!
//! A pure function of the finished syllable, never of typing order (I7): the engine calls it
//! on the final letters, so `hoaf`, `hofa` and `hoaf` + more keys all agree.

use super::letters::{Mod, Unit};
use super::validity::split;

/// Index (into `units`) of the vowel that carries the tone, or `None` without a vowel.
/// `modern`: oà/uý (tone on the 2nd vowel of oa/oe/uy); old style: òa/úy (1st vowel).
pub fn tone_index(units: &[Unit], modern: bool) -> Option<usize> {
    let (_, start, end) = split(units);
    let mut nuc = &units[start..end];
    // a vowel typed twice and kept literally (aa → áaa, tòaa) does not move the tone
    while nuc.len() >= 3 && nuc[nuc.len() - 1] == nuc[nuc.len() - 2] {
        nuc = &nuc[..nuc.len() - 1];
    }
    match nuc.len() {
        0 => return None,
        1 => return Some(start),
        _ => {}
    }
    // ă â ê ô ơ ư win; with two modified vowels (ươ, ươi) the last one (ơ).
    if let Some(k) = nuc.iter().rposition(|u| u.md != Mod::None) {
        return Some(start + k);
    }
    let has_coda = end < units.len();
    Some(
        start
            + match nuc.len() {
                2 => {
                    let pair = (nuc[0].ch, nuc[1].ch);
                    let style_pair = matches!(pair, (b'o', b'a') | (b'o', b'e') | (b'u', b'y'));
                    // uo ue ie ye ea: the diacritic still to come (uô, uê, iê, yê) will carry the
                    // tone, so it already sits on the second vowel
                    let pending = matches!(
                        pair,
                        (b'u', b'o') | (b'u', b'e') | (b'i', b'e') | (b'y', b'e') | (b'e', b'a')
                    );
                    if has_coda || pending || (style_pair && modern) {
                        1
                    } else {
                        0
                    }
                }
                // oai oay oeo oao uya uyu: the middle vowel
                _ => 1,
            },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn idx(s: &str, modern: bool) -> usize {
        let units: Vec<Unit> = s.chars().map(|c| Unit::from_char(c).unwrap().0).collect();
        tone_index(&units, modern).unwrap()
    }

    #[test]
    fn placement_rules() {
        // single vowel, modified vowel priority, ươ → ơ
        assert_eq!(idx("ba", true), 1);
        assert_eq!(idx("sưa", true), 1); // ưa → ư
        assert_eq!(idx("đươc", true), 2); // ươ → ơ (index 2)
        assert_eq!(idx("tiêng", true), 2); // ê
        assert_eq!(idx("khuyên", true), 4); // uyê → ê
                                            // qu / gi swallow the u / i
        assert_eq!(idx("qua", true), 2);
        assert_eq!(idx("gia", true), 2);
        // open diphthongs: first vowel (mùa, mía, hài), closed: second (hoàn)
        assert_eq!(idx("mua", true), 1);
        assert_eq!(idx("mia", true), 1);
        assert_eq!(idx("hai", true), 1);
        assert_eq!(idx("hoan", true), 2);
        // style-dependent: modern oà/uý, old òa/úy
        assert_eq!(idx("hoa", true), 2);
        assert_eq!(idx("hoa", false), 1);
        assert_eq!(idx("thuy", true), 3);
        assert_eq!(idx("thuy", false), 2);
        // triphthongs: middle
        assert_eq!(idx("hoai", true), 2);
        assert_eq!(idx("khuya", true), 3);
    }
}
