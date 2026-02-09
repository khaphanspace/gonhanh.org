import XCTest
@testable import GoNhanh

// MARK: - Text Modifying Keys Tests

final class TextModifyingKeysTest: XCTestCase {

    func testCmdBackspaceIsInTextModifyingKeys() {
        XCTAssertTrue(textModifyingKeys.contains(0x33))  // kVK_Delete (Backspace)
    }

    func testCmdDeleteIsInTextModifyingKeys() {
        XCTAssertTrue(textModifyingKeys.contains(0x75))  // kVK_ForwardDelete
    }
}
