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
        case .about: return "bolt.fill"
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

    @Published var excludedApps: [ExcludedApp] = [
        ExcludedApp(bundleId: "com.apple.Terminal", name: "Terminal", icon: NSWorkspace.shared.icon(forFile: "/System/Applications/Utilities/Terminal.app")),
        ExcludedApp(bundleId: "com.microsoft.VSCode", name: "Visual Studio Code", icon: NSWorkspace.shared.icon(forFile: "/Applications/Visual Studio Code.app"))
    ]

    init() {
        isEnabled = UserDefaults.standard.object(forKey: SettingsKey.enabled) as? Bool ?? true
        currentMethod = InputMode(rawValue: UserDefaults.standard.integer(forKey: SettingsKey.method)) ?? .telex
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
        .onReceive(NotificationCenter.default.publisher(for: .navigateToPage)) { notification in
            if let pageName = notification.object as? String {
                switch pageName {
                case "home": selectedPage = .home
                case "settings": selectedPage = .settings
                case "about": selectedPage = .about
                default: break
                }
            }
        }
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

                // Version info - clickable to check updates
                Button {
                    NotificationCenter.default.post(name: .showUpdateWindow, object: nil)
                } label: {
                    VStack(spacing: 4) {
                        Text("Phiên bản \(AppMetadata.version)")
                            .font(.system(size: 12))
                            .foregroundColor(Color(NSColor.secondaryLabelColor))
                        HStack(spacing: 4) {
                            Image(systemName: "arrow.triangle.2.circlepath")
                                .font(.system(size: 10))
                            Text("Kiểm tra cập nhật")
                                .font(.system(size: 10))
                        }
                        .foregroundColor(Color(NSColor.tertiaryLabelColor))
                    }
                    .padding(.horizontal, 12)
                    .padding(.vertical, 6)
                    .background(
                        RoundedRectangle(cornerRadius: 6)
                            .fill(Color(NSColor.controlBackgroundColor).opacity(0.5))
                    )
                }
                .buttonStyle(.plain)
                .onHover { hovering in
                    if hovering {
                        NSCursor.pointingHand.push()
                    } else {
                        NSCursor.pop()
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
            .padding(.bottom, 16)
        }
        .frame(width: 200)
    }

    private func navButton(page: NavigationPage) -> some View {
        NavButtonView(page: page, isSelected: selectedPage == page) {
            withAnimation(.easeOut(duration: 0.15)) {
                selectedPage = page
            }
        }
    }

    // MARK: - Content

    @ViewBuilder
    private var content: some View {
        switch selectedPage {
        case .home:
            HomePageView(appState: appState, selectedPage: $selectedPage)
                .padding(32)
                .frame(maxWidth: .infinity, maxHeight: .infinity)
        case .settings:
            ScrollView(showsIndicators: false) {
                SettingsPageView(appState: appState)
                    .padding(32)
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)
        case .about:
            AboutPageView()
                .padding(32)
                .frame(maxWidth: .infinity, maxHeight: .infinity)
        }
    }
}

// MARK: - Nav Button with Hover
struct NavButtonView: View {
    let page: NavigationPage
    let isSelected: Bool
    let action: () -> Void
    @State private var isHovered = false

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
            .frame(maxWidth: .infinity)
            .contentShape(Rectangle())
            .background(
                RoundedRectangle(cornerRadius: 8)
                    .fill(isSelected ? Color(NSColor.controlBackgroundColor).opacity(0.6) :
                          isHovered ? Color(NSColor.controlBackgroundColor).opacity(0.3) : Color.clear)
            )
            .animation(.easeOut(duration: 0.1), value: isHovered)
        }
        .buttonStyle(.plain)
        .onHover { hovering in
            isHovered = hovering
            if hovering && !isSelected {
                NSCursor.pointingHand.push()
            } else {
                NSCursor.pop()
            }
        }
        .accessibilityLabel(page.rawValue)
        .accessibilityHint(page.accessibilityHint)
        .accessibilityAddTraits(isSelected ? .isSelected : [])
    }
}

// MARK: - Home Page

struct HomePageView: View {
    @ObservedObject var appState: AppState
    @Binding var selectedPage: NavigationPage

    var body: some View {
        VStack(alignment: .leading, spacing: 20) {
            // Section: Trạng thái
            HIGSection(header: "Trạng thái") {
                HIGRow {
                    HStack(spacing: 10) {
                        Image(systemName: appState.isEnabled ? "checkmark.circle.fill" : "xmark.circle.fill")
                            .font(.system(size: 13))
                            .foregroundColor(appState.isEnabled ? Color.green : Color(NSColor.tertiaryLabelColor))
                            .frame(width: 18)
                        Text(appState.isEnabled ? "Đang hoạt động" : "Đã tắt")
                    }
                    Spacer()
                    Toggle("", isOn: $appState.isEnabled)
                        .toggleStyle(.switch)
                        .labelsHidden()
                }

                HIGRow {
                    HStack(spacing: 10) {
                        Image(systemName: "command")
                            .font(.system(size: 13))
                            .foregroundColor(Color(NSColor.secondaryLabelColor))
                            .frame(width: 18)
                        Text("Phím tắt bật/tắt")
                    }
                    Spacer()
                    HStack(spacing: 4) {
                        KeyboardBadge(text: "⌃")
                        KeyboardBadge(text: "Space")
                    }
                }

                // Navigable row: Kiểu gõ
                HIGRow(action: { selectedPage = .settings }) {
                    HStack(spacing: 10) {
                        Image(systemName: "keyboard")
                            .font(.system(size: 13))
                            .foregroundColor(Color(NSColor.secondaryLabelColor))
                            .frame(width: 18)
                        Text("Kiểu gõ")
                    }
                    Spacer()
                    Text(appState.currentMethod.name)
                        .foregroundColor(Color(NSColor.secondaryLabelColor))
                    Image(systemName: "chevron.right")
                        .font(.system(size: 11, weight: .medium))
                        .foregroundColor(Color(NSColor.tertiaryLabelColor))
                }

                // Navigable row: Từ viết tắt
                HIGRow(action: { selectedPage = .settings }) {
                    HStack(spacing: 10) {
                        Image(systemName: "text.badge.plus")
                            .font(.system(size: 13))
                            .foregroundColor(Color(NSColor.secondaryLabelColor))
                            .frame(width: 18)
                        Text("Từ viết tắt")
                    }
                    Spacer()
                    Text("\(appState.shortcuts.count) mục")
                        .foregroundColor(Color(NSColor.secondaryLabelColor))
                    Image(systemName: "chevron.right")
                        .font(.system(size: 11, weight: .medium))
                        .foregroundColor(Color(NSColor.tertiaryLabelColor))
                }

                // Navigable row: Ứng dụng bỏ qua
                HIGRow(action: { selectedPage = .settings }) {
                    HStack(spacing: 10) {
                        Image(systemName: "app.badge.checkmark")
                            .font(.system(size: 13))
                            .foregroundColor(Color(NSColor.secondaryLabelColor))
                            .frame(width: 18)
                        Text("Ứng dụng bỏ qua")
                    }
                    Spacer()
                    Text("\(appState.excludedApps.count) mục")
                        .foregroundColor(Color(NSColor.secondaryLabelColor))
                    Image(systemName: "chevron.right")
                        .font(.system(size: 11, weight: .medium))
                        .foregroundColor(Color(NSColor.tertiaryLabelColor))
                }
            }

            // Section: Mẹo gõ nhanh
            HIGSection(header: "Mẹo gõ nhanh") {
                VStack(alignment: .leading, spacing: 6) {
                    tipItem("Gõ dấu 2 lần để huỷ", example: "caas → cas → ca")
                    tipItem("Gõ z để xoá dấu thanh", example: "cáz → ca")
                    tipItem("Gõ 0 để xoá tất cả dấu", example: "cấ0 → cu")
                    if appState.currentMethod == .telex {
                        tipItem("Gõ w thay ư hoặc ơ", example: "mw → mư, bown → bơn")
                        tipItem("Gõ [ hoặc ] thay ơ, ư", example: "b[n → bơn")
                    }
                }
                .padding(.horizontal, 12)
                .padding(.vertical, 10)
            }

            Spacer()
        }
    }

    private func tipItem(_ text: String, example: String) -> some View {
        HStack(spacing: 8) {
            Image(systemName: "lightbulb")
                .font(.system(size: 11))
                .foregroundColor(Color(NSColor.tertiaryLabelColor))
            Text(text)
                .font(.system(size: 12))
                .foregroundColor(Color(NSColor.secondaryLabelColor))
            Text("·")
                .foregroundColor(Color(NSColor.tertiaryLabelColor))
            Text(example)
                .font(.system(size: 12, design: .monospaced))
                .foregroundColor(Color(NSColor.tertiaryLabelColor))
        }
    }
}

// MARK: - Settings Page (HIG Compliant)

struct SettingsPageView: View {
    @ObservedObject var appState: AppState
    @State private var showingDeleteConfirmation = false
    @State private var itemToDelete: (type: String, index: Int)? = nil

    var body: some View {
        VStack(alignment: .leading, spacing: 20) {
            // Section: Kiểu gõ
            HIGSection(header: "Kiểu gõ") {
                methodRow(method: .telex, description: "aa → â · dd → đ · s → sắc")
                methodRow(method: .vni, description: "a6 → â · d9 → đ · 1 → sắc")
            }

            // Section: Gõ tắt (macOS System Settings style)
            VStack(alignment: .leading, spacing: 8) {
                Text("TỪ VIẾT TẮT")
                    .font(.system(size: 11, weight: .medium))
                    .foregroundColor(Color(NSColor.secondaryLabelColor))
                    .padding(.horizontal, 4)

                VStack(spacing: 0) {
                    if appState.shortcuts.isEmpty {
                        emptyState(
                            icon: "text.badge.plus",
                            title: "Chưa có từ viết tắt",
                            subtitle: "Nhấn + để thêm"
                        )
                    } else {
                        ForEach($appState.shortcuts) { $shortcut in
                            shortcutRow(shortcut: $shortcut)
                        }
                    }

                    Divider()
                        .background(Color(NSColor.separatorColor))

                    // Plus/Minus buttons
                    HStack(spacing: 0) {
                        Button(action: { /* TODO: Add shortcut */ }) {
                            Image(systemName: "plus")
                                .font(.system(size: 12, weight: .medium))
                                .foregroundColor(Color(NSColor.labelColor))
                                .frame(width: 24, height: 24)
                        }
                        .buttonStyle(.borderless)

                        Divider()
                            .frame(height: 16)

                        Button(action: {
                            if !appState.shortcuts.isEmpty {
                                appState.shortcuts.removeLast()
                            }
                        }) {
                            Image(systemName: "minus")
                                .font(.system(size: 12, weight: .medium))
                                .foregroundColor(appState.shortcuts.isEmpty ? Color(NSColor.tertiaryLabelColor) : Color(NSColor.labelColor))
                                .frame(width: 24, height: 24)
                        }
                        .buttonStyle(.borderless)
                        .disabled(appState.shortcuts.isEmpty)

                        Spacer()
                    }
                    .padding(.horizontal, 8)
                    .padding(.vertical, 4)
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

            // Section: Ngoại lệ (macOS System Settings style)
            VStack(alignment: .leading, spacing: 8) {
                Text("ỨNG DỤNG BỎ QUA")
                    .font(.system(size: 11, weight: .medium))
                    .foregroundColor(Color(NSColor.secondaryLabelColor))
                    .padding(.horizontal, 4)

                VStack(spacing: 0) {
                    // App list with toggles
                    if appState.excludedApps.isEmpty {
                        emptyState(
                            icon: "app.dashed",
                            title: "Chưa có ứng dụng nào",
                            subtitle: "Nhấn + để thêm ứng dụng"
                        )
                    } else {
                        ForEach($appState.excludedApps) { $app in
                            excludedAppRow(app: $app)
                        }
                    }

                    // Divider before buttons
                    Divider()
                        .background(Color(NSColor.separatorColor))

                    // Plus/Minus buttons row
                    HStack(spacing: 0) {
                        Button(action: { /* TODO: Add app */ }) {
                            Image(systemName: "plus")
                                .font(.system(size: 12, weight: .medium))
                                .foregroundColor(Color(NSColor.labelColor))
                                .frame(width: 24, height: 24)
                        }
                        .buttonStyle(.borderless)
                        .accessibilityLabel("Thêm ứng dụng")

                        Divider()
                            .frame(height: 16)

                        Button(action: {
                            // Remove selected/last app
                            if !appState.excludedApps.isEmpty {
                                appState.excludedApps.removeLast()
                            }
                        }) {
                            Image(systemName: "minus")
                                .font(.system(size: 12, weight: .medium))
                                .foregroundColor(appState.excludedApps.isEmpty ? Color(NSColor.tertiaryLabelColor) : Color(NSColor.labelColor))
                                .frame(width: 24, height: 24)
                        }
                        .buttonStyle(.borderless)
                        .disabled(appState.excludedApps.isEmpty)
                        .accessibilityLabel("Xoá ứng dụng")

                        Spacer()
                    }
                    .padding(.horizontal, 8)
                    .padding(.vertical, 4)
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

            Spacer()
        }
        .alert("Xác nhận xoá", isPresented: $showingDeleteConfirmation) {
            Button("Huỷ", role: .cancel) { }
            Button("Xoá", role: .destructive) {
                if let item = itemToDelete {
                    withAnimation {
                        if item.type == "shortcut" {
                            appState.shortcuts.remove(at: item.index)
                        } else {
                            appState.excludedApps.remove(at: item.index)
                        }
                    }
                }
            }
        } message: {
            Text("Bạn có chắc muốn xoá mục này?")
        }
    }

    // MARK: - Method Row

    private func methodRow(method: InputMode, description: String) -> some View {
        HIGRow(action: {
            withAnimation(.easeInOut(duration: 0.15)) {
                appState.currentMethod = method
            }
        }) {
            VStack(alignment: .leading, spacing: 2) {
                Text(method.name)
                    .font(.system(size: 13, weight: .medium))
                Text(description)
                    .font(.system(size: 11))
                    .foregroundColor(Color(NSColor.secondaryLabelColor))
            }
            Spacer()
            if appState.currentMethod == method {
                Image(systemName: "checkmark")
                    .font(.system(size: 14, weight: .medium))
                    .foregroundColor(.accentColor)
                    .transition(.scale.combined(with: .opacity))
            }
        }
        .animation(.easeOut(duration: 0.15), value: appState.currentMethod)
        .accessibilityElement(children: .combine)
        .accessibilityLabel("\(method.name). \(description)")
        .accessibilityValue(appState.currentMethod == method ? "Đang chọn" : "")
        .accessibilityHint("Nhấn đúp để chọn kiểu gõ này")
        .accessibilityAddTraits(appState.currentMethod == method ? .isSelected : [])
    }

    // MARK: - Shortcut Row (macOS System Settings style)

    private func shortcutRow(shortcut: Binding<ShortcutItem>) -> some View {
        HStack(spacing: 12) {
            // Key → Value
            Text(shortcut.wrappedValue.key)
                .font(.system(size: 13, weight: .medium, design: .monospaced))
                .foregroundColor(Color(NSColor.labelColor))
            Image(systemName: "arrow.right")
                .font(.system(size: 10))
                .foregroundColor(Color(NSColor.tertiaryLabelColor))
            Text(shortcut.wrappedValue.value)
                .font(.system(size: 13))
                .foregroundColor(Color(NSColor.secondaryLabelColor))
                .lineLimit(1)

            Spacer()

            // Toggle
            Toggle("", isOn: shortcut.isEnabled)
                .toggleStyle(.switch)
                .labelsHidden()
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 8)
        .accessibilityElement(children: .combine)
        .accessibilityLabel("Gõ tắt \(shortcut.wrappedValue.key) thành \(shortcut.wrappedValue.value), \(shortcut.wrappedValue.isEnabled ? "đang bật" : "đã tắt")")
    }

    // MARK: - Excluded App Row (macOS System Settings style)

    private func excludedAppRow(app: Binding<ExcludedApp>) -> some View {
        HStack(spacing: 12) {
            // App icon
            if let icon = app.wrappedValue.icon {
                Image(nsImage: icon)
                    .resizable()
                    .frame(width: 24, height: 24)
            } else {
                Image(systemName: "app.fill")
                    .font(.system(size: 20))
                    .foregroundColor(Color(NSColor.secondaryLabelColor))
                    .frame(width: 24, height: 24)
            }

            // App name with clear action label
            VStack(alignment: .leading, spacing: 2) {
                Text(app.wrappedValue.name)
                    .font(.system(size: 13))
                    .foregroundColor(Color(NSColor.labelColor))
                Text("Tắt gõ tiếng Việt")
                    .font(.system(size: 11))
                    .foregroundColor(Color(NSColor.tertiaryLabelColor))
            }

            Spacer()

            // Toggle with clear ON state
            Toggle("", isOn: app.isEnabled)
                .toggleStyle(.switch)
                .labelsHidden()
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 8)
        .accessibilityElement(children: .combine)
        .accessibilityLabel("Tắt gõ tiếng Việt trong \(app.wrappedValue.name), \(app.wrappedValue.isEnabled ? "đang bật" : "đã tắt")")
    }

    // MARK: - Empty State

    private func emptyState(icon: String, title: String, subtitle: String) -> some View {
        EmptyStateView(icon: icon, title: title, subtitle: subtitle)
    }
}

// MARK: - Empty State with Pulse Animation
struct EmptyStateView: View {
    let icon: String
    let title: String
    let subtitle: String
    @State private var isPulsing = false

    var body: some View {
        VStack(spacing: 8) {
            Image(systemName: icon)
                .font(.system(size: 24))
                .foregroundColor(Color(NSColor.tertiaryLabelColor))
                .opacity(isPulsing ? 1.0 : 0.6)
                .animation(
                    Animation.easeInOut(duration: 1.5).repeatForever(autoreverses: true),
                    value: isPulsing
                )
                .onAppear { isPulsing = true }
            Text(title)
                .font(.system(size: 13, weight: .medium))
                .foregroundColor(Color(NSColor.secondaryLabelColor))
            Text(subtitle)
                .font(.system(size: 11))
                .foregroundColor(Color(NSColor.tertiaryLabelColor))
        }
        .frame(maxWidth: .infinity)
        .padding(.vertical, 24)
        .accessibilityElement(children: .combine)
        .accessibilityLabel("\(title). \(subtitle)")
    }
}

// MARK: - About Page

struct AboutPageView: View {
    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            // Logo & Info (compact)
            HStack(spacing: 16) {
                Image(nsImage: AppMetadata.logo)
                    .resizable()
                    .frame(width: 64, height: 64)

                VStack(alignment: .leading, spacing: 4) {
                    Text(AppMetadata.name)
                        .font(.system(size: 18, weight: .bold))
                    Text("Bộ gõ tiếng Việt nhanh và nhẹ")
                        .font(.system(size: 12))
                        .foregroundColor(Color(NSColor.secondaryLabelColor))
                    Text("Phiên bản \(AppMetadata.version)")
                        .font(.system(size: 11, design: .monospaced))
                        .foregroundColor(Color(NSColor.tertiaryLabelColor))
                }
                Spacer()
            }
            .padding(.bottom, 8)

            // Links
            HIGSection(header: "Liên kết") {
                linkRow(icon: "chevron.left.forwardslash.chevron.right", title: "Mã nguồn GitHub", url: AppMetadata.repository)
                linkRow(icon: "book", title: "Hướng dẫn sử dụng", url: AppMetadata.website)
                linkRow(icon: "ant", title: "Góp ý & Báo lỗi", url: AppMetadata.issuesURL)
            }

            // Author
            HIGSection(header: "Tác giả") {
                linkRow(icon: "person", title: AppMetadata.author, url: AppMetadata.authorLinkedin)
                linkRow(icon: "heart", title: "Ủng hộ tác giả", url: "https://github.com/sponsors/khaphanspace")
            }

            Spacer()

            // Footer
            Text("Made with ❤️ in Vietnam")
                .font(.system(size: 11))
                .foregroundColor(Color(NSColor.tertiaryLabelColor))
                .frame(maxWidth: .infinity, alignment: .center)
        }
    }

    private func linkRow(icon: String, title: String, url: String) -> some View {
        LinkRowView(icon: icon, title: title, url: url)
    }
}

// MARK: - Link Row with Hover
struct LinkRowView: View {
    let icon: String
    let title: String
    let url: String
    @State private var isHovered = false

    var body: some View {
        Link(destination: URL(string: url)!) {
            HStack(spacing: 10) {
                Image(systemName: icon)
                    .font(.system(size: 13))
                    .foregroundColor(Color(NSColor.secondaryLabelColor))
                    .frame(width: 20)
                Text(title)
                    .font(.system(size: 13))
                    .foregroundColor(Color(NSColor.labelColor))
                Spacer()
                Image(systemName: "arrow.up.right")
                    .font(.system(size: 11))
                    .foregroundColor(isHovered ? Color(NSColor.secondaryLabelColor) : Color(NSColor.tertiaryLabelColor))
            }
            .padding(.horizontal, 12)
            .padding(.vertical, 12)
            .contentShape(Rectangle())
            .background(
                isHovered ? Color(NSColor.controlBackgroundColor).opacity(0.3) : Color.clear
            )
            .animation(.easeOut(duration: 0.1), value: isHovered)
        }
        .buttonStyle(.plain)
        .onHover { hovering in
            isHovered = hovering
            if hovering {
                NSCursor.pointingHand.push()
            } else {
                NSCursor.pop()
            }
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

            // Section content with grouped background + soft shadow
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
            .shadow(color: Color.black.opacity(0.04), radius: 4, x: 0, y: 2)
        }
    }
}

/// HIG-compliant row for lists/forms with hover feedback
struct HIGRow<Content: View>: View {
    let action: (() -> Void)?
    let content: Content
    @State private var isHovered = false

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
                .onHover { hovering in
                    isHovered = hovering
                    if hovering {
                        NSCursor.pointingHand.push()
                    } else {
                        NSCursor.pop()
                    }
                }
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
        .padding(.vertical, 12)
        .frame(maxWidth: .infinity, alignment: .leading)
        .contentShape(Rectangle())
        .background(
            action != nil && isHovered ?
            Color(NSColor.controlBackgroundColor).opacity(0.3) : Color.clear
        )
        .animation(.easeOut(duration: 0.1), value: isHovered)
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
