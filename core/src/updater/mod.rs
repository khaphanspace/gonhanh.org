//! Auto-update module for GoNhanh
//!
//! Provides version comparison utilities that can be used across all platforms.
//! HTTP calls are handled by the platform layer (Swift/C#/GTK) for flexibility.

/// Semantic version representation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub pre: Option<String>,
}

impl Version {
    /// Parse a version string like "1.2.3", "v1.2.3", or "1.2.3-pre.100"
    pub fn parse(s: &str) -> Option<Version> {
        let s = s.trim().strip_prefix('v').unwrap_or(s);

        // Split off pre-release suffix: "1.0.121-pre.368" → ("1.0.121", "pre.368")
        let (version_part, pre) = match s.find('-') {
            Some(idx) => (&s[..idx], Some(s[idx + 1..].to_string())),
            None => (s, None),
        };

        let parts: Vec<&str> = version_part.split('.').collect();
        if parts.len() < 2 {
            return None;
        }

        let major = parts[0].parse().ok()?;
        let minor = parts[1].parse().ok()?;
        let patch = parts.get(2).and_then(|p| p.parse().ok()).unwrap_or(0);

        Some(Version {
            major,
            minor,
            patch,
            pre,
        })
    }

    /// Compare two versions (semver rules)
    /// Returns: -1 if self < other, 0 if equal, 1 if self > other
    /// Pre-release has lower precedence: 1.0.1-pre.1 < 1.0.1
    pub fn compare(&self, other: &Version) -> i32 {
        if self.major != other.major {
            return if self.major < other.major { -1 } else { 1 };
        }
        if self.minor != other.minor {
            return if self.minor < other.minor { -1 } else { 1 };
        }
        if self.patch != other.patch {
            return if self.patch < other.patch { -1 } else { 1 };
        }
        // Semver: pre-release < release when major.minor.patch equal
        match (&self.pre, &other.pre) {
            (None, None) => 0,
            (Some(_), None) => -1, // self is pre-release, other is stable → self < other
            (None, Some(_)) => 1,  // self is stable, other is pre-release → self > other
            (Some(a), Some(b)) => a.cmp(b) as i32, // both pre-release: lexicographic
        }
    }

    /// Check if update is available (other > self)
    pub fn has_update(&self, other: &Version) -> bool {
        self.compare(other) < 0
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if let Some(pre) = &self.pre {
            write!(f, "-{}", pre)?;
        }
        Ok(())
    }
}

// ============================================================
// FFI Interface for Version Comparison
// ============================================================

/// Compare two version strings
/// Returns: -1 if v1 < v2, 0 if equal, 1 if v1 > v2, -99 if parse error
///
/// # Safety
/// Callers must ensure v1 and v2 are valid null-terminated C strings or null pointers.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn version_compare(v1: *const i8, v2: *const i8) -> i32 {
    let v1_str = unsafe {
        if v1.is_null() {
            return -99;
        }
        match std::ffi::CStr::from_ptr(v1).to_str() {
            Ok(s) => s,
            Err(_) => return -99,
        }
    };

    let v2_str = unsafe {
        if v2.is_null() {
            return -99;
        }
        match std::ffi::CStr::from_ptr(v2).to_str() {
            Ok(s) => s,
            Err(_) => return -99,
        }
    };

    let ver1 = match Version::parse(v1_str) {
        Some(v) => v,
        None => return -99,
    };

    let ver2 = match Version::parse(v2_str) {
        Some(v) => v,
        None => return -99,
    };

    ver1.compare(&ver2)
}

/// Check if an update is available
/// Returns: 1 if v2 > v1 (update available), 0 if not, -99 if parse error
#[no_mangle]
pub extern "C" fn version_has_update(current: *const i8, latest: *const i8) -> i32 {
    let result = version_compare(current, latest);
    if result == -99 {
        return -99;
    }
    if result < 0 {
        1
    } else {
        0
    }
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parse() {
        let v = Version::parse("1.2.3").unwrap();
        assert_eq!((v.major, v.minor, v.patch), (1, 2, 3));
        assert_eq!(v.pre, None);

        let v = Version::parse("v1.2.3").unwrap();
        assert_eq!((v.major, v.minor, v.patch), (1, 2, 3));

        let v = Version::parse("1.0").unwrap();
        assert_eq!((v.major, v.minor, v.patch), (1, 0, 0));

        assert_eq!(Version::parse("invalid"), None);
    }

    #[test]
    fn test_version_parse_prerelease() {
        let v = Version::parse("1.0.121-pre.368").unwrap();
        assert_eq!((v.major, v.minor, v.patch), (1, 0, 121));
        assert_eq!(v.pre, Some("pre.368".to_string()));

        let v = Version::parse("v2.1.0-beta.1").unwrap();
        assert_eq!((v.major, v.minor, v.patch), (2, 1, 0));
        assert_eq!(v.pre, Some("beta.1".to_string()));
    }

    #[test]
    fn test_version_compare() {
        let v1 = Version::parse("1.0.0").unwrap();
        let v2 = Version::parse("1.0.1").unwrap();
        let v3 = Version::parse("1.1.0").unwrap();
        let v4 = Version::parse("2.0.0").unwrap();

        assert_eq!(v1.compare(&v1), 0);
        assert_eq!(v1.compare(&v2), -1);
        assert_eq!(v2.compare(&v1), 1);
        assert_eq!(v1.compare(&v3), -1);
        assert_eq!(v1.compare(&v4), -1);
    }

    #[test]
    fn test_version_compare_prerelease() {
        // pre-release < stable when same major.minor.patch
        let pre = Version::parse("1.0.121-pre.368").unwrap();
        let stable = Version::parse("1.0.121").unwrap();
        assert_eq!(pre.compare(&stable), -1); // pre < stable

        // pre-release > lower stable version
        let pre = Version::parse("1.0.121-pre.368").unwrap();
        let older = Version::parse("1.0.120").unwrap();
        assert_eq!(pre.compare(&older), 1); // 121-pre > 120

        // THE BUG SCENARIO: should NOT suggest downgrade
        let current = Version::parse("1.0.121-pre.368").unwrap();
        let latest = Version::parse("1.0.120").unwrap();
        assert!(!current.has_update(&latest)); // no update, current is higher
    }

    #[test]
    fn test_has_update() {
        let current = Version::parse("1.0.9").unwrap();
        let latest = Version::parse("1.0.10").unwrap();
        assert!(current.has_update(&latest));

        let same = Version::parse("1.0.9").unwrap();
        assert!(!current.has_update(&same));
    }

    #[test]
    fn test_ffi_version_compare() {
        use std::ffi::CString;

        let v1 = CString::new("1.0.0").unwrap();
        let v2 = CString::new("1.0.1").unwrap();

        assert_eq!(version_compare(v1.as_ptr(), v2.as_ptr()), -1);
        assert_eq!(version_compare(v2.as_ptr(), v1.as_ptr()), 1);
        assert_eq!(version_compare(v1.as_ptr(), v1.as_ptr()), 0);
    }

    #[test]
    fn test_ffi_has_update() {
        use std::ffi::CString;

        let current = CString::new("1.0.9").unwrap();
        let latest = CString::new("1.0.10").unwrap();

        assert_eq!(version_has_update(current.as_ptr(), latest.as_ptr()), 1);
        assert_eq!(version_has_update(latest.as_ptr(), current.as_ptr()), 0);
    }

    #[test]
    fn test_ffi_prerelease_no_downgrade() {
        use std::ffi::CString;

        // 1.0.121-pre.368 should NOT downgrade to 1.0.120
        let current = CString::new("1.0.121-pre.368").unwrap();
        let latest = CString::new("1.0.120").unwrap();
        assert_eq!(version_has_update(current.as_ptr(), latest.as_ptr()), 0);

        // 1.0.121-pre.368 SHOULD update to 1.0.121 (stable release)
        let stable = CString::new("1.0.121").unwrap();
        assert_eq!(version_has_update(current.as_ptr(), stable.as_ptr()), 1);
    }
}
