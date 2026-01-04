#!/usr/bin/env swift
// Debug auto-type script for GoNhanh - Creates file, types, saves, closes
// Usage: swift scripts/debug-auto-type.swift

import Foundation
import CoreGraphics
import AppKit

// Keycodes
let keycodes: [Character: UInt16] = [
    "a": 0, "s": 1, "d": 2, "f": 3, "h": 4, "g": 5, "z": 6, "x": 7, "c": 8, "v": 9,
    "b": 11, "q": 12, "w": 13, "e": 14, "r": 15, "y": 16, "t": 17, "1": 18, "2": 19,
    "3": 20, "4": 21, "6": 22, "5": 23, "9": 25, "7": 26, "8": 28, "0": 29,
    "o": 31, "u": 32, "i": 34, "p": 35, "l": 37, "j": 38, "k": 40, "n": 45, "m": 46,
    " ": 49, ",": 43, ".": 47, "[": 33, "]": 30, ":": 41, "/": 44, "-": 27, "=": 24,
    "'": 39, ";": 41, "\\": 42
]

let specialKeys: [String: UInt16] = [
    "return": 36,
    "delete": 51,
    "tab": 48,
    "escape": 53,
    "left": 123,
    "right": 124,
    "down": 125,
    "up": 126
]

let configPath = "/tmp/gonhanh_config.txt"
let outputPath = "/tmp/gonhanh_debug_\(Int(Date().timeIntervalSince1970)).txt"

var typeDelay: UInt32 = 40000  // 40ms between keys

func typeKey(_ char: Character, shift: Bool = false) {
    let isUppercase = char.isUppercase || shift
    let lowerChar = Character(char.lowercased())
    guard let keycode = keycodes[lowerChar] else { return }
    guard let source = CGEventSource(stateID: .combinedSessionState) else { return }

    if let down = CGEvent(keyboardEventSource: source, virtualKey: keycode, keyDown: true),
       let up = CGEvent(keyboardEventSource: source, virtualKey: keycode, keyDown: false) {
        if isUppercase {
            down.flags = CGEventFlags.maskShift
        }
        down.post(tap: .cghidEventTap)
        usleep(5000)
        up.post(tap: .cghidEventTap)
        usleep(typeDelay)
    }
}

func typeSpecial(_ key: String, cmd: Bool = false) {
    guard let keycode = specialKeys[key] else { return }
    guard let source = CGEventSource(stateID: .combinedSessionState) else { return }

    if let down = CGEvent(keyboardEventSource: source, virtualKey: keycode, keyDown: true),
       let up = CGEvent(keyboardEventSource: source, virtualKey: keycode, keyDown: false) {
        if cmd {
            down.flags = CGEventFlags.maskCommand
        }
        down.post(tap: .cghidEventTap)
        usleep(5000)
        up.post(tap: .cghidEventTap)
        usleep(typeDelay)
    }
}

func typeCmd(_ char: Character) {
    guard let keycode = keycodes[char] else { return }
    guard let source = CGEventSource(stateID: .combinedSessionState) else { return }

    if let down = CGEvent(keyboardEventSource: source, virtualKey: keycode, keyDown: true),
       let up = CGEvent(keyboardEventSource: source, virtualKey: keycode, keyDown: false) {
        down.flags = CGEventFlags.maskCommand
        down.post(tap: .cghidEventTap)
        usleep(5000)
        up.post(tap: .cghidEventTap)
        usleep(typeDelay * 2)
    }
}

func typeString(_ str: String) {
    for char in str {
        typeKey(char)
    }
}

func setConfig(_ config: String) {
    try? config.write(toFile: configPath, atomically: true, encoding: .utf8)
    usleep(50000)
}

func clickAtCenter() {
    // Get screen size and click at center-ish position (for TextEdit window)
    guard let source = CGEventSource(stateID: .combinedSessionState) else { return }

    // Click position - roughly center of screen where TextEdit opens
    let point = CGPoint(x: 600, y: 400)

    if let mouseDown = CGEvent(mouseEventSource: source, mouseType: .leftMouseDown, mouseCursorPosition: point, mouseButton: .left),
       let mouseUp = CGEvent(mouseEventSource: source, mouseType: .leftMouseUp, mouseCursorPosition: point, mouseButton: .left) {
        mouseDown.post(tap: .cghidEventTap)
        usleep(50000)
        mouseUp.post(tap: .cghidEventTap)
        usleep(100000)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// DEBUG TEST CASES
// ═══════════════════════════════════════════════════════════════════════════════

struct TestLine {
    let input: String       // What to type (Telex)
    let expected: String    // What should appear
    let hasDelete: Int      // Number of backspaces after
}

let testCases: [TestLine] = [
    // Basic Vietnamese
    TestLine(input: "xin chafo ", expected: "xin chào ", hasDelete: 0),
    TestLine(input: "tieengs Vieetj ", expected: "tiếng Việt ", hasDelete: 0),

    // Auto-restore English
    TestLine(input: "text ", expected: "text ", hasDelete: 0),
    TestLine(input: "window ", expected: "window ", hasDelete: 0),
    TestLine(input: "view ", expected: "view ", hasDelete: 0),

    // Mixed line
    TestLine(input: "tooi ddi work ", expected: "tôi đi work ", hasDelete: 0),

    // Delete test - type wrong then fix
    TestLine(input: "saii", expected: "sai", hasDelete: 1),  // saii → delete → sai
    TestLine(input: " roofi", expected: " rồi", hasDelete: 0),

    // Stroke test (đ)
    TestLine(input: "ddeef ", expected: "để ", hasDelete: 0),
    TestLine(input: "dduwowcj ", expected: "được ", hasDelete: 0),

    // Edge cases
    TestLine(input: "law ", expected: "law ", hasDelete: 0),
    TestLine(input: "saw ", expected: "saw ", hasDelete: 0),
    TestLine(input: "raw ", expected: "raw ", hasDelete: 0),
]

func runDebugTest() {
    print("")
    print("═══════════════════════════════════════════")
    print("     GoNhanh Debug Auto-Type")
    print("═══════════════════════════════════════════")
    print("")
    print(" Output: \(outputPath)")
    print("")
    print(" Starting in 3 seconds...")
    print(" (TextEdit will open automatically)")
    print("")

    sleep(1)

    // Open TextEdit with new file
    let task = Process()
    task.launchPath = "/usr/bin/open"
    task.arguments = ["-a", "TextEdit", outputPath]

    // Create empty file first
    FileManager.default.createFile(atPath: outputPath, contents: nil, attributes: nil)

    try? task.run()
    task.waitUntilExit()

    sleep(2)  // Wait for TextEdit to open

    // Click on TextEdit window to focus text area
    print(" Clicking on TextEdit...")
    clickAtCenter()
    usleep(200000)

    // Set config
    setConfig("textedit,8000,15000,8000")

    print(" Typing test cases...")
    print("")

    // Type header
    typeString("=== GoNhanh Debug Test ===")
    typeSpecial("return")
    typeString("Date: \(Date())")
    typeSpecial("return")
    typeSpecial("return")

    // Type each test case
    for (i, test) in testCases.enumerated() {
        // Line number
        typeString("\(i + 1). ")

        // Type input
        typeString(test.input)

        // Apply deletes if needed
        for _ in 0..<test.hasDelete {
            typeSpecial("delete")
        }

        // New line
        typeSpecial("return")

        print(" [\(i + 1)/\(testCases.count)] \"\(test.input)\" → expected: \"\(test.expected)\"")
    }

    // Footer
    typeSpecial("return")
    typeString("=== End Test ===")

    usleep(500000)  // Wait 500ms

    // Save file (Cmd+S)
    print("")
    print(" Saving...")
    typeCmd("s")

    sleep(1)

    // Read and display result
    print("")
    print("═══════════════════════════════════════════")
    print("     RESULT")
    print("═══════════════════════════════════════════")

    if let content = try? String(contentsOfFile: outputPath, encoding: .utf8) {
        print(content)
    } else {
        print(" (Could not read file - check TextEdit)")
    }

    print("")
    print(" File saved: \(outputPath)")
    print(" Done!")
}

// Main
print("")
print(" Press Enter to start (or Ctrl+C to cancel)...")
_ = readLine()

runDebugTest()
