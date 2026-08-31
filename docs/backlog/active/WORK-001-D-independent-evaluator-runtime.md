# WORK-001-D: Independent Evaluator Runtime And Evidence Boundary

| Field | Value |
|---|---|
| Story ID | WORK-001-D |
| Type | Runtime / Evaluation Story |
| Parent Epic | WORK-001 |
| Priority | P0 |
| Status | Ready / Unclaimed |
| Source | GitHub Issue #29; WORK-001 P3; Desktop prerequisite chain section 20.4 |
| Selected Iteration | I239 (Planned / Unclaimed) |
| Depends On | WORK-001-C / I238 Complete; ADR-061; VALIDATION-001 evidence contract |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned until a separate I239 claim becomes effective |
| Claimed At | Not applicable |
| Source Issue | #29 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-31 |
| Handoff / Release Condition | A separate I239 claim must merge before implementation. Completion requires exact-head CI, independent runtime/security review, acceptance evidence and owner-first closeout. |

## Identity / Goal / Value

Provide a separately enforced evaluator runtime boundary that assesses an executor's
Completion Claim with fresh context and read-only authority by default. The evaluator may consume
Validation-001 records as evidence, but neither the executor nor the validator may self-certify a
Goal.

## Scope

- Define the evaluator admission and custody boundary against the P2 Completion Claim and
  Evaluation contracts.
- Construct a bounded fresh evaluation context from the exact subject revision and claim; executor
  reasoning is input context, never authoritative verdict evidence.
- Consume existing Validation-001 records as typed, provenance-preserving Evidence references.
- Enforce read-only evaluator tools by default and reject write, permission-grant, sandbox-fallback,
  and Desktop/Dashboard authority.
- Return deterministic, explicit outcomes for evaluator timeout, cancellation, provider failure,
  malformed output and unavailable evidence; none may become PASS implicitly.
- Add focused integration fixtures proving evaluator/executor separation, evidence provenance,
  exact-revision binding and safe failure behavior.
- Update the shared evaluation API documentation for the runtime boundary and migration handoff to
  P4.

## Exclusions

- No Mission final gate, Delivery object, UI-neutral projection or end-to-end product workflow;
  those belong to WORK-001-E/P4.
- No new persistence schema, Todo migration, SESSION-009 multi-client behavior or Desktop,
  Dashboard, GPUI or localization work.
- No change to the existing Validation-001 validator authority; it remains an evidence producer.
- No permission policy expansion, `/auto` behavior change, sandbox fallback, release, version,
  tag, publication or unrelated provider/runtime refactor.

## Dependencies And Decision Constraints

- WORK-001-C/I238 supplies the canonical `CompletionClaim`, `EvaluationSubject`, report, verdict and
  stale/rework transitions; do not duplicate them.
- ADR-061 and the Desktop prerequisite chain require evaluator independence, exact revision binding,
  locale-neutral identity and one shared work authority.
- VALIDATION-001 records may be referenced only with source identity and integrity metadata; they
  cannot issue a Goal verdict.
- Existing permission and tool wrappers remain the enforcement boundary. Any new public API or
  persistence behavior requires an ADR/change-control record before implementation.
- External/native/provider calls must be bounded and fail closed under AGENTS.md constraints.

## Acceptance For Behavior

- Given an executor claim, when evaluation starts, then the evaluator receives a fresh bounded
  context tied to the exact claim subject and cannot mutate the subject or grant permission.
- Given Validation-001 evidence, when it is attached to a criterion, then provenance and integrity
  are preserved and the validator is not treated as the evaluator.
- Given executor-provided reasoning or a malformed evaluator response, when a verdict is requested,
  then it cannot create PASS without criterion-level valid evidence.
- Given evaluator timeout, cancellation, provider failure, unavailable evidence or stale subject,
  when evaluation terminates, then the result is explicit non-PASS and the process remains alive.
- Given a read-only evaluator attempt to write, execute an unapproved tool, or alter permission,
  when admission occurs, then the operation is rejected and the claim remains unevaluated/reworkable.

## Acceptance For Technical Work

- [ ] Public evaluator/runtime and evidence types are documented and preserve P2 identity semantics.
- [ ] Integration fixtures prove fresh-context and authority separation, read-only admission,
      provenance checks, malformed-output rejection and all bounded failure outcomes.
- [ ] No evaluator result can bypass the P2 acceptance boundary or directly complete a Goal.
- [ ] Existing executor, Validation-001, Work Domain and Todo compatibility tests remain green.
- [ ] User/API documentation describes evaluator evidence, safe failures and the P4 handoff.
- [ ] Locked focused/workspace validation, governance validators and `git diff --check` pass at the
      stable candidate; exact-head CI and independent protected-scope review are recorded.

## State / Status Owners

- Story scope, claim and completion: this owner document.
- Iteration execution and baseline: `docs/iterations/I239-independent-evaluator-runtime.md`.
- Parent dependency order: `WORK-001-goal-oriented-work-evaluation-foundation.md`.
- Derived views: Board, Product Backlog, iterations README and manifest only.

## Residual Destination

Mission final evaluation, Delivery gating, UI-neutral projection and non-GPUI end-to-end closure
remain WORK-001-E/P4. Any persistence, provider capability or session-custody gap discovered here
requires a separately governed child or explicit change control.
