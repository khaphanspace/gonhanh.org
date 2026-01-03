//! Step 2: Dispatch & Execute
//!
//! Routes KeyType to appropriate action based on buffer state.
//! Handles double-key reversion (ss→s, ff→f, etc.)

use super::buffer::Buffer;
use super::state::BufferState;
use super::types::{Action, KeyType, MarkType, Tone};

/// Dispatch keystroke to appropriate action
#[inline]
pub fn dispatch(key_type: KeyType, buffer: &Buffer, _state: &BufferState) -> Action {
    match key_type {
        KeyType::Letter(_) => Action::Continue,

        KeyType::Tone(tone_key) => {
            // Check for double-key revert (ss, ff, rr, etc.)
            if is_tone_revert(tone_key, buffer) {
                Action::RevertTone
            } else {
                Action::AddTone(Tone::from(tone_key))
            }
        }

        KeyType::Mark(mark) => {
            // Check for double-key revert (www, etc.)
            if is_mark_revert(mark, buffer) {
                Action::Passthrough
            } else {
                Action::AddMark(mark)
            }
        }

        KeyType::Terminator => Action::CheckRestore,

        KeyType::Special | KeyType::Passthrough => Action::Passthrough,
    }
}

/// Check if tone key should revert (double press)
#[inline]
fn is_tone_revert(tone_key: u8, buffer: &Buffer) -> bool {
    // In Telex: ss→s, ff→f, rr→r, xx→x, jj→j
    // Check if last raw char matches current tone key
    if let Some(last) = buffer.last_raw() {
        return last as u8 == tone_key;
    }
    false
}

/// Check if mark should revert (double press like www)
#[inline]
fn is_mark_revert(mark: MarkType, buffer: &Buffer) -> bool {
    // Only HornOrBreve (w) can be reverted by double press
    if mark != MarkType::HornOrBreve {
        return false;
    }
    // Check if last raw char is 'w'
    if let Some(last) = buffer.last_raw() {
        return last == 'w';
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dispatch_letter() {
        let buffer = Buffer::new();
        let state = BufferState::new();
        let action = dispatch(KeyType::Letter(b'a'), &buffer, &state);
        assert_eq!(action, Action::Continue);
    }

    #[test]
    fn test_dispatch_tone() {
        let buffer = Buffer::new();
        let state = BufferState::new();
        let action = dispatch(KeyType::Tone(b's'), &buffer, &state);
        assert_eq!(action, Action::AddTone(Tone::Sac));
    }

    #[test]
    fn test_dispatch_tone_revert() {
        let mut buffer = Buffer::new();
        buffer.push_raw('s');
        let state = BufferState::new();
        // Second 's' should revert
        let action = dispatch(KeyType::Tone(b's'), &buffer, &state);
        assert_eq!(action, Action::RevertTone);
    }

    #[test]
    fn test_dispatch_mark() {
        let buffer = Buffer::new();
        let state = BufferState::new();
        let action = dispatch(KeyType::Mark(MarkType::Circumflex), &buffer, &state);
        assert_eq!(action, Action::AddMark(MarkType::Circumflex));
    }

    #[test]
    fn test_dispatch_mark_horn_revert() {
        let mut buffer = Buffer::new();
        buffer.push_raw('w');
        let state = BufferState::new();
        // Second 'w' should passthrough (revert)
        let action = dispatch(KeyType::Mark(MarkType::HornOrBreve), &buffer, &state);
        assert_eq!(action, Action::Passthrough);
    }

    #[test]
    fn test_dispatch_terminator() {
        let buffer = Buffer::new();
        let state = BufferState::new();
        let action = dispatch(KeyType::Terminator, &buffer, &state);
        assert_eq!(action, Action::CheckRestore);
    }

    #[test]
    fn test_dispatch_passthrough() {
        let buffer = Buffer::new();
        let state = BufferState::new();
        let action = dispatch(KeyType::Passthrough, &buffer, &state);
        assert_eq!(action, Action::Passthrough);
    }

    #[test]
    fn test_tone_from_telex() {
        assert_eq!(Tone::from(b's'), Tone::Sac);
        assert_eq!(Tone::from(b'f'), Tone::Huyen);
        assert_eq!(Tone::from(b'r'), Tone::Hoi);
        assert_eq!(Tone::from(b'x'), Tone::Nga);
        assert_eq!(Tone::from(b'j'), Tone::Nang);
    }

    #[test]
    fn test_tone_from_vni() {
        assert_eq!(Tone::from(1u8), Tone::Sac);
        assert_eq!(Tone::from(2u8), Tone::Huyen);
        assert_eq!(Tone::from(3u8), Tone::Hoi);
        assert_eq!(Tone::from(4u8), Tone::Nga);
        assert_eq!(Tone::from(5u8), Tone::Nang);
        assert_eq!(Tone::from(0u8), Tone::None);
    }
}
