import Foundation
import AppKit

// MARK: - App Metadata (Centralized)
// All project metadata in one place for consistency

enum AppMetadata {
    static let name = "Gõ Nhanh"

    // App Logo - dùng chung cho mọi nơi
    static var logo: NSImage {
        NSImage(named: "AppLogo") ?? NSApp.applicationIconImage ?? NSImage()
    }
    static let displayName = "Gõ Nhanh"
    static let tagline = "Bộ gõ tiếng Việt hiệu suất cao"
    static let version = Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "1.0.0"
    static let buildNumber = Bundle.main.infoDictionary?["CFBundleVersion"] as? String ?? "1"

    // Author
    static let author = "Kha Phan"
    static let authorEmail = "nhatkha1407@gmail.com"

    // Links
    static let website = "https://gonhanh.org"
    static let repository = "https://github.com/khaphanspace/gonhanh.org"
    static let issuesURL = "https://github.com/khaphanspace/gonhanh.org/issues"
    static let authorLinkedin = "https://www.linkedin.com/in/khaphanspace"

    // Legal
    static let copyright = "Copyright © 2025 \(author). All rights reserved."
    static let license = "GPL-3.0-or-later"

    // Tech
    static let techStack = "Rust + SwiftUI"

    // Credits for About panel
    static var credits: String {
        """
        \(tagline)

        Tác giả: \(author)

        Made with Rust + SwiftUI
        """
    }

    // Full about text
    static var aboutText: String {
        """
        \(displayName) v\(version)

        \(tagline)

        Tác giả: \(author)
        Website: \(website)
        GitHub: \(repository)

        \(copyright)
        License: \(license)
        """
    }
}

// MARK: - Settings Keys (Shared)

enum SettingsKey {
    static let enabled = "gonhanh.enabled"
    static let method = "gonhanh.method"
    static let hasCompletedOnboarding = "gonhanh.onboarding.completed"
    static let permissionGranted = "gonhanh.permission.granted"
}

// MARK: - Input Mode

enum InputMode: Int, CaseIterable {
    case telex = 0
    case vni = 1

    var name: String {
        switch self {
        case .telex: return "Telex"
        case .vni: return "VNI"
        }
    }

    var shortName: String {
        switch self {
        case .telex: return "T"
        case .vni: return "V"
        }
    }

    var description: String {
        switch self {
        case .telex: return "aw, ow, w, s, f, r, x, j"
        case .vni: return "a8, o9, 1-5"
        }
    }

    var fullDescription: String {
        switch self {
        case .telex: return "Telex (\(description))"
        case .vni: return "VNI (\(description))"
        }
    }
}
