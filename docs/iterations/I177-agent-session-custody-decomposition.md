# Iteration I177: Agent Session Custody Decomposition

> Document status: Complete
> Published plan date: 2026-08-07
> Planned objective: decompose private durable-custody and reconciliation responsibilities from `talos-agent/src/session.rs` without changing Session behavior.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: the existing `AppServerSession` remains the sole actor and state owner while private custody/reconciliation workflows have separate source ownership and all existing structured-submission, recovery, generation, pause/cancel, receipt, shutdown, and archive behavior remains unchanged.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 architecture session 2026-08-07 |
| Work Slice | Extract private durable custody/reconciliation, admission/rejection/receipt projection, pending-shutdown release, structured-turn finish, and pause/cancel helpers from `talos-agent/src/session.rs` while keeping `AppServerSession` as the sole actor and mutable state owner; preserve actor ordering, generation fences, receipts, recovery, pause/cancel, archive, diagnostics, event order, persistence protocol, and public API. |
| Claimed At | 2026-08-07 |
| Source Issue | None |
| Governance Claim PR | #161 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | No independent reviewer is currently available; exact-head CI, both governance validators, merge-time CAS, and no blocking review feedback are required. |
| Implementation PR | #162 |
| Last Updated | 2026-08-07 |
| Handoff / Release Condition | Closed after implementation PR #162 merged; any actor redesign, state ownership, persistence/event/diagnostic, public API, dependency, or behavior change requires a separate story and ADR where applicable. |

Before activation, follow `docs/sop/AGENT-COLLABORATION.md`. The claim is ineffective until the
finalized `Claimed` record is merged into `main`.

## Non-Terminal Inventory At Selection

| Item | State | Disposition |
|---|---|---|
| I159 | Blocked | Unchanged; TUI-037 disposition remains required. |
| I160 | Blocked | Unchanged; requires I159 closure. |
| I161 | Blocked | Unchanged; requires I160 and independent security review. |
| I162 | Blocked | Unchanged; requires I161 and publication authorization. |
| I164 | Paused | Historical superseded target; no activation. |
| Recovery PRs #120/#121 | Open archival evidence | Immutable; no implementation authority. |
| ARCH-034-R04 | Refinement | Native/panic/unsafe boundary remains excluded pending independent security review. |
| ARCH-034-R07 / I176 | Closed | Delivery evidence `1de3243d`; no overlap with agent Session custody ownership. |
| ARCH-034-R09..R11 | Ready / unclaimed | Retained for later independent claims; no overlap with this custody split. |

No Active, Review, or other Planned iteration overlaps this work. I171 architecture evidence and
the completed R01-R03 and R05-R07 seams are prerequisites/context only. Open recovery PRs #120/#121
remain immutable archival evidence and do not authorize implementation.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| ARCH-034-R08 | ARCH-034 | Ready | I171 architecture register, I169 durable-custody behavior, and existing agent Session tests | One behavior-preserving private custody/reconciliation source split behind `AppServerSession`. |

### Scope

- Keep `AppServerSession` as the sole actor, run-loop coordinator, and owner of mutable Session state.
- Move existing private custody reconciliation, admission/rejection/receipt projection, pending-shutdown release, structured-turn finish, and pause/cancel helpers into one private child module.
- Keep state access explicit through the current actor reference and helper parameters/results; the child module cannot own an independent actor, queue, store, or generation state machine.
- Add source-layout regression evidence that the custody module stays private and actor orchestration remains in `session.rs`.

### Non-Goals

- No run-loop, channel, queue, scheduler, retry, persistence protocol, generation-fence, receipt, pause/cancel, archive, event-ordering, diagnostic, public API, dependency, feature, or security-boundary change.
- No actor redesign and no changes to R04 or R09-R11.

### Acceptance

- Given existing callers, `AppServerSession` and every public Session API continue to compile unchanged.
- Given accepted, rejected, replayed, paused, cancelled, interrupted, completed, and shutdown submissions, durable-store transitions, generation checks, receipts, events, diagnostics, queue ordering, and archive decisions remain unchanged.
- Given the new source layout, custody/reconciliation helpers have private source ownership without independently owning or mutating actor state outside explicit parameters and the current `AppServerSession` receiver.
- Existing I169 durable-custody, crash-replay, generation, targeted-interrupt, stress, terminal-recovery, transcript-journal, and agent Session tests pass unchanged, with a focused source-layout regression added.

### Planned Validation

- `cargo test -p talos-agent --locked --no-fail-fast`
- `cargo fmt --all -- --check`
- `cargo check --workspace --locked`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo test --workspace --locked --no-fail-fast`
- `./scripts/release_preflight.sh`
- `scripts/validate_project_governance.sh .`
- `bash scripts/validate_collaboration_claims.sh .`
- `git diff --check`
- Mechanical source-body equivalence check against the effective claim merge, normalizing only module plumbing and required private visibility changes.
- Exact-head Unix/Windows CI and rebuilt CLI smoke.

### Documentation To Update

- Synchronize ARCH-034-R08, `docs/iterations/README.md`, `docs/backlog/PRODUCT-BACKLOG.md`, `docs/BOARD.md`, and the governance manifest.
- No user-facing behavior documentation change is expected because this infrastructure-only deliverable preserves behavior.

### Risks And Rollback

- Risk: moving stateful helpers changes borrow lifetimes, actor ordering, generation fences, durable custody, receipt/event projection, pause/cancel, shutdown release, or exact diagnostics despite compiling.
- Rollback: revert the private module move if mechanical equivalence and existing custody tests cannot be shown; behavior or actor-design changes require a separate story and, where applicable, an ADR.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-08-07 | Planning | I177 selected after inventorying blocked/paused work, confirming I176/R07 completion, and finding no overlapping effective claim or implementation PR. Governance-only claim PR #161 proposes ownership; the claim remains ineffective until its finalized head merges. |
| 2026-08-07 | Activation | Claim PR #161 exact-head CI `31163434854` passed; merge-time CAS confirmed finalized head `58876190abf9ed2f437090fec94464f009cf06e4`, no overlapping PR or blocking feedback, and the claim merged as `9bc6012cab231de877bc1a933d1575c841394aa8`. Implementation started from that effective claim. |
| 2026-08-07 | Review submission | Behavior-preserving custody helper decomposition was committed as `786aa571`; Draft implementation PR #162 was opened for exact-head CI and merge review. |
| 2026-08-07 | Completion | PR #162 squash-merged at `f505eea8` after exact-head CI `31166594367`; merge-time CAS, both governance validators, remote owner reconciliation, installer fixture, and whitespace checks passed. |

## Verification Evidence

- Claim exact-head CI `31163434854` passed Unix/Windows workspace, governance, remote owner reconciliation, installer fixture, and rebuilt CLI smoke checks.
- `cargo test -p talos-agent --locked --no-fail-fast`: passed 257 unit tests, all I169 custody/recovery/generation/interrupt/stress integration tests, the I177 source-layout regression, and 12 doctests.
- `cargo clippy -p talos-agent --all-targets --locked -- -D warnings`: passed.
- `./scripts/release_preflight.sh`: passed locked workspace format, check, Clippy, tests, doctests, governance, collaboration-claim, site, installer, and release gates.
- Re-extracting the custody helper token stream from `session/custody.rs` matched claim merge `9bc6012cab231de877bc1a933d1575c841394aa8` after normalizing only required `pub(super)` visibility, rustfmt whitespace, and syntax-neutral trailing commas.
- The first focused test attempt stopped before a code verdict because the filesystem had 344 MiB free; `cargo clean` removed only reproducible workspace build artifacts, restored about 25 GiB free, and the identical locked test command then passed.

## Completion Evidence

- Completion Commit: `f505eea8`

## Variance And Residuals

- R04 remains Refinement pending independent security review.
- R09-R11 remain separately owned and independently claimable after I177 closes.

## Retrospective

- Outcome: Complete; behavior-preserving private agent Session custody/reconciliation source decomposition delivered.
- Documentation: governance owners synchronized; no user-facing behavior documentation change was needed.
- Lessons: none recorded.
