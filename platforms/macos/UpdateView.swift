import SwiftUI

struct UpdateView: View {
    @Environment(\.colorScheme) private var colorScheme
    @ObservedObject var updateManager = UpdateManager.shared

    var body: some View {
        content
            .frame(width: 380)
            .background(Color(NSColor.windowBackgroundColor))
    }

    @ViewBuilder
    private var content: some View {
        switch updateManager.state {
        case .idle:
            idleView
        case .checking:
            checkingView
        case .upToDate:
            upToDateView
        case .available(let info):
            availableView(info)
        case .downloading(let progress):
            downloadingView(progress)
        case .readyToInstall:
            readyToInstallView
        case .installing:
            installingView
        case .error(let message):
            errorView(message)
        }
    }

    // MARK: - States

    private var idleView: some View {
        VStack(spacing: 20) {
            Image(nsImage: AppMetadata.logo)
                .resizable()
                .frame(width: 72, height: 72)

            VStack(spacing: 8) {
                Text(AppMetadata.name)
                    .font(.system(size: 18, weight: .semibold))
                    .foregroundColor(Color(NSColor.labelColor))

                Text("Phiên bản \(AppMetadata.version)")
                    .font(.system(size: 12))
                    .foregroundColor(Color(NSColor.tertiaryLabelColor))
            }

            if let lastCheck = updateManager.lastCheckDate {
                Text("Kiểm tra lần cuối: \(lastCheck.formatted(.relative(presentation: .named)))")
                    .font(.system(size: 11))
                    .foregroundColor(Color(NSColor.tertiaryLabelColor))
            }

            Button {
                updateManager.checkForUpdatesManually()
            } label: {
                Text("Kiểm tra cập nhật")
                    .font(.system(size: 13, weight: .medium))
                    .frame(width: 160)
                    .padding(.vertical, 8)
            }
            .buttonStyle(.borderedProminent)
            .padding(.top, 8)
        }
        .padding(.horizontal, 32)
        .padding(.vertical, 32)
    }

    private var checkingView: some View {
        VStack(spacing: 16) {
            ProgressView()
                .scaleEffect(1.2)

            Text("Đang kiểm tra...")
                .font(.system(size: 15, weight: .medium))
                .foregroundColor(Color(NSColor.labelColor))
        }
        .padding(.horizontal, 32)
        .padding(.vertical, 48)
    }

    private var upToDateView: some View {
        VStack(spacing: 20) {
            ZStack {
                Circle()
                    .fill(Color.green.opacity(0.1))
                    .frame(width: 64, height: 64)

                Image(systemName: "checkmark.circle.fill")
                    .font(.system(size: 36))
                    .foregroundColor(.green)
            }

            VStack(spacing: 8) {
                Text("Đã cập nhật mới nhất")
                    .font(.system(size: 16, weight: .semibold))
                    .foregroundColor(Color(NSColor.labelColor))

                Text("Phiên bản \(AppMetadata.version)")
                    .font(.system(size: 12))
                    .foregroundColor(Color(NSColor.tertiaryLabelColor))
            }

            Button {
                updateManager.checkForUpdatesManually()
            } label: {
                Text("Kiểm tra lại")
                    .font(.system(size: 13))
            }
            .buttonStyle(.plain)
            .foregroundColor(Color(NSColor.secondaryLabelColor))
            .padding(.top, 8)
        }
        .padding(.horizontal, 32)
        .padding(.vertical, 32)
    }

    private func availableView(_ info: UpdateInfo) -> some View {
        VStack(spacing: 0) {
            // Header
            VStack(spacing: 12) {
                ZStack {
                    Circle()
                        .fill(Color.accentColor.opacity(0.1))
                        .frame(width: 56, height: 56)

                    Image(systemName: "arrow.down.circle.fill")
                        .font(.system(size: 32))
                        .foregroundColor(.accentColor)
                }

                Text("Có phiên bản mới")
                    .font(.system(size: 16, weight: .semibold))
                    .foregroundColor(Color(NSColor.labelColor))

                // Version comparison
                HStack(spacing: 8) {
                    Text(AppMetadata.version)
                        .foregroundColor(Color(NSColor.secondaryLabelColor))

                    Image(systemName: "arrow.right")
                        .font(.system(size: 10, weight: .bold))
                        .foregroundColor(Color(NSColor.tertiaryLabelColor))

                    Text(info.version)
                        .foregroundColor(.green)
                        .fontWeight(.medium)
                }
                .font(.system(size: 13, design: .monospaced))
            }
            .padding(.top, 24)
            .padding(.bottom, 16)

            // Release notes
            let rawNotes = info.releaseNotes
                .trimmingCharacters(in: .whitespacesAndNewlines)
                .replacingOccurrences(of: "### ", with: "")
                .replacingOccurrences(of: "## ", with: "")

            if !rawNotes.isEmpty {
                VStack(alignment: .leading, spacing: 6) {
                    Text("Có gì mới")
                        .font(.system(size: 11, weight: .medium))
                        .foregroundColor(Color(NSColor.secondaryLabelColor))
                        .padding(.horizontal, 4)

                    ScrollView {
                        Text(rawNotes)
                            .font(.system(size: 12))
                            .foregroundColor(Color(NSColor.labelColor))
                            .frame(maxWidth: .infinity, alignment: .leading)
                            .lineSpacing(5)
                            .fixedSize(horizontal: false, vertical: true)
                            .padding(12)
                    }
                    .frame(height: 140)
                    .background(
                        RoundedRectangle(cornerRadius: 8)
                            .fill(Color(NSColor.controlBackgroundColor).opacity(0.5))
                    )
                    .overlay(
                        RoundedRectangle(cornerRadius: 8)
                            .stroke(Color(NSColor.separatorColor).opacity(0.5), lineWidth: 0.5)
                    )
                }
                .padding(.horizontal, 28)
                .padding(.bottom, 20)
            }

            // Actions
            VStack(spacing: 10) {
                Button {
                    updateManager.downloadUpdate(info)
                } label: {
                    Text("Cập nhật ngay")
                        .font(.system(size: 13, weight: .medium))
                        .frame(maxWidth: .infinity)
                        .padding(.vertical, 8)
                }
                .buttonStyle(.borderedProminent)

                HStack(spacing: 20) {
                    Button("Để sau") {
                        updateManager.state = .idle
                    }
                    .foregroundColor(Color(NSColor.secondaryLabelColor))

                    Button("Bỏ qua") {
                        updateManager.skipVersion(info.version)
                    }
                    .foregroundColor(Color(NSColor.tertiaryLabelColor))
                }
                .font(.system(size: 12))
                .buttonStyle(.plain)
            }
            .padding(.horizontal, 28)
            .padding(.bottom, 24)
        }
    }

    private func downloadingView(_ progress: Double) -> some View {
        VStack(spacing: 20) {
            ZStack {
                Circle()
                    .stroke(Color(NSColor.separatorColor).opacity(0.3), lineWidth: 4)
                    .frame(width: 72, height: 72)

                Circle()
                    .trim(from: 0, to: progress)
                    .stroke(Color.accentColor, style: StrokeStyle(lineWidth: 4, lineCap: .round))
                    .frame(width: 72, height: 72)
                    .rotationEffect(.degrees(-90))
                    .animation(.linear(duration: 0.2), value: progress)

                Text("\(Int(progress * 100))%")
                    .font(.system(size: 15, weight: .semibold))
                    .foregroundColor(Color(NSColor.labelColor))
            }

            VStack(spacing: 6) {
                Text("Đang tải về...")
                    .font(.system(size: 15, weight: .medium))
                    .foregroundColor(Color(NSColor.labelColor))

                Text("Vui lòng không tắt ứng dụng")
                    .font(.system(size: 11))
                    .foregroundColor(Color(NSColor.tertiaryLabelColor))
            }

            Button {
                updateManager.cancelDownload()
            } label: {
                Text("Hủy")
                    .font(.system(size: 13))
            }
            .buttonStyle(.plain)
            .foregroundColor(Color(NSColor.secondaryLabelColor))
            .padding(.top, 8)
        }
        .padding(.horizontal, 32)
        .padding(.vertical, 32)
    }

    private var readyToInstallView: some View {
        VStack(spacing: 20) {
            ZStack {
                Circle()
                    .fill(Color.orange.opacity(0.1))
                    .frame(width: 64, height: 64)

                Image(systemName: "arrow.clockwise.circle.fill")
                    .font(.system(size: 36))
                    .foregroundColor(.orange)
            }

            VStack(spacing: 8) {
                Text("Sẵn sàng cập nhật")
                    .font(.system(size: 16, weight: .semibold))
                    .foregroundColor(Color(NSColor.labelColor))

                Text("Nhấn nút bên dưới để khởi động lại")
                    .font(.system(size: 12))
                    .foregroundColor(Color(NSColor.tertiaryLabelColor))
            }

            Button {
                updateManager.installReadyUpdate()
            } label: {
                Text("Khởi động lại để cập nhật")
                    .font(.system(size: 13, weight: .medium))
                    .frame(width: 200)
                    .padding(.vertical, 8)
            }
            .buttonStyle(.borderedProminent)
            .padding(.top, 8)
        }
        .padding(.horizontal, 32)
        .padding(.vertical, 32)
    }

    private var installingView: some View {
        VStack(spacing: 16) {
            ProgressView()
                .scaleEffect(1.2)

            VStack(spacing: 6) {
                Text("Đang cài đặt...")
                    .font(.system(size: 15, weight: .medium))
                    .foregroundColor(Color(NSColor.labelColor))

                Text("Ứng dụng sẽ tự khởi động lại")
                    .font(.system(size: 11))
                    .foregroundColor(Color(NSColor.tertiaryLabelColor))
            }
        }
        .padding(.horizontal, 32)
        .padding(.vertical, 48)
    }

    private func errorView(_ message: String) -> some View {
        VStack(spacing: 20) {
            ZStack {
                Circle()
                    .fill(Color.orange.opacity(0.1))
                    .frame(width: 64, height: 64)

                Image(systemName: "exclamationmark.triangle.fill")
                    .font(.system(size: 32))
                    .foregroundColor(.orange)
            }

            VStack(spacing: 8) {
                Text("Không thể kiểm tra")
                    .font(.system(size: 16, weight: .semibold))
                    .foregroundColor(Color(NSColor.labelColor))

                Text(message)
                    .font(.system(size: 12))
                    .foregroundColor(Color(NSColor.secondaryLabelColor))
                    .multilineTextAlignment(.center)
                    .lineLimit(2)
            }

            Button {
                updateManager.checkForUpdatesManually()
            } label: {
                Text("Thử lại")
                    .font(.system(size: 13, weight: .medium))
                    .frame(width: 120)
                    .padding(.vertical, 8)
            }
            .buttonStyle(.borderedProminent)
            .padding(.top, 8)
        }
        .padding(.horizontal, 32)
        .padding(.vertical, 32)
    }

}

#Preview {
    UpdateView()
}
