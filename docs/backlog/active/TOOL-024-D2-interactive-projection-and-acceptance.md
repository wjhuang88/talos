# TOOL-024-D2: Interactive Projection And Cross-Platform Acceptance

> Document status: Active / Claimed (proposed; ineffective until the claim PR merges)

| Field | Value |
|---|---|
| Story ID | TOOL-024-D2 |
| Type | Product / CLI-TUI Integration |
| Priority | P1 |
| Status | Active / Claimed (proposed) |
| Source | [GitHub Issue #59](https://github.com/wjhuang88/talos/issues/59) |
| Selected Iteration | I228 Active / Claimed (proposed) |
| Depends On | TOOL-024-B/I222 Complete; TOOL-024-C/I224 Complete; TOOL-024-D1-B/I226 Complete |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 Sol mainline Issue #59 session |
| Work Slice | CLI/TUI projection of the existing supervised background-job start and process controls, user/model documentation, and integrated Unix/Windows acceptance only. No supervisor, permission, Windows ownership, persistence, Dashboard, `/auto`, release or Desktop changes. |
| Claimed At | 2026-08-26 |
| Source Issue | #59 |
| Governance Claim PR | To be backfilled before merge |
| Authorization Mode | Independent review |
| Authorization Evidence | I222/B, I224/C and I226/D1-B are Complete/Closed on current main; exact implementation boundaries are recorded in their owners and ADR-060/ADR-068. |
| Implementation PR | Not started |
| Last Updated | 2026-08-26 |
| Handoff / Release Condition | Claim and activation become effective only on target-branch merge. Implementation starts from that merge or later main; D2 must not modify `crates/talos-dashboard/**` or I213 authority. |

## Scope

- Expose the already-implemented `background: true` start receipt and `process` actions through the real CLI/TUI tool-event path.
- Preserve foreground command behavior when `background` is absent or false.
- Render display-safe job identity, lifecycle state, bounded output/cursor, timeout/cancel outcomes and explicit errors without leaking secrets or raw unbounded arguments.
- Add user/model guidance for intentional background use, bounded reads and cancellation.
- Add integrated Unix and Windows acceptance proving start returns promptly, process read/status/list/cancel work, and session shutdown reaps active jobs.

## Non-Goals

- No changes to `talos-agent`/`talos-tools` supervisor semantics, permission evaluation, Job Object ownership or process cleanup implementation.
- No persistence or restart recovery, scheduling, retries, PTY/stdin, Dashboard, `/auto`, Desktop, release, version, tag or publication work.
- No changes under `crates/talos-dashboard/**`, I213 owner files, PERM-006-D/E or Issue #378 cleanup.

## Acceptance

- Given a supported background command, when invoked through the real CLI/TUI path, then a concise job receipt appears without waiting for process exit.
- Given a returned job id and cursor, when `process` read/status/list/cancel is used, then the projection is bounded, display-safe, cursor-correct and session-owned.
- Given foreground input, when `background` is absent or false, then existing output and blocking behavior remain unchanged.
- Given timeout, cancellation, natural exit or session shutdown, when the result is projected, then the terminal state is explicit and no duplicate or unbounded event is shown.
- Given Unix and Windows supported hosts, when the integrated acceptance fixture runs, then start/read/cancel/shutdown evidence is recorded for both; unsupported platform behavior remains fail-closed.

## Required Validation

- Focused CLI/TUI/tool-event tests and real binary/runtime integration fixtures.
- Unix and Windows exact-head CI, including process-tree cleanup evidence supplied by I226.
- `cargo fmt --all -- --check`, locked workspace check/clippy/test, release preflight, both governance validators and `git diff --check`.
- Independent process/permission/API/TUI review bound to the final exact head and merge-time CAS.

## Completion Evidence

- Completion Commit: Pending.
- This owner must remain Review until an existing implementation merge proves the deliverable; a status-only closeout cannot self-certify.

## Residuals

- I223 / Issue #378 deferred human-validation cleanup remains separate and must be completed before closing TOOL-024 or Issue #59.
