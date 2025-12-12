#!/usr/bin/env swift
// Vietnamese typing test for GoNhanh - Tests medium and fast speeds

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

// TELEX: aa=â, ee=ê, oo=ô, aw=ă, ow=ơ, uw=ư, dd=đ | s=sắc, f=huyền, r=hỏi, x=ngã, j=nặng
// người=nguwowif, được=dduwowcj, trường=truwowngf, bình=binhf, định=ddinhj
// variable=varriable (varr để cancel dấu hỏi)
let telexInput = "vieejt nam xin chaof camr own nguwowif dduwowcj truwowngf khoong thaatj tooi laf developer code vieejt nam debug laf fix bugs nguwowngj khuyeenr chuyeenj quyeets ddinhj thuwowngf xuyeen hoaf binhf giaos ducj kinh tees xin looix tooi khoong bieets function nayf return gias trij varriable tuwj ddoongj"

// VNI: 6=^(â,ê,ô), 7=ơ/ư, 8=ă, 9=đ | 1=sắc, 2=huyền, 3=hỏi, 4=ngã, 5=nặng
let vniInput = "vie65t nam xin cha2o ca3m o7n ngu7o72i d9u7o75c tru7o72ng kho6ng tha65t to6i la2 developer code vie65t nam debug la2 fix bugs ngu7o75ng khuye63n chuye65n quye61t d9i5nh thu7o72ng xuye6n ho2a bi2nh gia1o du5c kinh te61 xin lo64i, to6i kho6ng bie61t. function na2y return gia1 tri5 variable tu75 d9o65ng"

// Test configs: name, config value, test typing delay (µs)
let testConfigs: [(String, String, UInt32)] = [
    ("40ms", "electron,8000,15000,8000", 40000),
    ("50ms", "electron,8000,15000,8000", 50000),
    ("60ms", "electron,8000,15000,8000", 60000),
]

func runTest(mode: String) {
    let testInput = mode == "vni" ? vniInput : telexInput

    print("")
    print(" Mode: \(mode.uppercased())")
    print(" Click vào input field ngay!")
    print("")
    print(" 3..."); sleep(1)
    print(" 2..."); sleep(1)
    print(" 1..."); sleep(1)

    for (configName, configValue, delay) in testConfigs {
        setConfig(configValue)
        typeDelay = delay

        print(" [\(configName.uppercased())] Đang gõ (delay: \(delay/1000)ms)...")

        // Type prefix with mode and config: [telex;fast;8000,15000,8000]
        typeString("[\(mode);\(configName);\(configValue.replacingOccurrences(of: "electron,", with: ""))] ")

        // Type test input
        for char in testInput.lowercased() {
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
    print("       GoNhanh Typing Test")
    print("══════════════════════════════════════════")
    print("")
    print("  [1] Telex")
    print("  [2] VNI")
    print("  [q] Quit")
    print("")
    print("Chọn: ", terminator: "")

    guard let input = readLine()?.trimmingCharacters(in: .whitespaces).lowercased() else { continue }

    switch input {
    case "1":
        runTest(mode: "telex")
    case "2":
        runTest(mode: "vni")
    case "q", "quit", "exit":
        print(" Bye!")
        exit(0)
    default:
        print(" Chọn 1, 2 hoặc q")
    }
}
