# TASK-001: Persistent Task Runtime Architecture Spike

| Field | Value |
|---|---|
| Story ID | TASK-001 |
| Type | Architecture and security spike |
| Priority | P2 |
| Status | Deferred (2026-07-16, I132) — ADR-043: task runtime not implemented; reusable components identified but not a delivered capability |
| Source | [GitHub Issue #38](https://github.com/wjhuang88/talos/issues/38) |
| Depends On | `RUNTIME-001`, `SESSION-004`, `PERM-005` |

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
