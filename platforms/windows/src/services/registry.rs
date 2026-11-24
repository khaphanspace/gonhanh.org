//! HKEY_CURRENT_USER helpers

use std::ptr::null_mut;

use windows_sys::Win32::System::Registry::*;

use crate::win::{from_wide, wide};

pub fn get_dword(path: &str, name: &str) -> Option<u32> {
    let (p, n) = (wide(path), wide(name));
    let (mut v, mut size) = (0u32, 4u32);
    let r = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            p.as_ptr(),
            n.as_ptr(),
            RRF_RT_REG_DWORD,
            null_mut(),
            &mut v as *mut _ as _,
            &mut size,
        )
    };
    (r == 0).then_some(v)
}

pub fn get_string(path: &str, name: &str) -> Option<String> {
    let (p, n) = (wide(path), wide(name));
    let mut size = 0u32;
    unsafe {
        if RegGetValueW(
            HKEY_CURRENT_USER,
            p.as_ptr(),
            n.as_ptr(),
            RRF_RT_REG_SZ,
            null_mut(),
            null_mut(),
            &mut size,
        ) != 0
        {
            return None;
        }
        let mut buf = vec![0u16; size as usize / 2 + 1];
        if RegGetValueW(
            HKEY_CURRENT_USER,
            p.as_ptr(),
            n.as_ptr(),
            RRF_RT_REG_SZ,
            null_mut(),
            buf.as_mut_ptr() as _,
            &mut size,
        ) != 0
        {
            return None;
        }
        Some(from_wide(&buf))
    }
}

/// A REG_BINARY value of at most `max` bytes
pub fn get_binary(path: &str, name: &str, max: usize) -> Option<Vec<u8>> {
    let (p, n) = (wide(path), wide(name));
    let mut buf = vec![0u8; max];
    let mut size = max as u32;
    let r = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            p.as_ptr(),
            n.as_ptr(),
            RRF_RT_REG_BINARY,
            null_mut(),
            buf.as_mut_ptr() as _,
            &mut size,
        )
    };
    (r == 0).then(|| {
        buf.truncate(size as usize);
        buf
    })
}

/// All DWORD values under a key (name, value)
pub fn dword_values(path: &str) -> Vec<(String, u32)> {
    let p = wide(path);
    let mut key: HKEY = null_mut();
    let mut out = Vec::new();
    unsafe {
        if RegOpenKeyExW(HKEY_CURRENT_USER, p.as_ptr(), 0, KEY_READ, &mut key) != 0 {
            return out;
        }
        for i in 0.. {
            let mut name = [0u16; 260];
            let (mut name_len, mut ty, mut value, mut size) = (260u32, 0u32, 0u32, 4u32);
            let r = RegEnumValueW(
                key,
                i,
                name.as_mut_ptr(),
                &mut name_len,
                null_mut(),
                &mut ty,
                &mut value as *mut _ as _,
                &mut size,
            );
            if r != 0 {
                break;
            }
            if ty == REG_DWORD {
                out.push((String::from_utf16_lossy(&name[..name_len as usize]), value));
            }
        }
        RegCloseKey(key);
    }
    out
}

pub struct Key(HKEY);

impl Key {
    pub fn set_dword(&self, name: &str, v: u32) {
        let n = wide(name);
        unsafe { RegSetValueExW(self.0, n.as_ptr(), 0, REG_DWORD, &v as *const _ as _, 4) };
    }
    pub fn set_string(&self, name: &str, v: &str) {
        let (n, w) = (wide(name), wide(v));
        unsafe {
            RegSetValueExW(
                self.0,
                n.as_ptr(),
                0,
                REG_SZ,
                w.as_ptr() as _,
                (w.len() * 2) as u32,
            )
        };
    }
}

/// Create/open `path` for writing and run `f`. false if the key could not be opened.
pub fn with_key(path: &str, f: impl FnOnce(&Key)) -> bool {
    let p = wide(path);
    let mut key: HKEY = null_mut();
    let r = unsafe {
        RegCreateKeyExW(
            HKEY_CURRENT_USER,
            p.as_ptr(),
            0,
            std::ptr::null(),
            0,
            KEY_WRITE,
            std::ptr::null(),
            &mut key,
            null_mut(),
        )
    };
    if r != 0 {
        return false;
    }
    let k = Key(key);
    f(&k);
    unsafe { RegCloseKey(key) };
    true
}

pub fn delete_tree(path: &str) {
    let p = wide(path);
    unsafe { RegDeleteTreeW(HKEY_CURRENT_USER, p.as_ptr()) };
}

pub fn delete_value(path: &str, name: &str) {
    let (p, n) = (wide(path), wide(name));
    unsafe { RegDeleteKeyValueW(HKEY_CURRENT_USER, p.as_ptr(), n.as_ptr()) };
}
