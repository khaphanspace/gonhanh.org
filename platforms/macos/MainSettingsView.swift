import SwiftUI

// MARK: - Navigation

enum NavigationPage: String, CaseIterable {
    case home = "Trang chủ"
    case settings = "Cài đặt"
    case about = "Giới thiệu"
    case update = "Cập nhật"
}

enum SettingsTab: String, CaseIterable {
    case method = "Kiểu gõ"
    case shortcut = "Gõ tắt"
    case exclude = "Ngoại lệ"
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

    var body: some View {
        HStack(spacing: 0) {
            // Sidebar
            sidebar

            // Content
            content
        }
        .frame(width: 700, height: 520)
        .background(Color(hex: "282828"))
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
                    .foregroundColor(.white)

                // Version + Update badge
                HStack(spacing: 12) {
                    Text(AppMetadata.version)
                        .font(.system(size: 12))
                        .foregroundColor(Color(hex: "6b6b6b"))

                    if case .available = updateManager.state {
                        updateBadge
                    }
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
        }
        .frame(width: 200)
        .background(Color(hex: "1e1e1e"))
    }

    private var updateBadge: some View {
        Button {
            selectedPage = .update
        } label: {
            HStack(spacing: 6) {
                Circle()
                    .fill(Color(hex: "30d158"))
                    .frame(width: 6, height: 6)
                Text("Cập nhật")
                    .font(.system(size: 11))
            }
            .padding(.horizontal, 10)
            .padding(.vertical, 4)
            .background(Color(hex: "30d158").opacity(0.15))
            .foregroundColor(Color(hex: "30d158"))
            .cornerRadius(20)
        }
        .buttonStyle(.plain)
    }

    private func navButton(page: NavigationPage) -> some View {
        Button {
            selectedPage = page
        } label: {
            Text(page.rawValue)
                .font(.system(size: 14))
                .foregroundColor(selectedPage == page ? .white : Color(hex: "9a9a9a"))
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding(.horizontal, 12)
                .padding(.vertical, 8)
                .background(
                    RoundedRectangle(cornerRadius: 8)
                        .fill(selectedPage == page ? Color.white.opacity(0.1) : Color.clear)
                )
        }
        .buttonStyle(.plain)
    }

    // MARK: - Content

    @ViewBuilder
    private var content: some View {
        ScrollView {
            switch selectedPage {
            case .home:
                HomePageView(appState: appState, onNavigate: { selectedPage = $0 })
            case .settings:
                SettingsPageView(appState: appState)
            case .about:
                AboutPageView()
            case .update:
                UpdatePageView()
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .padding(32)
    }
}

// MARK: - Home Page

struct HomePageView: View {
    @ObservedObject var appState: AppState
    var onNavigate: (NavigationPage) -> Void
    @State private var selectedSettingsTab: SettingsTab = .method

    var body: some View {
        VStack(spacing: 24) {
            // Status Card
            statusCard

            // Quick Info - Input Method
            quickMethodCard

            // Quick Stats
            HStack(spacing: 12) {
                quickStatCard(title: "Gõ tắt", value: "\(appState.shortcuts.count) từ") {
                    selectedSettingsTab = .shortcut
                    onNavigate(.settings)
                }
                quickStatCard(title: "Ngoại lệ", value: "\(appState.excludedApps.count) app") {
                    selectedSettingsTab = .exclude
                    onNavigate(.settings)
                }
            }

            // Tips
            tipsSection

            Spacer()
        }
    }

    private var statusCard: some View {
        HStack {
            HStack(spacing: 16) {
                Circle()
                    .fill(appState.isEnabled ? Color(hex: "30d158") : Color(hex: "ff453a"))
                    .frame(width: 12, height: 12)

                VStack(alignment: .leading, spacing: 4) {
                    Text(appState.isEnabled ? "Đang hoạt động" : "Đã tắt")
                        .font(.system(size: 14, weight: .medium))
                        .foregroundColor(.white)

                    HStack(spacing: 4) {
                        KeyboardBadge(text: "⌃")
                        KeyboardBadge(text: "Space")
                        Text("để bật/tắt")
                            .font(.system(size: 12))
                            .foregroundColor(Color(hex: "6b6b6b"))
                    }
                }
            }

            Spacer()

            Toggle("", isOn: $appState.isEnabled)
                .toggleStyle(MacToggleStyle())
        }
        .padding(20)
        .background(
            RoundedRectangle(cornerRadius: 16)
                .fill(
                    LinearGradient(
                        colors: appState.isEnabled
                            ? [Color(hex: "30d158").opacity(0.15), Color(hex: "30d158").opacity(0.05)]
                            : [Color(hex: "ff453a").opacity(0.15), Color(hex: "ff453a").opacity(0.05)],
                        startPoint: .topLeading,
                        endPoint: .bottomTrailing
                    )
                )
                .overlay(
                    RoundedRectangle(cornerRadius: 16)
                        .stroke(
                            appState.isEnabled ? Color(hex: "30d158").opacity(0.25) : Color(hex: "ff453a").opacity(0.25),
                            lineWidth: 1
                        )
                )
        )
    }

    private var quickMethodCard: some View {
        Button {
            onNavigate(.settings)
        } label: {
            HStack {
                HStack(spacing: 12) {
                    RoundedRectangle(cornerRadius: 12)
                        .fill(appState.currentMethod == .telex ? Color.blue : Color.orange)
                        .frame(width: 40, height: 40)
                        .overlay(
                            Text(appState.currentMethod.shortName)
                                .font(.system(size: 16, weight: .semibold))
                                .foregroundColor(.white)
                        )

                    VStack(alignment: .leading, spacing: 2) {
                        Text("Kiểu gõ")
                            .font(.system(size: 12))
                            .foregroundColor(Color(hex: "6b6b6b"))
                        Text(appState.currentMethod.name)
                            .font(.system(size: 14, weight: .medium))
                            .foregroundColor(.white)
                    }
                }

                Spacer()

                Text("→")
                    .foregroundColor(Color(hex: "6b6b6b"))
            }
            .padding(16)
            .background(
                RoundedRectangle(cornerRadius: 12)
                    .fill(Color.white.opacity(0.05))
            )
        }
        .buttonStyle(.plain)
    }

    private func quickStatCard(title: String, value: String, action: @escaping () -> Void) -> some View {
        Button(action: action) {
            VStack(alignment: .leading, spacing: 4) {
                Text(title)
                    .font(.system(size: 12))
                    .foregroundColor(Color(hex: "6b6b6b"))
                Text(value)
                    .font(.system(size: 16, weight: .medium))
                    .foregroundColor(.white)
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(16)
            .background(
                RoundedRectangle(cornerRadius: 12)
                    .fill(Color.white.opacity(0.05))
            )
        }
        .buttonStyle(.plain)
    }

    private var tipsSection: some View {
        VStack(alignment: .leading, spacing: 12) {
            Divider()
                .background(Color.white.opacity(0.1))

            Text("Mẹo gõ nhanh")
                .font(.system(size: 12))
                .foregroundColor(Color(hex: "6b6b6b"))

            HStack(spacing: 16) {
                tipItem(key: "aa", result: "â")
                tipItem(key: "dd", result: "đ")
                tipItem(key: "oo", result: "ô")
                tipItem(key: "s", result: "sắc")
                tipItem(key: "f", result: "huyền")
            }
        }
    }

    private func tipItem(key: String, result: String) -> some View {
        HStack(spacing: 4) {
            KeyboardBadge(text: key)
            Text("→ \(result)")
                .font(.system(size: 14))
                .foregroundColor(Color(hex: "9a9a9a"))
        }
    }
}

// MARK: - Settings Page

struct SettingsPageView: View {
    @ObservedObject var appState: AppState
    @State private var selectedTab: SettingsTab = .method

    var body: some View {
        VStack(alignment: .leading, spacing: 24) {
            // Tab bar
            HStack(spacing: 4) {
                ForEach(SettingsTab.allCases, id: \.self) { tab in
                    tabButton(tab)
                }
            }

            // Content
            switch selectedTab {
            case .method:
                methodContent
            case .shortcut:
                shortcutContent
            case .exclude:
                excludeContent
            }

            Spacer()
        }
    }

    private func tabButton(_ tab: SettingsTab) -> some View {
        Button {
            selectedTab = tab
        } label: {
            Text(tab.rawValue)
                .font(.system(size: 13))
                .foregroundColor(selectedTab == tab ? .white : Color(hex: "888888"))
                .padding(.horizontal, 12)
                .padding(.vertical, 6)
                .background(
                    RoundedRectangle(cornerRadius: 6)
                        .fill(selectedTab == tab ? Color.white.opacity(0.1) : Color.clear)
                )
        }
        .buttonStyle(.plain)
    }

    // MARK: - Method Content

    private var methodContent: some View {
        VStack(spacing: 8) {
            methodOption(
                method: .telex,
                description: "aa → â · dd → đ · s → sắc"
            )
            methodOption(
                method: .vni,
                description: "a6 → â · d9 → đ · 1 → sắc"
            )
        }
    }

    private func methodOption(method: InputMode, description: String) -> some View {
        Button {
            appState.currentMethod = method
        } label: {
            HStack {
                VStack(alignment: .leading, spacing: 4) {
                    Text(method.name)
                        .font(.system(size: 14, weight: .medium))
                        .foregroundColor(.white)
                    Text(description)
                        .font(.system(size: 12))
                        .foregroundColor(Color(hex: "6b6b6b"))
                }

                Spacer()

                if appState.currentMethod == method {
                    Image(systemName: "checkmark")
                        .font(.system(size: 14, weight: .semibold))
                        .foregroundColor(.blue)
                }
            }
            .padding(16)
            .background(
                RoundedRectangle(cornerRadius: 10)
                    .fill(appState.currentMethod == method ? Color.blue.opacity(0.15) : Color.clear)
                    .overlay(
                        RoundedRectangle(cornerRadius: 10)
                            .stroke(
                                appState.currentMethod == method ? Color.blue : Color.clear,
                                lineWidth: 1
                            )
                    )
            )
        }
        .buttonStyle(.plain)
    }

    // MARK: - Shortcut Content

    private var shortcutContent: some View {
        VStack(alignment: .leading, spacing: 16) {
            HStack {
                Spacer()
                Button("Thêm") {
                    // TODO: Add shortcut
                }
                .foregroundColor(.blue)
                .font(.system(size: 14))
            }

            VStack(spacing: 0) {
                ForEach(Array(appState.shortcuts.enumerated()), id: \.element.id) { index, shortcut in
                    listRow {
                        HStack {
                            Text(shortcut.key)
                                .foregroundColor(Color(hex: "6b6b6b"))
                            Text("→ \(shortcut.value)")
                                .foregroundColor(.white)

                            Spacer()

                            Button("Xoá") {
                                appState.shortcuts.remove(at: index)
                            }
                            .foregroundColor(.red)
                            .font(.system(size: 12))
                        }
                    }

                    if index < appState.shortcuts.count - 1 {
                        Divider()
                            .background(Color.white.opacity(0.06))
                    }
                }
            }
            .background(
                RoundedRectangle(cornerRadius: 12)
                    .fill(Color.white.opacity(0.05))
            )

            Text("Gõ từ viết tắt để chèn nhanh cụm từ.")
                .font(.system(size: 12))
                .foregroundColor(Color(hex: "4a4a4a"))
        }
    }

    // MARK: - Exclude Content

    private var excludeContent: some View {
        VStack(alignment: .leading, spacing: 16) {
            HStack {
                Spacer()
                Button("Thêm") {
                    // TODO: Add excluded app
                }
                .foregroundColor(.blue)
                .font(.system(size: 14))
            }

            VStack(spacing: 0) {
                ForEach(Array(appState.excludedApps.enumerated()), id: \.offset) { index, app in
                    listRow {
                        HStack {
                            Text(app)
                                .foregroundColor(.white)

                            Spacer()

                            Button("Xoá") {
                                appState.excludedApps.remove(at: index)
                            }
                            .foregroundColor(.red)
                            .font(.system(size: 12))
                        }
                    }

                    if index < appState.excludedApps.count - 1 {
                        Divider()
                            .background(Color.white.opacity(0.06))
                    }
                }
            }
            .background(
                RoundedRectangle(cornerRadius: 12)
                    .fill(Color.white.opacity(0.05))
            )

            Text("Tắt bộ gõ trong các app này.")
                .font(.system(size: 12))
                .foregroundColor(Color(hex: "4a4a4a"))
        }
    }

    private func listRow<Content: View>(@ViewBuilder content: () -> Content) -> some View {
        content()
            .font(.system(size: 14))
            .padding(.horizontal, 16)
            .padding(.vertical, 12)
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
                    .foregroundColor(.white)

                Text("Bộ gõ tiếng Việt cho macOS")
                    .font(.system(size: 14))
                    .foregroundColor(Color(hex: "6b6b6b"))

                Text("Version \(AppMetadata.version) (Build \(AppMetadata.buildNumber))")
                    .font(.system(size: 12))
                    .foregroundColor(Color(hex: "4a4a4a"))
            }

            // Author + Sponsor
            HStack {
                VStack(alignment: .leading, spacing: 4) {
                    Text("Tác giả")
                        .font(.system(size: 12))
                        .foregroundColor(Color(hex: "6b6b6b"))

                    Link(destination: URL(string: AppMetadata.authorLinkedin)!) {
                        HStack(spacing: 8) {
                            Text(AppMetadata.author)
                                .foregroundColor(.white)
                            Image(systemName: "link")
                                .foregroundColor(Color(hex: "0a66c2"))
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
                    .background(Color(hex: "db2777").opacity(0.15))
                    .foregroundColor(Color(hex: "f472b6"))
                    .cornerRadius(6)
                }
            }
            .padding(16)
            .background(
                RoundedRectangle(cornerRadius: 12)
                    .fill(Color.white.opacity(0.05))
            )

            // Links
            VStack(spacing: 0) {
                linkRow(icon: "link", title: "GitHub", url: AppMetadata.repository)
                Divider().background(Color.white.opacity(0.06))
                linkRow(icon: "book", title: "Hướng dẫn sử dụng", url: AppMetadata.website)
                Divider().background(Color.white.opacity(0.06))
                linkRow(icon: "exclamationmark.bubble", title: "Góp ý & Báo lỗi", url: AppMetadata.issuesURL)
            }
            .background(
                RoundedRectangle(cornerRadius: 12)
                    .fill(Color.white.opacity(0.05))
            )

            // Footer
            Text("Made with ❤️ in Vietnam")
                .font(.system(size: 12))
                .foregroundColor(Color(hex: "4a4a4a"))

            Spacer()
        }
    }

    private func linkRow(icon: String, title: String, url: String) -> some View {
        Link(destination: URL(string: url)!) {
            HStack {
                Image(systemName: icon)
                    .font(.system(size: 14))
                    .foregroundColor(Color(hex: "9a9a9a"))
                    .frame(width: 20)

                Text(title)
                    .font(.system(size: 14))
                    .foregroundColor(.white)

                Spacer()

                Text("→")
                    .foregroundColor(Color(hex: "6b6b6b"))
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
                .foregroundColor(.white)

            VStack(spacing: 16) {
                HStack {
                    VStack(alignment: .leading, spacing: 4) {
                        Text("Gõ Nhanh 2.2.0")
                            .font(.system(size: 14, weight: .medium))
                            .foregroundColor(.white)
                        Text("8.2 MB")
                            .font(.system(size: 12))
                            .foregroundColor(Color(hex: "6b6b6b"))
                    }

                    Spacer()

                    Button("Cập nhật") {
                        // TODO: Update
                    }
                    .buttonStyle(.borderedProminent)
                }

                Divider()
                    .background(Color.white.opacity(0.1))

                VStack(alignment: .leading, spacing: 8) {
                    Text("Có gì mới")
                        .font(.system(size: 12))
                        .foregroundColor(Color(hex: "6b6b6b"))

                    Text("Hỗ trợ macOS Sequoia\nCải thiện tốc độ\nSửa lỗi Safari")
                        .font(.system(size: 14))
                        .foregroundColor(.white)
                        .lineSpacing(4)
                }
            }
            .padding(20)
            .background(
                RoundedRectangle(cornerRadius: 12)
                    .fill(Color.white.opacity(0.05))
            )

            Spacer()
        }
    }
}

// MARK: - Components

struct KeyboardBadge: View {
    let text: String

    var body: some View {
        Text(text)
            .font(.system(size: 11, design: .monospaced))
            .foregroundColor(Color(hex: "9a9a9a"))
            .padding(.horizontal, 7)
            .padding(.vertical, 3)
            .background(
                RoundedRectangle(cornerRadius: 5)
                    .fill(Color.white.opacity(0.1))
            )
    }
}

struct MacToggleStyle: ToggleStyle {
    func makeBody(configuration: Configuration) -> some View {
        Button {
            configuration.isOn.toggle()
        } label: {
            RoundedRectangle(cornerRadius: 16)
                .fill(configuration.isOn ? Color(hex: "30d158") : Color(hex: "787880").opacity(0.32))
                .frame(width: 51, height: 31)
                .overlay(
                    Circle()
                        .fill(.white)
                        .frame(width: 27, height: 27)
                        .shadow(color: .black.opacity(0.2), radius: 2, y: 1)
                        .offset(x: configuration.isOn ? 10 : -10)
                        .animation(.easeInOut(duration: 0.2), value: configuration.isOn)
                )
        }
        .buttonStyle(.plain)
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
