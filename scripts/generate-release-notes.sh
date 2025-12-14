#!/bin/bash
# Generate release notes using AI (opencode CLI)
# Usage: ./generate-release-notes.sh [version] [from-ref]
# Examples:
#   ./generate-release-notes.sh                    # from last local tag to HEAD
#   ./generate-release-notes.sh v1.0.18            # from last local tag to HEAD
#   ./generate-release-notes.sh v1.0.18 v1.0.17   # from v1.0.17 to HEAD

VERSION="${1:-next}"
FROM_REF="$2"

# Determine starting point - prefer GitHub release (actual published release)
if [ -z "$FROM_REF" ]; then
    # Get most recent release from GitHub (not local tags which may not be released yet)
    FROM_REF=$(gh release view --json tagName -q .tagName 2>/dev/null || echo "")
fi

# Fallback: get from local tag
if [ -z "$FROM_REF" ]; then
    FROM_REF=$(git describe --tags --abbrev=0 HEAD^ 2>/dev/null || echo "")
fi

# Final fallback: last 20 commits
if [ -z "$FROM_REF" ]; then
    FROM_REF="HEAD~20"
fi

echo "📝 Generating release notes: $FROM_REF → HEAD" >&2

# Get commit list
COMMITS=$(git log "$FROM_REF"..HEAD --pretty=format:"- %s (%h)" 2>/dev/null)

# Get diff summary (files changed + stats)
DIFF_STAT=$(git diff "$FROM_REF"..HEAD --stat 2>/dev/null)

# Get detailed diff (limited to avoid being too long)
DIFF_CONTENT=$(git diff "$FROM_REF"..HEAD --no-color 2>/dev/null | head -500)

if [ -z "$COMMITS" ] && [ -z "$DIFF_STAT" ]; then
    echo "No changes found from $FROM_REF to HEAD" >&2
    exit 1
fi

echo "📊 Found $(echo "$COMMITS" | wc -l | tr -d ' ') commits" >&2

# Build AI prompt
PROMPT="Generate release notes for 'Gõ Nhanh' $VERSION (Vietnamese IME for macOS).

CRITICAL: Output ONLY the markdown release notes. NO preamble, NO explanation, NO thinking.
Start directly with ## or emoji header.

Rules:
- Analyze actual code changes, not just commit messages
- Group by: ✨ New (new features), 🐛 Fixed (bug fixes), ⚡ Improved (enhancements) - skip empty sections
- Each item: 1 line, concise, describe user-facing impact
- Write in Vietnamese (technical terms in English OK)
- Output markdown only, no explanations, no intro text

Commits:
$COMMITS

Files changed:
$DIFF_STAT

Code diff (truncated):
$DIFF_CONTENT
"

# Try opencode first, with timeout (macOS compatible)
AI_OUTPUT=""
if command -v opencode &> /dev/null; then
    # Use perl timeout for macOS compatibility (no coreutils needed)
    AI_OUTPUT=$(perl -e 'alarm 60; exec @ARGV' opencode run --format json "$PROMPT" 2>/dev/null | jq -r 'select(.type == "text") | .part.text' 2>/dev/null || echo "")
fi

# Validate AI output: must start with markdown header (## or emoji) not thinking text
is_valid_release_notes() {
    local text="$1"
    # Check if starts with ## or common emoji headers (✨🐛⚡)
    if echo "$text" | head -1 | grep -qE '^(##|✨|🐛|⚡|\*\*)'; then
        return 0
    fi
    return 1
}

# If AI output is valid (non-empty, has content, and looks like release notes)
if [ -n "$AI_OUTPUT" ] && [ ${#AI_OUTPUT} -gt 20 ] && is_valid_release_notes "$AI_OUTPUT"; then
    echo "$AI_OUTPUT"
else
    # Fallback: generate simple release notes from commits
    echo "⚠️  AI generation failed, using fallback" >&2
    echo "## What's Changed"
    echo ""
    echo "$COMMITS"
    echo ""
    echo "**Full Changelog**: https://github.com/khaphanspace/gonhanh.org/compare/$FROM_REF...$VERSION"
fi
