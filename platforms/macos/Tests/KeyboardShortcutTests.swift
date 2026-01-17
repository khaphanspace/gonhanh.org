import XCTest
@testable import GoNhanh

// MARK: - Keyboard Shortcut Tests

final class KeyboardShortcutTests: XCTestCase {

    override func tearDown() {
        UserDefaults.standard.removeObject(forKey: SettingsKey.toggleShortcut)
        super.tearDown()
    }

    // MARK: - Default Shortcut

    func testDefaultShortcut() {
        let defaultShortcut = KeyboardShortcut.default

        XCTAssertEqual(defaultShortcut.keyCode, 0x31)  // Space
        XCTAssertEqual(defaultShortcut.modifiers, CGEventFlags.maskControl.rawValue)
    }

    // MARK: - Display Parts

    func testDisplayPartsCtrlSpace() {
        let shortcut = KeyboardShortcut.default
        let parts = shortcut.displayParts

        XCTAssertEqual(parts, ["⌃", "Space"])
    }

    func testDisplayPartsCmdShift() {
        let modifiers = CGEventFlags([.maskCommand, .maskShift]).rawValue
        let shortcut = KeyboardShortcut(keyCode: 0xFFFF, modifiers: modifiers)
        let parts = shortcut.displayParts

        XCTAssertEqual(parts, ["⇧", "⌘"])
    }

    func testDisplayPartsCtrlShift() {
        let modifiers = CGEventFlags([.maskControl, .maskShift]).rawValue
        let shortcut = KeyboardShortcut(keyCode: 0xFFFF, modifiers: modifiers)
        let parts = shortcut.displayParts

        XCTAssertEqual(parts, ["⌃", "⇧"])
    }

    func testDisplayPartsAllModifiers() {
        let modifiers = CGEventFlags([.maskControl, .maskAlternate, .maskShift, .maskCommand]).rawValue
        let shortcut = KeyboardShortcut(keyCode: 0x00, modifiers: modifiers)  // A key
        let parts = shortcut.displayParts

        XCTAssertEqual(parts, ["⌃", "⌥", "⇧", "⌘", "A"])
    }

    func testDisplayPartsModifierOnlyNoKeyString() {
        let modifiers = CGEventFlags([.maskCommand, .maskShift]).rawValue
        let shortcut = KeyboardShortcut(keyCode: 0xFFFF, modifiers: modifiers)
        let parts = shortcut.displayParts

        // Should not contain empty string for modifier-only shortcuts
        XCTAssertFalse(parts.contains(""))
        XCTAssertEqual(parts.count, 2)
    }

    // MARK: - Modifier Only Detection

    func testIsModifierOnlyTrue() {
        let shortcut = KeyboardShortcut(keyCode: 0xFFFF, modifiers: CGEventFlags.maskCommand.rawValue)
        XCTAssertTrue(shortcut.isModifierOnly)
    }

    func testIsModifierOnlyFalse() {
        let shortcut = KeyboardShortcut.default
        XCTAssertFalse(shortcut.isModifierOnly)
    }

    func testIsModifierOnlyCmdShift() {
        let modifiers = CGEventFlags([.maskCommand, .maskShift]).rawValue
        let shortcut = KeyboardShortcut(keyCode: 0xFFFF, modifiers: modifiers)
        XCTAssertTrue(shortcut.isModifierOnly)
    }

    // MARK: - Persistence

    func testSaveAndLoad() {
        let modifiers = CGEventFlags([.maskCommand, .maskShift]).rawValue
        let shortcut = KeyboardShortcut(keyCode: 0xFFFF, modifiers: modifiers)

        shortcut.save()
        let loaded = KeyboardShortcut.load()

        XCTAssertEqual(loaded.keyCode, shortcut.keyCode)
        XCTAssertEqual(loaded.modifiers, shortcut.modifiers)
    }

    func testLoadReturnsDefaultWhenNoData() {
        UserDefaults.standard.removeObject(forKey: SettingsKey.toggleShortcut)

        let loaded = KeyboardShortcut.load()

        XCTAssertEqual(loaded, KeyboardShortcut.default)
    }

    func testSaveAndLoadModifierOnly() {
        let modifiers = CGEventFlags([.maskControl, .maskShift]).rawValue
        let shortcut = KeyboardShortcut(keyCode: 0xFFFF, modifiers: modifiers)

        shortcut.save()
        let loaded = KeyboardShortcut.load()

        XCTAssertTrue(loaded.isModifierOnly)
        XCTAssertEqual(loaded.modifiers, modifiers)
    }

    // MARK: - Equality

    func testEquality() {
        let shortcut1 = KeyboardShortcut(keyCode: 0x31, modifiers: CGEventFlags.maskControl.rawValue)
        let shortcut2 = KeyboardShortcut(keyCode: 0x31, modifiers: CGEventFlags.maskControl.rawValue)

        XCTAssertEqual(shortcut1, shortcut2)
    }

    func testInequalityDifferentKeyCode() {
        let shortcut1 = KeyboardShortcut(keyCode: 0x31, modifiers: CGEventFlags.maskControl.rawValue)
        let shortcut2 = KeyboardShortcut(keyCode: 0x00, modifiers: CGEventFlags.maskControl.rawValue)

        XCTAssertNotEqual(shortcut1, shortcut2)
    }

    func testInequalityDifferentModifiers() {
        let shortcut1 = KeyboardShortcut(keyCode: 0x31, modifiers: CGEventFlags.maskControl.rawValue)
        let shortcut2 = KeyboardShortcut(keyCode: 0x31, modifiers: CGEventFlags.maskCommand.rawValue)

        XCTAssertNotEqual(shortcut1, shortcut2)
    }

    func testEqualityModifierOnly() {
        let mods = CGEventFlags([.maskCommand, .maskShift]).rawValue
        let shortcut1 = KeyboardShortcut(keyCode: 0xFFFF, modifiers: mods)
        let shortcut2 = KeyboardShortcut(keyCode: 0xFFFF, modifiers: mods)

        XCTAssertEqual(shortcut1, shortcut2)
    }

    // MARK: - Key Code to String

    func testKeyCodeToStringSpecialKeys() {
        let testCases: [(keyCode: UInt16, expected: String)] = [
            (0x31, "Space"),
            (0x24, "↩"),
            (0x30, "⇥"),
            (0x33, "⌫"),
            (0x35, "⎋"),
            (0x7B, "←"),
            (0x7C, "→"),
            (0x7D, "↓"),
            (0x7E, "↑"),
        ]

        for (keyCode, expected) in testCases {
            let shortcut = KeyboardShortcut(keyCode: keyCode, modifiers: 0)
            let parts = shortcut.displayParts

            XCTAssertTrue(parts.contains(expected), "Key code \(keyCode) should map to \(expected)")
        }
    }

    func testKeyCodeToStringLetters() {
        let letterCodes: [(code: UInt16, letter: String)] = [
            (0x00, "A"), (0x01, "S"), (0x02, "D"), (0x03, "F"),
            (0x06, "Z"), (0x07, "X"), (0x08, "C"), (0x09, "V"),
        ]

        for (code, letter) in letterCodes {
            let shortcut = KeyboardShortcut(keyCode: code, modifiers: CGEventFlags.maskCommand.rawValue)
            let parts = shortcut.displayParts

            XCTAssertTrue(parts.contains(letter), "Key code \(code) should map to \(letter)")
        }
    }

    func testKeyCodeModifierOnlyReturnsEmpty() {
        let shortcut = KeyboardShortcut(keyCode: 0xFFFF, modifiers: CGEventFlags.maskCommand.rawValue)
        let parts = shortcut.displayParts

        // 0xFFFF should not add any key string, only modifier
        XCTAssertEqual(parts, ["⌘"])
    }

    // MARK: - Shortcut Matching (Key + Modifier)

    func testMatchesExact() {
        let shortcut = KeyboardShortcut.default  // Ctrl+Space
        XCTAssertTrue(shortcut.matches(keyCode: 0x31, flags: .maskControl))
    }

    func testMatchesExactMultipleModifiers() {
        let shortcut = KeyboardShortcut(keyCode: 0x00, modifiers: CGEventFlags([.maskCommand, .maskShift]).rawValue)
        XCTAssertTrue(shortcut.matches(keyCode: 0x00, flags: CGEventFlags([.maskCommand, .maskShift])))
    }

    func testMatchesRejectsExtraModifier() {
        // This is the bug PR #72 fixes: Ctrl+Space should NOT match Ctrl+Shift+Space
        let shortcut = KeyboardShortcut.default
        XCTAssertFalse(shortcut.matches(keyCode: 0x31, flags: CGEventFlags([.maskControl, .maskShift])))
        XCTAssertFalse(shortcut.matches(keyCode: 0x31, flags: CGEventFlags([.maskControl, .maskAlternate])))
    }

    func testMatchesRejectsMissingModifier() {
        let shortcut = KeyboardShortcut(keyCode: 0x31, modifiers: CGEventFlags([.maskControl, .maskShift]).rawValue)
        XCTAssertFalse(shortcut.matches(keyCode: 0x31, flags: .maskControl))
    }

    func testMatchesRejectsDifferentKeyCode() {
        let shortcut = KeyboardShortcut.default  // Ctrl+Space (0x31)
        XCTAssertFalse(shortcut.matches(keyCode: 0x00, flags: .maskControl))
    }

    func testMatchesRejectsNoModifiers() {
        let shortcut = KeyboardShortcut.default
        XCTAssertFalse(shortcut.matches(keyCode: 0x31, flags: CGEventFlags([])))
    }

    func testMatchesRejectsDifferentModifier() {
        let shortcut = KeyboardShortcut.default  // Ctrl+Space
        XCTAssertFalse(shortcut.matches(keyCode: 0x31, flags: .maskCommand))
    }

    func testMatchesIgnoresNonModifierFlags() {
        let shortcut = KeyboardShortcut.default
        var flags = CGEventFlags.maskControl
        flags.insert(.maskNumericPad)  // Non-modifier flag (numpad indicator)
        XCTAssertTrue(shortcut.matches(keyCode: 0x31, flags: flags))
    }

    func testMatchesReturnsFalseForModifierOnlyShortcut() {
        let shortcut = KeyboardShortcut(keyCode: 0xFFFF, modifiers: CGEventFlags.maskCommand.rawValue)
        XCTAssertFalse(shortcut.matches(keyCode: 0xFFFF, flags: .maskCommand))
    }

    // MARK: - Modifier-Only Shortcut Matching

    func testMatchesModifierOnlyExact() {
        let shortcut = KeyboardShortcut(keyCode: 0xFFFF, modifiers: CGEventFlags([.maskCommand, .maskShift]).rawValue)
        XCTAssertTrue(shortcut.matchesModifierOnly(flags: CGEventFlags([.maskCommand, .maskShift])))
    }

    func testMatchesModifierOnlyRejectsExtraModifier() {
        let shortcut = KeyboardShortcut(keyCode: 0xFFFF, modifiers: CGEventFlags([.maskCommand, .maskShift]).rawValue)
        XCTAssertFalse(shortcut.matchesModifierOnly(flags: CGEventFlags([.maskCommand, .maskShift, .maskAlternate])))
    }

    func testMatchesModifierOnlyRejectsMissingModifier() {
        let shortcut = KeyboardShortcut(keyCode: 0xFFFF, modifiers: CGEventFlags([.maskCommand, .maskShift]).rawValue)
        XCTAssertFalse(shortcut.matchesModifierOnly(flags: .maskCommand))
    }

    func testMatchesModifierOnlyReturnsFalseForKeyShortcut() {
        let shortcut = KeyboardShortcut.default  // Ctrl+Space
        XCTAssertFalse(shortcut.matchesModifierOnly(flags: .maskControl))
    }

    func testMatchesModifierOnlyIgnoresNonModifierFlags() {
        let shortcut = KeyboardShortcut(keyCode: 0xFFFF, modifiers: CGEventFlags([.maskCommand, .maskShift]).rawValue)
        var flags = CGEventFlags([.maskCommand, .maskShift])
        flags.insert(CGEventFlags(rawValue: 0x100000))
        XCTAssertTrue(shortcut.matchesModifierOnly(flags: flags))
    }

    // MARK: - Issue #157: Shift restore shortcut blocks uppercase typing

    /// Issue #157: When Shift is set as restore shortcut (modifier-only),
    /// typing Shift+A should NOT trigger restore - it should type uppercase 'A'.
    /// Modifier-only shortcuts should only trigger on modifier release (flagsChanged),
    /// not when a key is pressed with the modifier held (keyDown).
    func testIssue157_ShiftRestoreShortcutShouldNotBlockUppercaseTyping() {
        // Shift-only restore shortcut
        let restoreShortcut = KeyboardShortcut(keyCode: 0xFFFF, modifiers: CGEventFlags.maskShift.rawValue)
        XCTAssertTrue(restoreShortcut.isModifierOnly)

        // Bug reproduction: matchesRestoreShortcut was returning true for Shift+A
        // because matchesModifierOnly only checks flags, not keyCode.
        // This caused restore to trigger instead of typing uppercase letter.

        // The fix: matchesRestoreShortcut should return false for modifier-only
        // shortcuts when called from keyDown context (with a key pressed).
        // matchesModifierOnly(flags:) is meant for flagsChanged events only.

        // When typing Shift+A (keyCode=0x00, flags=.maskShift):
        // - matchesModifierOnly should still return true (flags match)
        // - But matchesRestoreShortcut should return false (key is pressed)
        let typingAWithShift = CGEventFlags.maskShift
        XCTAssertTrue(restoreShortcut.matchesModifierOnly(flags: typingAWithShift))

        // The matches(keyCode:flags:) should return false for modifier-only shortcuts
        // (this already works correctly)
        XCTAssertFalse(restoreShortcut.matches(keyCode: 0x00, flags: typingAWithShift))
    }
}
