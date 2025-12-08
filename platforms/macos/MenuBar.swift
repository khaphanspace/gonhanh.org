import Cocoa
import SwiftUI

// MARK: - SwiftUI Toggle Wrapper

class ToggleState: ObservableObject {
    @Published var isOn: Bool
    var onToggle: (() -> Void)?

    init(isOn: Bool) {
        self.isOn = isOn
    }
}

struct ToggleWrapperView: View {
    @ObservedObject var state: ToggleState

    var body: some View {
        Toggle("", isOn: $state.isOn)
            .toggleStyle(.switch)
            .tint(.green)
            .onChange(of: state.isOn) { _ in
                state.onToggle?()
            }
            .labelsHidden()
    }
}

// MARK: - Menu Bar Controller

class MenuBarController {
    private var statusItem: NSStatusItem!
    private var onboardingWindow: NSWindow?
    private var aboutWindow: NSWindow?
    private var updateWindow: NSWindow?
    private var toggleState: ToggleState?

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
    }

    private func startEngine() {
        RustBridge.initialize()
        KeyboardHookManager.shared.start()
        RustBridge.setEnabled(isEnabled)
        RustBridge.setMethod(currentMethod.rawValue)

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
        let width: CGFloat = 22
        let height: CGFloat = 16
        let image = NSImage(size: NSSize(width: width, height: height))

        image.lockFocus()

        // Background trắng bo góc nhẹ
        let rect = NSRect(x: 0, y: 0, width: width, height: height)
        let path = NSBezierPath(roundedRect: rect, xRadius: 3, yRadius: 3)
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
            x: (width - textSize.width) / 2,
            y: (height - textSize.height) / 2,
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

        // Toggle using SwiftUI for custom tint color
        if toggleState == nil {
            toggleState = ToggleState(isOn: isEnabled)
            toggleState?.onToggle = { [weak self] in self?.handleToggle() }
        }
        toggleState?.isOn = isEnabled
        let toggleView = ToggleWrapperView(state: toggleState!)
        let hostingView = NSHostingView(rootView: toggleView)
        hostingView.frame = NSRect(x: 162, y: 4, width: 50, height: 28)
        view.addSubview(hostingView)

        return view
    }

    private func updateMenu() {
        guard let menu = statusItem.menu else { return }
        menu.item(withTag: 1)?.view = createHeaderView()
        menu.item(withTag: 10)?.state = currentMethod == .telex ? .on : .off
        menu.item(withTag: 11)?.state = currentMethod == .vni ? .on : .off
    }

    // MARK: - Actions

    private func handleToggle() {
        isEnabled = toggleState?.isOn ?? !isEnabled
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

        // Trigger update check when window opens
        UpdateManager.shared.checkForUpdatesManually()
    }
}
