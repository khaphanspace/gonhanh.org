import AppKit
import SwiftUI

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
        timer?.invalidate()
        timer = nil
    }

    private func loadLog() {
        let path = "/tmp/gonhanh_debug.log"
        let attrs = try? FileManager.default.attributesOfItem(atPath: path)
        let size = attrs?[.size] as? UInt64 ?? 0

        // File cleared or deleted
        if size == 0 {
            if !logLines.isEmpty { logLines = []; lastFileSize = 0 }
            return
        }

        // File truncated (e.g., user cleared log) — reset and re-read
        if size < lastFileSize { lastFileSize = 0 }

        guard size != lastFileSize else { return }

        // Tail-read: only read new bytes since last poll
        guard let handle = FileHandle(forReadingAtPath: path) else { return }
        defer { handle.closeFile() }
        handle.seek(toFileOffset: lastFileSize)
        let newData = handle.readDataToEndOfFile()
        lastFileSize = size

        guard let newContent = String(data: newData, encoding: .utf8), !newContent.isEmpty else { return }
        let newLines = newContent.components(separatedBy: "\n").filter { !$0.isEmpty }
        let combined = logLines + newLines
        let tail = Array(combined.suffix(80))
        if tail != logLines { logLines = tail }
    }
}
