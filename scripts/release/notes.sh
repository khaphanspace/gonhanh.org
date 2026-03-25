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

# Resolve GitHub usernames for each commit (PR → commit hash fallback)
REPO="khaphanspace/gonhanh.org"
FORMATTED_COMMITS=$(echo "$COMMITS" | while IFS='|' read -r msg hash author; do
    login=""
    # Try 1: PR author (most accurate for external contributors)
    pr_num=$(echo "$msg" | grep -oE '\(#([0-9]+)\)' | tail -1 | grep -oE '[0-9]+' || true)
    if [ -n "$pr_num" ]; then
        login=$(gh api "repos/$REPO/pulls/$pr_num" --jq '.user.login' 2>/dev/null) || login=""
    fi
    # Try 2: Commit hash → GitHub author (works for all commits)
    if [ -z "$login" ]; then
        login=$(gh api "repos/$REPO/commits/$hash" --jq '.author.login' 2>/dev/null) || login=""
    fi
    if [ -n "$login" ]; then
        echo "- $msg bởi @$login"
    else
        echo "- $msg by $author"
    fi
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

### ⚡ Improvements
- Improvement description here

### 🐛 Bug Fixes
- Fix description here

**Full Changelog**: https://github.com/khaphanspace/gonhanh.org/compare/$FROM_REF...$VERSION

RULES:
1. Output ONLY markdown, start with '## What's Changed'
2. Group by type IN THIS ORDER: Features (new), Improvements (refactor/perf/docs), Fixes (bugs)
3. Skip empty sections - only include sections with actual changes
4. Each item: 1 line, user-facing impact, Vietnamese preferred (tech terms in English OK)
5. Platform prefix if applicable: (macOS), (linux). Always lowercase except macOS.
6. Combine related commits into single items
7. Ignore: release commits, version bumps, trivial changes
8. IMPORTANT: If a commit has a PR reference like (#NNN), ALWAYS keep it at the END of the bullet line in exact format (#NNN). Never change to (Issue #NNN) or drop it.
9. CRITICAL: EVERY bullet line MUST end with 'bởi @username'. This is taken from the commit data - preserve it exactly. When combining commits, use the 'bởi @username' from the most relevant commit. NEVER omit 'bởi @...'.

COMMITS ($COMMIT_COUNT):
$FORMATTED_COMMITS

FILES CHANGED:
$DIFF_STAT

CODE DIFF (truncated):
$DIFF_CONTENT"

# Try Claude Code first, fallback to commit-based generation
info "🤖 Calling Claude Code..."
AI_OUTPUT=$(cd /tmp && claude -p --output-format text --dangerously-skip-permissions "$PROMPT" 2>/dev/null) || true

# Strip leading/trailing blank lines
AI_OUTPUT=$(echo "$AI_OUTPUT" | sed '/./,$!d' | sed -e :a -e '/^\n*$/{$d;N;ba' -e '}')

# Validate Claude output
validate_release_notes() {
    local text="$1"
    [ -z "$text" ] && return 1
    [ ${#text} -lt 100 ] && return 1
    echo "$text" | head -1 | grep -qE '^## What'"'"'s Changed' || return 1
    echo "$text" | grep -qE '^### (✨|🐛|⚡)' || return 1
    echo "$text" | grep -q "Full Changelog" || return 1
    echo "$text" | head -3 | grep -qiE '(here|let me|i will|certainly|sure|of course)' && return 1
    return 0
}

if validate_release_notes "$AI_OUTPUT"; then
    success "✅ Release notes generated (AI)"
    echo "$AI_OUTPUT"
else
    # Fallback: generate from commit data directly
    info "⚠️  Claude output empty/invalid, generating from commits..."

    FEATURES=""
    IMPROVEMENTS=""
    FIXES=""

    while IFS= read -r line; do
        msg=$(echo "$line" | cut -d'|' -f1)
        # Strip conventional commit prefix
        display=$(echo "$msg" | sed -E 's/^(feat|fix|refactor|perf|docs|chore|style|test|ci|build)(\([^)]*\))?!?:[[:space:]]*//')
        case "$msg" in
            feat:*|feat\(*) FEATURES="${FEATURES}- ${display}\n" ;;
            fix:*|fix\(*)   FIXES="${FIXES}- ${display}\n" ;;
            *)              IMPROVEMENTS="${IMPROVEMENTS}- ${display}\n" ;;
        esac
    done <<< "$COMMITS"

    OUTPUT="## What's Changed\n"
    [ -n "$FEATURES" ] && OUTPUT="${OUTPUT}\n### ✨ New Features\n${FEATURES}"
    [ -n "$IMPROVEMENTS" ] && OUTPUT="${OUTPUT}\n### ⚡ Improvements\n${IMPROVEMENTS}"
    [ -n "$FIXES" ] && OUTPUT="${OUTPUT}\n### 🐛 Bug Fixes\n${FIXES}"
    OUTPUT="${OUTPUT}\n**Full Changelog**: https://github.com/$REPO/compare/$FROM_REF...$VERSION"

    RESULT=$(printf "$OUTPUT")
    success "✅ Release notes generated (fallback)"
    echo "$RESULT"
fi
