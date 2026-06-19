# SESSION-001: Interactive Session Lifecycle

| Field | Value |
|---|---|
| Story ID | SESSION-001 |
| Priority | P1 |
| Status | Planned |
| Depends On | MEM-002 and MEM-004 complete; CMD-001 registry foundation; ADR-005 and ADR-006 |
| Integrates With | TUI-010 command menu; `talos-session`; CLI/TUI composition root |

## Outcome

Users can create, resume, and fork sessions during an interactive Talos process without retaining
the wrong Agent context, persistence target, visible transcript, or runtime resources.

## Problem

Talos can create or resume sessions at process startup, but it does not have one runtime owner for
interactive session transitions. The removed `/new` implementation only cleared conversation
display state while the active Agent/Session retained old context. `/resume` and `/fork` also lack
a complete TUI runtime path.

Implementing these commands as local Conversation or TUI mutations would create split-brain state:
the screen, Agent history, JSONL/SQLite metadata, and subsequent writes could refer to different
sessions.

## Scope

### Typed Lifecycle Operations

Define session-owned operations for:

- `New`: create and activate a fresh workspace-scoped session.
- `ListResumable`: list valid sessions for the active workspace in deterministic order.
- `Resume`: load and activate a selected existing session.
- `Fork`: create and activate an independent child session from the current durable position.

CMD-001 exposes these as BuiltinCommands. Command handlers submit typed lifecycle operations; they
do not mutate Conversation/TUI state directly.

### Atomic Runtime Transition

A successful transition updates one coherent runtime unit:

- active session id and durable JSONL/SQLite target;
- Agent/provider context and turn state;
- conversation engine messages, usage/status snapshot, and branch metadata;
- visible TUI history hydration;
- Skill/MCP session-stable capability state and prompt-cache assumptions;
- cancellation tokens, background tasks, file handles, and other session-owned resources.

The old runtime remains active until the replacement is fully prepared. Preparation or hydration
failure must leave the old session usable and return a visible error. After commit, the old runtime
is shut down deterministically.

### Interaction Rules

- Session candidates are workspace-scoped using MEM-004 identity rules.
- Lifecycle transitions cannot race an active model/tool turn. The policy must explicitly choose
  refusal or user-confirmed cancellation and test that behavior.
- `Fork` never appends subsequent turns to the source session.
- Raw durable history remains authoritative; visible history is hydrated from the activated
  session rather than copied from the previous TUI state.
- `New` resets per-session usage and provenance while preserving process-level configuration.
- Startup flags and interactive commands reuse the same lifecycle service where practical.
- TUI-010 may provide selection UI, but selection presentation does not own lifecycle execution.

## Non-Goals

- Session deletion, rename, tagging, cloud synchronization, or remote control.
- A global session event bus or pub/sub registry.
- Silent cross-workspace resume.
- Reconfiguring model/provider as an accidental side effect of switching sessions.
- Persisting transient approval or popup UI state.

## Acceptance Criteria

- [ ] One session lifecycle service owns `New`, `ListResumable`, `Resume`, and `Fork` operations.
- [ ] `/new`, `/resume`, and `/fork` are registered as CMD-001 BuiltinCommands only after their
      typed runtime operations are executable.
- [ ] A successful transition atomically updates Agent context, persistence target, conversation
      state, status, and visible history.
- [ ] A failed transition leaves the original session active and usable without partial mutation.
- [ ] Resume candidates are limited to the active workspace and ordered deterministically.
- [ ] Forked turns persist only to the fork target and never contaminate the source session.
- [ ] Active-turn behavior is explicit and tested for model streaming and tool execution.
- [ ] Session-owned tasks and handles are cancelled/closed when the old runtime is replaced.
- [ ] Skill/MCP startup state and prompt-cache behavior are rebuilt or safely preserved according
      to their session contracts.
- [ ] CLI startup and interactive transitions do not maintain incompatible lifecycle algorithms.
- [ ] Tests cover new, resume, fork, rollback, workspace isolation, active-turn handling, visible
      history hydration, persistence routing, and resource cleanup.
- [ ] README command documentation is updated only when the commands become executable.

## Delivery Slices

1. Extract the lifecycle service from existing startup create/resume composition.
2. Add transactional `New` and `Resume`, including rollback and TUI history hydration.
3. Add isolated `Fork` with persistence-routing tests.
4. Register BuiltinCommands and connect optional TUI-010 session selection UI.

## Required Reads

- `docs/backlog/active/CMD-001-interactive-command-runtime-contract.md`
- `docs/backlog/active/MEM-002-conversation-context-continuity.md`
- `docs/backlog/active/MEM-004-workspace-session-topology.md`
- `docs/backlog/active/SKILL-001-runtime-skill-activation.md`
- `docs/backlog/active/MCP-001-session-mcp-integration.md`
- `docs/backlog/active/TUI-010-slash-command-menu.md`
- `docs/decisions/005-tui-event-architecture.md`
- `docs/decisions/006-event-architecture-boundary.md`
- `docs/decisions/016-layered-memory-architecture.md`
- `crates/talos-session/src/lib.rs`
- `crates/talos-cli/src/mode_runners.rs`
- `crates/talos-cli/src/tui_bridge.rs`
- `crates/talos-conversation/src/engine.rs`
