import SwiftUI

// MARK: - Navigation

enum NavigationPage: String, CaseIterable {
    case home = "Trang chủ"
    case settings = "Cài đặt"
    case about = "Giới thiệu"

    var icon: String {
        switch self {
        case .home: return "house"
        case .settings: return "gearshape"
        case .about: return "info.circle"
        }
    }

    var accessibilityHint: String {
        switch self {
        case .home: return "Xem trạng thái và thống kê"
        case .settings: return "Thay đổi kiểu gõ, gõ tắt và ngoại lệ"
        case .about: return "Thông tin về ứng dụng"
        }
    }
}

enum SettingsTab: String, CaseIterable {
    case method = "Kiểu gõ"
    case shortcut = "Gõ tắt"
    case exclude = "Ngoại lệ"

    var icon: String {
        switch self {
        case .method: return "keyboard"
        case .shortcut: return "text.badge.plus"
        case .exclude: return "xmark.app"
        }
    }
}

// MARK: - App State

class AppState: ObservableObject {
    static let shared = AppState()

    @Published var isEnabled: Bool {
        didSet {
            UserDefaults.standard.set(isEnabled, forKey: SettingsKey.enabled)
            RustBridge.setEnabled(isEnabled)
        }
    }

    @Published var currentMethod: InputMode {
        didSet {
            UserDefaults.standard.set(currentMethod.rawValue, forKey: SettingsKey.method)
            RustBridge.setMethod(currentMethod.rawValue)
        }
    }

    @Published var shortcuts: [ShortcutItem] = [
        ShortcutItem(key: "vn", value: "Việt Nam"),
        ShortcutItem(key: "hcm", value: "Hồ Chí Minh"),
        ShortcutItem(key: "sdt", value: "0912 345 678")
    ]

    @Published var excludedApps: [String] = ["Terminal", "Visual Studio Code"]

    init() {
        isEnabled = UserDefaults.standard.object(forKey: SettingsKey.enabled) as? Bool ?? true
        currentMethod = InputMode(rawValue: UserDefaults.standard.integer(forKey: SettingsKey.method)) ?? .telex
    }
}

struct ShortcutItem: Identifiable {
    let id = UUID()
    var key: String
    var value: String
}

// MARK: - Main Settings View

struct MainSettingsView: View {
    @StateObject private var appState = AppState.shared
    @ObservedObject var updateManager = UpdateManager.shared
    @State private var selectedPage: NavigationPage = .home
    @Environment(\.colorScheme) private var colorScheme

    private var isDark: Bool { colorScheme == .dark }

    var body: some View {
        HStack(spacing: 0) {
            // Sidebar with darker material (like macOS Settings)
            ZStack {
                VisualEffectBackground(material: .sidebar, blendingMode: .behindWindow)
                sidebar
            }
            .frame(width: 200)

            // Content with lighter background (like macOS Settings content area)
            ZStack {
                // Light mode: white, Dark mode: slightly lighter than sidebar
                if isDark {
                    VisualEffectBackground(material: .headerView, blendingMode: .behindWindow)
                } else {
                    Color(NSColor.windowBackgroundColor)
                }
                content
            }
        }
        .ignoresSafeArea()
        .frame(width: 700, height: 520)
    }

    // MARK: - Sidebar

    private var sidebar: some View {
        VStack(spacing: 0) {
            // Logo
            VStack(spacing: 16) {
                Image(nsImage: AppMetadata.logo)
                    .resizable()
                    .frame(width: 96, height: 96)

                Text(AppMetadata.name)
                    .font(.system(size: 24, weight: .bold))
                    .foregroundColor(Color(NSColor.labelColor))

                // Version + Update badge
                HStack(spacing: 12) {
                    Text(AppMetadata.version)
                        .font(.system(size: 12))
                        .foregroundColor(Color(NSColor.tertiaryLabelColor))

                    versionBadge
                }
            }
            .padding(.top, 32)
            .padding(.horizontal, 20)

            Spacer()

            // Navigation
            VStack(spacing: 4) {
                navButton(page: .home)
                navButton(page: .settings)
                navButton(page: .about)
            }
            .padding(12)
            .padding(.bottom, 16)
        }
        .frame(width: 200)
    }

    @ViewBuilder
    private var versionBadge: some View {
        if case .available = updateManager.state {
            Button {
                selectedPage = .update
            } label: {
                HStack(spacing: 6) {
                    Circle()
                        .fill(Color.green)
                        .frame(width: 6, height: 6)
                    Text("Cập nhật")
                        .font(.system(size: 11))
                }
                .padding(.horizontal, 10)
                .padding(.vertical, 4)
                .background(Color.green.opacity(0.15))
                .foregroundColor(Color.green)
                .cornerRadius(20)
            }
            .buttonStyle(.plain)
        } else {
            HStack(spacing: 6) {
                Circle()
                    .fill(Color(NSColor.tertiaryLabelColor))
                    .frame(width: 6, height: 6)
                Text("Mới nhất")
                    .font(.system(size: 11))
            }
            .padding(.horizontal, 10)
            .padding(.vertical, 4)
            .background(Color(NSColor.quaternaryLabelColor).opacity(0.5))
            .foregroundColor(Color(NSColor.tertiaryLabelColor))
            .cornerRadius(20)
        }
    }

    private func navButton(page: NavigationPage) -> some View {
        Button {
            selectedPage = page
        } label: {
            HStack(spacing: 10) {
                Image(systemName: page.icon)
                    .font(.system(size: 14))
                    .foregroundColor(selectedPage == page ? .accentColor : Color(NSColor.secondaryLabelColor))
                    .frame(width: 20)
                Text(page.rawValue)
                    .font(.system(size: 13))
                    .foregroundColor(selectedPage == page ? Color(NSColor.labelColor) : Color(NSColor.secondaryLabelColor))
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(.horizontal, 12)
            .padding(.vertical, 8)
            .background(
                RoundedRectangle(cornerRadius: 8)
                    .fill(selectedPage == page ? Color(NSColor.controlBackgroundColor).opacity(0.5) : Color.clear)
            )
        }
        .buttonStyle(.plain)
        .accessibilityLabel(page.rawValue)
        .accessibilityHint(page.accessibilityHint)
        .accessibilityAddTraits(selectedPage == page ? .isSelected : [])
    }

    // MARK: - Content

    @ViewBuilder
    private var content: some View {
        ScrollView {
            switch selectedPage {
            case .home:
                HomePageView(appState: appState)
            case .settings:
                SettingsPageView(appState: appState)
            case .about:
                AboutPageView()
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .padding(32)
    }
}

// MARK: - Home Page (HIG Compliant)

struct HomePageView: View {
    @ObservedObject var appState: AppState
    var onNavigate: (NavigationPage) -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 20) {
            // Section: Trạng thái
            HIGSection(header: "Trạng thái") {
                HIGRow {
                    HStack(spacing: 12) {
                        Circle()
                            .fill(appState.isEnabled ? Color.green : Color.red)
                            .frame(width: 8, height: 8)
                        Text(appState.isEnabled ? "Đang hoạt động" : "Đã tắt")
                    }
                    Spacer()
                    Toggle("", isOn: $appState.isEnabled)
                        .toggleStyle(.switch)
                        .labelsHidden()
                }

                HIGRow {
                    Text("Phím tắt bật/tắt")
                    Spacer()
                    HStack(spacing: 4) {
                        KeyboardBadge(text: "⌃")
                        KeyboardBadge(text: "Space")
                    }
                }
            }

            // Section: Kiểu gõ
            HIGSection(header: "Kiểu gõ") {
                HIGRow(action: { onNavigate(.settings) }) {
                    HStack(spacing: 12) {
                        Image(systemName: "keyboard")
                            .font(.system(size: 16))
                            .foregroundColor(.accentColor)
                            .frame(width: 24)
                        Text("Phương thức")
                    }
                    Spacer()
                    Text(appState.currentMethod.name)
                        .foregroundColor(Color(NSColor.secondaryLabelColor))
                    Image(systemName: "chevron.right")
                        .font(.system(size: 12, weight: .semibold))
                        .foregroundColor(Color(NSColor.tertiaryLabelColor))
                }
            }

            // Section: Thống kê
            HIGSection(header: "Thống kê") {
                HIGRow(action: { onNavigate(.settings) }) {
                    HStack(spacing: 12) {
                        Image(systemName: "text.badge.plus")
                            .font(.system(size: 16))
                            .foregroundColor(.orange)
                            .frame(width: 24)
                        Text("Gõ tắt")
                    }
                    Spacer()
                    Text("\(appState.shortcuts.count) từ")
                        .foregroundColor(Color(NSColor.secondaryLabelColor))
                    Image(systemName: "chevron.right")
                        .font(.system(size: 12, weight: .semibold))
                        .foregroundColor(Color(NSColor.tertiaryLabelColor))
                }

                HIGRow(action: { onNavigate(.settings) }) {
                    HStack(spacing: 12) {
                        Image(systemName: "xmark.app")
                            .font(.system(size: 16))
                            .foregroundColor(.red)
                            .frame(width: 24)
                        Text("Ngoại lệ")
                    }
                    Spacer()
                    Text("\(appState.excludedApps.count) app")
                        .foregroundColor(Color(NSColor.secondaryLabelColor))
                    Image(systemName: "chevron.right")
                        .font(.system(size: 12, weight: .semibold))
                        .foregroundColor(Color(NSColor.tertiaryLabelColor))
                }
            }

            // Section: Mẹo gõ nhanh
            HIGSection(header: "Mẹo gõ nhanh (\(appState.currentMethod.name))") {
                if appState.currentMethod == .telex {
                    tipsGrid(tips: [
                        ("aa", "â"), ("aw", "ă"), ("dd", "đ"),
                        ("ee", "ê"), ("oo", "ô"), ("ow", "ơ"),
                        ("s", "sắc"), ("f", "huyền"), ("r", "hỏi")
                    ])
                } else {
                    tipsGrid(tips: [
                        ("a6", "â"), ("a8", "ă"), ("d9", "đ"),
                        ("e6", "ê"), ("o6", "ô"), ("o7", "ơ"),
                        ("1", "sắc"), ("2", "huyền"), ("3", "hỏi")
                    ])
                }
            }

            Spacer()
        }
    }

    private func tipsGrid(tips: [(String, String)]) -> some View {
        LazyVGrid(columns: [
            GridItem(.flexible()),
            GridItem(.flexible()),
            GridItem(.flexible())
        ], spacing: 8) {
            ForEach(tips, id: \.0) { tip in
                HStack(spacing: 6) {
                    KeyboardBadge(text: tip.0)
                    Text("→")
                        .font(.system(size: 11))
                        .foregroundColor(Color(NSColor.tertiaryLabelColor))
                    Text(tip.1)
                        .font(.system(size: 13))
                        .foregroundColor(Color(NSColor.secondaryLabelColor))
                }
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding(.vertical, 6)
            }
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 8)
    }
}

// MARK: - Settings Page (HIG Compliant)

struct SettingsPageView: View {
    @ObservedObject var appState: AppState

    var body: some View {
        VStack(alignment: .leading, spacing: 20) {
            // Section: Kiểu gõ
            HIGSection(header: "Kiểu gõ") {
                HIGRow(action: { appState.currentMethod = .telex }) {
                    HStack(spacing: 12) {
                        RoundedRectangle(cornerRadius: 6)
                            .fill(Color.blue)
                            .frame(width: 28, height: 28)
                            .overlay(
                                Text("T")
                                    .font(.system(size: 14, weight: .bold))
                                    .foregroundColor(.white)
                            )
                        VStack(alignment: .leading, spacing: 2) {
                            Text("Telex")
                                .font(.system(size: 13))
                            Text("aa → â · dd → đ · s → sắc")
                                .font(.system(size: 11))
                                .foregroundColor(Color(NSColor.secondaryLabelColor))
                        }
                    }
                    Spacer()
                    if appState.currentMethod == .telex {
                        Image(systemName: "checkmark")
                            .font(.system(size: 14, weight: .semibold))
                            .foregroundColor(.accentColor)
                    }
                }

                HIGRow(action: { appState.currentMethod = .vni }) {
                    HStack(spacing: 12) {
                        RoundedRectangle(cornerRadius: 6)
                            .fill(Color.orange)
                            .frame(width: 28, height: 28)
                            .overlay(
                                Text("V")
                                    .font(.system(size: 14, weight: .bold))
                                    .foregroundColor(.white)
                            )
                        VStack(alignment: .leading, spacing: 2) {
                            Text("VNI")
                                .font(.system(size: 13))
                            Text("a6 → â · d9 → đ · 1 → sắc")
                                .font(.system(size: 11))
                                .foregroundColor(Color(NSColor.secondaryLabelColor))
                        }
                    }
                    Spacer()
                    if appState.currentMethod == .vni {
                        Image(systemName: "checkmark")
                            .font(.system(size: 14, weight: .semibold))
                            .foregroundColor(.accentColor)
                    }
                }
            }

            // Section: Gõ tắt
            HIGSection(header: "Gõ tắt", headerTrailing: {
                Button(action: { /* TODO: Add shortcut */ }) {
                    Image(systemName: "plus")
                        .font(.system(size: 12, weight: .medium))
                }
                .buttonStyle(.borderless)
            }) {
                ForEach(appState.shortcuts) { shortcut in
                    HIGRow {
                        Text(shortcut.key)
                            .font(.system(size: 13, weight: .medium, design: .monospaced))
                            .frame(width: 50, alignment: .leading)
                        Image(systemName: "arrow.right")
                            .font(.system(size: 10))
                            .foregroundColor(Color(NSColor.tertiaryLabelColor))
                        Text(shortcut.value)
                            .font(.system(size: 13))
                        Spacer()
                        Button(action: {
                            if let index = appState.shortcuts.firstIndex(where: { $0.id == shortcut.id }) {
                                appState.shortcuts.remove(at: index)
                            }
                        }) {
                            Image(systemName: "minus.circle.fill")
                                .font(.system(size: 14))
                                .foregroundColor(Color(NSColor.tertiaryLabelColor))
                        }
                        .buttonStyle(.borderless)
                    }
                }

                if appState.shortcuts.isEmpty {
                    HIGRow {
                        Text("Chưa có từ viết tắt nào")
                            .font(.system(size: 13))
                            .foregroundColor(Color(NSColor.tertiaryLabelColor))
                    }
                }
            }

            // Section: Ngoại lệ
            HIGSection(header: "Ứng dụng ngoại lệ", headerTrailing: {
                Button(action: { /* TODO: Add app */ }) {
                    Image(systemName: "plus")
                        .font(.system(size: 12, weight: .medium))
                }
                .buttonStyle(.borderless)
            }) {
                ForEach(appState.excludedApps.indices, id: \.self) { index in
                    HIGRow {
                        Image(systemName: "app.fill")
                            .font(.system(size: 14))
                            .foregroundColor(Color(NSColor.secondaryLabelColor))
                        Text(appState.excludedApps[index])
                            .font(.system(size: 13))
                        Spacer()
                        Button(action: { appState.excludedApps.remove(at: index) }) {
                            Image(systemName: "minus.circle.fill")
                                .font(.system(size: 14))
                                .foregroundColor(Color(NSColor.tertiaryLabelColor))
                        }
                        .buttonStyle(.borderless)
                    }
                }

                if appState.excludedApps.isEmpty {
                    HIGRow {
                        Text("Chưa có ứng dụng nào")
                            .font(.system(size: 13))
                            .foregroundColor(Color(NSColor.tertiaryLabelColor))
                    }
                }
            }

            Text("Bộ gõ sẽ tự động tắt khi bạn sử dụng các ứng dụng trong danh sách ngoại lệ.")
                .font(.system(size: 11))
                .foregroundColor(Color(NSColor.tertiaryLabelColor))
                .padding(.horizontal, 4)

            Spacer()
        }
    }
}

// MARK: - About Page

struct AboutPageView: View {
    var body: some View {
        VStack(spacing: 24) {
            // Logo & Info
            VStack(spacing: 16) {
                Image(nsImage: AppMetadata.logo)
                    .resizable()
                    .frame(width: 80, height: 80)

                Text(AppMetadata.name)
                    .font(.system(size: 18, weight: .bold))
                    .foregroundColor(Color(NSColor.labelColor))

                Text("Bộ gõ tiếng Việt cho macOS")
                    .font(.system(size: 14))
                    .foregroundColor(Color(NSColor.tertiaryLabelColor))

                Text("Version \(AppMetadata.version) (Build \(AppMetadata.buildNumber))")
                    .font(.system(size: 12))
                    .foregroundColor(Color(NSColor.quaternaryLabelColor))
            }

            // Author + Sponsor
            HStack {
                VStack(alignment: .leading, spacing: 4) {
                    Text("Tác giả")
                        .font(.system(size: 12))
                        .foregroundColor(Color(NSColor.tertiaryLabelColor))

                    Link(destination: URL(string: AppMetadata.authorLinkedin)!) {
                        HStack(spacing: 8) {
                            Text(AppMetadata.author)
                                .foregroundColor(Color(NSColor.labelColor))
                            Image(systemName: "link")
                                .foregroundColor(.accentColor)
                        }
                        .font(.system(size: 14))
                    }
                }

                Spacer()

                Link(destination: URL(string: "https://github.com/sponsors/khaphanspace")!) {
                    HStack(spacing: 6) {
                        Image(systemName: "heart.fill")
                            .font(.system(size: 12))
                        Text("Sponsor")
                            .font(.system(size: 12))
                    }
                    .padding(.horizontal, 12)
                    .padding(.vertical, 6)
                    .background(Color.pink.opacity(0.15))
                    .foregroundColor(Color.pink)
                    .cornerRadius(6)
                }
            }
            .padding(16)
            .background(
                RoundedRectangle(cornerRadius: 12)
                    .fill(Color(NSColor.controlBackgroundColor).opacity(0.3))
            )

            // Links
            VStack(spacing: 0) {
                linkRow(icon: "link", title: "GitHub", url: AppMetadata.repository)
                Divider().background(Color(NSColor.separatorColor))
                linkRow(icon: "book", title: "Hướng dẫn sử dụng", url: AppMetadata.website)
                Divider().background(Color(NSColor.separatorColor))
                linkRow(icon: "exclamationmark.bubble", title: "Góp ý & Báo lỗi", url: AppMetadata.issuesURL)
            }
            .background(
                RoundedRectangle(cornerRadius: 12)
                    .fill(Color(NSColor.controlBackgroundColor).opacity(0.3))
            )

            // Footer
            Text("Made with ❤️ in Vietnam")
                .font(.system(size: 12))
                .foregroundColor(Color(NSColor.quaternaryLabelColor))

            Spacer()
        }
    }

    private func linkRow(icon: String, title: String, url: String) -> some View {
        Link(destination: URL(string: url)!) {
            HStack {
                Image(systemName: icon)
                    .font(.system(size: 14))
                    .foregroundColor(Color(NSColor.secondaryLabelColor))
                    .frame(width: 20)

                Text(title)
                    .font(.system(size: 14))
                    .foregroundColor(Color(NSColor.labelColor))

                Spacer()

                Image(systemName: "chevron.right")
                    .font(.system(size: 12, weight: .medium))
                    .foregroundColor(Color(NSColor.tertiaryLabelColor))
            }
            .padding(.horizontal, 16)
            .padding(.vertical, 12)
        }
        .buttonStyle(.plain)
    }
}

// MARK: - Update Page

struct UpdatePageView: View {
    @ObservedObject var updateManager = UpdateManager.shared

    var body: some View {
        VStack(alignment: .leading, spacing: 24) {
            Text("Cập nhật")
                .font(.system(size: 16, weight: .medium))
                .foregroundColor(Color(NSColor.labelColor))

            VStack(spacing: 16) {
                HStack {
                    VStack(alignment: .leading, spacing: 4) {
                        Text("Gõ Nhanh 2.2.0")
                            .font(.system(size: 14, weight: .medium))
                            .foregroundColor(Color(NSColor.labelColor))
                        Text("8.2 MB")
                            .font(.system(size: 12))
                            .foregroundColor(Color(NSColor.tertiaryLabelColor))
                    }

                    Spacer()

                    Button("Cập nhật") {
                        // TODO: Update
                    }
                    .buttonStyle(.borderedProminent)
                }

                Divider()
                    .background(Color(NSColor.separatorColor))

                VStack(alignment: .leading, spacing: 8) {
                    Text("Có gì mới")
                        .font(.system(size: 12))
                        .foregroundColor(Color(NSColor.tertiaryLabelColor))

                    Text("Hỗ trợ macOS Sequoia\nCải thiện tốc độ\nSửa lỗi Safari")
                        .font(.system(size: 14))
                        .foregroundColor(Color(NSColor.labelColor))
                        .lineSpacing(4)
                }
            }
            .padding(20)
            .background(
                RoundedRectangle(cornerRadius: 12)
                    .fill(Color(NSColor.controlBackgroundColor).opacity(0.3))
            )

            Spacer()
        }
    }
}

// MARK: - Window Control Button

enum WindowControlType {
    case close, minimize, zoom

    var color: Color {
        switch self {
        case .close: return Color(hex: "ff5f57")
        case .minimize: return Color(hex: "febc2e")
        case .zoom: return Color(hex: "28c840")
        }
    }

    var icon: String {
        switch self {
        case .close: return "xmark"
        case .minimize: return "minus"
        case .zoom: return "plus"
        }
    }
}

struct WindowControlButton: View {
    let type: WindowControlType
    @State private var isHovered = false

    var body: some View {
        Button {
            switch type {
            case .close:
                NSApp.keyWindow?.close()
            case .minimize:
                NSApp.keyWindow?.miniaturize(nil)
            case .zoom:
                NSApp.keyWindow?.zoom(nil)
            }
        } label: {
            ZStack {
                Circle()
                    .fill(type.color)
                    .frame(width: 12, height: 12)

                if isHovered {
                    Image(systemName: type.icon)
                        .font(.system(size: 7, weight: .bold))
                        .foregroundColor(.black.opacity(0.6))
                }
            }
        }
        .buttonStyle(.plain)
        .onHover { hovering in
            isHovered = hovering
        }
    }
}

// MARK: - Visual Effect Background

struct VisualEffectBackground: NSViewRepresentable {
    var material: NSVisualEffectView.Material = .hudWindow
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

// MARK: - HIG Components

/// HIG-compliant section container with header
struct HIGSection<Content: View, Trailing: View>: View {
    let header: String
    let headerTrailing: Trailing
    let content: Content

    init(header: String, @ViewBuilder headerTrailing: () -> Trailing = { EmptyView() }, @ViewBuilder content: () -> Content) {
        self.header = header
        self.headerTrailing = headerTrailing()
        self.content = content()
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            // Section header
            HStack {
                Text(header.uppercased())
                    .font(.system(size: 11, weight: .medium))
                    .foregroundColor(Color(NSColor.secondaryLabelColor))
                Spacer()
                headerTrailing
            }
            .padding(.horizontal, 4)

            // Section content with grouped background
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

/// HIG-compliant row for lists/forms
struct HIGRow<Content: View>: View {
    let action: (() -> Void)?
    let content: Content

    init(action: (() -> Void)? = nil, @ViewBuilder content: () -> Content) {
        self.action = action
        self.content = content()
    }

    var body: some View {
        Group {
            if let action = action {
                Button(action: action) {
                    rowContent
                }
                .buttonStyle(.plain)
            } else {
                rowContent
            }
        }
    }

    private var rowContent: some View {
        HStack(spacing: 8) {
            content
        }
        .font(.system(size: 13))
        .foregroundColor(Color(NSColor.labelColor))
        .padding(.horizontal, 12)
        .padding(.vertical, 10)
        .frame(maxWidth: .infinity, alignment: .leading)
        .contentShape(Rectangle())
    }
}

struct KeyboardBadge: View {
    let text: String

    var body: some View {
        Text(text)
            .font(.system(size: 11, weight: .medium, design: .rounded))
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

// MARK: - Color Extension

extension Color {
    init(hex: String) {
        let hex = hex.trimmingCharacters(in: CharacterSet.alphanumerics.inverted)
        var int: UInt64 = 0
        Scanner(string: hex).scanHexInt64(&int)
        let a, r, g, b: UInt64
        switch hex.count {
        case 3: // RGB (12-bit)
            (a, r, g, b) = (255, (int >> 8) * 17, (int >> 4 & 0xF) * 17, (int & 0xF) * 17)
        case 6: // RGB (24-bit)
            (a, r, g, b) = (255, int >> 16, int >> 8 & 0xFF, int & 0xFF)
        case 8: // ARGB (32-bit)
            (a, r, g, b) = (int >> 24, int >> 16 & 0xFF, int >> 8 & 0xFF, int & 0xFF)
        default:
            (a, r, g, b) = (255, 0, 0, 0)
        }
        self.init(
            .sRGB,
            red: Double(r) / 255,
            green: Double(g) / 255,
            blue: Double(b) / 255,
            opacity: Double(a) / 255
        )
    }
}

// MARK: - Preview

#Preview {
    MainSettingsView()
}
