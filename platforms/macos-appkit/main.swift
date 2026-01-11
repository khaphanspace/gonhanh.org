// main.swift - Pure AppKit entry point (no SwiftUI)
// Target: <10MB RAM

import Cocoa

// Create and run the application
let app = NSApplication.shared
let delegate = AppDelegate()
app.delegate = delegate
app.run()
