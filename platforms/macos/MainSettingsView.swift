import SwiftUI
import UniformTypeIdentifiers
import Combine
import AppKit

// MARK: - Sound Manager

class SoundManager {
    static let shared = SoundManager()

    private init() {}

    func playToggleSound(enabled: Bool) {
        guard AppState.shared.soundEnabled else { return }
        // Use different system sounds for on/off states
        // "Tink" for enabling Vietnamese, "Pop" for disabling
        let soundName = enabled ? "Tink" : "Pop"
        NSSound(named: NSSound.Name(soundName))?.play()
    }
}

// MARK: - Navigation

enum NavigationPage: String, CaseIterable {
    case settings = "Cài đặt"
    case about = "Giới thiệu"

    var icon: String {
        switch self {
        case .settings: return "gearshape"
        case .about: return "bolt.fill"
        }
    }
}

// MARK: - Update Status

enum UpdateStatus: Equatable {
    case idle, checking, upToDate, available(String), error

    var isChecking: Bool { if case .checking = self { return true }; return false }
    var isAvailable: Bool { if case .available = self { return true }; return false }
}

// MARK: - App State

class AppState: ObservableObject {
    static let shared = AppState()

    private var isSilentUpdate = false
    private var cancellables = Set<AnyCancellable>()
    private var launchAtLoginTimer: Timer?

    @Published var isEnabled: Bool {
        didSet {
            RustBridge.setEnabled(isEnabled)
            NotificationCenter.default.post(name: .menuStateChanged, object: nil)
            guard !isSilentUpdate else { return }
            UserDefaults.standard.set(isEnabled, forKey: SettingsKey.enabled)
            if isSmartModeEnabled {
                // Check if a special panel app (Spotlight, Raycast) is currently focused
                if let activePanelApp = SpecialPanelAppDetector.getActiveSpecialPanelApp() {
                    savePerAppMode(bundleId: activePanelApp, enabled: isEnabled)
                } else if let bundleId = NSWorkspace.shared.frontmostApplication?.bundleIdentifier {
                    savePerAppMode(bundleId: bundleId, enabled: isEnabled)
                }
            }
        }
    }

    @Published var currentMethod: InputMode {
        didSet {
            UserDefaults.standard.set(currentMethod.rawValue, forKey: SettingsKey.method)
            RustBridge.setMethod(currentMethod.rawValue)
            NotificationCenter.default.post(name: .menuStateChanged, object: nil)
        }
    }

    @Published var isSmartModeEnabled: Bool = true {
        didSet { UserDefaults.standard.set(isSmartModeEnabled, forKey: SettingsKey.smartModeEnabled) }
    }

    @Published var autoWShortcut: Bool = true {
        didSet {
            UserDefaults.standard.set(autoWShortcut, forKey: SettingsKey.autoWShortcut)
            RustBridge.setSkipWShortcut(!autoWShortcut)
        }
    }

    @Published var escRestore: Bool = false {
        didSet {
            UserDefaults.standard.set(escRestore, forKey: SettingsKey.escRestore)
            RustBridge.setEscRestore(escRestore)
        }
    }

    @Published var modernTone: Bool = true {
        didSet {
            UserDefaults.standard.set(modernTone, forKey: SettingsKey.modernTone)
            RustBridge.setModernTone(modernTone)
        }
    }

    @Published var englishAutoRestore: Bool = false {
        didSet {
            UserDefaults.standard.set(englishAutoRestore, forKey: SettingsKey.englishAutoRestore)
            RustBridge.setEnglishAutoRestore(englishAutoRestore)
        }
    }

    @Published var soundEnabled: Bool = false {
        didSet {
            UserDefaults.standard.set(soundEnabled, forKey: SettingsKey.soundEnabled)
        }
    }

    @Published var toggleShortcut: KeyboardShortcut {
        didSet {
            toggleShortcut.save()
            NotificationCenter.default.post(name: .shortcutChanged, object: toggleShortcut)
        }
    }

    @Published var updateStatus: UpdateStatus = .idle
    @Published var shortcuts: [ShortcutItem] = []
    @Published var isLaunchAtLoginEnabled: Bool = false
    @Published var requiresManualLaunchAtLogin: Bool = false

    // MARK: - Init

    init() {
        isEnabled = UserDefaults.standard.object(forKey: SettingsKey.enabled) as? Bool ?? true
        currentMethod = InputMode(rawValue: UserDefaults.standard.integer(forKey: SettingsKey.method)) ?? .telex
        toggleShortcut = KeyboardShortcut.load()

        loadSmartMode()
        loadAutoWShortcut()
        loadEscRestore()
        loadModernTone()
        loadEnglishAutoRestore()
        loadSoundEnabled()
        loadShortcuts()
        setupObservers()
        setupLaunchAtLoginMonitoring()
        checkForUpdates()
    }

    private func loadSmartMode() {
        if UserDefaults.standard.object(forKey: SettingsKey.smartModeEnabled) == nil {
            isSmartModeEnabled = true
            UserDefaults.standard.set(true, forKey: SettingsKey.smartModeEnabled)
        } else {
            isSmartModeEnabled = UserDefaults.standard.bool(forKey: SettingsKey.smartModeEnabled)
        }
    }

    private func loadAutoWShortcut() {
        if UserDefaults.standard.object(forKey: SettingsKey.autoWShortcut) == nil {
            autoWShortcut = true
            UserDefaults.standard.set(true, forKey: SettingsKey.autoWShortcut)
        } else {
            autoWShortcut = UserDefaults.standard.bool(forKey: SettingsKey.autoWShortcut)
        }
        RustBridge.setSkipWShortcut(!autoWShortcut)
    }

    private func loadEscRestore() {
        if UserDefaults.standard.object(forKey: SettingsKey.escRestore) == nil {
            escRestore = false
            UserDefaults.standard.set(false, forKey: SettingsKey.escRestore)
        } else {
            escRestore = UserDefaults.standard.bool(forKey: SettingsKey.escRestore)
        }
        RustBridge.setEscRestore(escRestore)
    }

    private func loadModernTone() {
        if UserDefaults.standard.object(forKey: SettingsKey.modernTone) == nil {
            modernTone = true
            UserDefaults.standard.set(true, forKey: SettingsKey.modernTone)
        } else {
            modernTone = UserDefaults.standard.bool(forKey: SettingsKey.modernTone)
        }
        RustBridge.setModernTone(modernTone)
    }

    private func loadEnglishAutoRestore() {
        if UserDefaults.standard.object(forKey: SettingsKey.englishAutoRestore) == nil {
            englishAutoRestore = false
            UserDefaults.standard.set(false, forKey: SettingsKey.englishAutoRestore)
        } else {
            englishAutoRestore = UserDefaults.standard.bool(forKey: SettingsKey.englishAutoRestore)
        }
        RustBridge.setEnglishAutoRestore(englishAutoRestore)
    }

    private func loadSoundEnabled() {
        if UserDefaults.standard.object(forKey: SettingsKey.soundEnabled) == nil {
            soundEnabled = false
            UserDefaults.standard.set(false, forKey: SettingsKey.soundEnabled)
        } else {
            soundEnabled = UserDefaults.standard.bool(forKey: SettingsKey.soundEnabled)
        }
    }

    private func loadShortcuts() {
        if let data = UserDefaults.standard.data(forKey: SettingsKey.shortcuts),
           let saved = try? JSONDecoder().decode([ShortcutItem].self, from: data) {
            shortcuts = saved
        } else {
            shortcuts = [
                ShortcutItem(key: "vn", value: "Việt Nam", isEnabled: false),
                ShortcutItem(key: "hn", value: "Hà Nội", isEnabled: false),
                ShortcutItem(key: "hcm", value: "Hồ Chí Minh", isEnabled: false),
                ShortcutItem(key: "tphcm", value: "Thành phố Hồ Chí Minh", isEnabled: false),
            ]
        }
    }

    // MARK: - Observers

    private func setupObservers() {
        $shortcuts
            .dropFirst()
            .debounce(for: .milliseconds(300), scheduler: RunLoop.main)
            .sink { [weak self] shortcuts in
                let validShortcuts = shortcuts.filter { !$0.key.isEmpty && !$0.value.isEmpty }
                self?.syncShortcutsToEngine(validShortcuts)
                if let data = try? JSONEncoder().encode(shortcuts) {
                    UserDefaults.standard.set(data, forKey: SettingsKey.shortcuts)
                }
            }
            .store(in: &cancellables)
    }

    // MARK: - Launch at Login

    private func setupLaunchAtLoginMonitoring() {
        isLaunchAtLoginEnabled = LaunchAtLoginManager.shared.isEnabled

        // Only auto-enable on first launch (when user hasn't configured yet)
        let hasConfigured = UserDefaults.standard.bool(forKey: SettingsKey.launchAtLoginConfigured)
        if !hasConfigured && !isLaunchAtLoginEnabled {
            autoEnableLaunchAtLogin()
            UserDefaults.standard.set(true, forKey: SettingsKey.launchAtLoginConfigured)
        }

        launchAtLoginTimer = Timer.scheduledTimer(withTimeInterval: 2.0, repeats: true) { [weak self] _ in
            DispatchQueue.main.async { self?.refreshLaunchAtLoginStatus() }
        }
    }

    private func autoEnableLaunchAtLogin() {
        do {
            try LaunchAtLoginManager.shared.enable()
            refreshLaunchAtLoginStatus()
            requiresManualLaunchAtLogin = false
        } catch {
            // Auto-enable failed, user needs to manually enable
            requiresManualLaunchAtLogin = true
        }
    }

    func refreshLaunchAtLoginStatus() {
        let newStatus = LaunchAtLoginManager.shared.isEnabled
        if newStatus != isLaunchAtLoginEnabled { isLaunchAtLoginEnabled = newStatus }
        // Clear manual requirement flag if now enabled
        if isLaunchAtLoginEnabled { requiresManualLaunchAtLogin = false }
    }

    func enableLaunchAtLogin() {
        do {
            try LaunchAtLoginManager.shared.enable()
            refreshLaunchAtLoginStatus()
            requiresManualLaunchAtLogin = false
        } catch {
            requiresManualLaunchAtLogin = true
            openLoginItemsSettings()
        }
    }

    func disableLaunchAtLogin() {
        do {
            try LaunchAtLoginManager.shared.disable()
            refreshLaunchAtLoginStatus()
        } catch {
            // If disable fails, open settings for manual action
            openLoginItemsSettings()
        }
    }

    func toggleLaunchAtLogin(_ enabled: Bool) {
        if enabled {
            enableLaunchAtLogin()
        } else {
            disableLaunchAtLogin()
        }
    }

    func openLoginItemsSettings() {
        if let url = URL(string: "x-apple.systempreferences:com.apple.LoginItems-Settings.extension") {
            NSWorkspace.shared.open(url)
        }
    }

    // MARK: - Per-App Mode

    func savePerAppMode(bundleId: String, enabled: Bool) {
        var modes = UserDefaults.standard.dictionary(forKey: SettingsKey.perAppModes) as? [String: Bool] ?? [:]
        if enabled { modes.removeValue(forKey: bundleId) } else { modes[bundleId] = false }
        UserDefaults.standard.set(modes, forKey: SettingsKey.perAppModes)
    }

    func getPerAppMode(bundleId: String) -> Bool {
        let modes = UserDefaults.standard.dictionary(forKey: SettingsKey.perAppModes) as? [String: Bool] ?? [:]
        return modes[bundleId] ?? true
    }

    func setEnabledSilently(_ enabled: Bool) {
        isSilentUpdate = true
        isEnabled = enabled
        isSilentUpdate = false
    }

    func toggle() { isEnabled.toggle() }
    func setMethod(_ method: InputMode) { currentMethod = method }

    // MARK: - Shortcuts

    func syncShortcutsToEngine(_ validShortcuts: [ShortcutItem]? = nil) {
        let toSync = validShortcuts ?? shortcuts.filter { !$0.key.isEmpty && !$0.value.isEmpty }
        RustBridge.syncShortcuts(toSync.map { ($0.key, $0.value, $0.isEnabled) })
    }

    func exportShortcuts() -> String {
        var lines = [";Gõ Nhanh - Bảng gõ tắt"]
        for shortcut in shortcuts where !shortcut.key.isEmpty {
            lines.append("\(shortcut.key):\(shortcut.value)")
        }
        return lines.joined(separator: "\n")
    }

    func importShortcuts(from content: String) -> Int {
        let lines = content.components(separatedBy: .newlines)
        var imported = 0
        for line in lines {
            let trimmed = line.trimmingCharacters(in: .whitespaces)
            guard !trimmed.isEmpty, !trimmed.hasPrefix(";"),
                  let colonIndex = trimmed.firstIndex(of: ":") else { continue }
            let trigger = String(trimmed[..<colonIndex]).trimmingCharacters(in: .whitespaces)
            let replacement = String(trimmed[trimmed.index(after: colonIndex)...]).trimmingCharacters(in: .whitespaces)
            guard !trigger.isEmpty else { continue }
            if let idx = shortcuts.firstIndex(where: { $0.key == trigger }) {
                shortcuts[idx].value = replacement
                shortcuts[idx].isEnabled = true
            } else {
                shortcuts.append(ShortcutItem(key: trigger, value: replacement, isEnabled: true))
            }
            imported += 1
        }
        return imported
    }

    // MARK: - Updates

    func checkForUpdates() {
        updateStatus = .checking
        let startTime = Date()
        UpdateChecker.shared.checkForUpdates { [weak self] result in
            let elapsed = Date().timeIntervalSince(startTime)
            let delay = max(0, 1.5 - elapsed)
            DispatchQueue.main.asyncAfter(deadline: .now() + delay) {
                switch result {
                case .available(let info): self?.updateStatus = .available(info.version)
                case .upToDate: self?.updateStatus = .upToDate
                case .error: self?.updateStatus = .error
                }
            }
        }
    }
}

// MARK: - Models

struct ShortcutItem: Identifiable, Codable {
    var id = UUID()
    var key: String
    var value: String
    var isEnabled: Bool = true
}

// MARK: - View Modifiers

struct CardBackground: ViewModifier {
    func body(content: Content) -> some View {
        content
            .background(RoundedRectangle(cornerRadius: 10).fill(Color(NSColor.controlBackgroundColor).opacity(0.5)))
            .overlay(RoundedRectangle(cornerRadius: 10).stroke(Color(NSColor.separatorColor).opacity(0.5), lineWidth: 0.5))
    }
}

extension View {
    func cardBackground() -> some View { modifier(CardBackground()) }
}

// MARK: - Reusable Components

struct SettingsRow<Content: View>: View {
    let content: Content
    init(@ViewBuilder content: () -> Content) { self.content = content() }
    var body: some View {
        HStack { content }
            .padding(.horizontal, 12)
            .padding(.vertical, 10)
    }
}

struct SettingsToggleRow: View {
    let title: String
    let subtitle: String?
    @Binding var isOn: Bool

    init(_ title: String, subtitle: String? = nil, isOn: Binding<Bool>) {
        self.title = title
        self.subtitle = subtitle
        self._isOn = isOn
    }

    var body: some View {
        SettingsRow {
            VStack(alignment: .leading, spacing: 2) {
                Text(title).font(.system(size: 13))
                if let subtitle = subtitle {
                    Text(subtitle).font(.system(size: 11)).foregroundColor(Color(NSColor.secondaryLabelColor))
                }
            }
            Spacer()
            Toggle("", isOn: $isOn).toggleStyle(.switch).labelsHidden()
        }
    }
}

struct KeyCap: View {
    let text: String
    var body: some View {
        Text(text)
            .font(.system(size: 11, weight: .medium))
            .foregroundColor(Color(NSColor.secondaryLabelColor))
            .padding(.horizontal, 6)
            .padding(.vertical, 3)
            .background(RoundedRectangle(cornerRadius: 4).fill(Color(NSColor.controlBackgroundColor).opacity(0.8)))
            .overlay(RoundedRectangle(cornerRadius: 4).stroke(Color(NSColor.separatorColor).opacity(0.5), lineWidth: 0.5))
    }
}

struct ClickableTextField: NSViewRepresentable {
    @Binding var text: String

    func makeNSView(context: Context) -> NSTextField {
        let textField = NSTextField()
        textField.isBordered = false
        textField.drawsBackground = false
        textField.focusRingType = .none
        textField.font = .systemFont(ofSize: 13)
        textField.delegate = context.coordinator
        textField.cell?.lineBreakMode = .byTruncatingTail
        return textField
    }

    func updateNSView(_ nsView: NSTextField, context: Context) {
        if nsView.stringValue != text { nsView.stringValue = text }
    }

    func makeCoordinator() -> Coordinator { Coordinator(self) }

    class Coordinator: NSObject, NSTextFieldDelegate {
        var parent: ClickableTextField
        init(_ parent: ClickableTextField) { self.parent = parent }
        func controlTextDidChange(_ obj: Notification) {
            guard let textField = obj.object as? NSTextField else { return }
            parent.text = textField.stringValue
        }
    }
}

struct VisualEffectBackground: NSViewRepresentable {
    var material: NSVisualEffectView.Material = .sidebar
    var blendingMode: NSVisualEffectView.BlendingMode = .behindWindow

    func makeNSView(context: Context) -> NSVisualEffectView {
        let view = NSVisualEffectView()
        view.material = material
        view.blendingMode = blendingMode
        view.state = .active
        return view
    }

    func updateNSView(_ nsView: NSVisualEffectView, context: Context) {
        nsView.material = material
        nsView.blendingMode = blendingMode
    }
}

// MARK: - Main Settings View

struct MainSettingsView: View {
    @ObservedObject private var appState = AppState.shared
    @State private var selectedPage: NavigationPage = .settings
    @Environment(\.colorScheme) private var colorScheme

    var body: some View {
        HStack(spacing: 0) {
            ZStack {
                VisualEffectBackground(material: .sidebar, blendingMode: .behindWindow)
                sidebar
            }
            .frame(width: 200)

            ZStack {
                if colorScheme == .dark {
                    VisualEffectBackground(material: .headerView, blendingMode: .behindWindow)
                } else {
                    Color(NSColor.windowBackgroundColor)
                }
                content
            }
        }
        .ignoresSafeArea()
        .frame(width: 700, height: 480)
        .onReceive(NotificationCenter.default.publisher(for: .showSettingsPage)) { notification in
            if let page = notification.object as? NavigationPage { selectedPage = page }
        }
    }

    // MARK: - Sidebar

    private var sidebar: some View {
        VStack(spacing: 0) {
            VStack(spacing: 12) {
                Image(nsImage: AppMetadata.logo).resizable().frame(width: 96, height: 96)
                Text(AppMetadata.name).font(.system(size: 20, weight: .bold))
                UpdateBadgeView(status: appState.updateStatus) { appState.checkForUpdates() }
            }
            .padding(.top, 40)

            Spacer()

            VStack(spacing: 4) {
                ForEach(NavigationPage.allCases, id: \.self) { page in
                    NavButton(page: page, isSelected: selectedPage == page) { selectedPage = page }
                }
            }
            .padding(.horizontal, 12)
            .padding(.bottom, 20)
        }
    }

    // MARK: - Content

    @ViewBuilder
    private var content: some View {
        switch selectedPage {
        case .settings:
            ScrollView(showsIndicators: false) {
                SettingsPageView(appState: appState).padding(28)
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)
        case .about:
            AboutPageView().padding(28).frame(maxWidth: .infinity, maxHeight: .infinity)
        }
    }
}

// MARK: - Update Badge

struct UpdateBadgeView: View {
    let status: UpdateStatus
    let onCheck: () -> Void
    @State private var hovered = false
    @State private var rotation: Double = 0

    private var statusText: String? {
        switch status {
        case .idle: return nil
        case .checking: return "Kiểm tra"
        case .upToDate: return "Mới nhất"
        case .available: return "Cập nhật"
        case .error: return "Thất bại"
        }
    }

    private var statusIcon: (name: String, color: Color)? {
        switch status {
        case .upToDate: return ("checkmark.circle.fill", .green)
        case .available: return ("arrow.up.circle.fill", .orange)
        case .error: return ("exclamationmark.triangle.fill", .orange)
        default: return nil
        }
    }

    var body: some View {
        HStack(spacing: 3) {
            Text("v\(AppMetadata.version)")
            if status.isChecking {
                Image(systemName: "arrow.clockwise.circle.fill")
                    .font(.system(size: 12))
                    .foregroundColor(.secondary)
                    .rotationEffect(.degrees(rotation))
                    .onAppear { withAnimation(.linear(duration: 1).repeatForever(autoreverses: false)) { rotation = 360 } }
                    .onDisappear { rotation = 0 }
            } else if let icon = statusIcon {
                Image(systemName: icon.name).font(.system(size: 12)).foregroundColor(icon.color)
            }
            if let text = statusText { Text(text) }
        }
        .font(.system(size: 11))
        .foregroundColor(Color(NSColor.tertiaryLabelColor))
        .padding(.horizontal, 8)
        .padding(.vertical, 4)
        .background(Capsule().fill(hovered ? Color(NSColor.controlBackgroundColor).opacity(0.5) : Color.clear))
        .onHover { h in
            hovered = h
            if status.isAvailable { if h { NSCursor.pointingHand.push() } else { NSCursor.pop() } }
        }
        .onTapGesture {
            guard !status.isChecking else { return }
            if status.isAvailable {
                if case .available(let info) = UpdateManager.shared.state {
                    UpdateManager.shared.downloadUpdate(info)
                    NotificationCenter.default.post(name: .showUpdateWindow, object: nil)
                }
            } else { onCheck() }
        }
    }
}

// MARK: - Nav Button

struct NavButton: View {
    let page: NavigationPage
    let isSelected: Bool
    let action: () -> Void
    @State private var hovered = false

    var body: some View {
        HStack(spacing: 10) {
            Image(systemName: page.icon)
                .font(.system(size: 14))
                .foregroundColor(isSelected ? Color(NSColor.labelColor) : Color(NSColor.secondaryLabelColor))
                .frame(width: 20)
            Text(page.rawValue)
                .font(.system(size: 13))
                .foregroundColor(isSelected ? Color(NSColor.labelColor) : Color(NSColor.secondaryLabelColor))
            Spacer()
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 8)
        .background(
            RoundedRectangle(cornerRadius: 8)
                .fill(isSelected ? Color(NSColor.controlBackgroundColor).opacity(0.6) :
                      hovered ? Color(NSColor.controlBackgroundColor).opacity(0.4) : Color.clear)
        )
        .contentShape(Rectangle())
        .onHover { hovered = $0 }
        .onTapGesture { action() }
    }
}

// MARK: - Settings Page

struct SettingsPageView: View {
    @ObservedObject var appState: AppState
    @State private var isRecordingShortcut = false
    @State private var showShortcutsSheet = false

    var body: some View {
        VStack(alignment: .leading, spacing: 20) {
            // Input method settings
            VStack(spacing: 0) {
                SettingsToggleRow("Bộ gõ tiếng Việt", isOn: $appState.isEnabled)
                Divider().padding(.leading, 12)
                inputMethodRow
                if appState.currentMethod == .telex {
                    Divider().padding(.leading, 12)
                    SettingsToggleRow("Tự chuyển W → Ư ở đầu từ",
                                      subtitle: "Gõ 'w' đầu từ sẽ thành 'ư'",
                                      isOn: $appState.autoWShortcut)
                }
            }
            .cardBackground()

            // Toggle shortcut section
            VStack(spacing: 0) {
                ShortcutRecorderRow(shortcut: $appState.toggleShortcut,
                                    isRecording: $isRecordingShortcut)
            }
            .cardBackground()

            // Other options
            VStack(spacing: 0) {
                LaunchAtLoginToggleRow(appState: appState)
                Divider().padding(.leading, 12)
                SettingsToggleRow("Chuyển chế độ thông minh",
                                  subtitle: "Tự động nhớ trạng thái Anh/Việt cho từng ứng dụng",
                                  isOn: $appState.isSmartModeEnabled)
                Divider().padding(.leading, 12)
                SettingsToggleRow("Đặt dấu kiểu mới (oà, uý)",
                                  subtitle: "Thay vì kiểu cũ (òa, úy)",
                                  isOn: $appState.modernTone)
                Divider().padding(.leading, 12)
                SettingsToggleRow("Gõ ESC để chuyển về tiếng Anh",
                                  subtitle: "Nhấn ESC hoàn tác dấu tiếng Việt về ASCII",
                                  isOn: $appState.escRestore)
                Divider().padding(.leading, 12)
                SettingsToggleRow("Phát âm thanh khi chuyển ngôn ngữ",
                                  subtitle: "Phát tiếng khi bật/tắt gõ tiếng Việt",
                                  isOn: $appState.soundEnabled)
                Divider().padding(.leading, 12)
                shortcutsRow
            }
            .cardBackground()

            // Experimental features
            VStack(spacing: 0) {
                experimentalHeader
                Divider().padding(.leading, 12)
                SettingsToggleRow("Tự động khôi phục từ tiếng Anh",
                                  subtitle: "Tự nhận diện và khôi phục từ tiếng Anh khi gõ sai",
                                  isOn: $appState.englishAutoRestore)
            }
            .cardBackground()

            Spacer()
        }
        .sheet(isPresented: $showShortcutsSheet) { ShortcutsSheet(appState: appState) }
    }

    private var inputMethodRow: some View {
        SettingsRow {
            Text("Kiểu gõ").font(.system(size: 13))
            Spacer()
            Picker("", selection: $appState.currentMethod) {
                ForEach(InputMode.allCases, id: \.self) { Text($0.name).tag($0) }
            }
            .labelsHidden()
            .frame(width: 100)
        }
    }

    private var experimentalHeader: some View {
        SettingsRow {
            HStack(spacing: 8) {
                Text("Tính năng thử nghiệm").font(.system(size: 13, weight: .medium))
                Text("Beta")
                    .font(.system(size: 9, weight: .semibold))
                    .foregroundColor(.white)
                    .padding(.horizontal, 6)
                    .padding(.vertical, 2)
                    .background(Capsule().fill(Color.orange))
            }
            Spacer()
            Link(destination: URL(string: "https://github.com/khaphanspace/gonhanh.org/issues/26")!) {
                HStack(spacing: 4) {
                    Image(systemName: "bubble.left.and.bubble.right")
                    Text("Góp ý")
                }
                .font(.system(size: 11))
                .foregroundColor(.accentColor)
            }
            .buttonStyle(.plain)
        }
    }

    private var shortcutsRow: some View {
        Button(action: { showShortcutsSheet = true }) {
            HStack {
                VStack(alignment: .leading, spacing: 2) {
                    Text("Từ viết tắt").font(.system(size: 13)).foregroundColor(Color(NSColor.labelColor))
                    Text(appState.shortcuts.isEmpty
                         ? "Chưa có từ viết tắt"
                         : "\(appState.shortcuts.filter(\.isEnabled).count)/\(appState.shortcuts.count) đang bật")
                        .font(.system(size: 11))
                        .foregroundColor(Color(NSColor.secondaryLabelColor))
                }
                Spacer()
                Image(systemName: "chevron.right")
                    .font(.system(size: 12, weight: .medium))
                    .foregroundColor(Color(NSColor.tertiaryLabelColor))
            }
            .padding(.horizontal, 12)
            .padding(.vertical, 10)
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
    }
}

// MARK: - Shortcuts Sheet

struct ShortcutsSheet: View {
    @ObservedObject var appState: AppState
    @Environment(\.dismiss) private var dismiss

    // Form state
    @State private var formKey: String = ""
    @State private var formValue: String = ""
    @State private var editingId: UUID? = nil  // For form editing
    @State private var selectedIds: Set<UUID> = []  // For table multi-selection

    private var isEditing: Bool { editingId != nil }
    private var canSave: Bool { !formKey.isEmpty && !formValue.isEmpty }

    var body: some View {
        VStack(spacing: 0) {
            header
            Divider()
            formSection
            Divider()
            tableContent
            Divider()
            toolbar
        }
        .frame(width: 480, height: 420)
    }

    private var header: some View {
        HStack {
            VStack(alignment: .leading, spacing: 2) {
                Text("Từ viết tắt").font(.system(size: 15, weight: .semibold))
                Text("\(appState.shortcuts.count) mục").font(.system(size: 11)).foregroundColor(.secondary)
            }
            Spacer()
            Button("Xong") { dismiss() }.keyboardShortcut(.escape, modifiers: [])
        }
        .padding(.horizontal, 20)
        .padding(.vertical, 12)
    }

    private var formSection: some View {
        VStack(spacing: 10) {
            HStack(spacing: 10) {
                VStack(alignment: .leading, spacing: 3) {
                    Text("Viết tắt").font(.system(size: 11)).foregroundColor(.secondary)
                    TextField("vd: tphcm", text: $formKey)
                        .textFieldStyle(.roundedBorder)
                        .frame(width: 120)
                }
                VStack(alignment: .leading, spacing: 3) {
                    Text("Nội dung").font(.system(size: 11)).foregroundColor(.secondary)
                    TextField("vd: Thành phố Hồ Chí Minh", text: $formValue)
                        .textFieldStyle(.roundedBorder)
                }
            }
            HStack(spacing: 8) {
                if isEditing {
                    Button("Huỷ") { clearForm() }
                    Button("Xoá", role: .destructive) { deleteSelected() }
                        .foregroundColor(.red)
                }
                Spacer()
                Button(isEditing ? "Cập nhật" : "Thêm") { saveForm() }
                    .disabled(!canSave)
                    .keyboardShortcut(.return, modifiers: [])
            }
        }
        .padding(.horizontal, 20)
        .padding(.vertical, 12)
        .background(Color(NSColor.controlBackgroundColor).opacity(0.5))
    }

    @ViewBuilder
    private var tableContent: some View {
        if appState.shortcuts.isEmpty {
            VStack(spacing: 8) {
                Image(systemName: "text.badge.plus").font(.system(size: 32)).foregroundColor(.secondary)
                Text("Chưa có từ viết tắt").font(.system(size: 13)).foregroundColor(.secondary)
                Text("Điền form ở trên để thêm mới").font(.system(size: 11)).foregroundColor(Color(NSColor.tertiaryLabelColor))
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)
        } else {
            Table(appState.shortcuts, selection: $selectedIds) {
                TableColumn("") { item in
                    Toggle("", isOn: Binding(
                        get: { item.isEnabled },
                        set: { newValue in
                            if let idx = appState.shortcuts.firstIndex(where: { $0.id == item.id }) {
                                appState.shortcuts[idx].isEnabled = newValue
                            }
                        }
                    ))
                    .toggleStyle(.checkbox)
                    .labelsHidden()
                }
                .width(24)

                TableColumn("Viết tắt") { item in
                    Text(item.key)
                        .font(.system(size: 12, weight: .medium))
                        .foregroundColor(item.isEnabled ? .primary : .secondary)
                }
                .width(min: 80, ideal: 100, max: 140)

                TableColumn("Nội dung") { item in
                    Text(item.value)
                        .font(.system(size: 12))
                        .foregroundColor(item.isEnabled ? .primary : .secondary)
                        .lineLimit(1)
                }
            }
            .tableStyle(.inset(alternatesRowBackgrounds: true))
            .onDeleteCommand { deleteSelected() }
            .onChange(of: selectedIds) { newSelection in
                // Load single selection into form for editing
                if newSelection.count == 1, let id = newSelection.first,
                   let item = appState.shortcuts.first(where: { $0.id == id }) {
                    selectItem(item)
                } else {
                    clearForm()
                }
            }
        }
    }

    private var toolbar: some View {
        HStack(spacing: 8) {
            Button(action: importShortcuts) {
                Image(systemName: "square.and.arrow.down").frame(width: 20, height: 20)
            }
            .buttonStyle(.borderless).help("Nhập từ file")
            Button(action: exportShortcuts) {
                Image(systemName: "square.and.arrow.up").frame(width: 20, height: 20)
            }
            .buttonStyle(.borderless).disabled(appState.shortcuts.isEmpty).help("Xuất ra file")
            Spacer()
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 6)
    }

    private func selectItem(_ item: ShortcutItem) {
        editingId = item.id
        formKey = item.key
        formValue = item.value
    }

    private func clearForm() {
        editingId = nil
        formKey = ""
        formValue = ""
    }

    private func saveForm() {
        if let id = editingId, let idx = appState.shortcuts.firstIndex(where: { $0.id == id }) {
            // Update existing - only key and value, keep isEnabled unchanged
            appState.shortcuts[idx].key = formKey
            appState.shortcuts[idx].value = formValue
        } else {
            // Add new - default enabled
            appState.shortcuts.append(ShortcutItem(key: formKey, value: formValue, isEnabled: true))
        }
        clearForm()
    }

    private func deleteSelected() {
        guard !selectedIds.isEmpty else { return }
        appState.shortcuts.removeAll { selectedIds.contains($0.id) }
        selectedIds.removeAll()
        clearForm()
    }

    private func importShortcuts() {
        let panel = NSOpenPanel()
        panel.title = "Nhập từ viết tắt"
        panel.allowedContentTypes = [.plainText, .init(filenameExtension: "txt")!]
        panel.allowsMultipleSelection = false
        panel.canChooseDirectories = false
        if panel.runModal() == .OK, let url = panel.url,
           let content = try? String(contentsOf: url, encoding: .utf8) {
            _ = appState.importShortcuts(from: content)
        }
    }

    private func exportShortcuts() {
        let panel = NSSavePanel()
        panel.title = "Xuất từ viết tắt"
        panel.nameFieldStringValue = "gonhanh-shortcuts.txt"
        panel.allowedContentTypes = [.plainText]
        if panel.runModal() == .OK, let url = panel.url {
            try? appState.exportShortcuts().write(to: url, atomically: true, encoding: .utf8)
        }
    }
}

// MARK: - About Page

struct AboutPageView: View {
    var body: some View {
        VStack(spacing: 24) {
            Spacer()
            VStack(spacing: 12) {
                Image(nsImage: AppMetadata.logo).resizable().frame(width: 80, height: 80)
                Text(AppMetadata.name).font(.system(size: 20, weight: .bold))
                Text("Bộ gõ tiếng Việt nhanh và nhẹ").font(.system(size: 13)).foregroundColor(Color(NSColor.secondaryLabelColor))
                Text("Phiên bản \(AppMetadata.version)").font(.system(size: 12)).foregroundColor(Color(NSColor.tertiaryLabelColor))
            }
            HStack(spacing: 12) {
                AboutLink(icon: "chevron.left.forwardslash.chevron.right", title: "GitHub", url: AppMetadata.repository)
                AboutLink(icon: "ant", title: "Báo lỗi", url: AppMetadata.issuesURL)
                AboutLink(icon: "heart", title: "Ủng hộ", url: AppMetadata.sponsorURL)
            }
            Spacer()
            VStack(spacing: 8) {
                HStack(spacing: 4) {
                    Text("Phát triển bởi").foregroundColor(Color(NSColor.tertiaryLabelColor))
                    AuthorLink(name: AppMetadata.author, url: AppMetadata.authorLinkedin)
                }
                .font(.system(size: 12))
                Text("Từ Việt Nam với ❤️").font(.system(size: 11)).foregroundColor(Color(NSColor.tertiaryLabelColor))
            }
            .padding(.bottom, 8)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}

struct AboutLink: View {
    let icon: String
    let title: String
    let url: String
    @State private var hovered = false

    var body: some View {
        Link(destination: URL(string: url)!) {
            VStack(spacing: 6) {
                Image(systemName: icon).font(.system(size: 18))
                Text(title).font(.system(size: 11))
            }
            .frame(width: 80, height: 60)
            .background(RoundedRectangle(cornerRadius: 8).fill(Color(NSColor.controlBackgroundColor).opacity(hovered ? 0.8 : 0.5)))
            .overlay(RoundedRectangle(cornerRadius: 8).stroke(Color(NSColor.separatorColor).opacity(0.5), lineWidth: 0.5))
        }
        .buttonStyle(.plain)
        .foregroundColor(Color(NSColor.labelColor))
        .onHover { hovered = $0 }
    }
}

struct AuthorLink: View {
    let name: String
    let url: String
    @State private var hovered = false

    var body: some View {
        Link(destination: URL(string: url)!) { Text(name).underline(hovered) }
            .buttonStyle(.plain)
            .foregroundColor(Color.accentColor)
            .onHover { hovered = $0 }
    }
}

// MARK: - Shortcut Recorder

private let systemShortcuts: Set<String> = [
    "⌘Space", "⌘⇥", "⌘Q", "⌘W", "⌘H", "⌘M",
    "⌘⇧3", "⌘⇧4", "⌘⇧5",
    "⌃↑", "⌃↓", "⌃←", "⌃→",
]

struct ShortcutRecorderRow: View {
    @Binding var shortcut: KeyboardShortcut
    @Binding var isRecording: Bool
    @State private var hovered = false
    @State private var recordedObserver: NSObjectProtocol?
    @State private var cancelledObserver: NSObjectProtocol?
    @State private var windowObserver: NSObjectProtocol?

    private var hasConflict: Bool { systemShortcuts.contains(shortcut.displayParts.joined()) }

    var body: some View {
        HStack {
            VStack(alignment: .leading, spacing: 2) {
                Text("Phím tắt bật/tắt").font(.system(size: 13))
                Text("Nhấn để thay đổi (tối thiểu 2 phím, VD: ⌘⇧)")
                    .font(.system(size: 11))
                    .foregroundColor(Color(NSColor.secondaryLabelColor))
            }
            Spacer()
            shortcutDisplay
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 10)
        .background((hovered || isRecording) ? Color(NSColor.controlBackgroundColor).opacity(0.3) : .clear)
        .contentShape(Rectangle())
        .onHover { hovered = $0 }
        .onTapGesture { isRecording ? stopRecording() : startRecording() }
        .onDisappear { stopRecording() }
    }

    @ViewBuilder
    private var shortcutDisplay: some View {
        HStack(spacing: 4) {
            if isRecording {
                Text("Nhấn phím...")
                    .font(.system(size: 11, weight: .medium))
                    .foregroundColor(.accentColor)
                    .padding(.horizontal, 6)
                    .padding(.vertical, 3)
                    .background(RoundedRectangle(cornerRadius: 4).stroke(Color.accentColor, lineWidth: 1))
            } else {
                ForEach(shortcut.displayParts, id: \.self) { KeyCap(text: $0) }
                if hasConflict {
                    Image(systemName: "exclamationmark.triangle.fill")
                        .font(.system(size: 12))
                        .foregroundColor(.orange)
                        .help("Phím tắt này có thể xung đột với hệ thống")
                }
            }
        }
    }

    private func startRecording() {
        isRecording = true
        recordedObserver = NotificationCenter.default.addObserver(forName: .shortcutRecorded, object: nil, queue: .main) { notification in
            if let captured = notification.object as? KeyboardShortcut { shortcut = captured }
            stopRecording()
        }
        cancelledObserver = NotificationCenter.default.addObserver(forName: .shortcutRecordingCancelled, object: nil, queue: .main) { _ in stopRecording() }
        windowObserver = NotificationCenter.default.addObserver(forName: NSWindow.didResignKeyNotification, object: nil, queue: .main) { _ in stopRecording() }
        startShortcutRecording()
    }

    private func stopRecording() {
        stopShortcutRecording()
        [recordedObserver, cancelledObserver, windowObserver].compactMap { $0 }.forEach { NotificationCenter.default.removeObserver($0) }
        recordedObserver = nil
        cancelledObserver = nil
        windowObserver = nil
        isRecording = false
    }
}

// MARK: - Launch at Login Toggle Row

struct LaunchAtLoginToggleRow: View {
    @ObservedObject var appState: AppState

    var body: some View {
        SettingsRow {
            VStack(alignment: .leading, spacing: 2) {
                Text("Khởi động cùng hệ thống")
                    .font(.system(size: 13))
                Text("Tự động chạy ứng dụng khi đăng nhập")
                    .font(.system(size: 11))
                    .foregroundColor(Color(NSColor.secondaryLabelColor))
            }
            Spacer()
            Toggle("", isOn: Binding(
                get: { appState.isLaunchAtLoginEnabled },
                set: { appState.toggleLaunchAtLogin($0) }
            ))
            .toggleStyle(.switch)
            .labelsHidden()
        }
    }
}

// MARK: - Preview

#Preview { MainSettingsView() }
