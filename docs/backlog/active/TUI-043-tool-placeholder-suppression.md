# TUI-043: Suppress Leaked Tool-Call Compatibility Placeholder

| Field | Value |
|---|---|
| Story ID | TUI-043 |
| Type | TUI / Bug Story |
| Priority | P1 |
| Status | Ready — I201 Planned / Unclaimed |
| Source | [GitHub Issue #111](https://github.com/wjhuang88/talos/issues/111) |
| Selected Iteration | I201 — Planned / Unclaimed |
| Depends On | Existing OpenAI request placeholder; canonical TUI ordered-content lifecycle |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #111 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-14 |
| Handoff / Release Condition | After the ordered TUI predecessor is dispositioned, establish an effective I201 claim on `main`; implement only from that claim merge or later current `main`. |

## Identity / Goal / Value

Prevent the OpenAI compatibility placeholder `Calling tools…` from becoming a standalone transcript row when the same assistant response proceeds to structured tool calls, while preserving legitimate text otherwise.

## Scope

- Buffer only an exact standalone Unicode-ellipsis or three-dot placeholder.
- Discard it when the next meaningful event is a structured tool call.
- Flush it normally when no tool call follows.
- Preserve tool rows, ordering, approval, error, and persistence behavior.

## Exclusions

- No provider request serialization change, core Message redesign, general phrase filtering, or tool protocol change.

## Dependencies

Existing OpenAI request placeholder; canonical TUI ordered-content lifecycle

## Decision Links And Constraints

- Match a complete standalone line only; never hide larger legitimate sentences.
- Do not persist or render an empty replacement row.
- The structured tool-call row remains authoritative and appears exactly once.

## Uncertainty And Validation Path

I201 is the selected runnable iteration. Implementation still requires its own effective claim and
must keep the pending-marker state local to the TUI-visible ordered-content boundary.

## State / Status Owners

- Story status and acceptance: this file.
- Remote request state and discussion: GitHub Issue #111.
- Current operating view: `docs/BOARD.md`.
- Compact selection view: `docs/backlog/PRODUCT-BACKLOG.md`.

## User-Facing Documentation

Update user or SDK documentation only when observable behavior or a public integration contract changes.
Do not present this Story as shipped while it remains Ready.

## Required Reads

- crates/talos-provider/src/openai_request.rs
- crates/talos-provider/src/openai_sse.rs
- crates/talos-tui/src/app.rs
- crates/talos-tui/src/app_stream.rs

## Acceptance For Behavior / Technical Work

- Tool-only turns suppress both observed placeholder variants and retain every structured call.
- Split chunks and surrounding whitespace are handled.
- Non-tool and larger-sentence uses remain visible.
- TUI ordering, approval, cancellation, failure, retry, and timeout tests remain green.

## Residual Destination

Broader provider capability negotiation or synthetic-status filtering requires a separate owner.
