import AppKit

/// Plain AppKit entry point: the always-running input method must not load the SwiftUI app
/// lifecycle (it costs tens of MB). SwiftUI is only touched when a settings/about window opens.
@main
enum GoNhanhMain {
    static func main() {
        let app = NSApplication.shared
        let delegate = AppDelegate()
        app.delegate = delegate
        app.run()
    }
}

class AppDelegate: NSObject, NSApplicationDelegate {
    var menuBar: MenuBarController?
    func applicationDidFinishLaunching(_: Notification) {
        // Register default settings before anything else
        registerDefaultSettings()

        NSApp.setActivationPolicy(.accessory)
        // Resolve the current source before MenuBarController starts the engine, so
        // native Vietnamese/non-Latin sources are gated correctly on cold launch.
        InputSourceObserver.shared.start()
        menuBar = MenuBarController()
    }

    func applicationWillTerminate(_: Notification) {
        // Cancel any pending restart-on-close (window may have closed during quit)
        menuBar?.cancelPendingRestart()
        KeyboardHookManager.shared.stop()
        InputSourceObserver.shared.stop()
    }

    private func registerDefaultSettings() {
        UserDefaults.standard.register(defaults: [
            SettingsKey.enabled: true,
            SettingsKey.method: InputMode.telex.rawValue,
            SettingsKey.perAppMode: true,
            SettingsKey.autoWShortcut: true,
            SettingsKey.bracketShortcut: false,
            SettingsKey.restoreShortcutEnabled: false,
            SettingsKey.modernTone: true,
            SettingsKey.englishAutoRestore: true,
            SettingsKey.autoCapitalize: false,
            SettingsKey.soundEnabled: false,
            SettingsKey.allowForeignConsonants: false,
            SettingsKey.advancedMode: false,
            SettingsKey.restartOnClose: true,
            SettingsKey.sessionTapMode: false,
        ])
    }
}
