# Iteration I175: Conversation Engine Decomposition

> Document status: Active
> Published plan date: 2026-08-07
> Planned objective: decompose private command and projection responsibilities from `talos-conversation/src/engine.rs` without changing conversation behavior or public paths.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: the existing `ConversationEngine` keeps the same public API, turn/steering state ownership, command behavior, output ordering, transcript formats, and extension snapshots while private command and projection helpers have separate source ownership.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 architecture session 2026-08-07 |
| Work Slice | Move existing private slash-command dispatch and transcript/extension projection helpers into private modules behind the current `ConversationEngine` facade; preserve public paths, turn/steering state ownership, command semantics, exact output text/order, transcript formats, plugin/skill behavior, and extension snapshot contents. |
| Claimed At | 2026-08-07 |
| Source Issue | None |
| Governance Claim PR | #154 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | No independent reviewer is currently available; exact-head CI, both governance validators, merge-time CAS, and no blocking review feedback are required. |
| Implementation PR | Not started |
| Last Updated | 2026-08-07 |
| Handoff / Release Condition | Release if the split requires any public API, state-ownership, command/output, transcript, extension snapshot, plugin/skill, dependency, or behavior change. |

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
| ARCH-034-R05 / I174 | Closed | Delivery evidence `e4248bfedd17c91aebb24c80c60580fcbcebec62`; no overlap with conversation engine ownership. |
| ARCH-034-R07..R11 | Ready / unclaimed | Retained for later independent claims; no overlap with conversation command/projection ownership. |

No Active, Review, or other Planned iteration overlaps this work. I171 architecture evidence and
the completed R02, R03, and R05 seams are prerequisites/context only.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| ARCH-034-R06 | ARCH-034 | Ready | I171 architecture register and existing conversation behavior tests | One behavior-preserving private source split behind the current `ConversationEngine` facade. |

### Scope

- Keep `ConversationEngine` state ownership, all public methods, and current public module paths.
- Move private slash-command parsing/dispatch helpers into a private command-owned module.
- Move private transcript formatting and extension snapshot projection helpers into a private projection-owned module.
- Add source-layout regression evidence that the modules stay private and the public engine facade remains the compatibility boundary.

### Non-Goals

- No command semantics, output wording/order, steering queue semantics, plugin/skill behavior, transcript format, extension snapshot schema, public API, dependency, feature, or storage change.
- No changes to R04 or R07-R11.

### Acceptance

- Given the existing `ConversationEngine` API, downstream code continues to build with all current public paths and signatures.
- Given slash commands and agent events, command handling preserves current state transitions, output text, and output ordering.
- Given transcript export and extension diagnostics, plain-text/Markdown transcript formats and extension snapshot contents remain unchanged.
- Given the new source layout, command and projection helpers remain private behind the engine facade and no helper module owns turn/steering state transitions.

### Planned Validation

- `cargo test -p talos-conversation --locked --no-fail-fast`
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

- Synchronize ARCH-034-R06, `docs/iterations/README.md`, `docs/backlog/PRODUCT-BACKLOG.md`, `docs/BOARD.md`, and the governance manifest.
- No user-facing behavior documentation change is expected because the deliverable preserves behavior.

### Risks And Rollback

- Risk: moving command/projection helpers changes borrow boundaries, output order, transcript formatting, or extension diagnostics despite compiling.
- Rollback: revert the private module move if exact mechanical equivalence and existing conversation tests cannot be shown; behavior corrections require a separate story.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-08-07 | Planning | I175 selected after inventorying blocked/paused work, confirming I174/R05 completion, and finding no overlapping claim or implementation PR. Governance-only claim PR #154 proposes ownership; the claim remains ineffective until merge. |
| 2026-08-07 | Activation | Claim PR #154 exact-head CI `31150975042` passed; merge-time CAS confirmed head `6d07a9b6` against base `ed1ea8e7`, no overlap or blocking review, and the claim merged as `69c345f0`. Implementation starts from that effective claim. |

## Verification Evidence

- Claim exact-head CI `31150975042` passed Unix/Windows workspace, governance, and rebuilt CLI smoke checks.
- Merge-time CAS confirmed finalized claim head `6d07a9b6ecee08f0d94e2150168096ddaba81428`, base `ed1ea8e71fc741d3f7780ddd676ba0721a8407e5`, no overlapping claim/implementation PR, and no blocking reviews/comments.

## Completion Evidence

- Completion Commit: not assigned; retain Planned until the claim and implementation evidence exist.

## Variance And Residuals

- R04 remains Refinement pending independent security review.
- R07-R11 remain separately owned and independently claimable after I175 closes.

## Retrospective

- Outcome: pending.
- Documentation: pending implementation result; no user-facing behavior documentation change is planned.
- Lessons: none recorded.
