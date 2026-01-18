//
//  SpecialPanelAppDetector.swift
//  GoNhanh
//
//  Detects special panel apps (Spotlight, Raycast) that don't trigger
//  NSWorkspaceDidActivateApplicationNotification
//
//  PERFORMANCE: Uses caching and fast-path detection to avoid expensive
//  CGWindowListCopyWindowInfo and AX queries on every keystroke.
//

import Cocoa
import ApplicationServices

/// Detects special panel/overlay apps like Spotlight and Raycast
class SpecialPanelAppDetector {

    // MARK: - Properties

    /// List of special panel app bundle identifiers
    static let specialPanelApps: [String] = [
        "com.apple.Spotlight",
        "com.raycast.macos",
        "com.runningwithcrayons.Alfred",  // Alfred launcher
        "com.apple.inputmethod.EmojiFunctionRowItem"
    ]

    /// Last detected frontmost app (for tracking changes)
    private static var lastFrontMostApp: String = ""

    // MARK: - Cache

    /// PERFORMANCE: Uses CFAbsoluteTimeGetCurrent() instead of Date() for faster timestamp
    private enum Cache {
        static var result: String?
        static var timestamp: CFAbsoluteTime = 0
        static let ttl: CFAbsoluteTime = 0.3  // 300ms

        static func get() -> String?? {  // Double optional: nil = miss, .some(nil) = cached nil
            CFAbsoluteTimeGetCurrent() - timestamp < ttl ? .some(result) : nil
        }

        static func set(_ value: String?) {
            result = value
            timestamp = CFAbsoluteTimeGetCurrent()
        }

        static func clear() {
            result = nil
            timestamp = 0
        }
    }

    // MARK: - Detection Methods

    /// Check if a bundle ID is a special panel app
    static func isSpecialPanelApp(_ bundleId: String?) -> Bool {
        guard let bundleId = bundleId else { return false }
        return specialPanelApps.contains { bundleId.hasPrefix($0) || bundleId == $0 }
    }

    /// Fast path: check focused element only (cheapest AX query)
    /// Returns bundle ID if focused element belongs to a special panel app
    private static func getFocusedSpecialPanelApp() -> String? {
        let systemWide = AXUIElementCreateSystemWide()
        var focusedElement: CFTypeRef?

        guard AXUIElementCopyAttributeValue(systemWide, kAXFocusedUIElementAttribute as CFString, &focusedElement) == .success,
              let element = focusedElement else {
            return nil
        }

        var pid: pid_t = 0
        guard AXUIElementGetPid(element as! AXUIElement, &pid) == .success, pid > 0,
              let app = NSRunningApplication(processIdentifier: pid),
              let bundleId = app.bundleIdentifier,
              isSpecialPanelApp(bundleId) else {
            return nil
        }

        return bundleId
    }

    /// Get the currently active special panel app (if any)
    /// Uses caching and fast-path to avoid expensive operations on every call
    static func getActiveSpecialPanelApp() -> String? {
        // Check cache first
        if let cached = Cache.get() { return cached }

        // Fast path: check focused element (single AX query)
        if let focusedApp = getFocusedSpecialPanelApp() {
            Cache.set(focusedApp)
            return focusedApp
        }

        // Slow path: full window scan (only if fast path failed)
        let result = getActiveSpecialPanelAppFullScan()
        Cache.set(result)
        return result
    }

    /// Full scan: expensive operation, only called when fast path fails
    private static func getActiveSpecialPanelAppFullScan() -> String? {
        // Method 1: Use CGWindowListCopyWindowInfo to find on-screen windows
        if let windowList = CGWindowListCopyWindowInfo([.optionOnScreenOnly, .excludeDesktopElements], kCGNullWindowID) as? [[String: Any]] {
            for window in windowList {
                guard let ownerPID = window[kCGWindowOwnerPID as String] as? pid_t,
                      let windowLayer = window[kCGWindowLayer as String] as? Int else {
                    continue
                }

                // Spotlight and Raycast typically use high window layers (above normal windows)
                if windowLayer > 0 {
                    if let app = NSRunningApplication(processIdentifier: ownerPID),
                       let bundleId = app.bundleIdentifier,
                       isSpecialPanelApp(bundleId) {
                        return bundleId
                    }
                }
            }
        }

        // Method 2: Check each special panel app directly
        for panelAppId in specialPanelApps {
            let runningApps = NSRunningApplication.runningApplications(withBundleIdentifier: panelAppId)

            for app in runningApps where app.isActive {
                return panelAppId
            }
        }

        return nil
    }

    /// Invalidate cache (call when app switch is detected)
    static func invalidateCache() {
        Cache.clear()
    }
    
    // MARK: - Smart Switch Integration
    
    /// Check if a special panel app has become active or inactive
    /// Returns: (appChanged: Bool, newBundleId: String?, isSpecialPanelApp: Bool)
    static func checkForAppChange() -> (appChanged: Bool, newBundleId: String?, isSpecialPanelApp: Bool) {
        // Check if a special panel app is currently active
        let activePanelApp = getActiveSpecialPanelApp()
        
        if let panelApp = activePanelApp {
            // A special panel app is active
            if panelApp != lastFrontMostApp {
                lastFrontMostApp = panelApp
                return (true, panelApp, true)
            }
            return (false, panelApp, true)
        }
        
        // No special panel app is active
        // If we were previously in a special panel app, we've returned to a normal app
        if isSpecialPanelApp(lastFrontMostApp) {
            let workspaceApp = NSWorkspace.shared.frontmostApplication?.bundleIdentifier
            if let app = workspaceApp {
                lastFrontMostApp = app
                return (true, app, false)
            }
        }
        
        return (false, nil, false)
    }
    
    /// Update the last frontmost app (call this when NSWorkspaceDidActivateApplicationNotification fires)
    static func updateLastFrontMostApp(_ bundleId: String) {
        lastFrontMostApp = bundleId
    }
    
    /// Get the last known frontmost app
    static func getLastFrontMostApp() -> String {
        return lastFrontMostApp
    }
}
