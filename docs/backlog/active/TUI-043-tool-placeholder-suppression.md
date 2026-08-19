# TUI-043: Suppress Leaked Tool-Call Compatibility Placeholder

| Field | Value |
|---|---|
| Story ID | TUI-043 |
| Type | TUI / Bug Story |
| Priority | P1 |
| Status | Review / Claimed |
| Source | [GitHub Issue #111](https://github.com/wjhuang88/talos/issues/111) |
| Selected Iteration | I201 — Review / Claimed |
| Depends On | Existing OpenAI request placeholder; canonical TUI ordered-content lifecycle |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline planning session |
| Work Slice | I201/TUI-043 only: suppress an exact standalone tool-call compatibility marker at the TUI ordered-content boundary while preserving legitimate text, tool rows, ordering and persistence. No provider protocol, core Message, execution, permission, persistence, broad renderer or release changes. |
| Claimed At | 2026-08-19 |
| Source Issue | #111 |
| Governance Claim PR | #306 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | PR #306 merged to `main` as `78cb1ddd` from exact base `8069ea6a` after exact-head CI `32209314843`, independent Agent governance review `5336890794`, both validators and merge-time CAS. This effective claim authorizes only the bounded I201/TUI-043 implementation. |
| Implementation PR | #309 |
| Last Updated | 2026-08-19 |
| Handoff / Release Condition | Claim #306 is effective at merge `78cb1ddd`; implement only from that merge or later current `main`. Per-child CI, Agent technical review and CAS remain merge gates; eligible human review moves to VALIDATION-002/I211/Issue #302 while this Story stays Review. |

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

I201 is the selected runnable iteration. Its claim is effective through PR #306 merge `78cb1ddd`;
implementation must keep the pending-marker state local to the TUI-visible ordered-content boundary.

## 2026-08-19 Claim Preparation

The governance-only claim merged through PR #306 as `78cb1ddd` from exact base `8069ea6a` after
I197 implementation merge `d98f37e7`. No implementation branch existed before that claim became
effective.

## State / Status Owners

- Story status and acceptance: this file.
- Remote request state and discussion: GitHub Issue #111.
- Current operating view: `docs/BOARD.md`.
- Compact selection view: `docs/backlog/PRODUCT-BACKLOG.md`.

## User-Facing Documentation

Update user or SDK documentation only when observable behavior or a public integration contract changes.
Do not present this Story as shipped while implementation and deferred human validation remain open.

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

## 2026-08-18 Deferred Human Validation Timing

The maintainer selected the long-task Deferred Human Validation Mode. The exact/split/negative and
ordering acceptance remains unchanged. After an implementation passes exact-head CI, independent
Agent technical review and CAS, its natural-person suppression-safety review may be recorded as an
Issue #302 row for I211. TUI-043 remains Review until that row passes; the deferral cannot justify a
global phrase filter or provider/protocol scope expansion.

## 2026-08-19 Claim Activation Checkpoint

PR #306 final head `153e470f` merged to `main` as `78cb1ddd` after exact-head CI `32209314843`, both
governance validators, independent Agent review `5336890794` and merge-time CAS passed. No
Rust/Cargo or implementation change was included. TUI-043 is now `Active / Claimed`; implementation
may start only from `78cb1ddd` or later current `main` and remains limited to the published Work Slice.

## 2026-08-19 Implementation Review Checkpoint

Implementation commits `68f4fb7b` and `d1fef291` are published through PR #309 from branch base
`25fe1f0c`. Fourteen focused state/event tests cover both marker spellings, split chunks,
whitespace, preceding text, larger legitimate sentences, terminal flush, unconfirmed starts, no
blank replacement, multiple tool calls, and the rule that direct result/approval events cannot
confirm suppression. The full `talos-tui` suite, strict package Clippy, formatting, both governance
validators, `git diff --check` and release preflight passed. TUI-043 stays `Review`; final
exact-head PR gates and the Issue #302 / I211 natural-person suppression-safety row remain open.

## 2026-08-19 Implementation Merge Checkpoint

PR #309 final head `d8d414ce3f2d65c6859fa4f30566efb3ac94196c` passed exact-head CI `32220300200`,
independent Agent technical review `5338185591`, both governance validators and merge-time CAS,
then merged to `main` as `7f5a6df2122d9b5ed70e55e59281e3e4e127f18c`. This Story remains
`Review / Claimed` with `Completion Commit: Pending` until its Issue #302/I211 natural-person row
passes; the merge does not self-certify acceptance.
