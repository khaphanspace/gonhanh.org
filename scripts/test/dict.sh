#!/bin/bash
# Dictionary Tests - Combined Summary
# Thresholds: VN 100%, EN 97%

cd "$(dirname "$0")/../../core"

# Run Vietnamese test and capture output (ignore test exit code)
VN_OUTPUT=$(cargo test --test vietnamese_dict_test vietnamese_dictionary_coverage -- --nocapture 2>&1 || true)
VN_TOTAL=$(echo "$VN_OUTPUT" | grep "Total words" | grep -oE '[0-9]+' | tail -1)
VN_PASSED=$(echo "$VN_OUTPUT" | grep "Passed" | grep -oE '[0-9]+' | tail -1)
VN_FAILED=$(echo "$VN_OUTPUT" | grep "Failed" | grep -oE '[0-9]+' | tail -1)
VN_RATE=$(echo "scale=2; $VN_PASSED * 100 / $VN_TOTAL" | bc)

# Run English test and capture output (ignore test exit code)
EN_OUTPUT=$(cargo test --test english_100k_test english_100k_failures -- --nocapture 2>&1 || true)
EN_TOTAL=$(echo "$EN_OUTPUT" | grep "Total words" | grep -oE '[0-9]+' | tail -1)
EN_PASSED=$(echo "$EN_OUTPUT" | grep "Passed" | grep -oE '[0-9]+' | tail -1)
EN_FAILED=$(echo "$EN_OUTPUT" | grep "Failed" | grep -oE '[0-9]+' | tail -1)
EN_RATE=$(echo "scale=2; $EN_PASSED * 100 / $EN_TOTAL" | bc)

# Extract failure causes from table (format: │ Tone (s/f/r/x/j)│                 1255 │)
EN_TONE=$(echo "$EN_OUTPUT" | grep "Tone (s/f" | grep -oE '[0-9]+' | head -1)
EN_VOWEL=$(echo "$EN_OUTPUT" | grep "Vowel (aa" | grep -oE '[0-9]+' | head -1)
EN_BOTH=$(echo "$EN_OUTPUT" | grep "│ Both" | grep -oE '[0-9]+' | head -1)
EN_UNKNOWN=$(echo "$EN_OUTPUT" | grep "│ Unknown" | grep -oE '[0-9]+' | head -1)

# Calculate cause percentages
TONE_PCT=$(echo "scale=1; $EN_TONE * 100 / $EN_FAILED" | bc)
VOWEL_PCT=$(echo "scale=1; $EN_VOWEL * 100 / $EN_FAILED" | bc)
BOTH_PCT=$(echo "scale=1; $EN_BOTH * 100 / $EN_FAILED" | bc)
UNKNOWN_PCT=$(echo "scale=1; $EN_UNKNOWN * 100 / $EN_FAILED" | bc)

# Print combined summary table
echo ""
echo "┌──────────────────────────────────────────────────────────────────┐"
echo "│                    DICTIONARY TEST RESULTS                       │"
echo "├──────────────┬──────────┬──────────┬──────────┬─────────┬────────┤"
echo "│ Dictionary   │ Total    │ Passed   │ Failed   │ Rate    │ Target │"
echo "├──────────────┼──────────┼──────────┼──────────┼─────────┼────────┤"
printf "│ Vietnamese   │ %8s │ %8s │ %8s │ %6s%% │  100%% │\n" "$VN_TOTAL" "$VN_PASSED" "$VN_FAILED" "$VN_RATE"
echo "├──────────────┼──────────┼──────────┼──────────┼─────────┼────────┤"
printf "│ English      │ %8s │ %8s │ %8s │ %6s%% │   97%% │\n" "$EN_TOTAL" "$EN_PASSED" "$EN_FAILED" "$EN_RATE"
printf "│  - Tone      │          │          │ %8s │ %6s%% │        │\n" "$EN_TONE" "$TONE_PCT"
printf "│  - Vowel     │          │          │ %8s │ %6s%% │        │\n" "$EN_VOWEL" "$VOWEL_PCT"
printf "│  - Both      │          │          │ %8s │ %6s%% │        │\n" "$EN_BOTH" "$BOTH_PCT"
printf "│  - Unknown   │          │          │ %8s │ %6s%% │        │\n" "$EN_UNKNOWN" "$UNKNOWN_PCT"
echo "└──────────────┴──────────┴──────────┴──────────┴─────────┴────────┘"

# Check thresholds
VN_OK=$(echo "$VN_RATE >= 100" | bc)
EN_OK=$(echo "$EN_RATE >= 97" | bc)

if [ "$VN_OK" -eq 1 ] && [ "$EN_OK" -eq 1 ]; then
    echo ""
    echo "✅ All dictionary tests passed"
    exit 0
else
    echo ""
    [ "$VN_OK" -eq 0 ] && echo "❌ Vietnamese: $VN_RATE% < 100%"
    [ "$EN_OK" -eq 0 ] && echo "❌ English: $EN_RATE% < 97%"
    exit 1
fi
