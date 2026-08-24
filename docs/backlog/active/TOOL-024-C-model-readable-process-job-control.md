# TOOL-024-C: Model-Readable Process Job Control

> Document status: Complete / Closed — implementation merged via PR #386; owner-first closeout pending

| Field | Value |
|---|---|
| Story ID | TOOL-024-C |
| Type | Product / Tool / Runtime Story |
| Priority | P0 |
| Parent Epic | [TOOL-024](TOOL-024-background-command-jobs.md) |
| Source | [GitHub Issue #59](https://github.com/wjhuang88/talos/issues/59) |
| Selected Iteration | I224 Complete / Closed |
| Depends On | TOOL-024-A/I188 Accepted; TOOL-024-B/I222 Complete; RUNTIME-005 Complete; PERM-006-C/I221 Complete |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 Sol mainline Issue #59 session 2026-08-24 |
| Work Slice | Implement only the model-visible session-scoped `process` tool over the existing I222 manager: typed read/status/list/cancel actions, bounded cursor/output/wait responses, ownership and redaction checks, idempotent cancel, existing permission pipeline and runtime fixture/docs. Exclude I222 supervisor redesign, Windows Job Object/D1, TUI/D2, Dashboard/I213, I223, persistence, `/auto`, release/publication, Desktop and PERM-006-D/E. |
| Claimed At | 2026-08-24 |
| Source Issue | #59 |
| Governance Claim PR | #385 |
| Authorization Mode | Independent review |
| Authorization Evidence | Claim PR #385 exact head `12931fef1400f7ce53fe82f3d3453036d2227c56` passed CI `32699927266`, independent permission/security/API review `5391959581`, merge-time CAS, and merged as `ae009ce68f4f3f5d49803e7d8978a021c2c9d3da`. Implementation evidence is recorded below. |
| Implementation PR | #386 — merged as `60b0367cf749397bf1167e189e820e82e32baf03` |
| Completion Commit | `60b0367cf749397bf1167e189e820e82e32baf03` |
| Last Updated | 2026-08-24 |
| Handoff / Release Condition | C is closed with pre-existing implementation evidence. TOOL-024-D1/D2 and I223 remain separate; Windows remains fail-closed. |

Completion Commit: `60b0367cf749397bf1167e189e820e82e32baf03`. This pre-existing implementation
merge is the evidence; the closeout status commit cannot self-certify the behavior implementation.

## Goal And Deliverable

Add one model-visible `process` tool over the existing I222 session-owned background-job manager.
The runnable deliverable is a real runtime/agent path that can perform bounded `read`, `status`,
`list` and `cancel` actions for jobs owned by the current session, with stable job identity,
ordered output cursors, explicit truncation/eviction, bounded wait, and fail-closed ownership and
permission behavior.

## Scope

- Define typed process-tool input/output and stable action names: `read`, `status`, `list`, `cancel`.
- Register the tool through the existing Agent tool pipeline and the I222 manager instance.
- Enforce current runtime/session ownership; unknown and foreign IDs reveal no metadata.
- Return bounded display-safe summaries and ordered output events without cursor repetition.
- Clamp `max_bytes` and `wait_ms`; report truncation and `dropped_before` explicitly.
- Make cancel idempotent and ensure the existing supervisor performs termination/reap exactly once.
- Preserve foreground behavior, existing permission semantics, and B's Windows fail-closed behavior.
- Add focused unit/integration tests and a real non-test runtime fixture or transcript.
- Update model tool guidance and public runtime/tool documentation for the process contract.

## Non-Goals And Exclusions

- No Windows Job Object or Windows background spawn; TOOL-024-D1 owns that decision and code.
- No TUI, CLI, Dashboard, `crates/talos-dashboard/**`, I213/WEB-001 owner, or presentation redesign.
- No persistence or cross-restart recovery, PTY/stdin, arbitrary PID attachment, remote jobs,
  scheduling, retry policy, or autonomous follow-up turns.
- No `/auto`, PERM-006-D/E, release/version/tag/publication, Desktop, or new global event bus.
- Do not change the I222 supervisor contract beyond the smallest public seam required by C; any
  authority overlap or schema-breaking change pauses for change control.

## Acceptance

- [x] A real `process(read)` call returns only events after the supplied cursor and advances it.
- [x] `status` reports every terminal state and safe exit metadata without replaying output.
- [x] `list` is session-scoped, bounded, stable, and redacts environment/secrets/raw arguments.
- [x] `cancel` is idempotent, ownership-checked, bounded, and reaps through the I222 manager.
- [x] Unknown/foreign IDs fail closed without revealing whether a job exists.
- [x] Per-read bytes, wait duration, job count and retained output are hard bounded and explicit.
- [x] Permission denial/rejected admission starts no process and creates no control authority.
- [x] Foreground and I222 background start behavior regressions remain green.
- [x] Unix runtime fixture exercises start, read, status and cancel through the actual tool registry.
- [x] Public tool/model guidance documents bounded reads, non-busy polling and explicit cancellation.
- [x] Focused locked tests, full workspace checks, Clippy, release preflight, governance validators,
  exact-head CI and independent security/API review pass.

## Validation And Review Gates

The implementation owner must record changed-file inventory and prove no Dashboard/I213 overlap.
Because this is process and permission-adjacent, an independent Agent-role permission/security/API
review is mandatory and must bind the exact implementation head. Local convergence is required
before the first stable push; substantive remote corrections invalidate prior CI/review evidence.

## Rollback And Residuals

If any ownership, bounding, cancellation or redaction invariant fails, keep TOOL-024-C in Review or
Blocked and disable process-tool registration; do not weaken I222. Windows remains fail-closed and
all residual manual/device rows remain with I223 / Issue #378.

## State / Status Owners

- Story status and acceptance: this owner.
- Iteration execution: `docs/iterations/I224-tool024c-model-readable-process-job-control.md`.
- Parent sequencing: `docs/backlog/active/TOOL-024-background-command-jobs.md`.
- Deferred evidence: `docs/iterations/I223-issue59-deferred-human-validation-cleanup.md` and Issue #378.

## Execution Checkpoint (2026-08-24)

Implementation locally converged from claim merge `ae009ce68f4f3f5d49803e7d8978a021c2c9d3da`.
The current candidate adds only the session-owned `process` tool and supervisor projection seam;
Windows, Dashboard/I213, TUI, persistence, release and I223 remain excluded. Focused agent checks
and the real AppServerSession registration fixture pass. `talos-tools` and `talos-runtime` locked
tests pass; workspace library validation reached the macOS Seatbelt tests, which are blocked in
this host by `sandbox_apply: Operation not permitted`. Full exact-head CI and independent review
remain required before implementation merge.
Stable implementation commit: `dcdffc56` plus launch-cancel reap fix `eabb7d3a`; implementation PR #386 is open. Its exact head is resolved by PR API and bound by CI/review evidence rather than self-referential owner text.
Owner status is Review / Claimed pending exact-head CI and independent permission/security/API review.

The preceding execution checkpoint is a preserved pre-merge record. It is superseded by the
Completion Checkpoint below; current status and evidence are authoritative there.

## Completion Checkpoint (2026-08-24)

PR #386 exact head `d42c060d618e61218c4c1efe0651e74830807256` was based on
`ae009ce68f4f3f5d49803e7d8978a021c2c9d3da`, passed exact-head CI `32719779528` (5/5), and received
independent permission/security/API/process approval `5394777902`. It merged as
`60b0367cf749397bf1167e189e820e82e32baf03` after merge-time CAS. Focused locked tests, full release
preflight and both governance validators passed locally. The delivered scope is complete; no
Dashboard/I213, Windows D1/D2, release/publication, persistence or I223 authority was added.
TOOL-024-C is Complete / Closed. A status-only closeout commit is not its Completion Commit.
