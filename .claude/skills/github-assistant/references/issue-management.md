# Issue Management Reference

## Creating Issues

```bash
# Basic
gh issue create --title "Bug: X" --body "Description"

# With options
gh issue create --title "Bug" --body "Desc" \
  --label "bug,priority-high" --assignee "@me" --milestone "v1.0"

# Using template
gh issue create --template "Bug Report"

# HEREDOC body
gh issue create --title "Bug" --body "$(cat <<'EOF'
## Description
Bug details

## Steps to reproduce
1. Step 1
2. Step 2
EOF
)"
```

## Listing & Viewing

```bash
gh issue list --state all|open|closed
gh issue list --author "@me" --assignee "@me" --label "bug"
gh issue list --search "is:open no:assignee sort:created-asc"
gh issue view 123 --comments --web
gh issue list --json number,title,labels,assignees
```

## Editing Issues

```bash
# Labels
gh issue edit 123 --add-label "bug" --remove-label "todo"

# Assignees
gh issue edit 123 --add-assignee "@me,user1"
gh issue edit 123 --remove-assignee "user1"

# Title/body/milestone
gh issue edit 123 --title "New" --milestone "v2.0"
```

## Comments & State

```bash
gh issue comment 123 --body "Comment"
gh issue comment 123 --edit-last --body "Updated"
gh issue close 123 --reason completed --comment "Fixed"
gh issue reopen 123 --comment "Reopening"
```

## Advanced

```bash
gh issue status          # Overview
gh issue lock/unlock 123
gh issue pin/unpin 123
gh issue transfer 123 --to owner/other-repo
```

## Search Syntax

`is:open` `author:X` `assignee:X` `no:assignee` `label:"bug"` `milestone:"v1.0"` `created:>2024-01-01` `sort:created-asc`
