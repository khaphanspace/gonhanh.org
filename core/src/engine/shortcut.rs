//! Shortcut Table - Abbreviation expansion
//!
//! Allows users to define shortcuts like "vn" → "Việt Nam"

use std::collections::HashMap;

/// Trigger condition for shortcut
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TriggerCondition {
    /// Trigger immediately when buffer matches
    Immediate,
    /// Trigger when word boundary (space, punctuation) is pressed
    OnWordBoundary,
}

/// Case handling mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CaseMode {
    /// Keep replacement exactly as defined
    Exact,
    /// Match case of trigger: "VN" → "VIỆT NAM", "vn" → "Việt Nam"
    MatchCase,
}

/// A single shortcut entry
#[derive(Debug, Clone)]
pub struct Shortcut {
    /// Trigger string (lowercase for matching)
    pub trigger: String,
    /// Replacement text
    pub replacement: String,
    /// When to trigger
    pub condition: TriggerCondition,
    /// How to handle case
    pub case_mode: CaseMode,
    /// Whether this shortcut is enabled
    pub enabled: bool,
}

impl Shortcut {
    pub fn new(trigger: &str, replacement: &str) -> Self {
        Self {
            trigger: trigger.to_lowercase(),
            replacement: replacement.to_string(),
            condition: TriggerCondition::OnWordBoundary,
            case_mode: CaseMode::MatchCase,
            enabled: true,
        }
    }

    pub fn immediate(trigger: &str, replacement: &str) -> Self {
        Self {
            trigger: trigger.to_lowercase(),
            replacement: replacement.to_string(),
            condition: TriggerCondition::Immediate,
            case_mode: CaseMode::Exact,
            enabled: true,
        }
    }
}

/// Shortcut match result
#[derive(Debug)]
pub struct ShortcutMatch {
    /// Number of characters to backspace
    pub backspace_count: usize,
    /// Replacement text to output
    pub output: String,
    /// Whether to include the trigger key in output
    pub include_trigger_key: bool,
}

/// Shortcut table manager
#[derive(Debug, Default)]
pub struct ShortcutTable {
    /// Shortcuts indexed by trigger (lowercase)
    shortcuts: HashMap<String, Shortcut>,
    /// Sorted triggers by length (longest first) for matching
    sorted_triggers: Vec<String>,
}

impl ShortcutTable {
    pub fn new() -> Self {
        Self {
            shortcuts: HashMap::new(),
            sorted_triggers: vec![],
        }
    }

    /// Create with default Vietnamese shortcuts
    pub fn with_defaults() -> Self {
        let mut table = Self::new();

        // Common abbreviations
        table.add(Shortcut::new("vn", "Việt Nam"));
        table.add(Shortcut::new("hcm", "Hồ Chí Minh"));
        table.add(Shortcut::new("hn", "Hà Nội"));
        table.add(Shortcut::new("dc", "được"));
        table.add(Shortcut::new("ko", "không"));

        table
    }

    /// Add a shortcut
    pub fn add(&mut self, shortcut: Shortcut) {
        let trigger = shortcut.trigger.clone();
        self.shortcuts.insert(trigger.clone(), shortcut);
        self.rebuild_sorted_triggers();
    }

    /// Remove a shortcut
    pub fn remove(&mut self, trigger: &str) -> Option<Shortcut> {
        let result = self.shortcuts.remove(&trigger.to_lowercase());
        if result.is_some() {
            self.rebuild_sorted_triggers();
        }
        result
    }

    /// Check if buffer matches any shortcut
    ///
    /// Returns (trigger, shortcut) if match found
    pub fn lookup(&self, buffer: &str) -> Option<(&str, &Shortcut)> {
        let buffer_lower = buffer.to_lowercase();

        // Longest-match-first
        for trigger in &self.sorted_triggers {
            if buffer_lower == *trigger {
                if let Some(shortcut) = self.shortcuts.get(trigger) {
                    if shortcut.enabled {
                        return Some((trigger, shortcut));
                    }
                }
            }
        }
        None
    }

    /// Try to match buffer with trigger key
    ///
    /// # Arguments
    /// * `buffer` - Current buffer content (as string)
    /// * `key_char` - The key that was just pressed
    /// * `is_word_boundary` - Whether key_char is a word boundary
    ///
    /// # Returns
    /// ShortcutMatch if a shortcut should be triggered
    pub fn try_match(
        &self,
        buffer: &str,
        key_char: Option<char>,
        is_word_boundary: bool,
    ) -> Option<ShortcutMatch> {
        let (trigger, shortcut) = self.lookup(buffer)?;

        match shortcut.condition {
            TriggerCondition::Immediate => {
                let output = self.apply_case(buffer, &shortcut.replacement, shortcut.case_mode);
                Some(ShortcutMatch {
                    backspace_count: trigger.len(),
                    output,
                    include_trigger_key: false,
                })
            }
            TriggerCondition::OnWordBoundary => {
                if is_word_boundary {
                    let mut output =
                        self.apply_case(buffer, &shortcut.replacement, shortcut.case_mode);
                    // Append the trigger key (space, etc.)
                    if let Some(ch) = key_char {
                        output.push(ch);
                    }
                    Some(ShortcutMatch {
                        backspace_count: trigger.len(),
                        output,
                        include_trigger_key: true,
                    })
                } else {
                    None
                }
            }
        }
    }

    /// Apply case transformation based on mode
    fn apply_case(&self, trigger: &str, replacement: &str, mode: CaseMode) -> String {
        match mode {
            CaseMode::Exact => replacement.to_string(),
            CaseMode::MatchCase => {
                if trigger.chars().all(|c| c.is_uppercase()) {
                    // All uppercase → replacement all uppercase
                    replacement.to_uppercase()
                } else if trigger
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false)
                {
                    // First char uppercase → capitalize replacement
                    let mut chars = replacement.chars();
                    match chars.next() {
                        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                        None => String::new(),
                    }
                } else {
                    // Lowercase → keep replacement as-is
                    replacement.to_string()
                }
            }
        }
    }

    /// Rebuild sorted triggers list (longest first)
    fn rebuild_sorted_triggers(&mut self) {
        self.sorted_triggers = self.shortcuts.keys().cloned().collect();
        self.sorted_triggers
            .sort_by_key(|s| std::cmp::Reverse(s.len()));
    }

    /// Check if shortcut table is empty
    pub fn is_empty(&self) -> bool {
        self.shortcuts.is_empty()
    }

    /// Get number of shortcuts
    pub fn len(&self) -> usize {
        self.shortcuts.len()
    }

    /// Clear all shortcuts
    pub fn clear(&mut self) {
        self.shortcuts.clear();
        self.sorted_triggers.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_shortcut() {
        let mut table = ShortcutTable::new();
        table.add(Shortcut::new("vn", "Việt Nam"));

        let result = table.try_match("vn", Some(' '), true);
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.backspace_count, 2);
        assert_eq!(m.output, "Việt Nam ");
    }

    #[test]
    fn test_case_matching() {
        let mut table = ShortcutTable::new();
        table.add(Shortcut::new("vn", "Việt Nam"));

        // Lowercase
        let m = table.try_match("vn", Some(' '), true).unwrap();
        assert_eq!(m.output, "Việt Nam ");

        // Uppercase
        let m = table.try_match("VN", Some(' '), true).unwrap();
        assert_eq!(m.output, "VIỆT NAM ");

        // Title case
        let m = table.try_match("Vn", Some(' '), true).unwrap();
        assert_eq!(m.output, "Việt Nam ");
    }

    #[test]
    fn test_immediate_shortcut() {
        let mut table = ShortcutTable::new();
        table.add(Shortcut::immediate("w", "ư"));

        // Immediate triggers without word boundary
        let result = table.try_match("w", None, false);
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.output, "ư");
        assert!(!m.include_trigger_key);
    }

    #[test]
    fn test_word_boundary_required() {
        let mut table = ShortcutTable::new();
        table.add(Shortcut::new("vn", "Việt Nam"));

        // Without word boundary - should not match
        let result = table.try_match("vn", Some('a'), false);
        assert!(result.is_none());

        // With word boundary - should match
        let result = table.try_match("vn", Some(' '), true);
        assert!(result.is_some());
    }

    #[test]
    fn test_longest_match() {
        let mut table = ShortcutTable::new();
        table.add(Shortcut::new("h", "họ"));
        table.add(Shortcut::new("hcm", "Hồ Chí Minh"));

        // "hcm" should match the longer shortcut
        let (trigger, _) = table.lookup("hcm").unwrap();
        assert_eq!(trigger, "hcm");
    }

    #[test]
    fn test_disabled_shortcut() {
        let mut table = ShortcutTable::new();
        let mut shortcut = Shortcut::new("vn", "Việt Nam");
        shortcut.enabled = false;
        table.add(shortcut);

        let result = table.lookup("vn");
        assert!(result.is_none());
    }
}
