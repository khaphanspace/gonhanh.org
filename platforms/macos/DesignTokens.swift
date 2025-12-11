import SwiftUI

// MARK: - Design Tokens
// Based on Easy AI Notebook Human Interface Guidelines v1.0

extension Color {
    // MARK: - Brand Colors - Warm Indigo (Primary/Accent)

    /// Primary actions, links, focus states - #5046e5
    static let accent = Color(hex: 0x5046e5)

    /// Hover states - #4338ca
    static let accentHover = Color(hex: 0x4338ca)

    /// Active/pressed states - #3730a3
    static let accentActive = Color(hex: 0x3730a3)

    /// Backgrounds, selected states - #eef2ff
    static let accentSubtle = Color(hex: 0xeef2ff)

    /// Borders, dividers - #c7d2fe
    static let accentMuted = Color(hex: 0xc7d2fe)

    // MARK: - Brand Colors - Warm Amber (AI)

    /// AI indicators, special features - #f59e0b
    static let ai = Color(hex: 0xf59e0b)

    /// AI content backgrounds - #fffbeb
    static let aiSubtle = Color(hex: 0xfffbeb)

    /// AI-related borders - #fde68a
    static let aiMuted = Color(hex: 0xfde68a)

    // MARK: - Ink (Text) Colors

    /// Primary text, headings - #1a1a1a
    static let ink = Color(hex: 0x1a1a1a)

    /// Body text, descriptions - #525252
    static let inkSecondary = Color(hex: 0x525252)

    /// Placeholder, disabled, captions - #8a8a8a
    static let inkTertiary = Color(hex: 0x8a8a8a)

    /// Subtle text, metadata - #b3b3b3
    static let inkQuaternary = Color(hex: 0xb3b3b3)

    // MARK: - Surface (Background) Colors

    /// Main background - #ffffff
    static let surface = Color(hex: 0xffffff)

    /// Secondary backgrounds, sidebars - #fafafa
    static let surfaceSecondary = Color(hex: 0xfafafa)

    /// Tertiary backgrounds, hover states - #f5f5f5
    static let surfaceTertiary = Color(hex: 0xf5f5f5)

    /// Cards, modals, dropdowns - #ffffff
    static let surfaceElevated = Color(hex: 0xffffff)

    // MARK: - Border Colors

    /// Default borders - #e5e5e5
    static let border = Color(hex: 0xe5e5e5)

    /// Subtle dividers - #f0f0f0
    static let borderSubtle = Color(hex: 0xf0f0f0)

    // MARK: - Semantic Colors

    /// Success - Confirmations, positive actions - #10b981
    static let success = Color(hex: 0x10b981)

    /// Success backgrounds - #ecfdf5
    static let successSubtle = Color(hex: 0xecfdf5)

    /// Warning - Cautions, alerts - #f59e0b
    static let warning = Color(hex: 0xf59e0b)

    /// Warning backgrounds - #fffbeb
    static let warningSubtle = Color(hex: 0xfffbeb)

    /// Error - Errors, destructive actions - #ef4444
    static let error = Color(hex: 0xef4444)

    /// Error backgrounds - #fef2f2
    static let errorSubtle = Color(hex: 0xfef2f2)

    /// Info - Informational, tips - #3b82f6
    static let info = Color(hex: 0x3b82f6)

    /// Info backgrounds - #eff6ff
    static let infoSubtle = Color(hex: 0xeff6ff)

    // MARK: - Dark Mode Colors

    /// Dark mode ink - #fafafa
    static let inkDark = Color(hex: 0xfafafa)

    /// Dark mode ink secondary - #a3a3a3
    static let inkSecondaryDark = Color(hex: 0xa3a3a3)

    /// Dark mode surface - #0a0a0a
    static let surfaceDark = Color(hex: 0x0a0a0a)

    /// Dark mode surface secondary - #141414
    static let surfaceSecondaryDark = Color(hex: 0x141414)

    /// Dark mode border - #2a2a2a
    static let borderDark = Color(hex: 0x2a2a2a)
}

// MARK: - Color Hex Initializer

extension Color {
    init(hex: UInt, alpha: Double = 1.0) {
        self.init(
            .sRGB,
            red: Double((hex >> 16) & 0xff) / 255,
            green: Double((hex >> 08) & 0xff) / 255,
            blue: Double((hex >> 00) & 0xff) / 255,
            opacity: alpha
        )
    }
}

// MARK: - Adaptive Colors (Light/Dark Mode)

extension Color {
    /// Adaptive ink color for light/dark mode
    static func adaptiveInk(_ colorScheme: ColorScheme) -> Color {
        colorScheme == .dark ? .inkDark : .ink
    }

    /// Adaptive ink secondary color for light/dark mode
    static func adaptiveInkSecondary(_ colorScheme: ColorScheme) -> Color {
        colorScheme == .dark ? .inkSecondaryDark : .inkSecondary
    }

    /// Adaptive surface color for light/dark mode
    static func adaptiveSurface(_ colorScheme: ColorScheme) -> Color {
        colorScheme == .dark ? .surfaceDark : .surface
    }

    /// Adaptive surface secondary color for light/dark mode
    static func adaptiveSurfaceSecondary(_ colorScheme: ColorScheme) -> Color {
        colorScheme == .dark ? .surfaceSecondaryDark : .surfaceSecondary
    }

    /// Adaptive border color for light/dark mode
    static func adaptiveBorder(_ colorScheme: ColorScheme) -> Color {
        colorScheme == .dark ? .borderDark : .border
    }
}

// MARK: - Design Token Constants

struct DesignTokens {
    // MARK: - Spacing (based on 4px base unit)

    struct Spacing {
        static let space0: CGFloat = 0
        static let spacePx: CGFloat = 1
        static let space0_5: CGFloat = 2
        static let space1: CGFloat = 4
        static let space1_5: CGFloat = 6
        static let space2: CGFloat = 8
        static let space2_5: CGFloat = 10
        static let space3: CGFloat = 12
        static let space4: CGFloat = 16
        static let space5: CGFloat = 20
        static let space6: CGFloat = 24
        static let space8: CGFloat = 32
        static let space10: CGFloat = 40
        static let space12: CGFloat = 48
        static let space16: CGFloat = 64
        static let space20: CGFloat = 80
        static let space24: CGFloat = 96
    }

    // MARK: - Border Radius

    struct Radius {
        static let none: CGFloat = 0
        static let sm: CGFloat = 4
        static let md: CGFloat = 8
        static let lg: CGFloat = 12
        static let xl: CGFloat = 16
        static let xxl: CGFloat = 24
        static let full: CGFloat = 9999
    }

    // MARK: - Typography

    struct Typography {
        // Font sizes
        static let textXs: CGFloat = 12
        static let textSm: CGFloat = 14
        static let textBase: CGFloat = 16
        static let textLg: CGFloat = 18
        static let textXl: CGFloat = 20
        static let text2xl: CGFloat = 24
        static let text3xl: CGFloat = 30
        static let text4xl: CGFloat = 36
        static let text5xl: CGFloat = 48
        static let text6xl: CGFloat = 60
    }

    // MARK: - Animation Timing

    struct Animation {
        static let durationFast: Double = 0.1      // 100ms - Micro-interactions
        static let durationSimple: Double = 0.15  // 150ms - Simple transitions
        static let durationStandard: Double = 0.2 // 200ms - Standard transitions
        static let durationComplex: Double = 0.3  // 300ms - Complex transitions
        static let durationElaborate: Double = 0.5 // 500ms - Elaborate animations
    }
}
