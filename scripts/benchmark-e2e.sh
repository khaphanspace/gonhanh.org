#!/bin/bash
# scripts/benchmark-e2e.sh
# Comprehensive E2E benchmark with per-step timing and CPU/RAM monitoring
# Exit code: 0 = PASS, 1 = FAIL

set -e

# === COLORS ===
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# === THRESHOLDS ===
MAX_IDLE_CPU=1.0          # % CPU when idle
MAX_TYPING_CPU=5.0        # % CPU when typing
MAX_MEMORY_GROWTH_MB=2    # MB growth over session
MAX_P99_LATENCY_MS=1.0    # ms for 99th percentile

# === OUTPUT FILES ===
RESULTS_JSON="/tmp/gonhanh_benchmark_results.json"
PERF_DATA="/tmp/gonhanh_perf_data.jsonl"
STEP_LOG="/tmp/gonhanh_benchmark_steps.log"

# === HELPER FUNCTIONS ===
timestamp() {
    date "+%H:%M:%S.%3N"
}

log_step() {
    local step_name="$1"
    local status="$2"
    local duration="$3"
    local details="$4"
    echo "$(timestamp) | $step_name | $status | ${duration}ms | $details" >> "$STEP_LOG"
    printf "${CYAN}[%s]${NC} %-25s %s\n" "$(timestamp)" "$step_name" "$details"
}

get_cpu() {
    ps -p $PID -o %cpu= 2>/dev/null | tr -d ' ' || echo "0"
}

get_ram_mb() {
    local rss=$(ps -p $PID -o rss= 2>/dev/null | tr -d ' ')
    if [ -n "$rss" ]; then
        echo "scale=2; $rss / 1024" | bc
    else
        echo "0"
    fi
}

# === START ===
echo ""
echo -e "${BLUE}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║${NC}          ${YELLOW}Gõ Nhanh E2E Performance Benchmark${NC}                ${BLUE}║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo "Thresholds:"
echo "  • CPU (idle)   : <${MAX_IDLE_CPU}%"
echo "  • CPU (typing) : <${MAX_TYPING_CPU}%"
echo "  • Memory growth: <${MAX_MEMORY_GROWTH_MB}MB"
echo "  • P99 latency  : <${MAX_P99_LATENCY_MS}ms"
echo ""

# Initialize step log
echo "timestamp | step | status | duration_ms | details" > "$STEP_LOG"

START_TIME=$(date +%s%3N)

# === STEP 1: CHECK GONHANH PROCESS ===
log_step "1. Process Check" "START" "0" "Finding GoNhanh..."
PID=$(pgrep -x GoNhanh)
if [ -z "$PID" ]; then
    log_step "1. Process Check" "FAIL" "0" "GoNhanh not running!"
    echo -e "${RED}ERROR: GoNhanh is not running. Start it first.${NC}"
    exit 1
fi
log_step "1. Process Check" "OK" "0" "PID=$PID"

# === STEP 2: INITIAL METRICS ===
log_step "2. Initial Metrics" "START" "0" "Capturing baseline..."

CPU_INIT=$(get_cpu)
RAM_INIT=$(get_ram_mb)

echo ""
echo -e "${YELLOW}═══ INITIAL STATE ═══${NC}"
echo "  PID      : $PID"
echo "  CPU      : ${CPU_INIT}%"
echo "  RAM      : ${RAM_INIT}MB"
echo ""

log_step "2. Initial Metrics" "OK" "0" "CPU=${CPU_INIT}% RAM=${RAM_INIT}MB"

# === STEP 3: IDLE CPU MEASUREMENT ===
log_step "3. Idle CPU Test" "START" "0" "Measuring 5 seconds..."
echo -e "${YELLOW}═══ IDLE CPU TEST (5 samples) ═══${NC}"

IDLE_CPU_TOTAL=0
IDLE_SAMPLES=()
for i in {1..5}; do
    CPU=$(get_cpu)
    RAM=$(get_ram_mb)
    IDLE_SAMPLES+=($CPU)
    IDLE_CPU_TOTAL=$(echo "$IDLE_CPU_TOTAL + $CPU" | bc)
    printf "  Sample %d: CPU=%5s%%  RAM=%sMB\n" "$i" "$CPU" "$RAM"
    sleep 1
done

IDLE_AVG=$(echo "scale=2; $IDLE_CPU_TOTAL / 5" | bc)
echo "  ─────────────────────"
echo -e "  ${GREEN}Average: ${IDLE_AVG}%${NC}"
echo ""

log_step "3. Idle CPU Test" "OK" "5000" "avg=${IDLE_AVG}%"

# === STEP 4: TYPING TEST ===
log_step "4. Typing Test" "START" "0" "Preparing typing simulator..."
echo -e "${YELLOW}═══ TYPING TEST ═══${NC}"

# Create typing simulator with detailed text
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
    usleep(25000)  // ~40 chars/sec
}

// Mixed Vietnamese + English text (realistic typing patterns)
let text = "Chafo cacs banfj, minhf ddang test Gox Nhanh. Smart auto restore: text, expect, perfect, window, with, their, wow, luxury, tesla, life, issue, feature, express, wonderful, support, core, care, saas, sax, push, work, hard, user. DDawsk Lawsk Kroong. Thanks for your support."
print("Typing \(text.count) characters...")
for c in text { typeKey(c) }
print("Done")
EOF

echo "  Text: 280+ chars (Vietnamese + English mixed)"
echo "  Speed: ~40 chars/sec"
echo ""
echo "  Monitoring during typing:"

TYPING_START=$(date +%s%3N)

# Run typing in background
swift /tmp/typing_test.swift 2>/dev/null &
TYPING_PID=$!

# Monitor CPU/RAM while typing
TYPING_CPU_TOTAL=0
TYPING_SAMPLES=0
PEAK_CPU=0
PEAK_RAM=0

while kill -0 $TYPING_PID 2>/dev/null; do
    CPU=$(get_cpu)
    RAM=$(get_ram_mb)

    if [ -n "$CPU" ] && [ "$CPU" != "0.0" ]; then
        TYPING_CPU_TOTAL=$(echo "$TYPING_CPU_TOTAL + $CPU" | bc)
        TYPING_SAMPLES=$((TYPING_SAMPLES + 1))

        # Track peaks
        if (( $(echo "$CPU > $PEAK_CPU" | bc -l) )); then
            PEAK_CPU=$CPU
        fi
        if (( $(echo "$RAM > $PEAK_RAM" | bc -l) )); then
            PEAK_RAM=$RAM
        fi

        printf "    [%02d] CPU=%5s%% RAM=%sMB\n" "$TYPING_SAMPLES" "$CPU" "$RAM"
    fi
    sleep 0.5
done

TYPING_END=$(date +%s%3N)
TYPING_DURATION=$((TYPING_END - TYPING_START))

if [ $TYPING_SAMPLES -gt 0 ]; then
    TYPING_AVG=$(echo "scale=2; $TYPING_CPU_TOTAL / $TYPING_SAMPLES" | bc)
else
    TYPING_AVG="0"
fi

echo "  ─────────────────────"
echo -e "  ${GREEN}Average CPU: ${TYPING_AVG}%${NC}"
echo "  Peak CPU  : ${PEAK_CPU}%"
echo "  Peak RAM  : ${PEAK_RAM}MB"
echo "  Duration  : ${TYPING_DURATION}ms"
echo ""

log_step "4. Typing Test" "OK" "$TYPING_DURATION" "avg=${TYPING_AVG}% peak=${PEAK_CPU}%"

# === STEP 5: MEMORY CHECK ===
log_step "5. Memory Check" "START" "0" "Checking memory growth..."
echo -e "${YELLOW}═══ MEMORY ANALYSIS ═══${NC}"

sleep 1  # Allow GC/cleanup

RAM_AFTER=$(get_ram_mb)
RAM_GROWTH=$(echo "scale=2; $RAM_AFTER - $RAM_INIT" | bc)

echo "  Before typing: ${RAM_INIT}MB"
echo "  After typing : ${RAM_AFTER}MB"
echo "  Growth       : ${RAM_GROWTH}MB"
echo ""

log_step "5. Memory Check" "OK" "1000" "before=${RAM_INIT}MB after=${RAM_AFTER}MB growth=${RAM_GROWTH}MB"

# === STEP 6: LATENCY ANALYSIS (from PerfMetrics if available) ===
log_step "6. Latency Analysis" "START" "0" "Parsing PerfMetrics..."
echo -e "${YELLOW}═══ LATENCY ANALYSIS ═══${NC}"

DETECT_AVG="N/A"; DETECT_P99="N/A"
IME_AVG="N/A"; IME_P99="N/A"
INJECT_AVG="N/A"; INJECT_P99="N/A"
TOTAL_AVG="N/A"; TOTAL_P99="N/A"
BOTTLENECK="unknown"

if [ -f "$PERF_DATA" ] && [ -s "$PERF_DATA" ]; then
    if command -v jq &> /dev/null; then
        # Parse JSONL data
        DETECT_VALS=$(jq -r '.detectMethod // empty' "$PERF_DATA" 2>/dev/null | sort -n)
        IME_VALS=$(jq -r '.ime_key_ext // empty' "$PERF_DATA" 2>/dev/null | sort -n)
        INJECT_VALS=$(jq -r '.injectSync // empty' "$PERF_DATA" 2>/dev/null | sort -n)
        TOTAL_VALS=$(jq -r '.total // empty' "$PERF_DATA" 2>/dev/null | sort -n)

        if [ -n "$DETECT_VALS" ]; then
            DETECT_AVG=$(echo "$DETECT_VALS" | awk '{sum+=$1} END {printf "%.3f", sum/NR}')
            DETECT_COUNT=$(echo "$DETECT_VALS" | wc -l | tr -d ' ')
            DETECT_P99=$(echo "$DETECT_VALS" | tail -n $((DETECT_COUNT * 1 / 100 + 1)) | head -1)
        fi

        if [ -n "$IME_VALS" ]; then
            IME_AVG=$(echo "$IME_VALS" | awk '{sum+=$1} END {printf "%.3f", sum/NR}')
            IME_COUNT=$(echo "$IME_VALS" | wc -l | tr -d ' ')
            IME_P99=$(echo "$IME_VALS" | tail -n $((IME_COUNT * 1 / 100 + 1)) | head -1)
        fi

        if [ -n "$INJECT_VALS" ]; then
            INJECT_AVG=$(echo "$INJECT_VALS" | awk '{sum+=$1} END {printf "%.3f", sum/NR}')
            INJECT_COUNT=$(echo "$INJECT_VALS" | wc -l | tr -d ' ')
            INJECT_P99=$(echo "$INJECT_VALS" | tail -n $((INJECT_COUNT * 1 / 100 + 1)) | head -1)
        fi

        if [ -n "$TOTAL_VALS" ]; then
            TOTAL_AVG=$(echo "$TOTAL_VALS" | awk '{sum+=$1} END {printf "%.3f", sum/NR}')
            TOTAL_COUNT=$(echo "$TOTAL_VALS" | wc -l | tr -d ' ')
            TOTAL_P99=$(echo "$TOTAL_VALS" | tail -n $((TOTAL_COUNT * 1 / 100 + 1)) | head -1)
        fi

        # Find bottleneck
        DETECT_NUM=${DETECT_AVG:-0}
        IME_NUM=${IME_AVG:-0}
        INJECT_NUM=${INJECT_AVG:-0}
        MAX_AVG=$(echo "$DETECT_NUM $IME_NUM $INJECT_NUM" | tr ' ' '\n' | sort -rn | head -1)
        if [ "$MAX_AVG" = "$DETECT_NUM" ]; then BOTTLENECK="detectMethod"
        elif [ "$MAX_AVG" = "$IME_NUM" ]; then BOTTLENECK="ime_key_ext"
        elif [ "$MAX_AVG" = "$INJECT_NUM" ]; then BOTTLENECK="injectSync"
        fi

        echo "  ┌──────────────────────────────────────┐"
        echo "  │  Step            Avg(ms)    P99(ms)  │"
        echo "  ├──────────────────────────────────────┤"
        printf "  │  detectMethod    %-9s %-9s│\n" "$DETECT_AVG" "$DETECT_P99"
        printf "  │  ime_key_ext     %-9s %-9s│\n" "$IME_AVG" "$IME_P99"
        printf "  │  injectSync      %-9s %-9s│\n" "$INJECT_AVG" "$INJECT_P99"
        echo "  ├──────────────────────────────────────┤"
        printf "  │  TOTAL           %-9s %-9s│\n" "$TOTAL_AVG" "$TOTAL_P99"
        echo "  └──────────────────────────────────────┘"
        echo ""
        echo "  Bottleneck: $BOTTLENECK"
    else
        echo "  WARNING: jq not installed, skipping detailed analysis"
    fi
else
    echo "  No PerfMetrics data available"
    echo "  (Enable PerfMetrics in app for per-step breakdown)"
fi
echo ""

log_step "6. Latency Analysis" "OK" "0" "bottleneck=$BOTTLENECK"

# === STEP 7: RESULTS SUMMARY ===
END_TIME=$(date +%s%3N)
TOTAL_DURATION=$((END_TIME - START_TIME))

echo -e "${BLUE}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║${NC}                    ${YELLOW}BENCHMARK RESULTS${NC}                        ${BLUE}║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""

PASS=true
FAILURES=""

# Check each threshold
check_threshold() {
    local name="$1"
    local value="$2"
    local max="$3"
    local unit="$4"

    if [ "$value" = "N/A" ]; then
        printf "  %-18s %10s %s  ${YELLOW}[SKIP]${NC}\n" "$name" "$value" "$unit"
        return
    fi

    if (( $(echo "$value > $max" | bc -l) )); then
        printf "  %-18s %10s %s  ${RED}[FAIL]${NC} (max: %s)\n" "$name" "$value" "$unit" "$max"
        PASS=false
        FAILURES="${FAILURES}• $name: $value$unit > $max$unit\n"
    else
        printf "  %-18s %10s %s  ${GREEN}[PASS]${NC}\n" "$name" "$value" "$unit"
    fi
}

echo "  Metric             Value      Unit   Status"
echo "  ─────────────────────────────────────────────"
check_threshold "Idle CPU" "$IDLE_AVG" "$MAX_IDLE_CPU" "%"
check_threshold "Typing CPU" "$TYPING_AVG" "$MAX_TYPING_CPU" "%"
check_threshold "Memory Growth" "$RAM_GROWTH" "$MAX_MEMORY_GROWTH_MB" "MB"
check_threshold "P99 Latency" "$TOTAL_P99" "$MAX_P99_LATENCY_MS" "ms"
echo ""
echo "  Peak CPU: ${PEAK_CPU}%  |  Peak RAM: ${PEAK_RAM}MB"
echo "  Total Duration: ${TOTAL_DURATION}ms"
echo ""

# Save JSON results
cat > "$RESULTS_JSON" << EOF
{
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "duration_ms": $TOTAL_DURATION,
  "process": {
    "pid": $PID,
    "initial_cpu": $CPU_INIT,
    "initial_ram_mb": $RAM_INIT
  },
  "idle": {
    "cpu_avg": $IDLE_AVG,
    "samples": 5
  },
  "typing": {
    "cpu_avg": $TYPING_AVG,
    "cpu_peak": $PEAK_CPU,
    "ram_peak_mb": $PEAK_RAM,
    "samples": $TYPING_SAMPLES,
    "duration_ms": $TYPING_DURATION
  },
  "memory": {
    "before_mb": $RAM_INIT,
    "after_mb": $RAM_AFTER,
    "growth_mb": $RAM_GROWTH
  },
  "latency": {
    "detectMethod": { "avg_ms": "${DETECT_AVG:-null}", "p99_ms": "${DETECT_P99:-null}" },
    "ime_key_ext": { "avg_ms": "${IME_AVG:-null}", "p99_ms": "${IME_P99:-null}" },
    "injectSync": { "avg_ms": "${INJECT_AVG:-null}", "p99_ms": "${INJECT_P99:-null}" },
    "total": { "avg_ms": "${TOTAL_AVG:-null}", "p99_ms": "${TOTAL_P99:-null}" }
  },
  "bottleneck": "$BOTTLENECK",
  "passed": $PASS
}
EOF

echo "  Results saved: $RESULTS_JSON"
echo "  Step log: $STEP_LOG"
echo ""

# Cleanup
rm -f /tmp/typing_test.swift

if [ "$PASS" = true ]; then
    echo -e "${GREEN}╔══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║                     BENCHMARK PASSED ✓                       ║${NC}"
    echo -e "${GREEN}╚══════════════════════════════════════════════════════════════╝${NC}"
    exit 0
else
    echo -e "${RED}╔══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${RED}║                     BENCHMARK FAILED ✗                       ║${NC}"
    echo -e "${RED}╚══════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "${RED}Failures:${NC}"
    echo -e "$FAILURES"
    exit 1
fi
