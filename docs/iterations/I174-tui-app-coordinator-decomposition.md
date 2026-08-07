# Iteration I174: TUI App Coordinator Decomposition

> Document status: Review
> Published plan date: 2026-08-07
> Planned objective: decompose `talos-tui/src/app.rs` into private input, stream/output, and frame-coordination modules without changing TUI behavior or public paths.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: the existing `Tui` coordinator runs through the same public API and event loop while input dispatch, stream/output handling, and frame construction have separate private source ownership.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 architecture session 2026-08-07 |
| Work Slice | Move existing TUI input dispatch, stream/UI-output handling, and frame construction helpers into private modules behind the current `app` facade; preserve `Tui` fields and public paths, `Tui::run` lifecycle and `tokio::select!` priority, render order, key/mouse/approval behavior, scrollback anchoring, cursor placement, output strings, and terminal restoration. |
| Claimed At | 2026-08-07 |
| Source Issue | None |
| Governance Claim PR | #151 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | No independent reviewer is currently available; exact-head CI, both governance validators, merge-time CAS, and no blocking review feedback are required. |
| Implementation PR | #152 |
| Last Updated | 2026-08-07 |
| Handoff / Release Condition | Claim merge precedes implementation branch; release if the split requires any public API, lifecycle, select-order, rendering, input, output, or terminal behavior change. |

## Non-Terminal Inventory At Selection

| Item | State | Disposition |
|---|---|---|
| I159 | Blocked | Unchanged; TUI-037 disposition remains required. |
| I160 | Blocked | Unchanged; requires I159 Complete. |
| I161 | Blocked | Unchanged; requires I160 and independent security review. |
| I162 | Blocked | Unchanged; requires I161 and publication authorization. |
| I164 | Paused | Historical superseded target; no activation. |
| Recovery PRs #120/#121 | Open archival evidence | Immutable; no implementation authority. |
| ARCH-034-R04 | Refinement | Native/panic/unsafe boundary remains excluded pending independent security review. |
| ARCH-034-R06..R11 | Ready / unclaimed | Retained for later independent claims; no overlap with TUI coordinator source ownership. |

No Active, Review, or other Planned iteration overlaps this work. I173/R03 is Complete with
Completion Commit `e4818e34c1e047c41d41abc1f7859c7984008e83`.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| ARCH-034-R05 | ARCH-034 | Ready | I171 architecture register and existing TUI behavior tests | One behavior-preserving private source split behind the current `Tui` facade. |

### Scope

- Keep `Tui`, its fields, public methods, and `Tui::run` in the existing public `app` facade.
- Move private key, mouse, approval, panel-action, and scroll-input dispatch helpers into one input-owned module.
- Move private stream polling/finalization and `UiOutput` projection helpers into one output-owned module.
- Move private frame construction, history anchoring, and frame-scroll helpers into one frame-owned module.
- Add source-layout regression evidence that the modules stay private and `Tui::run` remains the lifecycle coordinator.

### Non-Goals

- No visual redesign, layout or render-order change, key binding, panel behavior, event protocol,
  state-field, terminal lifecycle, public API, dependency, feature, or output wording change.
- No changes to TUI-041/TUI-042/TUI-043, R04, or R06–R11.

### Acceptance

- Given the existing `talos_tui::Tui` API, when downstream code builds after decomposition, then all current public paths and method signatures remain available.
- Given the current event loop, when input, UI output, stream chunks, and ticks compete, then `Tui::run` retains the exact lifecycle ownership and `tokio::select!` branch order.
- Given existing TUI scenarios, when snapshots, cursor, approval, stream, scrollback, exit-summary, and terminal tests run, then their behavior and expected output remain unchanged.
- Given the new source layout, when the regression guard inspects it, then input, output, and frame helpers remain private behind `app.rs` and no module owns the main run loop.

### Planned Validation

- `cargo test -p talos-tui --locked --no-fail-fast`
- `cargo fmt --all -- --check`
- `cargo check --workspace --locked`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo test --workspace --locked --no-fail-fast`
- `./scripts/release_preflight.sh`
- `scripts/validate_project_governance.sh .`
- `bash scripts/validate_collaboration_claims.sh .`
- `git diff --check`
- Exact-head Unix/Windows CI and rebuilt CLI smoke.

### Documentation To Update

- Update `docs/reference/ARCHITECTURE.md` only if it currently describes the affected source ownership; preserve all user-facing TUI behavior documentation because the deliverable changes no behavior.
- Synchronize ARCH-034-R05, `docs/iterations/README.md`, `docs/backlog/PRODUCT-BACKLOG.md`, `docs/BOARD.md`, and the governance manifest.

### Risks And Rollback

- Risk: moving interdependent methods changes visibility, select ordering, rendering order, or cursor/scroll state despite compiling.
- Rollback: revert the private module move if exact mechanical equivalence and current TUI tests cannot be shown; behavior corrections require a separate TUI story.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-08-07 | Planning | Governance-only claim PR #151 prepared after confirming no Active, Review, or Planned iteration and no overlapping claim or implementation PR; the claim is ineffective until merge. |
| 2026-08-07 | Activation | Claim PR #151 exact-head CI `31147027316` passed; merge-time CAS confirmed head `0ffb5400` against base `58fa3683`, and the claim merged as `49ff4e24`. Implementation starts from that effective claim. |
| 2026-08-07 | Implementation | Moved existing private input, stream/UI-output, and frame-coordination methods into three private `app` submodules. `Tui`, every public method, `Tui::run`, terminal lifecycle, and all method bodies remain mechanically equivalent apart from required `pub(super)` visibility. Added source-layout and public-root-path regression coverage. |
| 2026-08-07 | Review | Implementation PR #152 opened from implementation commit `e4248bfe`; exact-head CI and merge-time CAS remain required. |

## Verification Evidence

- Claim exact-head CI `31147027316` passed Unix/Windows workspace, governance, and rebuilt CLI smoke checks.
- `cargo test -p talos-tui --locked --no-fail-fast`: passed (487 unit tests, 2 I174 integration tests, and 2 doc tests).
- `cargo fmt --all -- --check`, `cargo check --workspace --locked`, and `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.
- `cargo test --workspace --locked --no-fail-fast`: passed, including all integration and doc tests.
- `./scripts/release_preflight.sh`: passed.
- `scripts/validate_project_governance.sh .`, `bash scripts/validate_collaboration_claims.sh .`, and `git diff --check`: passed.
- Mechanical `diff -uBw` comparison of all moved input, output, and frame method ranges against the claim baseline: no differences after removing required `pub(super)` visibility.
- Exact-head implementation CI and rebuilt CLI smoke remain pending on PR #152.

## Completion Evidence

- Completion Commit: not assigned; retain Planned/Review until implementation evidence exists.

## Variance And Residuals

- R04 remains Refinement pending independent security review.
- R06–R11 remain separately owned and independently claimable after I174 closes.

## Retrospective

- Outcome: pending.
- Documentation: pending the implementation result; no user-facing behavior documentation change is planned.
- Lessons: none recorded.
