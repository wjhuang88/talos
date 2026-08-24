# TOOL-024-C: Model-Readable Process Job Control

> Document status: Active / Claimed — proposed through governance PR #385; ineffective until merge

| Field | Value |
|---|---|
| Story ID | TOOL-024-C |
| Type | Product / Tool / Runtime Story |
| Priority | P0 |
| Parent Epic | [TOOL-024](TOOL-024-background-command-jobs.md) |
| Source | [GitHub Issue #59](https://github.com/wjhuang88/talos/issues/59) |
| Selected Iteration | I224 Active / Claimed through governance PR #385; ineffective until merge |
| Depends On | TOOL-024-A/I188 Accepted; TOOL-024-B/I222 Complete; RUNTIME-005 Complete; PERM-006-C/I221 Complete |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 Sol mainline Issue #59 session 2026-08-24 |
| Work Slice | Implement only the model-visible session-scoped `process` tool over the existing I222 manager: typed read/status/list/cancel actions, bounded cursor/output/wait responses, ownership and redaction checks, idempotent cancel, existing permission pipeline and runtime fixture/docs. Exclude I222 supervisor redesign, Windows Job Object/D1, TUI/D2, Dashboard/I213, I223, persistence, `/auto`, release/publication, Desktop and PERM-006-D/E. |
| Claimed At | 2026-08-24 |
| Source Issue | #59 |
| Governance Claim PR | #385 |
| Authorization Mode | Independent review |
| Authorization Evidence | Maintainer Issue #59 long-task objective selects the ordered C child after I222/B completion. Draft claim PR #385 proposes this bounded slice; claim and activation are ineffective until merge. Independent permission/security/API review, exact-head CI and merge-time CAS are required before implementation merge. |
| Implementation PR | Not started |
| Last Updated | 2026-08-24 |
| Handoff / Release Condition | Implementation starts only from the #385 claim merge or later `main`; owner must remain Review / Claimed until implementation evidence and protected review exist. TOOL-024-D and I223 remain separate. |

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

- [ ] A real `process(read)` call returns only events after the supplied cursor and advances it.
- [ ] `status` reports every terminal state and safe exit metadata without replaying output.
- [ ] `list` is session-scoped, bounded, stable, and redacts environment/secrets/raw arguments.
- [ ] `cancel` is idempotent, ownership-checked, bounded, and reaps through the I222 manager.
- [ ] Unknown/foreign IDs fail closed without revealing whether a job exists.
- [ ] Per-read bytes, wait duration, job count and retained output are hard bounded and explicit.
- [ ] Permission denial/rejected admission starts no process and creates no control authority.
- [ ] Foreground and I222 background start behavior regressions remain green.
- [ ] Unix runtime fixture exercises start, read, status and cancel through the actual tool registry.
- [ ] Public tool/model guidance documents bounded reads, non-busy polling and explicit cancellation.
- [ ] Focused locked tests, full workspace checks, Clippy, release preflight, governance validators,
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
