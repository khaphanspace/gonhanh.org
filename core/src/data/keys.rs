//! macOS virtual keycodes

// Letters
pub const A: u16 = 0;
pub const S: u16 = 1;
pub const D: u16 = 2;
pub const F: u16 = 3;
pub const H: u16 = 4;
pub const G: u16 = 5;
pub const Z: u16 = 6;
pub const X: u16 = 7;
pub const C: u16 = 8;
pub const V: u16 = 9;
pub const B: u16 = 11;
pub const Q: u16 = 12;
pub const W: u16 = 13;
pub const E: u16 = 14;
pub const R: u16 = 15;
pub const Y: u16 = 16;
pub const T: u16 = 17;
pub const O: u16 = 31;
pub const U: u16 = 32;
pub const I: u16 = 34;
pub const P: u16 = 35;
pub const L: u16 = 37;
pub const J: u16 = 38;
pub const K: u16 = 40;
pub const N: u16 = 45;
pub const M: u16 = 46;

// Numbers
pub const N1: u16 = 18;
pub const N2: u16 = 19;
pub const N3: u16 = 20;
pub const N4: u16 = 21;
pub const N5: u16 = 23;
pub const N6: u16 = 22;
pub const N7: u16 = 26;
pub const N8: u16 = 28;
pub const N9: u16 = 25;
pub const N0: u16 = 29;

// Special
pub const SPACE: u16 = 49;
pub const DELETE: u16 = 51;
pub const TAB: u16 = 48;
pub const RETURN: u16 = 36;
pub const ENTER: u16 = 76;
pub const ESC: u16 = 53;
pub const LEFT: u16 = 123;
pub const RIGHT: u16 = 124;
pub const DOWN: u16 = 125;
pub const UP: u16 = 126;

// Punctuation
pub const DOT: u16 = 47;
pub const COMMA: u16 = 43;
pub const SLASH: u16 = 44;
pub const SEMICOLON: u16 = 41;
pub const QUOTE: u16 = 39;
pub const LBRACKET: u16 = 33;
pub const RBRACKET: u16 = 30;
pub const BACKSLASH: u16 = 42;
pub const MINUS: u16 = 27;
pub const EQUAL: u16 = 24;
pub const BACKQUOTE: u16 = 50;

/// Check if key breaks word (space, punctuation, arrows, etc.)
pub fn is_break(key: u16) -> bool {
    matches!(
        key,
        SPACE | TAB | RETURN | ENTER | ESC | LEFT | RIGHT | UP | DOWN |
        DOT | COMMA | SLASH | SEMICOLON | QUOTE | LBRACKET | RBRACKET |
        BACKSLASH | MINUS | EQUAL | BACKQUOTE
    )
}

/// Check if key is a vowel (a, e, i, o, u, y)
pub fn is_vowel(key: u16) -> bool {
    matches!(key, A | E | I | O | U | Y)
}

/// Check if key is a letter
pub fn is_letter(key: u16) -> bool {
    matches!(
        key,
        A | B | C | D | E | F | G | H | I | J | K | L | M |
        N | O | P | Q | R | S | T | U | V | W | X | Y | Z
    )
}
