# ARCH-030: Remaining Production Root Residual Register

**Status**: Planned
**Priority**: P3
**Created**: 2026-06-28
**Parent**: Two-month architecture optimization M10/M11
**Selected iteration**: Not selected

I093 activation note (2026-07-04): selected for release-readiness audit only. The residual roots
remain watchlist items; no decomposition slice is activated by I093 activation alone.

I093 A13 readiness result (2026-07-04): `docs/reference/REL-002-READINESS-REPORT-2026-07-04.md`
classifies REL-002 risk by residual root. Session SQLite and Git tool roots are the highest
self-bootstrap risks before Talos-primary continuity/git workflows expand.

## Problem

The 2026-06-27/28 architecture optimization cycle reduced several production roots, but a small set
of large modules remains. Some are still large because they own behavior-sensitive flows, host-tool
fallbacks, SQLite schemas, or test-coupled workflows. Continuing to split them blindly would create
more risk than value.

These roots need explicit ownership so future agents do not treat them as invisible debt or perform
speculative rewrites without concrete acceptance criteria.

## Residual Roots

Updated 2026-07-10 after four-month architecture cleanup plan completion.

| Root | Current Lines | Previous | Status | Notes |
|---|---|---|---|---|
| `crates/talos-cli/src/mode_runners.rs` | **672** ✅ | 2290 | Decomposed | Session handlers → session_handlers.rs (958), interactive mode → mode_interactive.rs (184), tests → mode_runners_tests.rs (315) |
| `crates/talos-tui/src/app.rs` | 1005 | 1005 | Unchanged | Frame/input/cursor flows are visual-risk sensitive |
| `crates/talos-session/src/sqlite.rs` | 986 | 983 | Unchanged | SQLite schema + FTS search + fork metadata |
| `crates/talos-exploration/src/lib.rs` | 958 | 958 | Unchanged | Store SQL and citation validation |
| `crates/talos-tools/src/git.rs` | **660** ✅ | 1285 | Decomposed | Write tools extracted to git_write.rs (454 lines) |
| `crates/talos-provider/src/openai.rs` | **313** ✅ | 2365 | Decomposed | SSE parsing extracted to openai_sse.rs |
| `crates/talos-provider/src/lib.rs` | **291** ✅ | 1677 | Decomposed | Request assembly + stream parsing extracted |
| `crates/talos-tui/src/state.rs` | **450** ✅ | 1469 | Decomposed | BottomPanelState extracted to panel_state.rs |
| `crates/talos-permission/src/lib.rs` | **451** ✅ | 1630 | Decomposed | Rule + resource types extracted; tests separated |
| `crates/talos-exploration/src/ingestion.rs` | 799 | 799 | Unchanged | Ingestion/chunking/synthesis together |

### Resolved Roots (below 800-line threshold)

| Root | Final Lines | Decomposition |
|---|---|---|
| `openai.rs` | 313 | openai_sse.rs (2065), openai_request.rs (262) |
| `lib.rs` (provider) | 291 | anthropic_request.rs (462), anthropic_stream.rs (961) |
| `state.rs` (tui) | 450 | panel_state.rs (537), state_tests.rs (500) |
| `lib.rs` (permission) | 451 | rule.rs (162), resource.rs (76), workspace_trust.rs (206), permission_tests.rs (970) |
| `git.rs` (tools) | 660 | git_write.rs (454), git_tests.rs (227) |
| `mode_runners.rs` (cli) | 672 | session_handlers.rs (958), mode_interactive.rs (184), mode_runners_tests.rs (315) |

### Remaining Over-Threshold Roots

| Root | Lines | Gap | Recommended Next Slice |
|---|---|---|---|
| `app.rs` (tui) | 1005 | 205 over | Extract frame/cursor/output queue helpers with visual-risk tests |
| `sqlite.rs` | 986 | 186 over | Split schema/migration SQL from fork/query helpers |
| `lib.rs` (exploration) | 958 | 158 over | Split schema/migration SQL from citation validation |
| `ingestion.rs` | 799 | At threshold | Split chunking helpers or synthesis builder |

## Acceptance Criteria

- [ ] A residual root is activated only as a new ARCH story/iteration with a runnable deliverable.
- [ ] The selected slice preserves public API and behavior unless a separate ADR or feature story
  authorizes behavior changes.
- [ ] Validation includes the targeted crate tests, workspace gates, governance validation, and
  before/after line counts.
- [ ] Duplicate-logic disposition is recorded for every activated residual slice.

## Validation Notes

This is a residual register, not an implementation story. Validation is governance consistency plus
future activation discipline.
