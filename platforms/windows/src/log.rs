//! Debug log (advanced setting), like macOS /tmp/gonhanh_debug.log

use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

static ENABLED: AtomicBool = AtomicBool::new(false);

pub fn set_enabled(on: bool) {
    ENABLED.store(on, Ordering::Relaxed);
}

pub fn enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

/// %TEMP%\gonhanh_debug.log
pub fn file_path() -> PathBuf {
    std::env::temp_dir().join("gonhanh_debug.log")
}

pub fn write(message: &str) {
    crate::win::debug_output(message);
    if !enabled() {
        return;
    }
    let (h, m, s, ms) = crate::win::local_time();
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path())
    {
        let _ = writeln!(f, "{h:02}:{m:02}:{s:02}.{ms:03} {message}\r");
    }
}
