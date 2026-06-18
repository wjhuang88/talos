# Architecture Audit 2026-06-18

## Scope

This report closes ARCH-002 Phase 1 and Phase 2: documentation validation plus architecture
audit. It does not implement the larger refactors; those are registered as follow-up backlog
stories with separate acceptance criteria.

## Evidence

- `cargo metadata --no-deps --format-version 1`
- `cargo tree --workspace --depth 1 --prefix none`
- `find crates -name '*.rs' -print | xargs wc -l | sort -nr | head -40`
- `rg` audits for `talos_agent`, `rmcp`, `rusqlite`, `talos_config`, duplicate tool result
  and tool definition types
- Manual reads of `docs/reference/ARCHITECTURE.md`, `docs/iterations/README.md`,
  `docs/BOARD.md`, `docs/backlog/PRODUCT-BACKLOG.md`, and ARCH-002

## Confirmed Workspace Shape

The workspace currently has 16 crates:

| Crate | Current architectural role |
| --- | --- |
| `talos-core` | Foundation traits and protocol/message types. |
| `talos-config` | User config schema, validation, env substitution, opencode import. |
| `talos-provider` | Anthropic, OpenAI-compatible, and mock provider adapters. |
| `talos-agent` | Turn loop, prompt assembly, tool execution, compaction, session integration. |
| `talos-permission` | Permission rules and approval decisions. |
| `talos-plugin` | Hook/event plugin foundation. |
| `talos-sandbox` | Process sandbox and hardening boundary. |
| `talos-skill` | Skill discovery and loading. |
| `talos-tools` | Built-in tools including file, search, symbol, tree, and Git tools. |
| `talos-session` | Session persistence and workspace topology. |
| `talos-mcp` | MCP client/server integration. |
| `talos-rpc` | JSON-RPC API layer. |
| `talos-cli` | Composition root, CLI modes, TUI bridge, registry construction. |
| `talos-conversation` | TUI-facing conversation state and typed UI events. |
| `talos-evolution` | Runtime learning hook and SQLite-backed evolution store. |
| `talos-tui` | Inline terminal UI, markdown rendering, styled scrollback. |

## Phase 1 Documentation Validation

| Area | Result | Fix / residual |
| --- | --- | --- |
| ADR references | ADR files referenced by active docs exist through ADR-021. | No missing ADR file found in active references checked. |
| Architecture reference | Crate list is current, but UI event table and styled scrollback description were stale. | Updated in `docs/reference/ARCHITECTURE.md`. |
| AGENTS rules | Hard constraints match current project posture. | No change required. |
| Backlog statuses | ARCH-002 row was still planned; I026/TOOL/Git statuses had already been mostly synchronized. | ARCH-002 moved to complete; follow-up stories added. |
| Iteration index | `docs/iterations/README.md` omitted I025 and I026. | Added I025/I026 rows. |
| BOARD | ARCH-002 appeared as Next even though this audit is now complete. | Board updated after owner docs. |
| Manifest | `next_actions` still referenced already-completed TOOL-002/CODE-002 work. | Manifest refreshed. |
| README | Current feature summary already reflects I026 tool/prompt/Git work. | No change required in this slice. |

## Phase 2 Coupling Audit

### P0: Boundary issues that should be fixed before more API surface is added

| Issue | Evidence | Proposed fix | Effort |
| --- | --- | --- | --- |
| `talos-mcp` has a dead dependency on `talos-agent` | `crates/talos-mcp/Cargo.toml` declares `talos-agent`; `rg "use talos_agent" crates/talos-mcp/src` finds no source usage. | Remove dependency and verify `cargo check -p talos-mcp`. | XS |
| `talos-rpc` is coupled to concrete `Agent` | `crates/talos-rpc/src/methods/mod.rs` exposes `MethodContext { agent: Arc<Agent> }`. | Introduce an RPC-facing runtime trait or narrow adapter and move concrete `Agent` wiring to `talos-cli`. | M |
| Duplicate `ToolResult` names in `talos-core` | `talos-core/src/tool.rs` has execution `ToolResult`; `talos-core/src/message.rs` has message `ToolResult` with `tool_use_id`. | Rename message-layer type to `MessageToolResult` and update imports. | S |

Owner story: `docs/backlog/active/ARCH-003-crate-boundary-cleanup.md`.

### P1: Anti-corruption layer gaps

| Issue | Evidence | Proposed fix | Effort |
| --- | --- | --- | --- |
| `rmcp` types leak through `talos-mcp` public surface | `rmcp::model::Tool`, `rmcp::ErrorData`, request context and server handler types appear in public modules. | Define Talos-owned MCP descriptor/error DTOs at the facade boundary; keep `rmcp` inside transport/server adapter modules. | L |
| SQLite error types leak into store APIs | `talos-evolution::EvolutionError::Store(#[from] rusqlite::Error)` and `talos-session::SqliteError(#[from] rusqlite::Error)`. | Add crate-owned store error enums; preserve source with `#[source]` where useful. | M |
| `talos-mcp` imports `talos-config` directly | `client/manager.rs` takes `&McpConfig` / `&McpServerConfig`. | Move to `McpClientConfig` / `McpServerLaunchConfig` in `talos-mcp`; convert in CLI composition root. | M |
| `talos-agent::caching::ToolDefinition` duplicates `talos-core::provider::ToolDefinition` | `rg "struct ToolDefinition"` finds both. | Reuse `talos_core::provider::ToolDefinition` or rename if semantics have intentionally diverged. | S |

Owner story: `docs/backlog/active/ARCH-004-anti-corruption-layers.md`.

### P2: God module decomposition candidates

Current largest files:

| File | Lines | Proposed split |
| --- | ---: | --- |
| `crates/talos-agent/src/lib.rs` | 2833 | `turn_loop.rs`, `tool_execution.rs`, `event_flow.rs`, move tests to focused modules. |
| `crates/talos-tui/src/app.rs` | 2516 | `scrollback.rs`, `tool_display.rs`, `markdown_render.rs`, `event_loop.rs`. |
| `crates/talos-tools/src/lib.rs` | 2484 | One module per tool family: `bash`, `file`, `search`, `diff_stat`, plus existing `git`, `symbol`, `tree`. |
| `crates/talos-cli/src/main.rs` | 2236 | `registry.rs`, `provider_setup.rs`, `session_setup.rs`, `tui_bridge.rs`. |
| `crates/talos-session/src/lib.rs` | 1736 | Keep public facade; move tests and topology helpers out. |
| `crates/talos-skill/src/lib.rs` | 1483 | `parser.rs`, `manager.rs`, `loader.rs`. |

Owner story: `docs/backlog/active/ARCH-005-god-module-decomposition.md`.

### P2: Prompt/cache stability residuals

I026 implemented template rendering and Anthropic cache-control emission. Remaining architecture
questions are session-stability and observability:

| Issue | Evidence | Proposed fix | Effort |
| --- | --- | --- | --- |
| System prompt prefix is not explicitly frozen per session in a reusable session object | Prompt builder can assemble stable sections, but the session-level invariant is not documented as an executable boundary. | Add a prompt session snapshot / stable prompt object and verify no mid-session tool/skill/context mutation changes the prefix. | M |
| Cache hit/miss metadata is not exposed | Provider response metadata does not surface cache hit/miss observations. | Add provider metadata field and logs/metrics in OBS-001 or prompt-cache story. | M |
| MCP discovery can still alter tool set between sessions and may invalidate prompt cache | Tool discovery happens at startup, but cache behavior is not recorded as a stable contract. | Document and test startup-only discovery for TUI/CLI composition. | S |

Owner story: `docs/backlog/active/ARCH-006-prompt-cache-stability.md`.

## Target Architecture State

- `talos-core` remains the zero-internal-dependency foundation.
- `talos-cli` remains the primary composition root for config, concrete providers, concrete
  agent, MCP startup, TUI bridge, and RPC server wiring.
- API crates (`talos-rpc`, `talos-mcp`) depend on narrow Talos-owned DTOs and traits rather than
  concrete application/runtime types.
- External library types (`rmcp`, `rusqlite`) stay behind adapter/store modules and do not shape
  Talos public APIs.
- Tool execution result and message result types have distinct names and import paths.
- Large modules are split by responsibility without changing behavior in the same slice.
- Prompt/cache behavior is represented by a session-stable prompt snapshot and provider-specific
  emission layer.

## Prioritized Follow-up Backlog

| Priority | Story | Purpose |
| --- | --- | --- |
| P1 | ARCH-003 Crate Boundary Cleanup | Remove dead dependencies and concrete RPC runtime coupling before remote/API growth. |
| P2 | ARCH-004 Anti-Corruption Layers | Stop external library/config types from defining Talos crate APIs. |
| P2 | ARCH-006 Prompt Cache Stability | Finish the session-stable prompt/cache contract after I026 Phase A. |
| P3 | ARCH-005 God Module Decomposition | Reduce maintenance risk in the largest modules after boundary APIs are stable. |

## Residual Risk

No functional code was refactored in this audit slice. The main residual risk is that large-file
decomposition will be harder if new features continue to land in `talos-agent/src/lib.rs`,
`talos-tui/src/app.rs`, `talos-tools/src/lib.rs`, and `talos-cli/src/main.rs` before ARCH-003/004
stabilize the boundaries.
