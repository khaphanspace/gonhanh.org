#!/bin/bash
# Generate release notes using Claude Code CLI
# Usage: ./generate-release-notes.sh [version] [from-ref]
# Examples:
#   ./generate-release-notes.sh                    # from last GitHub release to HEAD
#   ./generate-release-notes.sh v1.0.18            # from last GitHub release to HEAD
#   ./generate-release-notes.sh v1.0.18 v1.0.17   # from v1.0.17 to HEAD
#
# STRICT MODE: Script will FAIL if release notes cannot be generated properly.
# No fallbacks - ensures every release has quality notes.

set -e  # Exit on any error

VERSION="${1:-next}"
FROM_REF="$2"

# Colors for terminal output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

error() { echo -e "${RED}❌ $1${NC}" >&2; exit 1; }
info() { echo -e "${YELLOW}$1${NC}" >&2; }
success() { echo -e "${GREEN}$1${NC}" >&2; }

# Check required tools
command -v claude &> /dev/null || error "Claude Code CLI not found. Install: https://docs.anthropic.com/en/docs/claude-code"
command -v gh &> /dev/null || error "GitHub CLI (gh) not found"

# Determine FROM_REF - strictly from GitHub releases only
if [ -z "$FROM_REF" ]; then
    info "📍 Getting last release from GitHub..."
    FROM_REF=$(gh release view --json tagName -q .tagName 2>/dev/null) || error "No previous release found on GitHub. Use: $0 $VERSION <from-ref>"
fi

info "📝 Generating release notes: $FROM_REF → HEAD"

# Validate FROM_REF exists
git rev-parse "$FROM_REF" &>/dev/null || error "Reference '$FROM_REF' not found in git history"

# Get commit list (exclude release commits and merge commits)
COMMITS=$(git log "$FROM_REF"..HEAD --pretty=format:"%s|%h|%an" --no-merges 2>/dev/null | grep -v "^release:" || true)

if [ -z "$COMMITS" ]; then
    error "No commits found between $FROM_REF and HEAD (excluding release commits)"
fi

COMMIT_COUNT=$(echo "$COMMITS" | wc -l | tr -d ' ')
info "📊 Found $COMMIT_COUNT commits"

# Format commits for readability
FORMATTED_COMMITS=$(echo "$COMMITS" | while IFS='|' read -r msg hash author; do
    echo "- $msg ($hash) by $author"
done)

# Get diff summary
DIFF_STAT=$(git diff "$FROM_REF"..HEAD --stat 2>/dev/null)

# Get detailed diff (limited)
DIFF_CONTENT=$(git diff "$FROM_REF"..HEAD --no-color 2>/dev/null | head -800)

# Build prompt for Claude
PROMPT="Generate release notes for 'Gõ Nhanh' $VERSION (Vietnamese IME for macOS/Linux).

OUTPUT FORMAT - Follow this EXACTLY:
## What's Changed

### ✨ New Features
- Feature description here

### 🐛 Bug Fixes
- Fix description here

### ⚡ Improvements
- Improvement description here

**Full Changelog**: https://github.com/khaphanspace/gonhanh.org/compare/$FROM_REF...$VERSION

RULES:
1. Output ONLY markdown, start with '## What's Changed'
2. Group by type: Features (new), Fixes (bugs), Improvements (refactor/perf/docs)
3. Skip empty sections - only include sections with actual changes
4. Each item: 1 line, user-facing impact, Vietnamese preferred (tech terms in English OK)
5. Platform prefix if applicable: (macOS), (Linux)
6. Combine related commits into single items
7. Ignore: release commits, version bumps, trivial changes

COMMITS ($COMMIT_COUNT):
$FORMATTED_COMMITS

FILES CHANGED:
$DIFF_STAT

CODE DIFF (truncated):
$DIFF_CONTENT"

info "🤖 Calling Claude Code..."

# Call Claude Code with strict settings
AI_OUTPUT=$(claude -p --output-format text --dangerously-skip-permissions "$PROMPT" 2>/dev/null) || error "Claude Code failed to execute"

# Validate output quality - STRICT validation
validate_release_notes() {
    local text="$1"

    # Must not be empty
    [ -z "$text" ] && { echo "FAIL: empty" >&2; return 1; }

    # Must be at least 100 chars (meaningful content)
    [ ${#text} -lt 100 ] && { echo "FAIL: too short (<100 chars)" >&2; return 1; }

    # Must start with "## What's Changed" exactly
    echo "$text" | head -1 | grep -qE '^## What'"'"'s Changed' || { echo "FAIL: missing '## What's Changed' header" >&2; return 1; }

    # Must contain at least one section with emoji header
    echo "$text" | grep -qE '^### (✨|🐛|⚡)' || { echo "FAIL: missing section headers (### ✨/🐛/⚡)" >&2; return 1; }

    # Must contain changelog link
    echo "$text" | grep -q "Full Changelog" || { echo "FAIL: missing Full Changelog link" >&2; return 1; }

    # Must not contain AI preamble/thinking
    echo "$text" | head -3 | grep -qiE '(here|let me|i will|certainly|sure|of course)' && { echo "FAIL: contains AI preamble" >&2; return 1; }

    return 0
}

if validate_release_notes "$AI_OUTPUT"; then
    success "✅ Release notes generated successfully"
    echo "$AI_OUTPUT"
else
    error "Generated release notes failed validation. Output was:
---
$AI_OUTPUT
---
Please check Claude Code configuration or generate manually."
fi
