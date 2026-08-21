#!/usr/bin/env python3
"""Validate delivery-stage policy scenarios used by Talos governance."""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parent.parent
DEFAULT_FIXTURES = ROOT / "scripts" / "fixtures" / "delivery-workflow-cases.json"
PROTECTED = {"security", "sandbox", "permission", "process-hardening"}


def violations(case: dict[str, Any]) -> list[str]:
    kind = str(case["kind"])
    errors: list[str] = []

    if case.get("activation_effective") and not case.get("claim_effective"):
        errors.append("activation cannot become effective before the claim")
    if case.get("implementation_started") and not case.get("claim_effective"):
        errors.append("implementation cannot start before target-branch claim truth")
    if case.get("implementation_pushed") and not case.get("local_convergence_passed"):
        errors.append("the first implementation push requires local convergence")
    if case.get("merge_requested"):
        if not case.get("exact_head_evidence"):
            errors.append("merge requires exact-head evidence")
        if not case.get("merge_time_cas"):
            errors.append("merge requires merge-time CAS")
    if case.get("substantive_change_after_review") and not case.get("fresh_exact_head_evidence"):
        errors.append("substantive post-review changes require fresh exact-head evidence")
    if kind in PROTECTED and case.get("merge_requested"):
        if not case.get("independent_review"):
            errors.append("protected scope requires independent review")
        if not case.get("review_covers_assigned_risks"):
            errors.append("protected review must cover the assigned risk surface")
    if case.get("review_used"):
        if not case.get("review_exact_head"):
            errors.append("review evidence must bind the exact head")
        if not case.get("review_verdict"):
            errors.append("review evidence requires an explicit verdict")
        if not case.get("review_covers_assigned_risks"):
            errors.append("review evidence must cover assigned requirements and risks")
    if kind == "release":
        if not case.get("release_preflight"):
            errors.append("release requires repository preflight")
        if not case.get("immutable_tag_policy"):
            errors.append("release must preserve immutable tag policy")
        if not case.get("github_before_cargo"):
            errors.append("GitHub release must precede Cargo publication")
        if not case.get("independent_review"):
            errors.append("release requires independent review")
    if kind == "bounded-maintenance":
        forbidden = (
            "changes_behavior",
            "changes_public_api",
            "changes_security",
            "changes_dependencies",
            "changes_release_authority",
            "changes_persistence",
            "changes_owner_state",
            "leaves_residual_work",
        )
        if any(case.get(field) for field in forbidden):
            errors.append("bounded maintenance cannot change governed behavior/state or leave residuals")
    if case.get("completion_claimed") and not case.get("completion_commit_preexisting"):
        errors.append("completion requires a pre-existing implementation/evidence commit")
    return errors


def main() -> int:
    fixture_path = Path(sys.argv[1]) if len(sys.argv) > 1 else DEFAULT_FIXTURES
    cases = json.loads(fixture_path.read_text(encoding="utf-8"))
    if not isinstance(cases, list) or not cases:
        raise ValueError("workflow fixture must contain a non-empty JSON array")

    failed = 0
    for case in cases:
        actual = not violations(case)
        expected = bool(case["valid"])
        if actual != expected:
            failed += 1
            print(
                f"FAIL {case['name']}: expected valid={expected}, "
                f"violations={violations(case)}",
                file=sys.stderr,
            )

    if failed:
        print(f"delivery workflow validation failed: {failed} case(s)", file=sys.stderr)
        return 1
    print(f"delivery workflow validation passed: {len(cases)} case(s)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
