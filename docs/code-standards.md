# Gõ Nhanh: Code Standards & Guidelines

## Rust Coding Standards

### Formatting & Linting
- **Formatter**: `cargo fmt` (automatic, non-negotiable)
- **Linter**: `cargo clippy -- -D warnings` (no warnings allowed)
- **Pre-commit**: Format check runs automatically on all PRs

### Code Style
- **Naming**: snake_case for functions/variables, CamelCase for types
- **Comments**: Inline comments for "why", not "what"
- **Documentation**: Public items require rustdoc comments (`///`)
- **Module Layout**:
  ```rust
  //! Module-level documentation

  // Imports
  use crate::module::Item;

  // Constants
  pub const MAX_BUFFER: usize = 64;

  // Type definitions
  pub struct MyStruct { }

  // Public functions
  pub fn my_function() { }

  // Private implementation
  fn private_helper() { }

  #[cfg(test)]
  mod tests { }
  ```

### Zero-Dependency Philosophy
- **Core** (`core/src/`): **Absolutely no external dependencies** in production
- **Rationale**: FFI library must be lightweight and self-contained
- **Allowed**: Only `rstest` in dev-dependencies for test parametrization
- **Example**: Character tables are hardcoded, not loaded from files

### API Guidelines
- **FFI Safety**: All public functions marked `extern "C"` are unsafe by contract
- **Memory**: FFI results must be freed by caller (`ime_free(ptr)`)
- **Error Handling**: Return `null` or default value, never panic (C compatibility)
- **Documentation**: C/FFI comments in code blocks (see lib.rs examples)

### Testing
- **Coverage**: Every module must have `#[cfg(test)] mod tests { }` and integration tests in `core/tests/`
- **Parametrization**: Use `#[rstest]` for multiple test cases
- **Integration**: `core/tests/` directory for full pipeline tests (160+ tests, 2100+ lines)
- **Test Files**:
  - `unit_test.rs` - Individual module tests
  - `typing_test.rs` - Full keystroke sequences (Telex + VNI)
  - `engine_test.rs` - Engine state + initialization
  - `integration_test.rs` - End-to-end keystroke→output
  - `paragraph_test.rs` - Multi-word paragraph typing
- **Naming**: `test_feature_case_expected` (e.g., `test_telex_a_s_returns_á`)
- **Run**: `make test` or `cd core && cargo test`

### Examples
```rust
/// Convert Vietnamese vowel to uppercase.
///
/// # Arguments
/// * `ch` - Lowercase Vietnamese vowel (a-z)
///
/// # Returns
/// Uppercase equivalent using Unicode rules
pub fn to_uppercase(ch: char) -> char {
    ch.to_uppercase().next().unwrap_or(ch)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case('á', 'Á')]
    #[case('ơ', 'Ơ')]
    #[case('a', 'A')]
    fn test_vietnamese_uppercase(#[case] input: char, #[case] expected: char) {
        assert_eq!(to_uppercase(input), expected);
    }
}
```

## Swift Coding Standards

### Style Guide
- **Authority**: [Google Swift Style Guide](https://google.github.io/swift-style-guide/)
- **Formatting**: Follow Xcode default formatting (4-space indentation)
- **Naming**: camelCase for variables/functions, PascalCase for types/enums

### File Organization
```swift
// MARK: - Imports
import Foundation
import AppKit

// MARK: - Constants & Type Definitions
let kSomeConstant = "value"

class MyClass {
    // MARK: - Properties
    var property: Type

    // MARK: - Lifecycle
    init() { }

    // MARK: - Public Methods
    func publicMethod() { }

    // MARK: - Private Methods
    private func privateMethod() { }
}

// MARK: - Extensions
extension MyClass: SomeProtocol { }
```

### Accessibility & Permissions
- **Accessibility Permission**: Required for CGEventTap (keyboard hook)
- **User Prompt**: Show alert if permission denied on first run
- **Debug Mode**: Check `/tmp/gonhanh_debug.log` for detailed logs
  ```swift
  func debugLog(_ message: String) {
      let logPath = "/tmp/gonhanh_debug.log"
      guard FileManager.default.fileExists(atPath: logPath) else { return }
      // ... write to file
  }
  ```

### Error Handling
- **Assertions**: Use for debug-only checks
- **Errors**: Handle gracefully with user-facing messages
- **Logging**: Debug logs for troubleshooting, not production errors

### Concurrency
- **Main Thread**: UI updates always on `DispatchQueue.main`
- **Background**: Use `async` for network requests, CPU-intensive work
- **Example**:
  ```swift
  DispatchQueue.main.async {
      self.updateUI()
  }
  ```

## FFI (Foreign Function Interface) Conventions

### C ABI Compatibility
- **Representation**: `#[repr(C)]` for all shared structs
- **Types**: Use fixed-size types (u8, u16, u32, not usize)
- **Alignment**: Match Rust layout exactly in Swift struct

### Struct Layout (ImeResult Example)
```rust
// Rust
#[repr(C)]
pub struct Result {
    pub chars: [u32; 32],    // Fixed-size array (128 bytes)
    pub action: u8,           // 1 byte
    pub backspace: u8,        // 1 byte
    pub count: u8,            // 1 byte
    pub _pad: u8,             // 1 byte padding
}
```

```swift
// Swift - MUST match Rust layout byte-for-byte
struct ImeResult {
    var chars: (UInt32, UInt32, ..., UInt32)  // 32 elements
    var action: UInt8
    var backspace: UInt8
    var count: UInt8
    var _pad: UInt8
}
```

### Pointer Management
- **Ownership**: Function that allocates owns the pointer
- **Deallocation**: Caller must call `ime_free(ptr)` to deallocate
- **Safety**: Use `defer { ime_free(ptr) }` to guarantee cleanup

```swift
guard let resultPtr = ime_key(keyCode, caps, ctrl) else { return }
defer { ime_free(resultPtr) }

let result = resultPtr.pointee
// Process result...
```

### Function Declarations
```swift
// Import with exact name and signature
@_silgen_name("ime_key")
func ime_key(_ key: UInt16, _ caps: Bool, _ ctrl: Bool) -> UnsafeMutablePointer<ImeResult>?

// Safety: Check for null, use defer for cleanup
if let resultPtr = ime_key(keyCode, caps, ctrl) {
    defer { ime_free(resultPtr) }
    // Safe to use
}
```

## Commit Message Format

Follow [Conventional Commits](https://www.conventionalcommits.org/) specification.

### Format
```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types
| Type | Purpose | Example |
|------|---------|---------|
| **feat** | New feature | `feat(engine): add shortcut table expansion` |
| **fix** | Bug fix | `fix(transform): correct ư placement in compound vowels` |
| **docs** | Documentation | `docs(ffi): clarify memory ownership in ime_free` |
| **style** | Formatting | `style(rust): apply cargo fmt to engine module` |
| **test** | Tests | `test(validation): add edge case for invalid syllables` |
| **chore** | Build/CI | `chore(ci): update GitHub Actions workflow` |
| **refactor** | Code reorganization | `refactor(buffer): optimize circular buffer lookup` |

### Scope
- `engine`, `input`, `data`, `ffi`, `ui`, `macos`, `ci`, `docs`, etc.
- Specific to file/module being changed

### Subject Line
- Imperative mood: "add", not "adds" or "added"
- Lowercase first letter
- No period at end
- Maximum 50 characters
- Use as continuation of "If applied, this commit will..."

### Body
- Explain what and why, not how
- Wrap at 72 characters
- Separate from subject with blank line
- Reference issues: "Closes #123"

### Examples
```
feat(engine): add user shortcut table support

Implement ShortcutTable struct to store user-defined abbreviations
with priority matching. Allows users to define custom transforms
like "hv" → "không" (no space).

Closes #45
```

```
fix(transform): handle ư vowel in compound patterns

The ư vowel was not correctly recognized in sequences like "ưu" and
"ươ" due to missing pattern in validation. Add explicit check for
horn modifier on u vowel.

Fixes #78
```

## Documentation Standards

### Code Comments
- **Module-level**: `//!` with purpose and usage examples
- **Function-level**: `///` with Args, Returns, Safety (if applicable)
- **Inline**: `//` for non-obvious logic (skip obvious comments)

### Examples in Docs
```rust
/// Process keystroke and transform Vietnamese text.
///
/// # Arguments
/// * `key` - macOS virtual keycode (0-127)
/// * `caps` - true if Shift or CapsLock pressed
///
/// # Returns
/// Pointer to Result struct with action, backspace count, output chars.
/// Caller must free with `ime_free()`.
///
/// # Example
/// ```c
/// ImeResult* r = ime_key(keys::A, false, false);
/// if (r && r->action == 1) {
///     // Send r->backspace deletes, then r->chars
/// }
/// ime_free(r);
/// ```
#[no_mangle]
pub extern "C" fn ime_key(key: u16, caps: bool, ctrl: bool) -> *mut Result { }
```

## Version Numbering

- **Semantic Versioning**: MAJOR.MINOR.PATCH (e.g., 1.2.3)
- **MAJOR**: Breaking changes (rare)
- **MINOR**: New features, backward compatible
- **PATCH**: Bug fixes only
- **Release**: Tag with `v` prefix (e.g., `v1.2.3`)
- **Current Version**: v1.0.21+ (macOS + Windows production-ready)

## Pull Request Guidelines

- **Title**: Follow commit message format
- **Description**: Reference related issues
- **Changes**: One logical change per PR (no mega-PRs)
- **Tests**: All new code must have tests
- **CI**: Must pass format, clippy, and test suite
- **Review**: At least one approval before merge

---

**Last Updated**: 2025-12-10
**Enforced By**: GitHub Actions CI (`ci.yml`)
**Test Coverage**: 160+ integration tests across 5 test files in `core/tests/`
