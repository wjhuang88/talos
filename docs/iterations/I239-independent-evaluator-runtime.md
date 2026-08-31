# Iteration I239: Independent Evaluator Runtime And Evidence Boundary

> Document status: Review / Claimed
> Published plan date: 2026-08-31
> Planned objective: implement WORK-001-D/P3 as a separately enforced evaluator boundary that
> consumes Validation evidence without granting evaluator, validator or executor self-certification.
> MVP deliverable: a runnable integration fixture demonstrates fresh evaluator context, read-only
> admission, exact-revision binding and explicit safe outcomes for every evaluator failure class.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 Sol mainline WORK-001 session |
| Work Slice | WORK-001-D / I239 P3 only: fresh evaluator context, read-only admission, Validation-001 evidence consumption, exact-revision binding and explicit fail-closed outcomes. |
| Claimed At | 2026-08-31 |
| Source Issue | #29 |
| Governance Claim PR | #447 |
| Authorization Mode | Independent review |
| Authorization Evidence | WORK-001-C/I238 Complete on `main@76c9b7fd`; ADR-061; VALIDATION-001 evidence contract; exact-head governance CI and independent review for PR #447 |
| Implementation PR | #448 (latest candidate includes evidence-boundary fix) |
| Last Updated | 2026-08-31 |
| Handoff / Release Condition | Claim/activation is proposed by PR #447 and becomes effective only after merge; P4 remains separately governed. |

## Published Baseline

### Selected Story

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| WORK-001-D | WORK-001 | Active / Claimed (proposed; ineffective until PR #447 merges) | WORK-001-C/I238 Complete; ADR-061; VALIDATION-001 | Independent evaluator runtime boundary with safe evidence consumption and no self-certification. |

### Non-Terminal Inventory And Disposition

| State | Iterations / owners | Disposition |
|---|---|---|
| Active | None on current `main@76c9b7fd` | I239 activation is proposed only by this claim; no existing active implementation is displaced. |
| Review | None on current main | No review work is bypassed. |
| Planned / Unclaimed | I207, I208 | Preserve unclaimed steering children; do not activate or transfer their scope. |
| Blocked | WORK-001-E/P4 and unrelated blocked owners | Keep blocked pending I239 completion and its own claim. |
| Paused | I164 | Superseded startup target; do not restore. |

Open archival Draft recovery PRs #120/#121 remain excluded. This branch contains only the proposed
governance claim; implementation starts from the claim merge or a later `main` head.

## Scope

- Establish a fresh evaluator context and admission boundary around P2 evaluation subjects.
- Keep evaluator tools read-only by default and preserve existing permission enforcement.
- Import Validation-001 records as integrity-checked evidence references, never as verdict authority.
- Produce criterion-level reports through the P2 API and map timeout, cancellation, provider error,
  malformed output, unavailable evidence and stale revision to explicit non-PASS outcomes.
- Add integration tests and update `docs/reference/WORK-EVALUATION-API.md`.

## Non-Goals

- No Mission final gate, Delivery, UI projection, Desktop/Dashboard/GPUI or localization.
- No persistence migration, Todo behavior change, SESSION-009 multi-client work or `/auto` change.
- No permission/sandbox policy expansion, release, version, tag or publication.

## Planned Validation

- Focused evaluator, evidence and permission-boundary tests plus existing Work/Evaluation tests.
- Locked workspace check, Clippy, tests, format, both governance validators and `git diff --check`.
- Exact-head CI, independent runtime/security review and merge-time CAS after stable local convergence.

## Local Convergence Checkpoint (2026-08-31)

The P3 candidate adds `talos_agent::evaluator::IndependentEvaluator`, a tool-free provider assessor,
read-only evaluator admission, provenance-bound Validation evidence snapshots, outer deadline and
cancellation enforcement and P2 report revalidation. Focused evaluator tests cover malformed
output, assessor deadline/cancellation, valid report acceptance, side-effecting tool rejection and missing evidence
integrity. No persistence, Mission gate, UI, Desktop, Dashboard, permission expansion, release or
publication behavior is included.

The stable candidate changed-file inventory is six implementation/documentation files:
`crates/talos-agent/src/evaluator.rs`, `crates/talos-agent/src/lib.rs`,
`crates/talos-runtime/src/lib.rs`, `docs/reference/WORK-EVALUATION-API.md`, this iteration and its
WORK-001-D owner. No Dashboard, Desktop or unrelated governance authority is included.

## Activation Rule

This claim proposes Active/Claimed atomically in PR #447. Both remain ineffective until that record
reaches `main`; no implementation authority exists before the merge. Implementation starts from the
claim merge or a later `main` head.

## Completion Rule

Implementation must merge before owner-first closeout. The closeout must cite an already-existing
implementation merge SHA as `Completion Commit`; a status-only commit cannot self-certify completion.
P4 remains blocked until this closeout reaches `main`.
