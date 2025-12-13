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
}

// MARK: - Menu Popover View (Minimal)

struct MenuPopoverView: View {
    @ObservedObject var state: MenuState
    let closeMenu: () -> Void
    let openSettings: () -> Void

    var body: some View {
        VStack(spacing: 0) {
            // Header
            HStack(spacing: 10) {
                Image(nsImage: AppMetadata.logo)
                    .resizable()
                    .frame(width: 32, height: 32)

                VStack(alignment: .leading, spacing: 2) {
                    Text(AppMetadata.name)
                        .font(.system(size: 13, weight: .semibold))
                    HStack(spacing: 4) {
                        Text(state.isEnabled ? state.currentMethod.name : "Đã tắt")
                        Text("·")
                            .foregroundColor(Color(NSColor.tertiaryLabelColor))
                        Text("⌃Space")
                            .foregroundColor(Color(NSColor.tertiaryLabelColor))
                    }
                    .font(.system(size: 11))
                    .foregroundColor(Color(NSColor.secondaryLabelColor))
                }

                Spacer()

                Toggle("", isOn: Binding(
                    get: { state.isEnabled },
                    set: { _ in state.toggle() }
                ))
                .toggleStyle(.switch)
                .labelsHidden()
                .scaleEffect(0.8)
            }
            .padding(.horizontal, 12)
            .padding(.vertical, 10)

            Divider()

            // Method selection
            VStack(spacing: 0) {
                simpleMethodRow(method: .telex)
                simpleMethodRow(method: .vni)
            }
            .padding(.vertical, 4)

            Divider()

            // Menu items
            VStack(spacing: 0) {
                menuItem(title: "Cài đặt...", shortcut: "⌘,") {
                    closeMenu()
                    openSettings()
                }
                menuItem(title: "Kiểm tra cập nhật", shortcut: nil) {
                    closeMenu()
                    NotificationCenter.default.post(name: .showUpdateWindow, object: nil)
                }
            }
            .padding(.vertical, 4)

            Divider()

            // Quit
            menuItem(title: "Thoát", shortcut: "⌘Q") {
                NSApp.terminate(nil)
            }
            .padding(.vertical, 4)
        }
        .frame(width: 220)
    }

    private func simpleMethodRow(method: InputMode) -> some View {
        Button {
            state.setMethod(method)
        } label: {
            HStack(spacing: 8) {
                Image(systemName: state.currentMethod == method ? "checkmark" : "")
                    .font(.system(size: 11, weight: .medium))
                    .frame(width: 14)
                    .foregroundColor(Color(NSColor.labelColor))
                Text(method.name)
                    .font(.system(size: 13))
                    .foregroundColor(Color(NSColor.labelColor))
                Spacer()
            }
            .padding(.horizontal, 12)
            .padding(.vertical, 6)
            .contentShape(Rectangle())
        }
        .buttonStyle(MenuItemButtonStyle())
    }

    private func menuItem(title: String, shortcut: String?, action: @escaping () -> Void) -> some View {
        Button(action: action) {
            HStack {
                Text(title)
                    .font(.system(size: 13))
                    .foregroundColor(Color(NSColor.labelColor))
                Spacer()
                if let shortcut = shortcut {
                    Text(shortcut)
                        .font(.system(size: 11))
                        .foregroundColor(Color(NSColor.tertiaryLabelColor))
                }
            }
            .padding(.horizontal, 12)
            .padding(.vertical, 6)
            .contentShape(Rectangle())
        }
        .buttonStyle(MenuItemButtonStyle())
    }
}

// MARK: - Menu Item Button Style (System-like hover)

struct MenuItemButtonStyle: ButtonStyle {
    @State private var isHovered = false

    func makeBody(configuration: Configuration) -> some View {
        configuration.label
            .background(
                RoundedRectangle(cornerRadius: 4)
                    .fill(isHovered || configuration.isPressed ? Color.accentColor : Color.clear)
            )
            .foregroundColor(isHovered || configuration.isPressed ? .white : nil)
            .onHover { isHovered = $0 }
    }
}

// MARK: - Menu Bar Controller

class MenuBarController: NSObject {
    private var statusItem: NSStatusItem!
    private var menuPanel: NSPanel?
    private var eventMonitor: Any?

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

        DispatchQueue.main.asyncAfter(deadline: .now() + 3) {
            UpdateManager.shared.checkForUpdatesSilently()
        }
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
                closeMenu: { [weak self] in self?.closeMenu() },
                openSettings: { [weak self] in self?.showSettings() }
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

            // Add visual effect background
            let visualEffect = NSVisualEffectView(frame: NSRect(origin: .zero, size: contentSize))
            visualEffect.material = .menu
            visualEffect.state = .active
            visualEffect.wantsLayer = true
            visualEffect.layer?.cornerRadius = 8
            visualEffect.layer?.masksToBounds = true
            visualEffect.layer?.borderWidth = 0.5
            visualEffect.layer?.borderColor = NSColor.separatorColor.withAlphaComponent(0.5).cgColor

            hostingController.view.wantsLayer = true
            hostingController.view.layer?.cornerRadius = 8
            hostingController.view.layer?.masksToBounds = true

            panel.contentView = visualEffect
            visualEffect.addSubview(hostingController.view)
            hostingController.view.frame = visualEffect.bounds

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

        // Monitor clicks outside
        eventMonitor = NSEvent.addGlobalMonitorForEvents(matching: [.leftMouseDown, .rightMouseDown]) { [weak self] event in
            if let panel = self?.menuPanel, !panel.frame.contains(event.locationInWindow) {
                self?.closeMenu()
            }
        }
    }

    private func closeMenu() {
        menuPanel?.orderOut(nil)
        if let monitor = eventMonitor {
            NSEvent.removeMonitor(monitor)
            eventMonitor = nil
        }
    }

    // MARK: - Event Handlers

    @objc private func handleToggleVietnamese() {
        menuState.toggle()
    }

    @objc private func handleMenuStateChanged() {
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
        UpdateManager.shared.checkForUpdatesManually()
    }
}
