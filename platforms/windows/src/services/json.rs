//! Minimal JSON reader, enough for the GitHub releases API (no external crates)

#[derive(Clone, Debug, Default, PartialEq)]
pub enum Value {
    #[default]
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Value>),
    Object(Vec<(String, Value)>),
}

static NULL: Value = Value::Null;

impl Value {
    /// Missing key / not an object → Null
    pub fn get(&self, key: &str) -> &Value {
        match self {
            Value::Object(members) => members
                .iter()
                .find(|(k, _)| k == key)
                .map_or(&NULL, |(_, v)| v),
            _ => &NULL,
        }
    }
    pub fn is_true(&self) -> bool {
        matches!(self, Value::Bool(true))
    }
    pub fn as_str(&self) -> &str {
        match self {
            Value::String(s) => s,
            _ => "",
        }
    }
    pub fn items(&self) -> &[Value] {
        match self {
            Value::Array(items) => items,
            _ => &[],
        }
    }
}

pub fn parse(text: &str) -> Option<Value> {
    let mut p = Parser {
        s: text.as_bytes(),
        i: 0,
        depth: 0,
    };
    let v = p.value()?;
    p.ws();
    (p.i == p.s.len()).then_some(v)
}

struct Parser<'a> {
    s: &'a [u8],
    i: usize,
    depth: u32,
}

impl Parser<'_> {
    fn ws(&mut self) {
        while self.i < self.s.len() && matches!(self.s[self.i], b' ' | b'\t' | b'\n' | b'\r') {
            self.i += 1;
        }
    }

    fn eat(&mut self, c: u8) -> bool {
        self.ws();
        if self.s.get(self.i) == Some(&c) {
            self.i += 1;
            true
        } else {
            false
        }
    }

    fn literal(&mut self, word: &str) -> bool {
        if self.s[self.i..].starts_with(word.as_bytes()) {
            self.i += word.len();
            true
        } else {
            false
        }
    }

    fn hex4(&mut self) -> Option<u32> {
        let digits = std::str::from_utf8(self.s.get(self.i..self.i + 4)?).ok()?;
        let v = u32::from_str_radix(digits, 16).ok()?;
        self.i += 4;
        Some(v)
    }

    fn string(&mut self) -> Option<String> {
        if !self.eat(b'"') {
            return None;
        }
        let mut out: Vec<u8> = Vec::new();
        loop {
            let c = *self.s.get(self.i)?;
            self.i += 1;
            match c {
                b'"' => return String::from_utf8(out).ok(),
                b'\\' => {
                    let e = *self.s.get(self.i)?;
                    self.i += 1;
                    let ch = match e {
                        b'"' => '"',
                        b'\\' => '\\',
                        b'/' => '/',
                        b'b' => '\u{8}',
                        b'f' => '\u{c}',
                        b'n' => '\n',
                        b'r' => '\r',
                        b't' => '\t',
                        b'u' => {
                            let mut cp = self.hex4()?;
                            if (0xD800..0xDC00).contains(&cp)
                                && self.s[self.i..].starts_with(b"\\u")
                            {
                                self.i += 2; // surrogate pair
                                let lo = self.hex4()?;
                                cp = 0x10000
                                    + ((cp - 0xD800) << 10)
                                    + (lo.wrapping_sub(0xDC00) & 0x3FF);
                            }
                            char::from_u32(cp).unwrap_or('\u{FFFD}')
                        }
                        _ => return None,
                    };
                    out.extend_from_slice(ch.encode_utf8(&mut [0; 4]).as_bytes());
                }
                c => out.push(c),
            }
        }
    }

    fn value(&mut self) -> Option<Value> {
        self.depth += 1;
        if self.depth > 64 {
            return None; // GitHub JSON is shallow; refuse pathological input
        }
        self.ws();
        let v = match *self.s.get(self.i)? {
            b'{' => {
                self.i += 1;
                let mut members = Vec::new();
                if !self.eat(b'}') {
                    loop {
                        self.ws();
                        let key = self.string()?;
                        if !self.eat(b':') {
                            return None;
                        }
                        members.push((key, self.value()?));
                        if !self.eat(b',') {
                            break;
                        }
                    }
                    if !self.eat(b'}') {
                        return None;
                    }
                }
                Value::Object(members)
            }
            b'[' => {
                self.i += 1;
                let mut items = Vec::new();
                if !self.eat(b']') {
                    loop {
                        items.push(self.value()?);
                        if !self.eat(b',') {
                            break;
                        }
                    }
                    if !self.eat(b']') {
                        return None;
                    }
                }
                Value::Array(items)
            }
            b'"' => Value::String(self.string()?),
            _ if self.literal("true") => Value::Bool(true),
            _ if self.literal("false") => Value::Bool(false),
            _ if self.literal("null") => Value::Null,
            _ => {
                let start = self.i;
                while self.i < self.s.len()
                    && matches!(
                        self.s[self.i],
                        b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9'
                    )
                {
                    self.i += 1;
                }
                Value::Number(
                    std::str::from_utf8(&self.s[start..self.i])
                        .ok()?
                        .parse()
                        .ok()?,
                )
            }
        };
        self.depth -= 1;
        Some(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nested_values_and_unicode() {
        let v = parse(r#"{"a":[1,true,null,"xệ\n"],"b":{"c":"d"}}"#).unwrap();
        assert_eq!(v.get("a").items().len(), 4);
        assert_eq!(v.get("a").items()[3].as_str(), "xệ\n");
        assert_eq!(v.get("b").get("c").as_str(), "d");
        assert_eq!(v.get("missing"), &Value::Null);
    }

    #[test]
    fn rejects_garbage() {
        assert!(parse(r#"{"a":}"#).is_none());
        assert!(parse("[1,2").is_none());
        assert!(parse("1 2").is_none());
    }
}
