# PR Management Reference

## Creating PRs

```bash
# Basic
gh pr create --title "Title" --body "Description"

# With options
gh pr create --title "feat: X" --body "Desc" --base main --draft \
  --reviewer user1,user2 --assignee "@me" --label "enhancement"

# Auto-fill from commits
gh pr create --fill

# HEREDOC body
gh pr create --title "Title" --body "$(cat <<'EOF'
## Summary
- Change 1

## Test plan
- [ ] Tests pass
EOF
)"
```

## Listing & Viewing

```bash
gh pr list --state open|closed|merged|all
gh pr list --author "@me" --reviewer "@me" --label "bug"
gh pr list --search "is:open review:required"
gh pr view 123 --comments --web
gh pr list --json number,title,author,state
```

## Reviewing

```bash
gh pr review 123 --approve --body "LGTM!"
gh pr review 123 --request-changes --body "Please fix X"
gh pr review 123 --comment --body "Question about line 42"
```

## Merging

```bash
gh pr merge 123 --squash --delete-branch
gh pr merge 123 --auto --squash  # Auto-merge after checks
gh pr merge 123 --admin          # Override requirements
```

## CI/Checks & Comments

```bash
gh pr checks 123 --watch --fail-fast
gh pr comment 123 --body "Comment"
gh pr comment 123 --edit-last --body "Updated"
gh pr diff 123 --name-only
```

## API Access

```bash
gh api repos/{owner}/{repo}/pulls/123/comments
gh api repos/{owner}/{repo}/pulls/123 -X PATCH -f title="New"
```
