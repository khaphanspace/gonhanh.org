@testable import GoNhanh
import XCTest

// MARK: - Update Manager Tests

final class UpdateManagerTests: XCTestCase {
    func testSharedInstanceExists() {
        XCTAssertNotNil(UpdateManager.shared)
    }

    func testDefaultUpdateAvailableIsFalse() {
        XCTAssertFalse(UpdateManager.shared.updateAvailable)
    }
}
