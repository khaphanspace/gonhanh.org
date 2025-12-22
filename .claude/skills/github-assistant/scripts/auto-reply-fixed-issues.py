#!/usr/bin/env python3
"""
Auto-reply to GitHub issues that have been fixed since last release.

Usage:
    python auto-reply-fixed-issues.py [--dry-run] [--repo owner/repo]

Features:
- Gets latest release tag
- Finds commits since release that reference issues
- Matches with open issues
- Generates reply with commit links and file changes
- Posts comments (or dry-run preview)
"""

import subprocess
import json
import re
import sys
import argparse
from typing import Optional


def run_cmd(cmd: list[str], capture: bool = True) -> str:
    """Run command and return output."""
    result = subprocess.run(cmd, capture_output=capture, text=True)
    if result.returncode != 0 and capture:
        print(f"Error: {result.stderr}", file=sys.stderr)
    return result.stdout.strip() if capture else ""


def get_repo_info() -> tuple[str, str]:
    """Get owner/repo from git remote."""
    remote = run_cmd(["git", "remote", "get-url", "origin"])
    # Parse: git@github.com:owner/repo.git or https://github.com/owner/repo.git
    match = re.search(r"[:/]([^/]+)/([^/]+?)(?:\.git)?$", remote)
    if match:
        return match.group(1), match.group(2)
    raise ValueError(f"Cannot parse repo from: {remote}")


def get_latest_release() -> Optional[dict]:
    """Get latest release info."""
    result = run_cmd(["gh", "release", "view", "--json", "tagName,publishedAt,name"])
    if not result:
        return None
    return json.loads(result)


def get_commits_since_tag(tag: str) -> list[dict]:
    """Get commits since tag with files changed."""
    # Get commit hashes since tag
    log = run_cmd(["git", "log", f"{tag}..HEAD", "--pretty=format:%H|%s"])
    if not log:
        return []

    commits = []
    for line in log.strip().split("\n"):
        if not line:
            continue
        parts = line.split("|", 1)
        sha = parts[0]
        message = parts[1] if len(parts) > 1 else ""

        # Get files changed
        files = run_cmd(["git", "diff-tree", "--no-commit-id", "--name-only", "-r", sha])
        file_list = files.split("\n") if files else []

        commits.append({
            "sha": sha,
            "short_sha": sha[:7],
            "message": message,
            "files": [f for f in file_list if f]
        })

    return commits


def extract_issue_refs(message: str) -> list[int]:
    """Extract issue numbers from commit message."""
    # Match: #123, fixes #123, closes #123, resolves #123
    patterns = [
        r"(?:fix(?:es)?|close[sd]?|resolve[sd]?)\s*#(\d+)",
        r"#(\d+)"
    ]
    issues = set()
    for pattern in patterns:
        for match in re.finditer(pattern, message, re.IGNORECASE):
            issues.add(int(match.group(1)))
    return list(issues)


def get_open_issues() -> list[dict]:
    """Get all open issues."""
    result = run_cmd([
        "gh", "issue", "list",
        "--state", "open",
        "--json", "number,title,author,labels",
        "--limit", "100"
    ])
    if not result:
        return []
    return json.loads(result)


def get_file_diff_lines(sha: str, filepath: str) -> tuple[int, int]:
    """Get start and end lines of changes in a file for a commit."""
    diff = run_cmd(["git", "show", sha, "--", filepath, "-U0", "--pretty=format:"])
    lines = []
    for line in diff.split("\n"):
        match = re.match(r"^@@.*\+(\d+)(?:,(\d+))?", line)
        if match:
            start = int(match.group(1))
            count = int(match.group(2)) if match.group(2) else 1
            lines.extend(range(start, start + count))

    if lines:
        return min(lines), max(lines)
    return 1, 1


def generate_reply(issue_num: int, commits: list[dict], owner: str, repo: str, version: str) -> str:
    """Generate reply message for an issue."""
    reply = f"Cảm ơn bạn đã báo lỗi và sử dụng Gõ Nhanh!\n\n"
    reply += f"## ✅ Đã fix trong {version}\n\n"

    # Add commits
    for commit in commits:
        sha = commit["short_sha"]
        url = f"https://github.com/{owner}/{repo}/commit/{commit['sha']}"
        reply += f"**Commit:** [`{sha}`]({url})\n"

    # Collect all files
    all_files = set()
    for commit in commits:
        all_files.update(commit["files"])

    if all_files:
        reply += "\n**Files changed:**\n"
        for filepath in sorted(all_files)[:5]:  # Limit to 5 files
            # Get line numbers from first commit that changed this file
            for commit in commits:
                if filepath in commit["files"]:
                    start, end = get_file_diff_lines(commit["sha"], filepath)
                    file_url = f"https://github.com/{owner}/{repo}/blob/{commit['sha']}/{filepath}#L{start}-L{end}"
                    reply += f"- [`{filepath}`]({file_url})\n"
                    break

    reply += f"\n---\n\n📥 **Vui lòng cập nhật lên {version}** để sử dụng bản fix này."
    return reply


def main():
    parser = argparse.ArgumentParser(description="Auto-reply to fixed issues")
    parser.add_argument("--dry-run", action="store_true", help="Preview without posting")
    parser.add_argument("--repo", help="Override repo (owner/repo)")
    args = parser.parse_args()

    # Get repo info
    if args.repo:
        owner, repo = args.repo.split("/")
    else:
        owner, repo = get_repo_info()

    print(f"📦 Repository: {owner}/{repo}")

    # Get latest release
    release = get_latest_release()
    if not release:
        print("❌ No releases found")
        return

    tag = release["tagName"]
    version = release.get("name", tag)
    print(f"🏷️  Latest release: {version} ({tag})")

    # Get commits since release
    commits = get_commits_since_tag(tag)
    print(f"📝 Commits since release: {len(commits)}")

    if not commits:
        print("✅ No new commits since release")
        return

    # Build issue -> commits mapping
    issue_commits: dict[int, list[dict]] = {}
    for commit in commits:
        refs = extract_issue_refs(commit["message"])
        for issue_num in refs:
            if issue_num not in issue_commits:
                issue_commits[issue_num] = []
            issue_commits[issue_num].append(commit)

    print(f"🔗 Issues referenced in commits: {list(issue_commits.keys())}")

    # Get open issues
    open_issues = get_open_issues()
    open_nums = {i["number"] for i in open_issues}
    print(f"📋 Open issues: {len(open_issues)}")

    # Find fixed issues
    fixed_issues = [num for num in issue_commits.keys() if num in open_nums]
    print(f"✅ Fixed issues to reply: {fixed_issues}")

    if not fixed_issues:
        print("🎉 No open issues to reply to")
        return

    # Generate and post replies
    for issue_num in fixed_issues:
        commits_for_issue = issue_commits[issue_num]
        reply = generate_reply(issue_num, commits_for_issue, owner, repo, version)

        print(f"\n{'='*60}")
        print(f"Issue #{issue_num}")
        print(f"{'='*60}")
        print(reply)

        if not args.dry_run:
            print(f"\n💬 Posting comment to #{issue_num}...")
            run_cmd(["gh", "issue", "comment", str(issue_num), "--body", reply], capture=False)
            print(f"✅ Comment posted to #{issue_num}")
        else:
            print(f"\n🔍 [DRY-RUN] Would post to #{issue_num}")


if __name__ == "__main__":
    main()
