import AppKit
import SwiftUI

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
            Divider().padding(.horizontal, 14)
            SettingsToggleRow(
                "Khởi động lại khi đóng cài đặt",
                subtitle: "Tự động giải phóng RAM của cài đặt khi đóng",
                isOn: $appState.restartOnClose
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
                    Text("Tuỳ chỉnh cách Gõ Nhanh hoạt động cho từng ứng dụng")
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
                appState.perAppProfiles[bundleId] = PerAppConfig.fromDetected(bundleId: bundleId)
            }
        }
    }

    private var sortedProfiles: [(key: String, value: PerAppConfig)] {
        appState.perAppProfiles.sorted { $0.key < $1.key }
    }
}

// MARK: - Per-App Profile Row

struct PerAppProfileRow: View {
    let bundleId: String
    let config: PerAppConfig
    let onChange: (PerAppConfig) -> Void
    let onRemove: () -> Void
    @State private var removeHovered = false
    @State private var resetHovered = false

    private let labelWidth: CGFloat = 48
    private let labelColor = Color(NSColor.tertiaryLabelColor)
    private let labelFont = Font.system(size: 10)

    var body: some View {
        VStack(spacing: 10) {
            // Header
            HStack(spacing: 8) {
                AppIconView(bundleId: bundleId)
                VStack(alignment: .leading, spacing: 2) {
                    HStack(spacing: 6) {
                        Text(appName).font(.system(size: 12, weight: .medium))
                        if let hint = detectedHint {
                            Text(hint)
                                .font(.system(size: 9))
                                .foregroundColor(labelColor)
                                .padding(.horizontal, 5)
                                .padding(.vertical, 1)
                                .background(labelColor.opacity(0.1))
                                .clipShape(RoundedRectangle(cornerRadius: 3))
                        }
                    }
                    Text(bundleId)
                        .font(.system(size: 9, design: .monospaced))
                        .foregroundColor(labelColor)
                        .lineLimit(1).truncationMode(.middle)
                }
                Spacer()
                Button(action: { onChange(PerAppConfig.fromDetected(bundleId: bundleId)) }) {
                    Image(systemName: "arrow.counterclockwise.circle.fill")
                        .font(.system(size: 14))
                        .foregroundColor(resetHovered ? .accentColor : Color(NSColor.quaternaryLabelColor))
                }
                .buttonStyle(.plain).onHover { resetHovered = $0 }
                .help("Reset về mặc định")
                Button(action: onRemove) {
                    Image(systemName: "xmark.circle.fill")
                        .font(.system(size: 14))
                        .foregroundColor(removeHovered ? .red : Color(NSColor.quaternaryLabelColor))
                }
                .buttonStyle(.plain).onHover { removeHovered = $0 }
            }
            .padding(.horizontal, 14)
            .padding(.top, 10)

            // Delay
            VStack(alignment: .leading, spacing: 2) {
                HStack(spacing: 6) {
                    Text("Delay").font(labelFont).foregroundColor(labelColor)
                        .frame(width: labelWidth, alignment: .leading)
                    Slider(value: delaySliderBinding, in: 0 ... Double(DelayPreset.allCases.count - 1), step: 1)
                    Text(delayPresetName)
                        .font(.system(size: 10, weight: .medium))
                        .foregroundColor(delayPresetColor)
                        .frame(width: 52, alignment: .trailing)
                }
                Text("Tăng nếu bị nuốt chữ · Giảm nếu app phản hồi nhanh")
                    .font(.system(size: 10))
                    .foregroundColor(Color(NSColor.tertiaryLabelColor))
                    .padding(.leading, labelWidth + 6)
            }
            .padding(.horizontal, 14)

            // GN · Inject
            HStack(spacing: 8) {
                profilePicker("Bật Gõ Nhanh", selection: enabledBinding, width: 80) {
                    Text("Tự động").tag(0)
                    Text("Bật").tag(1)
                    Text("Tắt").tag(-1)
                }
                profilePicker("Kiểu Inject", selection: injectionBinding, width: 110) {
                    ForEach(InjectionOverride.allCases, id: \.rawValue) { Text($0.name).tag($0.rawValue) }
                }
                Spacer()
            }
            .padding(.horizontal, 14)
        }
        .padding(.bottom, 12)
    }

    private func profilePicker(_ label: String, selection: Binding<some Hashable>, width: CGFloat, @ViewBuilder content: () -> some View) -> some View {
        HStack(spacing: 4) {
            Text(label).font(labelFont).foregroundColor(labelColor)
                .lineLimit(1).fixedSize()
            Picker("", selection: selection, content: content)
                .labelsHidden().frame(width: width)
        }
    }

    // MARK: - Helpers

    private var enabledBinding: Binding<Int> {
        Binding(
            get: { config.enabledState },
            set: { v in var c = config; c.enabledState = v; onChange(c) }
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
        (DelayPreset(rawValue: config.delayPreset) ?? .none).name
    }

    private var delayPresetColor: Color {
        (DelayPreset(rawValue: config.delayPreset) ?? .none).color
    }

    /// Show detected default injection method as badge (e.g. "fast", "slow")
    private var detectedHint: String? {
        getDetectedDefault(for: bundleId)?.method
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
