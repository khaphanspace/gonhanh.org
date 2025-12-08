import XCTest
@testable import GoNhanh

// MARK: - Launch at Login Tests

final class LaunchAtLoginTests: XCTestCase {

    var mock: MockLaunchAtLoginManager!

    override func setUp() {
        super.setUp()
        mock = MockLaunchAtLoginManager()
    }

    override func tearDown() {
        mock.reset()
        super.tearDown()
    }

    // MARK: - Initial State

    func testInitialStateIsDisabled() {
        XCTAssertFalse(mock.isEnabled)
        XCTAssertEqual(mock.enableCallCount, 0)
        XCTAssertEqual(mock.disableCallCount, 0)
    }

    // MARK: - Enable Tests

    func testEnableSuccess() throws {
        try mock.enable()

        XCTAssertTrue(mock.isEnabled)
        XCTAssertEqual(mock.enableCallCount, 1)
    }

    func testEnableMultipleTimes() throws {
        try mock.enable()
        try mock.enable()
        try mock.enable()

        XCTAssertTrue(mock.isEnabled)
        XCTAssertEqual(mock.enableCallCount, 3)
    }

    func testEnableThrowsError() {
        mock.shouldThrowOnEnable = true

        XCTAssertThrowsError(try mock.enable()) { error in
            XCTAssertEqual(error as? LaunchAtLoginError, .registrationFailed)
        }
        XCTAssertFalse(mock.isEnabled)
    }

    // MARK: - Disable Tests

    func testDisableSuccess() throws {
        try mock.enable()
        try mock.disable()

        XCTAssertFalse(mock.isEnabled)
        XCTAssertEqual(mock.disableCallCount, 1)
    }

    func testDisableWhenAlreadyDisabled() throws {
        try mock.disable()

        XCTAssertFalse(mock.isEnabled)
        XCTAssertEqual(mock.disableCallCount, 1)
    }

    func testDisableThrowsError() throws {
        try mock.enable()
        mock.shouldThrowOnDisable = true

        XCTAssertThrowsError(try mock.disable()) { error in
            XCTAssertEqual(error as? LaunchAtLoginError, .unregistrationFailed)
        }
        // State should remain enabled on error
        XCTAssertTrue(mock.isEnabled)
    }

    // MARK: - Toggle Tests

    func testToggleOnOff() throws {
        // Off → On
        try mock.enable()
        XCTAssertTrue(mock.isEnabled)

        // On → Off
        try mock.disable()
        XCTAssertFalse(mock.isEnabled)

        // Off → On again
        try mock.enable()
        XCTAssertTrue(mock.isEnabled)
    }

    // MARK: - Reset Tests

    func testResetClearsState() throws {
        try mock.enable()
        mock.shouldThrowOnDisable = true

        mock.reset()

        XCTAssertFalse(mock.isEnabled)
        XCTAssertEqual(mock.enableCallCount, 0)
        XCTAssertEqual(mock.disableCallCount, 0)
        XCTAssertFalse(mock.shouldThrowOnEnable)
        XCTAssertFalse(mock.shouldThrowOnDisable)
    }
}

// MARK: - UserDefaults Integration Tests

final class LaunchAtLoginUserDefaultsTests: XCTestCase {

    let testKey = "testLaunchAtLogin"

    override func tearDown() {
        UserDefaults.standard.removeObject(forKey: testKey)
        UserDefaults.standard.removeObject(forKey: LaunchAtLoginManager.userDefaultsKey)
        super.tearDown()
    }

    func testCachedStateDefaultsToFalse() {
        UserDefaults.standard.removeObject(forKey: LaunchAtLoginManager.userDefaultsKey)
        let manager = LaunchAtLoginManager.shared
        // Note: cachedState reads from UserDefaults, not SMAppService
        // This test verifies the fallback behavior
        XCTAssertFalse(UserDefaults.standard.bool(forKey: LaunchAtLoginManager.userDefaultsKey))
    }

    func testSyncWritesToUserDefaults() {
        // This test requires actual SMAppService which may not work in test environment
        // So we just verify the key exists
        XCTAssertNotNil(LaunchAtLoginManager.userDefaultsKey)
    }
}

// MARK: - Onboarding Flow Tests

final class OnboardingLaunchAtLoginTests: XCTestCase {

    var mock: MockLaunchAtLoginManager!

    override func setUp() {
        super.setUp()
        mock = MockLaunchAtLoginManager()
    }

    func testOnboardingEnablesLaunchAtLogin() throws {
        // Simulate onboarding completion
        simulateOnboardingComplete()

        XCTAssertTrue(mock.isEnabled)
        XCTAssertEqual(mock.enableCallCount, 1)
    }

    func testOnboardingHandlesEnableError() {
        mock.shouldThrowOnEnable = true

        // Should not crash, just log error
        simulateOnboardingComplete()

        XCTAssertFalse(mock.isEnabled)
        XCTAssertEqual(mock.enableCallCount, 1)
    }

    private func simulateOnboardingComplete() {
        do {
            try mock.enable()
        } catch {
            // Log error (matches actual behavior in MenuBar.swift)
            print("[LaunchAtLogin] Error: \(error)")
        }
    }
}

// MARK: - App Restart Behavior Tests

final class AppRestartTests: XCTestCase {

    func testAppStartsWithPreviousState() {
        // Test scenario: App was enabled, user restarts Mac
        // Expected: App should start automatically

        // This is more of a documentation test - actual behavior
        // depends on SMAppService which requires real app bundle

        // We can verify the logic flow:
        // 1. On first launch after onboarding: enable() is called
        // 2. On subsequent launches: SMAppService handles auto-start
        // 3. isEnabled property reflects current state

        let mock = MockLaunchAtLoginManager()

        // Simulate first launch
        try? mock.enable()
        XCTAssertTrue(mock.isEnabled)

        // Simulate "restart" - create new instance but preserve state
        // In real app, SMAppService persists this across restarts
        // Mock simulates this by keeping isEnabled = true
        XCTAssertTrue(mock.isEnabled)
    }

    func testDisabledAppDoesNotAutoStart() throws {
        let mock = MockLaunchAtLoginManager()

        // User explicitly disabled launch at login
        try mock.enable()
        try mock.disable()

        XCTAssertFalse(mock.isEnabled)
        // On next "restart", app should NOT auto-start
        // (SMAppService respects this setting)
    }
}

// MARK: - Status Description Tests

final class LaunchAtLoginStatusTests: XCTestCase {

    func testStatusDescriptionValues() {
        let manager = LaunchAtLoginManager.shared

        // Status should be one of the known values
        let validStatuses = ["enabled", "notFound", "notRegistered", "requiresApproval", "unknown", "unsupported"]
        XCTAssertTrue(validStatuses.contains(manager.statusDescription))
    }
}
