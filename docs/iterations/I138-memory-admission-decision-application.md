# Iteration I138: Memory Admission Decision Application

> Document status: Planned, blocked on I137 decision
> Published plan date: 2026-07-16
> Planned objective: apply I137's predeclared Go/No-Go result without exceeding ADR-046.
> Baseline rule: Go permits the minimal policy replacement below; No-Go permits evidence closure only and no runtime change.
> MVP deliverable: either a verified deterministic admission replacement with explainable reason codes, or a verified no-change closure showing why the current behavior remains.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `MEM-009` decision application | `MEM-001` | Blocked on benchmark | I137 Go/No-Go report | Apply the evidence without speculative expansion. |

### Scope

- Go branch: replace only the current keyword/message-length admission decision with deterministic novelty and committed-utility signals already available at the Runtime boundary.
- Go branch: expose bounded reason codes to diagnostics/tests without transcript content or sensitive metadata.
- Go branch: preserve evidence confidence, four-layer memory, ADD-only semantics, and default-off associative injection.
- No-Go branch: record closure, retain current runtime behavior, and identify the exact reversal trigger.
- Implement a sparse content-free TLOG reference index only if I137 separately passes its declared material-benefit threshold and dependency direction stays acyclic.

### Non-Goals

- No second transcript copy, fifth memory layer, provider/model call for admission, schema/TLOG change, raw reasoning persistence, automatic associative injection, or new dependency.
- No tuning against production user data during unattended execution.

### Acceptance

- The chosen branch exactly matches I137's predeclared rule and is traceable to its report.
- Go: candidate regressions, reason-code stability, privacy filters, bounded cost, and disabled-mode compatibility pass.
- No-Go: no production code or behavior changes; owner docs record the evidence and reversal trigger.
- Existing memory retrieval, consolidation, contradiction, permission, and session tests remain green.

### Planned Validation

- Focused memory/runtime tests and I137 benchmark replay.
- Runtime mock-provider proof for enabled and disabled memory behavior on the Go branch.
- Standard locked workspace validation ladder, release preflight, governance validation, and `git diff --check`.

### Documentation To Update

- MEM-009, ADR-016/046 only as execution evidence (do not rewrite decisions), README if behavior changes, iteration index, Board, and execution package

### Risks And Rollback

- Risk: benchmark overfitting or accidental sensitive metadata persistence.
- Rollback: disable/revert the candidate policy and retain the current heuristic; any storage/public-API need stops the run for maintainer review.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|

## Verification Evidence

- Blocked on I137.

## Variance And Residuals

- None at publication.
