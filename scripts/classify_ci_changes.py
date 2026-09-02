#!/usr/bin/env python3
"""Classify a Git change set for fail-closed pull-request CI routing."""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path, PurePosixPath


SHA_PATTERN = re.compile(r"[0-9a-fA-F]{40}")
ROOT_DOCUMENTS = {"AGENTS.md", "CHANGELOG.md", "README.md", "README.zh-CN.md"}
TEXT_GOVERNANCE_FILES = {".agent-governance/manifest.yaml"}


@dataclass(frozen=True)
class Classification:
    """One deterministic CI routing result."""

    full_validation: bool
    reason: str


def _is_allowlisted_document(path: str) -> bool:
    if not path or any(ord(character) < 32 for character in path):
        return False
    if "\\" in path or "//" in path:
        return False
    parsed = PurePosixPath(path)
    if parsed.is_absolute() or any(part in {"", ".", ".."} for part in parsed.parts):
        return False
    if path in ROOT_DOCUMENTS:
        return True
    if path in TEXT_GOVERNANCE_FILES:
        return True
    if path.startswith("docs/") and path.endswith(".md"):
        return True
    return path.startswith(".github/workflows/") and path.endswith((".yml", ".yaml"))


def classify_name_status(payload: bytes) -> Classification:
    """Classify NUL-delimited `git diff --name-status` output."""

    if not payload or not payload.endswith(b"\0"):
        return Classification(True, "missing or malformed changed-path data")
    fields = payload[:-1].split(b"\0")
    if not fields or len(fields) % 2 != 0:
        return Classification(True, "malformed changed-path record")

    paths: list[str] = []
    for index in range(0, len(fields), 2):
        try:
            status = fields[index].decode("ascii")
            path = fields[index + 1].decode("utf-8")
        except UnicodeDecodeError:
            return Classification(True, "non-UTF-8 status or path")
        if status not in {"A", "M"}:
            return Classification(True, f"change status {status!r} requires full validation")
        if not _is_allowlisted_document(path):
            return Classification(True, f"non-allowlisted path: {path}")
        paths.append(path)

    if not paths:
        return Classification(True, "empty change set")
    return Classification(False, f"allowlisted documentation only ({len(paths)} path(s))")


def _changed_records(payload: bytes) -> list[tuple[str, str]]:
    fields = payload[:-1].split(b"\0")
    return [
        (fields[index].decode("ascii"), fields[index + 1].decode("utf-8"))
        for index in range(0, len(fields), 2)
    ]


def _tree_entry(repo: Path, revision: str, path: str) -> bytes | None:
    completed = subprocess.run(
        ["git", "-C", str(repo), "ls-tree", "-z", revision, "--", path],
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        timeout=30,
    )
    if completed.returncode != 0 or completed.stdout.count(b"\0") != 1:
        return None
    return completed.stdout


def _validate_document_blobs(
    repo: Path, base_sha: str, head_sha: str, records: list[tuple[str, str]]
) -> Classification | None:
    for status, path in records:
        try:
            head_entry = _tree_entry(repo, head_sha, path)
            base_entry = _tree_entry(repo, base_sha, path) if status == "M" else None
            blob = subprocess.run(
                ["git", "-C", str(repo), "cat-file", "blob", f"{head_sha}:{path}"],
                check=False,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                timeout=30,
            )
        except (OSError, subprocess.TimeoutExpired) as error:
            return Classification(True, f"unable to inspect document blob: {type(error).__name__}")
        entry_prefix = b"100644 blob "
        if head_entry is None or not head_entry.startswith(entry_prefix):
            return Classification(True, f"non-regular or executable document: {path}")
        if status == "M" and (base_entry is None or not base_entry.startswith(entry_prefix)):
            return Classification(True, f"document mode or type changed: {path}")
        if blob.returncode != 0:
            return Classification(True, f"ambiguous document blob: {path}")
        if b"\0" in blob.stdout:
            return Classification(True, f"binary document content: {path}")
        try:
            blob.stdout.decode("utf-8")
        except UnicodeDecodeError:
            return Classification(True, f"non-UTF-8 document content: {path}")
    return None


def classify_repository(repo: Path, base_sha: str, head_sha: str) -> Classification:
    """Read changed paths from Git without executing code from the changed tree."""

    if not SHA_PATTERN.fullmatch(base_sha) or not SHA_PATTERN.fullmatch(head_sha):
        return Classification(True, "missing or malformed base/head SHA")
    try:
        completed = subprocess.run(
            [
                "git",
                "-C",
                str(repo),
                "diff",
                "--name-status",
                "-z",
                "--find-renames",
                f"{base_sha}...{head_sha}",
            ],
            check=False,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=30,
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        return Classification(True, f"unable to inspect Git change set: {type(error).__name__}")
    if completed.returncode != 0:
        return Classification(True, "Git change-set inspection failed")
    result = classify_name_status(completed.stdout)
    if result.full_validation:
        return result
    blob_result = _validate_document_blobs(
        repo, base_sha, head_sha, _changed_records(completed.stdout)
    )
    return blob_result or result


def _write_github_output(path: Path, result: Classification) -> None:
    safe_reason = result.reason.replace("\r", " ").replace("\n", " ")
    with path.open("a", encoding="utf-8") as output:
        output.write(f"full_validation={'true' if result.full_validation else 'false'}\n")
        output.write(f"reason={safe_reason}\n")


def main() -> int:
    """Run the classifier and optionally publish GitHub Actions outputs."""

    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", type=Path, required=True)
    parser.add_argument("--base", required=True)
    parser.add_argument("--head", required=True)
    parser.add_argument("--github-output", type=Path)
    args = parser.parse_args()

    result = classify_repository(args.repo, args.base, args.head)
    if args.github_output is not None:
        _write_github_output(args.github_output, result)
    route = "full" if result.full_validation else "reduced"
    print(f"CI change classification: {route} ({result.reason})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
