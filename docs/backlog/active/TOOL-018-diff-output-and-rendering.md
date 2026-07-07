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
- `git_diff` can return bounded unified diff content for unstaged, staged, and path-filtered
  comparisons.
- `git_diff` remains read-only and bounded by line/byte limits.
- Host `git diff` usage, if retained as a fallback, is documented with replacement triggers and
  unavailable-host behavior.
- Tests cover plain edit diff fragments, standard unified diffs, and non-diff prose false positives.

### Acceptance Change (2026-07-07, FS10 revision 2)

Original acceptance required ref-to-ref comparisons in addition to the above. Ref-to-ref diff
(enumerating changed files between two arbitrary refs and diffing their blobs) requires a gix
tree-diff or host-git `git diff --name-only` fallback that is out of scope for this frontline
package. Unstaged, staged, and path-filtered comparisons cover the primary review-before-commit
workflow. Ref-to-ref is deferred to a future iteration under a separate story.

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

- **Scrollback diff rendering for `edit`/`diff` tool results**: `tool_display.rs` detects diff
  content by tool name (`edit`, `diff`) or unified diff markers. `+` lines get green foreground,
  `-` lines get red foreground, `@@` hunk headers get accent foreground.
- **`git_diff` unified diff content**: uses `similar::TextDiff::unified_diff()` with
  `diff --git`/`--- a/`/`+++ b/` headers. Old content from HEAD blob via gix `rev_parse`.
- **`git_diff` staged mode**: `staged: true` compares HEAD vs index (via `:path` index rev syntax).
  `staged: false` (default) compares HEAD vs worktree (all changes).
- **`git_diff` path filter**: `path` parameter filters results to files whose path starts with the
  given prefix.
- **False-positive prevention**: non-diff tools only get diff styling when unified diff markers are
  present; prose with `-`/`+` bullet lines is NOT styled.
- Tests: 3 scrollback diff tests + 3 git_diff integration tests (unified diff, staged, path filter).

### Residuals

- **Background coloring**: `HistoryAttrs` does not support background colors; foreground green/red
  is the primary visual signal.
- **Ref-to-ref comparison**: formally deferred per Acceptance Change above.

