#!/usr/bin/env python3
"""Validate the ADR-008 SQLite consumer boundary from locked Cargo metadata."""

from __future__ import annotations

import argparse
import copy
import json
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any


POLICY_PATH = Path("docs/reference/SQLITE-CONSUMER-POLICY.json")
POLICY_SCHEMA_VERSION = 1
CONSUMER_CLASSIFICATIONS = {"runtime", "quarantined-non-runtime"}
RUSQLITE = "rusqlite"
SQLITE_SYS = "libsqlite3-sys"


@dataclass(frozen=True)
class ConsumerPolicy:
    """Validated machine-readable ADR-008 consumer policy."""

    accepted_consumers: frozenset[str]
    quarantined_consumers: frozenset[str]


@dataclass(frozen=True)
class ValidationReport:
    """Computed policy facts for one resolved metadata graph."""

    consumers: frozenset[str]
    layered_packages: frozenset[str]
    rusqlite_versions: frozenset[str]
    sqlite_sys_versions: frozenset[str]
    quarantined_dependents: frozenset[str]
    errors: tuple[str, ...]


def _dependency_ids(node: dict[str, Any]) -> list[str]:
    return [dep["pkg"] for dep in node.get("deps", []) if isinstance(dep.get("pkg"), str)]


def parse_policy(document: dict[str, Any]) -> ConsumerPolicy:
    """Validate and parse the versioned ADR-008 consumer policy."""

    if set(document) != {"schema_version", "accepted_consumers"}:
        raise ValueError("policy must contain only schema_version and accepted_consumers")
    version = document["schema_version"]
    if version != POLICY_SCHEMA_VERSION:
        raise ValueError(f"unsupported SQLite consumer policy schema_version: {version}")
    entries = document["accepted_consumers"]
    if not isinstance(entries, list) or not entries:
        raise ValueError("policy accepted_consumers must be a non-empty array")

    consumers: set[str] = set()
    quarantined: set[str] = set()
    for index, entry in enumerate(entries):
        if (
            not isinstance(entry, dict)
            or set(entry) != {"name", "classification"}
            or not isinstance(entry.get("name"), str)
            or not isinstance(entry.get("classification"), str)
        ):
            raise ValueError(
                f"accepted_consumers[{index}] must contain only name and classification strings"
            )
        name = entry["name"]
        classification = entry["classification"]
        if not name:
            raise ValueError(f"accepted_consumers[{index}].name must not be empty")
        if classification not in CONSUMER_CLASSIFICATIONS:
            raise ValueError(f"unsupported SQLite consumer classification: {classification}")
        if name in consumers:
            raise ValueError(f"duplicate accepted SQLite consumer: {name}")
        consumers.add(name)
        if classification == "quarantined-non-runtime":
            quarantined.add(name)
    if quarantined != {"talos-models"}:
        raise ValueError(
            "policy must classify exactly talos-models as quarantined-non-runtime"
        )
    return ConsumerPolicy(
        accepted_consumers=frozenset(consumers),
        quarantined_consumers=frozenset(quarantined),
    )


def load_policy(project_root: Path) -> ConsumerPolicy:
    """Load the normative ADR-008 policy from the repository."""

    policy_path = project_root / POLICY_PATH
    try:
        document = json.loads(policy_path.read_text(encoding="utf-8"))
    except FileNotFoundError as error:
        raise ValueError(f"missing SQLite consumer policy: {POLICY_PATH}") from error
    except UnicodeDecodeError as error:
        raise ValueError(f"SQLite consumer policy is not valid UTF-8: {POLICY_PATH}") from error
    except json.JSONDecodeError as error:
        raise ValueError(f"malformed SQLite consumer policy {POLICY_PATH}: {error}") from error
    if not isinstance(document, dict):
        raise ValueError("SQLite consumer policy must be a JSON object")
    return parse_policy(document)


def evaluate(metadata: dict[str, Any], policy: ConsumerPolicy) -> ValidationReport:
    """Evaluate one Cargo metadata document against the accepted SQLite policy."""

    packages = metadata.get("packages")
    resolve = metadata.get("resolve")
    workspace_members = metadata.get("workspace_members")
    if not isinstance(packages, list) or not isinstance(resolve, dict):
        raise ValueError("metadata must contain packages and resolve objects")
    if not isinstance(workspace_members, list):
        raise ValueError("metadata must contain workspace_members")

    package_by_id = {
        package["id"]: package
        for package in packages
        if isinstance(package, dict) and isinstance(package.get("id"), str)
    }
    node_by_id = {
        node["id"]: node
        for node in resolve.get("nodes", [])
        if isinstance(node, dict) and isinstance(node.get("id"), str)
    }
    resolved_ids = set(node_by_id)
    workspace_ids = set(workspace_members)

    missing_packages = sorted((resolved_ids | workspace_ids) - set(package_by_id))
    missing_nodes = sorted(workspace_ids - resolved_ids)
    if missing_packages or missing_nodes:
        details = []
        if missing_packages:
            details.append(f"missing package records: {', '.join(missing_packages)}")
        if missing_nodes:
            details.append(f"missing resolve nodes: {', '.join(missing_nodes)}")
        raise ValueError("; ".join(details))

    name_by_id = {package_id: package["name"] for package_id, package in package_by_id.items()}

    def reaches(start: str, targets: set[str]) -> bool:
        pending = [start]
        seen: set[str] = set()
        while pending:
            current = pending.pop()
            if current in targets:
                return True
            if current in seen:
                continue
            seen.add(current)
            node = node_by_id.get(current)
            if node is not None:
                pending.extend(_dependency_ids(node))
        return False

    sqlite_sys_ids = {
        package_id
        for package_id in resolved_ids
        if name_by_id.get(package_id) == SQLITE_SYS
    }
    consumers: set[str] = set()
    transitively_reaching: set[str] = set()
    for workspace_id in workspace_ids:
        workspace_name = name_by_id[workspace_id]
        node = node_by_id[workspace_id]
        if reaches(workspace_id, sqlite_sys_ids):
            transitively_reaching.add(workspace_name)
        if any(
            dep_id not in workspace_ids and reaches(dep_id, sqlite_sys_ids)
            for dep_id in _dependency_ids(node)
        ):
            consumers.add(workspace_name)

    model_ids = {
        package_id
        for package_id in workspace_ids
        if name_by_id.get(package_id) in policy.quarantined_consumers
    }
    quarantined_dependents = {
        name_by_id[workspace_id]
        for workspace_id in workspace_ids - model_ids
        if reaches(workspace_id, model_ids)
    }

    def resolved_versions(name: str) -> set[str]:
        return {
            package_by_id[package_id]["version"]
            for package_id in resolved_ids
            if name_by_id.get(package_id) == name
        }

    rusqlite_versions = resolved_versions(RUSQLITE)
    sqlite_sys_versions = resolved_versions(SQLITE_SYS)

    def resolved_features(name: str) -> set[str]:
        features: set[str] = set()
        for package_id in resolved_ids:
            if name_by_id.get(package_id) == name:
                features.update(node_by_id[package_id].get("features", []))
        return features

    errors: list[str] = []
    unexpected = sorted(consumers - policy.accepted_consumers)
    missing = sorted(policy.accepted_consumers - consumers)
    if unexpected:
        errors.append(f"unexpected SQLite boundary consumer(s): {', '.join(unexpected)}")
    if missing:
        errors.append(f"missing accepted SQLite boundary consumer(s): {', '.join(missing)}")
    if len(rusqlite_versions) != 1:
        errors.append(
            "expected one resolved rusqlite version, found: "
            + (", ".join(sorted(rusqlite_versions)) or "none")
        )
    if len(sqlite_sys_versions) != 1:
        errors.append(
            "expected one resolved libsqlite3-sys version, found: "
            + (", ".join(sorted(sqlite_sys_versions)) or "none")
        )
    if sqlite_sys_ids and "bundled" not in resolved_features(SQLITE_SYS):
        errors.append("resolved libsqlite3-sys does not enable the bundled feature")
    if rusqlite_versions and "bundled" not in resolved_features(RUSQLITE):
        errors.append("resolved rusqlite does not enable the bundled feature")
    if quarantined_dependents:
        errors.append(
            "workspace package(s) depend on quarantined talos-models: "
            + ", ".join(sorted(quarantined_dependents))
        )

    return ValidationReport(
        consumers=frozenset(consumers),
        layered_packages=frozenset(transitively_reaching - consumers),
        rusqlite_versions=frozenset(rusqlite_versions),
        sqlite_sys_versions=frozenset(sqlite_sys_versions),
        quarantined_dependents=frozenset(quarantined_dependents),
        errors=tuple(errors),
    )


def _edge(target: str, kind: str | None = None, predicate: str | None = None) -> dict[str, Any]:
    return {
        "name": target,
        "pkg": target,
        "dep_kinds": [{"kind": kind, "target": predicate}],
    }


def _package(package_id: str, name: str, version: str = "0.7.0") -> dict[str, str]:
    return {"id": package_id, "name": name, "version": version}


def _apply_fixture_mutation(metadata: dict[str, Any], case: dict[str, Any]) -> None:
    mutation = case["mutation"]
    packages = metadata["packages"]
    nodes = metadata["resolve"]["nodes"]
    package_by_name = {package["name"]: package for package in packages}
    node_by_id = {node["id"]: node for node in nodes}

    if mutation == "none":
        return
    if mutation == "add_consumer":
        workspace_id = f"workspace-{case['name']}"
        external_id = f"external-{case['name']}"
        metadata["workspace_members"].append(workspace_id)
        packages.extend(
            [
                _package(workspace_id, f"talos-sixth-{case['name']}"),
                _package(external_id, f"alternate-sqlite-{case['name']}", "1.0.0"),
            ]
        )
        nodes.extend(
            [
                {
                    "id": workspace_id,
                    "deps": [
                        _edge(
                            external_id,
                            case.get("kind"),
                            case.get("target"),
                        )
                    ],
                    "features": [],
                },
                {
                    "id": external_id,
                    "deps": [_edge(package_by_name[SQLITE_SYS]["id"])],
                    "features": [],
                },
            ]
        )
        return
    if mutation == "remove_consumer":
        package_id = package_by_name[case["package"]]["id"]
        node_by_id[package_id]["deps"] = []
        return
    if mutation == "duplicate_versions":
        packages.extend(
            [
                _package("rusqlite-duplicate", RUSQLITE, "0.39.0"),
                _package("sqlite-sys-duplicate", SQLITE_SYS, "0.37.0"),
            ]
        )
        nodes.extend(
            [
                {"id": "rusqlite-duplicate", "deps": [], "features": ["bundled"]},
                {"id": "sqlite-sys-duplicate", "deps": [], "features": ["bundled"]},
            ]
        )
        return
    if mutation == "add_workspace_dependency":
        source_id = package_by_name[case["source"]]["id"]
        target_id = package_by_name[case["target"]]["id"]
        node_by_id[source_id]["deps"].append(_edge(target_id))
        return
    if mutation == "remove_bundled":
        for dependency in (RUSQLITE, SQLITE_SYS):
            dependency_id = package_by_name[dependency]["id"]
            node_by_id[dependency_id]["features"] = [
                feature
                for feature in node_by_id[dependency_id]["features"]
                if feature != "bundled"
            ]
        return
    raise ValueError(f"unknown fixture mutation: {mutation}")


def run_self_tests(fixture_root: Path, policy: ConsumerPolicy) -> None:
    """Run the controlled positive and negative metadata matrix."""

    base = json.loads((fixture_root / "base.json").read_text(encoding="utf-8"))
    cases = json.loads((fixture_root / "cases.json").read_text(encoding="utf-8"))["cases"]
    failures: list[str] = []
    for case in cases:
        metadata = copy.deepcopy(base)
        _apply_fixture_mutation(metadata, case)
        report = evaluate(metadata, policy)
        expected = case["expected_error_substrings"]
        actual = "\n".join(report.errors)
        if case.get("expected_consumers") is not None and sorted(report.consumers) != sorted(
            case["expected_consumers"]
        ):
            failures.append(
                f"{case['name']}: consumers={sorted(report.consumers)!r}, "
                f"expected={sorted(case['expected_consumers'])!r}"
            )
        if case.get("expected_layered_packages") is not None and sorted(
            report.layered_packages
        ) != sorted(case["expected_layered_packages"]):
            failures.append(
                f"{case['name']}: layered={sorted(report.layered_packages)!r}, "
                f"expected={sorted(case['expected_layered_packages'])!r}"
            )
        for substring in expected:
            if substring not in actual:
                failures.append(f"{case['name']}: missing expected error substring: {substring}")
        if not expected and report.errors:
            failures.append(f"{case['name']}: unexpected errors: {actual}")
        if expected and not report.errors:
            failures.append(f"{case['name']}: expected failure but validation passed")
    policy_document = json.loads(
        (fixture_root.parents[2] / POLICY_PATH).read_text(encoding="utf-8")
    )
    policy_cases = json.loads((fixture_root / "policy-cases.json").read_text(encoding="utf-8"))[
        "cases"
    ]
    for case in policy_cases:
        candidate = copy.deepcopy(policy_document)
        mutation = case["mutation"]
        if mutation == "unknown_version":
            candidate["schema_version"] = 2
        elif mutation == "remove_consumers":
            del candidate["accepted_consumers"]
        elif mutation == "malformed_consumer":
            candidate["accepted_consumers"][0] = {"name": 1, "classification": "runtime"}
        elif mutation == "duplicate_consumer":
            candidate["accepted_consumers"].append(candidate["accepted_consumers"][0])
        elif mutation == "invalid_classification":
            candidate["accepted_consumers"][0]["classification"] = "other"
        elif mutation == "add_policy_consumer":
            candidate["accepted_consumers"].append(
                {"name": "talos-policy-only", "classification": "runtime"}
            )
        else:
            failures.append(f"{case['name']}: unknown policy fixture mutation: {mutation}")
            continue
        try:
            candidate_policy = parse_policy(candidate)
            report = evaluate(copy.deepcopy(base), candidate_policy)
            actual = "\n".join(report.errors)
        except (KeyError, TypeError, ValueError) as error:
            actual = str(error)
        if case["expected_error_substring"] not in actual:
            failures.append(
                f"{case['name']}: missing expected error substring: "
                f"{case['expected_error_substring']}"
            )

    diagnostic_cases = json.loads(
        (fixture_root / "diagnostic-cases.json").read_text(encoding="utf-8")
    )["cases"]
    for case in diagnostic_cases:
        try:
            _parse_cargo_result(
                case["returncode"],
                bytes.fromhex(case["stdout_hex"]),
                bytes.fromhex(case["stderr_hex"]),
            )
            actual = ""
        except (UnicodeDecodeError, ValueError, RuntimeError) as error:
            actual = str(error)
        if case["expected_error_substring"] not in actual:
            failures.append(
                f"{case['name']}: missing expected error substring: "
                f"{case['expected_error_substring']}"
            )

    if failures:
        raise ValueError("fixture failures:\n" + "\n".join(failures))
    total_cases = len(cases) + len(policy_cases) + len(diagnostic_cases)
    print(f"SQLite consumer fixture validation passed: {total_cases} case(s)")


def _parse_cargo_result(returncode: int, stdout: bytes, stderr: bytes) -> dict[str, Any]:
    """Parse Cargo output while keeping invalid failure diagnostics inspectable."""

    if returncode != 0:
        diagnostic = stderr.decode("utf-8", errors="backslashreplace").rstrip()
        raise RuntimeError(f"cargo metadata --locked failed:\n{diagnostic}")
    try:
        decoded = stdout.decode("utf-8", errors="strict")
    except UnicodeDecodeError as error:
        raise ValueError("cargo metadata stdout is not valid UTF-8") from error
    return json.loads(decoded)


def load_locked_metadata(project_root: Path) -> dict[str, Any]:
    """Run repository-pinned Cargo metadata without mutating dependency resolution."""

    result = subprocess.run(
        ["cargo", "metadata", "--locked", "--format-version", "1"],
        cwd=project_root,
        check=False,
        capture_output=True,
    )
    return _parse_cargo_result(result.returncode, result.stdout, result.stderr)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("project_root", nargs="?", default=".")
    parser.add_argument("--metadata", type=Path, help="validate a controlled metadata JSON file")
    parser.add_argument("--self-test", action="store_true", help="run the shared fixture matrix")
    args = parser.parse_args()

    project_root = Path(args.project_root).resolve()
    try:
        policy = load_policy(project_root)
        metadata = (
            json.loads(args.metadata.read_text(encoding="utf-8"))
            if args.metadata is not None
            else load_locked_metadata(project_root)
        )
        report = evaluate(metadata, policy)
        if report.errors:
            for error in report.errors:
                print(f"ERROR: {error}", file=sys.stderr)
            return 1
        if args.self_test:
            run_self_tests(project_root / "scripts/fixtures/sqlite-consumer-metadata", policy)
        print(
            "SQLite consumer validation passed: "
            f"consumers={','.join(sorted(report.consumers))}; "
            f"layered={','.join(sorted(report.layered_packages)) or 'none'}; "
            f"rusqlite={','.join(sorted(report.rusqlite_versions))}; "
            f"libsqlite3-sys={','.join(sorted(report.sqlite_sys_versions))}; "
            "talos-models-dependents=0"
        )
    except (KeyError, OSError, TypeError, ValueError, RuntimeError) as error:
        print(f"ERROR: SQLite consumer validation could not run: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
