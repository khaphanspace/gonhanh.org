#!/bin/bash
# scripts/benchmark-auto.sh
# Fully automated benchmark: opens TextEdit, types text, measures CPU/RAM
# Returns JSON results for automated analysis

set -e

# === TARGETS ===
# Realistic targets for SwiftUI menubar app:
# - SwiftUI framework: ~25MB baseline
# - CPU during typing: includes AX queries + FFI + CGEvent
TARGET_CPU_IDLE=0.5    # % CPU when idle (achievable)
TARGET_CPU_TYPING=2.0  # % CPU during fast typing
TARGET_RAM_MB=50       # MB RAM (SwiftUI baseline ~35MB)
TARGET_RAM_GROWTH=5.0  # MB growth during session (normal Swift variance)

# === OUTPUT ===
RESULTS_FILE="/tmp/gonhanh_auto_benchmark.json"

# === COLORS ===
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

echo ""
echo -e "${CYAN}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║${NC}     ${YELLOW}Gõ Nhanh Auto Benchmark (TextEdit + Real Typing)${NC}        ${CYAN}║${NC}"
echo -e "${CYAN}╚════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo "Targets:"
echo "  • CPU idle   : <${TARGET_CPU_IDLE}%"
echo "  • CPU typing : <${TARGET_CPU_TYPING}%"
echo "  • RAM total  : <${TARGET_RAM_MB}MB"
echo "  • RAM growth : <${TARGET_RAM_GROWTH}MB"
echo ""

# === CHECK GONHANH ===
PID=$(pgrep -x GoNhanh)
if [ -z "$PID" ]; then
    echo -e "${RED}ERROR: GoNhanh not running${NC}"
    exit 1
fi
echo "GoNhanh PID: $PID"

# === BASELINE METRICS ===
echo ""
echo -e "${YELLOW}═══ BASELINE ═══${NC}"
RAM_BASELINE=$(ps -p $PID -o rss= | tr -d ' ')
RAM_BASELINE_MB=$(echo "scale=2; $RAM_BASELINE / 1024" | bc)
CPU_BASELINE=$(ps -p $PID -o %cpu= | tr -d ' ')
echo "  RAM: ${RAM_BASELINE_MB}MB"
echo "  CPU: ${CPU_BASELINE}%"

# === OPEN TEXTEDIT ===
echo ""
echo -e "${YELLOW}═══ OPENING TEXTEDIT ═══${NC}"

# Close existing TextEdit if any
osascript -e 'tell application "TextEdit" to quit' 2>/dev/null || true
sleep 0.5

# Create new TextEdit document
osascript << 'EOF'
tell application "TextEdit"
    activate
    make new document
end tell
EOF
sleep 1

echo "  TextEdit opened with new document"

# === TYPING TEST ===
echo ""
echo -e "${YELLOW}═══ TYPING TEST ═══${NC}"

# Create typing script
cat > /tmp/type_test.swift << 'SWIFT'
import Foundation
import CoreGraphics

// Vietnamese keycode mapping
let keycodes: [Character: UInt16] = [
    "a": 0, "s": 1, "d": 2, "f": 3, "h": 4, "g": 5, "z": 6, "x": 7, "c": 8, "v": 9,
    "b": 11, "q": 12, "w": 13, "e": 14, "r": 15, "y": 16, "t": 17, "1": 18, "2": 19,
    "3": 20, "4": 21, "6": 22, "5": 23, "9": 25, "7": 26, "8": 28, "0": 29,
    "o": 31, "u": 32, "i": 34, "p": 35, "l": 37, "j": 38, "k": 40, "n": 45, "m": 46,
    " ": 49, ".": 47, ",": 43
]

func typeKey(_ char: Character, shift: Bool = false) {
    let lowerChar = Character(char.lowercased())
    guard let keycode = keycodes[lowerChar],
          let source = CGEventSource(stateID: .combinedSessionState),
          let down = CGEvent(keyboardEventSource: source, virtualKey: keycode, keyDown: true),
          let up = CGEvent(keyboardEventSource: source, virtualKey: keycode, keyDown: false) else { return }

    if shift || char.isUppercase {
        down.flags = .maskShift
    }

    down.post(tap: .cghidEventTap)
    usleep(2000)
    up.post(tap: .cghidEventTap)
    usleep(20000)  // 50 chars/sec = fast typing
}

func typeEnter() {
    guard let source = CGEventSource(stateID: .combinedSessionState),
          let down = CGEvent(keyboardEventSource: source, virtualKey: 36, keyDown: true),
          let up = CGEvent(keyboardEventSource: source, virtualKey: 36, keyDown: false) else { return }
    down.post(tap: .cghidEventTap)
    usleep(2000)
    up.post(tap: .cghidEventTap)
    usleep(30000)
}

// === TEST CASES ===
// 1. Pure Vietnamese
let viet1 = "Xin chafo cacs banfj, minhf laff nguwowfif Vieetj Nam."
// 2. Mixed VN + EN (auto-restore test)
let mixed = "This text should restore. Nhuwng ddaay laff tieeengs Vieetj."
// 3. Complex words
let complex = "DDuwowfng ddi ddeesp tuyeetj vowfif. Kroong Buks, DDawsk Lawsk."
// 4. Tone marks
let tones = "Safsng huyeefn hoirf ngax naawjng. Thuaajn tuys."
// 5. English patterns (should restore)
let english = "window with their wow luxury tesla life issue feature express wonderful"

// Type all test cases
for c in viet1 { typeKey(c) }
typeEnter()
for c in mixed { typeKey(c) }
typeEnter()
for c in complex { typeKey(c) }
typeEnter()
for c in tones { typeKey(c) }
typeEnter()
for c in english { typeKey(c) }
typeEnter()

// Long typing session (stress test)
let stress = "Gox Nhanh laff bowj gox tieeengs Vieetj nhanh nhaaast, ddeefp nhaaast, vaff mieenx phis."
for _ in 1...3 {
    for c in stress { typeKey(c) }
    typeEnter()
}

print("Typing complete: ~400 characters")
SWIFT

echo "  Starting typing test (~400 chars)..."
echo "  Text includes: Vietnamese, English, mixed, tones, complex words"
echo ""

# Monitor while typing
# macOS compatible millisecond timestamp
TYPING_START=$(python3 -c 'import time; print(int(time.time() * 1000))')

# Start typing in background
swift /tmp/type_test.swift 2>/dev/null &
TYPING_PID=$!

# Collect metrics during typing
CPU_SAMPLES=()
RAM_SAMPLES=()
SAMPLE_COUNT=0

echo "  Monitoring during typing:"
while kill -0 $TYPING_PID 2>/dev/null; do
    CPU=$(ps -p $PID -o %cpu= 2>/dev/null | tr -d ' ')
    RAM_KB=$(ps -p $PID -o rss= 2>/dev/null | tr -d ' ')
    RAM_MB=$(echo "scale=2; $RAM_KB / 1024" | bc)

    if [ -n "$CPU" ]; then
        CPU_SAMPLES+=($CPU)
        RAM_SAMPLES+=($RAM_MB)
        SAMPLE_COUNT=$((SAMPLE_COUNT + 1))
        printf "    [%02d] CPU: %5s%%  RAM: %sMB\n" "$SAMPLE_COUNT" "$CPU" "$RAM_MB"
    fi
    sleep 0.3
done

TYPING_END=$(python3 -c 'import time; print(int(time.time() * 1000))')
TYPING_DURATION=$((TYPING_END - TYPING_START))

# Calculate stats
CPU_SUM=0
CPU_MAX=0
RAM_MAX=0
for cpu in "${CPU_SAMPLES[@]}"; do
    CPU_SUM=$(echo "$CPU_SUM + $cpu" | bc)
    if (( $(echo "$cpu > $CPU_MAX" | bc -l) )); then
        CPU_MAX=$cpu
    fi
done

for ram in "${RAM_SAMPLES[@]}"; do
    if (( $(echo "$ram > $RAM_MAX" | bc -l) )); then
        RAM_MAX=$ram
    fi
done

if [ ${#CPU_SAMPLES[@]} -gt 0 ]; then
    CPU_AVG=$(echo "scale=2; $CPU_SUM / ${#CPU_SAMPLES[@]}" | bc)
else
    CPU_AVG=0
fi

echo ""
echo "  ─────────────────────────────────"
echo "  Duration: ${TYPING_DURATION}ms"
echo "  Samples: ${#CPU_SAMPLES[@]}"

# === POST-TYPING METRICS ===
echo ""
echo -e "${YELLOW}═══ POST-TYPING METRICS ═══${NC}"
sleep 1

RAM_AFTER=$(ps -p $PID -o rss= | tr -d ' ')
RAM_AFTER_MB=$(echo "scale=2; $RAM_AFTER / 1024" | bc)
RAM_GROWTH=$(echo "scale=2; $RAM_AFTER_MB - $RAM_BASELINE_MB" | bc)
CPU_AFTER=$(ps -p $PID -o %cpu= | tr -d ' ')

echo "  RAM after: ${RAM_AFTER_MB}MB (growth: ${RAM_GROWTH}MB)"
echo "  CPU after: ${CPU_AFTER}%"

# === IDLE TEST (5 sec) ===
echo ""
echo -e "${YELLOW}═══ IDLE TEST (5s) ═══${NC}"
IDLE_CPU_SUM=0
for i in {1..5}; do
    CPU=$(ps -p $PID -o %cpu= | tr -d ' ')
    IDLE_CPU_SUM=$(echo "$IDLE_CPU_SUM + $CPU" | bc)
    printf "  [%d] CPU: %s%%\n" "$i" "$CPU"
    sleep 1
done
IDLE_CPU_AVG=$(echo "scale=2; $IDLE_CPU_SUM / 5" | bc)

# === CLOSE TEXTEDIT ===
echo ""
echo -e "${YELLOW}═══ CLEANUP ═══${NC}"
osascript -e 'tell application "TextEdit" to quit saving no' 2>/dev/null || true
rm -f /tmp/type_test.swift
echo "  TextEdit closed"

# === RESULTS ===
echo ""
echo -e "${CYAN}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║${NC}                      ${YELLOW}BENCHMARK RESULTS${NC}                        ${CYAN}║${NC}"
echo -e "${CYAN}╚════════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Check against targets
PASS=true

check_metric() {
    local name="$1"
    local value="$2"
    local target="$3"
    local unit="$4"

    if (( $(echo "$value > $target" | bc -l) )); then
        printf "  %-20s %8s %-4s ${RED}[FAIL]${NC} (target: <%s)\n" "$name" "$value" "$unit" "$target"
        PASS=false
    else
        printf "  %-20s %8s %-4s ${GREEN}[PASS]${NC}\n" "$name" "$value" "$unit"
    fi
}

echo "  Metric               Value    Unit   Status"
echo "  ─────────────────────────────────────────────────"
check_metric "CPU (typing avg)" "$CPU_AVG" "$TARGET_CPU_TYPING" "%"
check_metric "CPU (typing peak)" "$CPU_MAX" "3.0" "%"
check_metric "CPU (idle)" "$IDLE_CPU_AVG" "$TARGET_CPU_IDLE" "%"
check_metric "RAM (peak)" "$RAM_MAX" "$TARGET_RAM_MB" "MB"
check_metric "RAM (baseline)" "$RAM_BASELINE_MB" "$TARGET_RAM_MB" "MB"
check_metric "RAM (growth)" "$RAM_GROWTH" "$TARGET_RAM_GROWTH" "MB"
echo ""

# Save JSON
cat > "$RESULTS_FILE" << EOF
{
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "targets": {
    "cpu_idle_percent": $TARGET_CPU_IDLE,
    "cpu_typing_percent": $TARGET_CPU_TYPING,
    "ram_mb": $TARGET_RAM_MB,
    "ram_growth_mb": $TARGET_RAM_GROWTH
  },
  "baseline": {
    "ram_mb": $RAM_BASELINE_MB,
    "cpu_percent": $CPU_BASELINE
  },
  "typing": {
    "duration_ms": $TYPING_DURATION,
    "samples": ${#CPU_SAMPLES[@]},
    "cpu_avg": $CPU_AVG,
    "cpu_peak": $CPU_MAX,
    "ram_peak_mb": $RAM_MAX
  },
  "post_typing": {
    "ram_mb": $RAM_AFTER_MB,
    "ram_growth_mb": $RAM_GROWTH,
    "cpu_percent": $CPU_AFTER
  },
  "idle": {
    "cpu_avg": $IDLE_CPU_AVG
  },
  "passed": $PASS
}
EOF

echo "  Results: $RESULTS_FILE"
echo ""

if [ "$PASS" = true ]; then
    echo -e "${GREEN}╔════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║              ALL TARGETS MET - BENCHMARK PASSED ✓              ║${NC}"
    echo -e "${GREEN}╚════════════════════════════════════════════════════════════════╝${NC}"
    exit 0
else
    echo -e "${RED}╔════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${RED}║              TARGETS NOT MET - OPTIMIZATION NEEDED             ║${NC}"
    echo -e "${RED}╚════════════════════════════════════════════════════════════════╝${NC}"
    exit 1
fi
