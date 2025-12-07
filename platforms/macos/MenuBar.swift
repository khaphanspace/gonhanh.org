import Cocoa
import SwiftUI

// MARK: - Menu Tags

private enum MenuTag: Int {
    case enabled = 100
    case telex = 200
    case vni = 201
}

// MARK: - Menu Bar Controller

class MenuBarController {
    private var statusItem: NSStatusItem!
    private var onboardingWindow: NSWindow?
    private var aboutWindow: NSWindow?

    private var isEnabled = true
    private var currentMethod: InputMode = .telex

    init() {
        statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)

        // Listen for onboarding completion
        NotificationCenter.default.addObserver(
            self,
            selector: #selector(onboardingDidComplete),
            name: .onboardingCompleted,
            object: nil
        )

        // Check if need onboarding
        let hasOnboarded = UserDefaults.standard.bool(forKey: SettingsKey.hasCompletedOnboarding)
        let hasPermission = AXIsProcessTrusted()

        if hasOnboarded && hasPermission {
            // Normal startup
            loadSettings()
            setupUI()
            startEngine()
        } else {
            // Show onboarding first, use defaults for UI
            setupUI()
            showOnboarding()
        }
    }

    // MARK: - Setup

    private func loadSettings() {
        let defaults = UserDefaults.standard

        // Enabled (default: true)
        isEnabled = defaults.object(forKey: SettingsKey.enabled) != nil
            ? defaults.bool(forKey: SettingsKey.enabled)
            : true

        // Method (default: telex)
        let methodValue = defaults.integer(forKey: SettingsKey.method)
        currentMethod = InputMode(rawValue: methodValue) ?? .telex
    }

    private func setupUI() {
        updateStatusButton()
        setupMenu()
    }

    private func startEngine() {
        RustBridge.initialize()
        KeyboardHookManager.shared.start()
        RustBridge.setEnabled(isEnabled)
        RustBridge.setMethod(currentMethod.rawValue)
    }

    @objc private func onboardingDidComplete() {
        // Reload settings (user may have selected VNI in onboarding)
        loadSettings()
        updateStatusButton()
        updateMenu()
        startEngine()
    }

    // MARK: - Status Button

    private func updateStatusButton() {
        guard let button = statusItem.button else { return }

        if isEnabled {
            button.image = NSImage(systemSymbolName: "keyboard.fill", accessibilityDescription: AppMetadata.name)
            button.title = " \(currentMethod.shortName)"
        } else {
            button.image = NSImage(systemSymbolName: "keyboard", accessibilityDescription: AppMetadata.name)
            button.title = ""
        }
        button.imagePosition = .imageLeading
    }

    // MARK: - Menu

    private func setupMenu() {
        let menu = NSMenu()

        // Header
        let header = NSMenuItem()
        header.view = createHeaderView()
        menu.addItem(header)
        menu.addItem(.separator())

        // Enable toggle
        let enableItem = NSMenuItem(title: "Bật GoNhanh", action: #selector(toggleEnabled), keyEquivalent: "")
        enableItem.target = self
        enableItem.tag = MenuTag.enabled.rawValue
        enableItem.state = isEnabled ? .on : .off
        menu.addItem(enableItem)
        menu.addItem(.separator())

        // Method selection
        let methodLabel = NSMenuItem(title: "Kiểu gõ:", action: nil, keyEquivalent: "")
        methodLabel.isEnabled = false
        menu.addItem(methodLabel)

        let telexItem = NSMenuItem(title: "   Telex", action: #selector(selectTelex), keyEquivalent: "t")
        telexItem.keyEquivalentModifierMask = [.command, .shift]
        telexItem.target = self
        telexItem.tag = MenuTag.telex.rawValue
        telexItem.state = currentMethod == .telex ? .on : .off
        menu.addItem(telexItem)

        let vniItem = NSMenuItem(title: "   VNI", action: #selector(selectVNI), keyEquivalent: "v")
        vniItem.keyEquivalentModifierMask = [.command, .shift]
        vniItem.target = self
        vniItem.tag = MenuTag.vni.rawValue
        vniItem.state = currentMethod == .vni ? .on : .off
        menu.addItem(vniItem)

        menu.addItem(.separator())

        // About & Help
        let aboutItem = NSMenuItem(title: "Về \(AppMetadata.name)", action: #selector(showAbout), keyEquivalent: "")
        aboutItem.target = self
        menu.addItem(aboutItem)

        let helpItem = NSMenuItem(title: "Trợ giúp & Góp ý", action: #selector(openHelp), keyEquivalent: "?")
        helpItem.target = self
        menu.addItem(helpItem)

        menu.addItem(.separator())

        let quitItem = NSMenuItem(title: "Thoát", action: #selector(quit), keyEquivalent: "q")
        quitItem.target = self
        menu.addItem(quitItem)

        statusItem.menu = menu
    }

    private func createHeaderView() -> NSView {
        let view = NSView(frame: NSRect(x: 0, y: 0, width: 220, height: 44))

        let title = NSTextField(labelWithString: AppMetadata.name)
        title.font = .systemFont(ofSize: 13, weight: .semibold)
        title.frame = NSRect(x: 14, y: 22, width: 120, height: 16)
        view.addSubview(title)

        let status = NSTextField(labelWithString: isEnabled ? "Đang bật • \(currentMethod.name)" : "Đang tắt")
        status.font = .systemFont(ofSize: 11)
        status.textColor = isEnabled ? .systemGreen : .secondaryLabelColor
        status.frame = NSRect(x: 14, y: 6, width: 140, height: 14)
        view.addSubview(status)

        let version = NSTextField(labelWithString: "v\(AppMetadata.version)")
        version.font = .systemFont(ofSize: 10)
        version.textColor = .tertiaryLabelColor
        version.alignment = .right
        version.frame = NSRect(x: 160, y: 22, width: 46, height: 14)
        view.addSubview(version)

        return view
    }

    private func updateMenu() {
        guard let menu = statusItem.menu else { return }

        // Update header
        if let header = menu.items.first {
            header.view = createHeaderView()
        }

        // Update states
        menu.item(withTag: MenuTag.enabled.rawValue)?.state = isEnabled ? .on : .off
        menu.item(withTag: MenuTag.telex.rawValue)?.state = currentMethod == .telex ? .on : .off
        menu.item(withTag: MenuTag.vni.rawValue)?.state = currentMethod == .vni ? .on : .off
    }

    // MARK: - Actions

    @objc private func toggleEnabled() {
        isEnabled.toggle()
        UserDefaults.standard.set(isEnabled, forKey: SettingsKey.enabled)
        RustBridge.setEnabled(isEnabled)
        updateStatusButton()
        updateMenu()
    }

    @objc private func selectTelex() {
        setMethod(.telex)
    }

    @objc private func selectVNI() {
        setMethod(.vni)
    }

    private func setMethod(_ mode: InputMode) {
        currentMethod = mode
        UserDefaults.standard.set(mode.rawValue, forKey: SettingsKey.method)
        RustBridge.setMethod(mode.rawValue)
        updateStatusButton()
        updateMenu()
    }

    @objc private func showOnboarding() {
        if onboardingWindow == nil {
            let view = OnboardingView()
            let controller = NSHostingController(rootView: view)
            onboardingWindow = NSWindow(contentViewController: controller)
            onboardingWindow?.title = "Chào mừng đến với \(AppMetadata.name)"
            onboardingWindow?.styleMask = [.titled, .closable]
            onboardingWindow?.setContentSize(NSSize(width: 500, height: 420))
            onboardingWindow?.center()
        }
        onboardingWindow?.makeKeyAndOrderFront(nil)
        NSApp.activate(ignoringOtherApps: true)
    }

    @objc private func showAbout() {
        if aboutWindow == nil {
            let view = AboutView()
            let controller = NSHostingController(rootView: view)
            aboutWindow = NSWindow(contentViewController: controller)
            aboutWindow?.title = "Về \(AppMetadata.name)"
            aboutWindow?.styleMask = [.titled, .closable]
            aboutWindow?.setContentSize(NSSize(width: 340, height: 380))
            aboutWindow?.center()
        }
        aboutWindow?.makeKeyAndOrderFront(nil)
        NSApp.activate(ignoringOtherApps: true)
    }

    @objc private func openHelp() {
        if let url = URL(string: AppMetadata.issuesURL) {
            NSWorkspace.shared.open(url)
        }
    }

    @objc private func quit() {
        NSApp.terminate(nil)
    }
}
