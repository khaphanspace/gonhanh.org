import SwiftUI
import UniformTypeIdentifiers

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

enum UpdateStatus {
    case checking
    case upToDate
    case available(String)  // New version string
    case error
}

// MARK: - App State

class AppState: ObservableObject {
    static let shared = AppState()

    @Published var isEnabled: Bool {
        didSet {
            UserDefaults.standard.set(isEnabled, forKey: SettingsKey.enabled)
            RustBridge.setEnabled(isEnabled)
            NotificationCenter.default.post(name: .menuStateChanged, object: nil)
        }
    }

    @Published var currentMethod: InputMode {
        didSet {
            UserDefaults.standard.set(currentMethod.rawValue, forKey: SettingsKey.method)
            RustBridge.setMethod(currentMethod.rawValue)
            NotificationCenter.default.post(name: .menuStateChanged, object: nil)
        }
    }

    @Published var toggleShortcut: KeyboardShortcut {
        didSet {
            toggleShortcut.save()
            NotificationCenter.default.post(name: .shortcutChanged, object: toggleShortcut)
        }
    }

    @Published var updateStatus: UpdateStatus = .checking

    @Published var shortcuts: [ShortcutItem] = [
        ShortcutItem(key: "vn", value: "Việt Nam"),
        ShortcutItem(key: "hcm", value: "Hồ Chí Minh"),
        ShortcutItem(key: "sdt", value: "0912 345 678")
    ]

    @Published var excludedApps: [ExcludedApp] = [
        ExcludedApp(bundleId: "com.apple.Terminal", name: "Terminal", icon: NSWorkspace.shared.icon(forFile: "/System/Applications/Utilities/Terminal.app")),
        ExcludedApp(bundleId: "com.microsoft.VSCode", name: "Visual Studio Code", icon: NSWorkspace.shared.icon(forFile: "/Applications/Visual Studio Code.app"))
    ]

    init() {
        isEnabled = UserDefaults.standard.object(forKey: SettingsKey.enabled) as? Bool ?? true
        currentMethod = InputMode(rawValue: UserDefaults.standard.integer(forKey: SettingsKey.method)) ?? .telex
        toggleShortcut = KeyboardShortcut.load()
        checkForUpdates()
    }

    func checkForUpdates() {
        updateStatus = .checking
        UpdateChecker.shared.checkForUpdates { [weak self] result in
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


struct ShortcutItem: Identifiable {
    let id = UUID()
    var key: String
    var value: String
    var isEnabled: Bool = true
}

struct ExcludedApp: Identifiable {
    let id = UUID()
    var bundleId: String
    var name: String
    var icon: NSImage?
    var isEnabled: Bool = true
}

// MARK: - Main Settings View

struct MainSettingsView: View {
    @StateObject private var appState = AppState.shared
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
                    .frame(width: 80, height: 80)

                Text(AppMetadata.name)
                    .font(.system(size: 20, weight: .bold))

                // Version badge with update status
                updateBadge
            }
            .padding(.top, 28)

            Spacer()

            // Navigation
            VStack(spacing: 4) {
                ForEach(NavigationPage.allCases, id: \.self) { page in
                    NavButton(page: page, isSelected: selectedPage == page) {
                        withAnimation(.easeOut(duration: 0.15)) {
                            selectedPage = page
                        }
                    }
                }
            }
            .padding(.horizontal, 12)
            .padding(.bottom, 20)
        }
    }

    @ViewBuilder
    private var updateBadge: some View {
        switch appState.updateStatus {
        case .checking:
            HStack(spacing: 4) {
                Text("v\(AppMetadata.version)")
                ProgressView()
                    .scaleEffect(0.5)
                    .frame(width: 12, height: 12)
            }
            .font(.system(size: 11))
            .foregroundColor(Color(NSColor.tertiaryLabelColor))

        case .upToDate:
            HStack(spacing: 4) {
                Text("v\(AppMetadata.version)")
                Image(systemName: "checkmark.circle.fill")
                    .font(.system(size: 10))
                    .foregroundColor(.green)
                Text("Mới nhất")
            }
            .font(.system(size: 11))
            .foregroundColor(Color(NSColor.tertiaryLabelColor))

        case .available(let newVersion):
            Button {
                NotificationCenter.default.post(name: .showUpdateWindow, object: nil)
            } label: {
                HStack(spacing: 6) {
                    Text("v\(AppMetadata.version)")
                        .foregroundColor(Color(NSColor.tertiaryLabelColor))
                    HStack(spacing: 3) {
                        Image(systemName: "arrow.up.circle.fill")
                            .font(.system(size: 10))
                        Text("v\(newVersion)")
                    }
                    .foregroundColor(.white)
                    .padding(.horizontal, 6)
                    .padding(.vertical, 2)
                    .background(
                        Capsule()
                            .fill(Color.orange)
                    )
                }
                .font(.system(size: 11))
            }
            .buttonStyle(.plain)
            .onHover { hovering in
                if hovering { NSCursor.pointingHand.push() } else { NSCursor.pop() }
            }

        case .error:
            Button {
                appState.checkForUpdates()
            } label: {
                HStack(spacing: 4) {
                    Text("v\(AppMetadata.version)")
                    Image(systemName: "arrow.clockwise")
                        .font(.system(size: 10))
                }
                .font(.system(size: 11))
                .foregroundColor(Color(NSColor.tertiaryLabelColor))
            }
            .buttonStyle(.plain)
            .onHover { hovering in
                if hovering { NSCursor.pointingHand.push() } else { NSCursor.pop() }
            }
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

// MARK: - Nav Button

struct NavButton: View {
    let page: NavigationPage
    let isSelected: Bool
    let action: () -> Void

    @State private var hovered = false

    var body: some View {
        Button(action: action) {
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
                          hovered ? Color(NSColor.controlBackgroundColor).opacity(0.3) : Color.clear)
            )
            .animation(.easeInOut(duration: 0.15), value: hovered)
        }
        .buttonStyle(.plain)
        .onHover { hovered = $0 }
    }
}

// MARK: - Settings Page

struct SettingsPageView: View {
    @ObservedObject var appState: AppState
    @State private var isRecordingShortcut = false
    @State private var selectedShortcutId: UUID?
    @State private var selectedAppId: UUID?

    var body: some View {
        VStack(alignment: .leading, spacing: 20) {
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

            // Shortcuts section
            SectionView(title: "TỪ VIẾT TẮT") {
                if appState.shortcuts.isEmpty {
                    EmptyStateView(icon: "text.badge.plus", text: "Chưa có từ viết tắt")
                } else {
                    ForEach($appState.shortcuts) { $shortcut in
                        ShortcutRow(
                            shortcut: $shortcut,
                            isSelected: selectedShortcutId == shortcut.id
                        ) {
                            selectedShortcutId = shortcut.id
                            selectedAppId = nil
                        }
                        if shortcut.id != appState.shortcuts.last?.id {
                            Divider()
                        }
                    }
                }

                Divider()
                AddRemoveButtons(
                    onAdd: {
                        let newItem = ShortcutItem(key: "", value: "")
                        appState.shortcuts.append(newItem)
                        selectedShortcutId = newItem.id
                    },
                    onRemove: {
                        if let id = selectedShortcutId,
                           let idx = appState.shortcuts.firstIndex(where: { $0.id == id }) {
                            appState.shortcuts.remove(at: idx)
                            selectedShortcutId = appState.shortcuts.last?.id
                        }
                    },
                    removeDisabled: appState.shortcuts.isEmpty
                )
            }

            // Excluded apps section
            SectionView(title: "ỨNG DỤNG BỎ QUA") {
                if appState.excludedApps.isEmpty {
                    EmptyStateView(icon: "app.dashed", text: "Chưa có ứng dụng")
                } else {
                    ForEach($appState.excludedApps) { $app in
                        ExcludedAppRow(app: $app, isSelected: selectedAppId == app.id) {
                            selectedAppId = app.id
                            selectedShortcutId = nil
                        }
                        if app.id != appState.excludedApps.last?.id {
                            Divider()
                        }
                    }
                }

                Divider()
                AddRemoveButtons(
                    onAdd: { showAppPicker() },
                    onRemove: {
                        if let id = selectedAppId,
                           let idx = appState.excludedApps.firstIndex(where: { $0.id == id }) {
                            appState.excludedApps.remove(at: idx)
                            selectedAppId = appState.excludedApps.last?.id
                        }
                    },
                    removeDisabled: appState.excludedApps.isEmpty
                )
            }

            Spacer()
        }
        .contentShape(Rectangle())
    }

    private func showAppPicker() {
        let panel = NSOpenPanel()
        panel.title = "Chọn ứng dụng"
        panel.allowedContentTypes = [.application]
        panel.allowsMultipleSelection = false
        panel.directoryURL = URL(fileURLWithPath: "/Applications")

        if panel.runModal() == .OK, let url = panel.url {
            let name = url.deletingPathExtension().lastPathComponent
            let icon = NSWorkspace.shared.icon(forFile: url.path)
            let bundleId = Bundle(url: url)?.bundleIdentifier ?? url.lastPathComponent

            // Check if already added
            if !appState.excludedApps.contains(where: { $0.bundleId == bundleId }) {
                appState.excludedApps.append(ExcludedApp(bundleId: bundleId, name: name, icon: icon))
            }
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

                Text("Made with ❤️ in Vietnam")
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

struct ShortcutRecorderRow: View {
    @Binding var shortcut: KeyboardShortcut
    @Binding var isRecording: Bool

    @State private var hovered = false
    @State private var didCancel = false
    @State private var keyMonitor: Any?
    @State private var globalMonitor: Any?
    @State private var clickMonitor: Any?
    @State private var focusObserver: Any?

    var body: some View {
        HStack {
            Text("Phím tắt bật/tắt")
                .font(.system(size: 13))
                .foregroundColor(Color(NSColor.labelColor))
            Spacer()

            ZStack {
                if isRecording {
                    HStack(spacing: 4) {
                        Circle()
                            .fill(Color.accentColor)
                            .frame(width: 6, height: 6)
                        Text("Nhấn phím...")
                            .font(.system(size: 11, weight: .medium))
                    }
                    .foregroundColor(Color.accentColor)
                    .padding(.horizontal, 6)
                    .padding(.vertical, 3)
                } else {
                    HStack(spacing: 4) {
                        ForEach(shortcut.displayParts, id: \.self) { part in
                            KeyCap(text: part)
                        }
                    }
                }
            }
            .frame(minWidth: 80, alignment: .trailing)
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 10)
        .background(hovered ? Color(NSColor.controlBackgroundColor).opacity(0.3) : Color.clear)
        .contentShape(Rectangle())
        .onHover { hovered = $0 }
        .onTapGesture {
            if didCancel {
                didCancel = false
                return
            }
            if !isRecording { startRecording() }
        }
        .onDisappear { stopRecording() }
    }

    private func startRecording() {
        guard !isRecording else { return }
        isRecording = true

        keyMonitor = NSEvent.addLocalMonitorForEvents(matching: .keyDown) { event in
            handleKey(event)
            return nil
        }

        globalMonitor = NSEvent.addGlobalMonitorForEvents(matching: .keyDown) { event in
            handleKey(event)
        }

        clickMonitor = NSEvent.addLocalMonitorForEvents(matching: [.leftMouseDown, .rightMouseDown]) { event in
            stopRecording()
            return event
        }

        focusObserver = NotificationCenter.default.addObserver(
            forName: NSWindow.didResignKeyNotification,
            object: nil,
            queue: .main
        ) { _ in
            stopRecording()
        }
    }

    private func handleKey(_ event: NSEvent) {
        if event.keyCode == 0x35 {
            stopRecording()
            return
        }

        let modifiers = event.modifierFlags.intersection([.control, .option, .shift, .command])
        guard !modifiers.isEmpty else { return }

        var flags: UInt64 = 0
        if modifiers.contains(.control) { flags |= CGEventFlags.maskControl.rawValue }
        if modifiers.contains(.option) { flags |= CGEventFlags.maskAlternate.rawValue }
        if modifiers.contains(.shift) { flags |= CGEventFlags.maskShift.rawValue }
        if modifiers.contains(.command) { flags |= CGEventFlags.maskCommand.rawValue }

        shortcut = KeyboardShortcut(keyCode: event.keyCode, modifiers: flags)
        stopRecording()
    }

    private func stopRecording() {
        didCancel = true

        if let monitor = keyMonitor {
            NSEvent.removeMonitor(monitor)
            keyMonitor = nil
        }
        if let monitor = globalMonitor {
            NSEvent.removeMonitor(monitor)
            globalMonitor = nil
        }
        if let monitor = clickMonitor {
            NSEvent.removeMonitor(monitor)
            clickMonitor = nil
        }
        if let observer = focusObserver {
            NotificationCenter.default.removeObserver(observer)
            focusObserver = nil
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
    var onSelect: () -> Void
    @FocusState private var isFocused: Bool

    var body: some View {
        HStack(spacing: 8) {
            TextField("viết tắt", text: $shortcut.key)
                .font(.system(size: 13, weight: .medium, design: .monospaced))
                .textFieldStyle(.plain)
                .frame(width: 60)
                .focused($isFocused)
            Text("→")
                .font(.system(size: 11))
                .foregroundColor(Color(NSColor.tertiaryLabelColor))
            TextField("nội dung", text: $shortcut.value)
                .font(.system(size: 13))
                .textFieldStyle(.plain)
                .focused($isFocused)
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
        .onChange(of: isFocused) { focused in
            if focused { onSelect() }
        }
    }
}

struct ExcludedAppRow: View {
    @Binding var app: ExcludedApp
    var isSelected: Bool
    var onSelect: () -> Void

    var body: some View {
        HStack(spacing: 12) {
            if let icon = app.icon {
                Image(nsImage: icon)
                    .resizable()
                    .frame(width: 24, height: 24)
            } else {
                Image(systemName: "app.fill")
                    .font(.system(size: 20))
                    .foregroundColor(Color(NSColor.secondaryLabelColor))
                    .frame(width: 24, height: 24)
            }

            Text(app.name)
                .font(.system(size: 13))
                .foregroundColor(Color(NSColor.labelColor))
                .lineLimit(1)

            Spacer()

            Toggle("", isOn: $app.isEnabled)
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

struct AddRemoveButtons: View {
    let onAdd: () -> Void
    let onRemove: () -> Void
    let removeDisabled: Bool

    var body: some View {
        HStack(spacing: 0) {
            Button(action: onAdd) {
                Image(systemName: "plus")
                    .frame(width: 24, height: 24)
            }
            .buttonStyle(.borderless)

            Divider().frame(height: 16)

            Button(action: onRemove) {
                Image(systemName: "minus")
                    .frame(width: 24, height: 24)
            }
            .buttonStyle(.borderless)
            .disabled(removeDisabled)

            Spacer()
        }
        .padding(.horizontal, 8)
        .padding(.vertical, 4)
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
