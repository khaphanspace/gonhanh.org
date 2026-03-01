import SwiftUI

@main
struct GoNhanhApp: App {
    @NSApplicationDelegateAdaptor(AppDelegate.self) var appDelegate

    var body: some Scene {
        Settings {
            EmptyView()
        }
    }
}

class AppDelegate: NSObject, NSApplicationDelegate {
    var menuBar: MenuBarController?
    var isQuitting = false

    func applicationDidFinishLaunching(_: Notification) {
        // Register default settings before anything else
        registerDefaultSettings()

        NSApp.setActivationPolicy(.accessory)
        menuBar = MenuBarController()

        // Start observing input source changes
        InputSourceObserver.shared.start()
    }

    func applicationShouldTerminate(_: NSApplication) -> NSApplication.TerminateReply {
        isQuitting = true
        return .terminateNow
    }

    func applicationWillTerminate(_: Notification) {
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
        ])
    }
}
