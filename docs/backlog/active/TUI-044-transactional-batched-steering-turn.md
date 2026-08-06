# TUI-044: Transactional Batched Steering Turn

| Field | Value |
|---|---|
| Story ID | TUI-044 |
| Type | TUI / Runtime State Story |
| Priority | P1 |
| Status | **Complete (2026-08-06)** |
| Source | [GitHub Issue #119](https://github.com/wjhuang88/talos/issues/119) |
| Selected Iteration | [I169](../../iterations/I169-batched-steering-turn.md) |
| Decision | [ADR-056](../../decisions/056-transactional-steering-submission-boundary.md) — Accepted |
| Implementation PR | [#131](https://github.com/wjhuang88/talos/pull/131) — merged |
| Completion Commit | `685d3b4f4088a172551f8c844a89f5dee9469430` |
| Exact Accepted Head | `90165cace4625c0f27616b3e1b9871bcb6a10186` |
| Independent Follow-up | [#136](https://github.com/wjhuang88/talos/issues/136) — Open, non-blocking |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | GPT-5.6 Thinking / I169 implementation and acceptance sessions |
| Claimed At | 2026-08-01 |
| Activated At | 2026-08-02 02:32 +08:00 |
| Completed At | 2026-08-06 |
| Governance Claim PR | #123 |
| Preactivation Architecture PR | #129 |
| Implementation PR | #131 — merged |
| Authorization Mode | Single-maintainer merge |
| Handoff / Release Condition | Satisfied by exact-head automated acceptance, rebuilt real-terminal acceptance, and merge commit `685d3b4f4088a172551f8c844a89f5dee9469430`. |

## Outcome

Talos now treats compatible steering input accepted while one model Turn is active as one bounded,
transactional follow-up Turn while retaining each original item as a distinct ordered user message.
The implementation preserves Session and generation identity, item boundaries, attachments, FIFO
order, durable custody, retryability, replay equivalence and one-Actor execution authority.

For queued inputs A, B and C, the accepted behavior is:

```text
active Turn
  -> A, B, C accepted into the authoritative Engine queue
  -> matching authoritative Success
  -> prepare / reserve / send / durable receipt / Engine commit
  -> one later Actor-owned Turn
  -> Provider and transcript retain A, B, C as three ordered messages
```

## Accepted Contract

- `ConversationEngine` owns the visible structured queue before durable Actor acknowledgement.
- The cutoff is the Engine-accepted enqueue-sequence snapshot processed with matching authoritative
  completion; input not yet accepted belongs to a later batch.
- Transfer follows prepare/reserve/send/durable-accept/reconcile/commit semantics.
- After send, Engine escrow is recoverable but non-executable; only the addressed Session Actor may
  execute after journal-backed receipt.
- Structured operations, receipts and Turn events carry exact Session, generation, batch,
  reservation, receipt and Turn identities with monotonic sequence checks.
- Ordinary Submit never implicitly cancels or preempts an active Turn.
- Scheduler and user work share one Actor arbitration boundary with at most one active Turn.
- Success may advance pending work; Cancel/Error pauses unstarted work without deleting it.
- Successful transcript commit precedes pending-journal finalization; restart reconciliation does
  not re-execute a transcript-backed completed Turn.
- Initial and continuation Provider calls validate and send the same sealed exact request plan.
- Public single-item drain and legacy Session operations remain compatible through additive APIs.

## Acceptance Result

All Issue #119 acceptance criteria were satisfied. Exact-head machine validation and maintainer
real-terminal validation both passed.

The real-terminal walkthrough verified:

- new Session creation and exact `-c` / `--session <UUID>` restoration;
- three separately submitted messages reaching `3 queued` during one active tool Turn;
- three distinct FIFO message boundaries and exactly one continuation Turn;
- restart persistence with no ghost, duplicate or parallel model execution;
- CLI fork with distinct child UUID and bidirectional source/child write isolation;
- `/delete <UUID>` removal of transcript, SQLite, WAL, SHM, Session-list visibility and search
  visibility;
- retryable cleanup failure with no false-success output and successful retry after recovery;
- executable `talos storage maintenance --reconcile` with `failures 0` and `bounded=false`;
- cleanup of all disposable walkthrough artifacts and index entries.

## Completion Evidence

- Implementation PR: #131.
- Exact accepted implementation Head: `90165cace4625c0f27616b3e1b9871bcb6a10186`.
- Exact base and merge base during acceptance: `a03e25436a25f84f117a90362686fc8205e52dde`.
- Final standard CI: run `31010166558` / CI #1233, attempt 1; all four jobs successful.
- Release binary: Talos `0.6.1`.
- Release binary SHA-256: `2fe9f07679bd3f513165e849c59335ef11f47662852283c8f22051e954b2683d`.
- Source Session: `9e937a59-a700-47eb-9a29-2affb800aa00`.
- Fork child Session: `32c30467-8d3a-493e-bfb3-60b5e773c2ca`.
- Recovery Session: `a971f883-9749-445c-90c9-17fe23eb79a9`.
- Completion / merge commit: `685d3b4f4088a172551f8c844a89f5dee9469430`.
- Accepted decision: ADR-056.
- Source Issue: #119, completed and closed by governance closeout.

## Variance And Residuals

- Direct `/delete` cleanup-failure diagnostics do not yet print both executable recovery commands.
  The recovery behavior itself was proven correct; wording/actionability is independently owned by
  open Issue #136 and does not reopen this Story.
- Queue editing/reordering, cross-Session persistent steering, retry of an already-started terminal
  Turn, broader shutdown, general persistent tasks and multi-controller arbitration remain outside
  TUI-044.
- Recovery PR #120 and branch `recovery/pr-68-i169-20260731` remain immutable archival evidence.

## Historical Record

The complete activation plan, review remediation chronology and pre-merge evidence remain available
in repository history at merge commit `685d3b4f4088a172551f8c844a89f5dee9469430`, PR #131 and Issue
#119. This completed owner document records the final contract and disposition rather than repeating
every intermediate review checkpoint.
