@testable import GoNhanh
import XCTest

// MARK: - Mock Control State

private final class MockControlState: ControlStateHandling {
    var isEngineReady: Bool = true
    var isEnabled: Bool = true
    var isAllowedInputSource: Bool = true
    var currentInputSourceName: String? = "ABC"
    var currentMethod: InputMode = .telex
    var appVersion: String = "1.2.3"

    private(set) var setEnabledCalls: [Bool] = []
    private(set) var setMethodCalls: [InputMode] = []

    func setEnabled(_ enabled: Bool) {
        setEnabledCalls.append(enabled)
        isEnabled = enabled
    }

    func setMethod(_ method: InputMode) {
        setMethodCalls.append(method)
        currentMethod = method
    }
}

// MARK: - Control Protocol & Server Tests

final class ControlServerTests: XCTestCase {
    // MARK: - CLI Argument Parsing

    func testParseNoArgumentsDefaultsToGet() {
        XCTAssertEqual(ControlProtocol.parse(arguments: []), .remote(.get))
    }

    func testParseKnownRemoteCommands() {
        XCTAssertEqual(ControlProtocol.parse(arguments: ["vi"]), .remote(.vi))
        XCTAssertEqual(ControlProtocol.parse(arguments: ["en"]), .remote(.en))
        XCTAssertEqual(ControlProtocol.parse(arguments: ["on"]), .remote(.on))
        XCTAssertEqual(ControlProtocol.parse(arguments: ["off"]), .remote(.off))
        XCTAssertEqual(ControlProtocol.parse(arguments: ["toggle"]), .remote(.toggle))
        XCTAssertEqual(ControlProtocol.parse(arguments: ["telex"]), .remote(.telex))
        XCTAssertEqual(ControlProtocol.parse(arguments: ["vni"]), .remote(.vni))
        XCTAssertEqual(ControlProtocol.parse(arguments: ["status"]), .remote(.status))
    }

    func testParseVersionAndHelpFlags() {
        XCTAssertEqual(ControlProtocol.parse(arguments: ["version"]), .version)
        XCTAssertEqual(ControlProtocol.parse(arguments: ["-v"]), .version)
        XCTAssertEqual(ControlProtocol.parse(arguments: ["--version"]), .version)
        XCTAssertEqual(ControlProtocol.parse(arguments: ["help"]), .help)
        XCTAssertEqual(ControlProtocol.parse(arguments: ["-h"]), .help)
        XCTAssertEqual(ControlProtocol.parse(arguments: ["--help"]), .help)
    }

    func testParseRejectsUnknownAndExtraArguments() {
        XCTAssertEqual(ControlProtocol.parse(arguments: ["unknown"]), .invalid("unknown"))
        XCTAssertEqual(ControlProtocol.parse(arguments: ["vi", "extra"]), .invalid("vi extra"))
    }

    // MARK: - Wire Encoding

    func testResponseEncodeDecodeRoundTrip() {
        let okEmpty = ControlProtocol.Response.ok()
        XCTAssertEqual(ControlProtocol.Response.decode(okEmpty.encode()), okEmpty)

        let okUTF8 = ControlProtocol.Response.ok("● BẬT │ telex │ gonhanh")
        XCTAssertEqual(ControlProtocol.Response.decode(okUTF8.encode()), okUTF8)

        let err = ControlProtocol.Response.error("Gõ Nhanh chưa chạy.")
        XCTAssertEqual(ControlProtocol.Response.decode(err.encode()), err)

        XCTAssertNil(ControlProtocol.Response.decode(Data()))
    }

    // MARK: - Effective State Semantics

    func testGetReflectsEnabledAndInputSourceGate() {
        let state = MockControlState()
        let server = ControlServer(stateHandler: state)

        state.isEnabled = true
        state.isAllowedInputSource = true
        XCTAssertEqual(server.handleCommand(.get), .ok("vi"))

        state.isEnabled = false
        XCTAssertEqual(server.handleCommand(.get), .ok("en"))

        // Gated off by non-Latin input source (e.g., Japanese) -> must report "en"
        state.isEnabled = true
        state.isAllowedInputSource = false
        XCTAssertEqual(server.handleCommand(.get), .ok("en"))

        // Engine not ready (missing Accessibility) -> must report "en"
        state.isAllowedInputSource = true
        state.isEngineReady = false
        XCTAssertEqual(server.handleCommand(.get), .ok("en"))
    }

    func testViAndEnAreSilentOnStdout() {
        let state = MockControlState()
        state.isEnabled = false
        let server = ControlServer(stateHandler: state)

        XCTAssertEqual(server.handleCommand(.vi), .ok(""))
        XCTAssertTrue(state.isEnabled)
        XCTAssertEqual(state.setEnabledCalls, [true])

        XCTAssertEqual(server.handleCommand(.en), .ok(""))
        XCTAssertFalse(state.isEnabled)
        XCTAssertEqual(state.setEnabledCalls, [true, false])
    }

    func testEnPreservesPreferenceWhenGatedByDisallowedInputSource() {
        let state = MockControlState()
        state.isEnabled = true
        state.isAllowedInputSource = false
        let server = ControlServer(stateHandler: state)

        XCTAssertEqual(server.handleCommand(.en), .ok(""))
        XCTAssertTrue(state.isEnabled)
        XCTAssertTrue(state.setEnabledCalls.isEmpty)
    }

    func testOnOffToggleAndMethodCommandsReturnLinuxCompatibleStatus() {
        let state = MockControlState()
        state.isEnabled = false
        state.currentMethod = .telex
        let server = ControlServer(stateHandler: state)

        XCTAssertEqual(server.handleCommand(.on), .ok("● BẬT │ telex │ gonhanh"))
        XCTAssertTrue(state.isEnabled)

        XCTAssertEqual(server.handleCommand(.vni), .ok("● BẬT │ vni │ gonhanh"))
        XCTAssertEqual(state.currentMethod, .vni)

        XCTAssertEqual(server.handleCommand(.toggle), .ok("○ TẮT │ vni"))
        XCTAssertFalse(state.isEnabled)

        XCTAssertEqual(server.handleCommand(.toggle), .ok("● BẬT │ vni │ gonhanh"))
        XCTAssertTrue(state.isEnabled)

        XCTAssertEqual(server.handleCommand(.off), .ok("○ TẮT │ vni"))
        XCTAssertFalse(state.isEnabled)
    }

    func testStatusShowsDisallowedInputSourceReason() {
        let state = MockControlState()
        state.isEnabled = true
        state.isAllowedInputSource = false
        state.currentInputSourceName = "Japanese"
        let server = ControlServer(stateHandler: state)

        XCTAssertEqual(
            server.handleCommand(.status),
            .ok("○ TẮT │ telex │ input source hiện tại: Japanese")
        )
    }

    func testEnablingCommandsFailGracefullyWhenEngineNotReady() {
        let state = MockControlState()
        state.isEngineReady = false
        let server = ControlServer(stateHandler: state)

        XCTAssertFalse(server.handleCommand(.vi).isSuccess)
        XCTAssertFalse(server.handleCommand(.on).isSuccess)
        XCTAssertFalse(server.handleCommand(.toggle).isSuccess)
        XCTAssertFalse(server.handleCommand(.telex).isSuccess)
        XCTAssertFalse(server.handleCommand(.vni).isSuccess)
        // `en` stays non-disruptive for Vim InsertLeave hooks before onboarding completes
        XCTAssertEqual(server.handleCommand(.en), .ok(""))
    }

    // MARK: - Live CFMessagePort IPC

    func testLiveMessagePortRoundTripAndRapidSwitching() {
        let state = MockControlState()
        state.isEnabled = false
        let server = ControlServer(stateHandler: state)
        let portName = "org.gonhanh.GoNhanh.test.\(UUID().uuidString)"
        XCTAssertTrue(server.start(portName: portName))
        defer { server.stop() }

        for i in 0 ..< 50 {
            let target: ControlProtocol.Command = i % 2 == 0 ? .vi : .en
            let setResp = sendFromBackground(target, portName: portName)
            XCTAssertEqual(setResp, .ok(""))

            let getResp = sendFromBackground(.get, portName: portName)
            XCTAssertEqual(getResp, .ok(i % 2 == 0 ? "vi" : "en"))
        }
    }

    func testUnreachablePortReturnsError() {
        let portName = "org.gonhanh.GoNhanh.missing.\(UUID().uuidString)"
        let resp = ControlProtocol.sendRequest(.get, portName: portName, timeout: 0.1)
        XCTAssertFalse(resp.isSuccess)
        XCTAssertFalse(resp.payload.isEmpty)
    }

    private func sendFromBackground(
        _ command: ControlProtocol.Command,
        portName: String
    ) -> ControlProtocol.Response {
        var result: ControlProtocol.Response?
        let done = expectation(description: "IPC \(command.rawValue)")
        DispatchQueue.global(qos: .userInteractive).async {
            result = ControlProtocol.sendRequest(command, portName: portName, timeout: 1.0)
            done.fulfill()
        }
        wait(for: [done], timeout: 2.0)
        return result ?? .error("timeout")
    }
}
