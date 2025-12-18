import SwiftUI
import UniformTypeIdentifiers
import Combine

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
    case idle
    case checking
    case upToDate
    case available(String)  // New version string
    case error

    var isChecking: Bool {
        if case .checking = self { return true }
        return false
    }

    var isAvailable: Bool {
        if case .available = self { return true }
        return false
    }
}

// MARK: - App State

class AppState: ObservableObject {
    static let shared = AppState()

    // Flag for silent updates (from app switch) - won't trigger save
    private var isSilentUpdate = false

    @Published var isEnabled: Bool {
        didSet {
            // Always sync engine and update icon
            RustBridge.setEnabled(isEnabled)
            NotificationCenter.default.post(name: .menuStateChanged, object: nil)

            // Skip save when silent update (from app switch restore)
            guard !isSilentUpdate else { return }

            UserDefaults.standard.set(isEnabled, forKey: SettingsKey.enabled)

            // Save for current app if smart mode enabled
            if isSmartModeEnabled,
               let bundleId = NSWorkspace.shared.frontmostApplication?.bundleIdentifier {
                savePerAppMode(bundleId: bundleId, enabled: isEnabled)
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

    /// Smart mode: automatically remember ON/OFF state per app
    @Published var isSmartModeEnabled: Bool = true {
        didSet {
            UserDefaults.standard.set(isSmartModeEnabled, forKey: SettingsKey.smartModeEnabled)
        }
    }

    /// Auto W→Ư shortcut: when enabled, 'w' at word start converts to 'ư' (Telex only)
    @Published var autoWShortcut: Bool = true {
        didSet {
            UserDefaults.standard.set(autoWShortcut, forKey: SettingsKey.autoWShortcut)
            RustBridge.setSkipWShortcut(!autoWShortcut)  // Invert: skip = !auto
        }
    }

    /// Toggle Vietnamese input on/off
    func toggle() {
        isEnabled.toggle()
    }

    /// Set input method (Telex/VNI)
    func setMethod(_ method: InputMode) {
        currentMethod = method
    }

    @Published var toggleShortcut: KeyboardShortcut {
        didSet {
            toggleShortcut.save()
            NotificationCenter.default.post(name: .shortcutChanged, object: toggleShortcut)
        }
    }

    @Published var updateStatus: UpdateStatus = .idle

    @Published var shortcuts: [ShortcutItem] = []

    /// Launch at Login status (auto-refreshed)
    @Published var isLaunchAtLoginEnabled: Bool = false

    /// Timer for refreshing launch at login status
    private var launchAtLoginTimer: Timer?

    init() {
        isEnabled = UserDefaults.standard.object(forKey: SettingsKey.enabled) as? Bool ?? true
        currentMethod = InputMode(rawValue: UserDefaults.standard.integer(forKey: SettingsKey.method)) ?? .telex
        toggleShortcut = KeyboardShortcut.load()

        // Smart mode (default true for new installs)
        if UserDefaults.standard.object(forKey: SettingsKey.smartModeEnabled) == nil {
            isSmartModeEnabled = true
            UserDefaults.standard.set(true, forKey: SettingsKey.smartModeEnabled)
        } else {
            isSmartModeEnabled = UserDefaults.standard.bool(forKey: SettingsKey.smartModeEnabled)
        }

        // Auto W→Ư shortcut (default true for new installs)
        if UserDefaults.standard.object(forKey: SettingsKey.autoWShortcut) == nil {
            autoWShortcut = true
            UserDefaults.standard.set(true, forKey: SettingsKey.autoWShortcut)
        } else {
            autoWShortcut = UserDefaults.standard.bool(forKey: SettingsKey.autoWShortcut)
        }
        RustBridge.setSkipWShortcut(!autoWShortcut)  // Invert: skip = !auto

        // Load shortcuts from UserDefaults (or use defaults)
        if let data = UserDefaults.standard.data(forKey: SettingsKey.shortcuts),
           let saved = try? JSONDecoder().decode([ShortcutItem].self, from: data) {
            shortcuts = saved
        } else {
            // Default shortcuts (OFF by default)
            shortcuts = [
                ShortcutItem(key: "vn", value: "Việt Nam", isEnabled: false),
                ShortcutItem(key: "hn", value: "Hà Nội", isEnabled: false),
                ShortcutItem(key: "hcm", value: "Hồ Chí Minh", isEnabled: false),
                ShortcutItem(key: "tphcm", value: "Thành phố Hồ Chí Minh", isEnabled: false),
            ]
        }

        checkForUpdates()
        setupObservers()
        setupLaunchAtLoginMonitoring()
    }

    // MARK: - Launch at Login Monitoring

    /// Start monitoring launch at login status
    private func setupLaunchAtLoginMonitoring() {
        // Initial check
        isLaunchAtLoginEnabled = LaunchAtLoginManager.shared.isEnabled

        // Refresh every 2 seconds (user may toggle in System Settings)
        launchAtLoginTimer = Timer.scheduledTimer(withTimeInterval: 2.0, repeats: true) { [weak self] _ in
            DispatchQueue.main.async {
                self?.refreshLaunchAtLoginStatus()
            }
        }
    }

    /// Refresh launch at login status from system
    func refreshLaunchAtLoginStatus() {
        let newStatus = LaunchAtLoginManager.shared.isEnabled
        if newStatus != isLaunchAtLoginEnabled {
            isLaunchAtLoginEnabled = newStatus
        }
    }

    /// Enable launch at login (register with SMAppService)
    func enableLaunchAtLogin() {
        do {
            try LaunchAtLoginManager.shared.enable()
            refreshLaunchAtLoginStatus()
        } catch {
            // If registration fails or requires approval, open System Settings
            openLoginItemsSettings()
        }
    }

    /// Open macOS Login Items settings
    func openLoginItemsSettings() {
        // macOS 13+ Login Items URL
        if let url = URL(string: "x-apple.systempreferences:com.apple.LoginItems-Settings.extension") {
            NSWorkspace.shared.open(url)
        }
    }

    // MARK: - Per-App Mode (only stores OFF apps, default is ON)

    /// Save per-app mode: only store OFF apps, remove entry when ON (default)
    func savePerAppMode(bundleId: String, enabled: Bool) {
        var modes = UserDefaults.standard.dictionary(forKey: SettingsKey.perAppModes) as? [String: Bool] ?? [:]
        if enabled {
            modes.removeValue(forKey: bundleId)  // ON is default, no need to store
        } else {
            modes[bundleId] = false  // Only store OFF apps
        }
        UserDefaults.standard.set(modes, forKey: SettingsKey.perAppModes)
    }

    /// Get per-app mode: returns true (ON) if not stored
    func getPerAppMode(bundleId: String) -> Bool {
        let modes = UserDefaults.standard.dictionary(forKey: SettingsKey.perAppModes) as? [String: Bool] ?? [:]
        return modes[bundleId] ?? true  // Default ON
    }

    /// Silent update (from app switch) - won't trigger save, but still updates UI via Combine
    func setEnabledSilently(_ enabled: Bool) {
        isSilentUpdate = true
        isEnabled = enabled
        isSilentUpdate = false
    }

    private func setupObservers() {
        // Observe changes to sync to engine and save to UserDefaults
        $shortcuts
            .dropFirst() // Skip initial value
            .debounce(for: .milliseconds(300), scheduler: RunLoop.main)
            .sink { [weak self] shortcuts in
                self?.syncShortcutsToEngine()
                // Save to UserDefaults
                if let data = try? JSONEncoder().encode(shortcuts) {
                    UserDefaults.standard.set(data, forKey: SettingsKey.shortcuts)
                }
            }
            .store(in: &cancellables)
    }

    private var cancellables = Set<AnyCancellable>()

    /// Sync shortcuts to Rust engine
    func syncShortcutsToEngine() {
        let data = shortcuts.map { ($0.key, $0.value, $0.isEnabled) }
        RustBridge.syncShortcuts(data)
    }

    // MARK: - Import/Export Shortcuts (OpenKey format)

    /// Export shortcuts to text format (compatible with OpenKey/UniKey)
    func exportShortcuts() -> String {
        var lines = [";Gõ Nhanh - Bảng gõ tắt"]
        for shortcut in shortcuts where !shortcut.key.isEmpty {
            lines.append("\(shortcut.key):\(shortcut.value)")
        }
        return lines.joined(separator: "\n")
    }

    /// Import shortcuts from OpenKey-compatible format (merges with existing)
    func importShortcuts(from content: String) -> Int {
        let lines = content.components(separatedBy: .newlines)
        var imported = 0

        for line in lines {
            let trimmed = line.trimmingCharacters(in: .whitespaces)
            // Skip empty lines and comments
            guard !trimmed.isEmpty, !trimmed.hasPrefix(";") else { continue }

            // Parse trigger:replacement
            if let colonIndex = trimmed.firstIndex(of: ":") {
                let trigger = String(trimmed[..<colonIndex]).trimmingCharacters(in: .whitespaces)
                let replacement = String(trimmed[trimmed.index(after: colonIndex)...]).trimmingCharacters(in: .whitespaces)

                guard !trigger.isEmpty else { continue }

                // Check if trigger already exists
                if let idx = shortcuts.firstIndex(where: { $0.key == trigger }) {
                    shortcuts[idx].value = replacement
                    shortcuts[idx].isEnabled = true
                } else {
                    shortcuts.append(ShortcutItem(key: trigger, value: replacement, isEnabled: true))
                }
                imported += 1
            }
        }
        return imported
    }

    func checkForUpdates() {
        updateStatus = .checking
        let startTime = Date()
        UpdateChecker.shared.checkForUpdates { [weak self] result in
            // Ensure minimum 1.5s loading time for better UX
            let elapsed = Date().timeIntervalSince(startTime)
            let delay = max(0, 1.5 - elapsed)
            DispatchQueue.main.asyncAfter(deadline: .now() + delay) {
                switch result {
                case .available(let info):
                    self?.updateStatus = .available(info.version)
                case .upToDate:
                    self?.updateStatus = .upToDate
                case .error:
                    self?.updateStatus = .error
                }
            }
        }
    }
}


struct ShortcutItem: Identifiable, Codable {
    var id = UUID()
    var key: String
    var value: String
    var isEnabled: Bool = true
}

// MARK: - Main Settings View

struct MainSettingsView: View {
    @ObservedObject private var appState = AppState.shared
    @State private var selectedPage: NavigationPage = .settings
    @Environment(\.colorScheme) private var colorScheme

    private var isDark: Bool { colorScheme == .dark }

    var body: some View {
        HStack(spacing: 0) {
            // Sidebar
            ZStack {
                VisualEffectBackground(material: .sidebar, blendingMode: .behindWindow)
                sidebar
            }
            .frame(width: 200)

            // Content
            ZStack {
                if isDark {
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
            if let page = notification.object as? NavigationPage {
                selectedPage = page
            }
        }
    }

    // MARK: - Sidebar

    private var sidebar: some View {
        VStack(spacing: 0) {
            // Logo
            VStack(spacing: 12) {
                Image(nsImage: AppMetadata.logo)
                    .resizable()
                    .frame(width: 96, height: 96)

                Text(AppMetadata.name)
                    .font(.system(size: 20, weight: .bold))

                // Version badge with update status
                updateBadge
            }
            .padding(.top, 40)

            Spacer()

            // Navigation
            VStack(spacing: 4) {
                ForEach(NavigationPage.allCases, id: \.self) { page in
                    NavButton(page: page, isSelected: selectedPage == page) {
                        selectedPage = page
                    }
                }
            }
            .padding(.horizontal, 12)
            .padding(.bottom, 20)
        }
    }

    @ViewBuilder
    private var updateBadge: some View {
        UpdateBadgeView(status: appState.updateStatus) {
            appState.checkForUpdates()
        }
    }

    // MARK: - Content

    @ViewBuilder
    private var content: some View {
        switch selectedPage {
        case .settings:
            ScrollView(showsIndicators: false) {
                SettingsPageView(appState: appState)
                    .padding(28)
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)
        case .about:
            AboutPageView()
                .padding(28)
                .frame(maxWidth: .infinity, maxHeight: .infinity)
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
        case .idle: return nil
        case .checking: return nil  // Handled separately with animation
        case .upToDate: return ("checkmark.circle.fill", .green)
        case .available: return ("arrow.up.circle.fill", .orange)
        case .error: return ("exclamationmark.triangle.fill", .orange)
        }
    }

    var body: some View {
        HStack(spacing: 3) {
            Text("v\(AppMetadata.version)")

            // Icon (all filled circle style)
            if status.isChecking {
                Image(systemName: "arrow.clockwise.circle.fill")
                    .font(.system(size: 12))
                    .foregroundColor(.secondary)
                    .rotationEffect(.degrees(rotation))
                    .onAppear {
                        withAnimation(.linear(duration: 1).repeatForever(autoreverses: false)) {
                            rotation = 360
                        }
                    }
                    .onDisappear { rotation = 0 }
            } else if let icon = statusIcon {
                Image(systemName: icon.name)
                    .font(.system(size: 12))
                    .foregroundColor(icon.color)
            }

            // Text
            if let text = statusText {
                Text(text)
            }
        }
        .font(.system(size: 11))
        .foregroundColor(Color(NSColor.tertiaryLabelColor))
        .padding(.horizontal, 8)
        .padding(.vertical, 4)
        .background(
            Capsule()
                .fill(hovered ? Color(NSColor.controlBackgroundColor).opacity(0.5) : Color.clear)
        )
        .onHover { h in
            hovered = h
            if status.isAvailable {
                if h { NSCursor.pointingHand.push() } else { NSCursor.pop() }
            }
        }
        .onTapGesture {
            guard !status.isChecking else { return }
            if status.isAvailable {
                // Show update window AND start download immediately
                if case .available(let info) = UpdateManager.shared.state {
                    UpdateManager.shared.downloadUpdate(info)
                    NotificationCenter.default.post(name: .showUpdateWindow, object: nil)
                }
            } else {
                onCheck()
            }
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
            // Launch at Login warning banner
            if !appState.isLaunchAtLoginEnabled {
                LaunchAtLoginBanner {
                    appState.enableLaunchAtLogin()
                }
            }

            // General settings
            VStack(spacing: 0) {
                // Enable toggle
                HStack {
                    Text("Bộ gõ tiếng Việt")
                        .font(.system(size: 13))
                    Spacer()
                    Toggle("", isOn: $appState.isEnabled)
                        .toggleStyle(.switch)
                        .labelsHidden()
                }
                .padding(.horizontal, 12)
                .padding(.vertical, 10)

                Divider().padding(.leading, 12)

                // Input method
                HStack {
                    Text("Kiểu gõ")
                        .font(.system(size: 13))
                    Spacer()
                    Picker("", selection: $appState.currentMethod) {
                        ForEach(InputMode.allCases, id: \.self) { mode in
                            Text(mode.name).tag(mode)
                        }
                    }
                    .labelsHidden()
                    .frame(width: 100)
                }
                .padding(.horizontal, 12)
                .padding(.vertical, 10)

                Divider().padding(.leading, 12)

                // Shortcut (clickable)
                ShortcutRecorderRow(
                    shortcut: $appState.toggleShortcut,
                    isRecording: $isRecordingShortcut
                )
            }
            .background(
                RoundedRectangle(cornerRadius: 10)
                    .fill(Color(NSColor.controlBackgroundColor).opacity(0.5))
            )
            .overlay(
                RoundedRectangle(cornerRadius: 10)
                    .stroke(Color(NSColor.separatorColor).opacity(0.5), lineWidth: 0.5)
            )

            // Options section
            VStack(spacing: 0) {
                // Smart Mode
                HStack {
                    VStack(alignment: .leading, spacing: 2) {
                        Text("Chuyển chế độ thông minh")
                            .font(.system(size: 13))
                        Text("Tự động nhớ trạng thái cho từng ứng dụng")
                            .font(.system(size: 11))
                            .foregroundColor(Color(NSColor.secondaryLabelColor))
                    }
                    Spacer()
                    Toggle("", isOn: $appState.isSmartModeEnabled)
                        .toggleStyle(.switch)
                        .labelsHidden()
                }
                .padding(.horizontal, 12)
                .padding(.vertical, 10)

                // Auto W→Ư shortcut (Telex only)
                if appState.currentMethod == .telex {
                    Divider().padding(.leading, 12)

                    HStack {
                        VStack(alignment: .leading, spacing: 2) {
                            Text("Tự chuyển W → Ư ở đầu từ")
                                .font(.system(size: 13))
                            Text("Gõ 'w' đầu từ sẽ thành 'ư'")
                                .font(.system(size: 11))
                                .foregroundColor(Color(NSColor.secondaryLabelColor))
                        }
                        Spacer()
                        Toggle("", isOn: $appState.autoWShortcut)
                            .toggleStyle(.switch)
                            .labelsHidden()
                    }
                    .padding(.horizontal, 12)
                    .padding(.vertical, 10)
                }

                Divider().padding(.leading, 12)

                // Shortcuts - opens sheet
                Button(action: { showShortcutsSheet = true }) {
                    HStack {
                        VStack(alignment: .leading, spacing: 2) {
                            Text("Từ viết tắt")
                                .font(.system(size: 13))
                                .foregroundColor(Color(NSColor.labelColor))
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
            .background(
                RoundedRectangle(cornerRadius: 10)
                    .fill(Color(NSColor.controlBackgroundColor).opacity(0.5))
            )
            .overlay(
                RoundedRectangle(cornerRadius: 10)
                    .stroke(Color(NSColor.separatorColor).opacity(0.5), lineWidth: 0.5)
            )

            Spacer()
        }
        .sheet(isPresented: $showShortcutsSheet) {
            ShortcutsSheet(appState: appState)
        }
    }
}

// MARK: - Shortcuts Sheet

struct ShortcutsSheet: View {
    @ObservedObject var appState: AppState
    @Environment(\.dismiss) private var dismiss
    @State private var selectedId: UUID?
    @FocusState private var focusedField: UUID?

    var body: some View {
        VStack(spacing: 0) {
            // Header
            HStack {
                Text("Quản lý từ viết tắt")
                    .font(.system(size: 13, weight: .semibold))
                Spacer()
                Button("Xong") { dismiss() }
                    .buttonStyle(.borderedProminent)
                    .controlSize(.small)
            }
            .padding()

            Divider()

            // Content
            if appState.shortcuts.isEmpty {
                Spacer()
                EmptyStateView(icon: "text.badge.plus", text: "Chưa có từ viết tắt")
                Spacer()
            } else {
                ScrollView {
                    LazyVStack(spacing: 0) {
                        ForEach($appState.shortcuts) { $shortcut in
                            ShortcutRow(
                                shortcut: $shortcut,
                                isSelected: selectedId == shortcut.id,
                                focusedField: $focusedField
                            ) {
                                selectedId = shortcut.id
                            }
                            if shortcut.id != appState.shortcuts.last?.id {
                                Divider().padding(.leading, 12)
                            }
                        }
                    }
                }
            }

            Divider()

            // Toolbar
            HStack(spacing: 0) {
                Button(action: addShortcut) {
                    Image(systemName: "plus").frame(width: 28, height: 28)
                }
                .buttonStyle(.borderless)

                Divider().frame(height: 16)

                Button(action: removeSelected) {
                    Image(systemName: "minus").frame(width: 28, height: 28)
                }
                .buttonStyle(.borderless)
                .disabled(selectedId == nil)

                Spacer()

                Button(action: importShortcuts) {
                    Image(systemName: "square.and.arrow.down").frame(width: 28, height: 28)
                }
                .buttonStyle(.borderless)
                .help("Nhập")

                Divider().frame(height: 16)

                Button(action: exportShortcuts) {
                    Image(systemName: "square.and.arrow.up").frame(width: 28, height: 28)
                }
                .buttonStyle(.borderless)
                .disabled(appState.shortcuts.isEmpty)
                .help("Xuất")
            }
            .padding(.horizontal, 8)
            .padding(.vertical, 6)
        }
        .frame(width: 420, height: 400)
        .background(Color(NSColor.windowBackgroundColor))
    }

    private func addShortcut() {
        let item = ShortcutItem(key: "", value: "")
        appState.shortcuts.append(item)
        selectedId = item.id
        focusedField = item.id
    }

    private func removeSelected() {
        guard let id = selectedId,
              let idx = appState.shortcuts.firstIndex(where: { $0.id == id }) else { return }
        appState.shortcuts.remove(at: idx)
        selectedId = appState.shortcuts.last?.id
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

            // App Info - Centered
            VStack(spacing: 12) {
                Image(nsImage: AppMetadata.logo)
                    .resizable()
                    .frame(width: 80, height: 80)

                Text(AppMetadata.name)
                    .font(.system(size: 20, weight: .bold))

                Text("Bộ gõ tiếng Việt nhanh và nhẹ")
                    .font(.system(size: 13))
                    .foregroundColor(Color(NSColor.secondaryLabelColor))

                Text("Phiên bản \(AppMetadata.version)")
                    .font(.system(size: 12))
                    .foregroundColor(Color(NSColor.tertiaryLabelColor))
            }

            // Links - Horizontal buttons
            HStack(spacing: 12) {
                AboutLink(icon: "chevron.left.forwardslash.chevron.right", title: "GitHub", url: AppMetadata.repository)
                AboutLink(icon: "ant", title: "Báo lỗi", url: AppMetadata.issuesURL)
                AboutLink(icon: "heart", title: "Ủng hộ", url: AppMetadata.sponsorURL)
            }

            Spacer()

            // Footer
            VStack(spacing: 8) {
                HStack(spacing: 4) {
                    Text("Phát triển bởi")
                        .foregroundColor(Color(NSColor.tertiaryLabelColor))
                    AuthorLink(name: AppMetadata.author, url: AppMetadata.authorLinkedin)
                }
                .font(.system(size: 12))

                Text("Từ Việt Nam với ❤️")
                    .font(.system(size: 11))
                    .foregroundColor(Color(NSColor.tertiaryLabelColor))
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
                Image(systemName: icon)
                    .font(.system(size: 18))
                Text(title)
                    .font(.system(size: 11))
            }
            .frame(width: 80, height: 60)
            .background(
                RoundedRectangle(cornerRadius: 8)
                    .fill(Color(NSColor.controlBackgroundColor).opacity(hovered ? 0.8 : 0.5))
            )
            .overlay(
                RoundedRectangle(cornerRadius: 8)
                    .stroke(Color(NSColor.separatorColor).opacity(0.5), lineWidth: 0.5)
            )
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
        Link(destination: URL(string: url)!) {
            Text(name)
                .underline(hovered)
        }
        .buttonStyle(.plain)
        .foregroundColor(Color.accentColor)
        .onHover { hovered = $0 }
    }
}

// MARK: - Shortcut Recorder

/// System shortcuts that cannot be overridden by apps
private let systemShortcuts: Set<String> = [
    "⌘Space", "⌘⇥", "⌘Q", "⌘W", "⌘H", "⌘M",  // App shortcuts
    "⌘⇧3", "⌘⇧4", "⌘⇧5",                      // Screenshots
    "⌃↑", "⌃↓", "⌃←", "⌃→",                    // Spaces navigation
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
            Text("Phím tắt bật/tắt")
                .font(.system(size: 13))
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

    // MARK: - Recording (CGEventTap-based to capture system shortcuts like Ctrl+Space)

    private func startRecording() {
        isRecording = true

        // Listen for shortcut captured via CGEventTap
        recordedObserver = NotificationCenter.default.addObserver(
            forName: .shortcutRecorded, object: nil, queue: .main
        ) { notification in
            if let captured = notification.object as? KeyboardShortcut {
                shortcut = captured
            }
            stopRecording()
        }

        // Listen for recording cancelled (ESC pressed)
        cancelledObserver = NotificationCenter.default.addObserver(
            forName: .shortcutRecordingCancelled, object: nil, queue: .main
        ) { _ in
            stopRecording()
        }

        // Stop recording if window loses focus
        windowObserver = NotificationCenter.default.addObserver(
            forName: NSWindow.didResignKeyNotification, object: nil, queue: .main
        ) { _ in
            stopRecording()
        }

        // Start CGEventTap recording mode
        startShortcutRecording()
    }

    private func stopRecording() {
        // Stop CGEventTap recording mode
        stopShortcutRecording()

        // Remove observers
        if let observer = recordedObserver {
            NotificationCenter.default.removeObserver(observer)
            recordedObserver = nil
        }
        if let observer = cancelledObserver {
            NotificationCenter.default.removeObserver(observer)
            cancelledObserver = nil
        }
        if let observer = windowObserver {
            NotificationCenter.default.removeObserver(observer)
            windowObserver = nil
        }

        isRecording = false
    }
}

// MARK: - Reusable Components

struct SectionView<Content: View>: View {
    let title: String
    @ViewBuilder let content: Content

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(title)
                .font(.system(size: 11, weight: .medium))
                .foregroundColor(Color(NSColor.secondaryLabelColor))
                .padding(.horizontal, 4)

            VStack(spacing: 0) {
                content
            }
            .background(
                RoundedRectangle(cornerRadius: 10)
                    .fill(Color(NSColor.controlBackgroundColor).opacity(0.5))
            )
            .clipShape(RoundedRectangle(cornerRadius: 10))
            .overlay(
                RoundedRectangle(cornerRadius: 10)
                    .stroke(Color(NSColor.separatorColor).opacity(0.5), lineWidth: 0.5)
            )
        }
    }
}

struct ShortcutRow: View {
    @Binding var shortcut: ShortcutItem
    var isSelected: Bool
    var focusedField: FocusState<UUID?>.Binding
    var onSelect: () -> Void

    var body: some View {
        HStack(spacing: 8) {
            TextField("viết tắt", text: $shortcut.key)
                .font(.system(size: 13))
                .textFieldStyle(.plain)
                .frame(width: 60)
                .focused(focusedField, equals: shortcut.id)
            Text("→")
                .font(.system(size: 11))
                .foregroundColor(Color(NSColor.tertiaryLabelColor))
            TextField("nội dung", text: $shortcut.value)
                .font(.system(size: 13))
                .textFieldStyle(.plain)
                .focused(focusedField, equals: shortcut.id)
            Spacer()
            Toggle("", isOn: $shortcut.isEnabled)
                .toggleStyle(.switch)
                .labelsHidden()
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 8)
        .background(isSelected ? Color.accentColor.opacity(0.15) : Color.clear)
        .contentShape(Rectangle())
        .onTapGesture { onSelect() }
    }
}

struct EmptyStateView: View {
    let icon: String
    let text: String

    var body: some View {
        HStack {
            Spacer()
            VStack(spacing: 6) {
                Image(systemName: icon)
                    .font(.system(size: 20))
                    .foregroundColor(Color(NSColor.tertiaryLabelColor))
                Text(text)
                    .font(.system(size: 12))
                    .foregroundColor(Color(NSColor.tertiaryLabelColor))
            }
            .padding(.vertical, 20)
            Spacer()
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
            .background(
                RoundedRectangle(cornerRadius: 4)
                    .fill(Color(NSColor.controlBackgroundColor).opacity(0.8))
            )
            .overlay(
                RoundedRectangle(cornerRadius: 4)
                    .stroke(Color(NSColor.separatorColor).opacity(0.5), lineWidth: 0.5)
            )
    }
}

// MARK: - Launch at Login Banner

struct LaunchAtLoginBanner: View {
    let onOpenSettings: () -> Void

    @State private var hovered = false

    var body: some View {
        HStack(spacing: 10) {
            Image(systemName: "exclamationmark.triangle.fill")
                .font(.system(size: 14))
                .foregroundColor(.orange)

            VStack(alignment: .leading, spacing: 2) {
                Text("Chưa bật khởi động cùng hệ thống")
                    .font(.system(size: 12, weight: .medium))
                Text("Nhấn để bật")
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
        .background(
            RoundedRectangle(cornerRadius: 10)
                .fill(Color.orange.opacity(hovered ? 0.15 : 0.1))
        )
        .overlay(
            RoundedRectangle(cornerRadius: 10)
                .stroke(Color.orange.opacity(0.3), lineWidth: 0.5)
        )
        .contentShape(Rectangle())
        .onHover { h in
            hovered = h
            if h { NSCursor.pointingHand.push() } else { NSCursor.pop() }
        }
        .onTapGesture {
            onOpenSettings()
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

// MARK: - Preview

#Preview {
    MainSettingsView()
}
