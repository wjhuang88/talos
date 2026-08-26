# Iteration I228: TOOL-024-D2 Interactive Projection And Cross-Platform Acceptance

> Document status: Review / Claimed
> Published plan date: 2026-08-26
> Planned objective: complete the real CLI/TUI projection and integrated cross-platform acceptance for the already-supervised background command contract.
> Baseline rule: preserve this target; changed objectives require a new iteration ID.
> MVP deliverable: a runnable CLI/TUI path that starts, reads, lists, cancels and shuts down supervised jobs with bounded, display-safe evidence on Unix and Windows.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 Sol mainline Issue #59 session |
| Work Slice | TOOL-024-D2 projection, documentation and integrated Unix/Windows acceptance only; no supervisor or permission production changes. |
| Claimed At | 2026-08-26 |
| Source Issue | #59 |
| Governance Claim PR | #402 |
| Authorization Mode | Independent review |
| Authorization Evidence | I222/B, I224/C and I226/D1-B Complete/Closed on current main; ADR-060 and ADR-068 Accepted. |
| Implementation PR | #403 |
| Last Updated | 2026-08-26 |
| Handoff / Release Condition | Claim #402 is effective on `main@da9a79cd`; implementation candidate requires fresh exact-head CI/review/CAS before merge. |

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| TOOL-024-D2 | TOOL-024 / Issue #59 | Ready / Unclaimed | I222, I224 and I226 Complete/Closed | Real CLI/TUI projection and cross-platform acceptance without changing the existing supervisor contract. |

### Scope

- Connect existing background-job receipts and `process` actions to the normal CLI/TUI tool-event projection.
- Document intentional background use, bounded cursor reads and cancellation.
- Prove foreground compatibility and integrated Unix/Windows lifecycle behavior.

### Non-Goals

- Supervisor, permission, Job Object, persistence, Dashboard, `/auto`, Desktop, release or publication changes.
- I223 deferred human-validation cleanup.

### Acceptance

- A supported background command returns a visible bounded receipt promptly through the real CLI/TUI path.
- `process` list/status/read/cancel remains session-owned, cursor-correct and display-safe.
- Foreground command behavior is unchanged.
- Timeout/cancel/exit/shutdown terminal states are visible exactly once.
- Unix and Windows integrated acceptance evidence is recorded.

### Planned Validation

- Focused CLI/TUI and runtime integration tests plus rebuilt binary smoke.
- Locked workspace check, clippy, tests, release preflight, governance validators and exact-head CI.
- Independent protected-scope review and merge-time CAS.

### Documentation To Update

- TOOL-024-D2 owner, TOOL-024 parent, Issue #59 long task, relevant CLI/tool usage docs, Board, backlog and iteration index.

### Risks And Rollback

- Risk: projection accidentally changes supervisor/permission semantics or leaks raw command/output data.
- Rollback: retain existing model-visible process contract and disable only the new CLI/TUI projection path; foreground and fail-closed platform behavior remain intact.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-08-26 | Claim preparation | Prepared from `main@7dd04afd`; I222/B, I224/C and I226/D1-B are Complete/Closed. Claim and activation remain ineffective until the governance PR merges. |
| 2026-08-26 | Claim activation and local convergence | Claim #402 merged as `da9a79cd`; local candidate now covers CLI/TUI start/process/terminal projection, bounded display-safe summaries and SDK guidance. Foreground behavior and supervisor contracts remain unchanged. |
| 2026-08-26 | Independent review correction | PR #403 review `5420911368` rejected exact head `f5dc3415` for duplicate terminal semantics, missing real event-chain coverage and a source-breaking `ToolResultDisplay` field. The next local candidate restores the public struct shape, makes `BackgroundJobTerminal` the only terminal display authority and adds platform-neutral production-chain tests before fresh exact-head evidence. |

## Verification Evidence

- Focused conversation/CLI/TUI event-chain tests pass locally; full workspace locked validation and a fresh stable PR #403 head remain pending.

## Completion Evidence

- Completion Commit: Pending.

## Variance And Residuals

- I223 / Issue #378 remains separately governed and is required before closing the parent Epic and Issue #59.

## Retrospective

- Pending implementation.
