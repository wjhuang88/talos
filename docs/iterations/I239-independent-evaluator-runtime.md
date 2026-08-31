# Iteration I239: Independent Evaluator Runtime And Evidence Boundary

> Document status: Planned / Unclaimed
> Published plan date: 2026-08-31
> Planned objective: implement WORK-001-D/P3 as a separately enforced evaluator boundary that
> consumes Validation evidence without granting evaluator, validator or executor self-certification.
> MVP deliverable: a runnable integration fixture demonstrates fresh evaluator context, read-only
> admission, exact-revision binding and explicit safe outcomes for every evaluator failure class.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | WORK-001-D / I239 P3 only; implementation is unauthorized until a finalized claim merges |
| Claimed At | Not applicable |
| Source Issue | #29 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | WORK-001-C/I238 Complete on `main@76c9b7fd`; ADR-061; VALIDATION-001 evidence contract |
| Implementation PR | Not started |
| Last Updated | 2026-08-31 |
| Handoff / Release Condition | Claim/activation must merge before implementation; P4 remains separately governed. |

## Published Baseline

### Selected Story

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| WORK-001-D | WORK-001 | Ready / Unclaimed | WORK-001-C/I238 Complete; ADR-061; VALIDATION-001 | Independent evaluator runtime boundary with safe evidence consumption and no self-certification. |

### Non-Terminal Inventory And Disposition

| State | Iterations / owners | Disposition |
|---|---|---|
| Active | I225 / TOOL-024-D1-A remains separately governed | Preserve its decision-only scope; no overlap with WORK-001-D. |
| Review | None on current main | No review work is bypassed. |
| Planned | I207, I208 and other unclaimed steering children | Preserve; do not activate or transfer their scope. |
| Blocked | WORK-001-E/P4; unrelated blocked owners | Keep blocked pending I239 completion and its own claim. |
| Paused | I164 | Superseded startup target; do not restore. |

Open archival Draft recovery PRs #120/#121 remain excluded. No implementation branch or claim is
created by this planned iteration.

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

## Activation Rule

This proposal is Planned/Unclaimed. It grants no implementation authority. A governance claim must
first establish the responsible actor, exact Work Slice, authorization evidence and activation on
the target branch. Implementation starts from that claim merge or a later `main` head.

## Completion Rule

Implementation must merge before owner-first closeout. The closeout must cite an already-existing
implementation merge SHA as `Completion Commit`; a status-only commit cannot self-certify completion.
P4 remains blocked until this closeout reaches `main`.
