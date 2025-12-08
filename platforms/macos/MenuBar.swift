import Cocoa
import SwiftUI

// MARK: - Menu Bar Controller

class MenuBarController {
    private var statusItem: NSStatusItem!
    private var onboardingWindow: NSWindow?
    private var aboutWindow: NSWindow?
    private var toggleSwitch: NSSwitch?

    private var isEnabled = true
    private var currentMethod: InputMode = .telex
    private var isModernTone = true

    init() {
        statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)

        NotificationCenter.default.addObserver(
            self,
            selector: #selector(onboardingDidComplete),
            name: .onboardingCompleted,
            object: nil
        )

        NotificationCenter.default.addObserver(
            self,
            selector: #selector(handleToggleVietnamese),
            name: .toggleVietnamese,
            object: nil
        )

        setupMenu()
        updateStatusButton()

        if UserDefaults.standard.bool(forKey: SettingsKey.hasCompletedOnboarding) && AXIsProcessTrusted() {
            loadSettings()
            startEngine()
        } else {
            showOnboarding()
        }
    }

    @objc private func handleToggleVietnamese() {
        isEnabled.toggle()
        UserDefaults.standard.set(isEnabled, forKey: SettingsKey.enabled)
        RustBridge.setEnabled(isEnabled)
        updateStatusButton()
        updateMenu()
    }

    // MARK: - Setup

    private func loadSettings() {
        isEnabled = UserDefaults.standard.object(forKey: SettingsKey.enabled) as? Bool ?? true
        currentMethod = InputMode(rawValue: UserDefaults.standard.integer(forKey: SettingsKey.method)) ?? .telex
        isModernTone = UserDefaults.standard.object(forKey: SettingsKey.modernTone) as? Bool ?? true
    }

    private func startEngine() {
        RustBridge.initialize()
        KeyboardHookManager.shared.start()
        RustBridge.setEnabled(isEnabled)
        RustBridge.setMethod(currentMethod.rawValue)
        RustBridge.setModern(isModernTone)

        // Check for updates in background after a short delay
        DispatchQueue.main.asyncAfter(deadline: .now() + 3) {
            UpdateManager.shared.checkForUpdatesSilently()
        }
    }

    @objc private func onboardingDidComplete() {
        loadSettings()
        updateStatusButton()
        updateMenu()
        startEngine()
        enableLaunchAtLogin()
    }

    // MARK: - Status Button

    private func updateStatusButton() {
        guard let button = statusItem.button else { return }
        button.title = ""
        button.image = createStatusIcon(text: isEnabled ? "V" : "E")
    }

    private func createStatusIcon(text: String) -> NSImage {
        let size: CGFloat = 18
        let image = NSImage(size: NSSize(width: size, height: size))

        image.lockFocus()

        // Background trắng bo góc
        let rect = NSRect(x: 0, y: 0, width: size, height: size)
        let path = NSBezierPath(roundedRect: rect, xRadius: 4, yRadius: 4)
        NSColor.white.setFill()
        path.fill()

        // Text transparent (dùng .clear với blend mode)
        let font = NSFont.systemFont(ofSize: 13, weight: .bold)
        let attrs: [NSAttributedString.Key: Any] = [
            .font: font,
            .foregroundColor: NSColor.black
        ]
        let textSize = text.size(withAttributes: attrs)
        let textRect = NSRect(
            x: (size - textSize.width) / 2,
            y: (size - textSize.height) / 2,
            width: textSize.width,
            height: textSize.height
        )

        // Vẽ text với blend mode để tạo transparent
        NSGraphicsContext.current?.compositingOperation = .destinationOut
        text.draw(in: textRect, withAttributes: attrs)

        image.unlockFocus()
        image.isTemplate = false
        return image
    }

    // MARK: - Menu

    private func setupMenu() {
        let menu = NSMenu()

        // Header with toggle
        let header = NSMenuItem()
        header.view = createHeaderView()
        header.tag = 1
        menu.addItem(header)
        menu.addItem(.separator())

        // Input methods
        let telex = NSMenuItem(title: "Telex", action: #selector(selectTelex), keyEquivalent: "1")
        telex.keyEquivalentModifierMask = .command
        telex.target = self
        telex.tag = 10
        menu.addItem(telex)

        let vni = NSMenuItem(title: "VNI", action: #selector(selectVNI), keyEquivalent: "2")
        vni.keyEquivalentModifierMask = .command
        vni.target = self
        vni.tag = 11
        menu.addItem(vni)
        menu.addItem(.separator())

        // Tone style
        let modernTone = NSMenuItem(title: "Kiểu mới (hoà)", action: #selector(selectModernTone), keyEquivalent: "")
        modernTone.target = self
        modernTone.tag = 20
        menu.addItem(modernTone)

        let classicTone = NSMenuItem(title: "Kiểu cũ (hòa)", action: #selector(selectClassicTone), keyEquivalent: "")
        classicTone.target = self
        classicTone.tag = 21
        menu.addItem(classicTone)
        menu.addItem(.separator())

        // About & Help
        let about = NSMenuItem(title: "Giới thiệu \(AppMetadata.name)", action: #selector(showAbout), keyEquivalent: "")
        about.target = self
        menu.addItem(about)

        let checkUpdate = NSMenuItem(title: "Kiểm tra cập nhật...", action: #selector(checkForUpdates), keyEquivalent: "u")
        checkUpdate.keyEquivalentModifierMask = .command
        checkUpdate.target = self
        menu.addItem(checkUpdate)

        let help = NSMenuItem(title: "Góp ý & Báo lỗi", action: #selector(openHelp), keyEquivalent: "")
        help.target = self
        menu.addItem(help)
        menu.addItem(.separator())

        // Quit
        let quit = NSMenuItem(title: "Thoát", action: #selector(NSApp.terminate), keyEquivalent: "q")
        menu.addItem(quit)

        statusItem.menu = menu
        updateMenu()
    }

    private func createHeaderView() -> NSView {
        let view = NSView(frame: NSRect(x: 0, y: 0, width: 220, height: 36))

        // App name
        let name = NSTextField(labelWithString: AppMetadata.name)
        name.font = .systemFont(ofSize: 13, weight: .bold)
        name.frame = NSRect(x: 14, y: 10, width: 120, height: 16)
        view.addSubview(name)

        // Toggle switch (reuse existing)
        if toggleSwitch == nil {
            let toggle = NSSwitch()
            toggle.controlSize = .regular
            toggle.target = self
            toggle.action = #selector(toggleEnabled)
            toggle.sizeToFit()
            toggleSwitch = toggle
        }
        toggleSwitch?.state = isEnabled ? .on : .off
        toggleSwitch?.contentTintColor = .systemGreen
        toggleSwitch?.frame.origin = NSPoint(x: 170, y: 6)
        view.addSubview(toggleSwitch!)

        return view
    }

    private func updateMenu() {
        guard let menu = statusItem.menu else { return }
        menu.item(withTag: 1)?.view = createHeaderView()
        menu.item(withTag: 10)?.state = currentMethod == .telex ? .on : .off
        menu.item(withTag: 11)?.state = currentMethod == .vni ? .on : .off
        menu.item(withTag: 20)?.state = isModernTone ? .on : .off
        menu.item(withTag: 21)?.state = isModernTone ? .off : .on
    }

    // MARK: - Actions

    @objc private func toggleEnabled() {
        isEnabled.toggle()
        UserDefaults.standard.set(isEnabled, forKey: SettingsKey.enabled)
        RustBridge.setEnabled(isEnabled)
        updateStatusButton()
        // Update method checkmarks only, toggle animates itself
        guard let menu = statusItem.menu else { return }
        menu.item(withTag: 10)?.state = currentMethod == .telex ? .on : .off
        menu.item(withTag: 11)?.state = currentMethod == .vni ? .on : .off
    }

    @objc private func selectTelex() { setMethod(.telex) }
    @objc private func selectVNI() { setMethod(.vni) }

    private func setMethod(_ mode: InputMode) {
        currentMethod = mode
        UserDefaults.standard.set(mode.rawValue, forKey: SettingsKey.method)
        RustBridge.setMethod(mode.rawValue)
        updateStatusButton()
        updateMenu()
    }

    @objc private func selectModernTone() { setToneStyle(modern: true) }
    @objc private func selectClassicTone() { setToneStyle(modern: false) }

    private func setToneStyle(modern: Bool) {
        isModernTone = modern
        UserDefaults.standard.set(modern, forKey: SettingsKey.modernTone)
        RustBridge.setModern(modern)
        updateMenu()
    }

    private func enableLaunchAtLogin() {
        do {
            try LaunchAtLoginManager.shared.enable()
        } catch {
            debugLog("[LaunchAtLogin] Error: \(error)")
        }
    }

    private func showOnboarding() {
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
            let controller = NSHostingController(rootView: AboutView())
            let window = NSWindow(contentViewController: controller)
            window.title = "Giới thiệu \(AppMetadata.name)"
            window.styleMask = [.titled, .closable]
            window.setContentSize(controller.view.fittingSize)
            window.center()
            window.isReleasedWhenClosed = false
            aboutWindow = window
        }
        NSApp.activate(ignoringOtherApps: true)
        aboutWindow?.makeKeyAndOrderFront(nil)
    }

    @objc private func openHelp() {
        NSWorkspace.shared.open(URL(string: AppMetadata.issuesURL)!)
    }

    @objc private func checkForUpdates() {
        UpdateManager.shared.checkForUpdatesManually()
    }
}
