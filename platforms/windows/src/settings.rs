//! Settings in HKCU\SOFTWARE\GoNhanh + %APPDATA%\GoNhanh\shortcuts.txt (same as the earlier Windows apps).
//! Defaults match macOS registerDefaultSettings.

use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::hotkey::Hotkey;
use crate::log;
use crate::win::registry as reg;

const KEY: &str = r"SOFTWARE\GoNhanh";
const PER_APP_MODES_KEY: &str = r"SOFTWARE\GoNhanh\PerAppModes";
const PER_APP_PROFILES_KEY: &str = r"SOFTWARE\GoNhanh\PerAppProfiles";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method {
    Telex = 0,
    Vni = 1,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShortcutItem {
    pub key: String,
    pub value: String,
    pub enabled: bool,
}

impl ShortcutItem {
    pub fn new(key: &str, value: &str, enabled: bool) -> Self {
        ShortcutItem { key: key.into(), value: value.into(), enabled }
    }
}

#[derive(Clone, Debug)]
pub struct Settings {
    pub method: Method,
    pub modern_tone: bool,
    pub enabled: bool,
    pub first_run: bool,
    pub auto_start: bool,

    pub auto_w_shortcut: bool,
    pub bracket_shortcut: bool,
    pub english_auto_restore: bool,
    pub allow_foreign_consonants: bool,
    pub auto_capitalize: bool,
    pub auto_capitalize_excluded_apps: String, // "code.exe, cmd.exe"

    pub toggle_shortcut: Hotkey,
    pub secondary_toggle_enabled: bool,
    pub secondary_toggle_shortcut: Hotkey,
    pub restore_shortcut_enabled: bool,
    pub restore_shortcut: Hotkey,

    pub per_app_mode: bool,
    pub sound_enabled: bool,
    pub remote_desktop_mode: bool,
    pub debug_log: bool,

    /// Remembered on/off per app; keys are lowercase process names
    pub per_app_modes: BTreeMap<String, bool>,
    /// Per-app override: 0 auto, 1 always on, -1 always off; keys are lowercase process names
    pub per_app_profiles: BTreeMap<String, i32>,
    pub shortcuts: Vec<ShortcutItem>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            method: Method::Telex,
            modern_tone: true,
            enabled: true,
            first_run: true,
            auto_start: false,
            auto_w_shortcut: true,
            bracket_shortcut: false,
            english_auto_restore: true,
            allow_foreign_consonants: false,
            auto_capitalize: false,
            auto_capitalize_excluded_apps: String::new(),
            toggle_shortcut: Hotkey::DEFAULT_TOGGLE,
            secondary_toggle_enabled: false,
            secondary_toggle_shortcut: Hotkey::DEFAULT_SECONDARY_TOGGLE,
            restore_shortcut_enabled: false,
            restore_shortcut: Hotkey::DEFAULT_RESTORE,
            per_app_mode: true,
            sound_enabled: false,
            remote_desktop_mode: false,
            debug_log: false,
            per_app_modes: BTreeMap::new(),
            per_app_profiles: BTreeMap::new(),
            shortcuts: default_shortcuts(),
        }
    }
}

pub fn default_shortcuts() -> Vec<ShortcutItem> {
    vec![
        ShortcutItem::new("vn", "Việt Nam", false),
        ShortcutItem::new("hn", "Hà Nội", false),
        ShortcutItem::new("hcm", "Hồ Chí Minh", false),
        ShortcutItem::new("tphcm", "Thành phố Hồ Chí Minh", false),
    ]
}

pub fn data_dir() -> PathBuf {
    PathBuf::from(std::env::var_os("APPDATA").unwrap_or_default()).join("GoNhanh")
}

fn shortcuts_file() -> PathBuf {
    data_dir().join("shortcuts.txt")
}

impl Settings {
    pub fn load() -> Self {
        let d = Settings::default();
        let b = |name: &str, def: bool| reg::get_dword(KEY, name).map_or(def, |v| v == 1);
        let hk = |name: &str, def: Hotkey| reg::get_dword(KEY, name).map_or(def, Hotkey::from_int);
        let mut s = Settings {
            method: if reg::get_dword(KEY, "InputMethod") == Some(1) { Method::Vni } else { Method::Telex },
            modern_tone: b("ModernTone", d.modern_tone),
            enabled: b("Enabled", d.enabled),
            first_run: b("FirstRun", d.first_run),
            auto_start: b("AutoStart", d.auto_start),
            auto_w_shortcut: b("AutoWShortcut", d.auto_w_shortcut),
            bracket_shortcut: b("BracketShortcut", d.bracket_shortcut),
            english_auto_restore: b("EnglishAutoRestore", d.english_auto_restore),
            allow_foreign_consonants: b("AllowForeignConsonants", d.allow_foreign_consonants),
            auto_capitalize: b("AutoCapitalize", d.auto_capitalize),
            auto_capitalize_excluded_apps: reg::get_string(KEY, "AutoCapitalizeExcludedApps").unwrap_or_default(),
            toggle_shortcut: hk("ToggleShortcut", d.toggle_shortcut),
            secondary_toggle_enabled: b("SecondaryToggleEnabled", d.secondary_toggle_enabled),
            secondary_toggle_shortcut: hk("SecondaryToggleShortcut", d.secondary_toggle_shortcut),
            restore_shortcut_enabled: b("RestoreShortcutEnabled", d.restore_shortcut_enabled),
            restore_shortcut: hk("RestoreShortcut", d.restore_shortcut),
            per_app_mode: b("PerAppMode", d.per_app_mode),
            sound_enabled: b("SoundEnabled", d.sound_enabled),
            remote_desktop_mode: b("RemoteDesktopMode", d.remote_desktop_mode),
            debug_log: b("DebugLog", d.debug_log),
            ..d
        };
        for (name, v) in reg::dword_values(PER_APP_MODES_KEY) {
            s.per_app_modes.insert(name.to_lowercase(), v == 1);
        }
        for (name, v) in reg::dword_values(PER_APP_PROFILES_KEY) {
            s.per_app_profiles.insert(name.to_lowercase(), v as i32);
        }
        if let Ok(text) = std::fs::read_to_string(shortcuts_file()) {
            s.shortcuts = parse_shortcuts_file(&text);
        }
        s
    }

    /// Everything except per_app_modes (saved one by one) and shortcuts (own file)
    pub fn save(&self) {
        let ok = reg::with_key(KEY, |k| {
            let b = |name: &str, v: bool| k.set_dword(name, v as u32);
            k.set_dword("InputMethod", self.method as u32);
            b("ModernTone", self.modern_tone);
            b("Enabled", self.enabled);
            b("FirstRun", self.first_run);
            b("AutoStart", self.auto_start);
            b("AutoWShortcut", self.auto_w_shortcut);
            b("BracketShortcut", self.bracket_shortcut);
            b("EnglishAutoRestore", self.english_auto_restore);
            b("AllowForeignConsonants", self.allow_foreign_consonants);
            b("AutoCapitalize", self.auto_capitalize);
            k.set_string("AutoCapitalizeExcludedApps", &self.auto_capitalize_excluded_apps);
            k.set_dword("ToggleShortcut", self.toggle_shortcut.to_int());
            b("SecondaryToggleEnabled", self.secondary_toggle_enabled);
            k.set_dword("SecondaryToggleShortcut", self.secondary_toggle_shortcut.to_int());
            b("RestoreShortcutEnabled", self.restore_shortcut_enabled);
            k.set_dword("RestoreShortcut", self.restore_shortcut.to_int());
            b("PerAppMode", self.per_app_mode);
            b("SoundEnabled", self.sound_enabled);
            b("RemoteDesktopMode", self.remote_desktop_mode);
            b("DebugLog", self.debug_log);
        });
        if !ok {
            log::write("Failed to open settings key");
            return;
        }
        reg::delete_tree(PER_APP_PROFILES_KEY);
        reg::with_key(PER_APP_PROFILES_KEY, |k| {
            for (app, state) in &self.per_app_profiles {
                k.set_dword(app, *state as u32);
            }
        });
        self.update_auto_start();
    }

    pub fn save_per_app_mode(&mut self, app: &str, on: bool) {
        let app = app.to_lowercase();
        reg::with_key(PER_APP_MODES_KEY, |k| k.set_dword(&app, on as u32));
        self.per_app_modes.insert(app, on);
    }

    pub fn save_shortcuts(&self) {
        let text: String = self
            .shortcuts
            .iter()
            .map(|s| format!("{}\t{}\t{}\n", if s.enabled { 1 } else { 0 }, escape(&s.key), escape(&s.value)))
            .collect();
        let _ = std::fs::create_dir_all(data_dir());
        if std::fs::write(shortcuts_file(), text).is_err() {
            log::write("Failed to save shortcuts");
        }
    }

    fn update_auto_start(&self) {
        const RUN: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run";
        if self.auto_start {
            let exe = std::env::current_exe().unwrap_or_default();
            reg::with_key(RUN, |k| k.set_string("GoNhanh", &format!("\"{}\"", exe.display())));
        } else {
            reg::delete_value(RUN, "GoNhanh");
        }
    }

    pub fn is_auto_capitalize_excluded(&self, app: &str) -> bool {
        self.auto_capitalize_excluded_apps
            .split(',')
            .filter(|a| !a.trim().is_empty())
            .any(|a| normalize_app(a).eq_ignore_ascii_case(app))
    }
}

/// " Code " → "Code.exe"
pub fn normalize_app(name: &str) -> String {
    let name = name.trim();
    if name.to_lowercase().ends_with(".exe") {
        name.to_string()
    } else {
        format!("{name}.exe")
    }
}

/// Backslash, newline, CR and tab escaped (Issue #343 format, plus tab for our file)
pub fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out
}

pub fn unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some('t') => out.push('\t'),
            Some('\\') => out.push('\\'),
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
}

/// "<0|1>\t<key>\t<value>" per line
fn parse_shortcuts_file(text: &str) -> Vec<ShortcutItem> {
    text.lines()
        .filter_map(|line| {
            let mut f = line.splitn(3, '\t');
            let (flag, key, value) = (f.next()?, f.next()?, f.next()?);
            Some(ShortcutItem { key: unescape(key), value: unescape(value), enabled: flag == "1" })
        })
        .collect()
}

/// macOS-compatible export: "trigger:value" per line
pub fn export_shortcuts(items: &[ShortcutItem]) -> String {
    let mut out = String::from(";Gõ Nhanh - Bảng gõ tắt");
    for s in items.iter().filter(|s| !s.key.is_empty()) {
        let value = s.value.replace('\\', "\\\\").replace('\n', "\\n").replace('\r', "\\r");
        out.push_str(&format!("\n{}:{}", s.key, value));
    }
    out
}

/// Merge "trigger:value" lines into `into` (existing triggers updated). Returns the imported count.
pub fn import_shortcuts(content: &str, into: &mut Vec<ShortcutItem>) -> usize {
    let mut imported = 0;
    for line in content.lines().map(str::trim) {
        if line.is_empty() || line.starts_with(';') {
            continue;
        }
        let Some((trigger, value)) = line.split_once(':') else { continue };
        let trigger = trigger.trim();
        if trigger.is_empty() {
            continue;
        }
        let value = unescape(value.trim());
        match into.iter_mut().find(|s| s.key == trigger) {
            Some(existing) => {
                existing.value = value;
                existing.enabled = true;
            }
            None => into.push(ShortcutItem::new(trigger, &value, true)),
        }
        imported += 1;
    }
    imported
}
