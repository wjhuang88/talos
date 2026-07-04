# TUI-025: Tool Argument Line-Fit Display

**Status**: Complete (2026-07-04)
**Priority**: P3
**Source**: Maintainer request 2026-07-04

## Problem

Tool-call argument summaries were truncated with fixed per-field caps even when the visible line had
enough space to show more useful detail. This made commands and approval summaries harder to inspect
than necessary.

## Requirement

When rendering tool-call arguments in the TUI, show as much of the argument summary as the available
single-line space reasonably allows. If the summary cannot fit on one line, truncate with an
ellipsis instead of wrapping or overflowing.

## Scope

- Tool-call scrollback summaries generate a complete one-line argument summary first, then apply a
  single display budget.
- Approval panel state keeps the full argument summary; truncation happens at render time using the
  actual panel width.
- Legacy bubble widgets use the current render area width for argument truncation.

## Non-Goals

- Do not change model-visible tool arguments.
- Do not change exported transcript content.
- Do not wrap arguments across multiple lines.
- Do not change permission decisions or approval semantics.

## Acceptance

- Given a long but line-fit command argument, the summary displays the complete command.
- Given the same argument with a smaller line budget, the summary truncates with an ellipsis.
- Given approval state receives a long multibyte argument, it stores the full argument and leaves
  truncation to render time.

## Validation

- `cargo test -p talos-tui tool_args_summary_uses_available_budget_before_truncating`
- `cargo test -p talos-tui approval_state_preserves_full_multibyte_arguments`
