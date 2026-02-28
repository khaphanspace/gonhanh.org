import AppKit
import SwiftUI

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

// MARK: - App Picker Row

struct AppPickerRow: View {
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
