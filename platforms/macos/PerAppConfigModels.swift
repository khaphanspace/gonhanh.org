import SwiftUI

// MARK: - Per-App Profile Model

struct PerAppConfig: Codable, Equatable {
    var enabledState: Int = 0 // 0=auto, 1=on, -1=off
    var delayPreset: Int = 0
    var injectionOverride: Int = -1

    /// Create config pre-filled from system-detected defaults for this app
    static func fromDetected(bundleId: String) -> PerAppConfig {
        guard let info = getDetectedDefault(for: bundleId) else { return PerAppConfig() }
        return PerAppConfig(
            delayPreset: DelayPreset.closest(to: info.delays).rawValue,
            injectionOverride: InjectionOverride.from(detectedMethod: info.method).rawValue
        )
    }
}

// MARK: - Delay Preset

enum DelayPreset: Int, CaseIterable {
    case none = 0
    case low = 1
    case medium = 2
    case high = 3
    case veryHigh = 4

    var name: String {
        switch self {
        case .none: "Không"
        case .low: "Thấp"
        case .medium: "Vừa"
        case .high: "Cao"
        case .veryHigh: "Rất cao"
        }
    }

    /// Delays in µs; the same levels per-app detection uses (`Delays`).
    var delays: InjectDelays {
        switch self {
        case .none: Delays.none
        case .low: Delays.low
        case .medium: Delays.medium
        case .high: Delays.high
        case .veryHigh: Delays.veryHigh
        }
    }

    /// Find closest preset matching a delay tuple (by wait µs)
    static func closest(to delays: InjectDelays) -> DelayPreset {
        let wait = delays.1
        return allCases.min(by: {
            abs(Int($0.delays.1) - Int(wait)) < abs(Int($1.delays.1) - Int(wait))
        }) ?? .none
    }

    var color: Color {
        switch self {
        case .none: .blue
        case .low: .green
        case .medium: .orange
        case .high: Color(NSColor.systemRed)
        case .veryHigh: .purple
        }
    }
}

// MARK: - Injection Override (Kiểu inject)

/// User-facing injection method options (subset of internal InjectionMethod)
enum InjectionOverride: Int, CaseIterable {
    case auto = -1
    case fast = 0
    case slow = 1
    case charByChar = 2
    case selection = 3
    case emptyCharPrefix = 4

    var name: String {
        switch self {
        case .auto: "Tự động"
        case .fast: "Fast"
        case .slow: "Slow"
        case .charByChar: "Char-by-char"
        case .selection: "Selection"
        case .emptyCharPrefix: "Empty char"
        }
    }

    var subtitle: String {
        switch self {
        case .auto: "Để hệ thống chọn"
        case .fast: "Mặc định, backspace + text"
        case .slow: "Delay cao hơn cho Electron"
        case .charByChar: "Gõ từng ký tự, Safari/GDocs"
        case .selection: "Select + replace, combo box"
        case .emptyCharPrefix: "Phá autocomplete trình duyệt"
        }
    }

    /// The method this override forces; `nil` leaves the detected one.
    var method: InjectionMethod? {
        switch self {
        case .auto: nil
        case .fast: .fast
        case .slow: .slow
        case .charByChar: .charByChar
        case .selection: .selection
        case .emptyCharPrefix: .emptyCharPrefix
        }
    }

    /// Find override matching a detected method name string
    static func from(detectedMethod: String) -> InjectionOverride {
        allCases.first(where: { $0.internalMethodName == detectedMethod }) ?? .auto
    }

    /// Map to internal InjectionMethod string for RustBridge
    var internalMethodName: String {
        switch self {
        case .auto: "auto"
        case .fast: "fast"
        case .slow: "slow"
        case .charByChar: "charByChar"
        case .selection: "selection"
        case .emptyCharPrefix: "emptyCharPrefix"
        }
    }
}
