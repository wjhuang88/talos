# Talos Architecture Reference

Talos is a safety-first agent runtime built in Rust. It prioritizes minimal core logic, strict permission gating, and high extensibility through hooks and plugins.

## Design Principle: Simple Core, Flexible Extensions

> **Current-state note (2026-08-07).** The workspace and composition statements in this
> reference were checked against the root `Cargo.toml` and the current source tree for I180.
> Iteration labels in the tables below record when a capability first entered the plan; they are
> provenance, not a claim that the capability is still in that iteration's implementation shape.

Talos follows the Pi-inspired principle of building the simplest possible core and extending it incrementally:

1. **Core is minimal**: Turn loop + tools + provider. Nothing else.
2. **Complexity is introduced on demand**: Each iteration adds only what its features require.
3. **Abstractions emerge from implementation**: Traits are extracted when a second implementation appears, not designed upfront for hypothetical future needs.
4. **File-based by default**: Everything human-editable stays as files (TOML, Markdown, JSONL). Databases only when queries demand it.

## System Overview

The system operates as a stateful turn loop. It processes user input by orchestrating LLM calls, tool executions, and context management. Safety is enforced at every layer, from permission checks to sandboxed execution.

## Cargo Workspace Structure

The root `Cargo.toml` is the source of truth for workspace membership. It currently contains 21
members. The tables describe each member's current responsibility; the origin column preserves the
historical iteration or decision that introduced the boundary.

### Foundation And Product Composition Crates

| Crate | Origin | Current responsibility |
|-------|--------|----------------------|
| `talos-core` | I001 | Dependency-free protocol types, tool/provider/session contracts, and core errors. |
| `talos-config` | I001 | Configuration schema, validation, credential loading, and environment substitution. |
| `talos-provider` | I001 | LLM client abstractions and provider-specific implementations. |
| `talos-agent` | I001 | Agent orchestration, turn loop, session actor, and runtime scheduler. |
| `talos-cli` | I001 | Product composition root, command-line modes, permission wrappers, and terminal workflows. |

### Capability And Extension Crates

| Crate | Iteration | Responsibility |
|-------|-----------|----------------|
| `talos-tools` | I003 | Built-in shell, file, workspace, network, image, git, and symbol tools plus crate-owned contributions. |
| `talos-session` | I003 | JSONL transcript source of truth, session indexes, runtime state, and session-bound Todo tools. |
| `talos-sandbox` | I004 | Process execution abstraction and OS hardening controls. |
| `talos-permission` | I004 | Permission policy evaluation, resource matching, and approval decisions. |
| `talos-tui` | I005 | Terminal rendering, input, scrollback, and UI state for the CLI TUI. |
| `talos-skill` | I007 | SKILL.md discovery, parsing, indexing, and progressive-disclosure loading. |
| `talos-evolution` | I008 | Hook-driven observation, pattern extraction, storage, and prompt adaptation (ADR-001). |
| `talos-plugin` | I009 / ADR-027 | Hook registry/handlers, manifest validation, and explicit local WASM tool adapters. Native dynamic loading is not supported. |
| `talos-mcp` | I009 | MCP transport, DTOs, server dispatch, and client-side remote tool adapters. |
| `talos-rpc` | I009 | RPC protocol and runtime adapter boundary for remote interaction. |
| `talos-conversation` | I023 / ADR-052 | Agent/TUI conversation projection and typed `UiOutput` events; experimental product-oriented crate. |
| `talos-runtime` | RUNTIME-001 | Embeddable SDK facade with safe runtime construction and typed session handles; no CLI/TUI dependency. |
| `talos-memory` | I019 | SQLite-backed episodic/semantic/procedural memory and consolidation. |
| `talos-exploration` | I054 | Research-library ingestion and SQLite/FTS-backed search over local sources. |
| `talos-dashboard` | WEB-001 / ADR-031 | Product-only loopback control/dashboard library; not wired into the SDK. |
| `talos-models` | MC-002 (quarantined) | Historical/non-runtime SQLite catalog crate; runtime uses packaged `models.toml` and must not wire this member into CLI/TUI/runtime paths. |

### Crate Distribution Boundary (ADR-052)

The full workspace membership is defined by root `Cargo.toml` (source of truth). Per
[ADR-052](../decisions/052-sdk-publication-and-composition-boundary.md) the current distribution
boundary is:

- `talos-runtime` — supported SDK facade (embedders use `RuntimeBuilder` / `RuntimeHandle`).
- `talos-agent` — implementation dependency; published only to satisfy the runtime closure, NOT a
  supported SDK entrypoint.
- `talos-conversation` — experimental, product-oriented; published but NOT a general-purpose UI SDK.
- `talos-cli` and `talos-runtime` — share internal composition direction (single Agent/session/tool
  assembly), but this shared layer is NOT yet extracted; no new composition crate is authorized.
- Product-only (`talos-cli`, `talos-tui`, `talos-evolution`, `talos-dashboard`) and quarantined
  (`talos-models`) crates carry `publish = false`.

See `CRATE-PUBLICATION-MATRIX.md` for per-crate publication/readiness state.

### Session Persistence Boundary

`talos-session` is the persistence boundary for local conversation history and session indexes. Its
public API is intentionally re-exported from `lib.rs`, while implementation details live in focused
modules:

| Module | Responsibility |
|--------|----------------|
| `types.rs` | Public session data types and in-memory branch helpers. |
| `jsonl.rs` | Append-only JSONL source-of-truth persistence, replay, preview scanning, and compatibility reads for old JSONL lines. |
| `topology.rs` | Workspace directory identity helpers for workspace-scoped session layout. |
| `manager.rs` | `SessionManager` disk scanning, resume/list/search coordination, and lazy SQLite index access. |
| `sqlite.rs` | SQLite FTS/session index implementation. |
| `error.rs` | Session error surface. |

The session actor (`AppServerSession`) is part of `talos-agent`; queue protocol types (`SessionOp`,
`SessionEvent`, `SessionHandle`, `SessionConfig`) live in `talos-core::session`.

### Skill Loading Boundary

`talos-skill` owns SKILL.md parsing and progressive-disclosure loading. The CLI discovers Level 0
metadata at session startup and injects it into the Agent's stable prompt prefix; this crate
provides the parsed and indexed skill data while explicit Level 1/2 activation remains separate.

| Module | Responsibility |
|--------|----------------|
| `types.rs` | Public skill data types and disclosure-level enum. |
| `parser.rs` | Frontmatter splitting and validation for SKILL.md files. |
| `loader.rs` | Filesystem discovery, SKILL.md parsing, default search-path construction, and Level 0 index generation. |
| `manager.rs` | Progressive-disclosure cache: Level 0 index, Level 1 skill loading, Level 2 reference loading, trigger matching. |
| `token.rs` | Lightweight token estimation for skill index budgeting. |
| `error.rs` | Skill error surface. |

### CLI Runtime Boundary

`talos-cli` is the product composition root. It keeps argument parsing and top-level dispatch in
`main.rs`, while mode runners assemble Agent/session/provider/tool state from explicit runtime
inputs. There is no shared composition crate: `talos-runtime` is a separate SDK facade with its own
construction boundary.

| Module | Responsibility |
|--------|----------------|
| `main.rs` | `Cli`, `Mode`, logging setup, mode selection, and hook registry construction. |
| `mode_runners.rs` | Shared orchestration for TUI, legacy interactive, RPC, and MCP modes; re-exports print and inline entrypoints. |
| `mode_print.rs` / `mode_inline.rs` / `mode_interactive.rs` | Mode-specific execution bodies. |
| `registry.rs` | Explicit print/TUI/MCP/interactive tool profiles, contribution collision checks, plugin loading, and permission-aware wrappers. |
| `mode_runtime.rs` | Shared session metadata, model/runtime identity, prompt context, memory, and Todo prompt setup. |
| `tui_runtime_builder.rs` | Single TUI Agent/session construction boundary, including provider, MCP, scheduler, plugins, skills, and prompt policy. |
| `session_setup.rs` | Workspace/session resolution and session utility modes (`--search`, `--list`, `--learned`). |
| `provider_setup.rs` | Provider parsing and provider/client config construction. |
| `mcp_runtime.rs` | Session-scoped MCP startup, cached discovery results, child-process lifetime, and status projection. |
| `session_handlers.rs` / `model_lifecycle.rs` / `session_transition.rs` | Session command workflows, model activation transactions, and generation-fenced runtime replacement. |
| `runtime_adapter.rs` | RPC adapter over the canonical session SQ/EQ protocol. |
| `tui_bridge.rs` / `event_loop.rs` | TUI and legacy interactive event-loop bridges. |

### MCP Session Boundary

`talos-mcp` owns transport, dispatch, Talos-owned MCP DTOs, startup discovery, and remote tool
adapters. `talos-cli::mcp_runtime` is the composition boundary: it starts configured local stdio
servers once per session, retains their process lifetime, registers cached tool adapters through
the mode's existing permission wrapper, and projects startup status into conversation diagnostics.
The discovered tool set is session-stable; Talos does not mutate model-visible tools mid-session.
Unavailable servers are reported and skipped, while request timeouts and process drop cleanup keep
dependency failures bounded.

## Dependency Graph

The graph is acyclic, but it is not a single chain and not every workspace member directly depends
on `talos-core`. The following is the current composition shape derived from crate manifests:

```text
talos-cli (product root)
├── talos-agent ──┬── talos-core
│                 ├── talos-tools ── talos-core + talos-sandbox
│                 ├── talos-permission ── talos-core
│                 ├── talos-sandbox ── talos-core
│                 ├── talos-skill
│                 ├── talos-plugin ── talos-core + talos-permission
│                 ├── talos-memory
│                 └── talos-session ── talos-core
├── talos-provider ── talos-config ── talos-core
├── talos-conversation ── talos-core + talos-plugin
├── talos-tui ── talos-core + talos-permission + talos-conversation
├── talos-mcp ── talos-core + talos-plugin + talos-permission
└── talos-rpc ── talos-core + talos-plugin

talos-runtime (independent SDK root) composes agent/provider/tools/session/permission/
sandbox/skill/plugin without depending on talos-cli or talos-tui.
talos-dashboard, talos-exploration, talos-memory, and talos-skill have no internal workspace
dependencies, although product/orchestrator crates consume them; `talos-models` is quarantined and
has no runtime consumer.
```

The TUI information flow remains `AppServerSession → ConversationEngine → UiOutput → Tui`, with
user input returning over the corresponding command channel.

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
*   `HookHandler`: Interface for lifecycle hook handlers managed by `talos-plugin::HookRegistry`.
*   `Runtime`: RPC runtime contract implemented by the CLI adapter.

`PermissionEngine` is a concrete policy evaluator rather than a trait. Skill loading is provided
by concrete `talos-skill` loaders/managers, and plugin execution is exposed through explicit
`HookRegistry` and WASM adapter types; there are no `SkillProvider` or `PluginHost` traits in the
current source.

## Tool Presentation

`ToolRegistry` is the executable source of truth. Model-visible tools are selected by
`ToolPresentationPolicy`, which filters registered tools by explicit `ToolFamily` metadata plus an
always-on baseline for common file/search/edit workflows. The Agent derives both prompt tool
descriptions and native provider `ToolDefinition`s from the same selected set. If a model requests
a registered tool that was not presented, Talos returns a recoverable tool error and does not
execute the tool.

Tool prompt content is grouped into stable family sections. Adding or removing one family should
not rewrite unchanged family blocks, preserving provider cache friendliness.

### Tool Contribution And Profile Composition

Concrete tools are declared by the crate that implements them and selected explicitly by the
product composition root. `talos-core` owns only `AgentTool`, `ToolContribution`, source identity,
collision diagnostics, presentation policy, and `ToolRegistry`; it does not depend on concrete
tool crates.

| Owner | Contribution boundary | Composition verdict |
|---|---|---|
| `talos-tools` | Shell, file, workspace, network, image, git, and symbol contribution functions | Authoritative built-in declarations |
| `talos-session` | Session-bound Todo contributions | Authoritative session-tool declarations |
| `talos-cli` | Print/TUI/MCP/interactive profile selection and permission wrappers | Expected outer composition root |
| `talos-agent` scheduler | Runtime-created tools passed into print/TUI construction | Explicit runtime-injection exception; no static contribution can own the live scheduler handle |
| CLI MCP `status` | Product-local diagnostic tool | Explicit CLI product exception; not a reusable capability crate |
| Explicit WASM plugins | `ToolContribution` wrappers sourced as `plugin:<name>@<version>` | Local package selection is explicit; checked registration rejects duplicate names with both sources |
| MCP adapters | Session-discovered `AgentTool` values registered by the CLI's MCP profile | Process-isolated runtime extension; current adapter path preserves legacy unchecked registration and stable-per-session discovery |

Adding a reusable built-in starts in its implementing crate's contribution module, then adds an
explicit capability/profile selection at the applicable composition root. Do not duplicate the
constructor across every registry builder, hide registration in global initialization, or move
runtime/product-specific dependencies into `talos-core`.

## Crate Distribution

Talos has three distribution layers:

1. Binary/product distribution through the `talos` CLI release artifacts and installers.
2. The embeddable SDK facade through `talos-runtime`.
3. Standalone capability crates such as `talos-core`, `talos-config`, `talos-permission`,
   `talos-skill`, `talos-session`, and later provider/tool/storage crates.

The crate publication model follows the ripgrep-style boundary: product crates aggregate reusable
library crates, while reusable crates must not depend on CLI/TUI/product assumptions. Publication
readiness is tracked in `docs/reference/CRATE-PUBLICATION-MATRIX.md`. Real crates.io publication or
placeholder name reservation remains a separate maintainer-approved release action.

## TUI Event-Driven Architecture (I023, corrected by I115)

The TUI follows a single-directional information flow:
`AppServerSession → ConversationEngine → UI`.

### ConversationEngine (`talos-conversation`)

Projects ordered session events into conversation-visible messages, status, and model information.
The session actor owns authoritative user-turn lifecycle, Agent history, and successful turn-message
persistence. The TUI holds only rendering/input state.

```text
┌─────────────────────┐     UiOutput (mpsc)     ┌──────────────┐
│  ConversationEngine │ ──────────────────────> │     Tui      │
│  (ordered projection)│                        │  (UI state)  │
│                     │ <────────────────────── │              │
└─────────────────────┘     UserInput (mpsc)    └──────────────┘
```

State-critical events arrive as `SessionEvent::TurnEvent { session_id, turn_id, sequence, payload }` through a
non-lossy queue. `TurnEventPayload::Started` and `Completed` are the only authoritative user-turn
lifecycle. Provider `AgentEvent::TurnStart`/`TurnEnd` delimit provider responses inside a turn and
must not complete a user turn or drain steering.

Cancellation is part of the same contract. When TUI input produces
`UserInput::Cancel`, the integration layer must send `SessionOp::Interrupt` to
the session actor and let `ConversationEngine` update its own processing state
through an explicit cancellation method. UI-only cancellation hints are not a
valid backend interrupt.

### UiOutput Event Types

| Variant | Purpose |
|---------|---------|
| `Content(ContentOutput)` | Canonical FIFO content path: `Start`, `Delta`, `End`, or atomic `Block`. |
| `Stream { stream, source }` | Legacy public compatibility input only; in-tree runtime producers do not use it. |
| `Status { snapshot }` | Status update (model name, token usage, processing state). |
| `Tip { text, kind }` | Transient tip message with TTL auto-expiry. |
| `ToolCallStarted { name }` | Lightweight tool-start marker for paths that do not yet have full display metadata. |
| `ToolCall(ToolCallDisplay)` | Full tool call display event with tool name, arguments, provenance, and summary fields. |
| `ToolResult(ToolResultDisplay)` | Tool result display event with tool name, error flag, and content or summary policy. |
| `ToolApprovalRequest` | Inline approval request flowing through the same `UiOutput` channel; TUI returns the user's decision through a oneshot response. |
| `Exit` | Signal to terminate the UI loop. |

### Ordered Content Consumption

Live content shares the same FIFO `UiOutput` queue as tool, reasoning, status, and lifecycle
projections:

1. `Content::Start` opens one logical message block.
2. `Content::Delta` passes text to `consume_stream_chunk`, which splits complete lines and updates
   the preview.
3. `Content::End` finalizes the same block before any later FIFO tool/status output is handled.
4. `Content::Block` renders a complete user/system/reasoning message atomically.
5. `flush_pending_scrollback` writes styled lines to terminal scrollback.

There is no nested live-text receiver competing with `UiOutput` in `select!`; this prevents a later
tool boundary from closing a receiver that still contains earlier text.

### Line Padding System

Each scrollback line carries a three-column prefix aligned with the input box prefix (` > `):

| Source | First Line | Continuation |
|--------|-----------|--------------|
| User | ` > ` | `   ` |
| Assistant | ` ● ` | `   ` |
| System | ` # ` | `   ` |
| Error | ` ! ` | `   ` |
| Tool | ` ● ` | `   ` |

### Styled Scrollback

`ScrollbackLine` carries plain text, styled `HistorySegment`s, optional background color, and an
optional fill segment for full-row elements such as Markdown horizontal rules. User message lines
receive the Nord Polar Night background (`#3B4252`) via `crossterm::style::SetBackgroundColor`.
Empty padding lines fill the full terminal width with spaces so the background color covers the
entire row.

User messages are visually grouped with top/bottom padding rows (same background
color), creating a block effect. Each stream after the first is preceded by a
blank separator line when that stream's first non-empty chunk arrives.

Multiline user input is one stream block. Bracketed paste appends the pasted text
to the input buffer, including newlines; Enter submits the whole buffer. When the
user block is flushed to scrollback, only the first line receives the ` > `
prompt marker. Continuation lines retain the three-column alignment with spaces.

The same prefix rule applies to every `ContentOutput` source. Content blocks are
logical message blocks, but the TUI writes complete lines to terminal history as
soon as they arrive. The source prefix is rendered only for stream-local line 0;
all later lines use the blank three-column prefix. Incomplete trailing text stays
in the live preview until the next newline or stream completion.

`talos-tui` keeps this state in a private stream-render helper rather than in
the terminal writer. That helper owns the active source, stream-local line
counter, incomplete line buffer, preview text, source prefix rendering, and
source-specific scrollback rows such as the user block background padding.
`InlineTerminal` remains a single-line history writer; it does not parse message
blocks, markdown, or table layout.

The stream-render helper may hold complete stream-local lines internally for
future block renderers, but the default runtime mode is immediate line emission.
Hold mode is a private preparation boundary: it changes when `ScrollbackLine`s
are emitted from the helper, not how terminal history is written.

### Markdown And Block Rendering Direction

Markdown rendering must preserve the inline-terminal stability contract. The
live preview remains exactly one row. Markdown that can be represented as a
single streaming line may render in preview and flush complete lines to history
immediately. Markdown that requires block context, such as tables or fenced code
blocks, is held locally by the stream-render helper; while held, preview shows a
single-row animation/status such as `rendering table...` or `receiving code
block...`. When the block boundary is reached, the helper renders the block to
history rows and `InlineTerminal::insert_history` writes those rows one at a
time.

Block detection belongs to a deterministic TUI-side classifier, not to
`InlineTerminal` and not to `talos-conversation`. The classifier must expose
the block kind, held line/byte counts, and boundary hint so preview status can
explain why raw content is hidden. It must also have visible fallback behavior:
malformed, oversized, or unterminated blocks are flushed as plain rows rather
than dropped.

The detailed target design and test matrix are tracked as a proposal in
[`docs/proposals/tui-stream-markdown-rendering.md`](../proposals/tui-stream-markdown-rendering.md).

### Native Cursor Sync

After each `draw_frame` render, the native terminal cursor is repositioned to the input box position using `MoveTo(col, row)` + `Show`. The column is calculated as 3 (prefix width) + Unicode display width of text before the cursor. This ensures IME input, text selection, and other cursor-dependent features work correctly.

### Inline Terminal Rendering

The inline-by-default TUI (I022) uses a fixed viewport within the terminal. History content is written above the viewport using `insert_history(line, bg)`:

- **Non-bottom**: `\x1bM` pushes viewport down one row, history line written at the vacated position.
- **Bottom**: Scroll region `[1, viewport_top]` + `\r\n` scrolls history up, history line written at the bottom of the history area.
- When `bg` is set, the line is wrapped with `SetBackgroundColor` / `Reset` and padded to full terminal width with trailing spaces so the background color covers the entire row.
- Both branches set `needs_clear = true` so the next `draw_frame` performs a force-clear + full diff redraw of the viewport.

On exit, `restore()` clears the viewport area (`MoveTo` + `Clear(ClearType::FromCursorDown)`) before disabling raw mode and restoring the cursor.

### Preview Component

Always occupies exactly 1 row in the viewport. Shows `streaming_preview` content (partial stream content not yet terminated by `\n`). User messages have no trailing `\n` so they stay in preview until the AI stream arrives. The preview padding shows an animated 2-char braille spinner with Nord color gradient when `is_processing` is true, or 3 spaces when idle.

### Queued User Input

When a user submits normal text while a turn is already processing, the
conversation engine stores it in the steering queue and emits a queue status
update. The queued text is not rendered as a user message yet. After the active
turn ends, the bridge drains one queued message, calls `start_user_message` for
that drained text, emits the resulting user stream/status to the TUI, and only
then submits it to the session actor. This keeps scrollback, transcript state,
queue counters, and the actual session submission in the same order.

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

Permission checks use invocation-specific profiles. A simple tool exposes one facet derived from
`ToolNature`; a hybrid tool exposes every risk surface through `ToolPermissionFacet` plus a
resource kind such as path, domain, command, or remote. The permission engine evaluates all facets
conservatively: any denied facet denies the call, otherwise any ask facet requires approval, and
only an all-allow profile executes. Agent, CLI/TUI, MCP, and `talos-runtime` use the same profile
evaluation path.

## Context Compaction Pipeline

To handle long conversations, Talos uses a progressive compaction strategy. Layers are activated as context pressure increases:

1. **Pinned**: Critical system instructions and user-defined constraints that never expire.
2. **Fresh**: The most recent messages, kept in full detail.
3. **Summarized**: Mid-term history reduced to high-level summaries via LLM call.
4. **Archived**: Older history excluded from the active prompt.

> **Future consideration**: If RAG-based retrieval of older context becomes necessary (no reference project currently uses this), it would be implemented as an extension rather than a core compaction layer.

## Storage Architecture

Talos uses a progressive storage strategy (ADR-002). The phase headings below are a **historical
rollout record**, retained to preserve the original iteration evidence; they are not an exhaustive
current inventory. The current boundary is JSONL for transcript source-of-truth data, SQLite
indexes/structured stores where queryability requires them, and TOML/JSON/Markdown for user-editable
configuration and instructions.

### Phase 1: Pure Files (I001–I005)

No database dependency. All data is file-based:

*   **Sessions**: JSONL append-only logs (`~/.talos/sessions/<project>/<id>.jsonl`). One JSON object
    per line. Crash-safe (only the last line can be corrupted).
*   **Configuration**: TOML files with `${ENV_VAR}` substitution and layered merging
    (`~/.talos/config.toml` + `.talos/config.toml`).
*   **Permission rules**: Inline in configuration (no separate rule files yet).

### Phase 2: SQLite Introduction (I006)

Session metadata indexing and full-text search require a database:

*   **SQLite** (via `rusqlite/bundled`, ADR-008): `~/.talos/sessions/index.db`.
*   **Session messages** remain as JSONL files (source of truth). SQLite stores metadata only.
*   **FTS5** virtual table for full-text search across session content.
*   Storage is implemented directly with rusqlite calls; trait extraction is deferred until a
    second storage engine is real.

### Phase 3: SQLite Extension (I008)

Evolution engine requires structured queries for observations and patterns:

*   Bundled SQLite is also used by `talos-evolution` for `observations`, `patterns`, and
    `pattern_conflicts` tables.
*   Patterns include cognitive feedback fields: confidence, evidence counts, time decay (ADR-001).
*   Evolution storage is implemented directly with rusqlite calls under the same ADR-008 exception.

### Current File-Based Boundaries

These user-editable domains currently remain file-based:

*   **Configuration**: `~/.talos/config.toml`, with environment substitution and the separate
    `~/.talos/credentials.toml` snapshot merged when present.
*   **Skills**: Markdown files with YAML frontmatter under workspace/global `.talos/skills/` roots
    and the optional shared `~/.agents/skills/` root.
*   **Permission rules**: JSON configuration files (`.talos/permissions.json` in the project and
    `~/.talos/permissions.json` when present).
*   **Agent context**: Markdown files (`AGENTS.md` at project root and `~/.talos/AGENTS.md`).

### Current SQLite Boundaries

SQLite is used directly via `rusqlite/bundled` in `talos-session`, `talos-evolution`,
`talos-memory`, and `talos-exploration`; `talos-models` also contains a non-runtime catalog store.
No trait abstraction exists until a concrete second implementation is production-ready (YAGNI —
trait extraction happens when a real migration need exists).

`rusqlite/bundled` is an explicit ADR-008 exception to the general no-C/C++-bindings rule. SQLite is
compiled into the Talos binary, so users do not need a system SQLite installation. The final binary
is still platform-linked (for example, macOS system frameworks), so this is "SQLite self-contained",
not "fully static binary".

## Plugin System

The current extension boundary is explicit and layered, as constrained by ADR-027:

1. **Hooks**: `talos-plugin` provides `HookRegistry`, `HookHandler`, and built-in handlers for
   lifecycle events. This is an in-process product capability, not a dynamic library loader.
2. **WASM packages**: the CLI loads local packages selected by explicit paths, validates a
   `PluginManifest` whose executable carrier is currently `wasm`, and adapts read-only WASM tools
   through the existing `AgentTool`/`ToolRegistry` and permission path. Traps, timeouts, malformed
   modules, and bounded-output failures are reported as tool/plugin errors.
3. **MCP**: process-isolated external tools and resources use the separate `talos-mcp` client/server
   boundary and session-scoped startup described above.

Native `.so`, `.dll`, or `.dylib` loading, remote package installation, automatic plugin discovery,
and alternative carriers such as Lua are not current capabilities. Plugin manifests may describe
skills, tools, and hooks, but executable registration remains explicit at the product composition
root.

## Channel Topology Audit (ARCH-032, 2026-07-09)

An audit of all producer/consumer channels across the workspace verified continued compliance
with ADR-006 (no global event bus, no uncontrolled broadcast, no multi-consumer side channel).
The audit covered all src/ files in `talos-agent`, `talos-cli`, `talos-conversation`,
`talos-core`, `talos-evolution`, `talos-mcp`, `talos-memory`, `talos-permission`, `talos-plugin`,
`talos-runtime`, `talos-session`, and `talos-tui`.

### Channel Classification

Every channel in the workspace falls into one of these categories:

| Category | Pattern | ADR-006 Status | Count |
|---|---|---|---|
| SQ/EQ session seam | `mpsc::channel<SessionOp>` (bounded 512) + `mpsc::unbounded_channel<SessionEvent>` | Adopted (A+B) | 2 |
| Per-turn agent event channel | `mpsc::unbounded_channel<AgentEvent>` scoped to one turn | Compliant (A) | 1 |
| Per-turn result oneshot | `oneshot::channel<TurnRecord>` scoped to one turn | Compliant | 1 |
| L1 UI event loop | `mpsc::unbounded_channel<AppEvent>` single consumer | Adopted (A) | 1 |
| Conversation bridge | `mpsc::unbounded_channel<UiOutput/UserInput/AgentEvent>` | Compliant (A) | 5 |
| Session lifecycle | `mpsc::unbounded_channel<SessionLifecycleRequest>` | Compliant (A) | 1 |
| Watch state distribution | `watch::channel<Session/Sender<SessionOp>/ModelInfo>` | Compliant — state cache, not event broadcast | 3 |
| MCP request/response | `oneshot::channel` for JSON-RPC correlation | Compliant — bounded request/response | 5 |
| WASM watchdog | `std::sync::mpsc::channel<()>` scoped to one `execute_inner` call | Compliant — single-consumer, function-local | 1 |
| Per-turn stream | `mpsc::unbounded_channel<String>` for text stream to TUI | Compliant (A) | 1 |
| Sync crates (no channels) | `talos-session`, `talos-permission`, `talos-memory` | N/A — pure sync | 0 |
| Dashboard (no channels) | Pre-computed `DashboardSnapshot` served via HTTP | N/A — no channels | 0 |

**Zero `broadcast::channel` usages across the entire workspace.**

### Watch Channel Analysis

Three `watch::channel` instances exist in `run_tui_mode` (`talos-cli/src/mode_runners.rs:702-704`).
They distribute **state snapshots** (current `Session`, current `mpsc::Sender<SessionOp>`, current
`ModelInfo`) across session switches — not event streams. Each has exactly one sender (the session
handler task) and typed, named receivers. This is the "deterministic fan-out from a single consumer"
pattern that ADR-006 §73-75 explicitly endorses as the correct alternative to pub/sub.

### Hook Dispatch (Not a Channel)

The hook system (`HookRegistry` in `talos-plugin/src/registry.rs`) uses **sequential trait-method
dispatch**, not channels. `HookRegistry::dispatch()` iterates `HashMap<HookEventKind,
Vec<Arc<dyn HookHandler>>>` and calls `handler.on_event()` synchronously. `EvolutionHookHandler`
and `LoggingHandler` are registered per-Agent — there is no global event bus and no per-path
evolution observer channel (ADR-006 §117-129).

### Channel Topology Diagram

```text
┌─────────────┐     SessionOp (SQ)     ┌──────────────────┐
│  CLI / TUI  │ ──────────────────────► │ AppServerSession │
│  (producer) │                         │     (actor)      │
└─────────────┘                         └────────┬─────────┘
       ▲                                         │
       │                                         │ SessionEvent (EQ)
       │                                         ▼
       │                                  ┌──────────────┐
       │         UiOutput                 │   Bridge     │
       │ ◄─────────────────────────────── │ (conversation│
       │                                  │    loop)     │
       │         UserInput                └──────┬───────┘
       └──────────────────────────────►          │
                                                 │ SessionLifecycleRequest
                                                 ▼
                                          ┌──────────────┐
                                          │ Mode Runner  │
                                          │ (session mgmt)│
                                          └──────────────┘
```

All arrows are single-consumer mpsc. Watch channels (state distribution) are not shown — they
carry last-value state snapshots, not events.

### Compliance Verdict

**Fully compliant with ADR-006.** No deviations found. No remediation required. All channels
trace to concrete single-producer/single-consumer pairs or bounded request/response patterns. The
hook system uses trait-method dispatch (not channels), with per-Agent `HookRegistry` instances.

### Semantic Follow-Up (ARCH-033 / I115, 2026-07-11)

The topology verdict above remains historically accurate for ADR-006's narrow question: there was
no global broadcast bus and every channel had an explicit consumer. Its final “no remediation” claim
did not cover semantic ordering or state ownership. A follow-up audit found that nested
`StreamMessage` receivers created a second ordering domain, provider-response `TurnEnd` competed
with session `TurnCompleted`, CLI modes persisted/reconstructed turns differently, and RPC bypassed
the session seam.

I115 corrects those findings under ADR-039:

- canonical EQ events are `TurnEvent { session_id, turn_id, sequence, payload }`;
- one FIFO `UiOutput::Content` path carries live text with tool/reasoning/status projections;
- steering drains only on authoritative session completion;
- `AppServerSession` owns successful turn-message persistence;
- TUI, interactive, inline, print, embedded runtime, and RPC use the session protocol.
