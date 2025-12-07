//! GoNhanh Vietnamese IME Core
//!
//! Simple Vietnamese input method engine supporting Telex and VNI.
//!
//! # FFI Usage
//!
//! ```c
//! // Initialize once at app start
//! ime_init();
//! ime_method(0);  // 0=Telex, 1=VNI
//!
//! // Process each keystroke
//! ImeResult* r = ime_key(keycode, is_shift, is_ctrl);
//! if (r && r->action == 1) {
//!     // Send r->backspace deletes, then r->chars
//! }
//! ime_free(r);
//!
//! // Clean up on word boundary
//! ime_clear();
//! ```

pub mod data;
pub mod engine;
pub mod input;
pub mod updater;

use engine::{Engine, Result};
use std::sync::Mutex;

// Global engine instance (thread-safe via Mutex)
static ENGINE: Mutex<Option<Engine>> = Mutex::new(None);

// ============================================================
// FFI Interface
// ============================================================

/// Initialize the IME engine.
///
/// Must be called exactly once before any other `ime_*` functions.
/// Thread-safe: uses internal mutex.
///
/// # Panics
/// Panics if mutex is poisoned (only if previous call panicked).
#[no_mangle]
pub extern "C" fn ime_init() {
    let mut guard = ENGINE.lock().unwrap();
    *guard = Some(Engine::new());
}

/// Process a key event and return the result.
///
/// # Arguments
/// * `key` - macOS virtual keycode (0-127 for standard keys)
/// * `caps` - true if Shift or CapsLock is pressed
/// * `ctrl` - true if Cmd/Ctrl/Alt is pressed (bypasses IME)
///
/// # Returns
/// * Pointer to `Result` struct (caller must free with `ime_free`)
/// * `null` if engine not initialized
///
/// # Result struct
/// * `action`: 0=None (pass through), 1=Send (replace text), 2=Restore
/// * `backspace`: number of characters to delete
/// * `chars`: UTF-32 codepoints to insert
/// * `count`: number of valid chars
#[no_mangle]
pub extern "C" fn ime_key(key: u16, caps: bool, ctrl: bool) -> *mut Result {
    let mut guard = ENGINE.lock().unwrap();
    if let Some(ref mut e) = *guard {
        let r = e.on_key(key, caps, ctrl);
        Box::into_raw(Box::new(r))
    } else {
        std::ptr::null_mut()
    }
}

/// Set the input method.
///
/// # Arguments
/// * `method` - 0 for Telex, 1 for VNI
///
/// No-op if engine not initialized.
#[no_mangle]
pub extern "C" fn ime_method(method: u8) {
    let mut guard = ENGINE.lock().unwrap();
    if let Some(ref mut e) = *guard {
        e.set_method(method);
    }
}

/// Enable or disable the engine.
///
/// When disabled, `ime_key` returns action=0 (pass through).
/// No-op if engine not initialized.
#[no_mangle]
pub extern "C" fn ime_enabled(enabled: bool) {
    let mut guard = ENGINE.lock().unwrap();
    if let Some(ref mut e) = *guard {
        e.set_enabled(enabled);
    }
}

/// Set tone placement style.
///
/// # Arguments
/// * `modern` - true for modern style (hoà), false for classic (hòa)
///
/// No-op if engine not initialized.
#[no_mangle]
pub extern "C" fn ime_modern(modern: bool) {
    let mut guard = ENGINE.lock().unwrap();
    if let Some(ref mut e) = *guard {
        e.set_modern(modern);
    }
}

/// Clear the input buffer.
///
/// Call on word boundaries (space, punctuation, mouse click, focus change).
/// No-op if engine not initialized.
#[no_mangle]
pub extern "C" fn ime_clear() {
    let mut guard = ENGINE.lock().unwrap();
    if let Some(ref mut e) = *guard {
        e.clear();
    }
}

/// Free a result pointer returned by `ime_key`.
///
/// # Safety
/// * `r` must be a pointer returned by `ime_key`, or null
/// * Must be called exactly once per non-null `ime_key` return
/// * Do not use `r` after calling this function
#[no_mangle]
pub unsafe extern "C" fn ime_free(r: *mut Result) {
    if !r.is_null() {
        drop(Box::from_raw(r));
    }
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::keys;

    #[test]
    fn test_ffi_flow() {
        ime_init();
        ime_method(0); // Telex

        // Type 'a' + 's' -> á
        let r1 = ime_key(keys::A, false, false);
        assert!(!r1.is_null());
        unsafe { ime_free(r1) };

        let r2 = ime_key(keys::S, false, false);
        assert!(!r2.is_null());
        unsafe {
            assert_eq!((*r2).chars[0], 'á' as u32);
            ime_free(r2);
        }

        ime_clear();
    }
}
