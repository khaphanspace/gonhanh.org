//! V3 Engine Adapter
//!
//! Wraps v3::Processor to provide v1-compatible Engine interface.
//! This allows running v1 tests against v3 implementation.

use crate::data::keys;
use crate::engine::shortcut::Shortcut;
use crate::engine::Result;
use crate::v3::processor::{ProcessResult, Processor};

/// Stub shortcuts manager for v3 (shortcuts not yet implemented)
pub struct ShortcutsStub;

impl ShortcutsStub {
    pub fn len(&self) -> usize {
        0
    }
    pub fn is_empty(&self) -> bool {
        true
    }
    pub fn add(&mut self, _shortcut: Shortcut) {}
    pub fn remove(&mut self, _trigger: &str) {}
    pub fn clear(&mut self) {}
    pub fn lookup(&self, _trigger: &str) -> Option<(usize, &Shortcut)> {
        None
    }
}

/// V3 Engine with v1-compatible interface
pub struct Engine {
    processor: Processor,
    enabled: bool,
    method: u8,
    english_auto_restore: bool,
    // Track state for Result generation
    prev_buffer: String,
    // Shortcuts stub (v3 doesn't support shortcuts yet)
    shortcuts_stub: ShortcutsStub,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            processor: Processor::new(),
            enabled: true,
            method: 0,
            english_auto_restore: true,
            prev_buffer: String::new(),
            shortcuts_stub: ShortcutsStub,
        }
    }

    pub fn set_method(&mut self, method: u8) {
        self.method = method;
        self.processor.set_method(method);
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn set_english_auto_restore(&mut self, enabled: bool) {
        self.english_auto_restore = enabled;
    }

    // Stub methods for compatibility
    pub fn set_skip_w_shortcut(&mut self, _skip: bool) {}
    pub fn set_esc_restore(&mut self, _enabled: bool) {}
    pub fn set_free_tone(&mut self, _enabled: bool) {}
    pub fn set_modern_tone(&mut self, _modern: bool) {}
    pub fn set_auto_capitalize(&mut self, _enabled: bool) {}

    pub fn on_key(&mut self, key: u16, caps: bool, ctrl: bool) -> Result {
        self.on_key_ext(key, caps, ctrl, false)
    }

    pub fn on_key_ext(&mut self, key: u16, caps: bool, ctrl: bool, _shift: bool) -> Result {
        if !self.enabled {
            return Result::none();
        }

        // Convert key code to char
        let c = key_to_char(key, caps);

        // Process through v3
        let result = self.processor.process(c, caps, ctrl);

        // Convert ProcessResult to v1 Result
        self.convert_result(result)
    }

    pub fn clear(&mut self) {
        self.processor.clear();
        self.prev_buffer.clear();
    }

    pub fn clear_all(&mut self) {
        self.clear();
    }

    pub fn get_buffer_string(&self) -> String {
        self.processor.buffer_content()
    }

    pub fn finalize_word(&mut self) {
        // V3 handles this internally
    }

    pub fn restore_word(&mut self, _text: &str) {
        // TODO: Implement if needed
    }

    // Shortcut stubs (v3 doesn't support shortcuts yet)
    pub fn shortcuts(&self) -> &ShortcutsStub {
        &self.shortcuts_stub
    }

    pub fn shortcuts_mut(&mut self) -> &mut ShortcutsStub {
        &mut self.shortcuts_stub
    }

    // Debug methods
    pub fn debug_buffer_len(&self) -> usize {
        self.processor.buffer_content().chars().count()
    }

    pub fn debug_raw_input(&self) -> String {
        self.processor.raw_content()
    }

    pub fn debug_buffer_string(&self) -> String {
        self.processor.buffer_content()
    }

    fn convert_result(&mut self, result: ProcessResult) -> Result {
        match result {
            ProcessResult::Pass(_c) => {
                // Pass through - return the char
                Result::none()
            }
            ProcessResult::Transform { backspaces, output } => {
                let chars: Vec<char> = output.chars().collect();
                Result::send(backspaces, &chars)
            }
            ProcessResult::Restore { backspaces, output } => {
                let chars: Vec<char> = output.chars().collect();
                Result::send(backspaces, &chars)
            }
            ProcessResult::Commit => {
                self.prev_buffer.clear();
                Result::none()
            }
            ProcessResult::None => Result::none(),
            ProcessResult::ForeignMode => Result::none(),
        }
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert key code to char
fn key_to_char(key: u16, caps: bool) -> char {
    let base = match key {
        keys::A => 'a',
        keys::B => 'b',
        keys::C => 'c',
        keys::D => 'd',
        keys::E => 'e',
        keys::F => 'f',
        keys::G => 'g',
        keys::H => 'h',
        keys::I => 'i',
        keys::J => 'j',
        keys::K => 'k',
        keys::L => 'l',
        keys::M => 'm',
        keys::N => 'n',
        keys::O => 'o',
        keys::P => 'p',
        keys::Q => 'q',
        keys::R => 'r',
        keys::S => 's',
        keys::T => 't',
        keys::U => 'u',
        keys::V => 'v',
        keys::W => 'w',
        keys::X => 'x',
        keys::Y => 'y',
        keys::Z => 'z',
        keys::SPACE => ' ',
        keys::N0 => '0',
        keys::N1 => '1',
        keys::N2 => '2',
        keys::N3 => '3',
        keys::N4 => '4',
        keys::N5 => '5',
        keys::N6 => '6',
        keys::N7 => '7',
        keys::N8 => '8',
        keys::N9 => '9',
        _ => '?',
    };

    if caps && base.is_ascii_alphabetic() {
        base.to_ascii_uppercase()
    } else {
        base
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adapter_new() {
        let e = Engine::new();
        assert!(e.enabled);
        assert_eq!(e.method, 0);
    }

    #[test]
    fn test_adapter_basic() {
        let mut e = Engine::new();
        e.on_key(keys::A, false, false);
        assert!(!e.get_buffer_string().is_empty());
    }
}
