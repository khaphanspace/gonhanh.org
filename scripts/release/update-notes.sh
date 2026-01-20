#!/bin/bash
# Update release notes for existing GitHub releases
# Usage: ./update-release-notes.sh [--all | --from vX.Y.Z | vX.Y.Z vX.Y.Z ...]
# Examples:
#   ./update-release-notes.sh --all              # Update all releases
#   ./update-release-notes.sh --from v1.0.45     # Update v1.0.45 and newer
#   ./update-release-notes.sh v1.0.50 v1.0.51    # Update specific releases

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

error() { echo -e "${RED}❌ $1${NC}" >&2; }
info() { echo -e "${YELLOW}$1${NC}" >&2; }
success() { echo -e "${GREEN}✅ $1${NC}" >&2; }
header() { echo -e "${BLUE}$1${NC}" >&2; }

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# Check required tools
command -v gh &> /dev/null || { error "GitHub CLI (gh) not found"; exit 1; }
command -v claude &> /dev/null || { error "Claude Code CLI not found"; exit 1; }

# Get all releases sorted by version
get_releases() {
    gh release list --limit 100 --json tagName -q '.[].tagName' | sort -V
}

# Get previous release for a given version
get_previous_release() {
    local current="$1"
    local releases=($(get_releases))
    local prev=""

    for rel in "${releases[@]}"; do
        if [ "$rel" = "$current" ]; then
            echo "$prev"
            return
        fi
        prev="$rel"
    done
}

# Update a single release
update_release() {
    local version="$1"
    local from_ref="$2"

    if [ -z "$from_ref" ]; then
        from_ref=$(get_previous_release "$version")
    fi

    if [ -z "$from_ref" ]; then
        error "Cannot find previous release for $version"
        return 1
    fi

    header "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    info "📝 Updating $version (from $from_ref)"

    # Generate new release notes
    local notes
    notes=$("$SCRIPT_DIR/generate-release-notes.sh" "$version" "$from_ref" 2>/dev/null) || {
        error "Failed to generate notes for $version"
        return 1
    }

    # Update the release on GitHub
    echo "$notes" | gh release edit "$version" --notes-file - || {
        error "Failed to update release $version on GitHub"
        return 1
    }

    success "$version updated"
    echo ""
}

# Main
main() {
    if [ $# -eq 0 ]; then
        echo "Usage: $0 [--all | --from vX.Y.Z | vX.Y.Z vX.Y.Z ...]"
        echo ""
        echo "Options:"
        echo "  --all           Update all releases"
        echo "  --from vX.Y.Z   Update from version X.Y.Z to latest"
        echo "  vX.Y.Z ...      Update specific versions"
        echo ""
        echo "Available releases:"
        get_releases | tail -10
        exit 0
    fi

    local releases_to_update=()

    case "$1" in
        --all)
            releases_to_update=($(get_releases))
            # Skip the first one (oldest) as it has no previous
            releases_to_update=("${releases_to_update[@]:1}")
            ;;
        --from)
            if [ -z "$2" ]; then
                error "--from requires a version argument"
                exit 1
            fi
            local start_version="$2"
            local all_releases=($(get_releases))
            local found=false
            for rel in "${all_releases[@]}"; do
                if [ "$rel" = "$start_version" ]; then
                    found=true
                fi
                if $found; then
                    releases_to_update+=("$rel")
                fi
            done
            if ! $found; then
                error "Version $start_version not found"
                exit 1
            fi
            ;;
        *)
            releases_to_update=("$@")
            ;;
    esac

    local total=${#releases_to_update[@]}
    header "🔄 Updating $total releases..."
    echo ""

    local success_count=0
    local fail_count=0

    for version in "${releases_to_update[@]}"; do
        if update_release "$version"; then
            ((success_count++))
        else
            ((fail_count++))
        fi

        # Rate limit: wait between API calls
        sleep 2
    done

    echo ""
    header "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    success "Done: $success_count succeeded, $fail_count failed"
}

main "$@"
