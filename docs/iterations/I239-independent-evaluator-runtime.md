# Iteration I239: Independent Evaluator Runtime And Evidence Boundary

> Document status: Complete / Closed
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
| Implementation PR | #448, merged as `ac4ce436aca8681dbc2ea4e75560e2be999af242` |
| Last Updated | 2026-08-31 |
| Handoff / Release Condition | Complete; P4 remains separately governed and requires its own owner, iteration and claim. |

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

## 2026-08-31 Completion Checkpoint

I239 / WORK-001-D is Complete / Closed after implementation PR #448 merged to `main` as
`ac4ce436aca8681dbc2ea4e75560e2be999af242`. The implementation source head
`2ff8b9f4507537b41c912661745708e942d19d62` is an existing ancestor of that merge and is the
Completion Commit evidence; this status closeout is not used as its own evidence. Exact-head CI
`33359367715` passed all five jobs on the implementation head, and the independent Agent-role
runtime/security review recommendation was recorded at review `5063376615` (shared GitHub account
cannot represent natural-person identity separation). The final six-file inventory remained within
I239: evaluator runtime/API implementation and its owner/reference documentation only; no
Dashboard, Desktop, persistence, permission-policy, release or publication changes were included.

Completion Commit: `2ff8b9f4507537b41c912661745708e942d19d62`

P4 / WORK-001-E remains separately blocked until its own requirement intake, owner, runnable
iteration and effective claim are established.
