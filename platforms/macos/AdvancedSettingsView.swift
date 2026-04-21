import AppKit
import SwiftUI

// MARK: - Advanced Settings View

struct AdvancedSettingsView: View {
    @ObservedObject var appState: AppState
    @State private var showAddApp = false
    @State private var logEnabled = FileManager.default.fileExists(atPath: "/tmp/gonhanh_debug.log")
    @State private var showCleanPasteMappings = false

    var body: some View {
        ScrollView(showsIndicators: false) {
            VStack(alignment: .leading, spacing: 20) {
                cleanPasteSection
                performanceSection
                logSection
                perAppSection
                Spacer()
            }
        }
    }

    // MARK: - Clean Paste

    private var cleanPasteSection: some View {
        VStack(spacing: 0) {
            SettingsToggleRow(
                "Dán sạch",
                subtitle: "Chuẩn hoá văn bản khi dán bằng phím tắt",
                isOn: $appState.cleanPasteEnabled
            )

            if appState.cleanPasteEnabled {
                Divider().padding(.horizontal, 14)

                // Shortcut recorder
                SettingsRow {
                    HStack(spacing: 6) {
                        Image(systemName: "arrow.turn.down.right")
                            .font(.system(size: 10))
                            .foregroundColor(Color(NSColor.tertiaryLabelColor))
                        VStack(alignment: .leading, spacing: 2) {
                            Text("Phím tắt").font(.system(size: 13))
                            Text("Nhấn phím tắt để dán văn bản đã chuẩn hoá")
                                .font(.system(size: 11))
                                .foregroundColor(Color(NSColor.secondaryLabelColor))
                        }
                    }
                    Spacer()
                    CleanPasteShortcutRecorderRow(appState: appState)
                }

                Divider().padding(.horizontal, 14)

                // Char mappings button
                SettingsRow {
                    HStack(spacing: 6) {
                        Image(systemName: "arrow.turn.down.right")
                            .font(.system(size: 10))
                            .foregroundColor(Color(NSColor.tertiaryLabelColor))
                        VStack(alignment: .leading, spacing: 2) {
                            Text("Bảng thay thế ký tự").font(.system(size: 13))
                            let enabled = appState.cleanPasteMappings.filter(\.isEnabled).count
                            let total = appState.cleanPasteMappings.count
                            Text("\(total) mục · \(enabled) đang bật")
                                .font(.system(size: 11))
                                .foregroundColor(Color(NSColor.secondaryLabelColor))
                        }
                    }
                    Spacer()
                    Button(action: { showCleanPasteMappings = true }) {
                        Text("Tuỳ chỉnh")
                            .font(.system(size: 12))
                    }
                }
            }
        }
        .cardBackground()
        .sheet(isPresented: $showCleanPasteMappings) {
            CleanPasteMappingsSheet(appState: appState)
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

// MARK: - Clean Paste Shortcut Recorder

struct CleanPasteShortcutRecorderRow: View {
    @ObservedObject var appState: AppState
    @State private var isRecording = false
    @State private var recordedObserver: NSObjectProtocol?
    @State private var cancelledObserver: NSObjectProtocol?
    @State private var windowObserver: NSObjectProtocol?

    var body: some View {
        HStack(spacing: 4) {
            if isRecording {
                Text("Nhấn phím...")
                    .font(.system(size: 11, weight: .medium))
                    .foregroundColor(.accentColor)
                    .padding(.horizontal, 6)
                    .padding(.vertical, 3)
                    .background(RoundedRectangle(cornerRadius: 4).stroke(Color.accentColor, lineWidth: 1))
            } else {
                ForEach(appState.cleanPasteShortcut.displayParts, id: \.self) { KeyCap(text: $0) }
            }
        }
        .contentShape(Rectangle())
        .onTapGesture { isRecording ? stopRecording() : startRecording() }
        .onDisappear { stopRecording() }
    }

    private func startRecording() {
        isRecording = true
        recordedObserver = NotificationCenter.default.addObserver(forName: .shortcutRecorded, object: nil, queue: .main) { notification in
            if let captured = notification.object as? KeyboardShortcut {
                appState.cleanPasteShortcut = captured
            }
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

// MARK: - Clean Paste Mappings Sheet

struct CleanPasteMappingsSheet: View {
    @ObservedObject var appState: AppState

    @State private var formFrom: String = ""
    @State private var formTo: String = ""
    @State private var formLabel: String = ""
    @State private var editingId: UUID? = nil
    @State private var selectedIds: Set<UUID> = []

    private var isEditing: Bool { editingId != nil }
    private var canSave: Bool { !formFrom.isEmpty }

    private var subtitle: String {
        let enabled = appState.cleanPasteMappings.filter(\.isEnabled).count
        let total = appState.cleanPasteMappings.count
        return "\(total) mục · \(enabled) đang bật"
    }

    var body: some View {
        VStack(spacing: 0) {
            SheetHeader("Bảng thay thế ký tự", subtitle: subtitle)
            Divider()
            formSection
            Divider()
            tableContent
            Divider()
            SheetToolbar {
                Button(action: resetToDefaults) {
                    Label("Mặc định", systemImage: "arrow.counterclockwise")
                }
                .buttonStyle(.borderless)
            }
        }
        .frame(width: 500, height: 420)
    }

    private var formSection: some View {
        VStack(spacing: 10) {
            HStack(spacing: 10) {
                VStack(alignment: .leading, spacing: 3) {
                    Text("Ký tự gốc").font(.system(size: 11)).foregroundColor(.secondary)
                    TextField("vd: —", text: $formFrom)
                        .textFieldStyle(.roundedBorder)
                        .frame(width: 80)
                }
                VStack(alignment: .leading, spacing: 3) {
                    Text("Thay bằng").font(.system(size: 11)).foregroundColor(.secondary)
                    TextField("vd: -", text: $formTo)
                        .textFieldStyle(.roundedBorder)
                        .frame(width: 80)
                }
                VStack(alignment: .leading, spacing: 3) {
                    Text("Mô tả").font(.system(size: 11)).foregroundColor(.secondary)
                    TextField("vd: Em dash", text: $formLabel)
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
        if appState.cleanPasteMappings.isEmpty {
            VStack(spacing: 8) {
                Image(systemName: "character.textbox").font(.system(size: 32)).foregroundColor(.secondary)
                Text("Chưa có ký tự thay thế").font(.system(size: 13)).foregroundColor(.secondary)
                Text("Nhấn \"Mặc định\" để khôi phục danh sách mặc định").font(.system(size: 11)).foregroundColor(Color(NSColor.tertiaryLabelColor))
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)
        } else {
            Table(appState.cleanPasteMappings, selection: $selectedIds) {
                TableColumn("") { item in
                    Toggle("", isOn: Binding(
                        get: { item.isEnabled },
                        set: { newValue in
                            if let idx = appState.cleanPasteMappings.firstIndex(where: { $0.id == item.id }) {
                                appState.cleanPasteMappings[idx].isEnabled = newValue
                            }
                        }
                    ))
                    .toggleStyle(.checkbox)
                    .labelsHidden()
                }
                .width(24)

                TableColumn("Gốc") { item in
                    Text(item.from)
                        .font(.system(size: 12, weight: .medium, design: .monospaced))
                        .foregroundColor(item.isEnabled ? .primary : .secondary)
                }
                .width(min: 50, ideal: 60, max: 80)

                TableColumn("Thay bằng") { item in
                    Text(item.to.isEmpty ? "(xoá)" : item.to)
                        .font(.system(size: 12, design: .monospaced))
                        .foregroundColor(item.isEnabled ? .primary : .secondary)
                }
                .width(min: 50, ideal: 70, max: 90)

                TableColumn("Mô tả") { item in
                    Text(item.label)
                        .font(.system(size: 12))
                        .foregroundColor(item.isEnabled ? .primary : .secondary)
                        .lineLimit(1)
                }

                TableColumn("") { item in
                    DeleteButton { deleteItem(item.id) }
                }
                .width(28)
            }
            .tableStyle(.inset(alternatesRowBackgrounds: true))
            .onDeleteCommand { deleteSelected() }
            .onChange(of: selectedIds) { newSelection in
                if newSelection.count == 1, let id = newSelection.first,
                   let item = appState.cleanPasteMappings.first(where: { $0.id == id })
                {
                    selectItem(item)
                } else {
                    clearForm()
                }
            }
        }
    }

    private func selectItem(_ item: CleanPasteMapping) {
        editingId = item.id
        formFrom = item.from
        formTo = item.to
        formLabel = item.label
    }

    private func clearForm() {
        editingId = nil
        formFrom = ""
        formTo = ""
        formLabel = ""
    }

    private func saveForm() {
        if let id = editingId, let idx = appState.cleanPasteMappings.firstIndex(where: { $0.id == id }) {
            appState.cleanPasteMappings[idx].from = formFrom
            appState.cleanPasteMappings[idx].to = formTo
            appState.cleanPasteMappings[idx].label = formLabel
        } else {
            appState.cleanPasteMappings.append(CleanPasteMapping(from: formFrom, to: formTo, label: formLabel, isEnabled: true))
        }
        clearForm()
    }

    private func deleteSelected() {
        guard !selectedIds.isEmpty else { return }
        appState.cleanPasteMappings.removeAll { selectedIds.contains($0.id) }
        selectedIds.removeAll()
        clearForm()
    }

    private func deleteItem(_ id: UUID) {
        appState.cleanPasteMappings.removeAll { $0.id == id }
        if editingId == id { clearForm() }
        selectedIds.remove(id)
    }

    private func resetToDefaults() {
        appState.cleanPasteMappings = CleanPasteMapping.defaults
        clearForm()
        selectedIds.removeAll()
    }
}
