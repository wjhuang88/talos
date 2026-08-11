# Iteration I187: SESSION-008-A Partial-Turn Lifecycle Decision

> Document status: Planned
> Published plan date: 2026-08-11
> Planned objective: choose the lifecycle owner, durable incomplete-turn representation, replay contract and compatibility strategy required before bounded runtime shutdown can be implemented.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: a runnable characterization check against current cancellation/error paths plus a Proposed ADR that names one atomic/idempotent partial-turn contract and gates SESSION-008-B.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 implementation session 2026-08-11 |
| Work Slice | Implement only SESSION-008-A / I187: characterize every interrupted/provider-error/cancellation ownership path and produce the lifecycle, durable incomplete-turn, replay/context and TLOG compatibility decision. No SESSION-008-B, RUNTIME-005, TOOL-024, permission, TUI, provider, or successful-turn behavior change. |
| Claimed At | 2026-08-11 |
| Source Issue | #45 |
| Governance Claim PR | #194 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer direction authorizes this non-overlapping #49 prerequisite to proceed while I185/I186 review and terminal acceptance are batched for closeout; exact-head governance validation and no-overlap CAS remain required before merge. |
| Implementation PR | Not started |
| Last Updated | 2026-08-11 |
| Handoff / Release Condition | Merge #194 to establish ownership, then trace every cancel/error owner and produce the Proposed ADR and compatibility fixture. Independent review and ADR acceptance are a closeout gate before SESSION-008-B activation. |

## Published Baseline

- Selected story: `SESSION-008-A` from `SESSION-008` / Issue #45.
- Required reads: ADR-039, ADR-042, the legacy session error path, durable embedded session abort path, and current turn cancellation/completion arbitration.
- Scope: decision and characterization only; no durable schema migration or successful-turn behavior change.
- Non-goals: provider retry, side-effect replay, hidden/reasoning persistence, TUI changes, and RUNTIME-005 implementation.
- Acceptance: one lifecycle owner, one status-bearing atomic/idempotent operation, one replay/context projection, explicit TLOG compatibility, and a deterministic race matrix.
- Documentation: update the SESSION-008 owner and the runtime/session lifecycle reference if the Proposed ADR changes an integration contract.

## Non-Terminal Coordination Record

The maintainer directed that already-started I185/I186 review and terminal acceptance work may be
batched for closeout while this non-overlapping session decision slice proceeds. This is a priority
and sequencing change, not an authorization to bypass the independent review required for security-
sensitive permission/process slices. I185/I186 remain separate owners and do not provide evidence for
SESSION-008.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-08-11 | Selection | SESSION-008-A selected as the first executable prerequisite for #49. The owner remains ineffective until the claim record is merged to `main`; implementation review is deferred to the coordinated closeout. |

## Verification Evidence

- Pending claim merge and decision characterization.

## Completion Evidence

- No completion evidence. Completion requires an existing decision/characterization commit, exact validation, and the required review before the owner can become Complete.

## Residuals

- SESSION-008-B remains blocked until this decision is Accepted and its migration/compatibility consequences are explicit.
- RUNTIME-005-A/B/C and #49 remain blocked by the SESSION-008 chain.

## Retrospective

- Pending execution.
