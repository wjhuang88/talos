# Talos Product Backlog

Stories are organized by iteration. Each iteration is a vertical slice delivering runnable
functionality. Story format: `#I{iteration}-S{story}`.

## I001: "Hello Agent" (MVP)

**Delivers**: `talos "hello" -p` produces an LLM response.

### #I001-S1: Initialize Cargo workspace

**Description**: Create root `Cargo.toml` workspace with 5 crate skeletons: `talos-core`, `talos-config`, `talos-provider`, `talos-agent`, `talos-cli`. Each crate has a minimal `lib.rs` or `main.rs` that compiles.

**Acceptance Criteria**:
- [ ] `cargo check --workspace` exits 0
- [ ] `cargo build -p talos-cli` produces a binary
- [ ] Binary runs and prints version/help text
- [ ] Workspace uses Rust edition 2024

**Depends on**: None
**Estimate**: S

### #I001-S2: Core message types and event protocol

**Description**: Define the shared vocabulary in `talos-core`: `Message` enum (User/Assistant/ToolResult), `AgentEvent` enum (turn_start/text_delta/tool_call/turn_end/error), `StopReason`, `Usage` stats. All types derive `serde::Serialize`, `serde::Deserialize`.

**Acceptance Criteria**:
- [ ] All message types compile and are importable from other crates
- [ ] `serde` round-trip test passes: `Message` -> JSON -> `Message`
- [ ] No circular dependencies: `talos-core` depends on nothing
- [ ] Doc comments on all public types

**Depends on**: #I001-S1
**Estimate**: M

### #I001-S3: Minimal configuration system

**Description**: `talos-config` loads a minimal config: API key (from env var `ANTHROPIC_API_KEY` or `OPENAI_API_KEY`), model name, provider selection. Support `${ENV_VAR}` substitution in config values. Schema validation via `schemars`.

**Acceptance Criteria**:
- [ ] Config loads from `~/.talos/config.toml` with env var substitution
- [ ] Missing API key produces a clear error message (not a panic)
- [ ] Default config works without a config file (env-only mode)
- [ ] Config struct validated against JSON Schema at load time

**Depends on**: #I001-S1
**Estimate**: M

### #I001-S4: Anthropic streaming provider

**Description**: `talos-provider` implements streaming SSE connection to Anthropic Messages API. Define a `LanguageModel` trait with `stream()` method. Implement for Anthropic with proper error handling, retries on 429/5xx, and `CancellationToken` support.

**Acceptance Criteria**:
- [ ] `LanguageModel` trait defined in `talos-core`
- [ ] Anthropic provider streams text deltas via tokio channel
- [ ] API errors (401, 429, 500) produce typed errors, not panics
- [ ] Test with mock SSE server passes
- [ ] Request includes proper `cache_control` headers for prompt caching

**Depends on**: #I001-S2, #I001-S3
**Estimate**: L

### #I001-S5: Basic turn loop (no tools)

**Description**: `talos-agent` implements the simplest possible turn loop: build prompt -> call provider -> stream response -> return. Uses SQ/EQ pattern (bounded submission, unbounded event channels). No tool execution yet.

**Acceptance Criteria**:
- [ ] Agent receives user message, returns assistant response
- [ ] Events stream via tokio broadcast channel
- [ ] CancellationToken aborts mid-stream cleanly
- [ ] Unit test: mock provider -> agent returns expected response

**Depends on**: #I001-S4
**Estimate**: M

### #I001-S6: CLI print mode and stdin pipe

**Description**: `talos-cli` supports two modes: `talos "prompt" -p` (print and exit) and `echo "prompt" | talos -p` (stdin pipe). Streaming output to stdout. Exit code 0 on success, 1 on error. `--version` and `--help` flags.

**Acceptance Criteria**:
- [ ] `talos "What is 2+2?" -p` streams response to stdout and exits
- [ ] `echo "hello" | talos -p` works
- [ ] `talos --version` prints version
- [ ] `talos --help` prints usage
- [ ] Missing API key prints actionable error message
- [ ] `cargo test -p talos-cli` passes

**Depends on**: #I001-S5
**Estimate**: M

---

## I002: "Tool User"

**Delivers**: Agent can execute file and shell operations.

### #I002-S1: AgentTool trait and ToolRegistry

**Description**: Define `AgentTool` trait in `talos-core` with: `name()`, `description()`, `parameters()` (JSON Schema), `execute()` (async), `is_read_only()`. Implement `ToolRegistry` with `register()`, `get()`, `list()`.

**Acceptance Criteria**:
- [ ] `AgentTool` trait defined with all required methods
- [ ] `ToolRegistry` supports dynamic registration
- [ ] Tool parameters validated against JSON Schema before execution
- [ ] Doc comments on trait and all methods

**Depends on**: #I001-S2
**Estimate**: M

### #I002-S2: Bash tool

**Description**: Implement shell command execution tool. Runs commands via `tokio::process::Command`, captures stdout/stderr, enforces timeout (default 120s). Returns structured output.

**Acceptance Criteria**:
- [ ] `bash("ls -la")` returns stdout/stderr/exit-code
- [ ] Commands timeout after configurable duration
- [ ] Shell metacharacters work: pipes, redirects, globs
- [ ] Working directory defaults to project root
- [ ] Error output clearly marked vs normal output

**Depends on**: #I002-S1
**Estimate**: M

### #I002-S3: File read/write/edit tools

**Description**: Implement three file tools. `read` reads file content with line range support. `write` creates/overwrites files. `edit` applies string replacements. All operations are relative to workspace root.

**Acceptance Criteria**:
- [ ] `read("src/main.rs")` returns file content with line numbers
- [ ] `read("src/main.rs", 10, 20)` returns lines 10-20
- [ ] `write("new.txt", "content")` creates file
- [ ] `edit("file.txt", "old", "new")` replaces first occurrence
- [ ] Paths outside workspace root are rejected
- [ ] Binary files handled gracefully (error, not crash)

**Depends on**: #I002-S1
**Estimate**: M

### #I002-S4: Turn loop with tool execution

**Description**: Extend the agent turn loop to handle tool calls from LLM responses. When the model emits `tool_use`, execute the tool and feed results back. Support concurrent read-only tools (up to 10) and serial write tools. Loop until model emits no tool calls.

**Acceptance Criteria**:
- [ ] Model can call tools, results feed back, loop continues
- [ ] Read-only tools run concurrently (batch execution)
- [ ] Write tools run serially (one at a time)
- [ ] Turn terminates when model produces no tool calls
- [ ] Turn budget enforcement (max 50 tool calls per turn)
- [ ] Doom loop detection: same tool+args 3 times triggers warning

**Depends on**: #I001-S5, #I002-S2, #I002-S3
**Estimate**: L

### #I002-S5: JSONL session logging

**Description**: `talos-session` appends every message and event to a JSONL file. Sessions stored in `~/.talos/sessions/` organized by working directory. Simple append-only, no branching yet.

**Acceptance Criteria**:
- [ ] Every user message, assistant response, and tool result logged
- [ ] Session file is valid JSONL (one JSON object per line)
- [ ] New session created automatically on start
- [ ] Session ID is a UUID

**Depends on**: #I001-S2
**Estimate**: S

### #I002-S6: Interactive readline loop

**Description**: `talos-cli` gains interactive mode (no TUI yet, just readline). User types a prompt, agent responds, repeat. `Ctrl+C` cancels current turn, double `Ctrl+C` exits.

**Acceptance Criteria**:
- [ ] `talos` (no args) starts interactive loop
- [ ] User input -> agent response -> prompt again
- [ ] `Ctrl+C` cancels current agent turn
- [ ] Double `Ctrl+C` exits the program
- [ ] Streaming output visible during response

**Depends on**: #I002-S4
**Estimate**: M

---

## I003: "Safe Agent"

**Delivers**: Dangerous operations are caught and contained.

### #I003-S1: Permission rules engine

**Description**: `talos-permission` evaluates tool calls against rules. Rules loaded from config: allow/deny/ask per tool name and path pattern. Wildcard matching with glob patterns. Default: ask for write operations, allow read operations.

**Acceptance Criteria**:
- [ ] Rules evaluated per tool call before execution
- [ ] `allow` -> execute immediately
- [ ] `deny` -> rejected with clear error message
- [ ] `ask` -> prompt user for approval
- [ ] Glob patterns match paths correctly (`src/**/*.rs`)
- [ ] Default ruleset: read=allow, write=ask, bash=ask

**Depends on**: #I002-S1
**Estimate**: M

### #I003-S2: Interactive approval prompt

**Description**: When a tool call needs approval, present it to the user. Show: tool name, arguments, risk level. User can approve once, approve always (add rule), or deny.

**Acceptance Criteria**:
- [ ] Approval prompt shows tool name and truncated arguments
- [ ] User options: y (approve once), a (always approve), n (deny)
- [ ] "Always approve" persists rule to config
- [ ] Non-interactive mode (`-p`) defaults to deny for ask-rules

**Depends on**: #I003-S1
**Estimate**: S

### #I003-S3: Bubblewrap sandbox (Linux)

**Description**: `talos-sandbox` runs bash commands in a Bubblewrap sandbox. Read-only filesystem except workspace root. Network namespace isolation. User/PID namespace isolation.

**Acceptance Criteria**:
- [ ] Commands run with restricted filesystem (writable only in workspace)
- [ ] Network access blocked by default (configurable)
- [ ] Cannot escape via symlink, `/proc`, `/sys`, or `../`
- [ ] `.git` directory always read-only
- [ ] Command fails gracefully if bwrap not installed (fallback to no sandbox)

**Depends on**: #I002-S2
**Estimate**: L

### #I003-S4: sandbox-exec (macOS)

**Description**: Implement macOS sandbox using `sandbox-exec` with Seatbelt profile. Similar restrictions to Bubblewrap: filesystem write restricted to workspace, network restricted.

**Acceptance Criteria**:
- [ ] Seatbelt profile generated dynamically based on workspace path
- [ ] Write access limited to workspace and temp directories
- [ ] `.git` directory always read-only
- [ ] Graceful fallback if sandbox-exec unavailable

**Depends on**: #I002-S2
**Estimate**: L

### #I003-S5: Process hardening basics

**Description**: Apply security measures to the agent process itself. Sanitize environment variables (remove `LD_PRELOAD`, `DYLD_*`). Set resource limits (max CPU, memory). Prevent core dumps.

**Acceptance Criteria**:
- [ ] Dangerous env vars stripped from child processes
- [ ] Resource limits configurable (default: reasonable limits)
- [ ] Core dumps disabled for sandboxed processes

**Depends on**: #I003-S3 or #I003-S4
**Estimate**: M

### #I003-S6: Tool execution pipeline integration

**Description**: Wire permission engine and sandbox into the tool execution flow. Pipeline: permission check -> sandbox selection -> execute -> retry on denial. Applies to all tools, not just bash.

**Acceptance Criteria**:
- [ ] Every tool call goes through permission -> sandbox -> execute pipeline
- [ ] File tools respect filesystem restrictions
- [ ] Bash tools run in sandbox when available
- [ ] Failed sandbox execution can retry with elevated permissions (user approval)
- [ ] Integration test: agent attempts dangerous operation, pipeline blocks it

**Depends on**: #I003-S1, #I003-S3, #I003-S4
**Estimate**: M

---

## I004: "Smart Agent"

**Delivers**: Long conversations work without context overflow.

### #I004-S1: Token estimation

**Description**: Estimate token count for messages before sending to LLM. Character-based approximation (4 chars ~ 1 token) with provider-specific corrections. Track cumulative usage per session.

**Acceptance Criteria**:
- [ ] Token estimate within 20% of actual for English text
- [ ] Usage tracked per turn (input, output, cache_read, cache_write)
- [ ] Cost estimation based on model pricing

**Depends on**: #I001-S5
**Estimate**: S

### #I004-S2: Context compaction pipeline

**Description**: 5-layer compaction triggered when context nears model limit. Layer 1: budget (cap tool result sizes). Layer 2: trim (remove old tool results). Layer 3: microcompact (strip completed tool results by ID). Layer 4: collapse (summarize old turns). Layer 5: autocompact (LLM-based summarization).

**Acceptance Criteria**:
- [ ] Compaction triggers automatically at 80% context usage
- [ ] Manual `/compact` command available in interactive mode
- [ ] Compaction preserves recent turns (last 10) verbatim
- [ ] Summarization uses a separate compact LLM call
- [ ] After compaction, conversation continues seamlessly
- [ ] No infinite compact-fail-retry loops (circuit breaker: 3 failures -> stop)

**Depends on**: #I004-S1, #I001-S4
**Estimate**: XL

### #I004-S3: JSONL tree-branching sessions

**Description**: Extend session storage with parent-child relationships. Each entry has `id` and `parent_id`. Support forking from any point. Session resume via `-c` (continue last) and `-r` (select from history). Branching is implemented in JSONL only (no SQLite dependency yet).

**Acceptance Criteria**:
- [ ] `/fork` creates branch from current position
- [ ] `talos -c` resumes most recent session
- [ ] `talos -r` lists sessions by scanning JSONL directory
- [ ] Branch history preserved in single JSONL file
- [ ] Session metadata includes: timestamp, model, token count, working directory

**Depends on**: #I002-S5
**Estimate**: M

### #I004-S4: Context file loading (AGENTS.md)

**Description**: Load `AGENTS.md` files from working directory and parent directories. Concatenate all found files into system prompt context. Also load `~/.talos/AGENTS.md` as global context.

**Acceptance Criteria**:
- [ ] `AGENTS.md` loaded from cwd and all parent dirs up to git root
- [ ] Global `~/.talos/AGENTS.md` loaded first
- [ ] Content injected into system prompt
- [ ] `--no-context` flag disables loading
- [ ] Total context file size capped at 20,000 chars (head/tail truncation)

**Depends on**: #I001-S5
**Estimate**: S

### #I004-S5: Prompt caching strategy

**Description**: Structure the system prompt for provider-side caching. Stable prefix (identity + tools + context files) kept constant across turns. Only conversation history grows. Add `cache_control` markers for Anthropic.

**Acceptance Criteria**:
- [ ] System prompt structure: static prefix + dynamic conversation
- [ ] Anthropic `cache_control` breakpoints set correctly
- [ ] Cache hit rate tracked and reported in usage stats
- [ ] Tool definitions maintain stable ordering

**Depends on**: #I004-S1
**Estimate**: M

### #I004-S6: SQLite session index with FTS5

**Description**: Introduce `rusqlite` (bundled) as the first database dependency. Create `~/.talos/index.db` with session metadata table and FTS5 virtual table. JSONL files remain the source of truth; SQLite serves as a metadata index and search engine. Storage operations use rusqlite directly — no trait abstraction until a concrete second engine exists (ADR-002).

**Acceptance Criteria**:
- [ ] `rusqlite` with `bundled` feature compiles and links successfully
- [ ] SQLite module in `talos-session` with direct rusqlite calls for: `create_session`, `append_message`, `get_session`, `list_sessions`, `search_sessions`
- [ ] SQLite stores session metadata (id, project, timestamps, model, turn count, token total)
- [ ] FTS5 virtual table indexes session content for full-text search
- [ ] `talos -r` uses SQLite metadata for fast session listing (no directory scan)
- [ ] `talos --search <query>` uses FTS5 for full-text session search
- [ ] JSONL files remain the source of truth; SQLite is index only
- [ ] Migration: existing JSONL sessions are indexed on first run

**Depends on**: #I002-S5, #I004-S3
**Estimate**: L

---

## I005: "Skilled Agent"

**Delivers**: Skills system and multi-provider support.

### #I005-S1: SKILL.md parser and loader

**Description**: `talos-skill` discovers and parses SKILL.md files. YAML frontmatter (name, description, trigger conditions) + Markdown body (instructions). Discovery from `.talos/skills/`, `~/.talos/skills/`, and parent directories.

**Acceptance Criteria**:
- [ ] SKILL.md parsed with frontmatter + body
- [ ] Discovery from 3 locations: project, user home, parent dirs
- [ ] Invalid SKILL.md files produce warnings, not crashes
- [ ] Skill index (name + description only) injected into system prompt

**Depends on**: #I001-S2
**Estimate**: M

### #I005-S2: Progressive disclosure (3 levels)

**Description**: Skills load in 3 levels. Level 0: name + description in system prompt (~50 tokens each). Level 1: full SKILL.md content loaded on demand when task matches. Level 2: specific reference files from skill.

**Acceptance Criteria**:
- [ ] Level 0 always loaded (skill index in system prompt)
- [ ] Level 1 loaded when agent's task matches skill trigger
- [ ] Level 2 loaded when agent needs specific reference files
- [ ] Total skill index stays under 3000 tokens for 20 skills

**Depends on**: #I005-S1
**Estimate**: M

### #I005-S3: OpenAI provider

**Description**: Add OpenAI as a second provider. Streaming via SSE. Chat Completions API format. Support `OPENAI_API_KEY` and `OPENAI_BASE_URL` for compatible providers.

**Acceptance Criteria**:
- [ ] `talos --provider openai --model gpt-4o` works
- [ ] Streaming text deltas via SSE
- [ ] Tool calls in OpenAI format translated to internal format
- [ ] `OPENAI_BASE_URL` override works for compatible APIs
- [ ] Model switching in interactive mode via `/model`

**Depends on**: #I001-S4
**Estimate**: M

### #I005-S4: Evolution engine with cognitive feedback

**Description**: Implement the `talos-evolution` crate with the 4-phase learning loop (ADR-001): Observe → Accumulate → Extract → Apply. The exact signal taxonomy, confidence formulas, decay rates, and conflict resolution strategies will be designed at the start of I005 based on real usage data from I001-I004. Storage uses direct rusqlite calls extending the database from I004-S6. Skill creation from experience is one output channel — when a pattern stabilizes, it can be materialized as a SKILL.md.

**Acceptance Criteria**:
- [ ] `TurnObserver` captures structured observations per turn (tool calls, duration, outcome, signals)
- [ ] Signal taxonomy designed based on I001-I004 usage patterns (per ADR-001 cognitive feedback principles)
- [ ] `PatternExtractor` extracts preferences, project patterns, and error-avoidance rules
- [ ] Contradiction detection: new patterns are checked against existing ones before storage
- [ ] Patterns carry confidence scores with evidence backing and time decay
- [ ] Extraction triggers include signal-driven events, not just session boundaries
- [ ] SQLite tables: observations, patterns, pattern_conflicts
- [ ] `BehaviorAdapter` injects high-confidence patterns into system prompt
- [ ] Evolution data inspectable via `/learned` command
- [ ] User can disable evolution via config

**Depends on**: #I005-S1, #I002-S4, #I004-S6
**Estimate**: XL

### #I005-S5: System prompt assembly

**Description**: Assemble the full system prompt from: identity, tool descriptions, skill index, context files (AGENTS.md), and user preferences. Structure for optimal caching.

**Acceptance Criteria**:
- [ ] System prompt assembled from 5 sources with clear boundaries
- [ ] Order optimized for prompt caching (stable content first)
- [ ] Custom system prompt via `--system-prompt` flag
- [ ] Append via `--append-system-prompt` flag
- [ ] Total system prompt size logged for debugging

**Depends on**: #I004-S4, #I005-S1
**Estimate**: M

---

## I006: "Extensible Agent"

**Delivers**: WASM plugins and MCP integration.

### #I006-S1: WASM plugin runtime

**Description**: `talos-plugin` integrates wasmtime for sandboxed plugin execution. Plugins are WASM modules that export hook handlers. Host provides API for tool registration, event subscription, and config access.

**Acceptance Criteria**:
- [ ] WASM module loaded and executed in sandboxed runtime
- [ ] Plugin can register custom tools
- [ ] Plugin can subscribe to events (tool_call, message, etc.)
- [ ] Plugin cannot access host filesystem or network directly
- [ ] Plugin error does not crash the agent

**Depends on**: #I002-S1
**Estimate**: XL

### #I006-S2: Hook system (20+ extension points)

**Description**: Define hooks at key points in the agent lifecycle. Plugins register handlers. Hooks include: before_tool_call, after_tool_call, message_transform, system_prompt_transform, permission_check, session_start/end.

**Acceptance Criteria**:
- [ ] 20+ hook points defined and documented
- [ ] Plugins can register multiple handlers per hook
- [ ] Hook handlers run in registration order
- [ ] Hooks can modify or block operations
- [ ] Performance overhead < 1ms per hook invocation

**Depends on**: #I006-S1
**Estimate**: L

### #I006-S3: File-based plugin discovery

**Description**: Discover plugins from `.talos/plugins/*.wasm`, `~/.talos/plugins/*.wasm`. Auto-load on startup. Config `plugin` section for enabling/disabling.

**Acceptance Criteria**:
- [ ] `.wasm` files discovered and loaded from plugin directories
- [ ] Config can enable/disable specific plugins
- [ ] `--no-plugins` flag disables all plugins
- [ ] Plugin load errors produce warnings, not crashes

**Depends on**: #I006-S1, #I001-S3
**Estimate**: S

### #I006-S4: MCP client

**Description**: `talos-mcp` connects to external MCP servers. Discovers tools, resources, and prompts from servers. Converts MCP tool definitions to AgentTool implementations. Config via `mcp` section in config file.

**Acceptance Criteria**:
- [ ] Connect to MCP server via stdio or HTTP
- [ ] MCP tools available as AgentTool in the agent
- [ ] Tool results from MCP servers forwarded correctly
- [ ] Multiple MCP servers supported simultaneously
- [ ] Connection failures handled gracefully (retry + skip)

**Depends on**: #I002-S1
**Estimate**: L

### #I006-S5: MCP server

**Description**: Expose Talos tools as an MCP server. Other agents can connect and use Talos tools. Support stdio transport.

**Acceptance Criteria**:
- [ ] `talos --mcp-server` starts in MCP server mode
- [ ] All registered tools exposed via MCP protocol
- [ ] External MCP clients can call Talos tools
- [ ] Permission rules still enforced for external callers

**Depends on**: #I002-S1
**Estimate**: L

### #I006-S6: JSON-RPC server (stdio)

**Description**: `talos-rpc` implements JSON-RPC over stdio. Methods: session/start, session/list, turn/start, turn/interrupt, config/read. Enables IDE and tool integration.

**Acceptance Criteria**:
- [ ] `talos --mode rpc` starts JSON-RPC server on stdio
- [ ] Core methods work: start session, send prompt, receive response
- [ ] Streaming events delivered as JSON-RPC notifications
- [ ] Error responses follow JSON-RPC error format

**Depends on**: #I001-S5
**Estimate**: M

---

## I007: "Polished Agent"

**Delivers**: Full-featured, daily-usable coding agent with professional TUI.

### #I007-S0: TUI layout and interaction design

**Description**: Before implementing the TUI, design the complete layout, component hierarchy, interaction model, and keymap system. Reference Codex TUI architecture (80+ modules in `codex-rs/tui/src/`). Produce a design document covering: screen layout (chat viewport, bottom pane, status bar), HistoryCell types (message, exec, approval, patch, MCP), BottomPane view stack, slash command interface, approval overlay flow, keymap contexts, and `--no-alt-screen` inline mode. This design document will be the blueprint for all subsequent TUI stories.

**Acceptance Criteria**:
- [ ] Design document written at `docs/reference/TUI-DESIGN.md`
- [ ] Screen layout diagram with component boundaries
- [ ] HistoryCell type catalog with visual mockups (ASCII art)
- [ ] BottomPane view stack state machine
- [ ] Slash command catalog and filtering behavior
- [ ] Approval overlay interaction flow
- [ ] Keymap context hierarchy (App/Chat/Composer/Editor/Pager/List/Approval)
- [ ] Inline mode (`--no-alt-screen`) behavior specification
- [ ] Frame rate limiting strategy
- [ ] Markdown rendering approach

**Depends on**: #I002-S6 (interactive loop exists to understand UX needs)
**Estimate**: M

### #I007-S1: TUI with ratatui

**Description**: Full terminal UI with: chat viewport with HistoryCell rendering, bottom pane with ChatComposer (multiline input), status bar with model/tokens/cost/context usage, approval overlay for permission requests, diff display for file changes. Based on TUI-DESIGN.md from #I007-S0. Supports `--no-alt-screen` for inline mode preserving terminal scrollback. Frame rate limited rendering via FrameRequester.

**Acceptance Criteria**:
- [ ] `talos` launches full TUI by default
- [ ] Messages scroll with HistoryCell rendering (text, exec, approval, patch types)
- [ ] Status bar shows: model, tokens, cost, context usage
- [ ] `Ctrl+C` cancels current turn, double exits
- [ ] Approval overlay replaces editor when permission requested
- [ ] `--no-alt-screen` flag runs inline preserving scrollback
- [ ] Frame rate limiting prevents wasteful renders during streaming
- [ ] Works on macOS Terminal, iTerm2, Linux terminals
- [ ] Markdown rendering in assistant messages

**Depends on**: #I002-S6, #I007-S0
**Estimate**: XL

### #I007-S2: Steering and follow-up queues

**Description**: Two message queues for mid-run input. Steering: delivered after current tool batch, before next LLM call. Follow-up: delivered only when agent would stop. Both support one-at-a-time or all-at-once drain modes. ChatComposer enters "queue mode" when agent is running — Enter queues message instead of interrupting.

**Acceptance Criteria**:
- [ ] Enter while agent works -> queues steering message
- [ ] Alt+Enter -> queues follow-up message
- [ ] Escape -> cancels and restores queued messages to editor
- [ ] Drain mode configurable in settings
- [ ] ChatComposer shows queued message count indicator

**Depends on**: #I007-S1
**Estimate**: M

### #I007-S3: Interactive command system

**Description**: Slash commands in TUI: `/model` (switch), `/new` (new session), `/resume` (pick session), `/fork` (branch), `/compact` (manual compaction), `/help`, `/quit`, `/diff` (show git diff), `/status` (session config + token usage), `/vim` (toggle vim mode). Tab autocomplete for commands. Fuzzy filtering as you type.

**Acceptance Criteria**:
- [ ] `/model` opens model selector
- [ ] `/new` starts fresh session
- [ ] `/resume` lists recent sessions for selection
- [ ] `/compact` triggers manual compaction
- [ ] `/diff` shows git diff in TUI
- [ ] `/status` shows session config and token breakdown
- [ ] `/help` shows all commands
- [ ] Tab completes command names
- [ ] Fuzzy filter narrows command list as you type

**Depends on**: #I007-S1, #I004-S3
**Estimate**: M

### #I007-S4: Guardian AI sub-agent

**Description**: Auto-approve low-risk tool calls using a lightweight LLM call. Guardian reviews tool call + context and decides approve/deny. Circuit breaker: 3 consecutive denials blocks Guardian.

**Acceptance Criteria**:
- [ ] Guardian reviews tool calls when enabled in config
- [ ] Low-risk operations auto-approved without user prompt
- [ ] Guardian denial triggers user approval prompt
- [ ] Circuit breaker activates after 3 denials
- [ ] Guardian uses a cheaper/faster model than main agent

**Depends on**: #I003-S1, #I001-S4
**Estimate**: L

### #I007-S5: Headless and SDK modes

**Description**: Three execution modes. Interactive: full TUI. Headless (`talos exec`): autonomous execution for CI/automation, no TUI. SDK: Talos as a Rust library for embedding. All modes share the same core agent loop via AppServerSession abstraction (Codex pattern: TUI never calls agent loop directly).

**Acceptance Criteria**:
- [ ] `talos exec "run tests and fix failures" --max-turns 20` runs autonomously
- [ ] `talos exec` exits with code 0 on success, 1 on failure
- [ ] SDK: `AgentSession::new()` -> `session.prompt("hello")` works as library
- [ ] All modes share the same core agent loop
- [ ] Headless mode supports `--json` for machine-readable output

**Depends on**: #I007-S1
**Estimate**: L

### #I007-S6: Exec policy DSL rules

**Description**: Full DSL for command approval rules in `.talos/rules/*.rules`. Pattern matching on command name, arguments, paths. Support for trusted commands, forbidden patterns, and conditional rules.

**Acceptance Criteria**:
- [ ] Rule files loaded from `.talos/rules/` and `~/.talos/rules/`
- [ ] Rules match on command name, glob patterns, path prefixes
- [ ] `trusted` rules auto-approve, `forbidden` rules auto-deny
- [ ] Complex shell features (pipes, redirects) bypass rules and require approval
- [ ] Rules can reference environment variables

**Depends on**: #I003-S1
**Estimate**: M
