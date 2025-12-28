//! Shortcut Table - Abbreviation expansion
//!
//! Allows users to define shortcuts like "vn" → "Việt Nam"
//! Shortcuts can be specific to input methods (Telex/VNI) or apply to all.

use super::buffer::MAX;
use std::collections::HashMap;

/// Maximum replacement length in UTF-32 codepoints (matches Result.chars array size)
/// This limit ensures replacement fits in the FFI result buffer.
/// Note: Vietnamese characters with diacritics (ồ, ế, ẫ) count as 1 codepoint each.
pub const MAX_REPLACEMENT_LEN: usize = MAX - 1; // -1 to leave room for trailing space

/// Input method that shortcut applies to
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum InputMethod {
    /// Apply to all input methods
    #[default]
    All,
    /// Apply only to Telex
    Telex,
    /// Apply only to VNI
    Vni,
}

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
    /// Which input method this shortcut applies to
    pub input_method: InputMethod,
}

impl Shortcut {
    /// Validate and truncate replacement if it exceeds MAX_REPLACEMENT_LEN.
    /// Counts UTF-32 codepoints (Vietnamese diacritics = 1 codepoint each).
    fn validate_replacement(replacement: &str) -> String {
        let char_count = replacement.chars().count();
        if char_count <= MAX_REPLACEMENT_LEN {
            replacement.to_string()
        } else {
            // Truncate to MAX_REPLACEMENT_LEN codepoints
            replacement.chars().take(MAX_REPLACEMENT_LEN).collect()
        }
    }

    /// Create a new shortcut with word boundary trigger (applies to all input methods)
    /// Issue #86: Case-insensitive matching, smart case output (ko→không, KO→KHÔNG, Ko→Không)
    /// Replacement is truncated to MAX_REPLACEMENT_LEN (63) codepoints if too long.
    pub fn new(trigger: &str, replacement: &str) -> Self {
        Self {
            trigger: trigger.to_lowercase(), // Store lowercase for case-insensitive matching
            replacement: Self::validate_replacement(replacement),
            condition: TriggerCondition::OnWordBoundary,
            case_mode: CaseMode::MatchCase, // Smart case transformation
            enabled: true,
            input_method: InputMethod::All,
        }
    }

    /// Create an immediate trigger shortcut (applies to all input methods).
    /// Issue #86: Case-insensitive matching, smart case output
    /// Replacement is truncated to MAX_REPLACEMENT_LEN (63) codepoints if too long.
    pub fn immediate(trigger: &str, replacement: &str) -> Self {
        Self {
            trigger: trigger.to_lowercase(), // Store lowercase for case-insensitive matching
            replacement: Self::validate_replacement(replacement),
            condition: TriggerCondition::Immediate,
            case_mode: CaseMode::MatchCase, // Smart case transformation
            enabled: true,
            input_method: InputMethod::All,
        }
    }

    /// Create a Telex-specific shortcut with immediate trigger.
    /// Issue #86: Case-insensitive matching, smart case output
    /// Replacement is truncated to MAX_REPLACEMENT_LEN (63) codepoints if too long.
    pub fn telex(trigger: &str, replacement: &str) -> Self {
        Self {
            trigger: trigger.to_lowercase(), // Store lowercase for case-insensitive matching
            replacement: Self::validate_replacement(replacement),
            condition: TriggerCondition::Immediate,
            case_mode: CaseMode::MatchCase, // Smart case transformation
            enabled: true,
            input_method: InputMethod::Telex,
        }
    }

    /// Create a VNI-specific shortcut with immediate trigger.
    /// Issue #86: Case-insensitive matching, smart case output
    /// Replacement is truncated to MAX_REPLACEMENT_LEN (63) codepoints if too long.
    pub fn vni(trigger: &str, replacement: &str) -> Self {
        Self {
            trigger: trigger.to_lowercase(), // Store lowercase for case-insensitive matching
            replacement: Self::validate_replacement(replacement),
            condition: TriggerCondition::Immediate,
            case_mode: CaseMode::MatchCase, // Smart case transformation
            enabled: true,
            input_method: InputMethod::Vni,
        }
    }

    /// Set the input method for this shortcut
    pub fn for_method(mut self, method: InputMethod) -> Self {
        self.input_method = method;
        self
    }

    /// Check if shortcut applies to given input method
    ///
    /// - If shortcut is for `All`: matches any method
    /// - If shortcut is for `Telex`: matches `Telex` or `All` query
    /// - If shortcut is for `Vni`: matches `Vni` or `All` query
    pub fn applies_to(&self, query_method: InputMethod) -> bool {
        match self.input_method {
            // Shortcut for All → matches any query
            InputMethod::All => true,
            // Shortcut for specific method → matches if query is same method OR query is All
            InputMethod::Telex => {
                query_method == InputMethod::Telex || query_method == InputMethod::All
            }
            InputMethod::Vni => {
                query_method == InputMethod::Vni || query_method == InputMethod::All
            }
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

    /// Create with default Vietnamese shortcuts (common abbreviations)
    ///
    /// Note: "w" → "ư" is NOT a shortcut, it's handled by the engine
    /// as a vowel key with Vietnamese validation.
    ///
    /// Currently disabled - returns empty table
    pub fn with_defaults() -> Self {
        // Temporarily disabled default shortcuts
        Self::new()

        // Original defaults (uncomment to re-enable):
        // let mut table = Self::new();
        // table.add(Shortcut::new("vn", "Việt Nam"));
        // table.add(Shortcut::new("hcm", "Hồ Chí Minh"));
        // table.add(Shortcut::new("hn", "Hà Nội"));
        // table.add(Shortcut::new("dc", "được"));
        // table.add(Shortcut::new("ko", "không"));
        // table
    }

    /// Create with Telex defaults only
    pub fn with_telex_defaults() -> Self {
        // No Telex-specific shortcuts
        // "w" → "ư" is handled by the engine, not shortcuts
        Self::new()
    }

    /// Create with VNI defaults only
    pub fn with_vni_defaults() -> Self {
        Self::new()
    }

    /// Create with all defaults (common abbreviations)
    pub fn with_all_defaults() -> Self {
        let mut table = Self::new();

        // Common abbreviations (apply to all input methods)
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

    /// Remove a shortcut (exact match, case-sensitive)
    pub fn remove(&mut self, trigger: &str) -> Option<Shortcut> {
        let result = self.shortcuts.remove(trigger);
        if result.is_some() {
            self.rebuild_sorted_triggers();
        }
        result
    }

    /// Check if buffer matches any shortcut (for any input method)
    ///
    /// Returns (trigger, shortcut) if match found
    pub fn lookup(&self, buffer: &str) -> Option<(&str, &Shortcut)> {
        self.lookup_for_method(buffer, InputMethod::All)
    }

    /// Check if buffer matches any shortcut for specific input method
    ///
    /// Issue #86: Case-insensitive matching - "ko", "Ko", "KO" all match trigger "ko"
    /// Returns (original_buffer, shortcut) if match found
    pub fn lookup_for_method(
        &self,
        buffer: &str,
        method: InputMethod,
    ) -> Option<(&str, &Shortcut)> {
        let buffer_lower = buffer.to_lowercase();
        // Longest-match-first, case-insensitive match
        for trigger in &self.sorted_triggers {
            if buffer_lower == *trigger {
                if let Some(shortcut) = self.shortcuts.get(trigger) {
                    if shortcut.enabled && shortcut.applies_to(method) {
                        return Some((trigger, shortcut));
                    }
                }
            }
        }
        None
    }

    /// Try to match buffer with trigger key (for any input method)
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
        self.try_match_for_method(buffer, key_char, is_word_boundary, InputMethod::All)
    }

    /// Try to match buffer with trigger key for specific input method
    ///
    /// # Arguments
    /// * `buffer` - Current buffer content (as string)
    /// * `key_char` - The key that was just pressed
    /// * `is_word_boundary` - Whether key_char is a word boundary
    /// * `method` - The current input method (Telex/VNI)
    ///
    /// # Returns
    /// ShortcutMatch if a shortcut should be triggered
    pub fn try_match_for_method(
        &self,
        buffer: &str,
        key_char: Option<char>,
        is_word_boundary: bool,
        method: InputMethod,
    ) -> Option<ShortcutMatch> {
        let (trigger, shortcut) = self.lookup_for_method(buffer, method)?;

        match shortcut.condition {
            TriggerCondition::Immediate => {
                let output = self.apply_case(buffer, &shortcut.replacement, shortcut.case_mode);
                Some(ShortcutMatch {
                    // Use char count, not byte length (UTF-8 chars like đ are multi-byte)
                    backspace_count: trigger.chars().count(),
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
                        // Use char count, not byte length (UTF-8 chars like đ are multi-byte)
                        backspace_count: trigger.chars().count(),
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

    // Helper: Create table with one word-boundary shortcut
    fn table_with_shortcut(trigger: &str, replacement: &str) -> ShortcutTable {
        let mut table = ShortcutTable::new();
        table.add(Shortcut::new(trigger, replacement));
        table
    }

    // Helper: Create table with one immediate shortcut
    fn table_with_immediate(trigger: &str, replacement: &str) -> ShortcutTable {
        let mut table = ShortcutTable::new();
        table.add(Shortcut::immediate(trigger, replacement));
        table
    }

    // Helper: Create table with Telex-specific shortcut
    fn table_with_telex_shortcut(trigger: &str, replacement: &str) -> ShortcutTable {
        let mut table = ShortcutTable::new();
        table.add(Shortcut::telex(trigger, replacement));
        table
    }

    // Helper: Create table with VNI-specific shortcut
    fn table_with_vni_shortcut(trigger: &str, replacement: &str) -> ShortcutTable {
        let mut table = ShortcutTable::new();
        table.add(Shortcut::vni(trigger, replacement));
        table
    }

    // Helper: Assert shortcut matches and check output/backspace
    fn assert_shortcut_match(
        table: &ShortcutTable,
        buffer: &str,
        key_char: Option<char>,
        is_boundary: bool,
        expected_output: &str,
        expected_backspace: usize,
        method: InputMethod,
    ) {
        let result = table.try_match_for_method(buffer, key_char, is_boundary, method);
        assert!(
            result.is_some(),
            "Shortcut should match for buffer: {}",
            buffer
        );
        let m = result.unwrap();
        assert_eq!(m.output, expected_output);
        assert_eq!(m.backspace_count, expected_backspace);
    }

    // Helper: Assert no shortcut match
    fn assert_no_match(
        table: &ShortcutTable,
        buffer: &str,
        key_char: Option<char>,
        is_boundary: bool,
        method: InputMethod,
    ) {
        let result = table.try_match_for_method(buffer, key_char, is_boundary, method);
        assert!(
            result.is_none(),
            "Shortcut should NOT match for buffer: {}",
            buffer
        );
    }

    #[test]
    fn test_basic_shortcut() {
        let table = table_with_shortcut("vn", "Việt Nam");
        assert_shortcut_match(
            &table,
            "vn",
            Some(' '),
            true,
            "Việt Nam ",
            2,
            InputMethod::All,
        );
    }

    #[test]
    fn test_case_matching() {
        let table = table_with_shortcut("vn", "Việt Nam");

        // Lowercase "vn" → "Việt Nam" (as-is)
        assert_shortcut_match(
            &table,
            "vn",
            Some(' '),
            true,
            "Việt Nam ",
            2,
            InputMethod::All,
        );

        // Issue #86: Uppercase "VN" → "VIỆT NAM" (smart case)
        assert_shortcut_match(
            &table,
            "VN",
            Some(' '),
            true,
            "VIỆT NAM ",
            2,
            InputMethod::All,
        );

        // Issue #86: Title case "Vn" → "Việt Nam" (smart case)
        assert_shortcut_match(
            &table,
            "Vn",
            Some(' '),
            true,
            "Việt Nam ",
            2,
            InputMethod::All,
        );
    }

    #[test]
    fn test_immediate_shortcut() {
        let table = table_with_immediate("w", "ư");

        // Immediate triggers without word boundary
        let result = table.try_match("w", None, false);
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.output, "ư");
        assert!(!m.include_trigger_key);
    }

    #[test]
    fn test_word_boundary_required() {
        let table = table_with_shortcut("vn", "Việt Nam");

        // Without word boundary - should not match
        assert_no_match(&table, "vn", Some('a'), false, InputMethod::All);

        // With word boundary - should match
        assert_shortcut_match(
            &table,
            "vn",
            Some(' '),
            true,
            "Việt Nam ",
            2,
            InputMethod::All,
        );
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

    #[test]
    fn test_telex_specific_shortcut() {
        let table = table_with_telex_shortcut("w", "ư");

        // Should match for Telex
        assert_shortcut_match(&table, "w", None, false, "ư", 1, InputMethod::Telex);

        // Should NOT match for VNI
        assert_no_match(&table, "w", None, false, InputMethod::Vni);

        // Should match for All (fallback)
        assert_shortcut_match(&table, "w", None, false, "ư", 1, InputMethod::All);
    }

    #[test]
    fn test_vni_specific_shortcut() {
        let table = table_with_vni_shortcut("7", "ơ");

        // Should match for VNI
        assert_shortcut_match(&table, "7", None, false, "ơ", 1, InputMethod::Vni);

        // Should NOT match for Telex
        assert_no_match(&table, "7", None, false, InputMethod::Telex);
    }

    #[test]
    fn test_all_input_method_shortcut() {
        let table = table_with_shortcut("vn", "Việt Nam");

        // Should match for Telex
        assert_shortcut_match(
            &table,
            "vn",
            Some(' '),
            true,
            "Việt Nam ",
            2,
            InputMethod::Telex,
        );

        // Should match for VNI
        assert_shortcut_match(
            &table,
            "vn",
            Some(' '),
            true,
            "Việt Nam ",
            2,
            InputMethod::Vni,
        );

        // Should match for All
        assert_shortcut_match(
            &table,
            "vn",
            Some(' '),
            true,
            "Việt Nam ",
            2,
            InputMethod::All,
        );
    }

    #[test]
    fn test_with_defaults_has_common_shortcuts() {
        let table = ShortcutTable::with_defaults();

        // Default shortcuts are currently disabled - table should be empty
        assert!(table.is_empty());

        // "vn" → "Việt Nam" should NOT exist (disabled)
        let result = table.lookup_for_method("vn", InputMethod::All);
        assert!(result.is_none());

        // "w" is NOT a shortcut (handled by engine)
        let result = table.lookup_for_method("w", InputMethod::Telex);
        assert!(result.is_none());
    }

    #[test]
    fn test_shortcut_for_method_builder() {
        let shortcut = Shortcut::new("test", "Test").for_method(InputMethod::Telex);
        assert_eq!(shortcut.input_method, InputMethod::Telex);

        let shortcut = Shortcut::immediate("x", "y").for_method(InputMethod::Vni);
        assert_eq!(shortcut.input_method, InputMethod::Vni);
    }

    #[test]
    fn test_applies_to() {
        let all_shortcut = Shortcut::new("vn", "Việt Nam");
        assert!(all_shortcut.applies_to(InputMethod::All));
        assert!(all_shortcut.applies_to(InputMethod::Telex));
        assert!(all_shortcut.applies_to(InputMethod::Vni));

        let telex_shortcut = Shortcut::telex("test", "Test");
        assert!(telex_shortcut.applies_to(InputMethod::All));
        assert!(telex_shortcut.applies_to(InputMethod::Telex));
        assert!(!telex_shortcut.applies_to(InputMethod::Vni));

        let vni_shortcut = Shortcut::vni("7", "ơ");
        assert!(vni_shortcut.applies_to(InputMethod::All));
        assert!(!vni_shortcut.applies_to(InputMethod::Telex));
        assert!(vni_shortcut.applies_to(InputMethod::Vni));
    }

    #[test]
    fn test_replacement_validation_within_limit() {
        // Vietnamese text within limit (21 codepoints)
        let shortcut = Shortcut::new("tphcm", "Thành phố Hồ Chí Minh");
        assert_eq!(shortcut.replacement, "Thành phố Hồ Chí Minh");
        assert_eq!(shortcut.replacement.chars().count(), 21);
    }

    #[test]
    fn test_replacement_validation_truncation() {
        // Create a very long replacement (100 characters with Vietnamese)
        let long_text = "Đây là một đoạn văn bản rất dài để kiểm tra việc cắt ngắn. Nó có nhiều ký tự tiếng Việt có dấu như ồ, ế, ẫ, ơ, ư.";
        let char_count = long_text.chars().count();
        assert!(
            char_count > MAX_REPLACEMENT_LEN,
            "Test text should exceed limit"
        );

        let shortcut = Shortcut::new("long", long_text);
        let result_count = shortcut.replacement.chars().count();
        assert_eq!(
            result_count, MAX_REPLACEMENT_LEN,
            "Should truncate to MAX_REPLACEMENT_LEN"
        );
    }

    #[test]
    fn test_replacement_validation_vietnamese_diacritics() {
        // Each Vietnamese character with diacritic is 1 codepoint
        // "ồ" = 1 codepoint, "ế" = 1 codepoint, "ẫ" = 1 codepoint
        let vietnamese = "ồếẫơưáàảãạăắằẳẵặâấầẩẫậ"; // 22 Vietnamese chars
        let shortcut = Shortcut::new("viet", vietnamese);
        assert_eq!(shortcut.replacement.chars().count(), 22);
        assert_eq!(shortcut.replacement, vietnamese);
    }

    // =========================================================================
    // Issue #86: Smart Case-Aware Shortcuts
    // https://github.com/khaphanspace/gonhanh.org/issues/86
    // =========================================================================

    #[test]
    fn issue86_smart_case_lowercase() {
        let table = table_with_shortcut("ko", "không");
        assert_shortcut_match(&table, "ko", Some(' '), true, "không ", 2, InputMethod::All);
    }

    #[test]
    fn issue86_smart_case_uppercase() {
        let table = table_with_shortcut("ko", "không");
        assert_shortcut_match(&table, "KO", Some(' '), true, "KHÔNG ", 2, InputMethod::All);
    }

    #[test]
    fn issue86_smart_case_titlecase() {
        let table = table_with_shortcut("ko", "không");
        assert_shortcut_match(&table, "Ko", Some(' '), true, "Không ", 2, InputMethod::All);
    }

    #[test]
    fn issue86_smart_case_vn_lowercase() {
        let table = table_with_shortcut("vn", "Việt Nam");
        assert_shortcut_match(
            &table,
            "vn",
            Some(' '),
            true,
            "Việt Nam ",
            2,
            InputMethod::All,
        );
    }

    #[test]
    fn issue86_smart_case_vn_uppercase() {
        let table = table_with_shortcut("vn", "Việt Nam");
        assert_shortcut_match(
            &table,
            "VN",
            Some(' '),
            true,
            "VIỆT NAM ",
            2,
            InputMethod::All,
        );
    }

    #[test]
    fn issue86_smart_case_vn_titlecase() {
        let table = table_with_shortcut("vn", "Việt Nam");
        assert_shortcut_match(
            &table,
            "Vn",
            Some(' '),
            true,
            "Việt Nam ",
            2,
            InputMethod::All,
        );
    }

    #[test]
    fn issue86_smart_case_immediate_shortcut() {
        let table = table_with_immediate("dc", "được");

        let result = table.try_match("dc", None, false);
        assert!(result.is_some());
        assert_eq!(result.unwrap().output, "được");

        let result = table.try_match("DC", None, false);
        assert!(result.is_some());
        assert_eq!(result.unwrap().output, "ĐƯỢC");

        let result = table.try_match("Dc", None, false);
        assert!(result.is_some());
        assert_eq!(result.unwrap().output, "Được");
    }

    #[test]
    fn issue86_smart_case_lookup_case_insensitive() {
        let table = table_with_shortcut("ko", "không");
        assert!(table.lookup("ko").is_some());
        assert!(table.lookup("Ko").is_some());
        assert!(table.lookup("KO").is_some());
        assert!(table.lookup("kO").is_some());
    }

    #[test]
    fn issue86_smart_case_mixed_case_fallback() {
        let table = table_with_shortcut("ko", "không");
        assert_shortcut_match(&table, "kO", Some(' '), true, "không ", 2, InputMethod::All);
    }

    #[test]
    fn issue86_smart_case_hcm() {
        let table = table_with_shortcut("hcm", "Hồ Chí Minh");
        assert_shortcut_match(
            &table,
            "hcm",
            Some(' '),
            true,
            "Hồ Chí Minh ",
            3,
            InputMethod::All,
        );
        assert_shortcut_match(
            &table,
            "HCM",
            Some(' '),
            true,
            "HỒ CHÍ MINH ",
            3,
            InputMethod::All,
        );
        assert_shortcut_match(
            &table,
            "Hcm",
            Some(' '),
            true,
            "Hồ Chí Minh ",
            3,
            InputMethod::All,
        );
    }
}
