---
name: github-assistant
description: GitHub workflow automation using `gh` CLI. Use when creating PRs, reviewing code, managing issues, auto-replying to fixed issues, adding comments, assigning users, managing labels, merging PRs, or checking CI status. Includes auto-reply script and Vietnamese templates.
---

# GitHub Assistant

Automate GitHub PR/Issue workflows using `gh` CLI + auto-reply + Vietnamese templates.

## Prerequisites

Ensure `gh` CLI authenticated: `gh auth status`

## Auto-Reply Fixed Issues

Automatically find and reply to issues fixed since last release:

```bash
# Preview (dry-run)
python scripts/auto-reply-fixed-issues.py --dry-run

# Post comments
python scripts/auto-reply-fixed-issues.py

# Specific repo
python scripts/auto-reply-fixed-issues.py --repo owner/repo
```

**How it works:**
1. Gets latest release tag
2. Finds commits since release referencing issues (#123, fixes #123)
3. Matches with open issues
4. Auto-generates reply with commit links + file changes
5. Posts comments

## Quick Reference

### PR Operations
| Task | Command |
|------|---------|
| Create PR | `gh pr create --title "Title" --body "Desc" --base main` |
| Review PR | `gh pr review 123 --approve --body "LGTM"` |
| Merge PR | `gh pr merge 123 --squash --delete-branch` |
| Check status | `gh pr checks 123 --watch` |

### Issue Operations
| Task | Command |
|------|---------|
| List issues | `gh issue list` or `gh issue list --label "bug"` |
| Add comment | `gh issue comment 123 --body "Comment"` |
| Close issue | `gh issue close 123 --comment "Fixed in v1.0"` |
| Edit labels | `gh issue edit 123 --add-label "bug"` |

## Reply Templates (Vietnamese)

| Template | Use When |
|----------|----------|
| `templates/fix-confirmed.md` | Bug fixed and released |
| `templates/need-info.md` | Need more info from reporter |
| `templates/progress-update.md` | Status update |
| `templates/known-limitation.md` | Known limitation |
| `templates/multi-report.md` | Multiple reports |
| `templates/feature-dev.md` | Feature in dev/beta |

## References

- `references/pr-management.md` - PR workflows
- `references/issue-management.md` - Issue workflows

## Special Values

`@me` current user | `--state all|open|closed` filter | `--json fields` JSON output | `--web` browser
