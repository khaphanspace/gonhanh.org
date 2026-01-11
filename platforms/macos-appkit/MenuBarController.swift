// MenuBarController.swift - Pure AppKit (no SwiftUI/NSHostingController)
// Target: <10MB RAM

import Cocoa

class MenuBarController: NSObject, NSWindowDelegate {
    private var statusItem: NSStatusItem!
    private var settingsWindow: NSWindow?
    private var onboardingWindow: NSWindow?

    override init() {
        super.init()
        statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)

        setupMenu()
        setupNotifications()
        updateStatusButton()

        if UserDefaults.standard.bool(forKey: SettingsKey.hasCompletedOnboarding) && AXIsProcessTrusted() {
            startEngine()
        } else {
            showOnboarding()
        }
    }

    // MARK: - Menu Setup

    private func setupMenu() {
        let menu = NSMenu()

        // Header with app info
        let header = NSMenuItem()
        header.view = createHeaderView()
        header.tag = 1
        menu.addItem(header)
        menu.addItem(.separator())

        // Toggle on/off
        let toggle = NSMenuItem(title: "Bật/Tắt tiếng Việt", action: #selector(toggleVietnamese), keyEquivalent: "")
        toggle.target = self
        toggle.tag = 2
        menu.addItem(toggle)
        menu.addItem(.separator())

        // Input methods
        let telex = NSMenuItem(title: InputMode.telex.name, action: #selector(selectTelex), keyEquivalent: "")
        telex.target = self
        telex.tag = 10
        menu.addItem(telex)

        let vni = NSMenuItem(title: InputMode.vni.name, action: #selector(selectVNI), keyEquivalent: "")
        vni.target = self
        vni.tag = 11
        menu.addItem(vni)
        menu.addItem(.separator())

        // Settings
        let settings = NSMenuItem(title: "Cài đặt...", action: #selector(showSettings), keyEquivalent: ",")
        settings.target = self
        menu.addItem(settings)

        // About
        let about = NSMenuItem(title: "Giới thiệu", action: #selector(showAbout), keyEquivalent: "")
        about.target = self
        menu.addItem(about)
        menu.addItem(.separator())

        // Quit
        let quit = NSMenuItem(title: "Thoát Gõ Nhanh", action: #selector(NSApp.terminate), keyEquivalent: "q")
        menu.addItem(quit)

        statusItem.menu = menu
        updateMenu()
    }

    private func createHeaderView() -> NSView {
        let view = NSView(frame: NSRect(x: 0, y: 0, width: 220, height: 36))

        // App icon
        let iconView = NSImageView(frame: NSRect(x: 14, y: 4, width: 28, height: 28))
        iconView.image = NSImage(named: "AppIcon")
        iconView.imageScaling = .scaleProportionallyUpOrDown
        view.addSubview(iconView)

        // App name
        let nameLabel = NSTextField(labelWithString: "Gõ Nhanh")
        nameLabel.font = .systemFont(ofSize: 13, weight: .semibold)
        nameLabel.frame = NSRect(x: 48, y: 16, width: 100, height: 16)
        view.addSubview(nameLabel)

        // Status text
        let state = AppState.shared
        let statusText = state.isEnabled ? state.currentMethod.name : "Đã tắt"
        let shortcutDisplay = state.toggleShortcut.displayParts.joined()
        let statusLabel = NSTextField(labelWithString: "\(statusText) · \(shortcutDisplay)")
        statusLabel.font = .systemFont(ofSize: 11)
        statusLabel.textColor = .secondaryLabelColor
        statusLabel.frame = NSRect(x: 48, y: 2, width: 150, height: 14)
        statusLabel.tag = 100
        view.addSubview(statusLabel)

        return view
    }

    private func updateMenu() {
        guard let menu = statusItem.menu else { return }
        menu.item(withTag: 1)?.view = createHeaderView()
        menu.item(withTag: 10)?.state = AppState.shared.currentMethod == .telex ? .on : .off
        menu.item(withTag: 11)?.state = AppState.shared.currentMethod == .vni ? .on : .off
    }

    // MARK: - Status Button

    private func updateStatusButton() {
        guard let button = statusItem.button else { return }
        button.title = ""

        let observer = InputSourceObserver.shared
        let text: String
        if observer.isAllowedInputSource {
            text = AppState.shared.isEnabled ? "V" : "E"
        } else {
            text = observer.currentDisplayChar
        }
        button.image = createStatusIcon(text: text)
    }

    private func createStatusIcon(text: String) -> NSImage {
        let width: CGFloat = 22
        let height: CGFloat = 16
        let image = NSImage(size: NSSize(width: width, height: height))

        image.lockFocus()

        // Draw rounded rect background
        let rect = NSRect(x: 0, y: 0, width: width, height: height)
        let path = NSBezierPath(roundedRect: rect, xRadius: 3, yRadius: 3)
        NSColor.black.setFill()
        path.fill()

        // Cut out text as transparent
        let font = NSFont.systemFont(ofSize: 13, weight: .bold)
        let attrs: [NSAttributedString.Key: Any] = [
            .font: font,
            .foregroundColor: NSColor.black
        ]
        let textSize = text.size(withAttributes: attrs)
        let textRect = NSRect(
            x: (width - textSize.width) / 2,
            y: (height - textSize.height) / 2,
            width: textSize.width,
            height: textSize.height
        )
        NSGraphicsContext.current?.compositingOperation = .destinationOut
        text.draw(in: textRect, withAttributes: attrs)

        image.unlockFocus()
        image.isTemplate = true
        return image
    }

    // MARK: - Actions

    @objc private func toggleVietnamese() {
        AppState.shared.toggle()
        SoundManager.shared.playToggleSound(enabled: AppState.shared.isEnabled)
        updateStatusButton()
        updateMenu()
    }

    @objc private func selectTelex() {
        AppState.shared.setMethod(.telex)
        updateMenu()
    }

    @objc private func selectVNI() {
        AppState.shared.setMethod(.vni)
        updateMenu()
    }

    @objc private func showSettings() {
        if settingsWindow == nil {
            settingsWindow = SettingsWindowController.createWindow()
            settingsWindow?.delegate = self
        }
        NSApp.setActivationPolicy(.regular)
        NSApp.activate(ignoringOtherApps: true)
        settingsWindow?.makeKeyAndOrderFront(nil)
    }

    @objc private func showAbout() {
        NSApp.orderFrontStandardAboutPanel(nil)
    }

    // MARK: - Notifications

    private func setupNotifications() {
        let nc = NotificationCenter.default

        nc.addObserver(
            self,
            selector: #selector(handleToggleVietnamese),
            name: .toggleVietnamese,
            object: nil
        )

        nc.addObserver(
            self,
            selector: #selector(handleInputSourceChanged),
            name: .inputSourceChanged,
            object: nil
        )

        nc.addObserver(
            self,
            selector: #selector(handleShortcutChanged),
            name: .shortcutChanged,
            object: nil
        )

        nc.addObserver(
            self,
            selector: #selector(onboardingDidComplete),
            name: .onboardingCompleted,
            object: nil
        )

        nc.addObserver(
            self,
            selector: #selector(handleAppStateChanged),
            name: .appStateChanged,
            object: nil
        )
    }

    @objc private func handleToggleVietnamese() {
        AppState.shared.toggle()
        SoundManager.shared.playToggleSound(enabled: AppState.shared.isEnabled)
        updateStatusButton()
        updateMenu()
    }

    @objc private func handleInputSourceChanged() {
        updateStatusButton()
        updateMenu()
    }

    @objc private func handleShortcutChanged() {
        updateMenu()
    }

    @objc private func handleAppStateChanged() {
        updateStatusButton()
        updateMenu()
    }

    @objc private func onboardingDidComplete() {
        updateStatusButton()
        startEngine()
        enableLaunchAtLogin()
    }

    // MARK: - Engine

    private func startEngine() {
        RustBridge.initialize()
        KeyboardHookManager.shared.start()

        let state = AppState.shared
        RustBridge.setEnabled(state.isEnabled)
        RustBridge.setMethod(state.currentMethod.rawValue)
        RustBridge.setModernTone(state.modernTone)
        RustBridge.setSkipWShortcut(!state.autoWShortcut)
        RustBridge.setBracketShortcut(state.bracketShortcut)
        RustBridge.setEscRestore(state.escRestore)
        RustBridge.setEnglishAutoRestore(state.englishAutoRestore)
        RustBridge.setAutoCapitalize(state.autoCapitalize)

        state.syncShortcutsToEngine()
        PerAppModeManager.shared.start()
    }

    private func enableLaunchAtLogin() {
        do {
            try LaunchAtLoginManager.shared.enable()
        } catch {
            print("[LaunchAtLogin] Error: \(error)")
        }
    }

    // MARK: - Onboarding

    private func showOnboarding() {
        if onboardingWindow == nil {
            onboardingWindow = OnboardingWindowController.createWindow()
        }
        onboardingWindow?.makeKeyAndOrderFront(nil)
        NSApp.activate(ignoringOtherApps: true)
    }

    // MARK: - NSWindowDelegate

    func windowWillClose(_ notification: Notification) {
        guard let window = notification.object as? NSWindow,
              window === settingsWindow else { return }
        NSApp.setActivationPolicy(.accessory)
    }
}

// MARK: - Notifications

extension Notification.Name {
    static let onboardingCompleted = Notification.Name("onboardingCompleted")
    static let appStateChanged = Notification.Name("appStateChanged")
}
