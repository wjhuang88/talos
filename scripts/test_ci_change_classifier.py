#!/usr/bin/env python3
"""Dependency-free adversarial tests for change-aware CI classification."""

from __future__ import annotations

import importlib.util
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


sys.dont_write_bytecode = True
SCRIPT = Path(__file__).with_name("classify_ci_changes.py")
SPEC = importlib.util.spec_from_file_location("classify_ci_changes", SCRIPT)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError(f"unable to load {SCRIPT}")
MODULE = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = MODULE
SPEC.loader.exec_module(MODULE)


def payload(*records: tuple[str, str]) -> bytes:
    return b"".join(
        status.encode("ascii") + b"\0" + path.encode("utf-8") + b"\0"
        for status, path in records
    )


class ClassifierTests(unittest.TestCase):
    def assert_reduced(self, *records: tuple[str, str]) -> None:
        self.assertFalse(MODULE.classify_name_status(payload(*records)).full_validation)

    def assert_full(self, changed_paths: bytes) -> None:
        self.assertTrue(MODULE.classify_name_status(changed_paths).full_validation)

    def test_allowlisted_documentation_is_reduced(self) -> None:
        self.assert_reduced(("M", "README.md"), ("A", "docs/reference/new-guide.md"))
        self.assert_reduced(("M", "docs/backlog/active/GOV-005-change-aware-ci-routing.md"))

    def test_code_dependencies_and_runtime_control_files_are_full(self) -> None:
        for path in (
            "crates/talos-tui/src/app.rs",
            "Cargo.toml",
            "Cargo.lock",
            "scripts/release_preflight.sh",
            "scripts/fixtures/case.json",
            "docs/reference/policy.json",
            "target/generated.bin",
        ):
            with self.subTest(path=path):
                self.assert_full(payload(("M", path)))

    def test_governance_and_workflow_prose_are_reduced(self) -> None:
        for path in (
            "AGENTS.md",
            "docs/sop/TESTING.md",
            ".github/workflows/ci.yml",
            ".github/workflows/pages.yaml",
        ):
            with self.subTest(path=path):
                self.assert_reduced(("M", path))

    def test_plain_text_governance_manifest_is_reduced(self) -> None:
        self.assert_reduced(("M", ".agent-governance/manifest.yaml"))

    def test_mixed_change_is_full(self) -> None:
        self.assert_full(payload(("M", "docs/reference/guide.md"), ("M", "Cargo.toml")))

    def test_non_add_modify_status_is_full(self) -> None:
        for status in ("D", "R100", "C100", "T", "U", "X"):
            with self.subTest(status=status):
                self.assert_full(payload((status, "docs/reference/guide.md")))

    def test_malformed_empty_and_non_utf8_inputs_are_full(self) -> None:
        for changed_paths in (b"", b"M\0docs/reference/guide.md", b"M\0", b"M\0\xff\0"):
            with self.subTest(changed_paths=changed_paths):
                self.assert_full(changed_paths)

    def test_path_bypass_attempts_are_full(self) -> None:
        for path in (
            "/docs/reference/guide.md",
            "docs/../Cargo.md",
            "docs//reference/guide.md",
            "docs\\reference\\guide.md",
            "docs/reference/guide.md\nCargo.toml",
            "docs/reference/policy.json",
        ):
            with self.subTest(path=path):
                self.assert_full(payload(("M", path)))

    def test_missing_and_malformed_revisions_are_full(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo = Path(directory)
            for base, head in (("", "0" * 40), ("x" * 40, "0" * 40), ("0" * 40, "1" * 40)):
                with self.subTest(base=base, head=head):
                    self.assertTrue(MODULE.classify_repository(repo, base, head).full_validation)

    def test_real_git_changes_follow_fail_closed_routes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo = Path(directory)

            def git(*args: str) -> str:
                completed = subprocess.run(
                    ["git", "-C", str(repo), *args],
                    check=True,
                    stdout=subprocess.PIPE,
                    text=True,
                )
                return completed.stdout.strip()

            git("init", "--quiet")
            git("config", "user.name", "CI fixture")
            git("config", "user.email", "ci-fixture@example.invalid")
            (repo / "docs" / "reference").mkdir(parents=True)
            guide = repo / "docs" / "reference" / "guide.md"
            guide.write_text("baseline\n", encoding="utf-8")
            git("add", ".")
            git("commit", "--quiet", "-m", "baseline")
            base = git("rev-parse", "HEAD")

            guide.write_text("documentation update\n", encoding="utf-8")
            git("commit", "--quiet", "-am", "docs update")
            docs_head = git("rev-parse", "HEAD")
            self.assertFalse(MODULE.classify_repository(repo, base, docs_head).full_validation)

            git("mv", "docs/reference/guide.md", "docs/reference/renamed.md")
            git("commit", "--quiet", "-m", "rename docs")
            rename_head = git("rev-parse", "HEAD")
            self.assertTrue(MODULE.classify_repository(repo, docs_head, rename_head).full_validation)

            (repo / "Cargo.toml").write_text("[workspace]\n", encoding="utf-8")
            git("add", "Cargo.toml")
            git("commit", "--quiet", "-m", "mixed control change")
            mixed_head = git("rev-parse", "HEAD")
            self.assertTrue(MODULE.classify_repository(repo, base, mixed_head).full_validation)

    def test_real_git_rejects_disguised_non_prose_documents(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo = Path(directory)

            def git(*args: str) -> str:
                completed = subprocess.run(
                    ["git", "-C", str(repo), *args],
                    check=True,
                    stdout=subprocess.PIPE,
                    text=True,
                )
                return completed.stdout.strip()

            git("init", "--quiet")
            git("config", "user.name", "CI fixture")
            git("config", "user.email", "ci-fixture@example.invalid")
            (repo / "docs").mkdir()
            (repo / "docs" / "baseline.md").write_text("baseline\n", encoding="utf-8")
            git("add", ".")
            git("commit", "--quiet", "-m", "baseline")
            base = git("rev-parse", "HEAD")

            binary = repo / "docs" / "binary.md"
            binary.write_bytes(b"prose\0payload")
            git("add", ".")
            git("commit", "--quiet", "-m", "binary markdown")
            binary_head = git("rev-parse", "HEAD")
            self.assertTrue(MODULE.classify_repository(repo, base, binary_head).full_validation)

            executable = repo / "docs" / "executable.md"
            executable.write_text("#!/bin/sh\n", encoding="utf-8")
            executable.chmod(0o755)
            git("add", ".")
            git("commit", "--quiet", "-m", "executable markdown")
            executable_head = git("rev-parse", "HEAD")
            self.assertTrue(
                MODULE.classify_repository(repo, binary_head, executable_head).full_validation
            )

            symlink = repo / "docs" / "symlink.md"
            symlink.symlink_to("baseline.md")
            git("add", ".")
            git("commit", "--quiet", "-m", "symlink markdown")
            symlink_head = git("rev-parse", "HEAD")
            self.assertTrue(
                MODULE.classify_repository(repo, executable_head, symlink_head).full_validation
            )

            executable.chmod(0o644)
            git("add", ".")
            git("commit", "--quiet", "-m", "remove executable bit")
            mode_removal_head = git("rev-parse", "HEAD")
            self.assertTrue(
                MODULE.classify_repository(repo, symlink_head, mode_removal_head).full_validation
            )


if __name__ == "__main__":
    unittest.main(verbosity=2)
