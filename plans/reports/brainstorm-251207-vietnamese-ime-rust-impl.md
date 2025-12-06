# Vietnamese IME - Rust Implementation (Simple Design)

## Goal
Bộ gõ tiếng Việt đơn giản với 2 kiểu gõ: **Telex** và **VNI**.

---

## Core Concepts

### Backspace Technique
User gõ `as` → engine:
1. Xóa `a` (1 backspace)
2. Gửi `á`

### Buffer + Flags
Mỗi ký tự lưu: keycode, caps, tone (^/˘), mark (sắc/huyền/hỏi/ngã/nặng)

### Hook Result
Engine trả về: Action + số backspace + ký tự mới

---

## Module Structure

```
core/src/
├── lib.rs              # FFI exports
├── engine/
│   ├── mod.rs          # Engine struct + logic
│   └── buffer.rs       # Typing buffer
├── input/
│   ├── mod.rs          # Method trait
│   ├── telex.rs        # Telex rules
│   └── vni.rs          # VNI rules
└── data/
    ├── mod.rs          # Data utils
    ├── chars.rs        # Unicode char table
    └── keys.rs         # Platform keycodes
```

**3 folders, 9 files.**

---

## Data Types

```rust
// engine/buffer.rs
pub const MAX: usize = 32;

#[derive(Clone, Copy, Default)]
pub struct Char {
    pub key: u16,
    pub caps: bool,
    pub tone: u8,   // 0=none, 1=hat(^), 2=breve(˘)
    pub mark: u8,   // 0=none, 1=sắc, 2=huyền, 3=hỏi, 4=ngã, 5=nặng
}

pub struct Buffer {
    data: [Char; MAX],
    len: usize,
}
```

```rust
// engine/mod.rs
#[derive(Clone, Copy)]
pub enum Action {
    None,
    Send,
    Restore,
}

pub struct Result {
    pub action: Action,
    pub backspace: u8,
    pub chars: [u32; MAX],
    pub count: u8,
}

pub struct Engine {
    buf: Buffer,
    method: u8,     // 0=Telex, 1=VNI
    spell: bool,
    modern: bool,   // oà vs òa
}
```

---

## Input Method

```rust
// input/mod.rs
pub trait Method {
    fn is_mark(&self, key: u16) -> Option<u8>;
    fn is_tone(&self, key: u16) -> Option<u8>;
    fn is_d(&self, key: u16) -> bool;
    fn is_z(&self, key: u16) -> bool;
}

// input/telex.rs
pub struct Telex;
// s=sắc, f=huyền, r=hỏi, x=ngã, j=nặng
// aa/ee/oo=hat, aw/ow/uw=breve, dd=đ, z=remove

// input/vni.rs
pub struct Vni;
// 1=sắc, 2=huyền, 3=hỏi, 4=ngã, 5=nặng
// 6=hat, 7=ă, 8=ơ/ư, 9=đ, 0=remove
```

---

## Data (chars + keys)

```rust
// data/chars.rs
pub fn to_char(key: u16, caps: bool, tone: u8, mark: u8) -> Option<char> {
    // key=A, tone=1(hat), mark=1(sắc) -> 'Ấ'/'ấ'
}

// data/keys.rs (macOS)
pub const A: u16 = 0;
pub const S: u16 = 1;
pub const D: u16 = 2;
// ...
pub const SPACE: u16 = 49;
pub const DELETE: u16 = 51;
```

---

## Engine Logic

```rust
// engine/mod.rs
impl Engine {
    pub fn on_key(&mut self, key: u16, caps: bool, ctrl: bool) -> Result {
        if ctrl || is_break(key) {
            self.buf.clear();
            return Result::none();
        }

        if key == SPACE {
            self.buf.clear();
            return Result::none();
        }

        if key == DELETE {
            self.buf.pop();
            return Result::none();
        }

        self.process(key, caps)
    }

    fn process(&mut self, key: u16, caps: bool) -> Result {
        let m = self.method();

        if let Some(mark) = m.is_mark(key) {
            return self.add_mark(mark);
        }

        if let Some(tone) = m.is_tone(key) {
            return self.add_tone(key, tone, caps);
        }

        if m.is_d(key) {
            return self.add_d(caps);
        }

        if m.is_z(key) {
            return self.remove_mark();
        }

        self.buf.push(Char { key, caps, ..Default::default() });
        Result::none()
    }
}
```

---

## FFI

```rust
// lib.rs
mod engine;
mod input;
mod data;

use engine::{Engine, Result};

static mut ENGINE: Option<Engine> = None;

#[no_mangle]
pub extern "C" fn ime_init() {
    unsafe { ENGINE = Some(Engine::new()); }
}

#[no_mangle]
pub extern "C" fn ime_key(key: u16, caps: bool, ctrl: bool) -> *const Result {
    unsafe {
        ENGINE.as_mut().map_or(std::ptr::null(), |e| {
            Box::into_raw(Box::new(e.on_key(key, caps, ctrl)))
        })
    }
}

#[no_mangle]
pub extern "C" fn ime_method(m: u8) {
    unsafe { if let Some(e) = ENGINE.as_mut() { e.method = m; } }
}

#[no_mangle]
pub extern "C" fn ime_free(r: *mut Result) {
    if !r.is_null() { unsafe { drop(Box::from_raw(r)); } }
}
```

---

## Summary

| Folder | Files | Purpose |
|--------|-------|---------|
| engine/ | mod.rs, buffer.rs | Core logic |
| input/ | mod.rs, telex.rs, vni.rs | Input methods |
| data/ | mod.rs, chars.rs, keys.rs | Chars & keycodes |
| root | lib.rs | FFI |

**3 folders, 9 files, ~600 lines.**

---

## Skip (add later if needed)

- ❌ TCVN3, VNI Windows, Unicode Compound
- ❌ Macro, Smart switch, Convert tool
- ❌ Quick Telex (cc→ch), Quick consonant (f→ph)

---

*Updated: 2024-12-07*
