# Work Completion And Evaluation API

`talos-core::evaluation` is the storage-neutral contract for Goal completion claims. It does not
run an evaluator, persist records, invoke a provider, or change Goal state. Runtime and product
surfaces consume this contract in later WORK-001 phases.

## Authority boundary

An executor creates a `CompletionClaim` for one exact `EvaluationSubject`. The claim is always
evaluation-pending and contains only the executor's assertions and evidence references. It cannot
produce a Goal `Completed` state or a PASS verdict by itself.

An independent evaluator starts the state machine, inspects the subject, and submits an
`EvaluationReport`. A report must contain exactly one result for every criterion in the immutable
claim snapshot. Its aggregate `EvaluationVerdict` is derived deterministically:

- any required `Fail` produces `Fail`;
- otherwise any required `Inconclusive` produces `Inconclusive`;
- otherwise the result is `Pass` (optional criteria do not override required criteria).

Evidence and artifacts are references. Evidence from VALIDATION-001 can explain a result but never
becomes completion authority.

## Exact revisions and staleness

`EvaluationSubject` binds Mission and Goal `WorkIdentity` values plus a `WorkspaceRevision`.
Mission and Goal roles are validated explicitly. A verdict applies only to that exact tuple. If a
Mission, Goal, or workspace/content revision changes after a verdict, `observe_subject` transitions
the evaluation to `Stale`; callers must request rework and submit a new claim. Locale and other
presentation preferences are intentionally absent from the subject, so changing UI language does
not stale an otherwise unchanged evaluation.

## Lifecycle

```text
CompletionClaim -> Pending -> Evaluating -> Verdict
                                      \-> Stale -> Rework -> new claim
```

Only the evaluator-facing `accept_report` operation can create a verdict, and it is legal only
after `begin`. Reports for another claim or revision, duplicate criteria/findings, unknown IDs, and
malformed criteria are rejected. This is a state contract only; evaluator runtime, Mission final
gate, Delivery, persistence, SDK wiring and Desktop/Dashboard UI belong to later WORK-001 slices.

## Independent evaluator runtime (P3)

`talos_agent::evaluator::IndependentEvaluator` is the P3 runtime boundary around the core state
machine. It builds a fresh, bounded `EvaluatorRequest` from the immutable claim snapshot and
provenance-preserving Validation evidence references. Executor conversation/reasoning is not
authoritative input, and evaluator tools are admitted as `Read` or `Internal` only by default;
write, execute and network natures are rejected.

An `EvaluatorAssessor` returns one JSON `EvaluationReport`. The report is parsed and revalidated by
`EvaluationReport::new` and `Evaluation::accept_report`, including exact claim subject, criterion
coverage and derived aggregate verdict. Provider/tool use, malformed output, stale subjects,
unavailable evidence, cancellation and deadline expiry produce an explicit non-PASS failure and
never complete a Goal. `evaluate_with_cancellation` accepts a caller-owned cancellation token; the
outer runtime deadline also bounds assessors that ignore their supplied deadline.

Validation records remain evidence producers. They must carry a stable producer identity and an
integrity digest before entering the evaluator request; their status cannot directly issue a Goal
verdict. Mission final evaluation, Delivery gating and UI-neutral projection remain the separately
governed WORK-001-E/P4 boundary.

## Mission gate and UI-neutral projection (P4)

`talos_core::work::MissionGate` is a storage-neutral final gate. It reads the required Goal
identities, their existing revision-bound `Evaluation` values, and an independent
`MissionEvaluation`; it never mutates those inputs or creates a second work repository.

`MissionGate::evaluate` returns `MissionGateResult` with `DeliveryEligibility` and ordered
`WorkProjectionEvent` values. Delivery is eligible only when every required Goal has a current
`Verdict(Pass)` and the independent Mission evaluation also targets the exact Mission identity and
revision with `Pass`. Missing, stale, failed or inconclusive results produce an explicit
`DeliveryBlockReason` and `eligible: false`. The event list is presentation-neutral and can be
adapted by the existing CLI/TUI bridge; it contains no GPUI, locale, cursor or layout state.

The core regression fixture `work::tests::mission_gate_emits_deterministic_eligible_projection`
walks the non-GPUI happy path, while companion missing-Mission and stale-Goal tests prove that Goal
PASS alone and an old Goal revision cannot unlock Delivery. Desktop binding, persistence and
multi-client session behavior remain separate downstream work.
