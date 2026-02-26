import AppKit
import SwiftUI

// MARK: - Per-App Profile Model

struct PerAppConfig: Codable, Equatable {
    var enabled: Bool = true // Bật/Tắt Gõ Nhanh cho app này
    var delayPreset: Int = 0 // DelayPreset raw value
    var methodOverride: Int = -1 // MethodOverride raw value: -1=auto, 0=telex, 1=vni
    var injectionOverride: Int = -1 // InjectionOverride raw value: -1=auto, 0-4=specific
}

// MARK: - Delay Preset

enum DelayPreset: Int, CaseIterable {
    case auto = 0
    case none = 1
    case low = 2
    case medium = 3
    case high = 4
    case veryHigh = 5

    var name: String {
        switch self {
        case .auto: "Tự động"
        case .none: "Không"
        case .low: "Thấp"
        case .medium: "Vừa"
        case .high: "Cao"
        case .veryHigh: "Rất cao"
        }
    }

    /// Delay tuple: (backspace µs, wait µs, text µs)
    var delays: (UInt32, UInt32, UInt32) {
        switch self {
        case .auto: (0, 0, 0) // Sentinel: use system default
        case .none: (0, 0, 0) // Force zero delay
        case .low: (1000, 3000, 1500)
        case .medium: (3000, 8000, 3000)
        case .high: (8000, 25000, 8000)
        case .veryHigh: (12000, 25000, 12000)
        }
    }

    var color: Color {
        switch self {
        case .auto: Color(NSColor.secondaryLabelColor)
        case .none: .blue
        case .low: .green
        case .medium: .orange
        case .high: Color(NSColor.systemRed)
        case .veryHigh: .purple
        }
    }
}

// MARK: - Method Override (Kiểu gõ)

enum MethodOverride: Int, CaseIterable {
    case auto = -1
    case off = -2
    case telex = 0
    case vni = 1

    var name: String {
        switch self {
        case .auto: "Tự động"
        case .off: "Tắt"
        case .telex: "Telex"
        case .vni: "VNI"
        }
    }

    /// Cases shown in picker (excludes .off, handled by toggle)
    static let inputCases: [MethodOverride] = [.auto, .telex, .vni]
}

// MARK: - Injection Override (Kiểu inject)

/// User-facing injection method options (subset of internal InjectionMethod)
enum InjectionOverride: Int, CaseIterable {
    case auto = -1
    case fast = 0
    case slow = 1
    case charByChar = 2
    case selection = 3
    case emptyCharPrefix = 4

    var name: String {
        switch self {
        case .auto: "Tự động"
        case .fast: "Nhanh"
        case .slow: "Chậm"
        case .charByChar: "Từng ký tự"
        case .selection: "Chọn thay thế"
        case .emptyCharPrefix: "Empty char"
        }
    }

    var subtitle: String {
        switch self {
        case .auto: "Để hệ thống chọn"
        case .fast: "Mặc định, backspace + text"
        case .slow: "Delay cao hơn cho Electron"
        case .charByChar: "Gõ từng ký tự, Safari/GDocs"
        case .selection: "Select + replace, combo box"
        case .emptyCharPrefix: "Phá autocomplete trình duyệt"
        }
    }

    /// Map to internal InjectionMethod string for RustBridge
    var internalMethodName: String {
        switch self {
        case .auto: "auto"
        case .fast: "fast"
        case .slow: "slow"
        case .charByChar: "charByChar"
        case .selection: "selection"
        case .emptyCharPrefix: "emptyCharPrefix"
        }
    }
}

// MARK: - Advanced Settings View

struct AdvancedSettingsView: View {
    @ObservedObject var appState: AppState
    @State private var showAddApp = false
    @State private var logEnabled = FileManager.default.fileExists(atPath: "/tmp/gonhanh_debug.log")

    var body: some View {
        ScrollView(showsIndicators: false) {
            VStack(alignment: .leading, spacing: 20) {
                performanceSection
                logSection
                perAppSection
                Spacer()
            }
        }
    }

    // MARK: - Performance

    private var performanceSection: some View {
        VStack(spacing: 0) {
            SettingsToggleRow(
                "Tắt phát hiện Spotlight/Raycast",
                subtitle: "Bỏ qua panel app, giảm CPU/RAM sử dụng",
                isOn: $appState.disablePanelDetection
            )
        }
        .cardBackground()
    }

    // MARK: - Log

    private var logSection: some View {
        VStack(spacing: 0) {
            SettingsRow {
                VStack(alignment: .leading, spacing: 2) {
                    Text("Debug Log").font(.system(size: 13, weight: .medium))
                    Text("Ghi log xử lý phím vào /tmp/gonhanh_debug.log")
                        .font(.system(size: 11))
                        .foregroundColor(Color(NSColor.secondaryLabelColor))
                }
                Spacer()
                LogToggleButton(isEnabled: $logEnabled)
            }
            if logEnabled {
                Divider().padding(.leading, 12)
                LogViewerSection()
            }
        }
        .cardBackground()
    }

    // MARK: - Per-App Profiles

    private var perAppSection: some View {
        VStack(spacing: 0) {
            SettingsRow {
                VStack(alignment: .leading, spacing: 2) {
                    Text("Tuỳ chỉnh theo ứng dụng").font(.system(size: 13, weight: .medium))
                    Text("Ghi đè delay, kiểu gõ, kiểu inject cho từng app")
                        .font(.system(size: 11))
                        .foregroundColor(Color(NSColor.secondaryLabelColor))
                }
                Spacer()
                Button(action: { showAddApp = true }) {
                    Image(systemName: "plus.circle.fill")
                        .font(.system(size: 16))
                        .foregroundColor(.accentColor)
                }
                .buttonStyle(.plain)
            }

            ForEach(sortedProfiles, id: \.key) { entry in
                Divider().padding(.leading, 12)
                PerAppProfileRow(
                    bundleId: entry.key,
                    config: entry.value,
                    onChange: { appState.perAppProfiles[entry.key] = $0 },
                    onRemove: { appState.perAppProfiles.removeValue(forKey: entry.key) }
                )
            }
        }
        .cardBackground()
        .sheet(isPresented: $showAddApp) {
            AppPickerSheet(existingBundleIds: Set(appState.perAppProfiles.keys)) { bundleId in
                appState.perAppProfiles[bundleId] = PerAppConfig()
            }
        }
    }

    private var sortedProfiles: [(key: String, value: PerAppConfig)] {
        appState.perAppProfiles.sorted { $0.key < $1.key }
    }
}

// MARK: - Log Toggle

struct LogToggleButton: View {
    @Binding var isEnabled: Bool

    var body: some View {
        Toggle("", isOn: $isEnabled)
            .toggleStyle(.switch)
            .controlSize(.small)
            .onChange(of: isEnabled) { newValue in
                if newValue {
                    FileManager.default.createFile(atPath: "/tmp/gonhanh_debug.log", contents: nil)
                } else {
                    try? FileManager.default.removeItem(atPath: "/tmp/gonhanh_debug.log")
                }
            }
    }
}

// MARK: - Log Viewer

struct LogViewerSection: View {
    @State private var logLines: [String] = []
    @State private var timer: Timer?
    @State private var copyFeedback = false
    @State private var lastFileSize: UInt64 = 0

    var body: some View {
        ScrollViewReader { proxy in
            ScrollView(.vertical, showsIndicators: true) {
                if logLines.isEmpty {
                    Text("Gõ phím để bắt đầu ghi log.")
                        .font(.system(size: 11))
                        .foregroundColor(Color(NSColor.tertiaryLabelColor))
                        .frame(maxWidth: .infinity)
                        .padding(.vertical, 24)
                } else {
                    LazyVStack(alignment: .leading, spacing: 0) {
                        ForEach(Array(logLines.enumerated()), id: \.offset) { idx, line in
                            Text(line)
                                .font(.system(size: 10, design: .monospaced))
                                .foregroundColor(logColor(for: line))
                                .padding(.vertical, 1)
                                .id(idx)
                        }
                    }
                    .padding(.horizontal, 10)
                    .padding(.top, 6)
                    .padding(.bottom, 26)
                    .frame(maxWidth: .infinity, alignment: .leading)
                }
            }
            .frame(height: 150)
            .background(Color(NSColor.textBackgroundColor).opacity(0.5))
            .clipShape(RoundedRectangle(cornerRadius: 6))
            .overlay(
                RoundedRectangle(cornerRadius: 6)
                    .stroke(Color(NSColor.separatorColor).opacity(0.3), lineWidth: 0.5)
            )
            .overlay(alignment: .bottomTrailing) {
                if !logLines.isEmpty {
                    HStack(spacing: 4) {
                        logActionButton(
                            icon: copyFeedback ? "checkmark" : "doc.on.doc",
                            label: copyFeedback ? "Đã copy" : "Copy",
                            color: copyFeedback ? .green : Color(NSColor.secondaryLabelColor),
                            action: copyLog
                        )
                        logActionButton(
                            icon: "trash",
                            label: "Xoá",
                            color: Color(NSColor.secondaryLabelColor),
                            action: clearLog
                        )
                    }
                    .padding(.trailing, 14)
                    .padding(.bottom, 8)
                }
            }
            .padding(.horizontal, 12)
            .padding(.vertical, 8)
            .onChange(of: logLines.count) { _ in
                if let last = logLines.indices.last {
                    proxy.scrollTo(last, anchor: .bottom)
                }
            }
        }
        .onAppear { startPolling() }
        .onDisappear { stopPolling() }
    }

    private func logActionButton(icon: String, label: String, color: Color, action: @escaping () -> Void) -> some View {
        Button(action: action) {
            HStack(spacing: 3) {
                Image(systemName: icon).font(.system(size: 9))
                Text(label)
            }
            .font(.system(size: 10))
            .foregroundColor(color)
            .padding(.horizontal, 8)
            .padding(.vertical, 3)
            .background(Color(NSColor.controlBackgroundColor))
            .clipShape(RoundedRectangle(cornerRadius: 4))
            .overlay(
                RoundedRectangle(cornerRadius: 4)
                    .stroke(Color(NSColor.separatorColor).opacity(0.5), lineWidth: 0.5)
            )
        }
        .buttonStyle(.plain)
    }

    private func logColor(for line: String) -> Color {
        if line.contains("] K:") { return Color(NSColor.systemBlue) }
        if line.contains("] M:") { return Color(NSColor.systemOrange) }
        if line.contains("] Q:") { return Color(NSColor.systemPurple) }
        if line.contains("] P:") { return Color(NSColor.systemGreen) }
        return Color(NSColor.secondaryLabelColor)
    }

    private func copyLog() {
        NSPasteboard.general.clearContents()
        NSPasteboard.general.setString(logLines.joined(separator: "\n"), forType: .string)
        copyFeedback = true
        DispatchQueue.main.asyncAfter(deadline: .now() + 1.5) { copyFeedback = false }
    }

    private func clearLog() {
        try? "".write(toFile: "/tmp/gonhanh_debug.log", atomically: true, encoding: .utf8)
        logLines = []
        lastFileSize = 0
    }

    private func startPolling() {
        loadLog()
        timer = Timer.scheduledTimer(withTimeInterval: 1.0, repeats: true) { _ in loadLog() }
    }

    private func stopPolling() {
        timer?.invalidate(); timer = nil
    }

    private func loadLog() {
        let path = "/tmp/gonhanh_debug.log"
        // Skip re-read if file size unchanged (avoids reading MBs every second)
        let attrs = try? FileManager.default.attributesOfItem(atPath: path)
        let size = attrs?[.size] as? UInt64 ?? 0
        if size == 0 {
            if !logLines.isEmpty { logLines = []; lastFileSize = 0 }
            return
        }
        guard size != lastFileSize else { return }
        lastFileSize = size

        guard let data = FileManager.default.contents(atPath: path),
              let content = String(data: data, encoding: .utf8), !content.isEmpty
        else { return }
        let lines = content.components(separatedBy: "\n").filter { !$0.isEmpty }
        let tail = Array(lines.suffix(80))
        if tail != logLines { logLines = tail }
    }
}

// MARK: - Per-App Profile Row

struct PerAppProfileRow: View {
    let bundleId: String
    let config: PerAppConfig
    let onChange: (PerAppConfig) -> Void
    let onRemove: () -> Void
    @State private var removeHovered = false

    var body: some View {
        VStack(spacing: 8) {
            // Header: icon + name + remove
            HStack(spacing: 8) {
                AppIconView(bundleId: bundleId)
                VStack(alignment: .leading, spacing: 1) {
                    Text(appName).font(.system(size: 12, weight: .medium))
                    Text(bundleId)
                        .font(.system(size: 9, design: .monospaced))
                        .foregroundColor(Color(NSColor.tertiaryLabelColor))
                        .lineLimit(1)
                        .truncationMode(.middle)
                }
                Spacer()
                Button(action: onRemove) {
                    Image(systemName: "xmark.circle.fill")
                        .font(.system(size: 13))
                        .foregroundColor(removeHovered ? .red : Color(NSColor.quaternaryLabelColor))
                }
                .buttonStyle(.plain)
                .onHover { removeHovered = $0 }
            }
            .padding(.horizontal, 12)
            .padding(.top, 10)

            // Delay slider
            HStack(spacing: 6) {
                Text("Delay").font(.system(size: 10)).foregroundColor(Color(NSColor.tertiaryLabelColor))
                    .frame(width: 34, alignment: .leading)
                Slider(value: delaySliderBinding, in: 0 ... Double(DelayPreset.allCases.count - 1), step: 1)
                Text(delayPresetName)
                    .font(.system(size: 10, weight: .medium))
                    .foregroundColor(delayPresetColor)
                    .frame(width: 52, alignment: .trailing)
            }
            .padding(.horizontal, 12)

            // Trạng thái + Kiểu gõ + Kiểu inject
            HStack(spacing: 10) {
                HStack(spacing: 4) {
                    Text("GN").font(.system(size: 10)).foregroundColor(Color(NSColor.tertiaryLabelColor))
                        .lineLimit(1).fixedSize()
                    Picker("", selection: enabledBinding) {
                        Text("Bật").tag(true)
                        Text("Tắt").tag(false)
                    }
                    .labelsHidden()
                    .frame(width: 62)
                }
                HStack(spacing: 4) {
                    Text("Kiểu gõ").font(.system(size: 10)).foregroundColor(Color(NSColor.tertiaryLabelColor))
                        .lineLimit(1).fixedSize()
                    Picker("", selection: methodBinding) {
                        ForEach(MethodOverride.inputCases, id: \.rawValue) { Text($0.name).tag($0.rawValue) }
                    }
                    .labelsHidden()
                    .frame(width: 85)
                }
                HStack(spacing: 4) {
                    Text("Inject").font(.system(size: 10)).foregroundColor(Color(NSColor.tertiaryLabelColor))
                        .lineLimit(1).fixedSize()
                    Picker("", selection: injectionBinding) {
                        ForEach(InjectionOverride.allCases, id: \.rawValue) { Text($0.name).tag($0.rawValue) }
                    }
                    .labelsHidden()
                    .frame(width: 110)
                }
                Spacer()
            }
            .padding(.horizontal, 12)
        }
        .padding(.bottom, 10)
    }

    // MARK: - Helpers

    private var enabledBinding: Binding<Bool> {
        Binding(
            get: { config.enabled },
            set: { on in var c = config; c.enabled = on; onChange(c) }
        )
    }

    private var methodBinding: Binding<Int> {
        Binding(
            get: { config.methodOverride },
            set: { v in var c = config; c.methodOverride = v; onChange(c) }
        )
    }

    private var injectionBinding: Binding<Int> {
        Binding(
            get: { config.injectionOverride },
            set: { v in var c = config; c.injectionOverride = v; onChange(c) }
        )
    }

    private var delaySliderBinding: Binding<Double> {
        Binding(
            get: { Double(config.delayPreset) },
            set: { val in var c = config; c.delayPreset = Int(val.rounded()); onChange(c) }
        )
    }

    private var delayPresetName: String {
        (DelayPreset(rawValue: config.delayPreset) ?? .auto).name
    }

    private var delayPresetColor: Color {
        (DelayPreset(rawValue: config.delayPreset) ?? .auto).color
    }

    private var appName: String {
        if let url = NSWorkspace.shared.urlForApplication(withBundleIdentifier: bundleId) {
            return FileManager.default.displayName(atPath: url.path).replacingOccurrences(of: ".app", with: "")
        }
        return NSWorkspace.shared.runningApplications
            .first(where: { $0.bundleIdentifier == bundleId })?
            .localizedName ?? bundleId.components(separatedBy: ".").last ?? bundleId
    }
}

// MARK: - App Icon

struct AppIconView: View {
    let bundleId: String
    var body: some View {
        Group {
            if let url = NSWorkspace.shared.urlForApplication(withBundleIdentifier: bundleId) {
                Image(nsImage: NSWorkspace.shared.icon(forFile: url.path)).resizable()
            } else if let app = NSWorkspace.shared.runningApplications.first(where: { $0.bundleIdentifier == bundleId }),
                      let icon = app.icon
            {
                Image(nsImage: icon).resizable()
            } else {
                Image(systemName: "app.dashed").font(.system(size: 14)).foregroundColor(Color(NSColor.tertiaryLabelColor))
            }
        }
        .frame(width: 22, height: 22)
        .cornerRadius(4)
    }
}

// MARK: - App Picker Sheet

struct AppPickerSheet: View {
    let existingBundleIds: Set<String>
    let onAdd: (String) -> Void
    @Environment(\.dismiss) private var dismiss
    @State private var searchText = ""
    @State private var selectedBundleId: String?

    private var runningApps: [(name: String, bundleId: String)] {
        var seen = Set<String>()
        return NSWorkspace.shared.runningApplications
            .filter { $0.activationPolicy == .regular && $0.bundleIdentifier != nil }
            .compactMap { app -> (name: String, bundleId: String)? in
                guard let bid = app.bundleIdentifier,
                      !existingBundleIds.contains(bid), seen.insert(bid).inserted
                else { return nil }
                let name = app.localizedName
                    ?? bid.components(separatedBy: ".").last
                    ?? bid
                return (name, bid)
            }
            .sorted { $0.name.localizedCaseInsensitiveCompare($1.name) == .orderedAscending }
    }

    private var filtered: [(name: String, bundleId: String)] {
        if searchText.isEmpty { return runningApps }
        return runningApps.filter {
            $0.name.localizedCaseInsensitiveContains(searchText)
                || $0.bundleId.localizedCaseInsensitiveContains(searchText)
        }
    }

    var body: some View {
        VStack(spacing: 0) {
            SheetHeader("Thêm ứng dụng", subtitle: "Chọn app để tuỳ chỉnh")
            Divider()

            HStack(spacing: 8) {
                Image(systemName: "magnifyingglass").font(.system(size: 12)).foregroundColor(.secondary)
                TextField("Tìm kiếm...", text: $searchText).textFieldStyle(.plain).font(.system(size: 13))
                if !searchText.isEmpty {
                    Button(action: { searchText = "" }) {
                        Image(systemName: "xmark.circle.fill").font(.system(size: 12)).foregroundColor(.secondary)
                    }.buttonStyle(.plain)
                }
            }
            .padding(.horizontal, 12)
            .padding(.vertical, 8)

            Divider()

            if filtered.isEmpty {
                Text("Không tìm thấy").font(.system(size: 12)).foregroundColor(.secondary)
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                ScrollView {
                    LazyVStack(spacing: 0) {
                        ForEach(filtered, id: \.bundleId) { app in
                            AppPickerRow(name: app.name, bundleId: app.bundleId,
                                         isSelected: selectedBundleId == app.bundleId) { selectedBundleId = app.bundleId }
                            if app.bundleId != filtered.last?.bundleId { Divider().padding(.leading, 40) }
                        }
                    }
                }
            }

            Divider()
            HStack {
                Spacer()
                Button("Huỷ") { dismiss() }
                    .keyboardShortcut(.cancelAction)
                Button("Thêm") {
                    if let id = selectedBundleId { onAdd(id) }
                    dismiss()
                }
                .keyboardShortcut(.defaultAction)
                .disabled(selectedBundleId == nil)
            }
            .font(.system(size: 12))
            .padding(.horizontal, 16)
            .padding(.vertical, 10)
        }
        .frame(width: 400, height: 400)
    }
}

private struct AppPickerRow: View {
    let name: String
    let bundleId: String
    let isSelected: Bool
    let action: () -> Void
    @State private var hovered = false

    var body: some View {
        HStack(spacing: 8) {
            AppIconView(bundleId: bundleId)
            VStack(alignment: .leading, spacing: 1) {
                Text(name).font(.system(size: 12))
                Text(bundleId).font(.system(size: 9, design: .monospaced)).foregroundColor(.secondary).lineLimit(1)
            }
            Spacer()
            if isSelected {
                Image(systemName: "checkmark.circle.fill").foregroundColor(.accentColor).font(.system(size: 14))
            }
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 6)
        .background(
            isSelected ? Color.accentColor.opacity(0.1) :
                hovered ? Color(NSColor.controlBackgroundColor).opacity(0.4) : Color.clear
        )
        .contentShape(Rectangle())
        .onHover { hovered = $0 }
        .onTapGesture { action() }
    }
}
