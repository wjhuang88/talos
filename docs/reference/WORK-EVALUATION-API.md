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
