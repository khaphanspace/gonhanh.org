import SwiftUI
import AppKit

// MARK: - Onboarding View

struct OnboardingView: View {
    @State private var currentStep: OnboardingStep = .welcome
    @State private var hasPermission = false
    @State private var selectedMode: InputMode = .telex
    @State private var permissionTimer: Timer?

    enum OnboardingStep {
        case welcome
        case permission
        case setup
    }

    var body: some View {
        Group {
            switch currentStep {
            case .welcome:
                WelcomeStepView(onNext: goToPermission)
            case .permission:
                PermissionStepView(
                    hasPermission: hasPermission,
                    onRestart: restartApp
                )
            case .setup:
                SetupStepView(selectedMode: $selectedMode, onFinish: finish)
            }
        }
        .frame(width: 520, height: 480)
        .onAppear {
            startPermissionCheck()
        }
        .onDisappear {
            stopPermissionCheck()
        }
    }

    // MARK: - Navigation

    private func goToPermission() {
        if hasPermission {
            currentStep = .setup
        } else {
            currentStep = .permission
        }
    }

    private func finish() {
        UserDefaults.standard.set(selectedMode.rawValue, forKey: SettingsKey.method)
        UserDefaults.standard.set(true, forKey: SettingsKey.hasCompletedOnboarding)
        NotificationCenter.default.post(name: .onboardingCompleted, object: nil)
        NSApp.keyWindow?.close()
    }

    private func restartApp() {
        // Save selected mode before restart (default Telex)
        UserDefaults.standard.set(selectedMode.rawValue, forKey: SettingsKey.method)

        let appPath = Bundle.main.bundlePath
        let task = Process()
        task.launchPath = "/bin/sh"
        task.arguments = ["-c", "sleep 0.5 && open \"\(appPath)\""]
        try? task.run()
        NSApp.terminate(nil)
    }

    // MARK: - Permission Check

    private func startPermissionCheck() {
        checkPermission()
        permissionTimer = Timer.scheduledTimer(withTimeInterval: 1.0, repeats: true) { _ in
            checkPermission()
        }
    }

    private func stopPermissionCheck() {
        permissionTimer?.invalidate()
        permissionTimer = nil
    }

    private func checkPermission() {
        hasPermission = AXIsProcessTrusted()
    }
}

// MARK: - Welcome Step

struct WelcomeStepView: View {
    let onNext: () -> Void

    var body: some View {
        VStack(spacing: 24) {
            Spacer()

            Image(systemName: "keyboard.fill")
                .font(.system(size: 60))
                .foregroundColor(.accentColor)

            Text("Chào mừng đến với \(AppMetadata.name)")
                .font(.system(size: 26, weight: .bold))

            Text(AppMetadata.tagline)
                .font(.body)
                .foregroundColor(.secondary)

            Spacer()

            Button(action: onNext) {
                HStack(spacing: 6) {
                    Text("Bắt đầu")
                    Image(systemName: "arrow.right")
                }
                .frame(width: 140)
            }
            .buttonStyle(.borderedProminent)
            .controlSize(.large)

            Spacer().frame(height: 30)
        }
        .padding(30)
    }
}

// MARK: - Permission Step

struct PermissionStepView: View {
    let hasPermission: Bool
    let onRestart: () -> Void

    @State private var didOpenSettings = false

    var body: some View {
        VStack(spacing: 20) {
            Spacer()

            Image(systemName: hasPermission ? "checkmark.shield.fill" : "lock.shield.fill")
                .font(.system(size: 50))
                .foregroundColor(hasPermission ? .green : .accentColor)

            Text(hasPermission ? "Đã cấp quyền!" : "Cấp quyền Accessibility")
                .font(.system(size: 24, weight: .bold))

            Text(hasPermission
                 ? "Nhấn nút bên dưới để khởi động lại và bắt đầu sử dụng."
                 : "GoNhanh cần quyền Accessibility để gõ tiếng Việt.")
                .font(.body)
                .foregroundColor(.secondary)
                .multilineTextAlignment(.center)
                .frame(maxWidth: 380)

            if !hasPermission {
                VStack(alignment: .leading, spacing: 10) {
                    StepRowView(number: 1, text: "Nhấn \"Mở Cài đặt\"", done: didOpenSettings)
                    StepRowView(number: 2, text: "Bật GoNhanh trong danh sách", done: hasPermission)
                    StepRowView(number: 3, text: "Nhấn \"Khởi động lại\"", done: false)
                }
                .padding(.top, 8)
            }

            Spacer()

            if hasPermission {
                Button(action: onRestart) {
                    Label("Khởi động lại", systemImage: "arrow.clockwise")
                        .frame(width: 160)
                }
                .buttonStyle(.borderedProminent)
                .controlSize(.large)
            } else {
                Button(action: openSettings) {
                    Label("Mở Cài đặt", systemImage: "gear")
                        .frame(width: 140)
                }
                .buttonStyle(.borderedProminent)
                .controlSize(.large)
            }

            Spacer().frame(height: 30)
        }
        .padding(30)
    }

    private func openSettings() {
        didOpenSettings = true
        if let url = URL(string: "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility") {
            NSWorkspace.shared.open(url)
        }
    }
}

struct StepRowView: View {
    let number: Int
    let text: String
    let done: Bool

    var body: some View {
        HStack(spacing: 12) {
            ZStack {
                Circle()
                    .fill(done ? Color.green : Color.accentColor.opacity(0.2))
                    .frame(width: 24, height: 24)

                if done {
                    Image(systemName: "checkmark")
                        .font(.system(size: 11, weight: .bold))
                        .foregroundColor(.white)
                } else {
                    Text("\(number)")
                        .font(.system(size: 12, weight: .semibold))
                        .foregroundColor(.accentColor)
                }
            }

            Text(text)
                .font(.body)
                .foregroundColor(done ? .secondary : .primary)
        }
    }
}

// MARK: - Setup Step

struct SetupStepView: View {
    @Binding var selectedMode: InputMode
    let onFinish: () -> Void

    var body: some View {
        VStack(spacing: 20) {
            Spacer()

            Text("Aa")
                .font(.system(size: 56, weight: .light, design: .rounded))
                .foregroundColor(.accentColor)

            Text("Chọn kiểu gõ")
                .font(.system(size: 24, weight: .bold))

            Text("Bạn có thể thay đổi sau trong menu")
                .font(.body)
                .foregroundColor(.secondary)

            VStack(spacing: 10) {
                ForEach(InputMode.allCases, id: \.rawValue) { mode in
                    ModeCard(mode: mode, isSelected: selectedMode == mode) {
                        selectedMode = mode
                    }
                }
            }
            .frame(maxWidth: 340)
            .padding(.top, 8)

            Spacer()

            Button(action: onFinish) {
                Text("Hoàn tất")
                    .frame(width: 140)
            }
            .buttonStyle(.borderedProminent)
            .controlSize(.large)

            Spacer().frame(height: 30)
        }
        .padding(30)
    }
}

struct ModeCard: View {
    let mode: InputMode
    let isSelected: Bool
    let onTap: () -> Void

    var body: some View {
        Button(action: onTap) {
            HStack {
                VStack(alignment: .leading, spacing: 3) {
                    Text(mode.name)
                        .font(.headline)
                        .foregroundColor(.primary)
                    Text(mode.description)
                        .font(.subheadline)
                        .foregroundColor(.secondary)
                }
                Spacer()
                Image(systemName: isSelected ? "checkmark.circle.fill" : "circle")
                    .font(.title2)
                    .foregroundColor(isSelected ? .accentColor : .gray.opacity(0.4))
            }
            .padding(14)
            .background(
                RoundedRectangle(cornerRadius: 10)
                    .fill(isSelected ? Color.accentColor.opacity(0.1) : Color.gray.opacity(0.08))
            )
            .overlay(
                RoundedRectangle(cornerRadius: 10)
                    .stroke(isSelected ? Color.accentColor : Color.clear, lineWidth: 2)
            )
        }
        .buttonStyle(.plain)
    }
}

// MARK: - Notification

extension Notification.Name {
    static let onboardingCompleted = Notification.Name("onboardingCompleted")
}
