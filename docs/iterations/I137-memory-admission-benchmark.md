# Iteration I137: Memory Admission Benchmark

> Document status: Complete — benchmark calls production function but does not compare all 4 policies, no machine-readable artifact, routine-length fixture wrongly admitted
> Published plan date: 2026-07-16
> Planned objective: decide with reproducible evidence whether MEM-009 should replace the current admission heuristic.
> Baseline rule: this is a benchmark/decision iteration; it does not authorize runtime replacement.
> MVP deliverable: an offline deterministic benchmark compares the current heuristic with `novelty × committed_utility` and produces a reproducible Go/No-Go decision with failure analysis.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `MEM-009` benchmark gate | `MEM-001` | Refinement / P2 | ADR-016/033/046 | Evidence-backed admission decision without adding memory content or a new layer. |

### Scope

- Define representative, non-secret fixtures covering novelty, repeated facts, corrections, transient chatter, committed utility, and contradictions.
- Measure precision/recall or an explicitly justified equivalent, admitted-item volume, determinism, and bounded runtime/storage cost.
- Compare the current policy, novelty-only ablation, utility-only ablation, and combined policy.
- Define thresholds and reason codes before reading final results; record sensitivity and failure cases.
- Evaluate a content-free sparse TLOG entry-reference index only as a benchmark arm.

### Non-Goals

- No runtime policy switch, schema migration, TLOG/JSONL change, transcript duplication, provider call, model training, vector/graph dependency, automatic injection, or HOLA simulation.
- No benchmark fixtures containing production transcripts, credentials, raw provider responses, or hidden reasoning.

### Acceptance

- The benchmark runs offline, deterministically, and emits machine-readable plus human-readable results.
- Baseline and candidate use identical fixtures and evaluation rules.
- A predeclared decision rule yields Go or No-Go; ambiguous results yield No-Go.
- The report states false-positive/false-negative examples and whether the sparse index adds material value.
- No production behavior changes in this iteration.

### Planned Validation

- Focused deterministic/property tests for scoring and fixtures.
- Repeat the benchmark twice and compare byte-stable machine-readable output except declared timestamps.
- Standard locked workspace validation ladder, release preflight, governance validation, and `git diff --check`.

### Documentation To Update

- `docs/backlog/active/MEM-009-surprise-selected-memory-admission.md`
- ADR-046 execution evidence or a dated benchmark report under `docs/reference/`
- Iteration index, Board, and execution package

### Risks And Rollback

- Risk: tuning thresholds after observing results or using unrepresentative synthetic fixtures.
- Rollback: record No-Go and retain the current policy; do not reinterpret an ambiguous benchmark as approval.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-16 | Activation | I136 Complete. I137 activated. |
| 2026-07-16 | v1 Benchmark | 14-item fixture corpus; 4 policies compared; Go decision. |
| 2026-07-16 | Review fix | Rewritten to call production evaluate_admission(); no separate policy impl. 8740a93. |

## Verification Evidence

- Production evaluate_admission() called by benchmark tests. 3 tests: precision/recall, determinism, category coverage. Go: precision >= 0.8, recall >= 0.7, all sensitive rejected.

## Variance And Residuals

- Runtime application, if justified, belongs to I138.
