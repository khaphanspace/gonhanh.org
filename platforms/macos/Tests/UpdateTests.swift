import XCTest
@testable import GoNhanh

// MARK: - Update Status Tests

final class UpdateStatusTests: XCTestCase {

    // MARK: - State Checks

    func testIsCheckingTrue() {
        let status = UpdateStatus.checking
        XCTAssertTrue(status.isChecking)
    }

    func testIsCheckingFalseForOtherStates() {
        XCTAssertFalse(UpdateStatus.idle.isChecking)
        XCTAssertFalse(UpdateStatus.upToDate.isChecking)
        XCTAssertFalse(UpdateStatus.available("1.0.0").isChecking)
        XCTAssertFalse(UpdateStatus.downloading(0.5).isChecking)
        XCTAssertFalse(UpdateStatus.installing.isChecking)
        XCTAssertFalse(UpdateStatus.error.isChecking)
    }

    func testIsAvailableTrue() {
        let status = UpdateStatus.available("1.0.0")
        XCTAssertTrue(status.isAvailable)
    }

    func testIsAvailableFalseForOtherStates() {
        XCTAssertFalse(UpdateStatus.idle.isAvailable)
        XCTAssertFalse(UpdateStatus.checking.isAvailable)
        XCTAssertFalse(UpdateStatus.upToDate.isAvailable)
        XCTAssertFalse(UpdateStatus.downloading(0.5).isAvailable)
        XCTAssertFalse(UpdateStatus.installing.isAvailable)
        XCTAssertFalse(UpdateStatus.error.isAvailable)
    }

    func testIsDownloadingTrue() {
        let status = UpdateStatus.downloading(0.5)
        XCTAssertTrue(status.isDownloading)
    }

    func testIsDownloadingFalseForOtherStates() {
        XCTAssertFalse(UpdateStatus.idle.isDownloading)
        XCTAssertFalse(UpdateStatus.checking.isDownloading)
        XCTAssertFalse(UpdateStatus.upToDate.isDownloading)
        XCTAssertFalse(UpdateStatus.available("1.0.0").isDownloading)
        XCTAssertFalse(UpdateStatus.installing.isDownloading)
        XCTAssertFalse(UpdateStatus.error.isDownloading)
    }

    func testIsInstallingTrue() {
        let status = UpdateStatus.installing
        XCTAssertTrue(status.isInstalling)
    }

    func testIsInstallingFalseForOtherStates() {
        XCTAssertFalse(UpdateStatus.idle.isInstalling)
        XCTAssertFalse(UpdateStatus.checking.isInstalling)
        XCTAssertFalse(UpdateStatus.upToDate.isInstalling)
        XCTAssertFalse(UpdateStatus.available("1.0.0").isInstalling)
        XCTAssertFalse(UpdateStatus.downloading(0.5).isInstalling)
        XCTAssertFalse(UpdateStatus.error.isInstalling)
    }

    // MARK: - Busy State

    func testIsBusyForChecking() {
        XCTAssertTrue(UpdateStatus.checking.isBusy)
    }

    func testIsBusyForDownloading() {
        XCTAssertTrue(UpdateStatus.downloading(0.5).isBusy)
    }

    func testIsBusyForInstalling() {
        XCTAssertTrue(UpdateStatus.installing.isBusy)
    }

    func testIsBusyFalseForIdle() {
        XCTAssertFalse(UpdateStatus.idle.isBusy)
    }

    func testIsBusyFalseForUpToDate() {
        XCTAssertFalse(UpdateStatus.upToDate.isBusy)
    }

    func testIsBusyFalseForAvailable() {
        XCTAssertFalse(UpdateStatus.available("1.0.0").isBusy)
    }

    func testIsBusyFalseForError() {
        XCTAssertFalse(UpdateStatus.error.isBusy)
    }

    // MARK: - Download Progress

    func testDownloadProgressReturnsValue() {
        let status = UpdateStatus.downloading(0.75)
        XCTAssertEqual(status.downloadProgress, 0.75)
    }

    func testDownloadProgressNilForOtherStates() {
        XCTAssertNil(UpdateStatus.idle.downloadProgress)
        XCTAssertNil(UpdateStatus.checking.downloadProgress)
        XCTAssertNil(UpdateStatus.upToDate.downloadProgress)
        XCTAssertNil(UpdateStatus.available("1.0.0").downloadProgress)
        XCTAssertNil(UpdateStatus.installing.downloadProgress)
        XCTAssertNil(UpdateStatus.error.downloadProgress)
    }

    func testDownloadProgressBounds() {
        XCTAssertEqual(UpdateStatus.downloading(0.0).downloadProgress, 0.0)
        XCTAssertEqual(UpdateStatus.downloading(1.0).downloadProgress, 1.0)
    }

    // MARK: - Equality

    func testEqualityIdle() {
        XCTAssertEqual(UpdateStatus.idle, UpdateStatus.idle)
    }

    func testEqualityChecking() {
        XCTAssertEqual(UpdateStatus.checking, UpdateStatus.checking)
    }

    func testEqualityUpToDate() {
        XCTAssertEqual(UpdateStatus.upToDate, UpdateStatus.upToDate)
    }

    func testEqualityAvailable() {
        XCTAssertEqual(UpdateStatus.available("1.0.0"), UpdateStatus.available("1.0.0"))
    }

    func testInequalityAvailableDifferentVersion() {
        XCTAssertNotEqual(UpdateStatus.available("1.0.0"), UpdateStatus.available("2.0.0"))
    }

    func testEqualityDownloading() {
        XCTAssertEqual(UpdateStatus.downloading(0.5), UpdateStatus.downloading(0.5))
    }

    func testInequalityDownloadingDifferentProgress() {
        XCTAssertNotEqual(UpdateStatus.downloading(0.5), UpdateStatus.downloading(0.75))
    }

    func testEqualityInstalling() {
        XCTAssertEqual(UpdateStatus.installing, UpdateStatus.installing)
    }

    func testEqualityError() {
        XCTAssertEqual(UpdateStatus.error, UpdateStatus.error)
    }

    func testInequalityDifferentStates() {
        XCTAssertNotEqual(UpdateStatus.idle, UpdateStatus.checking)
        XCTAssertNotEqual(UpdateStatus.checking, UpdateStatus.upToDate)
        XCTAssertNotEqual(UpdateStatus.available("1.0.0"), UpdateStatus.downloading(0.5))
        XCTAssertNotEqual(UpdateStatus.downloading(0.5), UpdateStatus.installing)
        XCTAssertNotEqual(UpdateStatus.installing, UpdateStatus.error)
    }
}

// MARK: - Version Comparison Tests

final class VersionComparisonTests: XCTestCase {

    func testVersionCompareEqual() {
        let cmp = UpdateChecker.shared.compareVersions("1.0.0", "1.0.0")
        XCTAssertEqual(cmp, 0)
    }

    func testVersionCompareLessThan() {
        let cmp = UpdateChecker.shared.compareVersions("1.0.0", "1.0.1")
        XCTAssertEqual(cmp, -1)
    }

    func testVersionCompareGreaterThan() {
        let cmp = UpdateChecker.shared.compareVersions("1.0.1", "1.0.0")
        XCTAssertEqual(cmp, 1)
    }

    func testVersionCompareMajor() {
        XCTAssertEqual(UpdateChecker.shared.compareVersions("1.0.0", "2.0.0"), -1)
        XCTAssertEqual(UpdateChecker.shared.compareVersions("2.0.0", "1.0.0"), 1)
    }

    func testVersionCompareMinor() {
        XCTAssertEqual(UpdateChecker.shared.compareVersions("1.0.0", "1.1.0"), -1)
        XCTAssertEqual(UpdateChecker.shared.compareVersions("1.1.0", "1.0.0"), 1)
    }

    func testVersionComparePatch() {
        XCTAssertEqual(UpdateChecker.shared.compareVersions("1.0.0", "1.0.1"), -1)
        XCTAssertEqual(UpdateChecker.shared.compareVersions("1.0.1", "1.0.0"), 1)
    }

    func testVersionCompareMultiDigit() {
        XCTAssertEqual(UpdateChecker.shared.compareVersions("1.0.9", "1.0.10"), -1)
        XCTAssertEqual(UpdateChecker.shared.compareVersions("1.0.109", "1.0.110"), -1)
    }
}

// MARK: - Update Info Tests

final class UpdateInfoTests: XCTestCase {

    func testUpdateInfoInitialization() {
        let url = URL(string: "https://example.com/download.dmg")!
        let date = Date()
        let info = UpdateInfo(
            version: "1.0.1",
            downloadURL: url,
            releaseNotes: "Bug fixes",
            publishedAt: date
        )

        XCTAssertEqual(info.version, "1.0.1")
        XCTAssertEqual(info.downloadURL, url)
        XCTAssertEqual(info.releaseNotes, "Bug fixes")
        XCTAssertEqual(info.publishedAt, date)
    }

    func testUpdateInfoWithNilDate() {
        let url = URL(string: "https://example.com/download.dmg")!
        let info = UpdateInfo(
            version: "1.0.1",
            downloadURL: url,
            releaseNotes: "Bug fixes",
            publishedAt: nil
        )

        XCTAssertNil(info.publishedAt)
    }
}

// MARK: - Update State Tests

final class UpdateStateTests: XCTestCase {

    func testUpdateStateIdle() {
        let state = UpdateState.idle
        if case .idle = state {
            XCTAssertTrue(true)
        } else {
            XCTFail("Expected idle state")
        }
    }

    func testUpdateStateChecking() {
        let state = UpdateState.checking
        if case .checking = state {
            XCTAssertTrue(true)
        } else {
            XCTFail("Expected checking state")
        }
    }

    func testUpdateStateAvailable() {
        let url = URL(string: "https://example.com/download.dmg")!
        let info = UpdateInfo(version: "1.0.1", downloadURL: url, releaseNotes: "", publishedAt: nil)
        let state = UpdateState.available(info)

        if case .available(let i) = state {
            XCTAssertEqual(i.version, "1.0.1")
        } else {
            XCTFail("Expected available state")
        }
    }

    func testUpdateStateUpToDate() {
        let state = UpdateState.upToDate
        if case .upToDate = state {
            XCTAssertTrue(true)
        } else {
            XCTFail("Expected upToDate state")
        }
    }

    func testUpdateStateDownloading() {
        let state = UpdateState.downloading(progress: 0.5)
        if case .downloading(let progress) = state {
            XCTAssertEqual(progress, 0.5)
        } else {
            XCTFail("Expected downloading state")
        }
    }

    func testUpdateStateInstalling() {
        let state = UpdateState.installing
        if case .installing = state {
            XCTAssertTrue(true)
        } else {
            XCTFail("Expected installing state")
        }
    }

    func testUpdateStateError() {
        let state = UpdateState.error("Network error")
        if case .error(let message) = state {
            XCTAssertEqual(message, "Network error")
        } else {
            XCTFail("Expected error state")
        }
    }
}

// MARK: - Settings Key Tests

final class UpdateSettingsKeyTests: XCTestCase {

    func testAutoCheckUpdateKey() {
        XCTAssertEqual(SettingsKey.autoCheckUpdate, "gonhanh.update.autoCheck")
    }

    func testAutoUpdateKey() {
        XCTAssertEqual(SettingsKey.autoUpdate, "gonhanh.update.auto")
    }
}
