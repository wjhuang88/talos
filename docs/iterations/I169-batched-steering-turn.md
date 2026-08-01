# Iteration I169: Transactional Batched Steering Turn

> Document status: Planned
> Published plan date: 2026-08-01
> Planned objective: implement Issue #119 from current main using TUI-044 and ADR-056 while preserving structured item boundaries, transactional ownership and durable replay.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: a rebuilt Talos TUI proves A/B/C accepted during one active turn start one later model turn as three ordered user items, with rollback-safe ownership transfer and replay parity.

The effective Collaboration Claim is owned by `docs/backlog/active/TUI-044-transactional-batched-steering-turn.md`. I169 implementation may not start until that claim is on `main` and the required I170 baseline has merged.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `TUI-044` | None | Ready | TUI-026/I145; ADR-005/006/039/049/056; I170 | One transactional, bounded, structured follow-up turn after matching authoritative Success. |

### Recovery Provenance

- Recovered Issue: #119.
- Archival implementation PR: #120.
- Historical exact head: `c984b71022a16169f26dec9f2e4a73b78a41a93d`.
- Historical branch: `recovery/pr-68-i169-20260731`; immutable and never used for continued development.
- Recovery implementation is design/test evidence only. It is not current-main CI evidence and is not assumed compatible with the current tool contribution, registry, permission or session architecture.
- Current audit baseline: `main@c28fe6a6c70b0115e99372927a29ab4107b06b78`.

### Scope

- Structured Engine queue with stable item identity, kind/source, exact text, attachments and FIFO sequence.
- Deterministic queue-snapshot cutoff when matching authoritative completion is processed.
- Transactional `prepare/reserve/send/acknowledge/commit` ownership transfer and rollback.
- Session/turn/generation/sequence lifecycle validation with bounded content-free rejection diagnostics.
- Actor-owned pending arbitration, one active turn, no normal Submit preemption and source-aware scheduler ordering.
- Success auto-advance; Cancel/Error retain and pause; explicit user resume.
- Complete initial and continuation provider-request budgeting with output reserve.
- Live/persisted/resumed user-item boundary equivalence and no ghost/duplicate entries.
- Additive public protocol/API changes and preserved `ConversationEngine::drain_steering_queue` compatibility.
- Current-main governance, collaboration claim, release preflight, CI and real-TUI evidence.

### Non-Goals

- No I170 Windows shell/process/path/fixture implementation.
- No delimiter-only authoritative batch, concurrent model turns, persistent cross-session steering queue, arbitrary queue editing, semantic rewriting/deduplication, global bus, permission/sandbox redesign, or unrelated provider changes.
- No historical governance overwrite and no direct merge/rebase/modification of PR #120 or its recovery branch.

### Acceptance

- A/B/C remain three FIFO user items in Actor input, provider request adaptation, persistence and resume history while producing only one follow-up model turn.
- Cutoff-after items form the next batch.
- Full/closed/timeout/replaced sender, session replacement, lost Ack, start rejection and shutdown preserve all items for retry without duplicate execution.
- Stale/duplicate/wrong-session/wrong-turn/wrong-generation/regressive/gap/uncorrelated events do not mutate queue or turn state.
- Provider tool-use end is non-terminal for queue advancement.
- Cancel/Error pause and retain; explicit user Submit resumes deterministically.
- Scheduler cannot bypass Actor arbitration or preempt user work.
- Multiline, attachment, preview, slash and mixed-kind classification preserve boundaries and reject incompatible batching.
- Item, queue, batch, attachment, Actor pending, initial request, continuation request and output-reserve limits are visible and item-boundary safe.
- Resume contains no delimiter-only replay, ghost entry, duplicate item or unstarted persisted batch.
- Existing public single-item drain API remains available with FIFO behavior.

### Planned Validation

#### Engine

- single FIFO drain, multi-item batch, empty queue, cutoff-after input, Ack-only clear, rollback and duplicate submission protection

#### Lifecycle

- matching Success, Cancelled, Error, provider tool-use non-terminal, stale, duplicate, wrong session/turn/generation, regression, gap and shutdown

#### Channel / Transaction

- full, closed, timeout, sender/session replacement, Ack loss, actor start rejection and retry without duplication

#### Structured Input

- A/B/C distinct, multiline, attachments, slash, preview, scheduler, mixed incompatible kinds and item identity

#### Bounds / Persistence / Stress

- every Issue #119 bound; initial and continuation budgets; success persistence; Cancel/Error retention; resume parity; no ghost/duplicate/delimiter-only replay; fixed-seed enqueue/completion/cancel/retry/sender/scheduler/boundary stress

#### Full Gates

- `cargo fmt --all -- --check`
- `cargo check --workspace --locked`
- `cargo clippy --workspace --locked -- -D warnings`
- `cargo test --workspace --locked`
- `git diff --check`
- governance and collaboration validators
- release preflight
- Windows and Unix/macOS CI
- rebuilt real TUI walkthrough
- `--mock` or equivalent no-provider smoke
- Provider Request Preview / Mock Request checks

### Documentation To Update

- TUI-044 owner
- ADR-056
- TUI-026/ADR-049 only where the accepted boundary is extended
- iteration index, Product Backlog, Board and governance manifest
- README/user documentation after behavior exists
- Issue #119 synchronization comments and implementation PR links

### Risks And Rollback

- Risk: two owners believe they have submitted the same item. Require explicit Ack-correlated commit and tests.
- Risk: stale lifecycle events advance a new session. Bind every accepted event to session/turn/generation/sequence.
- Risk: context accounting omits tools, overlays or attachments. Estimate the complete request on initial and continuation paths.
- Risk: public protocol additions break exhaustive downstream matches. Keep legacy variants and document the pre-1.0 additive migration.
- Rollback: revert the current-main structured implementation while preserving new variants for durable compatibility if already released; never rewrite historical recovery objects.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-08-01 | Recovery audit | Current main retains only single-item `Vec<String>` steering drain. Historical structured implementation remains missing and must be redesigned on current Session and composition boundaries. |
| 2026-08-01 | Governance correction | Historical TUI-041 conflicts with current Issue #69 ownership. TUI-044 is established as the recovered Issue #119 Story. |

## Verification Evidence

- Pending claim merge, I170 baseline and implementation.

## Completion Evidence

- Completion Commit: pending.

## Variance And Residuals

- The historical baseline's delimiter-only MVP is explicitly obsolete; TUI-044/ADR-056 structured invariants are authoritative for this recovery.
- I169 remains independent from I170 and may rebase only onto a merged current-main I170 baseline, never onto the recovery branch.

## Retrospective

- Pending.
