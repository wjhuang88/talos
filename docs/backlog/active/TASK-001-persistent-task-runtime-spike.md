# TASK-001: Persistent Task Runtime Architecture Spike

| Field | Value |
|---|---|
| Story ID | TASK-001 |
| Type | Architecture and security spike |
| Priority | P2 |
| Status | Cancelled / Superseded (2026-11-07) — separate-engine proposal retired; durable recovery differences remain explicitly deferred below |
| Source | [GitHub Issue #38](https://github.com/wjhuang88/talos/issues/38) |
| Depends On | `RUNTIME-001`, `SESSION-004`, `PERM-005` |


**Status**: Cancelled / Superseded

## Current disposition — 2026-11-07

The maintainer requested closing #38 after comparing it with the existing Goal-oriented work
model. Retire the proposal for a parallel task/goal/execution system; future work must build on
shared Work/Goal/Mission, Session storage and the permission pipeline. This is a governance-only
scope disposition, not evidence that the original persistent multi-stage engine is delivered.
The original spike baseline remains below; I132 delivered a defer decision, not a runtime.

| Original scope | Current evidence / disposition |
|---|---|
| Goal/task identity, progress, completion and evaluation | [WORK-001](WORK-001-goal-oriented-work-evaluation-foundation.md) delivered shared Work/Goal/Mission, Todo compatibility and revision-bound evaluation / Mission gating. `crates/talos-core/src/work.rs` is explicitly storage-neutral. Reuse this domain, not a second Task authority. |
| User navigation, saved conversation recovery, evidence | [Desktop D6](DESKTOP-001-D6-durable-task-and-evidence.md) uses authoritative Session storage and Work projection. Transcript resume is not phase checkpoint recovery; absent durable evaluation is shown unavailable, not restored as PASS. |
| Running an individual background command | TOOL-024 / #59 provides process-scoped command jobs; it does not establish cross-restart task orchestration. |
| Permission-gated execution | PERM-006-A/B/C and I308 provide shared decision/admission/execution foundations. No previous approval may silently authorize a future resumed task operation. |
| Persistent phase recovery / autonomous scheduling | Not delivered by the evidence above; explicit deferred differences below. Goal-oriented UI/domain and the coding assistant's own Goal mode are not proof of a Talos persistent engine. |

## Retained differences and reactivation

This owner is the durable destination for the original unmet requirements. They remain **deferred,
unimplemented and unclaimed**, not silently transferred into completed WORK-001 or Desktop D6:

- Persisted multi-stage lifecycle and phase checkpoints, including step results, artifacts and
  recovery position; restart after phase 3 of a 5-phase task must resume without replaying effects.
- Explicit task-level resume/cancel/archive and retention behavior across sessions/restarts,
  with CLI/TUI projection of authoritative state and fresh permission evaluation at resume/fire time.
- Durable scheduling if selected by a concrete product requirement; current process/session-scoped
  jobs or follow-ups do not prove it. No daemon, automatic replay or direct-tool bypass is authorized.
- Concurrent durable tasks: ownership, cancellation, independent context/history and failure
  isolation, including recovery races and privacy boundaries.

When a maintainer selects one of these differences, create a separately claimed, runnable child
linked to this owner and update ADR-043 with storage/migration, recovery/idempotency, permission,
concurrency and cleanup decisions. Reuse existing runtime/domain/storage boundaries; do not create
an independent engine merely to reproduce Goal behavior. The original no-global-bus and permission
constraints remain in force. This closure neither activates the differences nor asserts they are
unnecessary.

## Closure evidence and validation

Baseline: `main@5b7d2c0a23b71237839e7a038f91a12d8590ad54`. WORK-001 records implementation
commits `f2b0b5c7e5f5080c9c36f7b7a1993af4246f6f91` (domain),
`209931e5` (completion/evaluation), `2ff8b9f4507537b41c912661745708e942d19d62`
(evaluator), and `29fb900e7f62afa37a83159b0d137efcc42b1d47` (Mission gate).
These are evidence for bounded foundations only, not Completion Commits for TASK-001's runtime.

Closure ledger: synchronize this owner, ADR-043 addendum, I132 residual pointer, Backlog, Board and
Issue matrix; preserve published baseline/history. Run both repository governance validators,
current skill validator and diff check; obtain independent exact-head governance/security-boundary
review and CI before merge, then close #38 as superseded. No code or fresh runtime test result is
claimed. Remaining capability differences stay in this owner's section above.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 |
| Work Slice | Governance-only #38 supersession and retained recovery-difference mapping; no runtime or policy change |
| Claimed At | 2026-11-07 |
| Source Issue | #38 |
| Governance Claim PR | Pending |
| Authorization Mode | Independent review |
| Authorization Evidence | Maintainer requested #38 closure; exact-head independent review and CI required before merge |
| Implementation PR | None |
| Last Updated | 2026-11-07 |
| Handoff / Release Condition | Terminal reconciliation effective on target merge; close #38 after merge; differences require fresh activation |

## Goal

Produce an ADR-ready recommendation for, or documented rejection of, a persistent resumable task capability before any engine is implemented.

## Scope

- Define the relationship between a task, agent turn, and model session.
- Evaluate Talos-owned checkpoint storage, crash recovery, cancellation, retention, and explicit resume.
- Define permission re-authorization for resumed write-capable actions.
- Record lifecycle, threat-model, and dependency-direction options in an ADR or reviewed defer/reject record.

## Acceptance

- A reviewable ADR or explicit defer/reject record states data model, storage boundary, resume semantics, permission re-authorization, and cleanup policy.
- The recommendation proves no global event bus, direct tool-execution bypass, or autonomous background process is required.
- Implementation work is split into separately scoped stories only after the decision is accepted.

## Non-Goals

- No task engine, scheduler, cron service, multi-agent orchestration, or automatic execution.
- No ApprovalBridge or current authorization-semantic change.

## Required Reads

- `docs/sop/LONG-RUNNING-TASK.md`
- `docs/decisions/006-event-architecture-boundary.md`
- `docs/backlog/active/RUNTIME-001-embeddable-agent-runtime-api.md`
- `docs/backlog/active/SESSION-004-binary-session-log-format.md`
- `docs/backlog/active/PERM-005-logical-tool-sandbox-enforcement.md`
