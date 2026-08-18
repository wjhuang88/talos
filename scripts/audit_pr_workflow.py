#!/usr/bin/env python3
"""Generate the I205 PR workflow evidence snapshot from GitHub REST data."""

from __future__ import annotations

import argparse
import collections
import concurrent.futures
import datetime as dt
import json
import re
import subprocess
from pathlib import Path
from typing import Any


REPOSITORY = "wjhuang88/talos"

# The scope is intentionally explicit. Adding or removing a PR changes the audit population and
# must be reviewed as data, rather than being hidden behind a date or title heuristic.
CHAINS: dict[str, list[tuple[int, str]]] = {
    "I202": [(229, "claim"), (230, "implementation"), (231, "closeout")],
    "I159": [(235, "planning-and-readiness"), (236, "implementation"), (237, "closeout")],
    "I160": [
        (238, "claim"),
        (239, "activation"),
        (240, "implementation"),
        (241, "derived-state-sync"),
        (242, "ci-routing-correction"),
        (243, "closeout"),
    ],
    "I161": [
        (244, "claim"),
        (246, "security-review-gate"),
        (247, "abandoned-activation"),
        (248, "activation"),
        (249, "security-review-record"),
        (250, "implementation"),
        (251, "security-matrix-closure"),
        (252, "closeout"),
    ],
    "I162": [(253, "claim"), (254, "activation"), (255, "implementation"), (256, "closeout")],
    "I204": [
        (257, "claim"),
        (258, "abandoned-candidate"),
        (259, "activation"),
        (260, "readiness-evidence"),
        (261, "closeout"),
    ],
    "I203": [(262, "claim"), (263, "activation"), (264, "implementation"), (265, "closeout")],
    "I209": [
        (273, "planning"),
        (276, "claim"),
        (277, "activation"),
        (279, "implementation"),
        (281, "closeout"),
    ],
    "I188": [(228, "decision-implementation"), (283, "closeout")],
    "I205": [(284, "claim"), (286, "activation")],
}

# These classifications are human-reviewed interpretations. The script verifies that every cited
# PR and comment still exists and retains the cited first line, then emits the interpretation next
# to the immutable GitHub identifiers.
CORRECTIONS: list[dict[str, Any]] = [
    {
        "pr": 226,
        "comment": 5290893479,
        "class": "stale-base-inventory",
        "kind": "mechanically_preventable",
        "summary": "I195 was omitted after its claim merged into the branch base.",
    },
    {
        "pr": 235,
        "comment": 5291877133,
        "class": "dependency-fact-error",
        "kind": "substantive_architecture",
        "summary": "The document_extract readiness decision contradicted its real scraper dependency.",
    },
    {
        "pr": 236,
        "comment": 5292595210,
        "class": "validator-exact-base-gap",
        "kind": "mechanically_preventable",
        "summary": "The unbound HEAD^ comparison missed an earlier active-owner change and preflight failed in CI.",
    },
    {
        "pr": 238,
        "comment": 5294558043,
        "class": "baseline-and-owner-drift",
        "kind": "mechanically_preventable",
        "summary": "Published baseline mutation, broken section placement and owner status drift required rework.",
    },
    {
        "pr": 241,
        "comment": 5296769853,
        "class": "owner-derived-state-drift",
        "kind": "mechanically_preventable",
        "summary": "Derived views moved to Review before their owner records.",
    },
    {
        "pr": 241,
        "comment": 5296887355,
        "class": "unparsed-yaml-and-history-rewrite",
        "kind": "mechanically_preventable",
        "summary": "The manifest was invalid YAML and an append-only checkpoint was rewritten in place.",
    },
    {
        "pr": 244,
        "comment": 5300016186,
        "class": "incomplete-security-review-scope",
        "kind": "substantive_security",
        "summary": "The proposed reviewer checklist omitted permission Deny precedence and other owner invariants.",
    },
    {
        "pr": 247,
        "comment": 5300311670,
        "class": "wrong-branch-ref",
        "kind": "mechanically_preventable",
        "summary": "GitHub opened activation from a stale shared root branch, so the PR was abandoned.",
    },
    {
        "pr": 250,
        "comment": 5301049998,
        "class": "sandbox-permission-security-defect",
        "kind": "substantive_security",
        "summary": "Independent review found implementation-level sandbox and permission boundary defects.",
    },
    {
        "pr": 258,
        "comment": 5305824849,
        "class": "release-scope-before-activation",
        "kind": "mechanically_preventable",
        "summary": "Candidate version and release-surface work started outside the readiness-only activated slice.",
    },
    {
        "pr": 264,
        "comment": 5307808009,
        "class": "publication-guard-mismatch",
        "kind": "substantive_release",
        "summary": "A reviewed-head change exposed a publish guard that still encoded the old crate boundary.",
    },
    {
        "pr": 273,
        "comment": 5313751610,
        "class": "manual-exact-head-transcription",
        "kind": "mechanically_preventable",
        "summary": "A hand-expanded SHA in a synchronization comment was incorrect.",
    },
    {
        "pr": 279,
        "comment": 5315276523,
        "class": "remote-owner-reconciliation",
        "kind": "mechanically_preventable",
        "summary": "A duplicate Issue row and missing remote reconciliation forced a docs-only head change.",
    },
]

DECISION_RE = re.compile(r"REQUEST CHANGES|(?<![-\w])APPROVE(?!\w)", re.IGNORECASE)
SHA_RE = re.compile(r"\b[0-9a-f]{8,40}\b", re.IGNORECASE)
HEAD_BINDING_RES = [
    re.compile(r"\*\*Reviewed head:?\*\*[:：]?\s*`([0-9a-f]{8,40})`", re.IGNORECASE),
    re.compile(r"绑定对象.{0,80}?head[:：]?\s*`([0-9a-f]{8,40})`", re.IGNORECASE),
    re.compile(r"绑定(?:精确| exact)?\s*head\s*`([0-9a-f]{8,40})`", re.IGNORECASE),
    re.compile(r"exact head[:：]?\s*`([0-9a-f]{8,40})`", re.IGNORECASE),
    re.compile(r"head[:：]?\s*`([0-9a-f]{8,40})`", re.IGNORECASE),
]


def gh_json(endpoint: str) -> Any:
    result = subprocess.run(
        ["gh", "api", endpoint],
        check=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    return json.loads(result.stdout)


def decision_from_comment(comment: dict[str, Any]) -> dict[str, Any] | None:
    body = comment.get("body") or ""
    first_line = body.splitlines()[0] if body else ""
    review_heading = first_line.lower()
    is_review_record = (
        first_line.startswith("## Independent")
        or first_line.startswith("## 独立")
        or first_line.startswith("Independent agent technical audit")
    )
    if not is_review_record:
        return None
    if "requested" in review_heading or "request" in review_heading and "changes" not in review_heading:
        return None
    first_line_matches = list(DECISION_RE.finditer(first_line))
    verdict_lines = [
        line
        for line in body.splitlines()
        if "verdict" in line.lower() or "结论" in line or "裁定" in line
    ]
    matches = first_line_matches or [
        match for line in verdict_lines for match in DECISION_RE.finditer(line)
    ]
    if matches:
        final_match = matches[-1]
        decision = "request_changes" if "REQUEST" in final_match.group(0).upper() else "approve"
    else:
        decision = "unknown"
    shas = SHA_RE.findall(body)
    head_binding = next(
        (
            match.group(1).lower()
            for pattern in HEAD_BINDING_RES
            if (match := pattern.search(body)) is not None
        ),
        None,
    )
    return {
        "comment_id": comment["id"],
        "created_at": comment["created_at"],
        "decision": decision,
        "head_binding": head_binding,
        "candidate_head_bindings": sorted(set(sha.lower() for sha in shas)),
        "first_line": first_line,
    }


def collect_pr(repository: str, chain: str, number: int, role: str) -> dict[str, Any]:
    pull = gh_json(f"repos/{repository}/pulls/{number}")
    comments = gh_json(f"repos/{repository}/issues/{number}/comments?per_page=100")
    reviews = gh_json(f"repos/{repository}/pulls/{number}/reviews?per_page=100")
    decisions = [item for comment in comments if (item := decision_from_comment(comment))]
    reviewed_heads = sorted(
        {item["head_binding"] for item in decisions if item["head_binding"] is not None}
    )
    return {
        "chain": chain,
        "number": number,
        "role": role,
        "title": pull["title"],
        "state": pull["state"],
        "draft": pull["draft"],
        "merged": pull["merged_at"] is not None,
        "created_at": pull["created_at"],
        "merged_at": pull["merged_at"],
        "base_sha": pull["base"]["sha"],
        "head_sha": pull["head"]["sha"],
        "merge_commit_sha": pull["merge_commit_sha"],
        "commits": pull["commits"],
        "changed_files": pull["changed_files"],
        "additions": pull["additions"],
        "deletions": pull["deletions"],
        "issue_comments": len(comments),
        "formal_reviews": len(reviews),
        "review_decisions": decisions,
        "review_rounds": len(decisions),
        "request_changes_rounds": sum(item["decision"] == "request_changes" for item in decisions),
        "approval_rounds": sum(item["decision"] == "approve" for item in decisions),
        "unknown_review_rounds": sum(item["decision"] == "unknown" for item in decisions),
        "unbound_review_rounds": sum(item["head_binding"] is None for item in decisions),
        "distinct_reviewed_heads": reviewed_heads,
        "reviewed_head_changes": max(len(reviewed_heads) - 1, 0),
    }


def verify_corrections(repository: str, comments_by_pr: dict[int, list[dict[str, Any]]]) -> list[dict[str, Any]]:
    output = []
    for correction in CORRECTIONS:
        number = correction["pr"]
        comments = comments_by_pr.setdefault(
            number,
            gh_json(f"repos/{repository}/issues/{number}/comments?per_page=100"),
        )
        comment = next((item for item in comments if item["id"] == correction["comment"]), None)
        if comment is None:
            raise RuntimeError(
                f"PR #{number} no longer exposes evidence comment {correction['comment']}"
            )
        output.append(
            {
                **correction,
                "evidence_created_at": comment["created_at"],
                "evidence_first_line": (comment.get("body") or "").splitlines()[0],
            }
        )
    return output


def summarize(pulls: list[dict[str, Any]], corrections: list[dict[str, Any]]) -> dict[str, Any]:
    roles = collections.Counter(item["role"] for item in pulls)
    chains = collections.Counter(item["chain"] for item in pulls)
    correction_kinds = collections.Counter(item["kind"] for item in corrections)
    merged = [item for item in pulls if item["merged"]]
    implementation_roles = {
        "implementation",
        "decision-implementation",
        "security-matrix-closure",
        "readiness-evidence",
    }
    implementation_prs = [item for item in pulls if item["role"] in implementation_roles]
    return {
        "chains": len(chains),
        "prs": len(pulls),
        "merged_prs": len(merged),
        "closed_unmerged_prs": sum(item["state"] == "closed" and not item["merged"] for item in pulls),
        "open_prs": sum(item["state"] == "open" for item in pulls),
        "implementation_or_evidence_prs": len(implementation_prs),
        "coordination_and_state_prs": len(pulls) - len(implementation_prs),
        "review_rounds": sum(item["review_rounds"] for item in pulls),
        "request_changes_rounds": sum(item["request_changes_rounds"] for item in pulls),
        "approval_rounds": sum(item["approval_rounds"] for item in pulls),
        "unknown_review_rounds": sum(item["unknown_review_rounds"] for item in pulls),
        "unbound_review_rounds": sum(item["unbound_review_rounds"] for item in pulls),
        "reviewed_head_changes": sum(item["reviewed_head_changes"] for item in pulls),
        "changed_files": sum(item["changed_files"] for item in pulls),
        "roles": dict(sorted(roles.items())),
        "prs_by_chain": dict(sorted(chains.items())),
        "correction_kinds": dict(sorted(correction_kinds.items())),
    }


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", default=REPOSITORY)
    parser.add_argument("--output", type=Path, required=True)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    entries = [
        (chain, number, role)
        for chain, chain_entries in CHAINS.items()
        for number, role in chain_entries
    ]
    with concurrent.futures.ThreadPoolExecutor(max_workers=8) as executor:
        pulls = list(
            executor.map(
                lambda entry: collect_pr(args.repo, entry[0], entry[1], entry[2]),
                entries,
            )
        )

    comments_by_pr: dict[int, list[dict[str, Any]]] = {}
    corrections = verify_corrections(args.repo, comments_by_pr)
    document = {
        "schema_version": 1,
        "repository": args.repo,
        "generated_at": dt.datetime.now(dt.timezone.utc).isoformat(),
        "scope": {
            "start": "2026-08-14",
            "end": "2026-08-18",
            "selection": "Explicit I159-I205-era delivery chains named in scripts/audit_pr_workflow.py",
        },
        "summary": summarize(pulls, corrections),
        "pull_requests": sorted(pulls, key=lambda item: item["number"]),
        "corrections": corrections,
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(document, indent=2, sort_keys=True) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
