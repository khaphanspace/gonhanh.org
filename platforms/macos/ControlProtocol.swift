import CoreFoundation
import Foundation

// MARK: - Control Protocol

/// Shared IPC protocol between GoNhanh.app and the `gonhanh` / `gn` CLI.
/// Uses `CFMessagePort` for synchronous, sub-millisecond request/response IPC.
enum ControlProtocol {
    static let portName = "org.gonhanh.GoNhanh.control"
    static let portEnvKey = "GONHANH_CONTROL_PORT"
    static let messageId: Int32 = 1
    static let statusOK: UInt8 = 0
    static let statusError: UInt8 = 1
    static let timeout: CFTimeInterval = 1.0

    // MARK: - Commands

    enum Command: String, Equatable, CaseIterable {
        case get
        case vi
        case en
        case on
        case off
        case toggle
        case telex
        case vni
        case status
        case version
    }

    enum CLIAction: Equatable {
        case remote(Command)
        case version
        case help
        case invalid(String)
    }

    // MARK: - Wire Response

    struct Response: Equatable {
        let isSuccess: Bool
        let payload: String

        static func ok(_ payload: String = "") -> Response {
            Response(isSuccess: true, payload: payload)
        }

        static func error(_ message: String) -> Response {
            Response(isSuccess: false, payload: message)
        }

        func encode() -> Data {
            var data = Data([isSuccess ? ControlProtocol.statusOK : ControlProtocol.statusError])
            if !payload.isEmpty {
                data.append(contentsOf: payload.utf8)
            }
            return data
        }

        static func decode(_ data: Data) -> Response? {
            guard let first = data.first else { return nil }
            let text = String(decoding: data.dropFirst(), as: UTF8.self)
            return Response(isSuccess: first == ControlProtocol.statusOK, payload: text)
        }
    }

    // MARK: - CLI Parsing & Formatting

    static let usageText = """
    Gõ Nhanh - Vietnamese Input Method

    Cách dùng: gonhanh [lệnh] (hoặc: gn [lệnh])

      (không tham số)  In trạng thái hiện tại: "vi" hoặc "en"
      vi               Bật gõ tiếng Việt (im lặng, dùng cho Vim/script)
      en               Tắt gõ tiếng Việt (im lặng, dùng cho Vim/script)
      on               Bật Gõ Nhanh và hiện trạng thái
      off              Tắt Gõ Nhanh và hiện trạng thái
      toggle           Đảo trạng thái bật/tắt
      telex            Chuyển sang Telex và bật Gõ Nhanh
      vni              Chuyển sang VNI và bật Gõ Nhanh
      status           Xem trạng thái riêng của Gõ Nhanh
      version          Xem phiên bản
      help             Hiển thị trợ giúp
    """

    static func parse(arguments: [String]) -> CLIAction {
        guard !arguments.isEmpty else {
            return .remote(.get)
        }

        guard arguments.count == 1 else {
            return .invalid(arguments.joined(separator: " "))
        }

        let arg = arguments[0]
        switch arg {
        case "vi":
            return .remote(.vi)
        case "en":
            return .remote(.en)
        case "on":
            return .remote(.on)
        case "off":
            return .remote(.off)
        case "toggle":
            return .remote(.toggle)
        case "telex":
            return .remote(.telex)
        case "vni":
            return .remote(.vni)
        case "status":
            return .remote(.status)
        case "version", "-v", "--version":
            return .version
        case "help", "-h", "--help":
            return .help
        default:
            return .invalid(arg)
        }
    }

    static func formatStatus(
        effectiveEnabled: Bool,
        methodName: String,
        inactiveReason: String? = nil
    ) -> String {
        let method = methodName.lowercased()
        if effectiveEnabled {
            return "● BẬT │ \(method) │ gonhanh"
        }
        if let inactiveReason, !inactiveReason.isEmpty {
            return "○ TẮT │ \(method) │ \(inactiveReason)"
        }
        return "○ TẮT │ \(method)"
    }

    // MARK: - Client Request

    static func sendRequest(
        _ command: Command,
        portName: String = ControlProtocol.portName,
        timeout: CFTimeInterval = ControlProtocol.timeout
    ) -> Response {
        guard let remotePort = CFMessagePortCreateRemote(kCFAllocatorDefault, portName as CFString) else {
            return .error("Gõ Nhanh chưa chạy.")
        }

        let requestData = Data(command.rawValue.utf8) as CFData
        var returnDataUnmanaged: Unmanaged<CFData>?
        let status = CFMessagePortSendRequest(
            remotePort,
            messageId,
            requestData,
            timeout,
            timeout,
            CFRunLoopMode.defaultMode.rawValue,
            &returnDataUnmanaged
        )

        guard status == kCFMessagePortSuccess,
              let responseCFData = returnDataUnmanaged?.takeRetainedValue()
        else {
            return .error("Không thể giao tiếp với Gõ Nhanh (mã lỗi \(status)).")
        }

        guard let response = Response.decode(responseCFData as Data) else {
            return .error("Phản hồi không hợp lệ từ Gõ Nhanh.")
        }

        return response
    }
}
