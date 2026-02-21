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

/// Process a key event with the actual Unicode character.
///
/// Used for Option-modified keys on macOS where the keycode doesn't change
/// but the actual character is different (e.g., Option+V produces √).
///
/// # Arguments
/// * `key` - macOS virtual keycode (0-127 for standard keys)
/// * `caps` - true if CapsLock is pressed (for uppercase letters)
/// * `ctrl` - true if Cmd/Ctrl is pressed (bypasses IME)
/// * `shift` - true if Shift key is pressed
/// * `char_code` - The actual Unicode character code (UTF-32). If > 0, uses this
///   for shortcut matching instead of deriving from keycode.
///
/// # Returns
/// * Pointer to `Result` struct (caller must free with `ime_free`)
/// * `null` if engine not initialized
///
/// # Example
/// When Option+V is pressed on macOS:
/// - keycode is still V (9)
/// - char_code is √ (0x221A)
/// - Engine uses √ for shortcut matching, allowing shortcuts like √√ → ✅
#[no_mangle]
pub extern "C" fn ime_key_with_char(
    key: u16,
    caps: bool,
    ctrl: bool,
    shift: bool,
    char_code: u32,
) -> *mut Result {
    let mut guard = lock_engine();
    if let Some(ref mut e) = *guard {
        let ch = if char_code > 0 {
            char::from_u32(char_code)
        } else {
            None
        };
        let r = e.on_key_with_char(key, caps, ctrl, shift, ch);
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

/// Set whether to skip w→ư shortcut in Telex mode.
///
/// When `skip` is true, typing 'w' stays as 'w' instead of
/// converting to 'ư'. Horn modifier still works: "ow" → "ơ", "uw" → "ư".
/// No-op if engine not initialized.
#[no_mangle]
pub extern "C" fn ime_skip_w_shortcut(skip: bool) {
    let mut guard = lock_engine();
    if let Some(ref mut e) = *guard {
        e.set_skip_w_shortcut(skip);
    }
}

/// Set whether bracket shortcuts are enabled: ] → ư, [ → ơ (Issue #159)
///
/// When `enabled` is true (default), ] types ư and [ types ơ in Telex mode.
/// No-op if engine not initialized.
#[no_mangle]
pub extern "C" fn ime_bracket_shortcut(enabled: bool) {
    let mut guard = lock_engine();
    if let Some(ref mut e) = *guard {
        e.set_bracket_shortcut(enabled);
    }
}

/// Set whether ESC key restores raw ASCII input.
///
/// When `enabled` is true (default), pressing ESC restores original keystrokes.
/// When `enabled` is false, ESC key is passed through without restoration.
/// No-op if engine not initialized.
#[no_mangle]
pub extern "C" fn ime_esc_restore(enabled: bool) {
    let mut guard = lock_engine();
    if let Some(ref mut e) = *guard {
        e.set_esc_restore(enabled);
    }
}

/// Set whether to enable free tone placement (skip validation).
///
/// When `enabled` is true, allows placing diacritics anywhere without
/// spelling validation (e.g., "Zìa" is allowed).
/// When `enabled` is false (default), validates Vietnamese spelling rules.
/// No-op if engine not initialized.
#[no_mangle]
pub extern "C" fn ime_free_tone(enabled: bool) {
    let mut guard = lock_engine();
    if let Some(ref mut e) = *guard {
        e.set_free_tone(enabled);
    }
}

/// Set whether to use modern orthography for tone placement.
///
/// When `modern` is true: hoà, thuý (tone on second vowel - new style)
/// When `modern` is false (default): hòa, thúy (tone on first vowel - traditional)
/// No-op if engine not initialized.
#[no_mangle]
pub extern "C" fn ime_modern(modern: bool) {
    let mut guard = lock_engine();
    if let Some(ref mut e) = *guard {
        e.set_modern_tone(modern);
    }
}

/// Enable/disable English auto-restore (experimental feature).
///
/// When `enabled` is true, automatically restores English words that were
/// accidentally transformed (e.g., "tẽt" → "text", "ễpct" → "expect").
/// When `enabled` is false (default), no auto-restore happens.
/// No-op if engine not initialized.
#[no_mangle]
pub extern "C" fn ime_english_auto_restore(enabled: bool) {
    let mut guard = lock_engine();
    if let Some(ref mut e) = *guard {
        e.set_english_auto_restore(enabled);
    }
}

/// Enable/disable auto-capitalize after sentence-ending punctuation.
///
/// When `enabled` is true, automatically capitalizes the first letter
/// after sentence-ending punctuation (. ! ? Enter).
/// When `enabled` is false (default), no auto-capitalize happens.
/// No-op if engine not initialized.
#[no_mangle]
pub extern "C" fn ime_auto_capitalize(enabled: bool) {
    let mut guard = lock_engine();
    if let Some(ref mut e) = *guard {
        e.set_auto_capitalize(enabled);
    }
}

/// Enable/disable foreign consonants (z, w, j, f) as valid initial consonants.
///
/// When `enabled` is true, allows z, w, j, f as valid Vietnamese consonants
/// for typing loanwords while still getting Vietnamese diacritics.
/// When `enabled` is false (default), these letters are treated as invalid initials.
/// No-op if engine not initialized.
#[no_mangle]
pub extern "C" fn ime_allow_foreign_consonants(enabled: bool) {
    let mut guard = lock_engine();
    if let Some(ref mut e) = *guard {
        e.set_allow_foreign_consonants(enabled);
    }
}

/// Clear the input buffer.
///
/// Call on word boundaries (space, punctuation).
/// Preserves word history for backspace-after-space feature.
/// No-op if engine not initialized.
#[no_mangle]
pub extern "C" fn ime_clear() {
    let mut guard = lock_engine();
    if let Some(ref mut e) = *guard {
        e.clear();
    }
}

/// Clear everything including word history.
///
/// Call when cursor position changes (mouse click, arrow keys, focus change).
/// This prevents accidental restore from stale history.
/// No-op if engine not initialized.
#[no_mangle]
pub extern "C" fn ime_clear_all() {
    let mut guard = lock_engine();
    if let Some(ref mut e) = *guard {
        e.clear_all();
    }
}

/// Get the full composed buffer as UTF-32 codepoints.
///
/// Used for "Select All + Replace" injection method where the entire
/// buffer content is needed instead of incremental backspace + chars.
///
/// # Arguments
/// * `out` - Pointer to output buffer for UTF-32 codepoints
/// * `max_len` - Maximum number of codepoints to write
///
/// # Returns
/// Number of codepoints written to `out`.
///
/// # Safety
/// `out` must point to valid memory of at least `max_len * sizeof(u32)` bytes.
#[no_mangle]
pub unsafe extern "C" fn ime_get_buffer(out: *mut u32, max_len: i64) -> i64 {
    if out.is_null() || max_len <= 0 {
        return 0;
    }

    let guard = lock_engine();
    if let Some(ref e) = *guard {
        let full = e.get_buffer_string();
        let utf32: Vec<u32> = full.chars().map(|c| c as u32).collect();
        let len = utf32.len().min(max_len as usize);
        std::ptr::copy_nonoverlapping(utf32.as_ptr(), out, len);
        len as i64
    } else {
        0
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
        // Auto-detect shortcut type:
        // - If trigger contains only non-letter chars (like "->", "=>"), use immediate trigger
        // - Otherwise use word boundary trigger (traditional abbreviations like "vn" → "Việt Nam")
        let is_symbol_trigger = trigger_str.chars().all(|c| !c.is_alphabetic());
        let shortcut = if is_symbol_trigger {
            engine::shortcut::Shortcut::immediate(trigger_str, replacement_str)
        } else {
            engine::shortcut::Shortcut::new(trigger_str, replacement_str)
        };
        e.shortcuts_mut().add(shortcut);
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
// Word Restore FFI
// ============================================================

/// Restore buffer from a Vietnamese word string.
///
/// Used when native app detects cursor at word boundary and user
/// wants to continue editing (e.g., backspace into previous word).
/// Parses Vietnamese characters back to buffer components.
///
/// # Arguments
/// * `word` - C string containing the Vietnamese word to restore
///
/// # Safety
/// Pointer must be a valid null-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn ime_restore_word(word: *const std::os::raw::c_char) {
    if word.is_null() {
        return;
    }
    let word_str = match std::ffi::CStr::from_ptr(word).to_str() {
        Ok(s) => s,
        Err(_) => return,
    };
    let mut guard = lock_engine();
    if let Some(ref mut e) = *guard {
        e.restore_word(word_str);
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

    #[test]
    #[serial]
    fn test_shortcut_ffi_symbol_trigger_immediate() {
        // Test that symbol-only triggers (like "->") are created as immediate shortcuts
        ime_init();
        ime_clear_shortcuts();
        ime_method(0); // Telex

        // Add arrow shortcut via FFI - should auto-detect as immediate
        let trigger = CString::new("->").unwrap();
        let replacement = CString::new("→").unwrap();

        unsafe {
            ime_add_shortcut(trigger.as_ptr(), replacement.as_ptr());
        }

        // Verify shortcut was added with immediate trigger
        let guard = lock_engine();
        if let Some(ref e) = *guard {
            assert_eq!(e.shortcuts().len(), 1);
            let shortcut = e.shortcuts().lookup("->").unwrap().1;
            assert_eq!(
                shortcut.condition,
                engine::shortcut::TriggerCondition::Immediate,
                "Symbol-only trigger should be immediate"
            );
        }
        drop(guard);

        ime_clear_shortcuts();
        ime_clear();
    }

    #[test]
    #[serial]
    fn test_shortcut_ffi_letter_trigger_word_boundary() {
        // Test that letter triggers (like "vn") are created as word boundary shortcuts
        ime_init();
        ime_clear_shortcuts();
        ime_method(0); // Telex

        // Add abbreviation shortcut via FFI - should be word boundary
        let trigger = CString::new("vn").unwrap();
        let replacement = CString::new("Việt Nam").unwrap();

        unsafe {
            ime_add_shortcut(trigger.as_ptr(), replacement.as_ptr());
        }

        // Verify shortcut was added with word boundary trigger
        let guard = lock_engine();
        if let Some(ref e) = *guard {
            assert_eq!(e.shortcuts().len(), 1);
            let shortcut = e.shortcuts().lookup("vn").unwrap().1;
            assert_eq!(
                shortcut.condition,
                engine::shortcut::TriggerCondition::OnWordBoundary,
                "Letter trigger should be word boundary"
            );
        }
        drop(guard);

        ime_clear_shortcuts();
        ime_clear();
    }

    /// Issue #161: Test that shortcuts containing numbers work correctly via FFI
    #[test]
    #[serial]
    fn test_shortcut_ffi_with_numbers() {
        ime_init();
        ime_clear_shortcuts();
        ime_method(0); // Telex

        // Add shortcut with number via FFI
        let trigger = CString::new("f1").unwrap();
        let replacement = CString::new("formula one").unwrap();

        unsafe {
            ime_add_shortcut(trigger.as_ptr(), replacement.as_ptr());
        }

        // Verify shortcut was added
        let guard = lock_engine();
        if let Some(ref e) = *guard {
            assert_eq!(e.shortcuts().len(), 1);
            let shortcut = e.shortcuts().lookup("f1").unwrap().1;
            assert_eq!(
                shortcut.condition,
                engine::shortcut::TriggerCondition::OnWordBoundary,
                "Mixed letter+number trigger should be word boundary"
            );
        }
        drop(guard);

        // Type "f1" + space and verify shortcut triggers
        let _ = ime_key(keys::F, false, false);
        let _ = ime_key(keys::N1, false, false);
        let r = ime_key(keys::SPACE, false, false);

        assert!(!r.is_null());
        let result = unsafe { &*r };
        assert_eq!(
            result.action,
            engine::Action::Send as u8,
            "Shortcut should trigger"
        );
        assert_eq!(result.backspace, 2, "Should backspace 2 chars (f1)");

        // Verify output
        let output: String = (0..result.count as usize)
            .filter_map(|i| char::from_u32(result.chars[i]))
            .collect();
        assert_eq!(output, "formula one ", "Should output replacement + space");

        unsafe { ime_free(r) };
        ime_clear_shortcuts();
        ime_clear();
    }

    #[test]
    #[serial]
    fn test_restore_word_ffi() {
        ime_init();
        ime_method(0); // Telex

        // Restore a Vietnamese word
        let word = CString::new("việt").unwrap();
        unsafe {
            ime_restore_word(word.as_ptr());
        }

        // Type 's' to add sắc mark - should change ệ to ế
        // Engine returns replacement for changed portion
        let r = ime_key(keys::S, false, false);
        assert!(!r.is_null());
        unsafe {
            assert_eq!((*r).action, 1, "Should send replacement");
            // Engine outputs the modified result
            assert!((*r).count > 0, "Should have output chars");
            ime_free(r);
        }

        ime_clear();
    }

    #[test]
    #[serial]
    fn test_restore_word_ffi_null_safety() {
        ime_init();

        // Should not crash with null pointer
        unsafe {
            ime_restore_word(std::ptr::null());
        }

        // Engine should still work
        let r = ime_key(keys::A, false, false);
        assert!(!r.is_null());
        unsafe { ime_free(r) };

        ime_clear();
    }

    /// Issue #275: Test that special character shortcuts work via ime_key_with_char
    /// Example: √√ → ✅ (Option+V twice produces checkmark)
    #[test]
    #[serial]
    fn test_special_char_shortcut_ffi() {
        ime_init();
        ime_clear_shortcuts();
        ime_method(0); // Telex

        // Add special character shortcut: √√ → ✅
        let trigger = CString::new("√√").unwrap();
        let replacement = CString::new("✅").unwrap();

        unsafe {
            ime_add_shortcut(trigger.as_ptr(), replacement.as_ptr());
        }

        // Verify shortcut was added as immediate (symbol-only trigger)
        let guard = lock_engine();
        if let Some(ref e) = *guard {
            assert_eq!(e.shortcuts().len(), 1);
            let shortcut = e.shortcuts().lookup("√√").unwrap().1;
            assert_eq!(
                shortcut.condition,
                engine::shortcut::TriggerCondition::Immediate,
                "Symbol-only trigger should be immediate"
            );
        }
        drop(guard);

        // Simulate typing √ (Option+V) twice using ime_key_with_char
        // First √ - should accumulate in shortcut_prefix
        let r1 = ime_key_with_char(keys::V, false, false, false, '√' as u32);
        assert!(!r1.is_null());
        let result1 = unsafe { &*r1 };
        assert_eq!(result1.action, 0, "First √ should not trigger yet");
        unsafe { ime_free(r1) };

        // Second √ - should match and trigger shortcut
        let r2 = ime_key_with_char(keys::V, false, false, false, '√' as u32);
        assert!(!r2.is_null());
        let result2 = unsafe { &*r2 };
        assert_eq!(
            result2.action,
            engine::Action::Send as u8,
            "Second √ should trigger shortcut"
        );
        assert_eq!(result2.backspace, 1, "Should backspace 1 char (first √)");

        // Verify output is ✅
        let output: String = (0..result2.count as usize)
            .filter_map(|i| char::from_u32(result2.chars[i]))
            .collect();
        assert_eq!(output, "✅", "Should output ✅");

        // Verify key_consumed flag is set
        assert!(
            (result2.flags & engine::FLAG_KEY_CONSUMED) != 0,
            "Key should be consumed"
        );

        unsafe { ime_free(r2) };
        ime_clear_shortcuts();
        ime_clear();
    }

    /// Issue #275: Test suffix matching - typing ≈ç√√ should still match √√
    /// When other special chars precede the shortcut, it should still trigger
    #[test]
    #[serial]
    fn test_special_char_shortcut_with_prefix() {
        ime_init();
        ime_clear_shortcuts();
        ime_method(0); // Telex

        // Add special character shortcut: √√ → ✅
        let trigger = CString::new("√√").unwrap();
        let replacement = CString::new("✅").unwrap();

        unsafe {
            ime_add_shortcut(trigger.as_ptr(), replacement.as_ptr());
        }

        // Type ≈ (Option+X) - should pass through
        let r1 = ime_key_with_char(keys::X, false, false, false, '≈' as u32);
        assert!(!r1.is_null());
        let result1 = unsafe { &*r1 };
        assert_eq!(result1.action, 0, "≈ should not trigger anything");
        unsafe { ime_free(r1) };

        // Type ç (Option+C) - should pass through
        let r2 = ime_key_with_char(keys::C, false, false, false, 'ç' as u32);
        assert!(!r2.is_null());
        let result2 = unsafe { &*r2 };
        assert_eq!(result2.action, 0, "ç should not trigger anything");
        unsafe { ime_free(r2) };

        // Type √ (Option+V) - first √, should accumulate
        let r3 = ime_key_with_char(keys::V, false, false, false, '√' as u32);
        assert!(!r3.is_null());
        let result3 = unsafe { &*r3 };
        assert_eq!(result3.action, 0, "First √ should not trigger yet");
        unsafe { ime_free(r3) };

        // Type √ (Option+V) - second √, should trigger √√ → ✅
        let r4 = ime_key_with_char(keys::V, false, false, false, '√' as u32);
        assert!(!r4.is_null());
        let result4 = unsafe { &*r4 };
        assert_eq!(
            result4.action,
            engine::Action::Send as u8,
            "Second √ should trigger shortcut (suffix match)"
        );
        assert_eq!(result4.backspace, 1, "Should backspace 1 char (first √)");

        // Verify output is ✅
        let output: String = (0..result4.count as usize)
            .filter_map(|i| char::from_u32(result4.chars[i]))
            .collect();
        assert_eq!(output, "✅", "Should output ✅");

        unsafe { ime_free(r4) };
        ime_clear_shortcuts();
        ime_clear();
    }
}
