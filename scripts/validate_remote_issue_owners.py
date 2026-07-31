#!/usr/bin/env python3
"""Validate that every open GitHub Issue has one synchronized owner document.

The latest local status matrix remains the declared reconciliation snapshot. This check compares
that snapshot with the live GitHub Issue set and verifies owner-document identity plus the
presence of a synchronization comment on every open Issue.
"""

from __future__ import annotations

import json
import os
import re
import sys
import urllib.error
import urllib.request
from pathlib import Path
from typing import Any, Iterable

ROOT = Path(__file__).resolve().parent.parent
REFERENCE_DIR = ROOT / "docs" / "reference"
MATRIX_GLOB = "ISSUE-DOC-CODE-STATUS-*.md"
ROW_RE = re.compile(
    r"^\| \[#(?P<issue>\d+)\]\([^)]*\) \| .*? \| "
    r"\[(?P<owner>[^]]+)\]\((?P<path>[^)]+)\) \| "
    r"(?P<status>[^|]+?) \|"
)
SOURCE_RE = re.compile(r"(?:GitHub Issue|Source Issue)\s*#?(?P<issue>\d+)", re.IGNORECASE)


class ValidationError(RuntimeError):
    pass


def fail(message: str) -> None:
    raise ValidationError(message)


def latest_matrix() -> Path:
    candidates = sorted(REFERENCE_DIR.glob(MATRIX_GLOB))
    if not candidates:
        fail(f"no status matrix matching {MATRIX_GLOB}")
    return candidates[-1]


def parse_matrix(path: Path) -> dict[int, tuple[str, Path, str]]:
    entries: dict[int, tuple[str, Path, str]] = {}
    for line_number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        match = ROW_RE.match(line)
        if not match:
            continue
        issue = int(match.group("issue"))
        if issue in entries:
            fail(f"{path}:{line_number}: duplicate Issue #{issue}")
        owner = match.group("owner").strip()
        owner_path = (path.parent / match.group("path")).resolve()
        status = match.group("status").strip()
        entries[issue] = (owner, owner_path, status)
    if not entries:
        fail(f"{path}: no open-Issue matrix rows found")
    return entries


def github_request(url: str, token: str) -> tuple[Any, Any]:
    request = urllib.request.Request(
        url,
        headers={
            "Accept": "application/vnd.github+json",
            "Authorization": f"Bearer {token}",
            "X-GitHub-Api-Version": "2022-11-28",
            "User-Agent": "talos-governance-validator",
        },
    )
    try:
        response = urllib.request.urlopen(request, timeout=30)
        return json.load(response), response
    except urllib.error.HTTPError as exc:
        body = exc.read().decode("utf-8", errors="replace")
        fail(f"GitHub API {exc.code} for {url}: {body}")
    except urllib.error.URLError as exc:
        fail(f"GitHub API request failed for {url}: {exc}")


def next_link(link_header: str | None) -> str | None:
    if not link_header:
        return None
    for part in link_header.split(","):
        section = part.strip().split(";")
        if len(section) == 2 and section[1].strip() == 'rel="next"':
            return section[0].strip()[1:-1]
    return None


def paginated(url: str, token: str) -> Iterable[dict[str, Any]]:
    while url:
        payload, response = github_request(url, token)
        if not isinstance(payload, list):
            fail(f"expected list response from {url}")
        yield from payload
        url = next_link(response.headers.get("Link"))


def validate_owner(issue: int, owner: str, owner_path: Path) -> str:
    try:
        relative_path = owner_path.relative_to(ROOT)
    except ValueError:
        fail(f"Issue #{issue}: owner path escapes repository: {owner_path}")
    if not owner_path.is_file():
        fail(f"Issue #{issue}: missing owner document {relative_path}")
    content = owner_path.read_text(encoding="utf-8")
    if owner not in content:
        fail(f"Issue #{issue}: owner ID {owner} not found in {relative_path}")
    source_issues = {int(match.group("issue")) for match in SOURCE_RE.finditer(content)}
    if issue not in source_issues:
        fail(f"Issue #{issue}: {relative_path} does not declare this remote source")
    return relative_path.as_posix()


def main() -> int:
    token = os.environ.get("GITHUB_TOKEN", "").strip()
    repository = os.environ.get("GITHUB_REPOSITORY", "").strip()
    if not token or not repository:
        fail("GITHUB_TOKEN and GITHUB_REPOSITORY are required")

    matrix_path = latest_matrix()
    matrix = parse_matrix(matrix_path)
    api_root = f"https://api.github.com/repos/{repository}"
    issues = list(paginated(f"{api_root}/issues?state=open&per_page=100", token))
    open_issue_numbers = {
        int(item["number"])
        for item in issues
        if "pull_request" not in item
    }
    matrix_numbers = set(matrix)

    missing = sorted(open_issue_numbers - matrix_numbers)
    stale = sorted(matrix_numbers - open_issue_numbers)
    if missing or stale:
        details = []
        if missing:
            details.append("missing open Issues: " + ", ".join(f"#{n}" for n in missing))
        if stale:
            details.append("matrix entries no longer open: " + ", ".join(f"#{n}" for n in stale))
        fail(f"{matrix_path.relative_to(ROOT)} is out of sync: {'; '.join(details)}")

    owner_ids: dict[str, int] = {}
    for issue in sorted(matrix):
        owner, owner_path, status = matrix[issue]
        if owner in owner_ids:
            fail(f"owner {owner} is assigned to both Issue #{owner_ids[owner]} and Issue #{issue}")
        owner_ids[owner] = issue
        relative_path = validate_owner(issue, owner, owner_path)

        comments = list(paginated(f"{api_root}/issues/{issue}/comments?per_page=100", token))
        synchronized = any(
            "Status reconciliation" in str(comment.get("body", ""))
            and owner in str(comment.get("body", ""))
            and relative_path in str(comment.get("body", ""))
            and status.split()[0] in str(comment.get("body", ""))
            for comment in comments
        )
        if not synchronized:
            fail(
                f"Issue #{issue}: no reconciliation comment names owner {owner}, "
                f"path {relative_path}, and status {status}"
            )

    print(
        "remote issue owner validation: passed "
        f"({len(matrix)} open Issues, matrix {matrix_path.relative_to(ROOT)})"
    )
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except ValidationError as exc:
        print(f"remote issue owner validation: {exc}", file=sys.stderr)
        raise SystemExit(1)
