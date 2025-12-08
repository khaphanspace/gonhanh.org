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
            content.frame(height: 340)
            Divider()
            footer
        }
        .frame(width: 480)
        .background(colorScheme == .dark ? Color.black.opacity(0.2) : Color.white)
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
        case 0:  WelcomeStep(colorScheme: colorScheme)
        case 1:  PermissionStep(colorScheme: colorScheme)
        case 2:  ReadyStep(colorScheme: colorScheme)
        case 10: SuccessStep(colorScheme: colorScheme)
        case 11: SetupStep(colorScheme: colorScheme, selectedMode: $selectedMode)
        default: EmptyView()
        }
    }

    private var footer: some View {
        HStack {
            // Step indicators
            HStack(spacing: 8) {
                ForEach(0..<totalSteps, id: \.self) { i in
                    Circle()
                        .fill(i == stepIndex ? Color.accentColor : (colorScheme == .dark ? Color.white.opacity(0.2) : Color.black.opacity(0.15)))
                        .frame(width: 8, height: 8)
                }
            }
            Spacer()
            if step == 1 {
                Button("Quay lại") { step = 0 }
                    .foregroundStyle(.secondary)
            }
            primaryButton
        }
        .padding(.horizontal, 24)
        .padding(.vertical, 16)
        .background(colorScheme == .dark ? Color.white.opacity(0.02) : Color.black.opacity(0.02))
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
    let colorScheme: ColorScheme

    var body: some View {
        StepLayout {
            Image(nsImage: AppMetadata.logo)
                .resizable()
                .frame(width: 88, height: 88)
                .shadow(color: .black.opacity(0.15), radius: 8, y: 4)

            Text("Chào mừng đến với \(AppMetadata.name)")
                .font(.system(size: 24, weight: .bold, design: .rounded))

            Text(AppMetadata.tagline)
                .font(.body)
                .foregroundStyle(.secondary)
        }
    }
}

private struct PermissionStep: View {
    let colorScheme: ColorScheme

    var body: some View {
        StepLayout {
            iconCircle(icon: "hand.raised.fill", color: .orange, colorScheme: colorScheme)

            Text("Cấp quyền Accessibility")
                .font(.system(size: 24, weight: .bold, design: .rounded))

            Text("Bật \(AppMetadata.name) trong System Settings để gõ tiếng Việt.")
                .font(.body)
                .foregroundStyle(.secondary)
                .multilineTextAlignment(.center)

            VStack(alignment: .leading, spacing: 12) {
                instructionRow(number: "1", text: "Mở Privacy & Security → Accessibility")
                instructionRow(number: "2", text: "Bật công tắc bên cạnh \(AppMetadata.name)")
            }
            .padding(.top, 8)
        }
    }

    private func instructionRow(number: String, text: String) -> some View {
        HStack(spacing: 12) {
            Text(number)
                .font(.caption)
                .fontWeight(.bold)
                .foregroundStyle(.white)
                .frame(width: 22, height: 22)
                .background(Circle().fill(Color.accentColor))

            Text(text)
                .font(.callout)
                .foregroundStyle(.secondary)
        }
    }
}

private struct ReadyStep: View {
    let colorScheme: ColorScheme

    var body: some View {
        StepLayout {
            iconCircle(icon: "checkmark.shield.fill", color: .green, colorScheme: colorScheme)

            Text("Đã cấp quyền")
                .font(.system(size: 24, weight: .bold, design: .rounded))

            Text("Nhấn \"Khởi động lại\" để áp dụng.")
                .font(.body)
                .foregroundStyle(.secondary)
        }
    }
}

private struct SuccessStep: View {
    let colorScheme: ColorScheme

    var body: some View {
        StepLayout {
            iconCircle(icon: "checkmark.circle.fill", color: .green, colorScheme: colorScheme, size: 88)

            Text("Sẵn sàng hoạt động")
                .font(.system(size: 24, weight: .bold, design: .rounded))

            Text("\(AppMetadata.name) đã được cấp quyền thành công.")
                .font(.body)
                .foregroundStyle(.secondary)
        }
    }
}

private struct SetupStep: View {
    let colorScheme: ColorScheme
    @Binding var selectedMode: InputMode

    var body: some View {
        StepLayout {
            iconCircle(icon: "keyboard", color: .blue, colorScheme: colorScheme)

            Text("Chọn kiểu gõ")
                .font(.system(size: 24, weight: .bold, design: .rounded))

            Text("Có thể thay đổi sau trong menu.")
                .font(.body)
                .foregroundStyle(.secondary)

            VStack(spacing: 10) {
                ForEach(InputMode.allCases, id: \.rawValue) { mode in
                    ModeOption(colorScheme: colorScheme, mode: mode, isSelected: selectedMode == mode) {
                        selectedMode = mode
                    }
                }
            }
            .frame(maxWidth: 280)
            .padding(.top, 12)
        }
    }
}

// MARK: - Components

private struct StepLayout<Content: View>: View {
    @ViewBuilder let content: Content

    var body: some View {
        VStack(spacing: 16) {
            Spacer()
            content
            Spacer()
        }
        .padding(.horizontal, 48)
    }
}

private func iconCircle(icon: String, color: Color, colorScheme: ColorScheme, size: CGFloat = 80) -> some View {
    ZStack {
        Circle()
            .fill(color.opacity(colorScheme == .dark ? 0.2 : 0.1))
            .frame(width: size, height: size)
        Image(systemName: icon)
            .font(.system(size: size * 0.45))
            .foregroundStyle(color)
    }
}

private struct ModeOption: View {
    let colorScheme: ColorScheme
    let mode: InputMode
    let isSelected: Bool
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            HStack(spacing: 14) {
                VStack(alignment: .leading, spacing: 4) {
                    Text(mode.name)
                        .font(.headline)
                        .foregroundStyle(.primary)
                    Text(mode.description)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
                Spacer()
                Image(systemName: isSelected ? "checkmark.circle.fill" : "circle")
                    .font(.system(size: 22))
                    .foregroundStyle(isSelected ? .blue : .secondary.opacity(0.4))
            }
            .padding(.horizontal, 16)
            .padding(.vertical, 12)
            .background(
                RoundedRectangle(cornerRadius: 10)
                    .fill(isSelected
                        ? Color.blue.opacity(colorScheme == .dark ? 0.2 : 0.08)
                        : (colorScheme == .dark ? Color.white.opacity(0.05) : Color.black.opacity(0.03)))
            )
            .overlay(
                RoundedRectangle(cornerRadius: 10)
                    .stroke(isSelected ? Color.blue.opacity(0.5) : Color.clear, lineWidth: 1.5)
            )
        }
        .buttonStyle(.plain)
        .onHover { hovering in
            if hovering {
                NSCursor.pointingHand.push()
            } else {
                NSCursor.pop()
            }
        }
    }
}

// MARK: - Notification

extension Notification.Name {
    static let onboardingCompleted = Notification.Name("onboardingCompleted")
}
