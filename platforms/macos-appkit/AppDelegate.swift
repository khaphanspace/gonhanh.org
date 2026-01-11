// AppDelegate.swift - Pure AppKit (no SwiftUI)
// Target: <10MB RAM

import Cocoa

class AppDelegate: NSObject, NSApplicationDelegate {
    var menuBar: MenuBarController?

    func applicationDidFinishLaunching(_ notification: Notification) {
        // Register default settings before anything else
        registerDefaultSettings()

        // Hide dock icon - menubar-only app
        NSApp.setActivationPolicy(.accessory)

        // Create menu bar controller
        menuBar = MenuBarController()

        // Start observing input source changes
        InputSourceObserver.shared.start()

        // Memory cleanup on app deactivation
        NotificationCenter.default.addObserver(
            forName: NSApplication.didResignActiveNotification,
            object: nil,
            queue: .main
        ) { _ in
            clearDetectionCache()
            RustBridge.clearBufferAll()
        }
    }

    func applicationWillTerminate(_ notification: Notification) {
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
            SettingsKey.escRestore: false,
            SettingsKey.modernTone: true,
            SettingsKey.englishAutoRestore: false,
            SettingsKey.autoCapitalize: false,
            SettingsKey.soundEnabled: false,
        ])
    }
}
