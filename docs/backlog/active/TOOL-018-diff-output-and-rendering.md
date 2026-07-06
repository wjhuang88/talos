# TOOL-018: Diff Output And Rendering Reliability

| Field | Value |
|---|---|
| Story ID | TOOL-018 |
| Priority | P1 |
| Status | Planned |
| Source | [GitHub Issue #20](https://github.com/wjhuang88/talos/issues/20), [GitHub Issue #21](https://github.com/wjhuang88/talos/issues/21) |
| Depends On | `TOOL-015`, `TUI-023`, `GIT-001` |

## Problem

`edit` returns bounded `-`/`+` diff-like text that is not always recognized by TUI diff rendering.
`git_diff` currently behaves more like a changed-file list than a real unified diff, limiting review
quality before commit.

## Acceptance

- `edit` result rendering uses the same semantic added/removed styling as recognized diffs.
- `git_diff` can return bounded unified diff content for unstaged, staged, path-filtered, and
  ref-to-ref comparisons.
- `git_diff` remains read-only and bounded by line/byte limits.
- Host `git diff` usage, if retained as a fallback, is documented with replacement triggers and
  unavailable-host behavior.
- Tests cover plain edit diff fragments, standard unified diffs, and non-diff prose false positives.

## Non-Goals

- No write-capable Git operation changes.
- No broad Git transport replacement in this story.

## Required Reads

- `crates/talos-tools/src/file_tools/write_edit_tools.rs`
- `crates/talos-tools/src/git.rs`
- `crates/talos-tui/src/widgets.rs`
- `crates/talos-tui/src/tool_display.rs`
- `docs/backlog/active/GIT-001-embedded-git-tools.md`

