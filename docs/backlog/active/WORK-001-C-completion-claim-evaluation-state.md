# WORK-001-C: Completion Claim And Evaluation State Model

| Field | Value |
|---|---|
| Story ID | WORK-001-C |
| Type | API / State Story |
| Parent Epic | WORK-001 |
| Priority | P0 |
| Status | Complete / Closed — Completion Commit `209931e5`; implementation PR #445 merged |
| Source | GitHub Issue #29; WORK-001 P2; Desktop prerequisite chain section 20.2 |
| Selected Iteration | I238 Active / Claimed |
| Depends On | WORK-001-B / I237 Complete; ADR-061 boundary; canonical `talos_core::work` domain |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline WORK-001 session |
| Work Slice | WORK-001-C / I238 P2 only: storage-neutral Completion Claim, typed Acceptance Criterion, exact-revision Evaluation subject/report/finding/verdict and deterministic staleness/transition rules in the shared Work Domain. No evaluator runtime, provider call, Validation execution, persistence, Mission final gate, UI, Desktop, Dashboard, permission, `/auto`, release or publication work. |
| Claimed At | 2026-08-31 |
| Source Issue | #29 |
| Governance Claim PR | #444 |
| Authorization Mode | Independent review |
| Authorization Evidence | Claim PR #444 exact head `821d8583ae27b55a5c45da55ecc9d0c5fa8cb110` passed CI `33341379928`, independent review `5472316362`, and merged to `main` as `7a6b5ed46170c744254b4283240ac263b57f87a9`. |
| Implementation PR | #445 — final head `cc2fa218`, merged as `209931e5` |
| Completion Commit | `209931e5bf56589e3ebee178c70e89a40e0c4db1` |
| Last Updated | 2026-08-31 |
| Handoff / Release Condition | Claim is effective after PR #444 merge; implementation must start from `main@7a6b5ed4` or later and remain within this P2 scope. |

## Identity / Goal / Value

Provide callers with one canonical, enforceable contract for claiming Goal completion and recording
criterion-level evaluation without allowing executor self-certification. P3 can then add an
independent evaluator runtime against stable P2 types instead of inventing state semantics inside a
provider or product surface.

## Scope

- Add typed Acceptance Criteria with stable identity, kind and required/optional semantics.
- Add an immutable Completion Claim that binds executor assertions and references to one exact
  Mission/Goal/workspace-content subject revision.
- Add criterion-level Evaluation results, evidence references, bounded findings and a deterministic
  aggregate verdict.
- Add explicit pending/evaluating/pass/fail/inconclusive/stale state transitions that reject direct
  executor completion and mismatched or older subject revisions.
- Make a previously passing Evaluation stale whenever a relevant bound subject revision changes;
  locale and presentation-only state are not part of that identity.
- Keep all types storage-neutral and serialization/schema-ready for later Runtime and persistence
  consumers; no second work-state authority is introduced.
- Document the public state contract and compatibility boundary for SDK consumers.

## Exclusions

- No independent evaluator Agent/runtime, provider selection, prompt/context construction or model
  call; those belong to WORK-001-D/P3.
- No validator execution or claim that VALIDATION-001 can issue a completion verdict; Validation
  remains an evidence producer.
- No Mission final delivery gate, coordinator, projection or end-to-end workflow; those belong to
  WORK-001-E/P4.
- No SQLite or other persistence migration, Todo behavior change, session custody change, public
  Runtime wiring, CLI/TUI command, Desktop/Dashboard/GPUI surface or localization work.
- No permission, sandbox, `/auto`, release, version, tag or publication change.

## Dependencies And Decision Constraints

- WORK-001-B/I237 Completion Commit `f2b0b5c7e5f5080c9c36f7b7a1993af4246f6f91`
  supplies the canonical Work Domain and Todo compatibility path.
- ADR-061 requires UUID subject identity, monotonic node revisions, exact Mission/Goal/workspace
  subject binding, one canonical work authority and locale-neutral evaluation identity.
- The goal-oriented workspace proposal requires executor claims to remain hints, independent
  criterion-level verdicts, and PASS to become stale after relevant mutation.
- `VALIDATION-001` may supply evidence references but cannot become the evaluator or completion
  authority.
- Any breaking change to existing public Work Domain APIs or any persistence behavior requires
  change control and a migration decision before implementation.

## Uncertainty And Validation Path

- **Confirmed:** I237 provides storage-neutral Mission/Goal/WorkUnit identity and monotonic revision
  fields in `talos-core`; no Evaluation types exist on the current main branch.
- **Confirmed:** current Validation records are evidence and do not own Goal verdicts.
- **Soft choice to validate in implementation review:** aggregate PASS requires every required
  criterion to pass; any required failure yields FAIL; otherwise the result is INCONCLUSIVE.
- **Assumption:** a storage-neutral library slice is sufficient for P2 because P3/P4 own runtime and
  reachability. Validate through a complete transition/staleness executable fixture and record this
  iteration as an explicit infrastructure-only exception rather than claiming shipped user flow.

## Acceptance For Behavior

- Given an executor and an exact Goal subject revision,
  when it submits a Completion Claim,
  then the claim enters evaluation-pending state and cannot directly set the Goal to Completed.
- Given criterion results for the exact claimed subject,
  when an Evaluation report is constructed,
  then the aggregate verdict is derived deterministically from required criterion verdicts and
  cannot contradict them.
- Given a PASS Evaluation,
  when any bound Mission, Goal or workspace-content revision changes,
  then the Evaluation becomes stale and no longer certifies completion.
- Given only a locale or presentation change,
  when staleness is checked,
  then the Evaluation identity and verdict remain unchanged.
- Given evidence produced by VALIDATION-001,
  when it is attached to a criterion result,
  then it remains a referenced fact and does not independently issue the Goal verdict.

## Acceptance For Technical Work

- [ ] Public types have documentation, serde and schema contracts without a new crate dependency
      cycle or a second mutable work authority.
- [ ] Unit/property fixtures cover valid and invalid subject binding, criterion aggregation,
      duplicate identity, illegal transitions, stale revisions and locale exclusion.
- [ ] A runnable state-machine fixture proves the full claim -> pending -> evaluating -> verdict ->
      stale/rework sequence without invoking an evaluator runtime.
- [ ] Existing `talos-core` WorkGraph/Todo compatibility tests remain green and no P1 behavior
      changes.
- [ ] Public API reference documentation explains executor/evaluator authority, revision binding,
      staleness and the P3/P4 boundary.
- [ ] Locked focused/workspace validation, both governance validators and `git diff --check` pass at
      the stable implementation candidate.
- [ ] Exact-head CI, independent architecture/API review and merge-time CAS are recorded before
      completion.

## State / Status Owners

- Story scope, acceptance, claim and completion: this file.
- Iteration execution and Published Baseline: `docs/iterations/I238-completion-claim-evaluation-state.md`.
- Parent dependency order: `docs/backlog/active/WORK-001-goal-oriented-work-evaluation-foundation.md`.
- Derived views: Board, Product Backlog, iterations README and manifest only.

## User-Facing Documentation

- `docs/reference/WORK-DOMAIN-TODO-COMPATIBILITY.md` remains the P1 compatibility reference.
- P2 implementation must add or update `docs/reference/WORK-EVALUATION-API.md`; it must state
  that no evaluator runtime, Mission delivery or Desktop product flow ships in this slice.

## Required Reads

- `AGENTS.md`
- `docs/sop/AGENT-COLLABORATION.md`
- `docs/sop/ITERATION-WORKFLOW.md`
- `docs/backlog/active/WORK-001-goal-oriented-work-evaluation-foundation.md`
- `docs/backlog/active/WORK-001-B-work-domain-and-todo-compatibility.md`
- `docs/iterations/I237-work-domain-and-todo-compatibility.md`
- `docs/decisions/061-canonical-work-domain-and-todo-migration.md`
- `docs/proposals/talos-desktop-goal-oriented-workspace.md`, sections 7, 8, 12 and 20
- `docs/reference/I196-WORK-001-A-CURRENT-STATE-MIGRATION-CONTRACT.md`
- `docs/backlog/active/RUNTIME-001-embeddable-agent-runtime-api.md`
- `docs/backlog/active/SESSION-009-multi-client-session-architecture.md`
- `docs/backlog/active/VALIDATION-001-internal-validation-service.md`
- `crates/talos-core/src/work.rs`

## Residual Destination

Evaluator runtime/context/evidence inspection remains WORK-001-D/P3. Mission completion/delivery
gating, UI-neutral projection and real end-to-end runtime reachability remain WORK-001-E/P4. Any
persistence or SDK wiring discovered during P2 requires a separately governed child or explicit
change control; it must not be absorbed into I238.
