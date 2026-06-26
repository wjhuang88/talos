# TUI-014: Grep Result Summary Rendering

**Status**: Refinement
**Priority**: P3
**Source**: User request 2026-06-26
**Iteration**: None yet

## Problem

`grep` renders all matching lines verbatim in the scrollback. A grep returning 50 matches
across 20 files produces 50 lines of output, pushing useful context off-screen. Other search
tools (`read`, `find_symbol`, `find_references`) already render a one-line summary instead.

## Scope

Add `grep` to the summary rendering path in `tool_display.rs`.

### Current state

`should_suppress_tool_result_content()` in `crates/talos-tui/src/tool_display.rs` classifies
tools into two sets:
- `ALWAYS_SUMMARIZE`: `read`, `list_symbols`, `find_symbol`, `find_references`, `http_request`,
  `web_search` → always shows one-line summary
- `THRESHOLD_SUMMARIZE` (> 30 lines): `glob`, `ls`, `list_imports` → summary only if output
  exceeds threshold

`grep` is in neither set — it always renders full content.

### Required behavior

Add `grep` to `THRESHOLD_SUMMARIZE`. When the shared threshold is exceeded, show a summary like:

```
   ⚠ grep matched 47 lines in 12 files, 3.2 KB
```

Short grep results (≤ threshold) continue to render inline as they do today.

### Threshold alignment with TUI-015

TUI-014 and TUI-015 share a single threshold constant (currently
`SUMMARIZE_OUTPUT_THRESHOLD_LINES = 30`). Above this threshold:
- Summarize-eligible tools (`read`, `grep`, `glob`, `ls`, `find_symbol`, etc.) → one-line summary
- Non-summarize tools (`bash`, `git_diff`, `edit`, etc.) → head+tail truncation (TUI-015)

This avoids the conflict where a THRESHOLD_SUMMARIZE tool's output is both "not long enough
to summarize" and "long enough to head+tail truncate" — a single threshold makes the decision
once, then routes to the appropriate display mode.

### Non-goals

- No change to the data returned to the model (full results preserved in message history).
- No change to `bash`, `git_diff`, or other tools (covered by TUI-015).
- No change to the grep tool itself — only the TUI display layer.

## Acceptance

- Given a grep returning ≤ 30 lines,
  When rendered in scrollback,
  Then all lines display inline as today.

- Given a grep returning > 30 lines,
  When rendered in scrollback,
  Then a one-line summary appears instead of the full output.

## Required Reads

- `crates/talos-tui/src/tool_display.rs` (`should_suppress_tool_result_content`, `suppressed_tool_result_summary`)
- `crates/talos-tui/src/app.rs` (scrollback rendering pipeline)
