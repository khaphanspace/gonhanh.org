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

    func applicationDidFinishLaunching(_ notification: Notification) {
        // Register default settings before anything else
        registerDefaultSettings()

        NSApp.setActivationPolicy(.accessory)
        menuBar = MenuBarController()

        // Connect to XPC daemon (handles keyboard, input source, etc.)
        IMEClient.shared.connect()
    }

    func applicationWillTerminate(_ notification: Notification) {
        // Daemon handles cleanup - nothing needed here
    }

    private func registerDefaultSettings() {
        UserDefaults.standard.register(defaults: [
            SettingsKey.enabled: true,
            SettingsKey.method: InputMode.telex.rawValue,
            SettingsKey.perAppMode: true,
            SettingsKey.autoWShortcut: true,
            SettingsKey.escRestore: false,
            SettingsKey.modernTone: true,
            SettingsKey.englishAutoRestore: false,
            SettingsKey.autoCapitalize: false,
            SettingsKey.soundEnabled: false,
        ])
    }
}
