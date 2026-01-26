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

// ═══════════════════════════════════════════════════════════════════════════════
// TEST PATTERNS - Covers README message + difficult Vietnamese patterns from docs
// ═══════════════════════════════════════════════════════════════════════════════
//
// Core message (from README.md):
//   - Gõ Nhanh: bộ gõ tiếng Việt miễn phí, nhanh, ổn định
//   - Đặt dấu đúng vị trí (hoà, khoẻ, thuỷ)
//   - Nhận diện tiếng Anh tự động
//
// Hard patterns (from vietnamese-language-system.md):
//   - ươ compound: được, người, nước, trường
//   - ưu cluster: lưu, cứu, ngưu, hưu
//   - ưa pattern: mưa, cửa, lửa
//   - Triple vowels: tuổi, mười, tươi, hươu, rượu, quyền, khuyên, chuyện
//   - New tone rules: hoà, khoẻ, thuỷ (vs old: hòa, khỏe, thủy)
//   - Stop finals (-p/-t/-c/-ch + sắc/nặng only): cấp, tập, mát, mạt
//   - ng/ngh rules: nghe, nghĩ, nghỉ (ngh before e,ê,i)
//   - g/gh rules: ghế (gh before e,ê,i)
//   - English detection: Windows, variable, function (no diacritics added)
//
// ═══════════════════════════════════════════════════════════════════════════════

// TELEX: aa=â, ee=ê, oo=ô, aw=ă, ow=ơ, uw=ư, dd=đ | s=sắc, f=huyền, r=hỏi, x=ngã, j=nặng
// Examples: người=nguwowif, được=dduwowcj, quyền=quyeenf, khuyên=khuyeen
// Cancel: varriable (double r cancels hỏi mark) → variable
let telexInput = "Gox Nhanh laf booj gox tieengs Vieetj mieenx phis nhanh oonr ddinhj. DDuwowcj phats trieenr danhf tawngj coongj ddoongf nguwowif dungf Vieetj Nam. DDawtj daaus ddungs vij tris hoaf khoer thuyr. Nhaanj dieen tieengs Anh tuwj ddoongj Wwindows varriable function. Tuooir muwowif tuwowif nhuw huwowu ruwowuj. Cuoois cungf cuwra luwra muwa nuwowcs. Caacs caaps taapj mats matj. Nghe nghix nghir ngowi ghees. Luwu cuwsu nguwu huwu. Quyeenf khuyeen quyeets chuyeenj."

// VNI: 6=^(â,ê,ô), 7=ơ/ư, 8=ă, 9=đ | 1=sắc, 2=huyền, 3=hỏi, 4=ngã, 5=nặng
// Examples: người=ngu7o72i, được=d9u7o75c, quyền=quye62n, khuyên=khuye6n
let vniInput = "Go4 Nhanh la2 bo65 go4 tie61ng Vie65t mie64n phi1 nhanh o63n d9i5nh. D9u7o75c pha1t trie63n da2nh ta85ng co65ng d9o62ng ngu7o72i du2ng Vie65t Nam. D9a85t da61u d9u1ng vi5 tri1 hoa2 khoe3 thuy3. Nha65n die65n tie61ng Anh tu75 d9o65ng Windows variable function. Tuo63i mu7o72i tu7o7i nhu7 hu7o7u ru7o75u. Cuo61i cu2ng cu73a lu73a mu7a nu7o71c. Ca1c ca61p ta65p ma1t ma5t. Nghe nghi4 nghi3 ngo7i ghe61. Lu7u cu71u ngu7u hu7u. Quye62n khuye6n quye61t chuye65n."

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
