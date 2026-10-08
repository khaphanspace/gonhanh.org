#!/bin/bash
# V2 gate: run the whole suite on engine_v2 and compare the failing tests with the documented
# differences (core/tests/v2-known-differences.txt). A new failure is a regression; a listed test
# that now passes means the list (and docs/behavior-changes.md) needs tightening.
set -uo pipefail
cd "$(dirname "$0")/../../core"
PROFILE=${GN_PROFILE:-gate}

LIB=$(cargo test --profile "$PROFILE" --features engine_v2 --lib -q 2>&1)
OUT=$(cargo test --profile "$PROFILE" --features engine_v2 --test suite -q -- --skip golden_digest 2>&1)
failed_in() { awk '/^failures:$/{f++} f==2 && /^    /{print $1}'; }
FAILED=$( { echo "$LIB" | failed_in; echo "$OUT" | failed_in; } | sort -u | grep -v '^$')
KNOWN=$(grep -v '^#' tests/v2-known-differences.txt | grep -v '^$' | sort -u)

NEW=$(comm -23 <(echo "$FAILED") <(echo "$KNOWN"))
FIXED=$(comm -13 <(echo "$FAILED") <(echo "$KNOWN"))
echo "$LIB" | grep "test result"; echo "$OUT" | grep "test result"
if [ -n "$NEW" ]; then echo "✗ new failures on V2:"; echo "$NEW"; exit 1; fi
if [ -n "$FIXED" ]; then echo "✗ listed as different but passing now (remove from tests/v2-known-differences.txt):"; echo "$FIXED"; exit 1; fi
echo "✅ V2 matches V1 except the $(echo "$KNOWN" | wc -l | tr -d ' ') documented differences"
