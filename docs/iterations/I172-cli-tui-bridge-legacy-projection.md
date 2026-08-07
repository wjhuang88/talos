# Iteration I172: CLI/TUI Bridge Legacy Projection Decomposition

> Document status: Active
> Published plan date: 2026-08-06
> Planned objective: extract one private legacy event/projection responsibility from `tui_bridge.rs` without changing bridge behavior.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: the existing CLI/TUI bridge runs the same legacy and structured-legacy event sequences through a private projection module, with the facade retaining all public and runtime entry points.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 architecture session 2026-08-06 |
| Work Slice | Private extraction of legacy `TurnEvent` and structured-legacy compatibility projection handlers only. |
| Claimed At | 2026-08-06 |
| Source Issue | None |
| Governance Claim PR | #140 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | No independent reviewer is currently available; exact-head CI, both governance validators, merge-time CAS, and no blocking review feedback are required. |
| Implementation PR | Not started |
| Last Updated | 2026-08-06 |
| Handoff / Release Condition | Claim merge precedes implementation branch; release if ordering/state equivalence cannot be proven. |

Before activation, follow `docs/sop/AGENT-COLLABORATION.md`. The claim is effective only after the
finalized record is merged into `main`.

## Published Baseline

### Selected Story

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| ARCH-034-R02 | ARCH-034 | Ready | I171/ARCH-034-D Complete; no overlapping claimant | A private legacy projection module behind the existing bridge facade. |

### Scope

- Move `handle_legacy_turn_event` and `handle_structured_legacy_projection` into one private module.
- Keep `BridgeTurnState`, `ProgressMode`, event matching, sequence checks, output text, and caller
  entry points behaviorally identical.
- Add focused source/layout or integration evidence that the facade delegates to the module.

### Non-Goals

- No new event bus, channel type, protocol, public API, UI behavior, dependency, session actor,
  persistence, custody, cancellation, or I169 lifecycle change.
- No extraction of structured turn ownership, attachments, skill commands, or runtime composition.

### Acceptance

- Given identical legacy and structured-legacy `SessionEvent` streams, when the bridge handles them,
  then output order, state transitions, sequence rejection, completion status, and queue snapshots
  remain identical.
- Given the existing `run_conversation_loop` entry point, when it receives a legacy turn event,
  then it delegates through the private module without changing channel or sender topology.
- The parent source file is a coordinator facade for this event family; no public path changes.

### Planned Validation

- `cargo fmt --all -- --check`
- `cargo check --workspace --locked`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo test --workspace --locked`
- `scripts/validate_project_governance.sh .`
- `bash scripts/validate_collaboration_claims.sh .`
- `git diff --check`
- Existing `talos-cli` bridge integration and source-layout tests covering legacy/structured event ordering.

### Documentation To Update

- ARCH-034-R02 owner and this iteration.
- `docs/iterations/README.md`, `docs/backlog/PRODUCT-BACKLOG.md`, and `docs/BOARD.md` after activation/completion.
- No user-facing feature documentation; behavior is intentionally unchanged.

### Risks And Rollback

- Risk: private visibility or ownership changes alter event ordering or state restoration.
- Rollback: revert the extraction commit; retain the pre-extraction facade and existing tests.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-08-06 | Planning | Claim-only preflight from `origin/main@56f419f7`; I158/I171 Complete, I159-I162 blocked, no overlapping implementation branch. |
| 2026-08-07 | Activation | Claim PR #140 passed exact-head CI run `31094244893`, merge-time CAS confirmed head `93f9a934` against base `56f419f7`, and merged at `46f72750`. Implementation branch starts from that effective claim; I172 is the sole Active iteration. |

## Verification Evidence

- Pending claim merge and implementation.

## Completion Evidence

- Completion Commit: not assigned; retain Planned/Review until implementation evidence exists.

## Variance And Residuals

- Remaining `tui_bridge.rs` responsibilities are outside this bounded slice and remain owned by R02
  for later non-overlapping seams or by their existing stories.

## Retrospective

- Pending execution.
