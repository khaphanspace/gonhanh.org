import SwiftUI

struct UpdateView: View {
    @Environment(\.colorScheme) private var colorScheme
    @ObservedObject var updateManager = UpdateManager.shared

    var body: some View {
        VStack(spacing: 0) {
            content
            Divider()
            footer
        }
        .frame(width: 360)
        .background(colorScheme == .dark ? Color.black.opacity(0.2) : Color.white)
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
            readyView
        case .error(let message):
            errorView(message)
        }
    }

    // MARK: - States

    private var idleView: some View {
        VStack(spacing: 16) {
            Spacer()

            Image(nsImage: AppMetadata.logo)
                .resizable()
                .frame(width: 64, height: 64)
                .shadow(color: .black.opacity(0.1), radius: 4, y: 2)

            Text(AppMetadata.name)
                .font(.system(size: 20, weight: .bold, design: .rounded))

            versionBadge(label: "Phiên bản", value: AppMetadata.version)

            if let lastCheck = updateManager.lastCheckDate {
                Text("Kiểm tra lần cuối: \(lastCheck.formatted(.relative(presentation: .named)))")
                    .font(.caption)
                    .foregroundStyle(.tertiary)
            }

            Spacer()

            Button("Kiểm tra cập nhật") {
                updateManager.checkForUpdatesManually()
            }
            .buttonStyle(.borderedProminent)

            Spacer()
        }
        .padding(.horizontal, 28)
        .padding(.vertical, 24)
    }

    private var checkingView: some View {
        VStack(spacing: 16) {
            Spacer()

            ProgressView()
                .scaleEffect(1.5)

            Text("Đang kiểm tra...")
                .font(.system(size: 18, weight: .medium, design: .rounded))

            Spacer()
        }
        .padding(.horizontal, 28)
        .padding(.vertical, 24)
    }

    private var upToDateView: some View {
        VStack(spacing: 16) {
            Spacer()

            iconCircle(icon: "checkmark", color: .green)

            Text("Đã cập nhật mới nhất")
                .font(.system(size: 18, weight: .bold, design: .rounded))

            versionBadge(label: "Phiên bản", value: AppMetadata.version)

            Spacer()

            Button("Kiểm tra lại") {
                updateManager.checkForUpdatesManually()
            }

            Spacer()
        }
        .padding(.horizontal, 28)
        .padding(.vertical, 24)
    }

    private func availableView(_ info: UpdateInfo) -> some View {
        VStack(spacing: 16) {
            Spacer()

            iconCircle(icon: "arrow.down", color: .blue)

            Text("Có phiên bản mới")
                .font(.system(size: 18, weight: .bold, design: .rounded))

            // Version comparison
            HStack(spacing: 16) {
                versionBadge(label: "Hiện tại", value: AppMetadata.version)

                Image(systemName: "arrow.right")
                    .font(.caption)
                    .foregroundStyle(.tertiary)

                versionBadge(label: "Mới", value: info.version, highlight: true)
            }

            // Release notes
            if !info.releaseNotes.isEmpty {
                ScrollView {
                    Text(info.releaseNotes)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }
                .frame(maxHeight: 80)
                .padding(10)
                .background(
                    RoundedRectangle(cornerRadius: 8)
                        .fill(colorScheme == .dark ? Color.white.opacity(0.06) : Color.black.opacity(0.03))
                )
            }

            Spacer()

            // Actions
            VStack(spacing: 10) {
                Button("Tải về") {
                    updateManager.downloadUpdate(info)
                }
                .buttonStyle(.borderedProminent)

                HStack(spacing: 20) {
                    Button("Để sau") {
                        updateManager.state = .idle
                    }
                    .foregroundStyle(.secondary)

                    Button("Bỏ qua") {
                        updateManager.skipVersion(info.version)
                    }
                    .foregroundStyle(.tertiary)
                }
                .font(.callout)
            }

            Spacer()
        }
        .padding(.horizontal, 28)
        .padding(.vertical, 24)
    }

    private func downloadingView(_ progress: Double) -> some View {
        VStack(spacing: 16) {
            Spacer()

            ZStack {
                Circle()
                    .stroke(colorScheme == .dark ? Color.white.opacity(0.1) : Color.black.opacity(0.08), lineWidth: 4)
                    .frame(width: 64, height: 64)

                Circle()
                    .trim(from: 0, to: progress)
                    .stroke(Color.accentColor, style: StrokeStyle(lineWidth: 4, lineCap: .round))
                    .frame(width: 64, height: 64)
                    .rotationEffect(.degrees(-90))

                Text("\(Int(progress * 100))%")
                    .font(.system(size: 14, weight: .bold, design: .rounded))
            }

            Text("Đang tải về...")
                .font(.system(size: 18, weight: .medium, design: .rounded))

            Spacer()

            Button("Hủy") {
                updateManager.cancelDownload()
            }
            .foregroundStyle(.secondary)

            Spacer()
        }
        .padding(.horizontal, 28)
        .padding(.vertical, 24)
    }

    private var readyView: some View {
        VStack(spacing: 16) {
            Spacer()

            iconCircle(icon: "checkmark", color: .green)

            Text("Sẵn sàng cài đặt")
                .font(.system(size: 18, weight: .bold, design: .rounded))

            Text("Ứng dụng sẽ thoát để cài đặt")
                .font(.callout)
                .foregroundStyle(.secondary)

            Spacer()

            Button("Cài đặt ngay") {
                updateManager.installUpdate()
            }
            .buttonStyle(.borderedProminent)

            Button("Để sau") {
                updateManager.state = .idle
            }
            .font(.callout)
            .foregroundStyle(.secondary)

            Spacer()
        }
        .padding(.horizontal, 28)
        .padding(.vertical, 24)
    }

    private func errorView(_ message: String) -> some View {
        VStack(spacing: 16) {
            Spacer()

            iconCircle(icon: "exclamationmark", color: .orange)

            Text("Lỗi kết nối")
                .font(.system(size: 18, weight: .bold, design: .rounded))

            Text(message)
                .font(.callout)
                .foregroundStyle(.secondary)
                .multilineTextAlignment(.center)

            Spacer()

            Button("Thử lại") {
                updateManager.checkForUpdatesManually()
            }
            .buttonStyle(.borderedProminent)

            Spacer()
        }
        .padding(.horizontal, 28)
        .padding(.vertical, 24)
    }

    // MARK: - Components

    private func iconCircle(icon: String, color: Color) -> some View {
        ZStack {
            Circle()
                .fill(color.opacity(colorScheme == .dark ? 0.2 : 0.1))
                .frame(width: 64, height: 64)

            Image(systemName: icon)
                .font(.system(size: 28, weight: .medium))
                .foregroundStyle(color)
        }
    }

    private func versionBadge(label: String, value: String, highlight: Bool = false) -> some View {
        HStack(spacing: 4) {
            Text(label)
                .font(.caption2)
                .foregroundStyle(.tertiary)
            Text(value)
                .font(.caption)
                .fontWeight(.medium)
                .foregroundStyle(highlight ? .green : .secondary)
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 4)
        .background(
            RoundedRectangle(cornerRadius: 6)
                .fill(colorScheme == .dark ? Color.white.opacity(0.08) : Color.black.opacity(0.04))
        )
    }

    // MARK: - Footer

    private var footer: some View {
        Link(destination: URL(string: AppMetadata.repository + "/releases")!) {
            HStack(spacing: 4) {
                Text("Xem trên GitHub")
                Image(systemName: "arrow.up.right")
            }
            .font(.caption)
            .foregroundStyle(.secondary)
        }
        .buttonStyle(.plain)
        .frame(maxWidth: .infinity)
        .padding(.vertical, 12)
        .background(colorScheme == .dark ? Color.white.opacity(0.02) : Color.black.opacity(0.02))
        .onHover { hovering in
            if hovering {
                NSCursor.pointingHand.push()
            } else {
                NSCursor.pop()
            }
        }
    }
}

#Preview {
    UpdateView()
}
