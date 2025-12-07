import Cocoa
import SwiftUI

// MARK: - Menu Bar Controller

class MenuBarController {
    private var statusItem: NSStatusItem!
    private var onboardingWindow: NSWindow?
    private var aboutWindow: NSWindow?

    private var isEnabled = true
    private var currentMethod: InputMode = .telex

    init() {
        statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)

        NotificationCenter.default.addObserver(
            self,
            selector: #selector(onboardingDidComplete),
            name: .onboardingCompleted,
            object: nil
        )

        let hasOnboarded = UserDefaults.standard.bool(forKey: SettingsKey.hasCompletedOnboarding)
        let hasPermission = AXIsProcessTrusted()

        if hasOnboarded && hasPermission {
            loadSettings()
            setupUI()
            startEngine()
        } else {
            setupUI()
            showOnboarding()
        }
    }

    // MARK: - Setup

    private func loadSettings() {
        let defaults = UserDefaults.standard

        isEnabled = defaults.object(forKey: SettingsKey.enabled) != nil
            ? defaults.bool(forKey: SettingsKey.enabled)
            : true

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
        loadSettings()
        updateStatusButton()
        updateMenu()
        startEngine()
    }

    // MARK: - Status Button

    private func updateStatusButton() {
        guard let button = statusItem.button else { return }

        // Hiển thị: V (Việt) hoặc E (English/tắt)
        if isEnabled {
            button.title = "V"
            button.contentTintColor = .controlAccentColor
        } else {
            button.title = "E"
            button.contentTintColor = .secondaryLabelColor
        }

        button.font = .systemFont(ofSize: 14, weight: .semibold)
    }

    // MARK: - Menu

    private func setupMenu() {
        let menu = NSMenu()

        // Header với trạng thái
        let headerItem = NSMenuItem()
        headerItem.view = createHeaderView()
        menu.addItem(headerItem)

        menu.addItem(.separator())

        // Toggle Bật/Tắt - item chính
        let toggleItem = NSMenuItem(
            title: isEnabled ? "Tắt gõ tiếng Việt" : "Bật gõ tiếng Việt",
            action: #selector(toggleEnabled),
            keyEquivalent: " "  // Space bar
        )
        toggleItem.target = self
        toggleItem.tag = 100
        menu.addItem(toggleItem)

        menu.addItem(.separator())

        // Kiểu gõ
        let telexItem = NSMenuItem(
            title: "Telex",
            action: #selector(selectTelex),
            keyEquivalent: "1"
        )
        telexItem.keyEquivalentModifierMask = [.command]
        telexItem.target = self
        telexItem.tag = 200
        telexItem.state = currentMethod == .telex ? .on : .off
        menu.addItem(telexItem)

        let vniItem = NSMenuItem(
            title: "VNI",
            action: #selector(selectVNI),
            keyEquivalent: "2"
        )
        vniItem.keyEquivalentModifierMask = [.command]
        vniItem.target = self
        vniItem.tag = 201
        vniItem.state = currentMethod == .vni ? .on : .off
        menu.addItem(vniItem)

        menu.addItem(.separator())

        // Giới thiệu
        let aboutItem = NSMenuItem(
            title: "Giới thiệu \(AppMetadata.name)",
            action: #selector(showAbout),
            keyEquivalent: ""
        )
        aboutItem.target = self
        menu.addItem(aboutItem)

        // Góp ý
        let feedbackItem = NSMenuItem(
            title: "Góp ý & Báo lỗi",
            action: #selector(openHelp),
            keyEquivalent: ""
        )
        feedbackItem.target = self
        menu.addItem(feedbackItem)

        menu.addItem(.separator())

        // Thoát
        let quitItem = NSMenuItem(
            title: "Thoát",
            action: #selector(quit),
            keyEquivalent: "q"
        )
        quitItem.target = self
        menu.addItem(quitItem)

        statusItem.menu = menu
    }

    private func createHeaderView() -> NSView {
        let view = NSView(frame: NSRect(x: 0, y: 0, width: 220, height: 50))

        // App name
        let title = NSTextField(labelWithString: AppMetadata.name)
        title.font = .systemFont(ofSize: 14, weight: .bold)
        title.frame = NSRect(x: 14, y: 28, width: 140, height: 18)
        view.addSubview(title)

        // Status indicator
        let statusDot = NSView(frame: NSRect(x: 14, y: 10, width: 8, height: 8))
        statusDot.wantsLayer = true
        statusDot.layer?.cornerRadius = 4
        statusDot.layer?.backgroundColor = isEnabled ? NSColor.systemGreen.cgColor : NSColor.systemGray.cgColor
        view.addSubview(statusDot)

        // Status text
        let statusText: String
        if isEnabled {
            statusText = "Đang bật • \(currentMethod.name)"
        } else {
            statusText = "Đang tắt"
        }
        let status = NSTextField(labelWithString: statusText)
        status.font = .systemFont(ofSize: 12)
        status.textColor = .secondaryLabelColor
        status.frame = NSRect(x: 26, y: 6, width: 120, height: 16)
        view.addSubview(status)

        // Version
        let version = NSTextField(labelWithString: "v\(AppMetadata.version)")
        version.font = .systemFont(ofSize: 10)
        version.textColor = .tertiaryLabelColor
        version.alignment = .right
        version.frame = NSRect(x: 160, y: 28, width: 46, height: 14)
        view.addSubview(version)

        return view
    }

    private func updateMenu() {
        guard let menu = statusItem.menu else { return }

        // Update header
        if let headerItem = menu.items.first {
            headerItem.view = createHeaderView()
        }

        // Update toggle text
        menu.item(withTag: 100)?.title = isEnabled ? "Tắt gõ tiếng Việt" : "Bật gõ tiếng Việt"

        // Update method states
        menu.item(withTag: 200)?.state = currentMethod == .telex ? .on : .off
        menu.item(withTag: 201)?.state = currentMethod == .vni ? .on : .off
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
            onboardingWindow?.title = AppMetadata.name
            onboardingWindow?.styleMask = [.titled, .closable]
            onboardingWindow?.setContentSize(controller.view.fittingSize)
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
            aboutWindow?.title = "Giới thiệu \(AppMetadata.name)"
            aboutWindow?.styleMask = [.titled, .closable]
            aboutWindow?.setContentSize(NSSize(width: 300, height: 340))
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
