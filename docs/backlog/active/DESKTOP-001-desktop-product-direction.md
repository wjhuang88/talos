# DESKTOP-001: Desktop Product Direction And Technology Boundary

| Field | Value |
|---|---|
| Story ID | DESKTOP-001 |
| Type | Product / Architecture Spike |
| Priority | P3 |
| Status | Deferred — proposal retained; no iteration selected |
| Source | [GitHub Issue #29](https://github.com/wjhuang88/talos/issues/29) |
| Selected Iteration | None |
| Depends On | RUNTIME-001 reusable runtime API; SESSION-009 multi-client model; permission and distribution decisions |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #29 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-01 |
| Handoff / Release Condition | Establish an effective claim and select an iteration before implementation. |

## Identity / Goal / Value

Preserve the desktop-product proposal as a governed architecture question without implying that a Tauri, WebView, or pure-Rust GUI implementation is authorized.

## Scope

- Compare Tauri/WebView, pure-Rust GUI, hybrid, and TUI-first continuation against repository constraints.
- Define core-runtime reuse, permission parity, packaging, update, and cross-platform support requirements.
- Produce an ADR/proposal before selecting a desktop implementation slice.

## Exclusions

- No desktop implementation, frontend framework adoption, or packaging pipeline in this Story.
- No weakening of Rust-first core ownership, permission, credential, or durable-session boundaries.

## Dependencies

RUNTIME-001 reusable runtime API; SESSION-009 multi-client model; permission and distribution decisions

## Decision Links And Constraints

- Desktop is a host/client surface above `talos-runtime`, not a second agent execution engine.
- Any JS/TS/WebView/native dependency requires explicit dependency and security review.
- Multi-client or reconnect behavior must consume SESSION-009 rather than invent connection-owned sessions.

## Uncertainty And Validation Path

Resume only when the maintainer selects a desktop outcome and a bounded technology-validation iteration. Recheck current GUI ecosystem and packaging constraints at that time.

## State / Status Owners

- Story status and acceptance: this file.
- Remote request state and discussion: GitHub Issue #29.
- Current operating view: `docs/BOARD.md`.
- Compact selection view: `docs/backlog/PRODUCT-BACKLOG.md`.

## User-Facing Documentation

Update user or SDK documentation only when observable behavior or a public integration contract changes.
Do not present this Story as shipped while it remains Deferred.

## Required Reads

- docs/backlog/active/RUNTIME-001-embeddable-agent-runtime-api.md
- docs/backlog/active/SESSION-009-multi-client-session-architecture.md
- docs/decisions/052-sdk-publication-and-composition-boundary.md
- crates/talos-runtime/

## Acceptance For Behavior / Technical Work

- A reviewed proposal identifies the selected host architecture and rejected alternatives.
- The selected design preserves runtime, permission, session, and credential ownership boundaries.
- A dedicated iteration and effective Collaboration Claim exist before production code begins.

## Residual Destination

Future implementation must use a new iteration; this Deferred owner remains the source for the product direction.
