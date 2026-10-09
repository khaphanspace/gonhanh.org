@testable import GoNhanh
import XCTest

/// The per-app injection table: which method and delays each app gets.
final class InjectionProfileTests: XCTestCase {
    private func resolve(_ id: String, _ role: String? = nil) -> InjectionProfile {
        InjectionProfiles.resolve(bundleId: id, role: role)
    }

    private func assertDelays(_ p: InjectionProfile, _ d: InjectDelays, file: StaticString = #filePath, line: UInt = #line) {
        XCTAssertEqual(p.delays.backspace, d.backspace, file: file, line: line)
        XCTAssertEqual(p.delays.wait, d.wait, file: file, line: line)
        XCTAssertEqual(p.delays.text, d.text, file: file, line: line)
    }

    func testUnknownAppsGetTheSafeDefault() {
        let p = resolve("com.example.unknown")
        XCTAssertEqual(p.method, .fast)
        assertDelays(p, Delays.low)
    }

    func testRemoteDesktopAndIPhoneMirroringPassKeysThrough() {
        for id in ["com.apple.ScreenContinuity", "com.carriez.rustdesk", "com.philandro.anydesk", "com.teamviewer.TeamViewer"] {
            XCTAssertEqual(resolve(id).method, .passthrough, id)
        }
    }

    func testAutocompleteElementsUseSelectionInAnyApp() {
        XCTAssertEqual(resolve("com.google.Chrome", "AXComboBox").method, .selection)
        XCTAssertEqual(resolve("com.example.unknown", "AXSearchField").method, .selection)
        // but never before the pass-through rules
        XCTAssertEqual(resolve("com.carriez.rustdesk", "AXComboBox").method, .passthrough)
    }

    func testSafariAddressBarAndContentDiffer() {
        XCTAssertEqual(resolve("com.apple.Safari", "AXTextField").method, .emptyCharPrefix)
        XCTAssertEqual(resolve("com.apple.Safari", "AXWebArea").method, .charByChar)
        assertDelays(resolve("com.apple.Safari"), Delays.medium)
    }

    func testBrowsersBreakAutocompleteWithTheEmptyCharPrefix() {
        for id in ["com.google.Chrome", "org.mozilla.firefox", "company.thebrowser.Browser", "com.microsoft.edgemac", "com.brave.Browser"] {
            let p = resolve(id)
            XCTAssertEqual(p.method, .emptyCharPrefix, id)
            assertDelays(p, Delays.medium)
        }
    }

    func testEditorsAndTerminalsAreSlow() {
        for id in ["com.microsoft.VSCode", "com.apple.Terminal", "dev.warp.Warp-Stable", "dev.zed.Zed"] {
            let p = resolve(id)
            XCTAssertEqual(p.method, .slow, id)
            assertDelays(p, Delays.high)
        }
    }

    func testJetBrainsTextFieldsUseSelectionAndTheEditorIsSlow() {
        XCTAssertEqual(resolve("com.jetbrains.intellij", "AXTextField").method, .selection)
        let editor = resolve("com.jetbrains.intellij", "AXTextArea")
        XCTAssertEqual(editor.method, .slow)
        assertDelays(editor, Delays.high)
    }

    func testOfficeAndElectronApps() {
        assertDelays(resolve("com.microsoft.Excel"), Delays.medium)
        assertDelays(resolve("com.microsoft.Outlook"), (8000, 15000, 8000))
        assertDelays(resolve("notion.id"), Delays.veryHigh)
        assertDelays(resolve("com.todesktop.230313mzl4w4u92"), (8000, 15000, 8000))
    }

    func testCharByCharApps() {
        XCTAssertEqual(resolve("com.adobe.Photoshop").method, .charByChar)
        XCTAssertEqual(resolve("com.adobe.Illustrator").method, .charByChar)
        XCTAssertEqual(resolve("texstudio").method, .charByChar)
        assertDelays(resolve("com.caudex.dev"), (5000, 15000, 5000))
        assertDelays(resolve("com.foxit-software.Foxit.PDF.Reader"), Delays.zero)
    }

    func testGamesInjectSynchronously() {
        XCTAssertEqual(resolve("com.riotgames.LeagueofLegends").method, .syncProxy)
    }

    func testUserOverridesReplaceMethodAndDelays() {
        let detected = resolve("com.google.Chrome")
        var config = PerAppConfig()
        config.delayPreset = DelayPreset.veryHigh.rawValue
        config.injectionOverride = InjectionOverride.slow.rawValue
        let p = detected.applying(config)
        XCTAssertEqual(p.method, .slow)
        assertDelays(p, Delays.veryHigh)
        XCTAssertTrue(p.tag.hasPrefix("override:"))

        // "auto" keeps the detected method; no config changes nothing
        config.injectionOverride = InjectionOverride.auto.rawValue
        XCTAssertEqual(detected.applying(config).method, .emptyCharPrefix)
        XCTAssertEqual(detected.applying(nil).method, .emptyCharPrefix)
        assertDelays(detected.applying(nil), Delays.medium)
    }

    func testPresetsAreTheDetectionLevels() {
        assertDelays(InjectionProfile(.fast, DelayPreset.low.delays, ""), Delays.low)
        XCTAssertEqual(DelayPreset.closest(to: Delays.high), .high)
        XCTAssertEqual(DelayPreset.closest(to: Delays.none), .none)
    }
}
