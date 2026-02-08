import XCTest
@testable import GoNhanh

// MARK: - Update Manager Tests

final class UpdateManagerTests: XCTestCase {

    func testSharedInstanceExists() {
        XCTAssertNotNil(UpdateManager.shared)
    }

    func testDefaultUpdateAvailableIsFalse() {
        XCTAssertFalse(UpdateManager.shared.updateAvailable)
    }
}
