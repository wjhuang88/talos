# Iteration I169: Transactional Batched Steering Turn

> Document status: Complete (2026-08-06)
> Published plan date: 2026-08-01
> Preactivation hardening date: 2026-08-02
> Activation date: 2026-08-02
> Completion date: 2026-08-06
> Activation baseline: `main@a9faf4a8b7db2b87eaf87a288338e36f5f2f7eae`
> Implementation branch: `feat/i169-tui-044-transactional-steering`
> Implementation PR: #131 — merged
> Completion Commit: `685d3b4f4088a172551f8c844a89f5dee9469430`
> Accepted exact Head: `90165cace4625c0f27616b3e1b9871bcb6a10186`
> Objective: implement Issue #119 / TUI-044 under ADR-056 with structured queue boundaries,
> transactional ownership transfer, durable pending custody, exact request planning and replay parity.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | GPT-5.6 Thinking / I169 implementation and acceptance sessions |
| Work Slice | Structured input identity, Engine escrow, durable Session pending journal and receipts, Actor arbitration, lifecycle correlation, exact Provider request planning, transcript/replay parity, Session fork/delete ownership, and acceptance evidence. |
| Claimed At | 2026-08-02 |
| Completed At | 2026-08-06 |
| Source Issue | #119 — completed |
| Governance Claim PR | #123 |
| Preactivation Architecture PR | #129 |
| Implementation PR | #131 — merged |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | The maintainer authorized formal activation, accepted exact-head automated and real-terminal evidence, classified #136 as a non-blocking independent residual, and explicitly authorized merge and closeout. |
| Last Updated | 2026-08-06 |
| Handoff / Release Condition | Satisfied. Completion evidence is recorded below; ADR-056 is Accepted and TUI-044 is Complete. |

## Selected Story And Decision

| Owner | Final State | Outcome |
|---|---|---|
| [TUI-044](../backlog/active/TUI-044-transactional-batched-steering-turn.md) | Complete | Compatible queued steering becomes one bounded later Turn while original user-item boundaries and FIFO order remain intact. |
| [ADR-056](../decisions/056-transactional-steering-submission-boundary.md) | Accepted | Durable receipt, unique execution authority, generation-safe lifecycle, Actor arbitration, transcript-before-journal finalization and exact request-plan boundaries are authoritative. |
| [Issue #119](https://github.com/wjhuang88/talos/issues/119) | Completed | Acceptance matrix satisfied and implementation merged. |
| [Issue #136](https://github.com/wjhuang88/talos/issues/136) | Open, non-blocking | Owned by TUI-047 for executable recovery-command wording on the direct `/delete` failure surface. |

## Delivered Scope

1. Structured Engine queue items retain stable item identity, Session/generation, source/kind, exact
   text, attachments, FIFO sequence and bounded metadata.
2. Matching authoritative Success takes one deterministic Engine-accepted cutoff snapshot; later
   input stays in a later batch.
3. Queue transfer follows prepare/reserve/send/durable-accept/reconcile/commit semantics.
4. Sent Engine escrow remains recoverable but cannot execute; journal-backed Actor acceptance is the
   only ownership acknowledgement.
5. The versioned Session-scoped pending journal supports idempotent acceptance, lost-Ack
   reconciliation, conflict rejection, bounded custody and restart recovery.
6. Session generation and exact batch/reservation/receipt/Turn identities flow through Actor,
   Scheduler, Bridge and canonical lifecycle events.
7. Actor arbitration enforces at most one active Turn, no ordinary Submit preemption and deterministic
   user/scheduler ordering.
8. Success advances eligible pending work; Cancel/Error pauses unstarted work without losing or
   reordering it.
9. A/B/C remain distinct ordered User or Multimodal messages in Actor input, Provider request,
   transcript and resumed history.
10. Transcript success is committed before pending-journal finalization; crash recovery finalizes by
    Turn identity without re-execution.
11. Every initial and continuation Provider call validates and sends the same sealed exact request
    plan.
12. Session restoration, fork isolation, deletion and retryable artifact ownership use one coherent
    transcript-last cleanup boundary.
13. Legacy public single-item drain and Session operations remain compatible through additive APIs.

## Validation And Acceptance

### Exact-head machine validation

Final CI run `31010166558` / CI #1233 executed against exact Head
`90165cace4625c0f27616b3e1b9871bcb6a10186`, attempt 1:

1. Format + Check + Clippy + Test — success.
2. Windows Rust workspace — success.
3. Windows installer fixture — success.
4. Remote Issue / Owner reconciliation — success.

No failed first attempt, rerun, cancellation or `action_required` gate remained.

### Rebuilt real-terminal validation

The maintainer built `cargo build --release --locked -p talos-cli` on macOS and recorded:

- Talos version `0.6.1`;
- binary SHA-256 `2fe9f07679bd3f513165e849c59335ef11f47662852283c8f22051e954b2683d`;
- source Session `9e937a59-a700-47eb-9a29-2affb800aa00`;
- fork child `32c30467-8d3a-493e-bfb3-60b5e773c2ca`;
- disposable recovery Session `a971f883-9749-445c-90c9-17fe23eb79a9`.

The walkthrough proved:

- clean branch, Head, main and merge-base binding;
- default new Session creation and exact `-c` / `--session` restoration;
- three separately submitted inputs reached `3 queued` during one active tool Turn;
- three distinct FIFO boundaries produced exactly one continuation Turn;
- restart preserved boundaries and exactly-once projection;
- no ghost, duplicate or parallel model execution;
- CLI fork created a distinct child with bidirectional source/child write isolation;
- `/delete <UUID>` removed transcript, SQLite, WAL, SHM, list and search visibility;
- injected cleanup failure emitted Error with no false success, preserved retryability and succeeded
  when the same delete command was retried after restoring the sidecar path;
- `talos storage maintenance --reconcile` completed with `failures 0` and `bounded=false`;
- all disposable child/recovery artifacts and index entries were removed.

## Actual Execution Timeline

| Date | Type | Record |
|---|---|---|
| 2026-08-01 | Recovery and ownership | Recovered Issue #119 and established TUI-044/I169 as the current owners; archival PR #120 remained immutable. |
| 2026-08-01 | Prerequisite | I170 completed through PR #126. |
| 2026-08-02 | Architecture hardening | PR #129 established ADR-056 preactivation contracts. |
| 2026-08-02 | Activation | Maintainer authorized implementation; a fresh branch and PR #131 were created from the recorded baseline. |
| 2026-08-02–05 | Implementation and remediation | Implemented durable custody, receipt reconciliation, generation-safe lifecycle, scheduler arbitration, exact request planning, crash recovery, Session mutation barriers and artifact ownership; independent review findings were corrected with focused and full regression evidence. |
| 2026-08-05 | Automated acceptance | Exact-head source and CI review returned `AUTOMATED / NON-MANUAL ACCEPTANCE PASSED`. |
| 2026-08-06 | Real-terminal acceptance | Maintainer completed the guided release-binary walkthrough; all I169 behavior gates passed. Missing direct-delete recovery-command wording was separated into non-blocking Issue #136. |
| 2026-08-06 | Merge | PR #131 merged exact Head `90165cace4625c0f27616b3e1b9871bcb6a10186` into main at `685d3b4f4088a172551f8c844a89f5dee9469430`. |
| 2026-08-06 | Closeout | TUI-044 and I169 moved to Complete, ADR-056 moved to Accepted, and Issue #119 was closed as completed. |

## Completion Evidence

- Completion Commit: `685d3b4f4088a172551f8c844a89f5dee9469430`.
- Implementation PR: #131, merged 2026-08-06.
- Exact implementation Head: `90165cace4625c0f27616b3e1b9871bcb6a10186`.
- Acceptance base / merge base: `a03e25436a25f84f117a90362686fc8205e52dde`.
- Final exact-head CI: `31010166558` / CI #1233.
- Real-terminal release binary digest:
  `2fe9f07679bd3f513165e849c59335ef11f47662852283c8f22051e954b2683d`.
- Accepted decision: ADR-056.
- Completed Story: TUI-044.
- Completed Issue: #119.
- Independent residual: #136 / TUI-047, Open and non-blocking.
- Recovery PR #120 and its branch remain archival and untouched.

## Variance And Residuals

- Issue #136 / TUI-047 owns the missing executable recovery-command text on direct `/delete`
  cleanup failure. Underlying cleanup retryability, transcript-last ownership, index consistency and
  no-false-success behavior are accepted and complete.
- Queue editing/reordering, persistent cross-Session steering, automatic retry of a started terminal
  Turn, broader graceful shutdown, general persistent tasks and multi-controller arbitration require
  separate owners.
- No release or REL-002 readiness claim is made by I169 completion.

## Retrospective

- A steering queue is an ownership-transfer protocol, not a string-joining convenience.
- Durable acknowledgement must distinguish custody from model-Turn start and support exact lost-Ack
  reconciliation.
- Generation changes require one durable fence and acknowledged retirement before publishing a new
  route.
- Transcript and pending work must remain separate durable concepts with transcript-first success
  finalization.
- Real-terminal acceptance exposed an actionability gap that automated semantics tests did not; the
  residual was separated without weakening accepted transactional behavior.
- Post-merge governance closeout is kept separate from implementation merge so lifecycle changes
  remain auditable.

## Historical Record

The complete published plan, review chronology and intermediate remediation evidence remain in git
history at `main@685d3b4f4088a172551f8c844a89f5dee9469430`, PR #131 and Issue #119. This final
iteration record intentionally summarizes the accepted boundary and evidence instead of retaining
obsolete handoff language as current state.
