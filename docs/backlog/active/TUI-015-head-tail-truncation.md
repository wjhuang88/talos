# TUI-015: Head+Tail Truncation For Long Tool Outputs

**Status**: Complete; retained lines changed to 3/3 on 2026-07-04
**Priority**: P3
**Source**: User request 2026-06-26 (Codex pattern); maintainer refinement 2026-07-04
**Iteration**: None yet

## Problem

When a tool output is NOT suppressed by the summary path (i.e., it's not `read`,
`find_symbol`, etc.), `build_tool_result_scrollback_lines()` renders **every line** of the
output into the scrollback. A 200-line `bash` output or a 150-line `git_diff` pushes 200/150
lines into the scrollback, flooding the viewport.

Codex solves this by showing the first few lines, an ellipsis with omitted count, then the
last few lines — giving the user enough context to understand what happened without flooding
the screen.

## Scope

Add head+tail truncation to `build_tool_result_scrollback_lines()` for non-suppressed tool
outputs that exceed a line threshold.

### Current state

`build_tool_result_scrollback_lines()` in `crates/talos-tui/src/tool_display.rs`:
1. If `should_suppress_tool_result_content()` → one-line summary
2. Else → renders every line, each truncated to 120 chars

There is no line-count cap on the "else" branch.

### Required behavior

Head+tail is the **fallback display mode** for non-summarize tools whose output exceeds the
shared threshold. The rendering pipeline becomes a single decision point:

```
output lines > SUMMARIZE_OUTPUT_THRESHOLD_LINES (30)?
  ├─ YES + tool in summarize set (read/grep/glob/ls/find_symbol/...) → one-line summary (TUI-014)
  ├─ YES + tool NOT in summarize set (bash/git_diff/edit/...)        → head+tail (this story)
  └─ NO → full render (unchanged)
```

Head+tail display format:
```
   ⚠ line 1 of output
   █  line 2 of output
   ...
   █  line 3 of output
   ⋯ 194 lines omitted
   █  line 198 of output
   ...
   █  line 200 of output
```

Design parameters:
- **Threshold**: shared with TUI-014, currently `SUMMARIZE_OUTPUT_THRESHOLD_LINES = 30`.
  Outputs ≤ threshold render fully as today.
- **Head lines**: first 3 lines.
- **Tail lines**: last 3 lines.
- **Separator**: `⋯ {N} lines omitted` in dim color, single line.
- Applies to all tools NOT caught by the summarize path.

### Non-goals

- No change to data sent to the model (full output in message history).
- No change to `/export` (full transcript preserved).
- No change to the summary path (suppressed tools stay suppressed).
- No change to the shared threshold or to the decision of which tools summarize versus head+tail
  truncate. This story only changes how many lines are retained once head+tail has already been
  selected.
- No scrollable inline viewer (future enhancement, not this story).

## Acceptance

- Given a tool output of ≤ 30 lines,
  When rendered in scrollback,
  Then all lines display as today (no change).

- Given a non-summarize tool output of 200 lines (e.g., `bash`),
  When rendered in scrollback,
  Then scrollback shows first 3 lines, `⋯ 194 lines omitted`, last 3 lines.

- Given a summarize-eligible tool output of 200 lines (e.g., `grep`),
  When rendered in scrollback,
  Then one-line summary appears (TUI-014 handles this, NOT head+tail).

- Given the head+tail retained-line setting changes,
  When the display decision is evaluated,
  Then the shared threshold, summarize-eligible tool list, non-summarize fallback path, and
  model-visible tool payload remain unchanged.

- Given `/export`,
  When transcript is exported,
  Then the full 200-line output is present (not truncated).

## Required Reads

- `crates/talos-tui/src/tool_display.rs` (`build_tool_result_scrollback_lines`, `should_suppress_tool_result_content`)
- `docs/backlog/active/TUI-014-grep-result-summary.md` (shared threshold design)
- Codex TUI reference (head+tail pattern)
