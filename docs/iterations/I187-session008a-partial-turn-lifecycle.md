# Iteration I187: SESSION-008-A Partial-Turn Lifecycle Decision

> Document status: Review
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
| Implementation PR | Pending |
| Last Updated | 2026-08-11 |
| Handoff / Release Condition | Obtain exact-head review of ADR-058 and the current-path matrix; accept the decision before separately claiming SESSION-008-B. |

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
| 2026-08-11 | Claim merge | PR #194 merged as `5bb83f80b7dd7216ed83ee69fd4de0ef954c32f7`; the bounded SESSION-008-A claim is effective. |
| 2026-08-11 | Decision implementation | ADR-058 and the I187 current-path characterization define one atomic terminal finalizer, closed safe-prefix admission, first-writer conflict semantics, restart projection and schema-compatible migration. No Rust behavior changed. |

## Verification Evidence

- Claim merge `5bb83f80b7dd7216ed83ee69fd4de0ef954c32f7` establishes ownership.
- `docs/reference/I187-SESSION-008-PARTIAL-TURN-CHARACTERIZATION.md` maps current Success, Error, Cancelled, panic, persistence and recovery paths to exact source owners and existing executable fixtures.
- Focused current-behavior fixtures passed for the legacy closed provider-error prefix, both durable failed-turn gaps, Success atomic/idempotent redaction, and explicit outcome-marker recovery; each selected command ran one test.
- `scripts/validate_project_governance.sh .` and `bash scripts/validate_collaboration_claims.sh .` passed with 0 warnings; `git diff --check` passed.
- Implementation-PR exact-head CI and independent review remain pending.

## Completion Evidence

- No completion evidence while ADR-058 is Proposed. Completion requires an existing decision/characterization commit, exact validation and the required review; a later status-only commit cannot certify itself.

## Residuals

- SESSION-008-B remains blocked until this decision is Accepted and its migration/compatibility consequences are explicit.
- RUNTIME-005-A/B/C and #49 remain blocked by the SESSION-008 chain.

## Retrospective

- Pending execution.
