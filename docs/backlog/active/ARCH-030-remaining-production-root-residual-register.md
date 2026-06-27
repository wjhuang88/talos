# ARCH-030: Remaining Production Root Residual Register

**Status**: Planned
**Priority**: P3
**Created**: 2026-06-28
**Parent**: Two-month architecture optimization M10/M11
**Selected iteration**: Not selected

## Problem

The 2026-06-27/28 architecture optimization cycle reduced several production roots, but a small set
of large modules remains. Some are still large because they own behavior-sensitive flows, host-tool
fallbacks, SQLite schemas, or test-coupled workflows. Continuing to split them blindly would create
more risk than value.

These roots need explicit ownership so future agents do not treat them as invisible debt or perform
speculative rewrites without concrete acceptance criteria.

## Residual Roots

| Root | Current Evidence | Next Safe Slice | Activation Trigger |
|---|---|---|---|
| `crates/talos-cli/src/mode_runners.rs` | 1500 lines after ARCH-024/I069; remaining mode orchestration is still large. | Split one mode orchestration or lifecycle helper behind stable `run_*` exports. | Next CLI feature touches mode setup, session lifecycle, model lifecycle, or inline/TUI handoff. |
| `crates/talos-tui/src/app.rs` | 1005 lines after ARCH-025/I070; frame/input/cursor flows are visual-risk sensitive. | Extract one frame/input helper with screenshot or focused TUI state tests. | Next TUI work touches frame rendering, cursor placement, bottom panel, or approval UI. |
| `crates/talos-session/src/sqlite.rs` | 983 lines; SQLite schema, FTS search, fork metadata, and tests are in one module. | Split schema/migration SQL or fork/query helpers without changing SQL semantics. | Next session-index feature touches SQLite schema, search, fork records, cleanup, or maintenance. |
| `crates/talos-exploration/src/lib.rs` | 958 lines after ARCH-029/I074; store SQL and citation validation remain together. | Split schema/migration SQL or citation validation helpers. | Next exploration storage feature touches schema, claims, syntheses, FTS, or citation checks. |
| `crates/talos-tools/src/git.rs` | 868 lines; read-only gix tools and write-capable host-git fallback tools share one module. | Split read-only tool group or host-git write helpers while preserving permission metadata. | Next Git tool feature touches push/pull/checkout/write operations or host fallback behavior. |
| `crates/talos-provider/src/openai.rs` | 848 lines after ARCH-028/I073; request assembly is split but SSE parsing/retry remain in root. | Split SSE stream parser or retry/error mapping helpers with provider tests. | Next provider protocol feature touches OpenAI streaming, usage extraction, retry, or tool-call chunks. |
| `crates/talos-provider/src/lib.rs` | 833 lines; Anthropic provider root still combines request assembly, transport, and stream parsing. | Split Anthropic request assembly or stream parser. | Next Anthropic protocol feature touches cache-control, thinking fields, request body, or stream parsing. |
| `crates/talos-exploration/src/ingestion.rs` | 799 lines; ingestion, chunking, extraction, and synthesis workflows are still together. | Split chunking helpers or synthesis builder. | Next ingestion feature touches fetched content, chunk budgets, claim extraction, or synthesis formatting. |

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
