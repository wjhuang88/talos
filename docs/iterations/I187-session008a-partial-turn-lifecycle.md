# Iteration I187: SESSION-008-A Partial-Turn Lifecycle Decision

> Document status: Complete
> Published plan date: 2026-08-11
> Planned objective: choose the lifecycle owner, durable incomplete-turn representation, replay contract and compatibility strategy required before bounded runtime shutdown can be implemented.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: a runnable characterization check against current cancellation/error paths plus a Proposed ADR that names one atomic/idempotent partial-turn contract and gates SESSION-008-B.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 implementation session 2026-08-11 |
| Work Slice | Implement only SESSION-008-A / I187: characterize every interrupted/provider-error/cancellation ownership path and produce the lifecycle, durable incomplete-turn, replay/context and TLOG compatibility decision. No SESSION-008-B, RUNTIME-005, TOOL-024, permission, TUI, provider, or successful-turn behavior change. |
| Claimed At | 2026-08-11 |
| Source Issue | #45 |
| Governance Claim PR | #194 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Claim merge `5bb83f80b7dd7216ed83ee69fd4de0ef954c32f7`; PR #195 final head `46549e82436dd7344a37604b5e7d7ce8e44350ca` passed CI `31553007431`, received independent approval `5261130488`, passed merge-time CAS and merged as `e288afb5d97026f7ccb3ce0f519a4a81f99fe104`. ADR-058 acceptance remains bound to independent review of the closeout head that changes its status. |
| Implementation PR | #195 |
| Last Updated | 2026-08-12 |
| Handoff / Release Condition | None for I187/SESSION-008-A. SESSION-008-B requires its own effective claim after ADR-058 acceptance; RUNTIME-005 remains blocked on B. |

Completion Commit: `e288afb5d97026f7ccb3ce0f519a4a81f99fe104`

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
| 2026-08-12 | Merge | PR #195 final head `46549e82` passed CI `31553007431`, independent review `5261130488` and merge-time CAS, then merged as `e288afb5`. The closeout proposes ADR-058 Accepted without authorizing SESSION-008-B. |

## Verification Evidence

- Claim merge `5bb83f80b7dd7216ed83ee69fd4de0ef954c32f7` establishes ownership.
- `docs/reference/I187-SESSION-008-PARTIAL-TURN-CHARACTERIZATION.md` maps current Success, Error, Cancelled, panic, persistence and recovery paths to exact source owners and existing executable fixtures.
- Focused current-behavior fixtures passed for the legacy closed provider-error prefix, both durable failed-turn gaps, Success atomic/idempotent redaction, and explicit outcome-marker recovery; each selected command ran one test.
- `scripts/validate_project_governance.sh .` and `bash scripts/validate_collaboration_claims.sh .` passed with 0 warnings; `git diff --check` passed.
- PR #195 final head `46549e82436dd7344a37604b5e7d7ce8e44350ca` passed CI
  `31553007431`, both governance validators and independent review `5261130488`; merge
  `e288afb5d97026f7ccb3ce0f519a4a81f99fe104` contains only the decision, characterization and
  synchronized governance facts, with no Rust/Cargo/runtime change.

## Completion Evidence

- Completion Commit: `e288afb5d97026f7ccb3ce0f519a4a81f99fe104`.

## Residuals

- SESSION-008-B becomes separately claimable only after this closeout receives independent review
  accepting ADR-058; no B implementation is included or authorized here.
- RUNTIME-005-A/B/C and Issue #49 remain blocked until SESSION-008-B completes.

## Retrospective

- The characterization separated current behavior from the desired durable contract before any
  schema or API implementation. This keeps ADR acceptance auditable without treating a proposal
  review as implementation authority.
