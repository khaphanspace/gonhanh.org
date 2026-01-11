// OnboardingWindowController.swift - Pure AppKit Onboarding UI
// Target: <10MB RAM

import Cocoa

class OnboardingWindowController {
    static func createWindow() -> NSWindow {
        let window = NSWindow(
            contentRect: NSRect(x: 0, y: 0, width: 500, height: 450),
            styleMask: [.titled, .closable],
            backing: .buffered,
            defer: false
        )
        window.title = "Gõ Nhanh"
        window.center()
        window.isReleasedWhenClosed = false

        let contentView = OnboardingView(frame: window.contentView!.bounds)
        contentView.autoresizingMask = [.width, .height]
        window.contentView = contentView

        return window
    }
}

class OnboardingView: NSView {
    private var continueButton: NSButton!
    private var statusLabel: NSTextField!

    override init(frame: NSRect) {
        super.init(frame: frame)
        setupUI()
    }

    required init?(coder: NSCoder) {
        super.init(coder: coder)
        setupUI()
    }

    private func setupUI() {
        // App icon
        let iconView = NSImageView(frame: NSRect(x: 210, y: 340, width: 80, height: 80))
        iconView.image = NSImage(named: "AppIcon")
        iconView.imageScaling = .scaleProportionallyUpOrDown
        addSubview(iconView)

        // Title
        let titleLabel = NSTextField(labelWithString: "Chào mừng đến với Gõ Nhanh")
        titleLabel.font = .systemFont(ofSize: 22, weight: .bold)
        titleLabel.alignment = .center
        titleLabel.frame = NSRect(x: 0, y: 290, width: 500, height: 30)
        addSubview(titleLabel)

        // Subtitle
        let subtitleLabel = NSTextField(labelWithString: "Bộ gõ tiếng Việt nhanh, nhẹ, chính xác")
        subtitleLabel.font = .systemFont(ofSize: 14)
        subtitleLabel.textColor = .secondaryLabelColor
        subtitleLabel.alignment = .center
        subtitleLabel.frame = NSRect(x: 0, y: 260, width: 500, height: 20)
        addSubview(subtitleLabel)

        // Instructions
        let instructions = """
        Để Gõ Nhanh hoạt động, bạn cần cấp quyền Accessibility.

        1. Nhấn "Tiếp tục" để mở System Settings
        2. Bật quyền cho Gõ Nhanh trong danh sách
        3. Quay lại đây để hoàn tất thiết lập
        """
        let instructionsLabel = NSTextField(wrappingLabelWithString: instructions)
        instructionsLabel.font = .systemFont(ofSize: 13)
        instructionsLabel.alignment = .center
        instructionsLabel.frame = NSRect(x: 40, y: 130, width: 420, height: 100)
        addSubview(instructionsLabel)

        // Status label
        statusLabel = NSTextField(labelWithString: "")
        statusLabel.font = .systemFont(ofSize: 12)
        statusLabel.textColor = .systemGreen
        statusLabel.alignment = .center
        statusLabel.frame = NSRect(x: 0, y: 90, width: 500, height: 20)
        addSubview(statusLabel)

        // Continue button
        continueButton = NSButton(title: "Tiếp tục", target: self, action: #selector(continueAction))
        continueButton.bezelStyle = .rounded
        continueButton.frame = NSRect(x: 190, y: 40, width: 120, height: 32)
        continueButton.keyEquivalent = "\r"
        addSubview(continueButton)

        // Check if already trusted
        updateAccessibilityStatus()

        // Start polling for accessibility permission
        Timer.scheduledTimer(withTimeInterval: 1.0, repeats: true) { [weak self] timer in
            if AXIsProcessTrusted() {
                timer.invalidate()
                self?.handleAccessibilityGranted()
            }
        }
    }

    private func updateAccessibilityStatus() {
        if AXIsProcessTrusted() {
            statusLabel.stringValue = "✓ Đã được cấp quyền Accessibility"
            statusLabel.textColor = .systemGreen
            continueButton.title = "Hoàn tất"
        } else {
            statusLabel.stringValue = "Đang chờ cấp quyền..."
            statusLabel.textColor = .secondaryLabelColor
            continueButton.title = "Cấp quyền"
        }
    }

    @objc private func continueAction() {
        if AXIsProcessTrusted() {
            completeOnboarding()
        } else {
            // Open Accessibility settings
            let opts = [kAXTrustedCheckOptionPrompt.takeUnretainedValue() as String: true] as CFDictionary
            AXIsProcessTrustedWithOptions(opts)
        }
    }

    private func handleAccessibilityGranted() {
        DispatchQueue.main.async {
            self.updateAccessibilityStatus()
        }
    }

    private func completeOnboarding() {
        UserDefaults.standard.set(true, forKey: SettingsKey.hasCompletedOnboarding)
        NotificationCenter.default.post(name: .onboardingCompleted, object: nil)
        window?.close()
    }
}
