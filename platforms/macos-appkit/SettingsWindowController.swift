// SettingsWindowController.swift - Pure AppKit Settings UI
// Target: <10MB RAM

import Cocoa

class SettingsWindowController {
    static func createWindow() -> NSWindow {
        let window = NSWindow(
            contentRect: NSRect(x: 0, y: 0, width: 500, height: 400),
            styleMask: [.titled, .closable, .miniaturizable],
            backing: .buffered,
            defer: false
        )
        window.title = "Gõ Nhanh - Cài đặt"
        window.center()
        window.isReleasedWhenClosed = false

        let contentView = SettingsView(frame: window.contentView!.bounds)
        contentView.autoresizingMask = [.width, .height]
        window.contentView = contentView

        return window
    }
}

// MARK: - Settings View

class SettingsView: NSView {
    private var tabView: NSTabView!

    override init(frame: NSRect) {
        super.init(frame: frame)
        setupUI()
    }

    required init?(coder: NSCoder) {
        super.init(coder: coder)
        setupUI()
    }

    private func setupUI() {
        // Tab View
        tabView = NSTabView(frame: bounds.insetBy(dx: 20, dy: 20))
        tabView.autoresizingMask = [.width, .height]

        // General Tab
        let generalTab = NSTabViewItem(identifier: "general")
        generalTab.label = "Chung"
        generalTab.view = createGeneralTab()
        tabView.addTabViewItem(generalTab)

        // Advanced Tab
        let advancedTab = NSTabViewItem(identifier: "advanced")
        advancedTab.label = "Nâng cao"
        advancedTab.view = createAdvancedTab()
        tabView.addTabViewItem(advancedTab)

        // About Tab
        let aboutTab = NSTabViewItem(identifier: "about")
        aboutTab.label = "Giới thiệu"
        aboutTab.view = createAboutTab()
        tabView.addTabViewItem(aboutTab)

        addSubview(tabView)
    }

    // MARK: - General Tab

    private func createGeneralTab() -> NSView {
        let view = NSView(frame: NSRect(x: 0, y: 0, width: 460, height: 300))
        var y: CGFloat = 260

        // Input method selection
        let methodLabel = NSTextField(labelWithString: "Kiểu gõ:")
        methodLabel.frame = NSRect(x: 20, y: y, width: 100, height: 22)
        view.addSubview(methodLabel)

        let methodPopup = NSPopUpButton(frame: NSRect(x: 120, y: y, width: 150, height: 26))
        methodPopup.addItems(withTitles: ["Telex", "VNI"])
        methodPopup.selectItem(at: AppState.shared.currentMethod.rawValue)
        methodPopup.target = self
        methodPopup.action = #selector(methodChanged(_:))
        view.addSubview(methodPopup)

        y -= 40

        // Toggle shortcut
        let shortcutLabel = NSTextField(labelWithString: "Phím tắt:")
        shortcutLabel.frame = NSRect(x: 20, y: y, width: 100, height: 22)
        view.addSubview(shortcutLabel)

        let shortcutDisplay = NSTextField(labelWithString: AppState.shared.toggleShortcut.displayParts.joined())
        shortcutDisplay.frame = NSRect(x: 120, y: y, width: 150, height: 22)
        shortcutDisplay.font = .monospacedSystemFont(ofSize: 12, weight: .regular)
        view.addSubview(shortcutDisplay)

        y -= 40

        // Options
        let options: [(String, Bool, Selector)] = [
            ("Nhớ chế độ theo ứng dụng", AppState.shared.perAppModeEnabled, #selector(perAppModeChanged(_:))),
            ("w → ư trong Telex", AppState.shared.autoWShortcut, #selector(autoWShortcutChanged(_:))),
            ("Dấu thanh chuẩn (không tự do)", AppState.shared.modernTone, #selector(modernToneChanged(_:))),
            ("Tự động khôi phục tiếng Anh", AppState.shared.englishAutoRestore, #selector(englishAutoRestoreChanged(_:))),
            ("Âm thanh khi bật/tắt", AppState.shared.soundEnabled, #selector(soundEnabledChanged(_:))),
        ]

        for (title, isOn, action) in options {
            let checkbox = NSButton(checkboxWithTitle: title, target: self, action: action)
            checkbox.frame = NSRect(x: 20, y: y, width: 300, height: 22)
            checkbox.state = isOn ? .on : .off
            view.addSubview(checkbox)
            y -= 30
        }

        return view
    }

    // MARK: - Advanced Tab

    private func createAdvancedTab() -> NSView {
        let view = NSView(frame: NSRect(x: 0, y: 0, width: 460, height: 300))
        var y: CGFloat = 260

        let advancedOptions: [(String, Bool, Selector)] = [
            ("[ ] → ơ ư (thay cho aa uw)", AppState.shared.bracketShortcut, #selector(bracketShortcutChanged(_:))),
            ("ESC khôi phục về ASCII", AppState.shared.escRestore, #selector(escRestoreChanged(_:))),
            ("Tự động viết hoa sau dấu câu", AppState.shared.autoCapitalize, #selector(autoCapitalizeChanged(_:))),
        ]

        for (title, isOn, action) in advancedOptions {
            let checkbox = NSButton(checkboxWithTitle: title, target: self, action: action)
            checkbox.frame = NSRect(x: 20, y: y, width: 350, height: 22)
            checkbox.state = isOn ? .on : .off
            view.addSubview(checkbox)
            y -= 30
        }

        return view
    }

    // MARK: - About Tab

    private func createAboutTab() -> NSView {
        let view = NSView(frame: NSRect(x: 0, y: 0, width: 460, height: 300))

        // App icon
        let iconView = NSImageView(frame: NSRect(x: 190, y: 200, width: 80, height: 80))
        iconView.image = NSImage(named: "AppIcon")
        iconView.imageScaling = .scaleProportionallyUpOrDown
        view.addSubview(iconView)

        // App name
        let nameLabel = NSTextField(labelWithString: "Gõ Nhanh")
        nameLabel.font = .systemFont(ofSize: 24, weight: .bold)
        nameLabel.alignment = .center
        nameLabel.frame = NSRect(x: 0, y: 160, width: 460, height: 30)
        view.addSubview(nameLabel)

        // Version
        let version = Bundle.main.object(forInfoDictionaryKey: "CFBundleShortVersionString") as? String ?? "1.0"
        let versionLabel = NSTextField(labelWithString: "Phiên bản \(version)")
        versionLabel.font = .systemFont(ofSize: 12)
        versionLabel.textColor = .secondaryLabelColor
        versionLabel.alignment = .center
        versionLabel.frame = NSRect(x: 0, y: 135, width: 460, height: 20)
        view.addSubview(versionLabel)

        // Description
        let descLabel = NSTextField(labelWithString: "Bộ gõ tiếng Việt nhanh, nhẹ, chính xác")
        descLabel.font = .systemFont(ofSize: 13)
        descLabel.textColor = .secondaryLabelColor
        descLabel.alignment = .center
        descLabel.frame = NSRect(x: 0, y: 100, width: 460, height: 20)
        view.addSubview(descLabel)

        // Website link
        let websiteButton = NSButton(title: "gonhanh.org", target: self, action: #selector(openWebsite))
        websiteButton.bezelStyle = .inline
        websiteButton.frame = NSRect(x: 185, y: 60, width: 90, height: 24)
        view.addSubview(websiteButton)

        return view
    }

    // MARK: - Actions

    @objc private func methodChanged(_ sender: NSPopUpButton) {
        AppState.shared.setMethod(InputMode(rawValue: sender.indexOfSelectedItem) ?? .telex)
    }

    @objc private func perAppModeChanged(_ sender: NSButton) {
        AppState.shared.perAppModeEnabled = sender.state == .on
    }

    @objc private func autoWShortcutChanged(_ sender: NSButton) {
        AppState.shared.autoWShortcut = sender.state == .on
    }

    @objc private func modernToneChanged(_ sender: NSButton) {
        AppState.shared.modernTone = sender.state == .on
    }

    @objc private func englishAutoRestoreChanged(_ sender: NSButton) {
        AppState.shared.englishAutoRestore = sender.state == .on
    }

    @objc private func soundEnabledChanged(_ sender: NSButton) {
        AppState.shared.soundEnabled = sender.state == .on
    }

    @objc private func bracketShortcutChanged(_ sender: NSButton) {
        AppState.shared.bracketShortcut = sender.state == .on
    }

    @objc private func escRestoreChanged(_ sender: NSButton) {
        AppState.shared.escRestore = sender.state == .on
    }

    @objc private func autoCapitalizeChanged(_ sender: NSButton) {
        AppState.shared.autoCapitalize = sender.state == .on
    }

    @objc private func openWebsite() {
        NSWorkspace.shared.open(URL(string: "https://gonhanh.org")!)
    }
}
