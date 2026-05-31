# Talos Architecture Reference

Talos is a safety-first agent runtime built in Rust. It prioritizes minimal core logic, strict permission gating, and high extensibility through hooks and plugins.

## Design Principle: Simple Core, Flexible Extensions

Talos follows the Pi-inspired principle of building the simplest possible core and extending it incrementally:

1. **Core is minimal**: Turn loop + tools + provider. Nothing else.
2. **Complexity is introduced on demand**: Each iteration adds only what its features require.
3. **Abstractions emerge from implementation**: Traits are extracted when a second implementation appears, not designed upfront for hypothetical future needs.
4. **File-based by default**: Everything human-editable stays as files (TOML, Markdown, JSONL). Databases only when queries demand it.

## System Overview

The system operates as a stateful turn loop. It processes user input by orchestrating LLM calls, tool executions, and context management. Safety is enforced at every layer, from permission checks to sandboxed execution.

## Cargo Workspace Structure

Talos crates are introduced progressively across iterations (see Implementation Roadmap for schedule).

### Core Crates (I001, always present)

| Crate | Responsibility |
|-------|----------------|
| `talos-core` | Foundation types, core traits, and error definitions. No internal dependencies. |
| `talos-config` | Configuration schema, validation, and environment substitution. |
| `talos-provider` | LLM client abstractions and provider-specific implementations. |
| `talos-agent` | Core orchestration logic and the agent turn loop. |
| `talos-cli` | Primary command-line interface and terminal user experience. |

### Extension Crates (introduced as needed)

| Crate | Iteration | Responsibility |
|-------|-----------|----------------|
| `talos-tools` | I003 | Implementations of standard system and developer tools. |
| `talos-session` | I003 | Persistence layer for message history and session state. |
| `talos-sandbox` | I004 | Process isolation, filesystem virtualization, and secure execution environments. |
| `talos-permission` | I004 | Policy engine, capability-based security, and user approval workflows. |
| `talos-tui` | I005 | Terminal user interface with ratatui + crossterm, evolving progressively. |
| `talos-skill` | I007 | Management of higher-level agent capabilities and task-specific instructions. |
| `talos-evolution` | I008 | Runtime self-evolution: observe, accumulate, extract, apply learning loop (ADR-001). |
| `talos-plugin` | I009 | Plugin runtime for third-party extensions (hook-based first, WASM as option). |
| `talos-mcp` | I009 | Model Context Protocol implementation for external tool and resource access. |
| `talos-rpc` | I009 | API layer for remote interaction and frontend integration. |

## Dependency Graph

The architecture follows a strict hierarchy to prevent circular dependencies.

```text
[ talos-cli / talos-rpc ]
          |
          v
    [ talos-agent ]
    /     |     \
   v      v      v
[tools][session][provider][permission][skill][plugin][mcp]
   \      |      /           |           |      /     /
    \     v     /            v           v     /     /
     [ talos-core ] <-------------------------------'
```

Every crate depends on `talos-core`. Intermediate crates like `talos-agent` aggregate functionality from specialized modules.

## Core Data Flow

Data flows through a structured pipeline to ensure consistency and safety.

```text
User Input -> Session (History) -> Agent Loop -> LLM Provider
                                                     |
                                                     v
Response <- Session (Update) <- Tool Execution <- LLM Output
```

1. **User Input**: Received via CLI or RPC.
2. **Session**: Input is appended to history. Context is prepared for the LLM.
3. **Agent Loop**: Coordinates the turn.
4. **LLM Provider**: Generates text or tool calls.
5. **Tool Execution**: Tool calls are validated, approved, and run in sandboxes.
6. **Response**: Final results are stored and returned to the user.

## Key Traits

Talos uses traits to decouple logic and allow for alternative implementations.

*   `AgentTool`: Interface for defining tool behavior, metadata, and input schemas.
*   `LanguageModel`: Abstraction for LLM providers to handle completion and streaming.
*   `SandboxProvider`: Defines how to spawn and manage isolated execution environments.
*   `PermissionEngine`: Logic for checking tool calls against active policies.
*   `SkillProvider`: Interface for loading and injecting domain-specific knowledge.
*   `PluginHost`: Manages the lifecycle and hooks for WASM-based extensions.

## Async Pattern (SQ/EQ)

Talos uses a dual-channel architecture for asynchronous communication.

*   **Submission Queue (SQ)**: A bounded channel for sending commands to the agent loop. This prevents the system from being overwhelmed by requests.
*   **Event Queue (EQ)**: An unbounded channel for streaming status updates, logs, and partial results back to the UI.

This separation ensures that the core agent loop remains responsive while providing real-time feedback.

## Turn Loop Lifecycle

Each turn in the agent loop follows a deterministic lifecycle.

1. **Prepare**: Fetch session history and compact context.
2. **Predict**: Send the prompt to the LLM.
3. **Analyze**: Parse the LLM response for content or tool calls.
4. **Execute**: If tool calls exist, route them through the tool pipeline.
5. **Observe**: Capture tool output and append it to the session.
6. **Finish**: Determine if the task is complete or if another turn is needed.

## Self-Evolution Engine (I008)

The evolution engine implements a 4-phase learning loop per ADR-001.

### Learning Loop

```text
Observe -> Extract -> Store -> Apply
   ^                           |
   |___________________________|
```

1. **Observe**: `TurnObserver` captures signals (error, correction, satisfaction, inefficiency) with intensity scores.
2. **Extract**: `PatternExtractor` identifies patterns from observations using rule-based logic with contradiction detection.
3. **Store**: `KnowledgeStore` persists patterns in SQLite with confidence scores and evidence counts.
4. **Apply**: `BehaviorAdapter` injects high-confidence patterns into the system prompt.

### Cognitive Feedback

Patterns use evidence-based confidence scoring with 70-day half-life time decay:

- **Confidence**: Increases with supporting evidence, decreases with contradictions
- **Time Decay**: Older evidence has less weight (half-life: 70 days)
- **Minimum Threshold**: Only patterns with confidence ≥ 0.7 and evidence ≥ 3 are injected

### Integration Points

- **TUI Evolution Panel**: Visual display of learned patterns (Ctrl+E to toggle)
- **`--learned` Command**: CLI command to inspect evolution data
- **System Prompt Assembly**: High-confidence patterns are injected as natural language instructions

## Tool Execution Pipeline

Tools never run with direct system access. They follow a four-stage pipeline.

```text
LLM Request -> [ Approval ] -> [ Sandbox ] -> [ Execute ] -> [ Retry ]
```

1. **Approval**: The `PermissionEngine` checks if the tool call matches allowed patterns. If not, it prompts the user.
2. **Sandbox**: The `SandboxProvider` creates an isolated environment (e.g., a restricted directory or container).
3. **Execute**: The tool runs inside the sandbox.
4. **Retry**: If the tool fails with a transient error, the pipeline can attempt a recovery or ask the LLM to fix the input.

## Context Compaction Pipeline

To handle long conversations, Talos uses a progressive compaction strategy. Layers are activated as context pressure increases:

1. **Pinned**: Critical system instructions and user-defined constraints that never expire.
2. **Fresh**: The most recent messages, kept in full detail.
3. **Summarized**: Mid-term history reduced to high-level summaries via LLM call.
4. **Archived**: Older history excluded from the active prompt.

> **Future consideration**: If RAG-based retrieval of older context becomes necessary (no reference project currently uses this), it would be implemented as an extension rather than a core compaction layer.

## Storage Architecture

Talos uses a progressive storage strategy (ADR-002). Storage complexity is introduced incrementally
as each iteration requires it.

### Phase 1: Pure Files (I001–I005)

No database dependency. All data is file-based:

*   **Sessions**: JSONL append-only logs (`~/.talos/sessions/<project>/<id>.jsonl`). One JSON object
    per line. Crash-safe (only the last line can be corrupted).
*   **Configuration**: TOML files with `${ENV_VAR}` substitution and layered merging
    (`~/.talos/config.toml` + `.talos/config.toml`).
*   **Permission rules**: Inline in configuration (no separate rule files yet).

### Phase 2: SQLite Introduction (I006)

Session metadata indexing and full-text search require a database:

*   **SQLite** (via `rusqlite`, bundled compilation): `~/.talos/index.db`.
*   **Session messages** remain as JSONL files (source of truth). SQLite stores metadata only.
*   **FTS5** virtual table for full-text search across session content.
*   All storage operations abstracted behind `SessionStore` trait for future engine migration.

### Phase 3: SQLite Extension (I008)

Evolution engine requires structured queries for observations and patterns:

*   Same SQLite database, extended with `observations`, `patterns`, and `pattern_conflicts` tables.
*   Patterns include cognitive feedback fields: confidence, evidence counts, time decay (ADR-001).
*   All evolution operations abstracted behind `EvolutionStore` trait.

### File-Based Domains (All Phases)

These domains remain file-based permanently because they must be human-editable:

*   **Configuration**: TOML files (layered: global → project).
*   **Skills**: Markdown files with YAML frontmatter (`.talos/skills/**/SKILL.md`).
*   **Permission rules**: TOML/DSL files (`.talos/rules/*.rules`).
*   **Agent context**: Markdown files (`AGENTS.md` at project root and `~/.talos/AGENTS.md`).

### Storage Implementation (Phases 2-3)

SQLite is used directly via rusqlite calls. No trait abstraction until a concrete second implementation exists (YAGNI — trait extraction happens when Turso or another engine is production-ready and we have real migration needs).

## Plugin System

Extensions in Talos follow a layered approach, starting simple and adding sandboxing when needed:

1. **Hook system** (I009, first): Function hooks at key lifecycle points (before_tool_call, after_tool_call, message_transform, etc.). Plugins register handlers. Simplest to implement and debug.

2. **Native plugins** (future): Dynamic library loading (`.so`/`.dylib`) for Rust plugins. Direct access to Talos APIs, zero serialization overhead.

3. **WASM sandboxing** (future, optional): For untrusted third-party plugins. Adds sandboxing at the cost of complexity and API restrictions.

The hook system is the foundation — WASM and native plugins are alternative hosting mechanisms for the same hook interface. This matches Pi's ExtensionAPI pattern: `registerTool`, `registerCommand`, `on(event)`.
