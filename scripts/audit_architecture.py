#!/usr/bin/env python3
"""Produce deterministic workspace architecture measurements as JSON.

This is repository analysis tooling, not a Talos runtime dependency. It uses
Cargo metadata for package authority and Git for change-frequency evidence.
"""

from __future__ import annotations

import argparse
import collections
import json
import re
import subprocess
import sys
from pathlib import Path
from typing import Any


TEST_ATTRIBUTE = "#[cfg(test)]"
MODULE_DECLARATION = re.compile(
    r"^(?:pub(?:\([^)]*\))?\s+)?mod\s+[A-Za-z_][A-Za-z0-9_]*\s*\{"
)
UNSAFE_SITE = re.compile(r"\bunsafe\s*(?:\{|fn\b|impl\b|trait\b)")
TEST_ONLY_FILE_STEMS = {"tests", "test_support"}


def run(root: Path, *args: str) -> str:
    try:
        completed = subprocess.run(
            args,
            cwd=root,
            check=True,
            capture_output=True,
            text=True,
        )
    except FileNotFoundError as error:
        raise RuntimeError(f"required host command is unavailable: {args[0]}") from error
    except subprocess.CalledProcessError as error:
        detail = error.stderr.strip() or error.stdout.strip() or "no diagnostic"
        raise RuntimeError(f"{' '.join(args)} failed: {detail}") from error
    return completed.stdout


def first_terminal_test_module(lines: list[str]) -> int | None:
    """Return the zero-based cfg line for the conventional trailing test module.

    Rust source in this workspace keeps unit-test modules at the end of a file.
    Intervening attributes such as ``#[allow(warnings)]`` are accepted. Isolated
    test-only constants do not truncate production measurement.
    """

    for index, line in enumerate(lines):
        if line.strip() != TEST_ATTRIBUTE:
            continue
        cursor = index + 1
        while cursor < len(lines) and lines[cursor].lstrip().startswith("#["):
            cursor += 1
        if cursor < len(lines) and MODULE_DECLARATION.match(lines[cursor].strip()):
            return index
    return None


def source_measurement(path: Path, crate_root: Path) -> dict[str, Any]:
    lines = path.read_text(encoding="utf-8").splitlines()
    relative = path.relative_to(crate_root).as_posix()
    test_only_file = (
        path.stem in TEST_ONLY_FILE_STEMS
        or path.stem.endswith("_tests")
        or "tests" in path.relative_to(crate_root).parts
    )
    test_start = None if test_only_file else first_terminal_test_module(lines)
    production_lines = [] if test_only_file else lines[:test_start]
    return {
        "path": relative,
        "raw_lines": len(lines),
        "production_lines": len(production_lines),
        "test_boundary_line": None if test_start is None else test_start + 1,
        "unsafe_lexical_candidates": sum(
            len(UNSAFE_SITE.findall(line)) for line in production_lines
        ),
    }


def dependency_cycles(graph: dict[str, list[str]]) -> list[list[str]]:
    cycles: set[tuple[str, ...]] = set()
    visiting: list[str] = []
    visited: set[str] = set()

    def canonical_cycle(nodes: list[str]) -> tuple[str, ...]:
        body = nodes[:-1]
        rotations = [tuple(body[index:] + body[:index]) for index in range(len(body))]
        return min(rotations)

    def visit(node: str) -> None:
        if node in visiting:
            start = visiting.index(node)
            cycles.add(canonical_cycle(visiting[start:] + [node]))
            return
        if node in visited:
            return
        visiting.append(node)
        for dependency in graph[node]:
            visit(dependency)
        visiting.pop()
        visited.add(node)

    for node in sorted(graph):
        visit(node)
    return [list(cycle) + [cycle[0]] for cycle in sorted(cycles)]


def hotspot_counts(root: Path, limit: int) -> dict[str, int]:
    output = run(
        root,
        "git",
        "log",
        f"-{limit}",
        "--format=format:__COMMIT__",
        "--name-only",
        "--",
        "crates",
    )
    counts: collections.Counter[str] = collections.Counter()
    current: set[str] = set()
    for line in [*output.splitlines(), "__COMMIT__"]:
        if line == "__COMMIT__":
            counts.update(current)
            current = set()
        elif line.endswith(".rs"):
            current.add(line)
    return dict(counts)


def inventory(root: Path, threshold: int, history: int) -> dict[str, Any]:
    metadata = json.loads(
        run(root, "cargo", "metadata", "--locked", "--no-deps", "--format-version", "1")
    )
    packages = {package["name"]: package for package in metadata["packages"]}
    graph: dict[str, list[str]] = {}
    for name, package in packages.items():
        graph[name] = sorted(
            dependency["name"]
            for dependency in package["dependencies"]
            if dependency.get("path") is not None and dependency["name"] in packages
        )

    fan_in: collections.Counter[str] = collections.Counter()
    for dependencies in graph.values():
        fan_in.update(dependencies)

    hotspots = hotspot_counts(root, history)
    crate_rows: list[dict[str, Any]] = []
    file_rows: list[dict[str, Any]] = []
    for name in sorted(packages):
        crate_root = Path(packages[name]["manifest_path"]).parent.resolve()
        measurements = [
            source_measurement(path, crate_root)
            for path in sorted((crate_root / "src").rglob("*.rs"))
        ]
        for measurement in measurements:
            workspace_path = (crate_root / measurement["path"]).relative_to(root).as_posix()
            measurement["path"] = workspace_path
            measurement["crate"] = name
            measurement["recent_commits"] = hotspots.get(workspace_path, 0)
            file_rows.append(measurement)
        crate_rows.append(
            {
                "name": name,
                "path": crate_root.relative_to(root).as_posix(),
                "internal_dependencies": graph[name],
                "fan_out": len(graph[name]),
                "fan_in": fan_in[name],
                "raw_lines": sum(row["raw_lines"] for row in measurements),
                "production_lines": sum(row["production_lines"] for row in measurements),
                "unsafe_lexical_candidates": sum(
                    row["unsafe_lexical_candidates"] for row in measurements
                ),
            }
        )

    large_files = sorted(
        (row for row in file_rows if row["production_lines"] >= threshold),
        key=lambda row: (-row["production_lines"], row["path"]),
    )
    hot_files = sorted(
        (
            row
            for row in file_rows
            if row["recent_commits"] > 0 and row["production_lines"] > 0
        ),
        key=lambda row: (-row["recent_commits"], -row["production_lines"], row["path"]),
    )[:30]

    return {
        "schema_version": 1,
        "baseline_commit": run(root, "git", "rev-parse", "HEAD").strip(),
        "method": {
            "package_authority": "cargo metadata --locked --no-deps --format-version 1",
            "production_lines": (
                "physical source lines excluding tests directories, tests.rs, *_tests.rs, "
                "test_support.rs, and the conventional trailing item-scoped #[cfg(test)] module"
            ),
            "hotspots": f"distinct commits touching each Rust file in the latest {history} commits",
            "large_file_threshold": threshold,
        },
        "workspace": {
            "crate_count": len(crate_rows),
            "raw_lines": sum(row["raw_lines"] for row in crate_rows),
            "production_lines": sum(row["production_lines"] for row in crate_rows),
            "unsafe_lexical_candidates": sum(
                row["unsafe_lexical_candidates"] for row in crate_rows
            ),
            "dependency_cycles": dependency_cycles(graph),
        },
        "crates": crate_rows,
        "large_files": large_files,
        "hot_files": hot_files,
    }


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("root", nargs="?", default=".", help="workspace root")
    parser.add_argument("--large-file-lines", type=int, default=500)
    parser.add_argument("--history", type=int, default=200)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    root = Path(args.root).resolve()
    try:
        result = inventory(root, args.large_file_lines, args.history)
    except (OSError, RuntimeError, ValueError, json.JSONDecodeError) as error:
        print(f"architecture audit failed: {error}", file=sys.stderr)
        return 1
    json.dump(result, sys.stdout, indent=2, sort_keys=True)
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
