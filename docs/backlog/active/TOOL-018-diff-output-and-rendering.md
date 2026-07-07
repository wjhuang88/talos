# TOOL-018: Diff Output And Rendering Reliability

| Field | Value |
|---|---|
| Story ID | TOOL-018 |
| Priority | P1 |
| Status | Complete (FS10: scrollback diff rendering + git_diff unified diff content) |
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
- **`git_diff` unified diff content**: `GitDiffTool::execute_inner` now produces real unified diff
  output using `similar::TextDiff::unified_diff()` with `--- a/`/`+++ b/` headers and `@@` hunk
  markers. For each changed file, old content is retrieved from the HEAD tree via
  `repo.rev_parse_single("HEAD:path")`, new content from the worktree, and the result is bounded
  by `max_lines`. Binary/unreadable files fall back to a simple `diff -- {path}` listing.
- Tests: 3 scrollback diff tests (edit fragments, unified diffs, prose false positives) + 1
  `git_diff` integration test verifying `diff --git`/`---`/`+++`/`-`/`+` content.

### Residuals

- **Background coloring**: the scrollback path (`HistoryAttrs`) does not support background colors,
  so `DIFF_ADDED_BG`/`DIFF_REMOVED_BG` from `widgets.rs::render_diff()` cannot be replicated. The
  foreground green/red distinction is the primary visual signal and is fully functional.
- **Staged vs unstaged filtering**: `git_diff` accepts a `staged` parameter but currently always
  compares HEAD vs worktree (all changes combined). Separate HEAD-vs-index (staged only) filtering
  is a future enhancement.
- **Path-filtered and ref-to-ref comparisons**: the acceptance mentions path-filtered and
  ref-to-ref comparisons; these are not yet implemented. The current implementation covers
  unstaged unified diff content, which is the primary use case.

