#!/usr/bin/env swift
// Vietnamese typing test - Marks at end pattern (gõ dấu sau)
// Example: tiếng = tienges (type full word, then add ee→ê, s→sắc at end)

import Foundation
import CoreGraphics

let keycodes: [Character: UInt16] = [
    "a": 0, "s": 1, "d": 2, "f": 3, "h": 4, "g": 5, "z": 6, "x": 7, "c": 8, "v": 9,
    "b": 11, "q": 12, "w": 13, "e": 14, "r": 15, "y": 16, "t": 17, "1": 18, "2": 19,
    "3": 20, "4": 21, "6": 22, "5": 23, "9": 25, "7": 26, "8": 28, "0": 29,
    "o": 31, "u": 32, "i": 34, "p": 35, "l": 37, "j": 38, "k": 40, "n": 45, "m": 46,
    " ": 49, ",": 43, ".": 47, "[": 33, "]": 30, ":": 41, "/": 44
]

let configPath = "/tmp/gonhanh_config.txt"

var typeDelay: UInt32 = 30000  // 30ms between keys (adjustable per config)

func typeKey(_ char: Character) {
    guard let keycode = keycodes[char] else { return }
    guard let source = CGEventSource(stateID: .combinedSessionState) else { return }
    if let down = CGEvent(keyboardEventSource: source, virtualKey: keycode, keyDown: true),
       let up = CGEvent(keyboardEventSource: source, virtualKey: keycode, keyDown: false) {
        down.post(tap: .cghidEventTap)
        usleep(5000)
        up.post(tap: .cghidEventTap)
        usleep(typeDelay)
    }
}

func typeString(_ str: String) {
    for char in str.lowercased() {
        typeKey(char)
    }
}

func setConfig(_ config: String) {
    try? config.write(toFile: configPath, atomically: true, encoding: .utf8)
    usleep(50000) // 50ms for config to take effect
}

// ═══════════════════════════════════════════════════════════════════════════════
// TELEX - Marks at end pattern (gõ dấu sau)
// ═══════════════════════════════════════════════════════════════════════════════
//
// Pattern: type word first, add marks at end
// - tienges → tiếng (e→ê, s→sắc)
// - vieetj → việt (ee→ê, j→nặng)
// - duocwwj → được (ww→ươ, j→nặng)
// - nguoiwwif → người (ww→ươ, f→huyền)
//
// TELEX marks: aa=â, ee=ê, oo=ô, aw=ă, ow=ơ, uw=ư, dd=đ
// TELEX tones: s=sắc, f=huyền, r=hỏi, x=ngã, j=nặng
//
// ═══════════════════════════════════════════════════════════════════════════════

// Full sentence test - marks at end style (gõ dấu sau toàn bộ)
let telexInput = "Gox Nhanh laf booj gox tieengs Vieetj mieenx phis nhanh oonr ddinhj. Luuw laij nuocws tuooir muowif. DDuowcj nguowif tuwowi."
// Expected: "Gõ Nhanh là bộ gõ tiếng Việt miễn phí nhanh ổn định. Lưu lại nước tuổi mười. Được người tươi."

// Test configs: name, config value, test typing delay (µs)
let testConfigs: [(String, String, UInt32)] = [
    ("60ms", "electron,8000,25000,8000", 60000),
    ("70ms", "electron,8000,30000,8000", 70000),
    ("80ms", "electron,10000,30000,10000", 80000),
]

func runTest() {
    print("")
    print(" Mode: TELEX - Marks at end")
    print(" Click vào input field ngay!")
    print("")
    print(" 3..."); sleep(1)
    print(" 2..."); sleep(1)
    print(" 1..."); sleep(1)

    for (configName, configValue, delay) in testConfigs {
        setConfig(configValue)
        typeDelay = delay

        print(" [\(configName.uppercased())] Đang gõ (delay: \(delay/1000)ms)...")

        // Type prefix with config
        typeString("[\(configName);\(configValue.replacingOccurrences(of: "electron,", with: ""))] ")

        // Type test input
        for char in telexInput.lowercased() {
            typeKey(char)
        }

        // Add spacing between tests
        typeString("   ")

        usleep(500000) // 500ms pause between tests
    }

    print(" Xong!")
}

// Main loop
while true {
    print("")
    print("══════════════════════════════════════════")
    print("   GoNhanh - Test Marks at End Pattern")
    print("══════════════════════════════════════════")
    print("")
    print("  [1] Run test (Telex - gõ dấu sau)")
    print("  [q] Quit")
    print("")
    print("Chọn: ", terminator: "")

    guard let input = readLine()?.trimmingCharacters(in: .whitespaces).lowercased() else { continue }

    switch input {
    case "1":
        runTest()
    case "q", "quit", "exit":
        print(" Bye!")
        exit(0)
    default:
        print(" Chọn 1 hoặc q")
    }
}
