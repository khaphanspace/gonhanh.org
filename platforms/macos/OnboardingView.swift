import SwiftUI
import AppKit

// MARK: - Onboarding View

struct OnboardingView: View {
    @State private var currentStep: OnboardingStep = .welcome
    @State private var permissionStatus: PermissionStatus = .unknown
    @State private var isCheckingPermission = false

    enum OnboardingStep {
        case welcome
        case permission
        case setup
        case done
    }

    enum PermissionStatus {
        case unknown
        case notGranted
        case granted
        case needsRestart
    }

    var body: some View {
        VStack(spacing: 0) {
            // Progress indicator
            ProgressIndicator(currentStep: currentStep)
                .padding(.top, 20)
                .padding(.bottom, 10)

            Divider()

            // Content
            Group {
                switch currentStep {
                case .welcome:
                    WelcomeStep(onNext: { currentStep = .permission })
                case .permission:
                    PermissionStep(
                        status: $permissionStatus,
                        isChecking: $isCheckingPermission,
                        onNext: { currentStep = .setup },
                        onRestart: restartApp
                    )
                case .setup:
                    SetupStep(onNext: { currentStep = .done })
                case .done:
                    DoneStep(onFinish: finishOnboarding)
                }
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)
        }
        .frame(width: 500, height: 400)
        .onAppear {
            checkPermission()
        }
    }

    private func checkPermission() {
        isCheckingPermission = true
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.3) {
            let trusted = AXIsProcessTrusted()
            if trusted {
                permissionStatus = .granted
            } else {
                permissionStatus = .notGranted
            }
            isCheckingPermission = false
        }
    }

    private func restartApp() {
        // Mark that we need to continue onboarding after restart
        UserDefaults.standard.set(false, forKey: SettingsKey.hasCompletedOnboarding)

        // Get the app's path
        let task = Process()
        task.launchPath = "/bin/sh"
        task.arguments = ["-c", "sleep 1 && open \"\(Bundle.main.bundlePath)\""]
        try? task.run()

        // Quit current instance
        NSApp.terminate(nil)
    }

    private func finishOnboarding() {
        UserDefaults.standard.set(true, forKey: SettingsKey.hasCompletedOnboarding)
        NSApp.keyWindow?.close()

        // Notify to start keyboard hook
        NotificationCenter.default.post(name: .onboardingCompleted, object: nil)
    }
}

// MARK: - Progress Indicator

struct ProgressIndicator: View {
    let currentStep: OnboardingView.OnboardingStep

    private var stepIndex: Int {
        switch currentStep {
        case .welcome: return 0
        case .permission: return 1
        case .setup: return 2
        case .done: return 3
        }
    }

    var body: some View {
        HStack(spacing: 8) {
            ForEach(0..<4) { index in
                Circle()
                    .fill(index <= stepIndex ? Color.accentColor : Color.gray.opacity(0.3))
                    .frame(width: 8, height: 8)
            }
        }
    }
}

// MARK: - Welcome Step

struct WelcomeStep: View {
    let onNext: () -> Void

    var body: some View {
        VStack(spacing: 24) {
            Spacer()

            // Icon
            Image(systemName: "keyboard.fill")
                .font(.system(size: 60))
                .foregroundColor(.accentColor)

            // Title
            Text("Chào mừng đến với \(AppMetadata.name)")
                .font(.system(size: 28, weight: .bold))

            // Description
            Text(AppMetadata.tagline)
                .font(.title3)
                .foregroundColor(.secondary)

            Text("Gõ tiếng Việt nhanh, chính xác với Telex hoặc VNI")
                .font(.body)
                .foregroundColor(.secondary)
                .multilineTextAlignment(.center)

            Spacer()

            // Next button
            Button(action: onNext) {
                HStack {
                    Text("Bắt đầu")
                    Image(systemName: "arrow.right")
                }
                .frame(width: 150)
            }
            .buttonStyle(.borderedProminent)
            .controlSize(.large)

            Spacer().frame(height: 30)
        }
        .padding(30)
    }
}

// MARK: - Permission Step

struct PermissionStep: View {
    @Binding var status: OnboardingView.PermissionStatus
    @Binding var isChecking: Bool
    let onNext: () -> Void
    let onRestart: () -> Void

    @State private var hasRequestedPermission = false
    @State private var showRestartPrompt = false

    var body: some View {
        VStack(spacing: 20) {
            Spacer()

            // Icon based on status
            Group {
                switch status {
                case .granted:
                    Image(systemName: "checkmark.shield.fill")
                        .foregroundColor(.green)
                case .needsRestart:
                    Image(systemName: "arrow.clockwise.circle.fill")
                        .foregroundColor(.orange)
                default:
                    Image(systemName: "lock.shield.fill")
                        .foregroundColor(.accentColor)
                }
            }
            .font(.system(size: 50))

            // Title
            Text(titleText)
                .font(.system(size: 24, weight: .bold))

            // Description
            Text(descriptionText)
                .font(.body)
                .foregroundColor(.secondary)
                .multilineTextAlignment(.center)
                .frame(maxWidth: 400)

            // Steps (when not granted)
            if status == .notGranted || status == .needsRestart {
                VStack(alignment: .leading, spacing: 12) {
                    StepRow(number: 1, text: "Nhấn \"Mở Cài đặt\" bên dưới", isCompleted: hasRequestedPermission)
                    StepRow(number: 2, text: "Bật GoNhanh trong danh sách", isCompleted: false)
                    StepRow(number: 3, text: "Quay lại và nhấn \"Khởi động lại\"", isCompleted: false)
                }
                .padding(.vertical, 10)
            }

            Spacer()

            // Actions
            HStack(spacing: 16) {
                if status == .granted {
                    Button(action: onNext) {
                        HStack {
                            Text("Tiếp tục")
                            Image(systemName: "arrow.right")
                        }
                        .frame(width: 150)
                    }
                    .buttonStyle(.borderedProminent)
                    .controlSize(.large)
                } else if status == .needsRestart || showRestartPrompt {
                    Button("Mở Cài đặt") {
                        openSystemSettings()
                    }
                    .controlSize(.large)

                    Button(action: onRestart) {
                        HStack {
                            Image(systemName: "arrow.clockwise")
                            Text("Khởi động lại")
                        }
                        .frame(width: 160)
                    }
                    .buttonStyle(.borderedProminent)
                    .controlSize(.large)
                } else {
                    Button(action: requestPermission) {
                        HStack {
                            Image(systemName: "gear")
                            Text("Mở Cài đặt")
                        }
                        .frame(width: 150)
                    }
                    .buttonStyle(.borderedProminent)
                    .controlSize(.large)
                    .disabled(isChecking)
                }
            }

            // Skip hint for granted
            if status != .granted {
                Button("Kiểm tra lại") {
                    recheckPermission()
                }
                .buttonStyle(.plain)
                .foregroundColor(.accentColor)
                .padding(.top, 8)
            }

            Spacer().frame(height: 30)
        }
        .padding(30)
    }

    private var titleText: String {
        switch status {
        case .granted:
            return "Đã cấp quyền!"
        case .needsRestart:
            return "Cần khởi động lại"
        default:
            return "Cấp quyền Input Monitoring"
        }
    }

    private var descriptionText: String {
        switch status {
        case .granted:
            return "GoNhanh đã có quyền cần thiết để hoạt động."
        case .needsRestart:
            return "Bạn đã cấp quyền! Khởi động lại app để áp dụng."
        default:
            return "GoNhanh cần quyền Input Monitoring để nhận phím bạn gõ và chuyển đổi thành tiếng Việt."
        }
    }

    private func requestPermission() {
        hasRequestedPermission = true
        openSystemSettings()

        // Show restart prompt after a delay
        DispatchQueue.main.asyncAfter(deadline: .now() + 2) {
            showRestartPrompt = true
        }
    }

    private func openSystemSettings() {
        if let url = URL(string: "x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent") {
            NSWorkspace.shared.open(url)
        }
    }

    private func recheckPermission() {
        isChecking = true
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.5) {
            let trusted = AXIsProcessTrusted()
            if trusted {
                status = .granted
            } else if hasRequestedPermission {
                status = .needsRestart
            }
            isChecking = false
        }
    }
}

struct StepRow: View {
    let number: Int
    let text: String
    let isCompleted: Bool

    var body: some View {
        HStack(spacing: 12) {
            ZStack {
                Circle()
                    .fill(isCompleted ? Color.green : Color.accentColor.opacity(0.2))
                    .frame(width: 24, height: 24)

                if isCompleted {
                    Image(systemName: "checkmark")
                        .font(.system(size: 12, weight: .bold))
                        .foregroundColor(.white)
                } else {
                    Text("\(number)")
                        .font(.system(size: 12, weight: .semibold))
                        .foregroundColor(.accentColor)
                }
            }

            Text(text)
                .font(.body)
                .foregroundColor(isCompleted ? .secondary : .primary)
        }
    }
}

// MARK: - Setup Step

struct SetupStep: View {
    let onNext: () -> Void

    @State private var selectedMode: InputMode = .telex

    var body: some View {
        VStack(spacing: 24) {
            Spacer()

            // Icon
            Image(systemName: "textformat.alt")
                .font(.system(size: 50))
                .foregroundColor(.accentColor)

            // Title
            Text("Chọn kiểu gõ")
                .font(.system(size: 24, weight: .bold))

            Text("Bạn có thể thay đổi sau trong menu")
                .font(.body)
                .foregroundColor(.secondary)

            // Mode selection
            VStack(spacing: 12) {
                ForEach(InputMode.allCases, id: \.rawValue) { mode in
                    ModeSelectionCard(
                        mode: mode,
                        isSelected: selectedMode == mode,
                        onSelect: { selectedMode = mode }
                    )
                }
            }
            .frame(maxWidth: 350)

            Spacer()

            // Next button
            Button(action: {
                // Save selected mode
                UserDefaults.standard.set(selectedMode.rawValue, forKey: SettingsKey.method)
                RustBridge.setMethod(selectedMode.rawValue)
                onNext()
            }) {
                HStack {
                    Text("Tiếp tục")
                    Image(systemName: "arrow.right")
                }
                .frame(width: 150)
            }
            .buttonStyle(.borderedProminent)
            .controlSize(.large)

            Spacer().frame(height: 30)
        }
        .padding(30)
    }
}

struct ModeSelectionCard: View {
    let mode: InputMode
    let isSelected: Bool
    let onSelect: () -> Void

    var body: some View {
        Button(action: onSelect) {
            HStack {
                VStack(alignment: .leading, spacing: 4) {
                    Text(mode.name)
                        .font(.headline)
                        .foregroundColor(.primary)

                    Text(mode.description)
                        .font(.subheadline)
                        .foregroundColor(.secondary)
                }

                Spacer()

                if isSelected {
                    Image(systemName: "checkmark.circle.fill")
                        .font(.title2)
                        .foregroundColor(.accentColor)
                } else {
                    Image(systemName: "circle")
                        .font(.title2)
                        .foregroundColor(.gray.opacity(0.5))
                }
            }
            .padding(16)
            .background(
                RoundedRectangle(cornerRadius: 10)
                    .fill(isSelected ? Color.accentColor.opacity(0.1) : Color.gray.opacity(0.1))
            )
            .overlay(
                RoundedRectangle(cornerRadius: 10)
                    .stroke(isSelected ? Color.accentColor : Color.clear, lineWidth: 2)
            )
        }
        .buttonStyle(.plain)
    }
}

// MARK: - Done Step

struct DoneStep: View {
    let onFinish: () -> Void

    var body: some View {
        VStack(spacing: 24) {
            Spacer()

            // Icon
            Image(systemName: "checkmark.circle.fill")
                .font(.system(size: 60))
                .foregroundColor(.green)

            // Title
            Text("Sẵn sàng!")
                .font(.system(size: 28, weight: .bold))

            // Description
            VStack(spacing: 8) {
                Text("GoNhanh đã được cài đặt thành công")
                    .font(.title3)

                Text("Bạn có thể bắt đầu gõ tiếng Việt ngay bây giờ")
                    .font(.body)
                    .foregroundColor(.secondary)
            }

            // Tips
            VStack(alignment: .leading, spacing: 12) {
                TipRow(icon: "menubar.rectangle", text: "Click icon trên menu bar để bật/tắt")
                TipRow(icon: "keyboard", text: "Gõ như bình thường, dấu sẽ tự động được thêm")
            }
            .padding(.vertical, 20)
            .padding(.horizontal, 30)
            .background(Color.gray.opacity(0.1))
            .cornerRadius(12)

            Spacer()

            // Finish button
            Button(action: onFinish) {
                Text("Hoàn tất")
                    .frame(width: 150)
            }
            .buttonStyle(.borderedProminent)
            .controlSize(.large)

            Spacer().frame(height: 30)
        }
        .padding(30)
    }
}

struct TipRow: View {
    let icon: String
    let text: String

    var body: some View {
        HStack(spacing: 12) {
            Image(systemName: icon)
                .font(.title3)
                .foregroundColor(.accentColor)
                .frame(width: 24)

            Text(text)
                .font(.body)
        }
    }
}

// MARK: - Notification

extension Notification.Name {
    static let onboardingCompleted = Notification.Name("onboardingCompleted")
}

// MARK: - Preview

struct OnboardingView_Previews: PreviewProvider {
    static var previews: some View {
        OnboardingView()
    }
}
