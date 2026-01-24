import Cocoa
import SwiftUI
import Combine

// MARK: - Notifications

extension Notification.Name {
    static let showSettingsPage = Notification.Name("showSettingsPage")
}

// MARK: - Menu Bar Controller

class MenuBarController: NSObject, NSWindowDelegate {
    private var statusItem: NSStatusItem!

    private var onboardingWindow: NSWindow?
    private var updateWindow: NSWindow?
    private var settingsWindow: NSWindow?

    private let appState = AppState.shared
    private var updateStateObserver: NSObjectProtocol?
    private var cancellables = Set<AnyCancellable>()

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

    deinit {
        if let observer = updateStateObserver {
            NotificationCenter.default.removeObserver(observer)
        }
    }

    // MARK: - Setup

    private func setupNotifications() {
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

        NotificationCenter.default.addObserver(
            self,
            selector: #selector(checkForUpdates),
            name: .showUpdateWindow,
            object: nil
        )

        NotificationCenter.default.addObserver(
            self,
            selector: #selector(handleInputSourceChanged),
            name: .inputSourceChanged,
            object: nil
        )

        NotificationCenter.default.addObserver(
            self,
            selector: #selector(handleShowSettingsPage),
            name: .showSettingsPage,
            object: nil
        )

        NotificationCenter.default.addObserver(
            self,
            selector: #selector(handleShortcutChanged),
            name: .shortcutChanged,
            object: nil
        )

        // Observe UpdateManager state changes to update menu
        updateStateObserver = NotificationCenter.default.addObserver(
            forName: .updateStateChanged,
            object: nil,
            queue: .main
        ) { [weak self] _ in
            self?.updateMenu()
        }

        // Observe AppState changes via Combine
        appState.$isEnabled
            .combineLatest(appState.$currentMethod)
            .receive(on: DispatchQueue.main)
            .sink { [weak self] _, _ in
                self?.updateStatusButton()
                self?.updateMenu()
            }
            .store(in: &cancellables)
    }

    @objc private func handleShowSettingsPage() {
        showSettings()
    }

    private func setupMenu() {
        let menu = NSMenu()

        // Header with toggle
        let header = NSMenuItem()
        header.view = createHeaderView()
        header.tag = 1
        menu.addItem(header)
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
        let settings = NSMenuItem(title: "Cài đặt...", action: #selector(showSettings), keyEquivalent: "")
        settings.target = self
        menu.addItem(settings)

        // About
        let about = NSMenuItem(title: "Giới thiệu", action: #selector(showAbout), keyEquivalent: "")
        about.target = self
        menu.addItem(about)

        // Update menu item (dynamic based on state)
        let updateItem = NSMenuItem(title: "", action: #selector(handleUpdateAction), keyEquivalent: "")
        updateItem.target = self
        updateItem.tag = 20
        menu.addItem(updateItem)
        menu.addItem(.separator())

        // Quit
        let quit = NSMenuItem(title: "Thoát \(AppMetadata.name)", action: #selector(NSApp.terminate), keyEquivalent: "q")
        menu.addItem(quit)

        statusItem.menu = menu
        updateMenu()
    }

    private func createHeaderView() -> NSView {
        let view = NSView(frame: NSRect(x: 0, y: 0, width: 220, height: 36))

        // App icon
        let iconView = NSImageView(frame: NSRect(x: 14, y: 4, width: 28, height: 28))
        iconView.image = AppMetadata.logo
        iconView.imageScaling = .scaleProportionallyUpOrDown
        view.addSubview(iconView)

        // App name + status
        let nameLabel = NSTextField(labelWithString: AppMetadata.name)
        nameLabel.font = .systemFont(ofSize: 13, weight: .semibold)
        nameLabel.frame = NSRect(x: 48, y: 16, width: 100, height: 16)
        view.addSubview(nameLabel)

        let statusText = appState.isEnabled ? appState.currentMethod.name : "Đã tắt"
        let statusLabel = NSTextField(labelWithString: "\(statusText) · \(appState.toggleShortcut.displayParts.joined())")
        statusLabel.font = .systemFont(ofSize: 11)
        statusLabel.textColor = .secondaryLabelColor
        statusLabel.frame = NSRect(x: 48, y: 2, width: 100, height: 14)
        statusLabel.tag = 100
        view.addSubview(statusLabel)

        // Toggle switch using SwiftUI
        let toggleView = NSHostingView(rootView:
            Toggle("", isOn: Binding(
                get: { [weak self] in self?.appState.isEnabled ?? true },
                set: { [weak self] newValue in
                    self?.appState.isEnabled = newValue
                    SoundManager.shared.playToggleSound(enabled: newValue)
                }
            ))
            .toggleStyle(.switch)
            .labelsHidden()
            .scaleEffect(0.8)
        )
        toggleView.frame = NSRect(x: 162, y: 4, width: 50, height: 28)
        view.addSubview(toggleView)

        return view
    }

    private func updateMenu() {
        guard let menu = statusItem.menu else { return }
        menu.item(withTag: 1)?.view = createHeaderView()
        menu.item(withTag: 10)?.state = appState.currentMethod == .telex ? .on : .off
        menu.item(withTag: 11)?.state = appState.currentMethod == .vni ? .on : .off

        // Update menu item based on UpdateManager state
        if let updateItem = menu.item(withTag: 20) {
            switch UpdateManager.shared.state {
            case .idle:
                updateItem.title = "Kiểm tra cập nhật"
                updateItem.isEnabled = true
            case .checking:
                updateItem.title = "⏳ Đang kiểm tra..."
                updateItem.isEnabled = false
            case .available(let info):
                updateItem.title = "⬇️ Cập nhật v\(info.version)"
                updateItem.isEnabled = true
            case .downloading(let progress):
                updateItem.title = "⏳ Đang tải... \(Int(progress * 100))%"
                updateItem.isEnabled = false
            case .installing:
                updateItem.title = "🔄 Đang cài đặt..."
                updateItem.isEnabled = false
            case .upToDate:
                updateItem.title = "✓ Đã mới nhất"
                updateItem.isEnabled = false
                // Reset to idle after 3s
                DispatchQueue.main.asyncAfter(deadline: .now() + 3) {
                    if case .upToDate = UpdateManager.shared.state {
                        UpdateManager.shared.state = .idle
                    }
                }
            case .error:
                updateItem.title = "⚠️ Lỗi - thử lại"
                updateItem.isEnabled = true
            }
        }
    }

    @objc private func selectTelex() { appState.setMethod(.telex) }
    @objc private func selectVNI() { appState.setMethod(.vni) }

    @objc private func handleUpdateAction() {
        switch UpdateManager.shared.state {
        case .available, .idle, .error, .upToDate:
            // Show update dialog for all clickable states
            checkForUpdates()
        default:
            // Downloading/installing - do nothing (menu item disabled anyway)
            break
        }
    }

    @objc private func showAbout() {
        showSettings()
        // Switch to About page after window is shown
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.05) {
            NotificationCenter.default.post(name: .showSettingsPage, object: NavigationPage.about)
        }
    }

    private func startEngine() {
        RustBridge.initialize()
        KeyboardHookManager.shared.start()
        RustBridge.setEnabled(appState.isEnabled)
        RustBridge.setMethod(appState.currentMethod.rawValue)

        // Sync all engine settings (these were loaded before engine was ready)
        RustBridge.setModernTone(appState.modernTone)
        RustBridge.setSkipWShortcut(!appState.autoWShortcut)
        RustBridge.setBracketShortcut(appState.bracketShortcut)
        RustBridge.setRestoreShortcutEnabled(appState.restoreShortcutEnabled)
        RustBridge.setEnglishAutoRestore(appState.englishAutoRestore)
        RustBridge.setAutoCapitalize(appState.autoCapitalize)
        RustBridge.setAllowForeignConsonants(appState.allowForeignConsonants)

        // Sync shortcuts and start per-app mode manager
        appState.syncShortcutsToEngine()
        PerAppModeManager.shared.start()

        // Reopen settings if coming from update
        if UserDefaults.standard.bool(forKey: SettingsKey.reopenSettingsAfterUpdate) {
            UserDefaults.standard.removeObject(forKey: SettingsKey.reopenSettingsAfterUpdate)
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.5) { [weak self] in
                self?.showSettings()
            }
        }

        DispatchQueue.main.asyncAfter(deadline: .now() + 3) {
            UpdateManager.shared.checkForUpdatesSilently()
        }
    }

    // MARK: - Status Button

    private func updateStatusButton() {
        DispatchQueue.main.async { [weak self] in
            guard let self = self, let button = self.statusItem.button else { return }
            button.title = ""

            let observer = InputSourceObserver.shared
            let text: String
            if observer.isAllowedInputSource {
                // ABC keyboard: show V (enabled) or E (disabled)
                text = self.appState.isEnabled ? "V" : "E"
            } else {
                // Non-ABC keyboard: show input source character
                text = observer.currentDisplayChar
            }
            button.image = self.createStatusIcon(text: text)
        }
    }

    private func createStatusIcon(text: String) -> NSImage {
        let width: CGFloat = 22
        let height: CGFloat = 16
        let image = NSImage(size: NSSize(width: width, height: height))

        image.lockFocus()

        // Draw rounded rect background (black for template)
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

        // Template image: macOS auto-adjusts color for light/dark menu bar
        image.isTemplate = true
        return image
    }

    // MARK: - Event Handlers

    @objc private func handleToggleVietnamese() {
        appState.toggle()
        SoundManager.shared.playToggleSound(enabled: appState.isEnabled)
    }

    @objc private func handleInputSourceChanged() {
        updateStatusButton()
        updateMenu()
    }

    @objc private func handleShortcutChanged() {
        updateMenu()
    }

    @objc private func onboardingDidComplete() {
        updateStatusButton()
        startEngine()
        enableLaunchAtLogin()
    }

    private func enableLaunchAtLogin() {
        do {
            try LaunchAtLoginManager.shared.enable()
        } catch {
            print("[LaunchAtLogin] Error: \(error)")
        }
    }

    // MARK: - Windows

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

    @objc private func showSettings() {
        if settingsWindow == nil {
            let controller = NSHostingController(rootView: MainSettingsView())
            controller.view.wantsLayer = true
            controller.view.layer?.backgroundColor = .clear
            let window = NSWindow(contentViewController: controller)
            window.title = "\(AppMetadata.name) - Cài đặt"
            window.styleMask = NSWindow.StyleMask([.titled, .closable, .miniaturizable, .fullSizeContentView])
            window.standardWindowButton(.zoomButton)?.isHidden = true
            window.setContentSize(NSSize(width: 700, height: 480))
            window.center()
            window.isReleasedWhenClosed = false
            window.titlebarAppearsTransparent = true
            window.titleVisibility = .hidden
            window.backgroundColor = .clear
            window.isOpaque = false
            window.hasShadow = true
            window.isMovableByWindowBackground = true
            window.delegate = self
            settingsWindow = window
        }
        // Show app in menu bar temporarily
        NSApp.setActivationPolicy(.regular)
        setupMainMenu()  // Set menu before showing window
        NSApp.activate(ignoringOtherApps: true)
        settingsWindow?.makeKeyAndOrderFront(nil)
        // Override default menu after window is shown (macOS may reset it)
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.05) { [weak self] in
            self?.setupMainMenu()
        }
        // Clear auto-focus on TextFields
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.1) { [weak self] in
            self?.settingsWindow?.makeFirstResponder(nil)
        }

        // Auto-check for updates when opening settings
        UpdateManager.shared.checkForUpdatesManually()
    }

    private func setupMainMenu() {
        // Clear existing menu first
        NSApp.mainMenu = nil
        let mainMenu = NSMenu()

        // App menu (shows app name in menu bar)
        let appMenu = NSMenu(title: AppMetadata.name)
        let appMenuItem = NSMenuItem(title: AppMetadata.name, action: nil, keyEquivalent: "")
        appMenuItem.submenu = appMenu

        // About
        let aboutItem = NSMenuItem(
            title: "Về \(AppMetadata.name)",
            action: #selector(showAbout),
            keyEquivalent: ""
        )
        aboutItem.target = self
        appMenu.addItem(aboutItem)

        appMenu.addItem(NSMenuItem.separator())

        // Check for updates
        let updateItem = NSMenuItem(
            title: "Kiểm tra cập nhật...",
            action: #selector(checkForUpdates),
            keyEquivalent: ""
        )
        updateItem.target = self
        appMenu.addItem(updateItem)

        appMenu.addItem(NSMenuItem.separator())

        // Settings
        let settingsItem = NSMenuItem(
            title: "Cài đặt...",
            action: #selector(showSettings),
            keyEquivalent: ""
        )
        settingsItem.target = self
        appMenu.addItem(settingsItem)

        appMenu.addItem(NSMenuItem.separator())

        // Close window (⌘W)
        appMenu.addItem(NSMenuItem(title: "Đóng cửa sổ", action: #selector(NSWindow.performClose(_:)), keyEquivalent: "w"))

        // Quit (⌘Q)
        let quitItem = NSMenuItem(
            title: "Thoát \(AppMetadata.name)",
            action: #selector(NSApplication.terminate(_:)),
            keyEquivalent: "q"
        )
        appMenu.addItem(quitItem)

        mainMenu.addItem(appMenuItem)

        // Edit menu (required for copy/paste in TextFields)
        let editMenu = NSMenu(title: "Sửa")
        let editMenuItem = NSMenuItem(title: "Sửa", action: nil, keyEquivalent: "")
        editMenuItem.submenu = editMenu

        editMenu.addItem(NSMenuItem(title: "Hoàn tác", action: Selector(("undo:")), keyEquivalent: "z"))
        editMenu.addItem(NSMenuItem(title: "Làm lại", action: Selector(("redo:")), keyEquivalent: "Z"))
        editMenu.addItem(NSMenuItem.separator())
        editMenu.addItem(NSMenuItem(title: "Cắt", action: #selector(NSText.cut(_:)), keyEquivalent: "x"))
        editMenu.addItem(NSMenuItem(title: "Sao chép", action: #selector(NSText.copy(_:)), keyEquivalent: "c"))
        editMenu.addItem(NSMenuItem(title: "Dán", action: #selector(NSText.paste(_:)), keyEquivalent: "v"))
        editMenu.addItem(NSMenuItem(title: "Chọn tất cả", action: #selector(NSText.selectAll(_:)), keyEquivalent: "a"))

        mainMenu.addItem(editMenuItem)

        NSApp.mainMenu = mainMenu
    }

    @objc private func checkForUpdates() {
        // Force recreate window if already downloading (sidebar auto-download triggered)
        if case .downloading = UpdateManager.shared.state {
            updateWindow = nil
        }

        if updateWindow == nil {
            let controller = NSHostingController(rootView: UpdateView())
            let window = NSWindow(contentViewController: controller)
            window.title = "Kiểm tra cập nhật"
            window.styleMask = [.titled, .closable]
            window.setContentSize(controller.view.fittingSize)
            window.center()
            window.isReleasedWhenClosed = false
            updateWindow = window
        }
        NSApp.activate(ignoringOtherApps: true)
        updateWindow?.makeKeyAndOrderFront(nil)

        // Skip re-check if already in progress
        switch UpdateManager.shared.state {
        case .available, .downloading, .installing:
            return
        default:
            UpdateManager.shared.checkForUpdatesManually()
        }
    }

    // MARK: - NSWindowDelegate

    func windowWillClose(_ notification: Notification) {
        guard let window = notification.object as? NSWindow,
              window === settingsWindow else { return }
        // Revert to background app when settings window closes
        NSApp.setActivationPolicy(.accessory)
    }
}
