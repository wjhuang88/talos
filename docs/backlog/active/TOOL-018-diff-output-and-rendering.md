# TOOL-018: Diff Output And Rendering Reliability

| Field | Value |
|---|---|
| Story ID | TOOL-018 |
| Priority | P1 |
| Status | In Progress (FS10: scrollback diff rendering for edit/diff tools complete; git_diff unified content residual) |
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

## FS10 Execution Evidence (2026-07-07)

### Implemented

- **Scrollback diff rendering for `edit`/`diff` tool results**: `tool_display.rs` now detects
  diff content by tool name (`edit`, `diff`) or unified diff markers (`diff --git`, `@@`,
  `--- `, `+++ `). When diff-aware, each line is classified and styled:
  - `+` lines (not `+++`): green foreground (`semantic::TEXT_SUCCESS`)
  - `-` lines (not `---`): red foreground (`semantic::TEXT_ERROR`)
  - `@@` hunk headers: accent foreground
  - Other lines: default secondary styling
- **False-positive prevention**: non-diff tools (e.g., `bash`, `read`) only get diff styling when
  unified diff markers are present; prose with `-`/`+` bullet lines is NOT styled as diff.
- 3 tests cover plain edit diff fragments, standard unified diffs, and non-diff prose false
  positives. Full TUI suite: 254 tests pass.

### Residuals

- **`git_diff` unified diff content**: `git_diff` currently returns a file-changed list
  (`diff -- <path>` per file) via `gix` status API, not real unified diff content with `+`/`-`
  lines. Producing bounded unified diff for unstaged/staged/ref-to-ref comparisons requires
  deeper `gix` diff API work or a bounded host-`git diff` fallback (documented with replacement
  triggers per the acceptance). This is a real work item for a future iteration; the scrollback
  rendering is already ready to style it once `git_diff` produces unified diff output.
- **Background coloring**: the scrollback path (`HistoryAttrs`) does not support background colors,
  so `DIFF_ADDED_BG`/`DIFF_REMOVED_BG` from `widgets.rs::render_diff()` cannot be replicated. The
  foreground green/red distinction is the primary visual signal and is fully functional.

