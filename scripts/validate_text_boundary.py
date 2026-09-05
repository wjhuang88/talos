#!/usr/bin/env python3
"""I246 dependency fitness checks; reads manifests without invoking Cargo/network."""

import argparse
from pathlib import Path
import tomllib
import unittest


def dependencies(manifest, workspace):
    """Include optional/build/target edges, but exclude test-only dependencies."""
    edges = set()
    tables = [manifest, *manifest.get("target", {}).values()]
    for table in tables:
        for section in ("dependencies", "build-dependencies"):
            for alias, spec in table.get(section, {}).items():
                if isinstance(spec, dict) and spec.get("workspace"):
                    spec = workspace[alias]
                edges.add(spec.get("package", alias) if isinstance(spec, dict) else alias)
    return edges


def violations(manifests, workspace=None):
    graph = {name: dependencies(m, workspace or {}) for name, m in manifests.items()}
    errors = []
    for name, edges in graph.items():
        parsers = {e for e in edges if e.startswith("arborium") or e.startswith("tree-sitter")}
        if parsers and name != "talos-text":
            errors.append(f"{name}: parser dependency outside talos-text: {sorted(parsers)}")

    def reachable(start):
        pending = list(graph.get(start, ()))
        seen = set()
        while pending:
            node = pending.pop()
            if node not in seen:
                seen.add(node)
                pending.extend(graph.get(node, ()))
        return seen

    forbidden = {"ratatui", "crossterm", "gpui", "talos-tui", "talos-desktop"}
    for name in ("talos-core", "talos-runtime", "talos-conversation", "talos-text"):
        bad = reachable(name) & forbidden
        if bad:
            errors.append(f"{name}: renderer dependency reaches {sorted(bad)}")
    for renderer, other in (("talos-tui", "talos-desktop"), ("talos-desktop", "talos-tui")):
        if other in reachable(renderer):
            errors.append(f"{renderer}: depends on other renderer {other}")
    return errors


class FitnessTests(unittest.TestCase):
    def test_builtin_adapter_and_independent_consumers_pass(self):
        self.assertEqual(violations({
            "talos-text": {"dependencies": {"arborium": {"optional": True}}},
            "talos-tui": {"dependencies": {"talos-text": "1", "ratatui": "1"}},
            "talos-tools": {"dependencies": {"talos-text": "1"}},
        }), [])

    def test_renamed_optional_target_parser_is_rejected(self):
        self.assertTrue(violations({"talos-tools": {"target": {"cfg(unix)": {
            "dependencies": {"parser": {"package": "arborium", "optional": True}}
        }}}}))

    def test_transitive_workspace_renderer_edge_is_rejected(self):
        manifests = {
            "talos-text": {"dependencies": {"bridge": "1"}},
            "bridge": {"build-dependencies": {"ui": {"workspace": True}}},
        }
        self.assertTrue(violations(manifests, {"ui": {"package": "gpui", "version": "1"}}))

    def test_renderers_must_not_depend_on_each_other(self):
        self.assertTrue(violations({"talos-desktop": {"dependencies": {"talos-tui": "1"}}}))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("root", type=Path, nargs="?", default=Path(__file__).resolve().parents[1])
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        result = unittest.TextTestRunner().run(unittest.defaultTestLoader.loadTestsFromTestCase(FitnessTests))
        return 0 if result.wasSuccessful() else 1
    root = tomllib.loads((args.root / "Cargo.toml").read_text())
    manifests = {}
    for member in root["workspace"]["members"]:
        for directory in args.root.glob(member):
            manifest = tomllib.loads((directory / "Cargo.toml").read_text())
            manifests[manifest["package"]["name"]] = manifest
    errors = violations(manifests, root["workspace"].get("dependencies", {}))
    for error in errors:
        print(error)
    print(f"Text boundary validation: {len(manifests)} workspace crates, {len(errors)} error(s)")
    return int(bool(errors))


if __name__ == "__main__":
    raise SystemExit(main())
