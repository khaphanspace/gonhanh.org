import SwiftUI

struct OnboardingView: View {
    @Environment(\.colorScheme) private var colorScheme
    @State private var step = 0
    @State private var hasPermission = false
    @State private var selectedMode: InputMode = .telex

    private let timer = Timer.publish(every: 1, on: .main, in: .common).autoconnect()
    private var totalSteps: Int { step >= 10 ? 2 : 3 }
    private var stepIndex: Int { step >= 10 ? step - 10 : step }

    var body: some View {
        VStack(spacing: 0) {
            content.frame(height: 320)
            Divider()
            footer
        }
        .frame(width: 440)
        .onAppear {
            hasPermission = AXIsProcessTrusted()
            if UserDefaults.standard.bool(forKey: SettingsKey.permissionGranted) && hasPermission {
                step = 10
            }
        }
        .onReceive(timer) { _ in
            hasPermission = AXIsProcessTrusted()
            if step == 1 && hasPermission { step = 2 }
        }
    }

    @ViewBuilder
    private var content: some View {
        switch step {
        case 0:  WelcomeStep()
        case 1:  PermissionStep()
        case 2:  ReadyStep()
        case 10: SuccessStep()
        case 11: SetupStep(selectedMode: $selectedMode)
        default: EmptyView()
        }
    }

    private var footer: some View {
        HStack {
            HStack(spacing: 6) {
                ForEach(0..<totalSteps, id: \.self) { i in
                    Circle()
                        .fill(i == stepIndex ? Color.accentColor : Color.secondary.opacity(0.3))
                        .frame(width: 6, height: 6)
                }
            }
            Spacer()
            if step == 1 {
                Button("Quay lại") { step = 0 }
            }
            primaryButton
        }
        .padding(.horizontal, 20)
        .padding(.vertical, 16)
    }

    @ViewBuilder
    private var primaryButton: some View {
        switch step {
        case 0:  Button("Tiếp tục") { step = 1 }.buttonStyle(.borderedProminent)
        case 1:  Button("Mở Cài đặt") { openSettings() }.buttonStyle(.borderedProminent)
        case 2:  Button("Khởi động lại") { restart() }.buttonStyle(.borderedProminent)
        case 10: Button("Tiếp tục") { step = 11 }.buttonStyle(.borderedProminent)
        case 11: Button("Hoàn tất") { finish() }.buttonStyle(.borderedProminent)
        default: EmptyView()
        }
    }

    private func openSettings() {
        NSWorkspace.shared.open(URL(string: "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")!)
    }

    private func restart() {
        UserDefaults.standard.set(selectedMode.rawValue, forKey: SettingsKey.method)
        UserDefaults.standard.set(true, forKey: SettingsKey.permissionGranted)
        UserDefaults.standard.set(false, forKey: SettingsKey.hasCompletedOnboarding)
        let task = Process()
        task.launchPath = "/bin/sh"
        task.arguments = ["-c", "sleep 0.5 && open \"\(Bundle.main.bundlePath)\""]
        try? task.run()
        NSApp.terminate(nil)
    }

    private func finish() {
        UserDefaults.standard.set(selectedMode.rawValue, forKey: SettingsKey.method)
        UserDefaults.standard.set(true, forKey: SettingsKey.hasCompletedOnboarding)
        NotificationCenter.default.post(name: .onboardingCompleted, object: nil)
        NSApp.keyWindow?.close()
    }
}

// MARK: - Steps

private struct WelcomeStep: View {
    var body: some View {
        StepLayout {
            Image(nsImage: AppMetadata.logo)
                .resizable()
                .frame(width: 80, height: 80)

            Text("Chào mừng đến với \(AppMetadata.name)")
                .font(.title2.bold())

            Text(AppMetadata.tagline)
                .foregroundStyle(.secondary)
        }
    }
}

private struct PermissionStep: View {
    var body: some View {
        StepLayout {
            Image(systemName: "hand.raised.fill")
                .font(.system(size: 40))
                .foregroundStyle(.orange)

            Text("Cấp quyền Accessibility")
                .font(.title2.bold())

            Text("Bật \(AppMetadata.name) trong System Settings để gõ tiếng Việt.")
                .foregroundStyle(.secondary)
                .multilineTextAlignment(.center)

            VStack(alignment: .leading, spacing: 8) {
                Label("Mở Privacy & Security → Accessibility", systemImage: "1.circle.fill")
                Label("Bật công tắc bên cạnh \(AppMetadata.name)", systemImage: "2.circle.fill")
            }
            .font(.callout)
            .foregroundStyle(.secondary)
            .padding(.top, 4)
        }
    }
}

private struct ReadyStep: View {
    var body: some View {
        StepLayout {
            Image(systemName: "checkmark.shield.fill")
                .font(.system(size: 40))
                .foregroundStyle(.green)

            Text("Đã cấp quyền")
                .font(.title2.bold())

            Text("Nhấn \"Khởi động lại\" để áp dụng.")
                .foregroundStyle(.secondary)
        }
    }
}

private struct SuccessStep: View {
    var body: some View {
        StepLayout {
            Image(systemName: "checkmark.circle.fill")
                .font(.system(size: 48))
                .foregroundStyle(.green)

            Text("Sẵn sàng hoạt động")
                .font(.title2.bold())

            Text("\(AppMetadata.name) đã được cấp quyền thành công.")
                .foregroundStyle(.secondary)
        }
    }
}

private struct SetupStep: View {
    @Binding var selectedMode: InputMode

    var body: some View {
        StepLayout {
            Image(systemName: "keyboard")
                .font(.system(size: 40))
                .foregroundStyle(.blue)

            Text("Chọn kiểu gõ")
                .font(.title2.bold())

            Text("Có thể thay đổi sau trong menu.")
                .foregroundStyle(.secondary)

            VStack(spacing: 8) {
                ForEach(InputMode.allCases, id: \.rawValue) { mode in
                    ModeOption(mode: mode, isSelected: selectedMode == mode) {
                        selectedMode = mode
                    }
                }
            }
            .frame(maxWidth: 260)
            .padding(.top, 8)
        }
    }
}

private struct ModeOption: View {
    let mode: InputMode
    let isSelected: Bool
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            HStack(spacing: 12) {
                VStack(alignment: .leading, spacing: 2) {
                    Text(mode.name)
                        .font(.headline)
                    Text(mode.description)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
                Spacer()
                Image(systemName: isSelected ? "checkmark.circle.fill" : "circle")
                    .font(.system(size: 20))
                    .foregroundStyle(isSelected ? Color.accentColor : Color.secondary.opacity(0.4))
            }
            .padding(.horizontal, 14)
            .padding(.vertical, 10)
            .background(
                RoundedRectangle(cornerRadius: 8)
                    .fill(isSelected ? Color.accentColor.opacity(0.1) : Color.secondary.opacity(0.05))
            )
            .overlay(
                RoundedRectangle(cornerRadius: 8)
                    .stroke(isSelected ? Color.accentColor.opacity(0.5) : .clear, lineWidth: 1)
            )
        }
        .buttonStyle(.plain)
    }
}

// MARK: - Components

private struct StepLayout<Content: View>: View {
    @ViewBuilder let content: Content

    var body: some View {
        VStack(spacing: 12) {
            Spacer()
            content
            Spacer()
        }
        .padding(.horizontal, 40)
    }
}

// MARK: - Notification

extension Notification.Name {
    static let onboardingCompleted = Notification.Name("onboardingCompleted")
}
