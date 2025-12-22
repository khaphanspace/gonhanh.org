---
description: GitHub workflow automation with natural language
argument-hint: <prompt>
---

Activate `github-assistant` skill and execute autonomously.

## Behavior

1. Parse user prompt to understand intent
2. Execute appropriate actions automatically
3. No confirmation needed - run to completion
4. Use Vietnamese templates for replies

## Example Prompts

```
/github reply all fixed issues
/github reply issue 123 with fix template
/github create pr for current branch
/github list open issues with bug label
/github review and merge pr 45
/github close resolved issues
/github check ci status for pr 67
```

## Auto-Reply Workflow

When prompt mentions "reply" + "issues" or "fixed":
1. Run: `python .claude/skills/github-assistant/scripts/auto-reply-fixed-issues.py --dry-run`
2. Review output
3. If issues found, run without --dry-run to post comments
4. Report results

## Intent Mapping

| Keywords | Action |
|----------|--------|
| reply, fixed, issues | Auto-reply to fixed issues |
| reply, issue, [number] | Reply to specific issue with template |
| create, pr | Create pull request |
| list, issues | List issues with filters |
| review, pr | Review pull request |
| merge, pr | Merge pull request |
| close, issue | Close issue(s) |
| checks, ci, status | Check CI status |

## User Prompt

$ARGUMENTS
