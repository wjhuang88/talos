# TOOL-020: Git Diff Ref-To-Ref Comparisons

| Field | Value |
|---|---|
| Story ID | TOOL-020 |
| Priority | P2 |
| Status | Planned |
| Source | `TOOL-018` Acceptance Change / FS16 review closure |
| Depends On | `TOOL-018`, `GIT-001` |

## Problem

`TOOL-018` delivered scrollback diff rendering and `git_diff` unified diff output for unstaged,
staged, and path-filtered worktree review. Its original acceptance also included ref-to-ref
comparisons, but that part was formally deferred because enumerating and diffing arbitrary refs
requires either a `gix` tree-diff implementation or a bounded host-`git diff` fallback design.

The deferred requirement must remain visible as planned follow-up work instead of being treated as
closed by the `TOOL-018` fallback.

## Acceptance

- `git_diff` exposes explicit ref comparison inputs without breaking existing `staged`, `path`, and
  `max_lines` behavior.
- Ref-to-ref output is bounded unified diff content with `diff --git`, `---`, `+++`, `@@`, `+`, and
  `-` markers.
- Path filtering works with ref-to-ref comparisons.
- Missing refs, binary or unreadable files, non-worktree repositories, and bare repositories degrade
  with clear errors.
- The implementation remains read-only.
- If host `git diff` fallback is used, unavailable-host behavior and the replacement trigger are
  documented before implementation.
- Tests cover ref-to-ref diff output, path-filtered ref-to-ref output, missing refs, and
  `max_lines` truncation.

## Non-Goals

- No write-capable Git operation changes.
- No `gix` upgrade unless separately authorized by dependency review.
- No host-Git dependency as the primary path without documented fallback behavior and replacement
  trigger.

## Required Reads

- `crates/talos-tools/src/git.rs`
- `docs/backlog/active/TOOL-018-diff-output-and-rendering.md`
- `docs/backlog/active/GIT-001-embedded-git-tools.md`
- `docs/decisions/007-process-hardening-unsafe.md`

