import XCTest
@testable import GoNhanh

// MARK: - Settings Key Tests

final class UpdateSettingsKeyTests: XCTestCase {

    func testAutoCheckUpdateKey() {
        XCTAssertEqual(SettingsKey.autoCheckUpdate, "gonhanh.update.autoCheck")
    }
}
