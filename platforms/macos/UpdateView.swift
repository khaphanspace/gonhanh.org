import SwiftUI

struct UpdateView: View {
    @ObservedObject var updateManager = UpdateManager.shared

    var body: some View {
        VStack(spacing: 0) {
            content
            Divider()
            footer
        }
        .frame(width: 360)
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

            VStack(spacing: 4) {
                Text("Phiên bản \(AppMetadata.version)")
                    .font(.title3)
                if let lastCheck = updateManager.lastCheckDate {
                    Text("Kiểm tra lần cuối: \(lastCheck.formatted(.relative(presentation: .named)))")
                        .font(.caption)
                        .foregroundStyle(.tertiary)
                }
            }

            Spacer()

            Button("Kiểm tra cập nhật") {
                updateManager.checkForUpdatesManually()
            }
            .buttonStyle(.borderedProminent)

            Spacer()
        }
        .padding(.horizontal, 28)
        .padding(.vertical, 20)
    }

    private var checkingView: some View {
        VStack(spacing: 16) {
            Spacer()

            ProgressView()
                .scaleEffect(1.5)

            Text("Đang kiểm tra...")
                .font(.title3)

            Spacer()
        }
        .padding(.horizontal, 28)
        .padding(.vertical, 20)
    }

    private var upToDateView: some View {
        VStack(spacing: 16) {
            Spacer()

            Image(systemName: "checkmark.circle.fill")
                .font(.system(size: 48))
                .foregroundStyle(.green)

            Text("Đã cập nhật mới nhất")
                .font(.title3.bold())

            Text("Phiên bản \(AppMetadata.version)")
                .foregroundStyle(.secondary)

            Spacer()

            Button("Kiểm tra lại") {
                updateManager.checkForUpdatesManually()
            }

            Spacer()
        }
        .padding(.horizontal, 28)
        .padding(.vertical, 20)
    }

    private func availableView(_ info: UpdateInfo) -> some View {
        VStack(spacing: 16) {
            Spacer()

            Image(systemName: "arrow.down.circle.fill")
                .font(.system(size: 48))
                .foregroundStyle(.blue)

            Text("Có phiên bản mới")
                .font(.title3.bold())

            // Version comparison
            HStack(spacing: 20) {
                VStack(spacing: 2) {
                    Text("Hiện tại")
                        .font(.caption)
                        .foregroundStyle(.tertiary)
                    Text(AppMetadata.version)
                        .foregroundStyle(.secondary)
                }

                Image(systemName: "arrow.right")
                    .foregroundStyle(.tertiary)

                VStack(spacing: 2) {
                    Text("Mới")
                        .font(.caption)
                        .foregroundStyle(.tertiary)
                    Text(info.version)
                        .fontWeight(.medium)
                        .foregroundStyle(.green)
                }
            }

            // Release notes
            if !info.releaseNotes.isEmpty {
                ScrollView {
                    Text(info.releaseNotes)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }
                .frame(maxHeight: 100)
                .padding(10)
                .background(
                    RoundedRectangle(cornerRadius: 6)
                        .fill(Color.secondary.opacity(0.1))
                )
            }

            Spacer()

            // Actions
            VStack(spacing: 8) {
                Button("Tải về") {
                    updateManager.downloadUpdate(info)
                }
                .buttonStyle(.borderedProminent)

                HStack(spacing: 16) {
                    Button("Để sau") {
                        updateManager.state = .idle
                    }
                    Button("Bỏ qua") {
                        updateManager.skipVersion(info.version)
                    }
                    .foregroundStyle(.secondary)
                }
                .font(.callout)
            }

            Spacer()
        }
        .padding(.horizontal, 28)
        .padding(.vertical, 20)
    }

    private func downloadingView(_ progress: Double) -> some View {
        VStack(spacing: 16) {
            Spacer()

            ZStack {
                Circle()
                    .stroke(Color.secondary.opacity(0.2), lineWidth: 4)
                    .frame(width: 60, height: 60)

                Circle()
                    .trim(from: 0, to: progress)
                    .stroke(Color.accentColor, style: StrokeStyle(lineWidth: 4, lineCap: .round))
                    .frame(width: 60, height: 60)
                    .rotationEffect(.degrees(-90))

                Text("\(Int(progress * 100))%")
                    .font(.caption.bold())
            }

            Text("Đang tải về...")
                .font(.title3)

            Spacer()

            Button("Hủy") {
                updateManager.cancelDownload()
            }

            Spacer()
        }
        .padding(.horizontal, 28)
        .padding(.vertical, 20)
    }

    private var readyView: some View {
        VStack(spacing: 16) {
            Spacer()

            Image(systemName: "checkmark.circle.fill")
                .font(.system(size: 48))
                .foregroundStyle(.green)

            Text("Sẵn sàng cài đặt")
                .font(.title3.bold())

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

            Spacer()
        }
        .padding(.horizontal, 28)
        .padding(.vertical, 20)
    }

    private func errorView(_ message: String) -> some View {
        VStack(spacing: 16) {
            Spacer()

            Image(systemName: "exclamationmark.triangle.fill")
                .font(.system(size: 48))
                .foregroundStyle(.orange)

            Text("Lỗi kết nối")
                .font(.title3.bold())

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
        .padding(.vertical, 20)
    }

    // MARK: - Footer

    private var footer: some View {
        Link(destination: URL(string: AppMetadata.repository + "/releases")!) {
            Label("Xem trên GitHub", systemImage: "arrow.up.right")
                .font(.caption)
        }
        .frame(maxWidth: .infinity)
        .padding(.vertical, 12)
    }
}

#Preview {
    UpdateView()
}
