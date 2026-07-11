# Iteration I115: Runtime Event Semantic Convergence

> Document status: Complete (2026-07-11)
> Published plan date: 2026-07-11
> Planned objective: Restore semantic single-data-flow behavior across turn lifecycle, rendering,
> persistence, and all runtime surfaces after the ARCH-032 topology-only audit missed independent
> ordering domains.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: a runnable Talos turn containing thinking, tools, and final text renders and
> reloads without dropped or duplicated content through the canonical session protocol.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `ARCH-033` | none | Ready from user correction | ADR-005/006/034; ARCH-032 evidence | One ordered runtime flow with authoritative lifecycle and persistence |

### Scope

- Flatten live UI content onto one FIFO event queue.
- Make the session lifecycle authoritative and sequence-addressable.
- Make session persistence single-owner and replay-equivalent.
- Converge product/runtime surfaces on the session protocol.
- Add tool-loop, ordering, replay, and cross-surface regression coverage.

### Non-Goals

- Global pub/sub, provider feature expansion, permission/sandbox changes, storage-format defaults,
  release/tag/publish work, or unrelated visual redesign.

### Acceptance

- Given a provider turn containing reasoning, text, a tool call/result, and continuation text,
  when Talos renders and persists the turn, then all content remains ordered and present live and
  after resume.
- Given queued steering during a tool loop, when a provider response ends with `tool_use`, then the
  queued message remains queued until the whole session turn completes.
- Given any supported CLI/RPC surface, when a turn completes, then it observes the same canonical
  `SessionEvent` lifecycle instead of constructing a surface-specific completion model.

### Planned Validation

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- focused runtime ordering/replay tests
- actual `talos` binary mock/tool-loop transcript

### Documentation To Update

- `docs/reference/ARCHITECTURE.md`
- `docs/backlog/active/ARCH-033-runtime-event-semantic-convergence.md`
- `docs/BOARD.md`
- `docs/iterations/README.md`
- `EVOLUTION.md`

### Risks And Rollback

- Risk: public protocol migration can break embedders or leave one mode on legacy semantics.
- Rollback: retain deprecated compatibility variants during the ADR-039 migration window while
  migrating every in-tree producer and validating surface parity before removal.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-11 | Inventory | I085 remains Paused; I106-I109 remain Review; I018-I020, I028, and I086-I089 retain their recorded deferred/planned dispositions; I114 is Complete. User-directed P0 data-loss correction takes priority without rewriting those baselines. |
| 2026-07-11 | Activation | ARCH-033 activated from the semantic audit: channel topology is ADR-006 compliant, but nested streams, split lifecycle authority, and multiple persistence writers violate the stronger single-flow objective. |
| 2026-07-11 | Implementation | Added ordered `TurnEvent` envelopes, flattened canonical UI content, made session completion authoritative, moved successful turn-message persistence into `AppServerSession`, removed the TUI submission/persistence side queue, and migrated interactive/inline/print/embedded/RPC surfaces. |
| 2026-07-11 | Closeout | ARCH-033 acceptance complete; I115 closed after full workspace, binary E2E, clippy, formatting, and governance validation. |

## Verification Evidence

- `cargo fmt --all -- --check`: pass.
- `cargo check --workspace`: pass.
- `cargo clippy --workspace -- -D warnings`: pass.
- `cargo test --workspace`: pass, including dashboard loopback tests, binary E2E tests, examples,
  and doc tests.
- `canonical_turn_events_are_contiguous_and_actor_persistence_replays_messages`: contiguous
  sequence, durable `session_id` propagation, replay-equivalent messages, zero duplicate persisted
  AgentEvents.
- `conversation_loop_keeps_steering_queued_across_provider_tool_end`: provider `tool_use` boundary
  does not drain steering.
- `canonical_tool_loop_uses_one_fifo_content_protocol_without_legacy_streams`: thinking, pre-tool
  text, and post-tool text stay on one FIFO content protocol.
- Runtime evidence: `mcp_client_e2e_routes_tool_call_through_fixture_server` executes the actual
  `talos --print --mock` binary through an MCP tool loop and asserts final continuation text;
  `rpc_mode_agent_run_uses_session_runtime_and_returns_final_text` exercises actual RPC mode.
- `scripts/validate_project_governance.sh .`: pass, 0 warnings.

## Variance And Residuals

- `UiOutput::Stream` and legacy unwrapped `SessionEvent` variants remain public compatibility
  inputs under ADR-039; no in-tree canonical producer uses them. Removal waits for a semver-major
  release or explicit migration decision.

## Retrospective

- Outcome: met. The original dropped-content condition was removed rather than masked with another
  renderer special case.
- Documentation: ARCHITECTURE, ADR-039, ARCH-032 correction, backlog, board, iteration index,
  governance manifest, and EVOLUTION synchronized.
- Lessons: EVOLUTION #38 records that topology-only audits do not prove semantic single flow.
