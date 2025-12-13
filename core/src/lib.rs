//! Gõ Nhanh Vietnamese IME Core
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
pub mod utils;

use engine::{Engine, Result};
use std::sync::Mutex;

// Global engine instance (thread-safe via Mutex)
static ENGINE: Mutex<Option<Engine>> = Mutex::new(None);

/// Lock the engine mutex, recovering from poisoned state if needed (for tests)
fn lock_engine() -> std::sync::MutexGuard<'static, Option<Engine>> {
    ENGINE.lock().unwrap_or_else(|e| e.into_inner())
}

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
    let mut guard = lock_engine();
    *guard = Some(Engine::new());
}

/// Process a key event and return the result.
///
/// # Arguments
/// * `key` - macOS virtual keycode (0-127 for standard keys)
/// * `caps` - true if CapsLock is pressed (for uppercase letters)
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
///
/// # Note
/// For VNI mode with Shift+number keys (to type @, #, $ etc.),
/// use `ime_key_ext` with the shift parameter.
#[no_mangle]
pub extern "C" fn ime_key(key: u16, caps: bool, ctrl: bool) -> *mut Result {
    let mut guard = lock_engine();
    if let Some(ref mut e) = *guard {
        let r = e.on_key(key, caps, ctrl);
        Box::into_raw(Box::new(r))
    } else {
        std::ptr::null_mut()
    }
}

/// Process a key event with extended parameters.
///
/// # Arguments
/// * `key` - macOS virtual keycode (0-127 for standard keys)
/// * `caps` - true if CapsLock is pressed (for uppercase letters)
/// * `ctrl` - true if Cmd/Ctrl/Alt is pressed (bypasses IME)
/// * `shift` - true if Shift key is pressed (for symbols like @, #, $)
///
/// # Returns
/// * Pointer to `Result` struct (caller must free with `ime_free`)
/// * `null` if engine not initialized
///
/// # VNI Shift+number behavior
/// In VNI mode, when `shift=true` and key is a number (0-9), the engine
/// will NOT apply VNI marks/tones. This allows typing symbols:
/// - Shift+2 → @ (not huyền mark)
/// - Shift+3 → # (not hỏi mark)
/// - etc.
#[no_mangle]
pub extern "C" fn ime_key_ext(key: u16, caps: bool, ctrl: bool, shift: bool) -> *mut Result {
    let mut guard = lock_engine();
    if let Some(ref mut e) = *guard {
        let r = e.on_key_ext(key, caps, ctrl, shift);
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
    let mut guard = lock_engine();
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
    let mut guard = lock_engine();
    if let Some(ref mut e) = *guard {
        e.set_enabled(enabled);
    }
}

/// Clear the input buffer.
///
/// Call on word boundaries (space, punctuation, mouse click, focus change).
/// No-op if engine not initialized.
#[no_mangle]
pub extern "C" fn ime_clear() {
    let mut guard = lock_engine();
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
// Shortcut FFI
// ============================================================

/// Add a shortcut to the engine.
///
/// # Arguments
/// * `trigger` - C string for trigger (e.g., "vn")
/// * `replacement` - C string for replacement (e.g., "Việt Nam")
///
/// # Safety
/// Both pointers must be valid null-terminated UTF-8 strings.
#[no_mangle]
pub unsafe extern "C" fn ime_add_shortcut(
    trigger: *const std::os::raw::c_char,
    replacement: *const std::os::raw::c_char,
) {
    if trigger.is_null() || replacement.is_null() {
        return;
    }

    let trigger_str = match std::ffi::CStr::from_ptr(trigger).to_str() {
        Ok(s) => s,
        Err(_) => return,
    };
    let replacement_str = match std::ffi::CStr::from_ptr(replacement).to_str() {
        Ok(s) => s,
        Err(_) => return,
    };

    let mut guard = lock_engine();
    if let Some(ref mut e) = *guard {
        e.shortcuts_mut().add(engine::shortcut::Shortcut::new(
            trigger_str,
            replacement_str,
        ));
    }
}

/// Remove a shortcut from the engine.
///
/// # Arguments
/// * `trigger` - C string for trigger to remove
///
/// # Safety
/// Pointer must be a valid null-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn ime_remove_shortcut(trigger: *const std::os::raw::c_char) {
    if trigger.is_null() {
        return;
    }

    let trigger_str = match std::ffi::CStr::from_ptr(trigger).to_str() {
        Ok(s) => s,
        Err(_) => return,
    };

    let mut guard = lock_engine();
    if let Some(ref mut e) = *guard {
        e.shortcuts_mut().remove(trigger_str);
    }
}

/// Clear all shortcuts from the engine.
#[no_mangle]
pub extern "C" fn ime_clear_shortcuts() {
    let mut guard = lock_engine();
    if let Some(ref mut e) = *guard {
        e.shortcuts_mut().clear();
    }
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::keys;
    use serial_test::serial;
    use std::ffi::CString;

    #[test]
    #[serial]
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

    #[test]
    #[serial]
    fn test_shortcut_ffi_add_and_clear() {
        ime_init();
        ime_clear_shortcuts(); // Clear any existing shortcuts
        ime_method(0); // Telex

        // Add a shortcut via FFI
        let trigger = CString::new("vn").unwrap();
        let replacement = CString::new("Việt Nam").unwrap();

        unsafe {
            ime_add_shortcut(trigger.as_ptr(), replacement.as_ptr());
        }

        // Verify shortcut was added by checking engine state
        let guard = lock_engine();
        if let Some(ref e) = *guard {
            assert_eq!(e.shortcuts().len(), 1);
        }
        drop(guard);

        // Clear all shortcuts
        ime_clear_shortcuts();

        // Verify shortcuts cleared
        let guard = lock_engine();
        if let Some(ref e) = *guard {
            assert_eq!(e.shortcuts().len(), 0);
        }
        drop(guard);

        ime_clear();
    }

    #[test]
    #[serial]
    fn test_shortcut_ffi_remove() {
        ime_init();
        ime_clear_shortcuts(); // Clear any existing shortcuts
        ime_method(0); // Telex

        // Add two shortcuts
        let trigger1 = CString::new("hn").unwrap();
        let replacement1 = CString::new("Hà Nội").unwrap();
        let trigger2 = CString::new("hcm").unwrap();
        let replacement2 = CString::new("Hồ Chí Minh").unwrap();

        unsafe {
            ime_add_shortcut(trigger1.as_ptr(), replacement1.as_ptr());
            ime_add_shortcut(trigger2.as_ptr(), replacement2.as_ptr());
        }

        // Verify both added
        let guard = lock_engine();
        if let Some(ref e) = *guard {
            assert_eq!(e.shortcuts().len(), 2);
        }
        drop(guard);

        // Remove one shortcut
        unsafe {
            ime_remove_shortcut(trigger1.as_ptr());
        }

        // Verify only one remains
        let guard = lock_engine();
        if let Some(ref e) = *guard {
            assert_eq!(e.shortcuts().len(), 1);
        }
        drop(guard);

        // Clean up
        ime_clear_shortcuts();
        ime_clear();
    }

    #[test]
    #[serial]
    fn test_shortcut_ffi_null_safety() {
        ime_init();

        // Should not crash with null pointers
        unsafe {
            ime_add_shortcut(std::ptr::null(), std::ptr::null());
            ime_remove_shortcut(std::ptr::null());
        }

        // Engine should still work
        let r = ime_key(keys::A, false, false);
        assert!(!r.is_null());
        unsafe { ime_free(r) };

        ime_clear();
    }

    #[test]
    #[serial]
    fn test_shortcut_ffi_unicode() {
        ime_init();
        ime_clear_shortcuts(); // Clear any existing shortcuts
        ime_method(0);

        // Test with Unicode in both trigger and replacement
        let trigger = CString::new("tphcm").unwrap();
        let replacement = CString::new("Thành phố Hồ Chí Minh").unwrap();

        unsafe {
            ime_add_shortcut(trigger.as_ptr(), replacement.as_ptr());
        }

        // Verify shortcut added with proper UTF-8 handling
        let guard = lock_engine();
        if let Some(ref e) = *guard {
            assert_eq!(e.shortcuts().len(), 1);
        }
        drop(guard);

        ime_clear_shortcuts();
        ime_clear();
    }
}
