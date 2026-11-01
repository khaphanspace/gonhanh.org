//! Clean Paste: text normalization for clipboard content.
//!
//! Applies formatting rules and character replacements to produce
//! clean, readable plaintext from messy clipboard input (Facebook,
//! ChatGPT, etc.).

/// A single character replacement mapping (from → to).
pub struct CharMapping {
    pub from: char,
    pub to: &'static str,
}

/// Default character mappings for AI/Unicode normalization.
pub const DEFAULT_MAPPINGS: &[CharMapping] = &[
    // Smart quotes → straight
    CharMapping {
        from: '\u{201C}',
        to: "\"",
    }, // "
    CharMapping {
        from: '\u{201D}',
        to: "\"",
    }, // "
    CharMapping {
        from: '\u{2018}',
        to: "'",
    }, // '
    CharMapping {
        from: '\u{2019}',
        to: "'",
    }, // '
    // Dashes
    CharMapping {
        from: '\u{2014}',
        to: "-",
    }, // — em dash
    CharMapping {
        from: '\u{2013}',
        to: "-",
    }, // – en dash
    // Symbols
    CharMapping {
        from: '\u{2026}',
        to: "...",
    }, // … ellipsis
    CharMapping {
        from: '\u{2022}',
        to: "-",
    }, // • bullet
    CharMapping {
        from: '\u{2192}',
        to: "->",
    }, // → arrow right
    CharMapping {
        from: '\u{2190}',
        to: "<-",
    }, // ← arrow left
    CharMapping {
        from: '\u{00D7}',
        to: "x",
    }, // × multiply
    // Whitespace
    CharMapping {
        from: '\u{00A0}',
        to: " ",
    }, // non-breaking space
    CharMapping {
        from: '\u{200B}',
        to: "",
    }, // zero-width space
    CharMapping {
        from: '\u{200C}',
        to: "",
    }, // zero-width non-joiner
    CharMapping {
        from: '\u{200D}',
        to: "",
    }, // zero-width joiner
    CharMapping {
        from: '\u{FEFF}',
        to: "",
    }, // BOM / zero-width no-break space
    CharMapping {
        from: '\u{202F}',
        to: " ",
    }, // narrow no-break space
    CharMapping {
        from: '\u{2060}',
        to: "",
    }, // word joiner
    CharMapping {
        from: '\u{3000}',
        to: " ",
    }, // fullwidth space (CJK)
];

/// Target platform for markdown conversion.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Platform {
    /// Keep markdown as-is (editors, IDE, default)
    None = 0,
    /// Discord: **bold**, *italic*, ~~strike~~, > quote, `code`, - list, NO headings/tables
    Discord = 1,
    /// Slack: *bold*, _italic_, ~strike~, > quote, `code`, NO headings/tables
    Slack = 2,
    /// Plain text: strip all markdown (Zalo, Facebook, Messenger, iMessage)
    PlainText = 3,
    /// Telegram: **bold**, *italic*, ~~strike~~, `code`, NO headings/tables
    Telegram = 4,
}

impl Platform {
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::Discord,
            2 => Self::Slack,
            3 => Self::PlainText,
            4 => Self::Telegram,
            _ => Self::None,
        }
    }
}

/// Clean text with formatting rules, character mappings, and platform-aware markdown.
///
/// # Arguments
/// * `input` - Raw text to clean
/// * `custom_mappings` - Custom character replacements (from_char, to_str)
/// * `apply_format` - Whether to apply formatting rules (collapse lines, etc.)
/// * `platform` - Target platform for markdown conversion (0=none, 1=discord, etc.)
///
/// Processing order:
/// 1. Replace chars via mappings (defaults + custom)
/// 2. Format text (trim, collapse, compact bullets) — if apply_format
/// 3. Convert markdown for target platform — if platform != None
pub fn clean_text(
    input: &str,
    custom_mappings: &[(char, &str)],
    apply_format: bool,
    platform: Platform,
) -> String {
    if input.is_empty() {
        return String::new();
    }

    // Step 1: Replace characters, strip escapes, fix LinkedIn hashtags
    let replaced = replace_chars(input, custom_mappings);
    let replaced = strip_backslash_escapes(&replaced);
    let replaced = fix_linkedin_hashtags(&replaced);

    // Step 2: Format
    let formatted = if apply_format {
        format_text(&replaced)
    } else {
        replaced
    };

    // Step 3: Platform-specific markdown conversion
    if platform != Platform::None {
        convert_markdown(&formatted, platform)
    } else {
        formatted
    }
}

/// Replace characters using default + custom mappings.
fn replace_chars(input: &str, custom_mappings: &[(char, &str)]) -> String {
    let mut result = String::with_capacity(input.len());

    for ch in input.chars() {
        // Check custom mappings first (user overrides)
        if let Some((_, to)) = custom_mappings.iter().find(|(from, _)| *from == ch) {
            result.push_str(to);
            continue;
        }
        // Check default mappings
        if let Some(mapping) = DEFAULT_MAPPINGS.iter().find(|m| m.from == ch) {
            result.push_str(mapping.to);
            continue;
        }
        result.push(ch);
    }

    result
}

/// Strip markdown escape backslashes: `\-` → `-`, `\.` → `.`, etc.
fn strip_backslash_escapes(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            if let Some(&next) = chars.peek() {
                // Only strip backslash before punctuation (markdown escapes)
                if next.is_ascii_punctuation() {
                    result.push(next);
                    chars.next();
                    continue;
                }
            }
        }
        result.push(ch);
    }
    result
}

/// Fix LinkedIn-style hashtags: `hashtag#buildinpublic` → `#buildinpublic`.
/// LinkedIn copies `#tag` as `hashtag#tag` in plain text clipboard.
fn fix_linkedin_hashtags(input: &str) -> String {
    let lower = input.to_lowercase();
    let bytes = input.as_bytes();
    let pat = b"hashtag#";
    let pat_len = pat.len();
    let mut result = String::with_capacity(input.len());
    let mut i = 0;

    while i < bytes.len() {
        if i + pat_len <= bytes.len() && lower.as_bytes()[i..i + pat_len] == *pat {
            result.push('#');
            i += pat_len;
        } else {
            // Safe: we're iterating byte-by-byte but need to handle UTF-8
            let ch = input[i..].chars().next().unwrap();
            result.push(ch);
            i += ch.len_utf8();
        }
    }

    result
}

/// Apply formatting rules to normalize text structure.
fn format_text(input: &str) -> String {
    let lines: Vec<&str> = input.lines().collect();
    let mut result: Vec<String> = Vec::with_capacity(lines.len());

    for line in &lines {
        // Trim trailing whitespace, collapse multiple spaces
        let trimmed = line.trim_end();
        let collapsed = collapse_spaces(trimmed);
        result.push(collapsed);
    }

    // Collapse 3+ consecutive blank lines → 1
    result = collapse_blank_lines(result);

    // Clean headings: remove blank line after, strip **bold** markers
    result = clean_headings(result);

    // Join colon lines with next non-empty line
    result = join_after_colon(result);

    // Compact bullet lists (remove blank lines between bullets)
    result = compact_bullets(result);

    result.join("\n")
}

/// Collapse multiple spaces into single space (preserving leading indent).
fn collapse_spaces(line: &str) -> String {
    if line.is_empty() {
        return String::new();
    }

    let mut result = String::with_capacity(line.len());
    let mut prev_space = false;
    let mut in_leading = true;

    for ch in line.chars() {
        if ch == ' ' {
            if in_leading {
                // Keep leading spaces as-is (preserve indentation)
                result.push(ch);
            } else if !prev_space {
                result.push(ch);
                prev_space = true;
            }
            // else: skip duplicate space
        } else {
            in_leading = false;
            prev_space = false;
            result.push(ch);
        }
    }

    result
}

/// Collapse 3+ consecutive blank lines into 1 blank line.
fn collapse_blank_lines(lines: Vec<String>) -> Vec<String> {
    let mut result: Vec<String> = Vec::with_capacity(lines.len());
    let mut blank_count = 0;

    for line in lines {
        if line.trim().is_empty() {
            blank_count += 1;
            if blank_count <= 1 {
                result.push(line);
            }
        } else {
            blank_count = 0;
            result.push(line);
        }
    }

    result
}

/// Clean heading lines: remove blank line after heading, strip **bold** markers.
fn clean_headings(lines: Vec<String>) -> Vec<String> {
    let mut result: Vec<String> = Vec::with_capacity(lines.len());
    let mut skip_next_blank = false;

    for line in lines {
        if skip_next_blank && line.trim().is_empty() {
            skip_next_blank = false;
            continue;
        }

        if let Some((level, text)) = parse_heading(line.trim_start()) {
            // Strip **bold** from heading text (common in Google Docs exports)
            let clean = strip_bold_markers(text);
            result.push(format!("{} {}", "#".repeat(level), clean));
            skip_next_blank = true;
        } else {
            skip_next_blank = false;
            result.push(line);
        }
    }

    result
}

/// If a line ends with `:` and next line is blank, remove the blank line.
fn join_after_colon(lines: Vec<String>) -> Vec<String> {
    let mut result: Vec<String> = Vec::with_capacity(lines.len());
    let mut skip_next_blank = false;

    for line in lines {
        if skip_next_blank && line.trim().is_empty() {
            skip_next_blank = false;
            continue;
        }
        skip_next_blank = line.trim_end().ends_with(':');
        result.push(line);
    }

    result
}

/// Remove blank lines between bullet items and normalize bullet indent.
fn compact_bullets(lines: Vec<String>) -> Vec<String> {
    let mut result: Vec<String> = Vec::with_capacity(lines.len());

    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim_start();

        if is_bullet_line(trimmed) {
            // Preserve leading indent from original line
            let indent_len = lines[i].len() - lines[i].trim_start().len();
            let indent = &lines[i][..indent_len];

            // Normalize bullet marker (* or •) to "-"
            let skip = if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
                2
            } else {
                // • (Unicode bullet, multi-byte) + space
                trimmed.chars().next().map_or(0, |c| c.len_utf8()) + 1
            };
            let bullet_content = trimmed[skip..].trim_start();
            result.push(format!("{}- {}", indent, bullet_content));

            // Skip blank lines between consecutive bullets
            let mut j = i + 1;
            while j < lines.len() {
                let next_trimmed = lines[j].trim();
                if next_trimmed.is_empty() {
                    // Peek ahead: if next non-blank is also bullet, skip this blank
                    let mut k = j + 1;
                    while k < lines.len() && lines[k].trim().is_empty() {
                        k += 1;
                    }
                    if k < lines.len() && is_bullet_line(lines[k].trim_start()) {
                        j += 1; // Skip blank line
                        continue;
                    }
                }
                break;
            }
            i = j;
        } else {
            result.push(lines[i].clone());
            i += 1;
        }
    }

    result
}

/// Check if a line starts with a bullet marker.
fn is_bullet_line(trimmed: &str) -> bool {
    trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("• ")
    // Unicode bullet (pre-replacement)
}

// ============================================================
// Platform Markdown Conversion
// ============================================================

/// Convert markdown to platform-specific format.
fn convert_markdown(input: &str, platform: Platform) -> String {
    let lines: Vec<&str> = input.lines().collect();
    let mut result: Vec<String> = Vec::with_capacity(lines.len());
    let mut in_code_block = false;

    for line in &lines {
        // Track code blocks (``` ... ```) — don't convert inside them
        if line.trim_start().starts_with("```") {
            in_code_block = !in_code_block;
            match platform {
                Platform::PlainText => {
                    // Strip code fences for plain text
                    if !in_code_block {
                        continue; // skip closing fence
                    }
                    continue; // skip opening fence
                }
                _ => {
                    result.push(line.to_string());
                    continue;
                }
            }
        }

        if in_code_block {
            result.push(line.to_string());
            continue;
        }

        let converted = convert_line(line, platform);
        result.push(converted);
    }

    result.join("\n")
}

/// Convert a single line of markdown for the target platform.
fn convert_line(line: &str, platform: Platform) -> String {
    let trimmed = line.trim_start();

    // Handle headings: # Heading → platform-specific
    if let Some(heading) = parse_heading(trimmed) {
        return convert_heading(heading.0, heading.1, platform);
    }

    // Handle horizontal rules: ---, ***, ___
    if is_horizontal_rule(trimmed) {
        return match platform {
            Platform::PlainText => String::new(),
            _ => "---".to_string(),
        };
    }

    // Handle blockquotes: > text
    if let Some(quote_text) = trimmed.strip_prefix("> ") {
        return match platform {
            Platform::PlainText => quote_text.to_string(),
            Platform::Discord | Platform::Slack | Platform::Telegram => {
                format!("> {}", quote_text)
            }
            Platform::None => line.to_string(),
        };
    }

    // Handle table lines (| col | col |)
    if trimmed.starts_with('|') && trimmed.ends_with('|') {
        return convert_table_line(trimmed, platform);
    }

    // Convert inline markdown formatting
    convert_inline(line, platform)
}

/// Parse heading: returns (level, text) or None.
fn parse_heading(line: &str) -> Option<(usize, &str)> {
    let mut level = 0;
    let mut chars = line.chars();
    while chars.as_str().starts_with('#') {
        level += 1;
        chars.next();
    }
    if level > 0 && level <= 6 {
        // Markdown heading requires space after # (e.g., "# Title", not "#hashtag")
        let remaining = chars.as_str();
        if remaining.starts_with(' ') {
            let rest = remaining.trim_start();
            if !rest.is_empty() {
                return Some((level, rest));
            }
        }
    }
    None
}

/// Strip wrapping **bold** markers from text (e.g., "**Title**" → "Title").
/// Common in Google Docs / Notion markdown exports where headings are also bolded.
fn strip_bold_markers(text: &str) -> &str {
    let t = text.trim();
    if t.starts_with("**") && t.ends_with("**") && t.len() > 4 {
        &t[2..t.len() - 2]
    } else {
        t
    }
}

/// Convert heading to platform format.
fn convert_heading(level: usize, text: &str, platform: Platform) -> String {
    let clean = strip_bold_markers(text);
    match platform {
        Platform::Discord | Platform::Telegram => {
            // Discord/Telegram: no heading support → always **bold**
            format!("**{}**", clean)
        }
        Platform::Slack => {
            // Slack: no heading → *bold* (Slack's bold syntax)
            if level == 1 {
                format!("*{}*", clean.to_uppercase())
            } else {
                format!("*{}*", clean)
            }
        }
        Platform::PlainText => {
            // Plain text: just the text, uppercase for h1
            if level == 1 {
                clean.to_uppercase()
            } else {
                clean.to_string()
            }
        }
        Platform::None => format!("{} {}", "#".repeat(level), clean), // strip **bold** even for raw markdown
    }
}

/// Check if line is a horizontal rule (---, ***, ___).
fn is_horizontal_rule(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.len() < 3 {
        return false;
    }
    let first = trimmed.chars().next().unwrap();
    (first == '-' || first == '*' || first == '_')
        && trimmed.chars().all(|c| c == first || c == ' ')
}

/// Convert table line for platforms that don't support tables.
fn convert_table_line(line: &str, platform: Platform) -> String {
    match platform {
        Platform::Discord | Platform::Slack | Platform::Telegram | Platform::PlainText => {
            // Check if it's a separator line (|---|---|)
            let inner = &line[1..line.len() - 1];
            if inner
                .chars()
                .all(|c| c == '-' || c == '|' || c == ':' || c == ' ')
            {
                return String::new(); // Skip separator
            }
            // Convert cells to "col1 | col2 | col3"
            let cells: Vec<&str> = inner.split('|').map(|c| c.trim()).collect();
            cells.join(" | ")
        }
        Platform::None => line.to_string(),
    }
}

/// Convert inline markdown formatting (bold, italic, code, strike).
fn convert_inline(line: &str, platform: Platform) -> String {
    let mut result = line.to_string();

    // Simplify redundant links [url](url) → url for all platforms
    result = simplify_links(&result);

    match platform {
        Platform::PlainText => {
            // Strip all inline markdown
            result = strip_inline_markdown(&result);
        }
        Platform::Slack => {
            // Slack: **bold** → *bold*, *italic* → _italic_, ~~strike~~ → ~strike~
            result = convert_bold_italic_slack(&result);
        }
        Platform::Discord | Platform::Telegram => {
            // Already standard markdown, keep as-is
        }
        Platform::None => {}
    }

    result
}

/// Simplify redundant markdown links where text == url.
/// `[https://example.com](https://example.com)` → `https://example.com`
/// `[Example](https://example.com)` → kept as-is (text differs from url)
fn simplify_links(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        if chars[i] == '[' {
            if let Some((text_end, url_end)) = find_markdown_link(&chars, i) {
                let text: String = chars[i + 1..text_end].iter().collect();
                let url: String = chars[text_end + 2..url_end].iter().collect();

                if text.trim() == url.trim() {
                    // Redundant link: [url](url) → url
                    result.push_str(url.trim());
                } else {
                    // Keep as-is: [text](url)
                    for &c in &chars[i..=url_end] {
                        result.push(c);
                    }
                }
                i = url_end + 1;
                continue;
            }
        }
        result.push(chars[i]);
        i += 1;
    }

    result
}

/// Strip all inline markdown markers for plain text output.
fn strip_inline_markdown(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        // Inline code: `text`
        if chars[i] == '`' {
            if let Some(end) = find_closing(&chars, i + 1, '`') {
                // Keep content, strip backticks
                for &c in &chars[i + 1..end] {
                    result.push(c);
                }
                i = end + 1;
                continue;
            }
        }
        // Bold: **text**
        if i + 1 < len && chars[i] == '*' && chars[i + 1] == '*' {
            if let Some(end) = find_closing_double(&chars, i + 2, '*') {
                for &c in &chars[i + 2..end] {
                    result.push(c);
                }
                i = end + 2;
                continue;
            }
        }
        // Italic: *text*
        if chars[i] == '*' {
            if let Some(end) = find_closing(&chars, i + 1, '*') {
                for &c in &chars[i + 1..end] {
                    result.push(c);
                }
                i = end + 1;
                continue;
            }
        }
        // Strikethrough: ~~text~~
        if i + 1 < len && chars[i] == '~' && chars[i + 1] == '~' {
            if let Some(end) = find_closing_double(&chars, i + 2, '~') {
                for &c in &chars[i + 2..end] {
                    result.push(c);
                }
                i = end + 2;
                continue;
            }
        }
        // Markdown link: [text](url) → text
        if chars[i] == '[' {
            if let Some((text_end, url_end)) = find_markdown_link(&chars, i) {
                for &c in &chars[i + 1..text_end] {
                    result.push(c);
                }
                i = url_end + 1;
                continue;
            }
        }
        result.push(chars[i]);
        i += 1;
    }

    result
}

/// Convert **bold** → *bold* and *italic* → _italic_ for Slack.
fn convert_bold_italic_slack(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        // **bold** → *bold*
        if i + 1 < len && chars[i] == '*' && chars[i + 1] == '*' {
            if let Some(end) = find_closing_double(&chars, i + 2, '*') {
                result.push('*');
                for &c in &chars[i + 2..end] {
                    result.push(c);
                }
                result.push('*');
                i = end + 2;
                continue;
            }
        }
        // *italic* → _italic_
        if chars[i] == '*' {
            if let Some(end) = find_closing(&chars, i + 1, '*') {
                result.push('_');
                for &c in &chars[i + 1..end] {
                    result.push(c);
                }
                result.push('_');
                i = end + 1;
                continue;
            }
        }
        // ~~strike~~ → ~strike~
        if i + 1 < len && chars[i] == '~' && chars[i + 1] == '~' {
            if let Some(end) = find_closing_double(&chars, i + 2, '~') {
                result.push('~');
                for &c in &chars[i + 2..end] {
                    result.push(c);
                }
                result.push('~');
                i = end + 2;
                continue;
            }
        }
        result.push(chars[i]);
        i += 1;
    }

    result
}

/// Find closing single delimiter (e.g., * or `).
fn find_closing(chars: &[char], start: usize, delim: char) -> Option<usize> {
    for i in start..chars.len() {
        if chars[i] == delim {
            return Some(i);
        }
    }
    None
}

/// Find closing double delimiter (e.g., ** or ~~).
fn find_closing_double(chars: &[char], start: usize, delim: char) -> Option<usize> {
    if chars.len() < 2 {
        return None;
    }
    for i in start..chars.len() - 1 {
        if chars[i] == delim && chars[i + 1] == delim {
            return Some(i);
        }
    }
    None
}

/// Find markdown link [text](url), returns (text_end, url_end).
fn find_markdown_link(chars: &[char], start: usize) -> Option<(usize, usize)> {
    // Find ]
    let text_end = find_closing(chars, start + 1, ']')?;
    // Must be followed by (
    if text_end + 1 >= chars.len() || chars[text_end + 1] != '(' {
        return None;
    }
    // Find )
    let url_end = find_closing(chars, text_end + 2, ')')?;
    Some((text_end, url_end))
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_input() {
        assert_eq!(clean_text("", &[], true, Platform::None), "");
    }

    #[test]
    fn test_no_changes_needed() {
        let input = "Hello world";
        assert_eq!(clean_text(input, &[], true, Platform::None), "Hello world");
    }

    #[test]
    fn test_smart_quotes_replacement() {
        let input = "\u{201C}Hello\u{201D} \u{2018}world\u{2019}";
        assert_eq!(
            clean_text(input, &[], false, Platform::None),
            "\"Hello\" 'world'"
        );
    }

    #[test]
    fn test_em_dash_replacement() {
        let input = "one \u{2014} two \u{2013} three";
        assert_eq!(
            clean_text(input, &[], false, Platform::None),
            "one - two - three"
        );
    }

    #[test]
    fn test_ellipsis_replacement() {
        let input = "wait\u{2026}";
        assert_eq!(clean_text(input, &[], false, Platform::None), "wait...");
    }

    #[test]
    fn test_bullet_replacement() {
        let input = "\u{2022} item one";
        assert_eq!(clean_text(input, &[], false, Platform::None), "- item one");
    }

    #[test]
    fn test_arrow_replacement() {
        let input = "A \u{2192} B \u{2190} C";
        assert_eq!(clean_text(input, &[], false, Platform::None), "A -> B <- C");
    }

    #[test]
    fn test_zero_width_chars_removed() {
        let input = "hello\u{200B}world\u{FEFF}test";
        assert_eq!(
            clean_text(input, &[], false, Platform::None),
            "helloworldtest"
        );
    }

    #[test]
    fn test_nbsp_replaced() {
        let input = "hello\u{00A0}world";
        assert_eq!(clean_text(input, &[], false, Platform::None), "hello world");
    }

    #[test]
    fn test_collapse_spaces() {
        let input = "hello    world   test";
        assert_eq!(
            clean_text(input, &[], true, Platform::None),
            "hello world test"
        );
    }

    #[test]
    fn test_collapse_blank_lines() {
        let input = "line1\n\n\n\nline2";
        assert_eq!(
            clean_text(input, &[], true, Platform::None),
            "line1\n\nline2"
        );
    }

    #[test]
    fn test_join_after_colon() {
        let input = "Điều kiện:\n\nhello";
        assert_eq!(
            clean_text(input, &[], true, Platform::None),
            "Điều kiện:\nhello"
        );
    }

    #[test]
    fn test_compact_bullets() {
        let input = "- item 1\n\n- item 2\n\n- item 3";
        assert_eq!(
            clean_text(input, &[], true, Platform::None),
            "- item 1\n- item 2\n- item 3"
        );
    }

    #[test]
    fn test_bullet_indent_normalization() {
        let input = "•     Item with lots of space\n\n•    Another item";
        assert_eq!(
            clean_text(input, &[], true, Platform::None),
            "- Item with lots of space\n- Another item"
        );
    }

    #[test]
    fn test_custom_mappings_override_defaults() {
        // Custom: em dash → " -- " instead of default "-"
        let custom = vec![('\u{2014}', " -- ")];
        let input = "hello\u{2014}world";
        assert_eq!(
            clean_text(input, &custom, false, Platform::None),
            "hello -- world"
        );
    }

    #[test]
    fn test_custom_mappings_new_chars() {
        let custom = vec![('©', "(c)")];
        let input = "Copyright ©";
        assert_eq!(
            clean_text(input, &custom, false, Platform::None),
            "Copyright (c)"
        );
    }

    #[test]
    fn test_format_off_only_replaces() {
        let input = "\u{201C}hello\u{201D}\n\n\n\nworld";
        // format off: replaces chars but keeps blank lines
        assert_eq!(
            clean_text(input, &[], false, Platform::None),
            "\"hello\"\n\n\n\nworld"
        );
    }

    #[test]
    fn test_facebook_style_post() {
        let input = "Title of Post: Important announcement\n\
                      \n\
                      Description of the post\u{2026}\n\
                      \n\
                      Link here: \n\
                      \n\
                      https://example.com\n\
                      \n\
                      Requirements:\n\
                      \n\
                      \u{2022}     First requirement item.\n\
                      \n\
                      \u{2022}    Second requirement item.\n\
                      \n\
                      \u{2022}    Third requirement item.";

        let expected = "Title of Post: Important announcement\n\
                        \n\
                        Description of the post...\n\
                        \n\
                        Link here:\n\
                        https://example.com\n\
                        \n\
                        Requirements:\n\
                        - First requirement item.\n\
                        - Second requirement item.\n\
                        - Third requirement item.";

        assert_eq!(clean_text(input, &[], true, Platform::None), expected);
    }

    #[test]
    fn test_preserve_single_blank_line_between_paragraphs() {
        let input = "Paragraph 1.\n\nParagraph 2.";
        assert_eq!(
            clean_text(input, &[], true, Platform::None),
            "Paragraph 1.\n\nParagraph 2."
        );
    }

    #[test]
    fn test_trailing_space_trimmed() {
        let input = "hello   \nworld   ";
        assert_eq!(clean_text(input, &[], true, Platform::None), "hello\nworld");
    }

    #[test]
    fn test_multiply_sign() {
        let input = "2 \u{00D7} 3";
        assert_eq!(clean_text(input, &[], false, Platform::None), "2 x 3");
    }

    #[test]
    fn test_colon_with_trailing_space() {
        // "Link: \n\nurl" → colon line has trailing space, after trim ends with ":"
        let input = "Link: \n\nhttps://example.com";
        assert_eq!(
            clean_text(input, &[], true, Platform::None),
            "Link:\nhttps://example.com"
        );
    }

    #[test]
    fn test_mixed_bullets_star() {
        let input = "* item 1\n\n* item 2";
        assert_eq!(
            clean_text(input, &[], true, Platform::None),
            "- item 1\n- item 2"
        );
    }

    // ============================================================
    // Platform Markdown Tests
    // ============================================================

    #[test]
    fn test_discord_heading_h1() {
        let input = "# Welcome to Server";
        assert_eq!(
            clean_text(input, &[], false, Platform::Discord),
            "**Welcome to Server**"
        );
    }

    #[test]
    fn test_discord_heading_h2() {
        let input = "## Features";
        assert_eq!(
            clean_text(input, &[], false, Platform::Discord),
            "**Features**"
        );
    }

    #[test]
    fn test_slack_heading() {
        let input = "# Announcement";
        assert_eq!(
            clean_text(input, &[], false, Platform::Slack),
            "*ANNOUNCEMENT*"
        );
    }

    #[test]
    fn test_slack_bold_italic_conversion() {
        let input = "This is **bold** and *italic* text";
        assert_eq!(
            clean_text(input, &[], false, Platform::Slack),
            "This is *bold* and _italic_ text"
        );
    }

    #[test]
    fn test_slack_strikethrough() {
        let input = "This is ~~deleted~~ text";
        assert_eq!(
            clean_text(input, &[], false, Platform::Slack),
            "This is ~deleted~ text"
        );
    }

    #[test]
    fn test_plain_text_strip_all() {
        let input = "**bold** and *italic* and `code` and ~~strike~~";
        assert_eq!(
            clean_text(input, &[], false, Platform::PlainText),
            "bold and italic and code and strike"
        );
    }

    #[test]
    fn test_plain_text_heading() {
        let input = "# Title\n## Subtitle";
        assert_eq!(
            clean_text(input, &[], false, Platform::PlainText),
            "TITLE\nSubtitle"
        );
    }

    #[test]
    fn test_plain_text_link() {
        let input = "Click [here](https://example.com) for info";
        assert_eq!(
            clean_text(input, &[], false, Platform::PlainText),
            "Click here for info"
        );
    }

    #[test]
    fn test_discord_keeps_inline_markdown() {
        let input = "This is **bold** and *italic* and ~~strike~~";
        assert_eq!(
            clean_text(input, &[], false, Platform::Discord),
            "This is **bold** and *italic* and ~~strike~~"
        );
    }

    #[test]
    fn test_table_to_discord() {
        let input = "| Name | Age |\n|------|-----|\n| Alice | 30 |";
        assert_eq!(
            clean_text(input, &[], false, Platform::Discord),
            "Name | Age\n\nAlice | 30"
        );
    }

    #[test]
    fn test_code_block_preserved() {
        let input = "```\nlet x = 1;\n```";
        assert_eq!(
            clean_text(input, &[], false, Platform::Discord),
            "```\nlet x = 1;\n```"
        );
    }

    #[test]
    fn test_code_block_stripped_plain() {
        let input = "```\nlet x = 1;\n```";
        assert_eq!(
            clean_text(input, &[], false, Platform::PlainText),
            "let x = 1;"
        );
    }

    #[test]
    fn test_blockquote_discord() {
        let input = "> This is a quote";
        assert_eq!(
            clean_text(input, &[], false, Platform::Discord),
            "> This is a quote"
        );
    }

    #[test]
    fn test_blockquote_plain() {
        let input = "> This is a quote";
        assert_eq!(
            clean_text(input, &[], false, Platform::PlainText),
            "This is a quote"
        );
    }

    #[test]
    fn test_horizontal_rule_discord() {
        let input = "above\n---\nbelow";
        assert_eq!(
            clean_text(input, &[], false, Platform::Discord),
            "above\n---\nbelow"
        );
    }

    #[test]
    fn test_horizontal_rule_plain() {
        let input = "above\n---\nbelow";
        assert_eq!(
            clean_text(input, &[], false, Platform::PlainText),
            "above\n\nbelow"
        );
    }

    #[test]
    fn test_full_document_to_discord() {
        let input = "# Tin tức\n\n## Chi tiết\n\nĐây là **nội dung** với *in nghiêng*.\n\n| Cột 1 | Cột 2 |\n|-------|-------|\n| A | B |";
        let expected = "**Tin tức**\n\n**Chi tiết**\n\nĐây là **nội dung** với *in nghiêng*.\n\nCột 1 | Cột 2\n\nA | B";
        assert_eq!(clean_text(input, &[], false, Platform::Discord), expected);
    }

    #[test]
    fn test_combined_format_and_platform() {
        // Test that format rules AND platform conversion work together
        let input = "# Title:\n\n\u{2022}     Item 1\n\n\u{2022}    Item 2";
        let expected = "**Title:**\n- Item 1\n- Item 2";
        assert_eq!(clean_text(input, &[], true, Platform::Discord), expected);
    }

    #[test]
    fn test_simplify_redundant_link_discord() {
        let input = "[https://github.com/khaphanspace/gonhanh.org](https://github.com/khaphanspace/gonhanh.org)";
        assert_eq!(
            clean_text(input, &[], false, Platform::Discord),
            "https://github.com/khaphanspace/gonhanh.org"
        );
    }

    #[test]
    fn test_keep_different_link_text() {
        let input = "[Click here](https://example.com)";
        assert_eq!(
            clean_text(input, &[], false, Platform::Discord),
            "[Click here](https://example.com)"
        );
    }

    #[test]
    fn test_simplify_link_plain_text() {
        // PlainText: [url](url) → url, then strip_inline_markdown won't touch it
        let input = "[https://example.com](https://example.com)";
        assert_eq!(
            clean_text(input, &[], false, Platform::PlainText),
            "https://example.com"
        );
    }

    #[test]
    fn test_nested_bullets_preserve_indent() {
        let input = "- Top level\n  - Nested item\n    - Deep nested";
        assert_eq!(
            clean_text(input, &[], true, Platform::None),
            "- Top level\n  - Nested item\n    - Deep nested"
        );
    }

    #[test]
    fn test_nested_bullets_star_to_dash() {
        let input = "- Parent\n  * Child 1\n  * Child 2";
        assert_eq!(
            clean_text(input, &[], true, Platform::None),
            "- Parent\n  - Child 1\n  - Child 2"
        );
    }

    #[test]
    fn test_nested_bullets_compact_blank_lines() {
        let input = "- Item 1\n\n  - Sub 1\n\n  - Sub 2\n\n- Item 2";
        assert_eq!(
            clean_text(input, &[], true, Platform::None),
            "- Item 1\n  - Sub 1\n  - Sub 2\n- Item 2"
        );
    }

    #[test]
    fn test_nested_meeting_notes_style() {
        let input =
            "- Backend\n  - Fix login API\n  - Update database\n- Frontend: Refactor components";
        assert_eq!(
            clean_text(input, &[], true, Platform::None),
            "- Backend\n  - Fix login API\n  - Update database\n- Frontend: Refactor components"
        );
    }

    #[test]
    fn test_heading_removes_blank_line_after() {
        let input = "# Title\n\nContent here";
        assert_eq!(
            clean_text(input, &[], true, Platform::None),
            "# Title\nContent here"
        );
    }

    #[test]
    fn test_heading_strips_bold_markers() {
        let input = "# **Easy AI \\- Retail**\n\n## **Định vị sản phẩm**";
        assert_eq!(
            clean_text(input, &[], true, Platform::None),
            "# Easy AI - Retail\n## Định vị sản phẩm"
        );
    }

    #[test]
    fn test_heading_strips_bold_discord() {
        let input = "# **Easy AI**\n\n## **Features**";
        assert_eq!(
            clean_text(input, &[], true, Platform::Discord),
            "**Easy AI**\n**Features**"
        );
    }

    #[test]
    fn test_backslash_escape_stripped() {
        let input = "Easy AI \\- Retail \\& E\\-commerce";
        assert_eq!(
            clean_text(input, &[], true, Platform::None),
            "Easy AI - Retail & E-commerce"
        );
    }

    #[test]
    fn test_google_docs_heading_with_bullets() {
        let input =
            "# **Heading**\n\n## **Sub**\n\n* Item 1\n* Item 2\n\n### **Deep**\n\n* Sub item";
        assert_eq!(
            clean_text(input, &[], true, Platform::None),
            "# Heading\n## Sub\n- Item 1\n- Item 2\n\n### Deep\n- Sub item"
        );
    }

    #[test]
    fn test_linkedin_hashtag_fix() {
        let input = "hashtag#buildinpublic hashtag#tuhocproduct";
        assert_eq!(
            clean_text(input, &[], true, Platform::None),
            "#buildinpublic #tuhocproduct"
        );
    }

    #[test]
    fn test_linkedin_hashtag_case_insensitive() {
        let input = "Hashtag#AI HASHTAG#startup";
        assert_eq!(clean_text(input, &[], true, Platform::None), "#AI #startup");
    }

    #[test]
    fn test_regular_hash_not_affected() {
        let input = "Use #channel for discussion";
        assert_eq!(
            clean_text(input, &[], true, Platform::None),
            "Use #channel for discussion"
        );
    }
}
