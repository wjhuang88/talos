# Iteration I172: CLI/TUI Bridge Legacy Projection Decomposition

> Document status: Complete
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
| Implementation PR | #144 (merged at `c1dc67ae`) |
| Last Updated | 2026-08-07 |
| Handoff / Release Condition | Complete after implementation commit `4084138d` and exact-head CI run `31137882248`. |

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
| 2026-08-07 | Completion | Implementation commit `4084138dc0652d3200045847d42518d9ecb66231` merged through PR #144 at `c1dc67ae8e3a117dd39ede91143c5f6bcd2d17c4`; exact-head CI `31137882248` passed all four checks. |

## Verification Evidence

- `cargo check -p talos-cli --locked`, focused CLI tests, all-target Clippy, governance validators,
  and `./scripts/release_preflight.sh` passed before PR creation.
- Exact-head CI `31137882248` passed Unix and Windows workspace checks, installer fixture, and
  remote issue/owner reconciliation.

## Completion Evidence

- Completion Commit: `4084138dc0652d3200045847d42518d9ecb66231`.
- Merge: PR #144 at `c1dc67ae8e3a117dd39ede91143c5f6bcd2d17c4`.
- Behavior-preservation boundary and residual responsibilities remain explicit in the R02 owner.

## Variance And Residuals

- Remaining `tui_bridge.rs` responsibilities are outside this bounded slice and remain owned by R02
  for later non-overlapping seams or by their existing stories.

## Retrospective

- A private module seam reduced facade size while retaining the existing public/runtime entry
  points. Source-layout regression coverage makes the ownership boundary reviewable.
- Remote governance drift from newly opened Issues #141–#146 was repaired in independent intake
  owner records before merge; no unrelated implementation behavior was mixed into R02.
