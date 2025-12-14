import Cocoa
import SwiftUI

// MARK: - Menu State

class MenuState: ObservableObject {
    static let shared = MenuState()

    @Published var isEnabled: Bool = true
    @Published var currentMethod: InputMode = .telex

    func toggle() {
        isEnabled.toggle()
        UserDefaults.standard.set(isEnabled, forKey: SettingsKey.enabled)
        RustBridge.setEnabled(isEnabled)
        NotificationCenter.default.post(name: .menuStateChanged, object: nil)
    }

    func setMethod(_ method: InputMode) {
        currentMethod = method
        UserDefaults.standard.set(method.rawValue, forKey: SettingsKey.method)
        RustBridge.setMethod(method.rawValue)
        NotificationCenter.default.post(name: .menuStateChanged, object: nil)
    }

    func load() {
        isEnabled = UserDefaults.standard.object(forKey: SettingsKey.enabled) as? Bool ?? true
        currentMethod = InputMode(rawValue: UserDefaults.standard.integer(forKey: SettingsKey.method)) ?? .telex
    }
}

extension Notification.Name {
    static let menuStateChanged = Notification.Name("menuStateChanged")
    static let showSettingsPage = Notification.Name("showSettingsPage")
}

// MARK: - Menu Popover

struct MenuPopoverView: View {
    @ObservedObject var state: MenuState
    let onClose: () -> Void

    @State private var shortcut = KeyboardShortcut.load()

    var body: some View {
        VStack(spacing: 0) {
            header
            Divider().padding(.horizontal, 8)
            methodSection
            Divider().padding(.horizontal, 8)
            actionSection
            Divider().padding(.horizontal, 8)
            quitSection
        }
        .frame(width: 240)
        .onReceive(NotificationCenter.default.publisher(for: .shortcutChanged)) { _ in
            shortcut = KeyboardShortcut.load()
        }
    }

    private var header: some View {
        HStack(spacing: 10) {
            Image(nsImage: AppMetadata.logo)
                .resizable()
                .frame(width: 32, height: 32)

            VStack(alignment: .leading, spacing: 2) {
                Text(AppMetadata.name)
                    .font(.system(size: 13, weight: .semibold))
                HStack(spacing: 4) {
                    Text(state.isEnabled ? state.currentMethod.name : "Đã tắt")
                    Text("·").foregroundColor(Color(NSColor.tertiaryLabelColor))
                    Text(shortcut.displayParts.joined()).foregroundColor(Color(NSColor.tertiaryLabelColor))
                }
                .font(.system(size: 11))
                .foregroundColor(Color(NSColor.secondaryLabelColor))
            }

            Spacer()

            Toggle("", isOn: Binding(get: { state.isEnabled }, set: { _ in state.toggle() }))
                .toggleStyle(.switch)
                .labelsHidden()
                .scaleEffect(0.8)
        }
        .padding(.horizontal, 14)
        .padding(.vertical, 10)
    }

    private var methodSection: some View {
        VStack(spacing: 0) {
            MenuItem(title: InputMode.telex.name, isChecked: state.currentMethod == .telex) {
                state.setMethod(.telex)
            }
            MenuItem(title: InputMode.vni.name, isChecked: state.currentMethod == .vni) {
                state.setMethod(.vni)
            }
        }
        .padding(.vertical, 4)
    }

    private var actionSection: some View {
        VStack(spacing: 0) {
            MenuItem(title: "Cài đặt...") {
                onClose()
                NotificationCenter.default.post(name: .showSettingsPage, object: NavigationPage.settings)
            }
            MenuItem(title: "Giới thiệu") {
                onClose()
                NotificationCenter.default.post(name: .showSettingsPage, object: NavigationPage.about)
            }
            MenuItem(title: "Kiểm tra cập nhật") {
                onClose()
                NotificationCenter.default.post(name: .showUpdateWindow, object: nil)
            }
        }
        .padding(.vertical, 4)
    }

    private var quitSection: some View {
        MenuItem(title: "Thoát \(AppMetadata.name)") {
            NSApp.terminate(nil)
        }
        .padding(.vertical, 4)
    }
}

// MARK: - Menu Item

struct MenuItem: View {
    let title: String
    var isChecked: Bool = false
    let action: () -> Void

    @State private var hovered = false

    var body: some View {
        Button(action: action) {
            HStack(spacing: 0) {
                Text(isChecked ? "✓" : "")
                    .font(.system(size: 13))
                    .frame(width: 20, alignment: .center)
                Text(title)
                    .font(.system(size: 13))
                Spacer()
            }
            .foregroundColor(hovered ? .white : Color(NSColor.labelColor))
            .padding(.horizontal, 10)
            .padding(.vertical, 4)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(RoundedRectangle(cornerRadius: 4).fill(hovered ? Color.accentColor : Color.clear))
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .padding(.horizontal, 6)
        .onHover { hovered = $0 }
    }
}


// MARK: - Menu Bar Controller

class MenuBarController: NSObject {
    private var statusItem: NSStatusItem!
    private var menuPanel: NSPanel?
    private var eventMonitor: Any?
    private var appDeactivateObserver: Any?

    private var onboardingWindow: NSWindow?
    private var updateWindow: NSWindow?
    private var settingsWindow: NSWindow?

    private let menuState = MenuState.shared

    override init() {
        super.init()
        statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)

        setupStatusButton()
        setupNotifications()

        if UserDefaults.standard.bool(forKey: SettingsKey.hasCompletedOnboarding) && AXIsProcessTrusted() {
            loadSettings()
            startEngine()
        } else {
            showOnboarding()
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
            selector: #selector(handleMenuStateChanged),
            name: .menuStateChanged,
            object: nil
        )

        NotificationCenter.default.addObserver(
            self,
            selector: #selector(handleShowSettingsPage),
            name: .showSettingsPage,
            object: nil
        )
    }

    @objc private func handleShowSettingsPage() {
        showSettings()
    }

    private func setupStatusButton() {
        guard let button = statusItem.button else { return }
        button.action = #selector(toggleMenu)
        button.target = self
        button.sendAction(on: [.leftMouseDown, .rightMouseDown])
        updateStatusButton()
    }

    private func loadSettings() {
        menuState.load()
    }

    private func startEngine() {
        RustBridge.initialize()
        KeyboardHookManager.shared.start()
        RustBridge.setEnabled(menuState.isEnabled)
        RustBridge.setMethod(menuState.currentMethod.rawValue)

        // Sync shortcuts and excluded apps from AppState
        syncShortcutsToEngine()
        syncExcludedAppsToEngine()
        ExcludedAppsManager.shared.start()

        DispatchQueue.main.asyncAfter(deadline: .now() + 3) {
            UpdateManager.shared.checkForUpdatesSilently()
        }
    }

    private func syncShortcutsToEngine() {
        let shortcuts = AppState.shared.shortcuts.map { ($0.key, $0.value, $0.isEnabled) }
        RustBridge.syncShortcuts(shortcuts)
    }

    private func syncExcludedAppsToEngine() {
        let bundleIds = AppState.shared.excludedApps
            .filter { $0.isEnabled }
            .map { $0.bundleId }
        ExcludedAppsManager.shared.setExcludedApps(bundleIds)
    }

    // MARK: - Status Button

    private func updateStatusButton() {
        guard let button = statusItem.button else { return }
        button.title = ""
        button.image = createStatusIcon(text: menuState.isEnabled ? "V" : "E")
    }

    private func createStatusIcon(text: String) -> NSImage {
        let width: CGFloat = 22
        let height: CGFloat = 16
        let image = NSImage(size: NSSize(width: width, height: height))

        image.lockFocus()

        let rect = NSRect(x: 0, y: 0, width: width, height: height)
        let path = NSBezierPath(roundedRect: rect, xRadius: 3, yRadius: 3)
        NSColor.white.setFill()
        path.fill()

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
        image.isTemplate = false
        return image
    }

    // MARK: - Menu Panel (No Arrow)

    @objc private func toggleMenu(_ sender: NSStatusBarButton) {
        if menuPanel?.isVisible == true {
            closeMenu()
        } else {
            showMenu()
        }
    }

    private func showMenu() {
        guard let button = statusItem.button,
              let buttonWindow = button.window else { return }

        // Create panel if needed
        if menuPanel == nil {
            let menuView = MenuPopoverView(
                state: menuState,
                onClose: { [weak self] in self?.closeMenu() }
            )

            let hostingController = NSHostingController(rootView: menuView)
            let contentSize = hostingController.view.fittingSize

            let panel = NSPanel(
                contentRect: NSRect(origin: .zero, size: contentSize),
                styleMask: [.nonactivatingPanel, .fullSizeContentView],
                backing: .buffered,
                defer: false
            )
            panel.isOpaque = false
            panel.backgroundColor = .clear
            panel.hasShadow = true
            panel.level = .popUpMenu
            panel.collectionBehavior = [.canJoinAllSpaces, .fullScreenAuxiliary]
            panel.contentViewController = hostingController

            // Add visual effect background with proper clipping
            let containerView = NSView(frame: NSRect(origin: .zero, size: contentSize))
            containerView.wantsLayer = true
            containerView.layer?.cornerRadius = 8
            containerView.layer?.masksToBounds = true

            let visualEffect = NSVisualEffectView(frame: containerView.bounds)
            visualEffect.material = .popover
            visualEffect.blendingMode = .behindWindow
            visualEffect.state = .active
            visualEffect.autoresizingMask = [.width, .height]

            containerView.addSubview(visualEffect)

            hostingController.view.frame = containerView.bounds
            hostingController.view.autoresizingMask = [.width, .height]
            containerView.addSubview(hostingController.view)

            panel.contentView = containerView

            menuPanel = panel
        }

        // Position below status item
        let buttonRect = button.convert(button.bounds, to: nil)
        let screenRect = buttonWindow.convertToScreen(buttonRect)
        let panelSize = menuPanel!.frame.size

        let x = screenRect.midX - panelSize.width / 2
        let y = screenRect.minY - panelSize.height - 4

        menuPanel?.setFrameOrigin(NSPoint(x: x, y: y))
        menuPanel?.makeKeyAndOrderFront(nil)

        // Monitor clicks outside (global events use screen coordinates)
        eventMonitor = NSEvent.addGlobalMonitorForEvents(matching: [.leftMouseDown, .rightMouseDown]) { [weak self] event in
            guard let panel = self?.menuPanel else { return }
            let clickLocation = NSEvent.mouseLocation
            if !panel.frame.contains(clickLocation) {
                self?.closeMenu()
            }
        }

        // Close when app loses focus
        appDeactivateObserver = NotificationCenter.default.addObserver(
            forName: NSApplication.didResignActiveNotification,
            object: nil,
            queue: .main
        ) { [weak self] _ in
            self?.closeMenu()
        }
    }

    private func closeMenu() {
        menuPanel?.orderOut(nil)
        if let monitor = eventMonitor {
            NSEvent.removeMonitor(monitor)
            eventMonitor = nil
        }
        if let observer = appDeactivateObserver {
            NotificationCenter.default.removeObserver(observer)
            appDeactivateObserver = nil
        }
    }

    // MARK: - Event Handlers

    @objc private func handleToggleVietnamese() {
        menuState.toggle()
    }

    @objc private func handleMenuStateChanged() {
        menuState.load()
        updateStatusButton()
    }

    @objc private func onboardingDidComplete() {
        loadSettings()
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
            settingsWindow = window
        }
        setupMainMenu()
        NSApp.activate(ignoringOtherApps: true)
        settingsWindow?.makeKeyAndOrderFront(nil)
    }

    private func setupMainMenu() {
        let mainMenu = NSMenu()

        // App menu (required for ⌘Q to work)
        let appMenu = NSMenu()
        let appMenuItem = NSMenuItem()
        appMenuItem.submenu = appMenu

        // Settings (⌘,)
        let settingsItem = NSMenuItem(
            title: "Cài đặt...",
            action: #selector(showSettings),
            keyEquivalent: ","
        )
        settingsItem.target = self
        appMenu.addItem(settingsItem)

        appMenu.addItem(NSMenuItem.separator())

        // Quit (⌘Q)
        let quitItem = NSMenuItem(
            title: "Thoát \(AppMetadata.name)",
            action: #selector(NSApplication.terminate(_:)),
            keyEquivalent: "q"
        )
        appMenu.addItem(quitItem)

        mainMenu.addItem(appMenuItem)
        NSApp.mainMenu = mainMenu
    }

    @objc private func checkForUpdates() {
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

        // Skip re-check if update is already available (from auto-check)
        if case .available = UpdateManager.shared.state { return }
        UpdateManager.shared.checkForUpdatesManually()
    }
}
