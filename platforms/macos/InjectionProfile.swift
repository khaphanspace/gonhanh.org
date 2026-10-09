import Foundation

// MARK: - Delays

/// Microseconds to wait after each synthetic backspace, after the last backspace, and after each
/// chunk of replacement text. Apps that drop fast synthetic events need more.
typealias InjectDelays = (backspace: UInt32, wait: UInt32, text: UInt32)

/// The delay levels: the single source for per-app detection and for the Advanced settings slider.
enum Delays {
    static let none: InjectDelays = (200, 800, 500)
    static let low: InjectDelays = (1000, 3000, 1500)
    static let medium: InjectDelays = (3000, 8000, 3000)
    static let high: InjectDelays = (8000, 25000, 8000)
    static let veryHigh: InjectDelays = (12000, 25000, 12000)
    /// No waiting: the method itself is synchronous or has its own timing.
    static let zero: InjectDelays = (0, 0, 0)
}

// MARK: - Method

enum InjectionMethod {
    case fast // Default: backspace + text with minimal delays
    case slow // Terminals/Electron: backspace + text with higher delays
    case charByChar // Safari Google Docs: backspace + text character-by-character
    case selection // Browser address bars: Shift+Left select + type replacement
    case axDirect // Spotlight primary: AX API direct text manipulation (macOS 13+)
    case emptyCharPrefix // Browser address bars: empty char to break autocomplete + extra backspace
    case syncProxy // Games: synchronous injection via CGEventTapPostEvent(proxy)
    case passthrough // iPhone Mirroring / remote desktop: pass every key through
}

// MARK: - Profile

/// How to replace text in one app: the method, the delays, and a short tag for the log.
struct InjectionProfile {
    let method: InjectionMethod
    let delays: InjectDelays
    let tag: String

    init(_ method: InjectionMethod, _ delays: InjectDelays, _ tag: String) {
        self.method = method
        self.delays = delays
        self.tag = tag
    }

    /// The profile with the user's per-app overrides applied (Advanced settings).
    func applying(_ config: PerAppConfig?) -> InjectionProfile {
        guard let config else { return self }
        let delays = DelayPreset(rawValue: config.delayPreset)?.delays ?? delays
        let method = InjectionOverride(rawValue: config.injectionOverride)?.method ?? method
        return InjectionProfile(method, delays, "override:\(tag)")
    }
}

// MARK: - Table

/// Which injection profile an app needs. A pure function of the app's bundle id and the role of
/// the focused element, so the rules can be tested without a running app. Rules are checked in
/// order and the first match wins; the tables are built once.
enum InjectionProfiles {
    /// Used when no app can be identified.
    static let unknownApp = InjectionProfile(.fast, Delays.none, "unknown")

    static func resolve(bundleId id: String, role: String?) -> InjectionProfile {
        // Keys that must reach the target untouched: the other side composes the text.
        if id == "com.apple.ScreenContinuity" {
            return .init(.passthrough, Delays.zero, "pass:iphone")
        }
        if remoteDesktop.contains(id) {
            return .init(.passthrough, Delays.zero, "pass:remote")
        }

        // Autocomplete UI elements take a selection, whatever app they are in.
        if role == "AXComboBox" {
            return .init(.selection, Delays.zero, "sel:combo")
        }
        if role == "AXSearchField" {
            return .init(.selection, Delays.zero, "sel:search")
        }

        // Spotlight: direct AX manipulation (macOS 13+)
        if SpecialPanelAppDetector.isSpotlight(id) || id == "com.apple.systemuiserver" {
            return .init(.axDirect, Delays.zero, "ax:spotlight")
        }

        // Safari: the address bar needs the empty-char prefix, content (Google Docs) needs char by char
        if safari.contains(id) {
            return role == "AXTextField"
                ? .init(.emptyCharPrefix, Delays.medium, "emptyChar:safari")
                : .init(.charByChar, Delays.medium, "char:safari")
        }
        // Other browsers: the prefix breaks autocomplete highlights in every context; medium delays
        // cover web apps that intercept popups (Telegram Web)
        if browsers.contains(id) {
            return .init(.emptyCharPrefix, Delays.medium, "emptyChar:browser")
        }

        let jetBrains = id.hasPrefix("com.jetbrains")
        if role == "AXTextField", jetBrains {
            return .init(.selection, Delays.zero, "sel:jb")
        }

        // Office: backspaces (a selection conflicts with autocomplete)
        if id == "com.microsoft.Excel" {
            return .init(.slow, Delays.medium, "slow:excel")
        }
        if id == "com.microsoft.Word" {
            return .init(.slow, Delays.medium, "slow:word")
        }
        if id == "com.microsoft.Outlook" {
            return .init(.slow, (8000, 15000, 8000), "slow:outlook")
        }

        // Electron apps
        if id == "com.todesktop.230313mzl4w4u92" {
            return .init(.slow, (8000, 15000, 8000), "slow:claude")
        }
        if id == "notion.id" {
            return .init(.slow, Delays.veryHigh, "slow:notion")
        }

        // Editors and terminals (Monaco, Electron, GPU terminals)
        if codeApps.contains(id) {
            return .init(.slow, Delays.high, "slow:code")
        }

        // Qt and other toolkits that need one character at a time
        if id == "texstudio" {
            return .init(.charByChar, Delays.medium, "char:texstudio")
        }
        if jetBrains {
            return .init(.slow, Delays.high, "slow:jb")
        }
        if id == "com.caudex.dev" {
            return .init(.charByChar, (5000, 15000, 5000), "char:caudex")
        }
        if id == "com.foxit-software.Foxit.PDF.Reader" {
            return .init(.charByChar, Delays.zero, "char:foxit")
        }
        // Adobe's text engine reads only the first character of a multi-character key event
        if id.hasPrefix("com.adobe.") {
            return .init(.charByChar, Delays.medium, "char:adobe")
        }

        // Games: synchronous proxy injection (Issue #264: Vietnamese typing in LOL)
        if id.hasPrefix("com.riotgames") {
            return .init(.syncProxy, Delays.zero, "sync:game")
        }

        return .init(.fast, Delays.low, "default")
    }

    // MARK: Data

    /// Remote desktop clients forward physical keystrokes to the other machine. Synthetic
    /// backspaces and Vietnamese characters are not forwarded, so the text would be garbled:
    /// composition has to happen on the remote machine.
    private static let remoteDesktop: Set<String> = [
        "com.carriez.rustdesk", "com.philandro.anydesk", "com.teamviewer.TeamViewer",
    ]

    private static let safari: Set<String> = [
        "com.apple.Safari", "com.apple.SafariTechnologyPreview",
    ]

    private static let browsers: Set<String> = [
        // The Browser Company
        "company.thebrowser.Browser", "company.thebrowser.Arc", "company.thebrowser.dia",
        // Firefox-based
        "org.mozilla.firefox", "org.mozilla.firefoxdeveloperedition", "org.mozilla.nightly",
        "org.waterfoxproject.waterfox", "io.gitlab.librewolf-community.librewolf",
        "one.ablaze.floorp", "org.torproject.torbrowser", "net.mullvad.mullvadbrowser",
        "app.zen-browser.zen",
        // Chromium-based
        "com.google.Chrome", "com.google.Chrome.canary", "com.google.Chrome.beta",
        "org.chromium.Chromium", "com.brave.Browser", "com.brave.Browser.beta",
        "com.brave.Browser.nightly", "com.microsoft.edgemac", "com.microsoft.edgemac.Beta",
        "com.microsoft.edgemac.Dev", "com.microsoft.edgemac.Canary", "com.vivaldi.Vivaldi",
        "com.vivaldi.Vivaldi.snapshot", "ru.yandex.desktop.yandex-browser", "net.imput.helium",
        // Opera
        "com.opera.Opera", "com.operasoftware.Opera", "com.operasoftware.OperaGX",
        "com.operasoftware.OperaAir", "com.opera.OperaNext",
        // WebKit-based and others
        "com.kagi.kagimacOS", "com.sigmaos.sigmaos.macos", "com.pushplaylabs.sidekick",
        "com.firstversionist.polypane", "ai.perplexity.comet", "com.duckduckgo.macos.browser",
        "com.openai.atlas",
    ]

    private static let codeApps: Set<String> = [
        // VSCode-based IDEs
        "com.microsoft.VSCode", "com.google.antigravity-ide", "com.todesktop.cursor",
        "com.visualstudio.code.oss", "com.vscodium",
        // Terminals
        "dev.warp.Warp-Stable", "com.mitchellh.ghostty", "net.kovidgoyal.kitty",
        "com.apple.Terminal", "com.googlecode.iterm2", "io.alacritty",
        "com.github.wez.wezterm", "co.zeit.hyper", "org.tabby",
        "com.raphaelamorim.rio", "com.termius-dmg.mac",
        // Other code editors
        "dev.zed.Zed", "com.sublimetext.4", "com.sublimetext.3", "com.panic.Nova",
    ]
}
