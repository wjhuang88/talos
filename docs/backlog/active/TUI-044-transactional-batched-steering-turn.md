# TUI-044: Transactional Batched Steering Turn

| Field | Value |
|---|---|
| Story ID | TUI-044 |
| Type | TUI / Runtime State Story |
| Priority | P1 |
| Status | Ready — I170 prerequisite satisfied at `main@592254d73a98166df48da0139a02df67e9cd2cd6`; implementation not started |
| Source | [GitHub Issue #119](https://github.com/wjhuang88/talos/issues/119) |
| Selected Iteration | I169 |
| Depends On | TUI-026/I145; ADR-005; ADR-006; ADR-039; ADR-049; ADR-056; completed I170 Windows validation baseline |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed — eligible for explicit activation |
| Responsible Actor | @wjhuang88 |
| Executing Agent | GPT-5.6 Thinking / talos recovery session 2026-08-01 |
| Work Slice | TUI-044/I169 only: transactional structured steering queue transfer, lifecycle correlation, Actor arbitration, complete request budgets and durable replay parity after the merged I170 baseline. |
| Claimed At | 2026-08-01 |
| Source Issue | #119 |
| Governance Claim PR | #123 — merged |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | PR #123 established the owner chain. I170 then completed in PR #126 at `592254d73a98166df48da0139a02df67e9cd2cd6`, satisfying the published prerequisite. A separate explicit activation is still required before implementation begins. |
| Implementation PR | Not started |
| Last Updated | 2026-08-01 |
| Handoff / Release Condition | Release only by explicit maintainer handoff or after a separate fresh I169 implementation PR is merged and completion evidence is recorded. |

The claim is effective on `main`, but it is not implementation authorization by itself. Activation must create a fresh branch from the then-current `main`, re-read Issue #119 and current governance, and keep archival PR #120 untouched.

This Story replaces only the conflicting historical identifier. Current `TUI-041` remains owned by Issue #69 and must not be overwritten.

## Identity / Goal / Value

A Talos user who submits several steering, correction, attachment-bearing, or additional-context items while one model turn is active needs those compatible items to become one bounded follow-up turn without losing their individual identity, boundaries, ordering, persistence semantics, or retryability.

## Recovery Provenance

- Recovered Issue: #119, reconstructed from deleted Issue #50.
- Archival recovery PR: #120; never merge as-is.
- Historical exact head: `c984b71022a16169f26dec9f2e4a73b78a41a93d`.
- Historical branch: `recovery/pr-68-i169-20260731`; immutable and not a development branch.
- Original fresh-audit baseline: `main@c28fe6a6c70b0115e99372927a29ab4107b06b78`.
- I170 prerequisite completion baseline: `main@592254d73a98166df48da0139a02df67e9cd2cd6`.
- A future activation must refresh again from the actual current `main`; this closeout does not freeze an implementation Head.
- Historical `TUI-041` identifier is obsolete for this scope because current main assigns TUI-041 to Issue #69. TUI-044 is the current authoritative Story ID.

## Scope

- Preserve `ConversationEngine` ownership of structured queued steering items until actor acknowledgement.
- Preserve stable item ID, source/kind, exact text, multiline boundaries, attachments, FIFO sequence and item-bound limits.
- Classify ordinary user input, slash/local commands, preview requests, scheduler work and attachment input before queue admission.
- Use a deterministic bridge-accepted queue snapshot cutoff; never claim unobservable keypress/source-time ordering.
- Transfer one compatible bounded prefix through an equivalent `prepare -> reserve -> send -> acknowledge -> commit` protocol.
- Validate authoritative lifecycle identity using session, turn, sender/runtime generation and monotonic sequence.
- Reject stale, duplicate, wrong-session, wrong-turn, wrong-generation, regressive, gap and uncorrelated terminal events without queue mutation.
- Make the Session Actor the sole owner after acknowledgement and preserve one active model turn with no ordinary Submit preemption.
- Use source-aware user/scheduler arbitration; only Success auto-advances, while Cancelled/Error retain and pause work until explicit user resumption.
- Preserve live actor input, successful durable persistence and resumed history as the same ordered user-item boundaries.
- Budget complete initial and continuation Provider Requests, including prompts, memory, workspace context, history, tools, structured items, multimodal attachments, continuation overlay and output reserve.
- Preserve public `ConversationEngine::drain_steering_queue` single-item FIFO compatibility while adding non-breaking batch APIs.

## Non-Goals

- No Windows shell, path, fixture or process portability work owned by completed I170.
- No concurrent model turns, persistent cross-session steering queue, arbitrary queue editing/reordering UI, semantic rewriting, summarization, deduplication, global event bus, permission redesign, sandbox redesign, or unrelated provider protocol change.
- No delimiter-joined string as authoritative Actor or durable representation.
- No implicit queue movement across `/new`, `/resume`, `/fork`, model or provider changes.
- No modification, rebase, merge or rewrite of recovery PR #120 or its branch.

## Decision Links And Constraints

- ADR-039 remains the authoritative ordered session event boundary.
- ADR-049 remains authoritative for engine-owned queue projection before transfer acknowledgement.
- ADR-056 defines the proposed transactional transfer, actor arbitration, lifecycle, replay and budget boundary; it remains Proposed until reviewed on a fresh current-main implementation.
- Public API changes remain additive and migration-aware under `AGENTS.md`.
- Completed I170 remains a prerequisite baseline and must not be mixed back into I169 scope.

## Acceptance For Behavior / Technical Work

- A/B/C accepted before the completion cutoff start one follow-up turn and remain three independent FIFO user messages.
- Inputs accepted after the cutoff remain queued for a later batch.
- Empty and single-item queues preserve expected behavior.
- Provider tool-use terminal events do not drain the queue.
- Only matching authoritative Success prepares automatic advancement.
- Cancelled/Error retain all queued and actor-pending work, pause automatic advancement, and allow explicit user resumption without duplicates.
- Full/closed/timeout/replaced sender, switched session, shutdown and actor start rejection preserve all original items and the visible snapshot.
- Actor acknowledgement is required before the engine commits prefix removal.
- Slash/local commands and incompatible kinds never enter or silently combine with user-turn batches.
- Scheduler and user work preserve one active turn and deterministic source-aware ordering.
- Queue, item, batch, attachment, actor and complete request budgets reject or split only at item boundaries.
- Successful durable replay has no joined A/B/C string, ghost entry, duplicate item or missing boundary.
- Legacy single-item drain callers remain source-compatible.

## Minimum Validation

- focused Engine, lifecycle, transaction, structured-input, bound, persistence and fixed-seed stress tests from Issue #119
- `cargo fmt --all -- --check`
- `cargo check --workspace --locked`
- `cargo clippy --workspace --locked -- -D warnings`
- `cargo test --workspace --locked`
- `git diff --check`
- project-governance and collaboration-claim validators
- release preflight
- exact-head Windows and Unix/macOS CI
- rebuilt real TUI acceptance
- provider mock/request-preview evidence for initial and continuation budgets

## Activation Gate

TUI-044/I169 is now selectable, not active.

Before implementation:

1. re-read current `main`, open Issues, PRs, branches and owner docs;
2. confirm no overlapping active implementation or newer steering owner exists;
3. create a fresh I169 branch from the exact current `main`;
4. record activation evidence in I169/TUI-044/Board before code changes;
5. keep recovery PR #120 and `recovery/pr-68-i169-20260731` immutable.

## State / Status Owners

- Story scope and acceptance: this file.
- Execution and evidence: `docs/iterations/I169-batched-steering-turn.md`.
- Architecture decision: `docs/decisions/056-transactional-steering-submission-boundary.md`.
- Remote discussion and recovered requirements: Issue #119.
- Historical implementation evidence only: Draft PR #120.
- Current operating view: `docs/BOARD.md`.

## Residual Destination

Any queue editing UX, persistent cross-session queue, multi-controller arbitration, provider-specific same-role adaptation beyond safe compatibility, or process portability expansion requires a separate owner and decision.
