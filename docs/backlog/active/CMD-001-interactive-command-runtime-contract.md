# CMD-001: Interactive Command Runtime Contract

| Field | Value |
|---|---|
| Priority | P1 |
| Type | Technical Story |
| Status | In Progress (truthful catalog landed; registry and copy/export remain) |
| Depends On | ADR-006 event boundary; existing ToolRegistry and session seam |
| Integrates With | SESSION-001, TUI-001, TUI-002, TUI-010, SKILL-001, MCP-001, MEM-005, MODEL-001, GIT-001, PLUGIN-001 |

## Problem

The conversation help text and completion list advertised commands without executable runtime
paths. Some were never wired, `/new` only cleared presentation state while leaving the active
Agent/Session intact, and the I014 `/copy` and `/export` paths were lost in later refactoring.
This makes command discovery actively misleading.

## Runtime Audit

| Command | Current state | Correct implementation owner |
|---|---|---|
| `/help` | Executable | Conversation command registry |
| `/quit`, `/exit` | Executable | Existing UI exit output |
| `/status` | Executable | Existing conversation status snapshot; expand only with real session data |
| `/plugins` | Executable | Existing observed provenance diagnostics; MCP-001 later adds loaded-server status |
| `/skills` | Executable Level 0 diagnostics | SKILL-001 adds explicit Level 1/2 activation |
| `/copy last`, `/copy all`, `/export <path>` | Regression: hidden | TUI-001 restoration through typed TUI actions and normal write permission routing |
| `/new`, `/resume`, `/fork` | Hidden; `/new` presentation-only implementation removed | SESSION-001 typed lifecycle operations replace or fork the active runtime, persistence target, and visible transcript together |
| `/compact` | Hidden | MEM-005 manual compaction policy and session actor integration |
| `/diff` | Hidden | GIT-001 read-only Git capability, rendered through the normal tool/result path |
| `/model` | Hidden | MODEL-001 catalog plus provider/session reconfiguration semantics |
| `/vim` | Hidden | TUI-002 composer/keymap state, not conversation state |
| `/mock-request` | Internal, hidden | Mock-provider diagnostics only; never part of the normal command catalog |

## Scope

1. Keep help, completion, and the future TUI-010 popup limited to commands that have an executable
   path in the active runtime state.
2. Introduce shared `CommandDefinition` metadata for parser, help, completion, and menu rendering.
   Runtime/UI commands own their definitions directly. Tool-backed commands reference a registered
   tool name and derive description, argument schema, read/write nature, and availability from the
   actual `AgentTool`/`ToolDefinition`; they must not duplicate those fields in the command layer.
3. Route commands to their real owner with typed operations. Conversation code must not pretend to
   perform Session, Provider, Git, clipboard, filesystem, or keymap work by mutating display state.
4. Support command availability predicates so commands can be hidden or disabled when their owner
   is unavailable.
5. Restore I014 copy/export behavior before closing TUI-001 again.

## Command Origins

The session-scoped `CommandRegistry` aggregates exactly two definition origins:

1. **BuiltinCommand** — registered by Talos modules. Its executor is an explicit typed owner such
   as Conversation, Session, TUI, or a named registered Tool. A tool-backed built-in command derives
   tool metadata from the live tool definition and executes through the normal tool pipeline.
2. **PluginCommand** — registered by a loaded plugin during plugin initialization. It carries plugin
   identity/version provenance and executes through the plugin adapter with timeout, output, host-call,
   and permission limits.

Both origins share the user-facing command metadata contract: stable id, invocation name, aliases,
usage, description, argument schema, availability, provenance, and executor reference. Help,
completion, and TUI-010 consume the merged registry rather than origin-specific lists.

Registration rules:

- Built-in command names are reserved and cannot be overridden by plugins.
- Plugin command ids are always namespaced by plugin id. User-facing short aliases are optional and
  accepted only when they do not conflict with built-in commands or another active plugin.
- Duplicate or malformed plugin command definitions reject that capability without crashing Talos.
- Plugin unload/failure removes its commands from the active registry and refreshes availability.
- Plugin commands cannot emit arbitrary internal events or mutate Session/TUI state directly. They
  return protocol-owned results and may request only explicitly allowed host operations.

## Relationship To Other Requirements

| Requirement | Command relationship |
|---|---|
| SKILL-001 | `/skills` and a future explicit activation command are BuiltinCommands owned by the Skill/session integration. A `SKILL.md` file does not gain arbitrary command registration rights. |
| MCP-001 / I034 | MCP tools remain ToolRegistry entries. `/plugins` is a BuiltinCommand reading session integration status. MCP prompts do not automatically become a third command origin; any future adapter requires an explicit protocol decision. |
| SESSION-001 | Owns runtime `New`, `Resume`, and `Fork`; CMD-001 only defines and dispatches their BuiltinCommand entries. |
| TUI-001 | First remediation consumer: restore `/copy` and permission-gated `/export` as typed BuiltinCommand actions. |
| TUI-010 | Presentation consumer only: renders, filters, and selects the merged registry and never owns a separate command list. |
| TUI-008 | Approval may share the future input-layer popup stack, but Command execution cannot directly manipulate approval UI. Permission requests stay on the existing unified event path. |
| MEM-005 | `/compact` is a BuiltinCommand delegating to session compaction policy; it does not call compaction code from Conversation/TUI state. |
| MODEL-001 | `/model` is a BuiltinCommand delegating model/provider session reconfiguration after catalog and cache semantics exist. |
| GIT-001 | `/diff` is an explicit tool-backed BuiltinCommand alias resolving the live `git_diff` definition and normal tool result rendering. |
| PLUGIN-001 | Defines the PluginCommand wire descriptor, adapter, resource limits, provenance, and lifecycle. CMD-001 defines the host registry contract. |
| DIST-001 | Governs installation and verification of packages that may later register PluginCommands; installed assets do not execute or register until plugin loading succeeds. |
| AGENT-001 / I035 | Shared config may enable plugins or command alias policy, but imported config cannot define executable command bodies. |
| ADR-006 | Command dispatch uses the existing single-consumer/session seam. Registry extensibility does not create global pub/sub. |

## Delivery Order

1. Keep the current built-in catalog truthful and restore TUI-001 regressions.
2. Extract the public BuiltinCommand definition/handler/registry contract and route help,
   completion, parser, and TUI-010 through it.
3. Add typed domain executors only as their owner stories land: Session, compaction, model, Git.
4. Leave PluginCommand ABI/runtime implementation to PLUGIN-001 after its ADR. Plugin support must
   not block this built-in registry Story.

## Non-Goals

- Do not implement all commands in one conversation-engine match statement.
- Do not present every Agent tool as a slash command. A tool appears only when an explicit command
  alias references it; model tool invocation and user command invocation remain distinct protocols.
- Do not bypass the permission pipeline for `/export` or future write-capable commands.
- Do not introduce a global event bus; follow ADR-006 and the existing single-consumer flow.
- Do not implement PluginCommand loading or Session lifecycle operations in this Story; PLUGIN-001
  and SESSION-001 own those outcomes.

## Acceptance Criteria

- [x] `/help` and completion no longer expose placeholder or internal-only commands.
- [x] `/new` no longer claims to create a session by clearing presentation state only.
- [x] A regression test executes every visible command and rejects `Unknown command` results.
- [ ] Parser, help, completion, and TUI-010 consume one shared `CommandDefinition` registry.
- [ ] Tool-backed command metadata and availability resolve from the live tool registry rather than
      copying tool descriptions or schemas into command code.
- [ ] `/copy` and `/export` have end-to-end TUI-facing tests and TUI-001 is closed again.
- [ ] Each remaining command is exposed only after its owner story supplies an executable typed path.
- [ ] `README.md` documents only commands proven executable through the runtime path.
- [ ] CMD-001, TUI-001, TUI-010, Product Backlog, iteration record, and Board statuses are synchronized.

## Required Reads

- `docs/decisions/006-event-architecture-boundary.md`
- `docs/backlog/active/TUI-001-completion.md`
- `docs/backlog/active/TUI-010-slash-command-menu.md`
- `docs/backlog/active/MEM-005-context-compaction-policy.md`
- `docs/backlog/active/MODEL-001-model-catalog-and-reasoning.md`
- `docs/backlog/active/GIT-001-embedded-git-tools.md`
- `docs/backlog/active/PLUGIN-001-wasm-runtime-plugins.md`
- `docs/backlog/active/MCP-001-session-mcp-integration.md`
- `docs/backlog/active/SESSION-001-interactive-session-lifecycle.md`
- `crates/talos-conversation/src/engine.rs`
- `crates/talos-cli/src/tui_bridge.rs`
