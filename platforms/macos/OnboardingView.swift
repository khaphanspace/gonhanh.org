import SwiftUI

// MARK: - Onboarding View

struct OnboardingView: View {
    @State private var step = 0
    @State private var hasPermission = false
    @State private var selectedMode: InputMode = .telex

    private let timer = Timer.publish(every: 1, on: .main, in: .common).autoconnect()
    private var stepIndex: Int { step >= 10 ? step - 10 : step }

    var body: some View {
        VStack(spacing: 0) {
            content.frame(height: 340)
            Divider()
            footer
        }
        .frame(width: 480)
        .onAppear {
            hasPermission = AXIsProcessTrusted()
            if UserDefaults.standard.bool(forKey: SettingsKey.permissionGranted) && hasPermission {
                step = 10
            }
        }
        .onReceive(timer) { _ in
            hasPermission = AXIsProcessTrusted()
        }
    }

    // MARK: - Content

    @ViewBuilder
    private var content: some View {
        switch step {
        case 0: WelcomeStep()
        case 1: PermissionStep(hasPermission: hasPermission)
        case 10: SuccessStep()
        case 11: SetupStep(selectedMode: $selectedMode)
        default: EmptyView()
        }
    }

    // MARK: - Footer

    private var footer: some View {
        HStack {
            HStack(spacing: 6) {
                ForEach(0..<2, id: \.self) { i in
                    Circle()
                        .fill(i == stepIndex ? Color.accentColor : Color.secondary.opacity(0.3))
                        .frame(width: 6, height: 6)
                }
            }
            Spacer()
            HStack(spacing: 12) {
                if step == 1 && !hasPermission {
                    Button("Quay lại") { step = 0 }
                }
                primaryButton
            }
        }
        .padding(.horizontal, 20)
        .padding(.vertical, 14)
    }

    @ViewBuilder
    private var primaryButton: some View {
        switch step {
        case 0:
            Button("Tiếp tục") { step = 1 }.buttonStyle(.borderedProminent)
        case 1:
            if hasPermission {
                Button("Khởi động lại") { restart() }.buttonStyle(.borderedProminent)
            } else {
                Button("Mở Cài đặt") { openAccessibilitySettings() }.buttonStyle(.borderedProminent)
            }
        case 10:
            Button("Tiếp tục") { step = 11 }.buttonStyle(.borderedProminent)
        case 11:
            Button("Hoàn tất") { finish() }.buttonStyle(.borderedProminent)
        default:
            EmptyView()
        }
    }

    // MARK: - Actions

    private func openAccessibilitySettings() {
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
        VStack(spacing: 16) {
            Spacer()
            Image(nsImage: AppMetadata.logo)
                .resizable()
                .frame(width: 80, height: 80)
            Text("Chào mừng đến với \(AppMetadata.name)")
                .font(.system(size: 22, weight: .bold))
            Text(AppMetadata.tagline)
                .foregroundStyle(.secondary)
            Spacer()
        }
        .padding(.horizontal, 40)
    }
}

private struct PermissionStep: View {
    let hasPermission: Bool

    var body: some View {
        VStack(spacing: 16) {
            Spacer()
            Image(systemName: hasPermission ? "checkmark.shield.fill" : "hand.raised.fill")
                .font(.system(size: 40))
                .foregroundStyle(hasPermission ? .green : .orange)
            Text(hasPermission ? "Đã cấp quyền" : "Cấp quyền Accessibility")
                .font(.system(size: 22, weight: .bold))
            if hasPermission {
                Text("Nhấn \"Khởi động lại\" để áp dụng.")
                    .foregroundStyle(.secondary)
            } else {
                Text("Bật \(AppMetadata.name) trong System Settings để gõ tiếng Việt.")
                    .foregroundStyle(.secondary)
                    .multilineTextAlignment(.center)
                VStack(alignment: .leading, spacing: 8) {
                    Label("Mở Privacy & Security → Accessibility", systemImage: "1.circle.fill")
                    Label("Bật công tắc bên cạnh \(AppMetadata.name)", systemImage: "2.circle.fill")
                }
                .font(.callout)
                .foregroundStyle(.secondary)
                .padding(.top, 8)
            }
            Spacer()
        }
        .padding(.horizontal, 40)
    }
}

private struct SuccessStep: View {
    var body: some View {
        VStack(spacing: 16) {
            Spacer()
            Image(systemName: "checkmark.circle.fill")
                .font(.system(size: 48))
                .foregroundStyle(.green)
            Text("Sẵn sàng hoạt động")
                .font(.system(size: 22, weight: .bold))
            Text("\(AppMetadata.name) đã được cấp quyền thành công.")
                .foregroundStyle(.secondary)
            Spacer()
        }
        .padding(.horizontal, 40)
    }
}

private struct SetupStep: View {
    @Binding var selectedMode: InputMode

    var body: some View {
        VStack(spacing: 16) {
            Spacer()
            Image(systemName: "keyboard")
                .font(.system(size: 40))
                .foregroundStyle(.blue)
            Text("Chọn kiểu gõ")
                .font(.system(size: 22, weight: .bold))
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
            Spacer()
        }
        .padding(.horizontal, 40)
    }
}

private struct ModeOption: View {
    let mode: InputMode
    let isSelected: Bool
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            HStack {
                VStack(alignment: .leading, spacing: 2) {
                    Text(mode.name).font(.headline)
                    Text(mode.description).font(.caption).foregroundStyle(.secondary)
                }
                Spacer()
                Image(systemName: isSelected ? "checkmark.circle.fill" : "circle")
                    .foregroundStyle(isSelected ? .blue : .secondary.opacity(0.4))
            }
            .padding(10)
            .background(RoundedRectangle(cornerRadius: 8).fill(isSelected ? Color.blue.opacity(0.1) : Color.secondary.opacity(0.05)))
            .overlay(RoundedRectangle(cornerRadius: 8).stroke(isSelected ? Color.blue.opacity(0.5) : .clear))
        }
        .buttonStyle(.plain)
    }
}

// MARK: - Notification

extension Notification.Name {
    static let onboardingCompleted = Notification.Name("onboardingCompleted")
}
