//! Small pure helpers for auto-capitalize and break-key characters.

use crate::data::keys;

/// `. ! ?` (not Enter): only these start a new sentence once a space follows.
pub fn is_sentence_ending(key: u16, shift: bool) -> bool {
    key == keys::DOT || (shift && key == keys::N1) || (shift && key == keys::SLASH)
}

/// Quotes, brackets, arrows, tab, esc keep a pending capital; other break keys cancel it.
pub fn should_reset_pending(key: u16, shift: bool) -> bool {
    let neutral = key == keys::QUOTE
        || key == keys::LBRACKET
        || key == keys::RBRACKET
        || (shift && key == keys::N9)
        || (shift && key == keys::N0)
        || key == keys::LEFT
        || key == keys::RIGHT
        || key == keys::UP
        || key == keys::DOWN
        || key == keys::TAB
        || key == keys::ESC;
    !neutral
}

/// The character a break key types (for shortcut matching), shifted or not.
pub fn break_char(key: u16, shift: bool) -> Option<char> {
    if shift {
        match key {
            keys::N1 => Some('!'),
            keys::N2 => Some('@'),
            keys::N3 => Some('#'),
            keys::N4 => Some('$'),
            keys::N5 => Some('%'),
            keys::N6 => Some('^'),
            keys::N7 => Some('&'),
            keys::N8 => Some('*'),
            keys::N9 => Some('('),
            keys::N0 => Some(')'),
            keys::MINUS => Some('_'),
            keys::EQUAL => Some('+'),
            keys::SEMICOLON => Some(':'),
            keys::QUOTE => Some('"'),
            keys::COMMA => Some('<'),
            keys::DOT => Some('>'),
            keys::SLASH => Some('?'),
            keys::BACKSLASH => Some('|'),
            keys::LBRACKET => Some('{'),
            keys::RBRACKET => Some('}'),
            keys::BACKQUOTE => Some('~'),
            _ => None,
        }
    } else {
        match key {
            keys::MINUS => Some('-'),
            keys::EQUAL => Some('='),
            keys::SEMICOLON => Some(';'),
            keys::QUOTE => Some('\''),
            keys::COMMA => Some(','),
            keys::DOT => Some('.'),
            keys::SLASH => Some('/'),
            keys::BACKSLASH => Some('\\'),
            keys::LBRACKET => Some('['),
            keys::RBRACKET => Some(']'),
            keys::BACKQUOTE => Some('`'),
            _ => None,
        }
    }
}
