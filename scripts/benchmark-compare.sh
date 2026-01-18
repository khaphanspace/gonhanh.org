#!/bin/bash
# Benchmark Compare: PR version vs Original version
# Fully automated - no user interaction required
# Results saved to CSV

set -e

TIMESTAMP=$(date +%Y%m%d-%H%M%S)
RESULT_CSV="/tmp/benchmark-results-${TIMESTAMP}.csv"
RESULT_LOG="/tmp/benchmark-results-${TIMESTAMP}.log"
RUNS=3

# Get project directory (parent of scripts/)
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

log() {
    echo "$1" | tee -a "$RESULT_LOG"
}

# Initialize CSV
echo "version,run,idle_cpu,typing_cpu,overhead,samples,timestamp" > "$RESULT_CSV"

log "=== GoNhanh Benchmark Compare ==="
log "Date: $(date)"
log "CSV: $RESULT_CSV"
log ""

# Typing test Swift code
create_typing_test() {
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
    usleep(30000)
}

let text = "Chafo cacs banfj, minhf ddang tesst Gox Nhanh. Smart auto restore: text, expect, perfect, window, with, their, wow, luxury, tesla, life, issue, feature, express, wonderful, support, core, care, saas, sax, push, work, hard, user. Per app memory: VS Code, Slack. Auto disable: Japanese, Korean, Chinese. DDawsk Lawsk, DDawsk Noong, Kroong Buks. Thanks for your wonderful support with thiss software."
for c in text { typeKey(c) }
EOF
}

# Run single benchmark and append to CSV
run_benchmark() {
    local PID=$1
    local RUN_NUM=$2
    local VERSION=$3

    # Measure idle (3 seconds)
    IDLE_CPU=0
    for i in {1..3}; do
        CPU=$(ps -p $PID -o %cpu= 2>/dev/null | tr -d ' ')
        IDLE_CPU=$(echo "$IDLE_CPU + $CPU" | bc)
        sleep 1
    done
    IDLE_AVG=$(echo "scale=2; $IDLE_CPU / 3" | bc)

    # Run typing test
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
        OVERHEAD=$(echo "scale=2; $TYPING_AVG - $IDLE_AVG" | bc)
    else
        TYPING_AVG="0"
        OVERHEAD="0"
    fi

    # Append to CSV
    echo "${VERSION},${RUN_NUM},${IDLE_AVG},${TYPING_AVG},${OVERHEAD},${SAMPLES},$(date +%H:%M:%S)" >> "$RESULT_CSV"

    log "${VERSION} Run ${RUN_NUM}: Idle=${IDLE_AVG}% Typing=${TYPING_AVG}% Overhead=${OVERHEAD}%"
}

cd "$PROJECT_DIR"

# Create typing test
create_typing_test

# Save current branch
CURRENT_BRANCH=$(git branch --show-current)
log "Current branch: $CURRENT_BRANCH"

# ============ PR VERSION ============
log ""
log "=== BUILDING PR VERSION ==="

make build > /tmp/build.log 2>&1 || { log "Build failed!"; exit 1; }
log "Build: OK"

pkill -9 GoNhanh 2>/dev/null || true
sleep 1
rm -rf /tmp/GoNhanh-PR.app 2>/dev/null || true
cp -r platforms/macos/build/Release/GoNhanh.app /tmp/GoNhanh-PR.app

log ""
log "=== TESTING PR VERSION ==="
open /tmp/GoNhanh-PR.app
sleep 3

PID=$(pgrep -x GoNhanh)
[ -z "$PID" ] && { log "Error: GoNhanh not running"; exit 1; }
log "PID: $PID"

for i in $(seq 1 $RUNS); do
    run_benchmark $PID $i "PR"
    sleep 1
done

# ============ ORIGINAL VERSION ============
log ""
log "=== BUILDING ORIGINAL VERSION ==="

git stash 2>/dev/null || true
git checkout main 2>/dev/null

make build > /tmp/build.log 2>&1 || { log "Build failed!"; exit 1; }
log "Build: OK"

pkill -9 GoNhanh 2>/dev/null || true
sleep 1
cp -r platforms/macos/build/Release/GoNhanh.app /Applications/

log ""
log "=== TESTING ORIGINAL VERSION ==="
open /Applications/GoNhanh.app
sleep 3

PID=$(pgrep -x GoNhanh)
[ -z "$PID" ] && { log "Error: GoNhanh not running"; exit 1; }
log "PID: $PID"

for i in $(seq 1 $RUNS); do
    run_benchmark $PID $i "ORIGINAL"
    sleep 1
done

# ============ RESTORE ============
log ""
log "=== RESTORING ==="
git checkout "$CURRENT_BRANCH" 2>/dev/null || true
git stash pop 2>/dev/null || true
log "Restored: $CURRENT_BRANCH"

# ============ SUMMARY ============
log ""
log "=== CSV CONTENT ==="
cat "$RESULT_CSV" | tee -a "$RESULT_LOG"

log ""
log "=== AVERAGES ==="
PR_AVG=$(awk -F',' '$1=="PR" {sum+=$4; count++} END {printf "%.2f", sum/count}' "$RESULT_CSV")
ORIG_AVG=$(awk -F',' '$1=="ORIGINAL" {sum+=$4; count++} END {printf "%.2f", sum/count}' "$RESULT_CSV")
DIFF=$(echo "scale=2; $PR_AVG - $ORIG_AVG" | bc)

log "PR Average:       ${PR_AVG}%"
log "Original Average: ${ORIG_AVG}%"
log "Difference:       ${DIFF}%"

log ""
log "=== DONE ==="
log "CSV: $RESULT_CSV"
log "Log: $RESULT_LOG"

# Cleanup
rm -f /tmp/typing_test.swift

echo ""
echo "Results saved to: $RESULT_CSV"
