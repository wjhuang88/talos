# Iteration I176: CLI Session Handler Decomposition

> Document status: Planned
> Published plan date: 2026-08-07
> Planned objective: decompose private provider/model and session-lifecycle responsibilities from `talos-cli/src/session_handlers.rs` without changing CLI or Session behavior.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: the existing `session_handlers` path keeps the same handler signatures, transition/UI channel ownership, persistence and model identity behavior, commit/rollback/publication ordering, diagnostics, and cleanup recovery while private provider/model and session-lifecycle workflows have separate source ownership.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 architecture session 2026-08-07 |
| Work Slice | Move existing private provider/connect/model workflows and session delete/new/resume/fork workflows into private modules behind the current `session_handlers` facade; preserve handler paths/signatures, transition and UI channel ownership, CLI syntax, persistence, model identity, commit/rollback/publication ordering, exact diagnostics, and cleanup recovery behavior. |
| Claimed At | 2026-08-07 |
| Source Issue | None |
| Governance Claim PR | Pending |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | No independent reviewer is currently available; exact-head CI, both governance validators, merge-time CAS, and no blocking review feedback are required. |
| Implementation PR | Not started |
| Last Updated | 2026-08-07 |
| Handoff / Release Condition | Release if the split requires any handler path/signature, transition/UI ownership, CLI syntax, persistence, model identity, ordering, diagnostic, cleanup recovery, dependency, or behavior change. |

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
| ARCH-034-R06 / I175 | Closed | Delivery evidence `5c45322245788e12316dffbe1f9cfacef390eff8`; no overlap with CLI session-handler ownership. |
| ARCH-034-R08..R11 | Ready / unclaimed | Retained for later independent claims; no overlap with this session-handler source split. |

No Active, Review, or other Planned iteration overlaps this work. I171 architecture evidence and
the completed R01-R03, R05, and R06 seams are prerequisites/context only. Open recovery PRs
#120/#121 remain immutable archival evidence and do not authorize implementation.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| ARCH-034-R07 | ARCH-034 | Ready | I171 architecture register and existing CLI Session/model lifecycle tests | One behavior-preserving private source split behind the current `session_handlers` facade. |

### Scope

- Keep the current `session_handlers` module path and all handler signatures.
- Keep `SessionTransition` locking, preparation, commit/rollback, publication, watch channels, and UI output ordering in the same workflows.
- Move provider setup/connect/registration and model activation workflows into a private provider/model module.
- Move session delete/new/resume/fork and owned-session rollback helpers into a private lifecycle module.
- Add source-layout regression evidence that both modules stay private and the facade remains the compatibility boundary.

### Non-Goals

- No CLI syntax, provider discovery, credential storage, config persistence, model resolution, Session storage, transaction ordering, UI output, diagnostic wording, cleanup recovery, public API, dependency, feature, or security-boundary change.
- No changes to R04 or R08-R11.

### Acceptance

- Given existing callers, every `session_handlers` handler path and signature continues to compile unchanged.
- Given provider connection and model activation flows, config writes, credential prompts, model identity resolution, runtime rebuild behavior, and exact diagnostics remain unchanged.
- Given delete/new/resume/fork flows, transition locking, runtime preparation, commit/rollback/publication order, watch-channel updates, UI output order, and recovery diagnostics remain unchanged.
- Given the new source layout, provider/model and session-lifecycle workflows have separate private source ownership without moving transition or UI channel authority into a new shared abstraction.
- Existing model activation, Session lifecycle, parser, and cleanup recovery tests pass unchanged, with a focused source-layout regression added.

### Planned Validation

- `cargo test -p talos-cli --locked --no-fail-fast`
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

- Synchronize ARCH-034-R07, `docs/iterations/README.md`, `docs/backlog/PRODUCT-BACKLOG.md`, `docs/BOARD.md`, and the governance manifest.
- No user-facing behavior documentation change is expected because the deliverable preserves behavior.

### Risks And Rollback

- Risk: moving async workflows changes visibility, borrow lifetimes, transition ordering, UI publication order, exact diagnostics, or test ownership despite compiling.
- Rollback: revert the private module move if mechanical equivalence and existing CLI/Session tests cannot be shown; behavior corrections require a separate story.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-08-07 | Planning | I176 selected after inventorying blocked/paused work, confirming I175/R06 completion, and finding no overlapping effective claim or implementation PR. A governance-only draft claim is being prepared; the claim remains ineffective until finalized and merged. |

## Verification Evidence

- Claim validation and implementation evidence will be appended after the claim and implementation phases.

## Completion Evidence

- Completion Commit: not assigned; retain Planned until the claim and implementation evidence exist.

## Variance And Residuals

- R04 remains Refinement pending independent security review.
- R08-R11 remain separately owned and independently claimable after I176 closes.

## Retrospective

- Outcome: pending.
- Documentation: pending implementation result; no user-facing behavior documentation change is planned.
- Lessons: none recorded.
