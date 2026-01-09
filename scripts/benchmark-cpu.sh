#!/bin/bash
# CPU Benchmark for GoNhanh
# Measures CPU usage during idle and typing

echo "=== GoNhanh CPU Benchmark ==="
echo ""

# Get PID
PID=$(pgrep -x GoNhanh)
if [ -z "$PID" ]; then
    echo "Error: GoNhanh not running"
    exit 1
fi
echo "GoNhanh PID: $PID"

# Measure idle CPU (5 samples, 1 sec apart)
echo ""
echo "1. Measuring IDLE CPU (5 seconds)..."
IDLE_CPU=0
for i in {1..5}; do
    CPU=$(ps -p $PID -o %cpu= 2>/dev/null | tr -d ' ')
    IDLE_CPU=$(echo "$IDLE_CPU + $CPU" | bc)
    sleep 1
done
IDLE_AVG=$(echo "scale=2; $IDLE_CPU / 5" | bc)
echo "   Idle average: ${IDLE_AVG}%"

# Now run typing test in background while measuring
echo ""
echo "2. Measuring TYPING CPU..."
echo "   Starting typing test..."

# Create a simple test file that types to /dev/null equivalent
cat > /tmp/typing_test.swift << 'EOF'
import Foundation
import CoreGraphics

let keycodes: [Character: UInt16] = [
    "a": 0, "s": 1, "d": 2, "f": 3, "h": 4, "g": 5, "z": 6, "x": 7, "c": 8, "v": 9,
    "b": 11, "q": 12, "w": 13, "e": 14, "r": 15, "y": 16, "t": 17,
    "o": 31, "u": 32, "i": 34, "p": 35, "l": 37, "j": 38, "k": 40, "n": 45, "m": 46, " ": 49
]

func typeKey(_ char: Character) {
    let lowerChar = Character(char.lowercased())
    guard let keycode = keycodes[lowerChar],
          let source = CGEventSource(stateID: .combinedSessionState),
          let down = CGEvent(keyboardEventSource: source, virtualKey: keycode, keyDown: true),
          let up = CGEvent(keyboardEventSource: source, virtualKey: keycode, keyDown: false) else { return }
    down.post(tap: .cghidEventTap)
    usleep(3000)
    up.post(tap: .cghidEventTap)
    usleep(30000)  // 30ms = ~33 chars/sec
}

// Vietnamese + English mixed text for realistic benchmark
let text = "Chafo cacs banfj, minhf ddang tesst Gox Nhanh. Smart auto restore: text, expect, perfect, window, with, their, wow, luxury, tesla, life, issue, feature, express, wonderful, support, core, care, saas, sax, push, work, hard, user. Per app memory: VS Code, Slack. Auto disable: Japanese, Korean, Chinese. DDawsk Lawsk, DDawsk Noong, Kroong Buks. Thanks for your wonderful support with thiss software."
for c in text { typeKey(c) }
EOF

# Run typing test in background
swift /tmp/typing_test.swift 2>/dev/null &
TYPING_PID=$!

# Measure CPU while typing
TYPING_CPU=0
SAMPLES=0
while kill -0 $TYPING_PID 2>/dev/null; do
    CPU=$(ps -p $PID -o %cpu= 2>/dev/null | tr -d ' ')
    if [ -n "$CPU" ] && [ "$CPU" != "0.0" ]; then
        TYPING_CPU=$(echo "$TYPING_CPU + $CPU" | bc)
        SAMPLES=$((SAMPLES + 1))
    fi
    sleep 0.5
done

if [ $SAMPLES -gt 0 ]; then
    TYPING_AVG=$(echo "scale=2; $TYPING_CPU / $SAMPLES" | bc)
else
    TYPING_AVG="N/A"
fi
echo "   Typing average: ${TYPING_AVG}% (${SAMPLES} samples)"

# Summary
echo ""
echo "=== RESULTS ==="
echo "Idle CPU:   ${IDLE_AVG}%"
echo "Typing CPU: ${TYPING_AVG}%"
echo ""

# Calculate overhead
if [ "$TYPING_AVG" != "N/A" ]; then
    OVERHEAD=$(echo "scale=2; $TYPING_AVG - $IDLE_AVG" | bc)
    echo "Overhead:   ${OVERHEAD}%"
fi

# Cleanup
rm -f /tmp/typing_test.swift
