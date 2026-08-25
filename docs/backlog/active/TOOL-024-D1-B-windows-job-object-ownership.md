# TOOL-024-D1-B: Windows Job Object Process-Tree Ownership

> Document status: Review / Claimed

| Field | Value |
|---|---|
| Story ID | TOOL-024-D1-B |
| Type | Product / Process-Security Implementation |
| Priority | P1 |
| Status | Review / Claimed |
| Source | [GitHub Issue #59](https://github.com/wjhuang88/talos/issues/59) |
| Selected Iteration | I226 Review / Claimed |
| Depends On | TOOL-024-D1-A / I225 Complete; ADR-068 Accepted on `main@93ee3253` |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 Sol mainline Issue #59 session |
| Work Slice | Implement only the Windows D1-B private launcher required by ADR-068: assigned-before-exec Job Object ownership, allowlisted stdout/stderr handle inheritance, kill-on-close descendant cleanup, typed fail-closed setup failures, and focused Windows tests. Preserve existing foreground identity, Unix launcher, Agent supervisor, process API and live-session boundary. |
| Claimed At | 2026-08-25 |
| Source Issue | #59 |
| Governance Claim PR | #393 |
| Authorization Mode | Independent review |
| Authorization Evidence | I225/ADR-068 closeout merged as `93ee3253`; exact decision commit `fca45c46`, CI `32797375011`, independent security review `5404361120`, closeout review `5405268380`. |
| Implementation PR | #394 — current candidate `b6c6e387` |
| Last Updated | 2026-08-25 |
| Handoff / Release Condition | Claim became effective on `main@d1f2a126` after PR #393 merge. Implementation starts from that merge or later. |

## Scope

- Add the minimum Windows-only API binding needed by ADR-068.
- Create/configure Job Object and allowlisted stdio handles before suspended process creation.
- Assign suspended process before resume; use kill-on-close for descendants.
- Integrate the existing supervisor contract without creating a second job registry.
- Add real Windows tests for marker ordering, child/grandchild cleanup, concurrent handle allowlisting, partial failures, cancellation, timeout and shutdown.

## Non-Goals

- No CLI/TUI projection or D2 work.
- No Dashboard/I213, permission schema, `/auto`, persistence, PTY/stdin, scheduler or restart survival.
- No release, version, tag, publication or Desktop work.
- No changes to Unix process groups or foreground PowerShell behavior.

## Acceptance

- Given an authorized Windows background request, when launcher setup succeeds, then the child is assigned before resume and only allowlisted stdio handles are inherited.
- Given assignment, resume, attribute-list, pipe, cancellation, timeout or shutdown failure, then no unowned child remains and a typed fail-closed result is returned.
- Given a child that creates a grandchild, when cancel/timeout/shutdown closes the Job, then both are terminated and reaped within the bounded deadline.
- Given concurrent process creation with unrelated inheritable handles, then the child cannot observe those handles.
- Existing foreground and Unix behavior remains unchanged.

## Required Validation

- Focused Windows launcher and supervisor tests.
- `cargo check --workspace --locked`, `cargo clippy --workspace --locked -- -D warnings`, `cargo test --workspace --locked`.
- `scripts/release_preflight.sh` and both governance validators.
- Independent Windows/process/unsafe/API review against ADR-068 and ADR-007.
- Exact-head Windows CI and merge-time CAS.

## Completion Evidence

- Completion Commit: Pending.
- The claim or status closeout cannot self-certify implementation completion.

## Residuals

- D2 and I223 remain separately governed after this story.
