//! GoNhanh Vietnamese IME Core
//!
//! Simple Vietnamese input method engine supporting Telex and VNI.

pub mod data;
pub mod engine;
pub mod input;

use engine::{Engine, Result};
use std::sync::Mutex;

// Global engine instance
static ENGINE: Mutex<Option<Engine>> = Mutex::new(None);

// ============================================================
// FFI Interface
// ============================================================

/// Initialize engine
#[no_mangle]
pub extern "C" fn ime_init() {
    let mut guard = ENGINE.lock().unwrap();
    *guard = Some(Engine::new());
}

/// Handle key event
/// Returns pointer to Result (must be freed with ime_free)
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

/// Set input method (0=Telex, 1=VNI)
#[no_mangle]
pub extern "C" fn ime_method(method: u8) {
    let mut guard = ENGINE.lock().unwrap();
    if let Some(ref mut e) = *guard {
        e.set_method(method);
    }
}

/// Enable/disable engine
#[no_mangle]
pub extern "C" fn ime_enabled(enabled: bool) {
    let mut guard = ENGINE.lock().unwrap();
    if let Some(ref mut e) = *guard {
        e.set_enabled(enabled);
    }
}

/// Set modern orthography (true=oà, false=òa)
#[no_mangle]
pub extern "C" fn ime_modern(modern: bool) {
    let mut guard = ENGINE.lock().unwrap();
    if let Some(ref mut e) = *guard {
        e.set_modern(modern);
    }
}

/// Clear buffer (new session)
#[no_mangle]
pub extern "C" fn ime_clear() {
    let mut guard = ENGINE.lock().unwrap();
    if let Some(ref mut e) = *guard {
        e.clear();
    }
}

/// Free result
#[no_mangle]
pub extern "C" fn ime_free(r: *mut Result) {
    if !r.is_null() {
        unsafe { drop(Box::from_raw(r)) };
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
        ime_free(r1);

        let r2 = ime_key(keys::S, false, false);
        assert!(!r2.is_null());
        unsafe {
            assert_eq!((*r2).chars[0], 'á' as u32);
        }
        ime_free(r2);

        ime_clear();
    }
}
