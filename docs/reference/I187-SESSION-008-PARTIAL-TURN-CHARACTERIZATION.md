# I187 SESSION-008 Partial-Turn Characterization

> Evidence date: 2026-08-11
> Source baseline: `main@5bb83f80b7dd7216ed83ee69fd4de0ef954c32f7`
> Scope: read-only characterization for ADR-058; no behavior claim

> Historical boundary: I193 / SESSION-008-B merged on 2026-08-14 as
> `1b5461cdcb03c7a896b814ccad2d93aa44010fc6`. This document remains the authoritative
> pre-I193 characterization referenced by SESSION-008-R1; it is not a statement of current
> post-merge behavior. ADR-058 and the I193 owner now define the implemented contract.

## Current Ownership Map

| Stage | Current owner | Current fact |
|---|---|---|
| Stable message construction | `Agent::run_inner` | Returns `persistence_projection(messages[persist_start..])` only when the agent future returns. Unfinished streamed assistant fragments are not pushed into the message vector. |
| Live progress and raw tool observation | `run_turn_with_forwarding` | A forwarding task emits ordered session progress and retains raw tool output only for legacy persistence metadata. |
| Cancellation | `run_turn_with_forwarding` | The cancellation branch aborts `agent_task`, waits for the forwarder, writes a terminal marker and returns `new_messages = []`. It cannot recover the agent future's partial vector. |
| Legacy partial persistence | `persist_turn_messages` | Provider Error appends projected partial messages one at a time with `bind_turn_id = false`, then appends the Error marker. |
| Durable Success | `DurableSession::commit_turn` | Under the write lock, appends filtered messages plus a Success marker through atomic replacement; same `turn_id` returns prior IDs. |
| Durable Error/Cancelled | `persist_terminal_outcome` | Appends only an Error/Cancelled marker through `DurableSession::session()`; no partial durable entries or `EntriesCommitted`. |
| Runtime restart | `RuntimeBuilder::build` | `DurableSession::read_messages()` supplies model history and filters hidden diagnostic/outcome markers. |
| Journal recovery | `PendingSubmissionStore::mark_committed` | Reads the latest explicit outcome marker and maps Success/Error/Cancelled to committed/terminal states; missing marker remains ambiguous. |

## Current Outcome Matrix

| Terminal path | Legacy visible messages | Durable visible messages | Marker | Atomic with messages | Turn-linked entries |
|---|---|---|---|---|---|
| Success | Complete turn | Complete turn | Success | Durable: yes; legacy: no | Durable messages: yes |
| Provider Error | Closed projected prefix when available | None | Error | No | Legacy partial messages: no |
| Cancelled | None from aborted agent task | None | Cancelled | No | Not applicable |
| Agent task panic | None | None | Error | No | Not applicable |
| Persistence failure | Best effort; error remains observable | No Success commit | Error best effort | No | Not guaranteed |

## Existing Executable Evidence

- `cargo test --locked -p talos-agent failed_continuation_preserves_completed_tool_prefix_without_trailing_fragment`
  demonstrates the legacy provider-error prefix behavior.
- `cargo test --locked -p talos-agent fixture_adr042_durable_failed_turn_aborts_with_real_durable`
  and `fixture_durable_transcript_empty_after_failed_turn` freeze the current ADR-042 gap.
- `cargo test --locked -p talos-session durable::tests::atomic_turn_is_idempotent_and_redacts_credentials`
  demonstrates Success atomicity, idempotency and redaction.
- `cargo test --locked -p talos-session --test i169_turn_outcome` demonstrates that markers stay
  hidden and only Success currently binds turn identity.
- `cargo test --locked -p talos-agent --test i169_terminal_outcome_recovery` demonstrates explicit
  Success/Error/Cancelled marker recovery rather than inference from ordinary transcript entries.

These fixtures prove current behavior only. The two durable failed-turn fixtures must be replaced or
reframed in SESSION-008-B after ADR-058 is Accepted; they are not acceptance evidence for the
proposed behavior.

## Gaps ADR-058 Must Close

1. Cancellation has no session-owned stable-prefix snapshot before the agent task is aborted.
2. Partial messages and the Error/Cancelled marker are not one atomic operation.
3. Legacy partial entries are not bound to `turn_id`.
4. Durable transcript projection exposes entries but not a normalized turn-outcome view for hosts.
5. Retry logic treats any prior Success evidence as idempotent but has no same-payload/conflicting-
   outcome contract for Error or Cancelled.
6. The current README truthfully documents success-only durable persistence and must not change
   until SESSION-008-B implements and verifies the new contract.

## Dependency Effect

- SESSION-008-B remains blocked until ADR-058 is Accepted.
- RUNTIME-005-A may consume the accepted policy vocabulary, but RUNTIME-005-B remains blocked until
  SESSION-008-B completes.
- TOOL-024 remains a later consumer of RUNTIME-005 and cannot be used to define this persistence
  contract.
