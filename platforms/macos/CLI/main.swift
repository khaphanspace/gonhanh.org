import CoreFoundation
import Darwin
import Foundation

// MARK: - Bundle Version Lookup

/// Resolve `CFBundleShortVersionString` from the enclosing `GoNhanh.app/Contents/Info.plist`
/// (following symlinks such as `/opt/homebrew/bin/gonhanh`), so `gonhanh version` works
/// even when `GoNhanh.app` is not currently running.
private func resolveBundledVersion() -> String? {
    var buffer = [CChar](repeating: 0, count: Int(MAXPATHLEN))
    var size = UInt32(buffer.count)
    let rawExecutablePath: String? = if _NSGetExecutablePath(&buffer, &size) == 0 {
        String(cString: buffer)
    } else {
        CommandLine.arguments.first
    }

    guard let rawExecutablePath else { return nil }

    var resolved = [CChar](repeating: 0, count: Int(PATH_MAX))
    let realExecPath: String = if realpath(rawExecutablePath, &resolved) != nil {
        String(cString: resolved)
    } else {
        URL(fileURLWithPath: rawExecutablePath).standardizedFileURL.path
    }

    let execDir = URL(fileURLWithPath: realExecPath).deletingLastPathComponent()
    // Supports Contents/SharedSupport/bin/gonhanh (../../Info.plist) and Contents/MacOS|Helpers (../Info.plist)
    let candidates = [
        execDir.appendingPathComponent("../../Info.plist").standardizedFileURL,
        execDir.appendingPathComponent("../Info.plist").standardizedFileURL,
    ]

    for plistURL in candidates where FileManager.default.isReadableFile(atPath: plistURL.path) {
        if let data = try? Data(contentsOf: plistURL),
           let plist = try? PropertyListSerialization.propertyList(from: data, format: nil) as? [String: Any],
           let version = plist["CFBundleShortVersionString"] as? String,
           !version.isEmpty
        {
            return version
        }
    }
    return nil
}

// MARK: - CLI Entry Point

private func writeStderr(_ message: String) {
    FileHandle.standardError.write(Data("\(message)\n".utf8))
}

private func runCLI() -> Int32 {
    let args = Array(CommandLine.arguments.dropFirst())
    let portName = ProcessInfo.processInfo.environment[ControlProtocol.portEnvKey] ?? ControlProtocol.portName

    switch ControlProtocol.parse(arguments: args) {
    case .help:
        print(ControlProtocol.usageText)
        return 0

    case .version:
        if let version = resolveBundledVersion() {
            print("Gõ Nhanh v\(version)")
            return 0
        }
        let response = ControlProtocol.sendRequest(.version, portName: portName)
        if response.isSuccess, !response.payload.isEmpty {
            print(response.payload)
        } else {
            print("Gõ Nhanh vunknown")
        }
        return 0

    case let .invalid(badArg):
        writeStderr(ControlProtocol.usageText)
        writeStderr("Lỗi: Lệnh không hợp lệ: \(badArg)")
        return 1

    case let .remote(command):
        let response = ControlProtocol.sendRequest(command, portName: portName)
        guard response.isSuccess else {
            writeStderr("Lỗi: \(response.payload)")
            return 1
        }
        if !response.payload.isEmpty {
            print(response.payload)
        }
        return 0
    }
}

exit(runCLI())
