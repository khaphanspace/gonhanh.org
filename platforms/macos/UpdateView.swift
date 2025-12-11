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
        .background(Color.adaptiveSurface(colorScheme))
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
        case .installing:
            installingView
        case .error(let message):
            errorView(message)
        }
    }

    // MARK: - States

    private var idleView: some View {
        VStack(spacing: DesignTokens.Spacing.space4) {
            Spacer()

            Image(nsImage: AppMetadata.logo)
                .resizable()
                .frame(width: 64, height: 64)
                .shadow(color: .black.opacity(0.1), radius: 4, y: 2)

            Text(AppMetadata.name)
                .font(.system(size: DesignTokens.Typography.textXl, weight: .bold, design: .rounded))

            versionBadge(label: "Phiên bản", value: AppMetadata.version)

            if let lastCheck = updateManager.lastCheckDate {
                Text("Kiểm tra lần cuối: \(lastCheck.formatted(.relative(presentation: .named)))")
                    .font(.caption)
                    .foregroundStyle(Color.inkTertiary)
            }

            Spacer()

            Button("Kiểm tra cập nhật") {
                updateManager.checkForUpdatesManually()
            }
            .buttonStyle(.borderedProminent)
            .tint(Color.accent)

            Spacer()
        }
        .padding(.horizontal, 28)
        .padding(.vertical, DesignTokens.Spacing.space6)
    }

    private var checkingView: some View {
        VStack(spacing: DesignTokens.Spacing.space4) {
            Spacer()

            ProgressView()
                .scaleEffect(1.5)

            Text("Đang kiểm tra...")
                .font(.system(size: DesignTokens.Typography.textLg, weight: .medium, design: .rounded))

            Spacer()
        }
        .padding(.horizontal, 28)
        .padding(.vertical, DesignTokens.Spacing.space6)
    }

    private var upToDateView: some View {
        VStack(spacing: DesignTokens.Spacing.space4) {
            Spacer()

            iconCircle(icon: "checkmark", color: Color.success)

            Text("Đã cập nhật mới nhất")
                .font(.system(size: DesignTokens.Typography.textLg, weight: .bold, design: .rounded))

            versionBadge(label: "Phiên bản", value: AppMetadata.version)

            Spacer()

            Button("Kiểm tra lại") {
                updateManager.checkForUpdatesManually()
            }

            Spacer()
        }
        .padding(.horizontal, 28)
        .padding(.vertical, DesignTokens.Spacing.space6)
    }

    private func availableView(_ info: UpdateInfo) -> some View {
        VStack(spacing: 0) {
            // Header
            VStack(spacing: DesignTokens.Spacing.space3) {
                ZStack {
                    Circle()
                        .fill(Color.accent.opacity(0.15))
                        .frame(width: 56, height: 56)

                    Image(systemName: "arrow.down.circle.fill")
                        .font(.system(size: 32))
                        .foregroundStyle(Color.accent)
                }

                Text("Có phiên bản mới")
                    .font(.system(size: 17, weight: .semibold))

                // Version comparison
                HStack(spacing: DesignTokens.Spacing.space2) {
                    Text(AppMetadata.version)
                        .foregroundStyle(Color.inkSecondary)

                    Image(systemName: "arrow.right")
                        .font(.system(size: 10, weight: .bold))
                        .foregroundStyle(Color.inkTertiary)

                    Text(info.version)
                        .foregroundStyle(Color.success)
                        .fontWeight(.medium)
                }
                .font(.system(size: 13, design: .monospaced))
            }
            .padding(.top, 28)
            .padding(.bottom, DesignTokens.Spacing.space5)

            // Release notes
            let notes = info.releaseNotes.trimmingCharacters(in: .whitespacesAndNewlines)
            if !notes.isEmpty {
                ScrollView {
                    Text(notes)
                        .font(.system(size: DesignTokens.Typography.textXs))
                        .foregroundStyle(Color.inkSecondary)
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .lineSpacing(3)
                }
                .frame(maxHeight: 100)
                .padding(DesignTokens.Spacing.space3)
                .background(
                    RoundedRectangle(cornerRadius: DesignTokens.Radius.md)
                        .fill(Color.adaptiveSurfaceSecondary(colorScheme))
                )
                .padding(.horizontal, DesignTokens.Spacing.space6)
                .padding(.bottom, DesignTokens.Spacing.space5)
            }

            // Actions
            VStack(spacing: DesignTokens.Spacing.space3) {
                Button {
                    updateManager.downloadUpdate(info)
                } label: {
                    Text("Cập nhật ngay")
                        .font(.system(size: 13, weight: .medium))
                        .frame(maxWidth: .infinity)
                        .padding(.vertical, DesignTokens.Spacing.space2)
                }
                .buttonStyle(.borderedProminent)
                .tint(Color.accent)
                .controlSize(.large)

                HStack(spacing: DesignTokens.Spacing.space6) {
                    Button("Để sau") {
                        updateManager.state = .idle
                    }

                    Button("Bỏ qua phiên bản này") {
                        updateManager.skipVersion(info.version)
                    }
                    .foregroundStyle(Color.inkTertiary)
                }
                .font(.system(size: DesignTokens.Typography.textXs))
                .buttonStyle(.plain)
                .foregroundStyle(Color.inkSecondary)
            }
            .padding(.horizontal, DesignTokens.Spacing.space6)
            .padding(.bottom, DesignTokens.Spacing.space6)
        }
    }

    private func downloadingView(_ progress: Double) -> some View {
        VStack(spacing: DesignTokens.Spacing.space4) {
            Spacer()

            ZStack {
                Circle()
                    .stroke(Color.adaptiveBorder(colorScheme), lineWidth: 4)
                    .frame(width: 64, height: 64)

                Circle()
                    .trim(from: 0, to: progress)
                    .stroke(Color.accent, style: StrokeStyle(lineWidth: 4, lineCap: .round))
                    .frame(width: 64, height: 64)
                    .rotationEffect(.degrees(-90))

                Text("\(Int(progress * 100))%")
                    .font(.system(size: DesignTokens.Typography.textSm, weight: .bold, design: .rounded))
            }

            Text("Đang tải về...")
                .font(.system(size: DesignTokens.Typography.textLg, weight: .medium, design: .rounded))

            Spacer()

            Button("Hủy") {
                updateManager.cancelDownload()
            }
            .foregroundStyle(Color.inkSecondary)

            Spacer()
        }
        .padding(.horizontal, 28)
        .padding(.vertical, DesignTokens.Spacing.space6)
    }

    private var installingView: some View {
        VStack(spacing: DesignTokens.Spacing.space4) {
            Spacer()

            ProgressView()
                .scaleEffect(1.5)

            Text("Đang cài đặt...")
                .font(.system(size: DesignTokens.Typography.textLg, weight: .medium, design: .rounded))

            Text("Ứng dụng sẽ tự khởi động lại")
                .font(.callout)
                .foregroundStyle(Color.inkSecondary)

            Spacer()
        }
        .padding(.horizontal, 28)
        .padding(.vertical, DesignTokens.Spacing.space6)
    }

    private func errorView(_ message: String) -> some View {
        VStack(spacing: DesignTokens.Spacing.space4) {
            Spacer()

            iconCircle(icon: "exclamationmark", color: Color.warning)

            Text("Lỗi kết nối")
                .font(.system(size: DesignTokens.Typography.textLg, weight: .bold, design: .rounded))

            Text(message)
                .font(.callout)
                .foregroundStyle(Color.inkSecondary)
                .multilineTextAlignment(.center)

            Spacer()

            Button("Thử lại") {
                updateManager.checkForUpdatesManually()
            }
            .buttonStyle(.borderedProminent)
            .tint(Color.accent)

            Spacer()
        }
        .padding(.horizontal, 28)
        .padding(.vertical, DesignTokens.Spacing.space6)
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
        HStack(spacing: DesignTokens.Spacing.space1) {
            Text(label)
                .font(.caption2)
                .foregroundStyle(Color.inkTertiary)
            Text(value)
                .font(.caption)
                .fontWeight(.medium)
                .foregroundStyle(highlight ? Color.success : Color.inkSecondary)
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 4)
        .background(
            RoundedRectangle(cornerRadius: DesignTokens.Radius.sm)
                .fill(Color.adaptiveSurfaceSecondary(colorScheme))
        )
    }

    // MARK: - Footer

    private var footer: some View {
        Link(destination: URL(string: AppMetadata.repository + "/releases")!) {
            HStack(spacing: DesignTokens.Spacing.space1) {
                Text("Xem trên GitHub")
                Image(systemName: "arrow.up.right")
            }
            .font(.caption)
            .foregroundStyle(Color.inkSecondary)
        }
        .buttonStyle(.plain)
        .frame(maxWidth: .infinity)
        .padding(.vertical, DesignTokens.Spacing.space3)
        .background(Color.adaptiveSurfaceSecondary(colorScheme))
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
