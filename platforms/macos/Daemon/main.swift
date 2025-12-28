import Foundation

/// GoNhanh IME Daemon Entry Point
/// This daemon handles all keyboard processing and runs as an XPC service

let daemon = IMEDaemon.shared
daemon.run()
