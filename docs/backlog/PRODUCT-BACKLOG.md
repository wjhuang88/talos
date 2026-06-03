# Talos Product Backlog

Stories are organized by iteration. Each iteration is a vertical slice delivering runnable
functionality. Story format: `#I{iteration}-S{story}`.

## I001: "Project Scaffold"

**Delivers**: Cargo workspace and core message types compile.

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

---

## I002: "Hello Agent" (MVP)

**Delivers**: `talos "hello" -p` produces an LLM response.

### #I002-S1: Minimal configuration system

**Description**: `talos-config` loads a minimal config: API key (from env var `ANTHROPIC_API_KEY` or `OPENAI_API_KEY`), model name, provider selection. Support `${ENV_VAR}` substitution in config values. Schema validation via `schemars`.

**Acceptance Criteria**:
- [ ] Config loads from `~/.talos/config.toml` with env var substitution
- [ ] Missing API key produces a clear error message (not a panic)
- [ ] Default config works without a config file (env-only mode)
- [ ] Config struct validated against JSON Schema at load time

**Depends on**: #I001-S1
**Estimate**: M

### #I002-S2: Anthropic streaming provider

**Description**: `talos-provider` implements streaming SSE connection to Anthropic Messages API. Define a `LanguageModel` trait with `stream()` method. Implement for Anthropic with proper error handling, retries on 429/5xx, and `CancellationToken` support.

**Acceptance Criteria**:
- [ ] `LanguageModel` trait defined in `talos-core`
- [ ] Anthropic provider streams text deltas via tokio channel
- [ ] API errors (401, 429, 500) produce typed errors, not panics
- [ ] Test with mock SSE server passes
- [ ] Request includes proper `cache_control` headers for prompt caching

**Depends on**: #I001-S2, #I002-S1
**Estimate**: L

### #I002-S3: Basic turn loop (no tools)

**Description**: `talos-agent` implements the simplest possible turn loop: build prompt -> call provider -> stream response -> return. Uses SQ/EQ pattern (bounded submission, unbounded event channels). No tool execution yet.

**Acceptance Criteria**:
- [ ] Agent receives user message, returns assistant response
- [ ] Events stream via tokio broadcast channel
- [ ] CancellationToken aborts mid-stream cleanly
- [ ] Unit test: mock provider -> agent returns expected response

**Depends on**: #I002-S2
**Estimate**: M

### #I002-S4: CLI print mode and stdin pipe

**Description**: `talos-cli` supports two modes: `talos "prompt" -p` (print and exit) and `echo "prompt" | talos -p` (stdin pipe). Streaming output to stdout. Exit code 0 on success, 1 on error. `--version` and `--help` flags.

**Acceptance Criteria**:
- [ ] `talos "What is 2+2?" -p` streams response to stdout and exits
- [ ] `echo "hello" | talos -p` works
- [ ] `talos --version` prints version
- [ ] `talos --help` prints usage
- [ ] Missing API key prints actionable error message
- [ ] `cargo test -p talos-cli` passes

**Depends on**: #I002-S3
**Estimate**: M

---

## I003: "Tool User"

**Delivers**: Agent can execute file and shell operations.

### #I003-S1: AgentTool trait and ToolRegistry

**Description**: Define `AgentTool` trait in `talos-core` with: `name()`, `description()`, `parameters()` (JSON Schema), `execute()` (async), `is_read_only()`. Implement `ToolRegistry` with `register()`, `get()`, `list()`.

**Acceptance Criteria**:
- [ ] `AgentTool` trait defined with all required methods
- [ ] `ToolRegistry` supports dynamic registration
- [ ] Tool parameters validated against JSON Schema before execution
- [ ] Doc comments on trait and all methods

**Depends on**: #I001-S2
**Estimate**: M

### #I003-S2: Bash tool

**Description**: Implement shell command execution tool. Runs commands via `tokio::process::Command`, captures stdout/stderr, enforces timeout (default 120s). Returns structured output.

**Acceptance Criteria**:
- [ ] `bash("ls -la")` returns stdout/stderr/exit-code
- [ ] Commands timeout after configurable duration
- [ ] Shell metacharacters work: pipes, redirects, globs
- [ ] Working directory defaults to project root
- [ ] Error output clearly marked vs normal output

**Depends on**: #I003-S1
**Estimate**: M

### #I003-S3: File read/write/edit tools

**Description**: Implement three file tools. `read` reads file content with line range support. `write` creates/overwrites files. `edit` applies string replacements. All operations are relative to workspace root.

**Acceptance Criteria**:
- [ ] `read("src/main.rs")` returns file content with line numbers
- [ ] `read("src/main.rs", 10, 20)` returns lines 10-20
- [ ] `write("new.txt", "content")` creates file
- [ ] `edit("file.txt", "old", "new")` replaces first occurrence
- [ ] Paths outside workspace root are rejected
- [ ] Binary files handled gracefully (error, not crash)

**Depends on**: #I003-S1
**Estimate**: M

### #I003-S4: Turn loop with tool execution

**Description**: Extend the agent turn loop to handle tool calls from LLM responses. When the model emits `tool_use`, execute the tool and feed results back. Support concurrent read-only tools (up to 10) and serial write tools. Loop until model emits no tool calls.

**Acceptance Criteria**:
- [ ] Model can call tools, results feed back, loop continues
- [ ] Read-only tools run concurrently (batch execution)
- [ ] Write tools run serially (one at a time)
- [ ] Turn terminates when model produces no tool calls
- [ ] Turn budget enforcement (max 50 tool calls per turn)
- [ ] Doom loop detection: same tool+args 3 times triggers warning

**Depends on**: #I002-S3, #I003-S2, #I003-S3
**Estimate**: L

### #I003-S5: JSONL session logging

**Description**: `talos-session` appends every message and event to a JSONL file. Sessions stored in `~/.talos/sessions/` organized by working directory. Simple append-only, no branching yet.

**Acceptance Criteria**:
- [ ] Every user message, assistant response, and tool result logged
- [ ] Session file is valid JSONL (one JSON object per line)
- [ ] New session created automatically on start
- [ ] Session ID is a UUID

**Depends on**: #I001-S2
**Estimate**: S

### #I003-S6: Interactive readline loop

**Description**: `talos-cli` gains interactive mode (no TUI yet, just readline). User types a prompt, agent responds, repeat. `Ctrl+C` cancels current turn, double `Ctrl+C` exits.

**Acceptance Criteria**:
- [ ] `talos` (no args) starts interactive loop
- [ ] User input -> agent response -> prompt again
- [ ] `Ctrl+C` cancels current agent turn
- [ ] Double `Ctrl+C` exits the program
- [ ] Streaming output visible during response

**Depends on**: #I003-S4
**Estimate**: M

---

## I004: "Safe Agent"

**Delivers**: Dangerous operations are caught and contained.

### #I004-S1: Permission rules engine

**Description**: `talos-permission` evaluates tool calls against rules. Rules loaded from config: allow/deny/ask per tool name and path pattern. Wildcard matching with glob patterns. Default: ask for write operations, allow read operations.

**Acceptance Criteria**:
- [ ] Rules evaluated per tool call before execution
- [ ] `allow` -> execute immediately
- [ ] `deny` -> rejected with clear error message
- [ ] `ask` -> prompt user for approval
- [ ] Glob patterns match paths correctly (`src/**/*.rs`)
- [ ] Default ruleset: read=allow, write=ask, bash=ask

**Depends on**: #I003-S1
**Estimate**: M

### #I004-S2: Interactive approval prompt

**Description**: When a tool call needs approval, present it to the user. Show: tool name, arguments, risk level. User can approve once, approve always (add rule), or deny.

**Acceptance Criteria**:
- [ ] Approval prompt shows tool name and truncated arguments
- [ ] User options: y (approve once), a (always approve), n (deny)
- [ ] "Always approve" persists rule to config
- [ ] Non-interactive mode (`-p`) defaults to deny for ask-rules

**Depends on**: #I004-S1
**Estimate**: S

### #I004-S3: Bubblewrap sandbox (Linux)

**Description**: `talos-sandbox` runs bash commands in a Bubblewrap sandbox. Read-only filesystem except workspace root. Network namespace isolation. User/PID namespace isolation.

**Acceptance Criteria**:
- [ ] Commands run with restricted filesystem (writable only in workspace)
- [ ] Network access blocked by default (configurable)
- [ ] Cannot escape via symlink, `/proc`, `/sys`, or `../`
- [ ] `.git` directory always read-only
- [ ] Command fails gracefully if bwrap not installed (fallback to no sandbox)

**Depends on**: #I003-S2
**Estimate**: L

### #I004-S4: sandbox-exec (macOS)

**Description**: Implement macOS sandbox using `sandbox-exec` with Seatbelt profile. Similar restrictions to Bubblewrap: filesystem write restricted to workspace, network restricted.

**Acceptance Criteria**:
- [ ] Seatbelt profile generated dynamically based on workspace path
- [ ] Write access limited to workspace and temp directories
- [ ] `.git` directory always read-only
- [ ] Graceful fallback if sandbox-exec unavailable

**Depends on**: #I003-S2
**Estimate**: L

### #I004-S5: Process hardening basics

**Description**: Apply security measures to the agent process itself. Sanitize environment variables (remove `LD_PRELOAD`, `DYLD_*`). Set resource limits (max CPU, memory). Prevent core dumps.

**Acceptance Criteria**:
- [ ] Dangerous env vars stripped from child processes
- [ ] Resource limits configurable (default: reasonable limits)
- [ ] Core dumps disabled for sandboxed processes

**Depends on**: #I004-S3 or #I004-S4
**Estimate**: M

### #I004-S6: Tool execution pipeline integration

**Description**: Wire permission engine and sandbox into the tool execution flow. Pipeline: permission check -> sandbox selection -> execute -> retry on denial. Applies to all tools, not just bash.

**Acceptance Criteria**:
- [ ] Every tool call goes through permission -> sandbox -> execute pipeline
- [ ] File tools respect filesystem restrictions
- [ ] Bash tools run in sandbox when available
- [ ] Failed sandbox execution can retry with elevated permissions (user approval)
- [ ] Integration test: agent attempts dangerous operation, pipeline blocks it

**Depends on**: #I004-S1, #I004-S3, #I004-S4
**Estimate**: M

---

## I005: "Smart Agent"

**Delivers**: Mock LLM for testing, basic TUI shell, context compaction, and prompt caching.

### #I005-S1: Mock LLM provider

**Description**: Implement `LanguageModel` trait from `talos-core` as a mock provider in `talos-provider` (`#[cfg(test)]` module or separate `talos-mock` dev-dependency). Configurable response sequences (preset replies in order), simulates `tool_use` responses (returns tool call requests), simulates errors (401, 429, 500), supports streaming (simulates SSE delta events). Enables full agent testing without real API keys or network calls.

**Acceptance Criteria**:
- [ ] Implements `LanguageModel` trait from `talos-core`
- [ ] Configurable response sequences — preset replies returned in order
- [ ] Simulates `tool_use` responses — returns tool call requests when configured
- [ ] Simulates errors — can be configured to return 401, 429, 500 responses
- [ ] Supports streaming — simulates SSE delta events for streaming tests
- [ ] Placed in `talos-provider` as `#[cfg(test)]` module or separate `talos-mock` dev-dependency
- [ ] Unit tests verify: normal response, tool_use response, error response, streaming response

**Depends on**: #I002-S2
**Estimate**: S

### #I005-S2: Basic TUI shell

**Description**: `talos-tui` crate with ratatui + crossterm. Chat viewport for message display, input area for user prompts, status bar showing model/tokens/cost, Ctrl+C handling (cancel turn / exit), streaming output display. This is the foundational TUI shell that all subsequent iterations build upon.

**Acceptance Criteria**:
- [ ] `talos-tui` crate created with ratatui + crossterm dependencies
- [ ] Chat viewport renders messages with scrolling
- [ ] Input area accepts user text input
- [ ] Status bar displays: current model, token count, estimated cost
- [ ] Ctrl+C cancels current agent turn; double Ctrl+C exits
- [ ] Streaming output renders incrementally without blocking the TUI
- [ ] TUI works with Mock LLM for testing without API keys

**Depends on**: #I003-S6
**Estimate**: M

### #I005-S3: Token estimation

**Description**: Estimate token count for messages before sending to LLM. Character-based approximation (4 chars ~ 1 token) with provider-specific corrections. Track cumulative usage per session.

**Acceptance Criteria**:
- [ ] Token estimate within 20% of actual for English text
- [ ] Usage tracked per turn (input, output, cache_read, cache_write)
- [ ] Cost estimation based on model pricing
- [ ] Token count displayed in TUI status bar

**Depends on**: #I002-S3
**Estimate**: S

### #I005-S4: Context file loading (AGENTS.md)

**Description**: Load `AGENTS.md` files from working directory and parent directories. Concatenate all found files into system prompt context. Also load `~/.talos/AGENTS.md` as global context.

**Acceptance Criteria**:
- [ ] `AGENTS.md` loaded from cwd and all parent dirs up to git root
- [ ] Global `~/.talos/AGENTS.md` loaded first
- [ ] Content injected into system prompt
- [ ] `--no-context` flag disables loading
- [ ] Total context file size capped at 20,000 chars (head/tail truncation)

**Depends on**: #I002-S3
**Estimate**: S

### #I005-S5: 5-layer context compaction

**Description**: 5-layer compaction triggered when context nears model limit. Layer 1: budget (cap tool result sizes). Layer 2: trim (remove old tool results). Layer 3: microcompact (strip completed tool results by ID). Layer 4: collapse (summarize old turns). Layer 5: autocompact (LLM-based summarization).

**Acceptance Criteria**:
- [ ] Compaction triggers automatically at 80% context usage
- [ ] Manual `/compact` command available in interactive mode
- [ ] Compaction preserves recent turns (last 10) verbatim
- [ ] Summarization uses a separate compact LLM call
- [ ] After compaction, conversation continues seamlessly
- [ ] No infinite compact-fail-retry loops (circuit breaker: 3 failures -> stop)

**Depends on**: #I005-S3, #I002-S2
**Estimate**: XL

### #I005-S6: Prompt caching strategy

**Description**: Structure the system prompt for provider-side caching. Stable prefix (identity + tools + context files) kept constant across turns. Only conversation history grows. Add `cache_control` markers for Anthropic.

**Acceptance Criteria**:
- [ ] System prompt structure: static prefix + dynamic conversation
- [ ] Anthropic `cache_control` breakpoints set correctly
- [ ] Cache hit rate tracked and reported in usage stats
- [ ] Tool definitions maintain stable ordering

**Depends on**: #I005-S3
**Estimate**: M

---

## I006: "Data Agent"

**Delivers**: Production-grade event loop, TUI tool call visualization, approval overlay, session branching, and SQLite search.

### #I006-S0: Production-grade event loop architecture

**Description**: Implement the event loop architecture defined in ADR-004. Replace the current ad-hoc `select!` loop with a structured state machine: single event channel (`mpsc::unbounded`), explicit `AppState` enum (Idle → WaitingForInput → AgentRunning → ToolExecuting → ShuttingDown), layered cancellation (CancellationToken tree: app → session → turn → tools), stdin via `std::thread`, and render/logic separation. This is foundational infrastructure for all subsequent interactive features.

**Acceptance Criteria**:
- [ ] `AppEvent` enum defined with all event types (UserInput, UserInterrupt, AgentStarted, AgentTextDelta, AgentToolCall, AgentToolResult, AgentCompleted, AgentError, ShutdownRequested)
- [ ] `AppState` enum with explicit state transitions (Idle, WaitingForInput, AgentRunning, ToolExecuting, ShuttingDown)
- [ ] Main loop processes events from single `mpsc::unbounded` channel
- [ ] stdin reader runs in `std::thread` (not `tokio::spawn`), sends events via `blocking_send`
- [ ] Signal handler runs in `tokio::spawn`, sends `UserInterrupt` events
- [ ] Layered cancellation: first Ctrl+C cancels turn, second Ctrl+C (within 2s) cancels session
- [ ] Shutdown sequence: cancel tools → cancel agent → close channel → runtime exits
- [ ] `render(&state)` called after every state transition
- [ ] Existing interactive mode functionality preserved (input, agent execution, streaming output)
- [ ] Double Ctrl+C exits immediately without hanging (no stdin blocking)
- [ ] Unit tests for state machine transitions
- [ ] Integration test: simulated event sequence (input → agent → tool → complete)

**Depends on**: #I005-S1 (Mock LLM for testing)
**Estimate**: L
**Reference**: ADR-004

### #I006-S1: TUI tool call bubbles + approval overlay

**Description**: Enhance TUI chat viewport with visual tool call bubbles (showing tool name, arguments, results). Replace CLI approval prompt with TUI approval overlay (y/a/n) rendered on top of chat viewport.

**Acceptance Criteria**:
- [ ] Tool calls rendered as distinct bubbles in chat viewport
- [ ] Tool results displayed inline with success/failure indicators
- [ ] Approval overlay appears when permission is required
- [ ] User options: y (approve once), a (always approve), n (deny)
- [ ] Overlay dismisses cleanly after decision
- [ ] Non-interactive mode defaults to deny for ask-rules

**Depends on**: #I006-S0, #I005-S2, #I004-S2
**Estimate**: M

### #I006-S2: JSONL tree-branching sessions

**Description**: Extend session storage with parent-child relationships. Each entry has `id` and `parent_id`. Support forking from any point. Session resume via `-c` (continue last) and `-r` (select from history). Branching is implemented in JSONL only (no SQLite dependency yet).

**Acceptance Criteria**:
- [ ] `/fork` creates branch from current position
- [ ] `talos -c` resumes most recent session
- [ ] `talos -r` lists sessions by scanning JSONL directory
- [ ] Branch history preserved in single JSONL file
- [ ] Session metadata includes: timestamp, model, token count, working directory

**Depends on**: #I003-S5
**Estimate**: M

### #I006-S3: SQLite session index with FTS5

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

**Depends on**: #I003-S5, #I006-S2
**Estimate**: L

### #I006-S4: Session search and resume commands

**Description**: CLI commands for searching and resuming sessions using the SQLite index. `talos --search <query>` for full-text search, `talos -r` for session listing, `talos -c <session-id>` for resuming a specific session.

**Acceptance Criteria**:
- [ ] `talos --search <query>` returns matching sessions with snippets
- [ ] `talos -r` lists sessions sorted by last activity
- [ ] `talos -c <session-id>` resumes the specified session
- [ ] Search results show: session ID, project, last message preview, date
- [ ] Results limited to 20 by default, `--limit` flag for more

**Depends on**: #I006-S3
**Estimate**: S

### #I006-S5: Session fork command

**Description**: `/fork` command in TUI and `talos --fork <session-id>` CLI flag. Creates a new session branch from the current position or a specified session. Forked session inherits all prior messages.

**Acceptance Criteria**:
- [ ] `/fork` in TUI creates branch from current position
- [ ] `talos --fork <session-id>` forks from specified session
- [ ] Forked session has independent message history after fork point
- [ ] TUI shows session branch indicator in status bar
- [ ] Fork metadata recorded in SQLite index

**Depends on**: #I006-S2, #I006-S3
**Estimate**: M

---

## I007: "Skilled Agent"

**Delivers**: Skills system with TUI sidebar, SKILL.md parsing, progressive disclosure, and OpenAI provider.

### #I007-S1: TUI skill index sidebar

**Description**: Add a sidebar panel to the TUI showing loaded skills. Each skill displays name, description, and trigger status. Sidebar toggles with a keybinding. Skills update dynamically as they are loaded/unloaded.

**Acceptance Criteria**:
- [ ] Sidebar panel renders on the right side of TUI
- [ ] Each skill shows: name, description, active/inactive status
- [ ] Sidebar toggles with configurable keybinding
- [ ] Skills list updates when new SKILL.md files are discovered
- [ ] Sidebar collapses to icon-only mode when space is limited

**Depends on**: #I005-S2
**Estimate**: S

### #I007-S2: SKILL.md parser and loader

**Description**: `talos-skill` discovers and parses SKILL.md files. YAML frontmatter (name, description, trigger conditions) + Markdown body (instructions). Discovery from `.talos/skills/`, `~/.talos/skills/`, and parent directories.

**Acceptance Criteria**:
- [ ] SKILL.md parsed with frontmatter + body
- [ ] Discovery from 3 locations: project, user home, parent dirs
- [ ] Invalid SKILL.md files produce warnings, not crashes
- [ ] Skill index (name + description only) injected into system prompt

**Depends on**: #I001-S2
**Estimate**: M

### #I007-S3: Progressive disclosure (3 levels)

**Description**: Skills load in 3 levels. Level 0: name + description in system prompt (~50 tokens each). Level 1: full SKILL.md content loaded on demand when task matches. Level 2: specific reference files from skill.

**Acceptance Criteria**:
- [ ] Level 0 always loaded (skill index in system prompt)
- [ ] Level 1 loaded when agent's task matches skill trigger
- [ ] Level 2 loaded when agent needs specific reference files
- [ ] Total skill index stays under 3000 tokens for 20 skills

**Depends on**: #I007-S2
**Estimate**: M

### #I007-S4: OpenAI provider

**Description**: Add OpenAI as a second provider. Streaming via SSE. Chat Completions API format. Support `OPENAI_API_KEY` and `OPENAI_BASE_URL` for compatible providers.

**Acceptance Criteria**:
- [ ] `talos --provider openai --model gpt-4o` works
- [ ] Streaming text deltas via SSE
- [ ] Tool calls in OpenAI format translated to internal format
- [ ] `OPENAI_BASE_URL` override works for compatible APIs
- [ ] Model switching in interactive mode via `/model`

**Depends on**: #I002-S2
**Estimate**: M

### #I007-S5: System prompt assembly

**Description**: Assemble the full system prompt from: identity, tool descriptions, skill index, context files (AGENTS.md), and user preferences. Structure for optimal caching.

**Acceptance Criteria**:
- [ ] System prompt assembled from 5 sources with clear boundaries
- [ ] Order optimized for prompt caching (stable content first)
- [ ] Custom system prompt via `--system-prompt` flag
- [ ] Append via `--append-system-prompt` flag
- [ ] Total system prompt size logged for debugging

**Depends on**: #I005-S4, #I007-S2
**Estimate**: M

---

## I008: "Learning Agent"

**Delivers**: Self-evolution engine with cognitive feedback and TUI insights panel.

### #I008-S1: TUI evolution insights panel + /learned command

**Description**: Add an evolution insights panel to the TUI showing learned patterns, confidence scores, and evidence counts. `/learned` command opens the panel. Panel displays: top patterns by confidence, recent observations, pattern conflicts, and time-decay visualization.

**Acceptance Criteria**:
- [ ] `/learned` command opens evolution insights panel in TUI
- [ ] Panel shows patterns sorted by confidence score
- [ ] Each pattern displays: description, confidence, evidence count, last reinforced date
- [ ] Pattern conflicts highlighted with resolution status
- [ ] Panel supports scrolling for long pattern lists
- [ ] Insights persist across sessions (loaded from SQLite)

**Depends on**: #I005-S2
**Estimate**: M

### #I008-S2: Evolution engine with cognitive feedback (ADR-001)

**Description**: Implement the `talos-evolution` crate with the 4-phase learning loop (ADR-001): Observe → Accumulate → Extract → Apply. The exact signal taxonomy, confidence formulas, decay rates, and conflict resolution strategies will be designed at the start of I008 based on real usage data from I001-I007. Storage uses direct rusqlite calls extending the database from I006-S3. Skill creation from experience is one output channel — when a pattern stabilizes, it can be materialized as a SKILL.md.

**Acceptance Criteria**:
- [ ] `TurnObserver` captures structured observations per turn (tool calls, duration, outcome, signals)
- [ ] Signal taxonomy designed based on I001-I007 usage patterns (per ADR-001 cognitive feedback principles)
- [ ] `PatternExtractor` extracts preferences, project patterns, and error-avoidance rules
- [ ] Contradiction detection: new patterns are checked against existing ones before storage
- [ ] Patterns carry confidence scores with evidence backing and time decay
- [ ] Extraction triggers include signal-driven events, not just session boundary
- [ ] SQLite tables: observations, patterns, pattern_conflicts
- [ ] `BehaviorAdapter` injects high-confidence patterns into system prompt
- [ ] Evolution data inspectable via `/learned` command
- [ ] User can disable evolution via config

**Depends on**: #I007-S2, #I003-S4, #I006-S3
**Estimate**: XL

---

## I009: "Extensible Agent"

**Delivers**: Hook system, MCP integration, plugin runtime, and TUI extensions for external tools.

### #I009-S1: TUI MCP tool markers + plugin status

**Description**: Enhance TUI to visually distinguish MCP-provided tools from built-in tools (special icon/badge). Add plugin status display showing loaded plugins, active hooks, and hook execution counts.

**Acceptance Criteria**:
- [ ] MCP-provided tools display with distinct marker/icon in tool call bubbles
- [ ] Plugin status panel shows: loaded plugins, active hooks, execution counts
- [ ] Hook execution logged in TUI (subtle indicator, not intrusive)
- [ ] Plugin load errors displayed as warnings in status bar
- [ ] `/plugins` command lists all loaded plugins and their hooks

**Depends on**: #I005-S2
**Estimate**: S

### #I009-S2: Hook system (20+ extension points)

**Description**: Define hooks at key points in the agent lifecycle. Hook system is pure Rust, no WASM dependency. Hooks at key lifecycle points: before_tool_call, after_tool_call, message_transform, system_prompt_transform, permission_check, session_start/end. Plugins register handlers.

**Acceptance Criteria**:
- [ ] 20+ hook points defined and documented
- [ ] Plugins can register multiple handlers per hook
- [ ] Hook handlers run in registration order
- [ ] Hooks can modify or block operations
- [ ] Performance overhead < 1ms per hook invocation

**Depends on**: None
**Estimate**: L

### #I009-S3: MCP client

**Description**: `talos-mcp` connects to external MCP servers. Discovers tools, resources, and prompts from servers. Converts MCP tool definitions to AgentTool implementations. Config via `mcp` section in config file.

**Acceptance Criteria**:
- [ ] Connect to MCP server via stdio or HTTP
- [ ] MCP tools available as AgentTool in the agent
- [ ] Tool results from MCP servers forwarded correctly
- [ ] Multiple MCP servers supported simultaneously
- [ ] Connection failures handled gracefully (retry + skip)

**Depends on**: #I003-S1
**Estimate**: L

### #I009-S4: MCP server

**Description**: Expose Talos tools as an MCP server. Other agents can connect and use Talos tools. Support stdio transport.

**Acceptance Criteria**:
- [ ] `talos --mcp-server` starts in MCP server mode
- [ ] All registered tools exposed via MCP protocol
- [ ] External MCP clients can call Talos tools
- [ ] Permission rules still enforced for external callers

**Depends on**: #I003-S1
**Estimate**: L

### #I009-S5: JSON-RPC server (stdio)

**Description**: `talos-rpc` implements JSON-RPC over stdio. Methods: session/start, session/list, turn/start, turn/interrupt, config/read. Enables IDE and tool integration.

**Acceptance Criteria**:
- [ ] `talos --mode rpc` starts JSON-RPC server on stdio
- [ ] Core methods work: start session, send prompt, receive response
- [ ] Streaming events delivered as JSON-RPC notifications
- [ ] Error responses follow JSON-RPC error format

**Depends on**: #I002-S3
**Estimate**: M

### #I009-S6: TUI provenance markers + `/plugins` command (deferred from I009)

**Description**: Consumer-side rendering of the `ToolProvenance` data already produced by the backend (ADR-009). MCP-provided tools should display with a distinct marker/icon in tool call bubbles. A `/plugins` slash command should list all loaded plugins and their hook registrations. This work was deferred from I009 through change control during R1 Review Closure; I009's backend/runtime surface is complete and the remaining gap is TUI consumption only.

**Status**: Deferred (moved through change control from I009 during R1 Review Closure). Resume in I010 R2/R3 or a dedicated follow-up iteration.

**Acceptance Criteria**:
- [ ] MCP-provided tools display with distinct marker/icon in TUI tool call bubbles
- [ ] `/plugins` command lists all loaded plugins and their hook registrations
- [ ] Plugin status indicator visible in TUI status bar when hooks are active
- [ ] Hook execution logged subtly in TUI (not intrusive)

**Depends on**: #I009-S1 (producer side, already complete), #I005-S2
**Estimate**: S

---

## I010: "Polished Agent"

**Delivers**: Release-ready terminal experience with Codex-like CLI/TUI integration, Nord theme, markdown rendering, diff display, and advanced features.

### #I010-S1: Nord theme application

**Description**: Apply the Nord color scheme (https://www.nordtheme.com/) across all TUI components per REFERENCE-PROJECTS.md §19. Define Ratatui `Color::Rgb` constants for all Nord palette colors. Verify WCAG AA contrast ratios for all text/background combinations.

**Acceptance Criteria**:
- [ ] Nord color palette defined as Ratatui `Color::Rgb` constants module
- [ ] All TUI components use Nord colors (no hardcoded hex values)
- [ ] Chat viewport, status bar, sidebar, overlays all themed consistently
- [ ] WCAG AA contrast ratio verified for all text/background combinations
- [ ] Dark/light mode toggle (Nord Polar Night vs Nord Snow Storm)

**Depends on**: #I005-S2
**Estimate**: M

### #I010-S2: Markdown rendering in assistant messages

**Description**: Render markdown in assistant messages: code blocks with syntax highlighting, headers, lists, links, bold/italic text, inline code. Use a markdown parser compatible with ratatui rendering.

**Acceptance Criteria**:
- [ ] Code blocks rendered with syntax highlighting (Rust, Python, JS, etc.)
- [ ] Headers displayed with appropriate sizing/bolding
- [ ] Lists (ordered and unordered) rendered correctly
- [ ] Links displayed as clickable or copyable
- [ ] Bold/italic text rendered with appropriate styling
- [ ] Inline code highlighted distinctly
- [ ] Long code blocks support scrolling within message bubble

**Depends on**: #I005-S2
**Estimate**: L

### #I010-S3: Diff display for file changes

**Description**: Visual diff rendering for file changes in chat viewport. Show added/removed/modified lines with color coding (green for additions, red for deletions). Support unified diff format.

**Acceptance Criteria**:
- [ ] File tool results with diffs rendered with line-by-line coloring
- [ ] Added lines highlighted in green, removed lines in red
- [ ] Line numbers displayed for context
- [ ] Large diffs support scrolling within the diff viewport
- [ ] `/diff` command shows git diff for current working directory

**Depends on**: #I005-S2, #I003-S3
**Estimate**: M

### #I010-S4: Steering and follow-up queues

**Description**: Two message queues for mid-run input. Steering: delivered after current tool batch, before next LLM call. Follow-up: delivered only when agent would stop. Both support one-at-a-time or all-at-once drain modes. ChatComposer enters "queue mode" when agent is running — Enter queues message instead of interrupting.

**Acceptance Criteria**:
- [ ] Enter while agent works -> queues steering message
- [ ] Alt+Enter -> queues follow-up message
- [ ] Escape -> cancels and restores queued messages to editor
- [ ] Drain mode configurable in settings
- [ ] ChatComposer shows queued message count indicator

**Depends on**: #I005-S2
**Estimate**: M

### #I010-S5: Slash commands with fuzzy filtering

**Description**: Slash commands in TUI: `/model` (switch), `/new` (new session), `/resume` (pick session), `/fork` (branch), `/compact` (manual compaction), `/diff` (show git diff), `/status` (session config + token usage), `/vim` (toggle vim mode), `/help`, `/quit`. Tab autocomplete for commands. Fuzzy filtering as you type.

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

**Depends on**: #I005-S2, #I006-S2
**Estimate**: M

### #I010-S6: Guardian AI sub-agent

**Description**: Auto-approve low-risk tool calls using a lightweight LLM call. Guardian reviews tool call + context and decides approve/deny. Circuit breaker: 3 consecutive denials blocks Guardian.

**Planning note**: Valid backlog story, but not part of the first I010 product-polish pass unless
activated through change control.

**Acceptance Criteria**:
- [ ] Guardian reviews tool calls when enabled in config
- [ ] Low-risk operations auto-approved without user prompt
- [ ] Guardian denial triggers user approval prompt
- [ ] Circuit breaker activates after 3 denials
- [ ] Guardian uses a cheaper/faster model than main agent

**Depends on**: #I004-S1, #I002-S2
**Estimate**: L

### #I010-S7: Headless and SDK modes

**Description**: Three execution modes plus a Codex-like terminal UX. Interactive supports both full-screen TUI and inline/no-alt-screen terminal mode that preserves normal scrollback and feels like a natural CLI extension. Headless (`talos exec`) runs autonomously for CI/automation, no TUI. SDK exposes Talos as a Rust library for embedding. All modes share the same core agent loop via AppServerSession abstraction (Codex pattern: TUI never calls agent loop directly). Canonical architecture defined in [ADR-005](../decisions/005-tui-event-architecture.md): bounded SQ (cap=512) / unbounded EQ seam; SQ/EQ protocol types in `talos-core`, session actor in `talos-agent`. This story is the convergence point for the three current run paths (`run_print_mode`, `run_interactive_mode`, `run_tui_mode`).

> **Re-scope 2026-06-01:** I008 self-evolution is no longer part of this story. Evolution
> ships in I008 as a builtin `HookHandler` registered per-Agent (see
> [ADR-005 → "Hook-Driven Evolution"](../decisions/005-tui-event-architecture.md#hook-driven-evolution-2026-06-01-pre-i008-re-scope)
> and `docs/iterations/I008-learning-agent.md`). #I010-S7 retains the architectural cleanup
> for cross-Agent / cross-session correlation, UI status broadcast, and the three-run-path
> convergence onto a single seam. These remain independently valuable.

**Acceptance Criteria**:
- [ ] `talos exec "run tests and fix failures" --max-turns 20` runs autonomously
- [ ] `talos exec` exits with code 0 on success, 1 on failure
- [ ] SDK: `AgentSession::new()` -> `session.prompt("hello")` works as library
- [ ] All modes share the same core agent loop
- [ ] Headless mode supports `--json` for machine-readable output
- [ ] All three run paths drive the agent only via `AppServerSession` (no direct `tokio::spawn` of the agent turn inside a run path)
- [ ] TUI approval requests flow through the canonical `AppServerSession`/EQ approval protocol, not through a `PermissionAwareTool { print_mode: true }` default-deny shim
- [ ] Inline/no-alt-screen mode preserves terminal scrollback and does not feel like a separate full-screen application
- [ ] Full-screen TUI and inline mode consume the same ordered EQ event stream, so assistant deltas, tool output, approvals, and status updates share one rendering contract
- [ ] Command output and approval prompts can be interleaved with the conversation without losing shell-like terminal ergonomics
- [ ] Existing I008 hook-based evolution remains stable during migration; no duplicate `TurnStart`/`TurnComplete` lifecycle events are introduced
- [ ] `event_loop.rs` dead variants removed (`ApprovalRequested`, `ApprovalResolved`, `ToggleSkillSidebar`, `SkillsUpdated`, `ApprovalChoice`)
- [ ] `cargo test --workspace` green after each path migration (ADR-005 phased-migration invariant)

**Depends on**: #I005-S2, ADR-005
**Estimate**: L

### #I010-S8: Exec policy DSL rules

**Description**: Full DSL for command approval rules in `.talos/rules/*.rules`. Pattern matching on command name, arguments, paths. Support for trusted commands, forbidden patterns, and conditional rules.

**Planning note**: Valid backlog story, but not part of the first I010 product-polish pass unless
activated through change control.

**Acceptance Criteria**:
- [ ] Rule files loaded from `.talos/rules/` and `~/.talos/rules/`
- [ ] Rules match on command name, glob patterns, path prefixes
- [ ] `trusted` rules auto-approve, `forbidden` rules auto-deny
- [ ] Complex shell features (pipes, redirects) bypass rules and require approval
- [ ] Rules can reference environment variables

**Depends on**: #I004-S1
**Estimate**: M

---

## ARCH: Architecture Review Remediation

> **Source**: architecture & security review (2026-06). These stories fix **existing defects** found
> in shipped code, not new features. They are self-contained — each lists exact file paths, line
> anchors, MUST DO / MUST NOT DO, and acceptance criteria so they can be executed without re-deriving
> the analysis.
>
> **Target window**: complete `#ARCH-S1`…`#ARCH-S4` **before starting I009**. They harden the
> security baseline that I009 (MCP/plugins exposing tools to external callers) depends on.
> Complete `#ARCH-S5`…`#ARCH-S7` in the same remediation lane as current correctness findings
> from the I006 session stack; `#ARCH-S5` and `#ARCH-S7` should land before I009, while
> `#ARCH-S6` may coordinate with `#I010-S7` if the active session handoff touches run-path migration.
>
> **Scope guard**: do NOT expand these into the larger I010 event-architecture migration. The
> three-run-path convergence and dead `event_loop.rs` variant deletion are owned by **#I010-S7** and
> governed by [ADR-005](../decisions/005-tui-event-architecture.md) / [ADR-006](../decisions/006-event-architecture-boundary.md).
> If a change here would require touching the agent turn-loop spawn model, STOP — that is I010 scope.

### #ARCH-S1: Link sandbox `unsafe` blocks to ADR-007 (compliance, P0)

**Description**: AGENTS.md Hard Constraint #2 requires every `unsafe` to have an ADR.
[ADR-007](../decisions/007-process-hardening-unsafe.md) now exists; this story makes the code
**discoverably** reference it and closes the gap recorded in `EVOLUTION.md`. **Documentation/comment
change only — zero runtime-logic change.**

**Files & anchors** (`crates/talos-sandbox/src/hardening.rs`):
- Module `# Safety` section, ~lines 18–25
- `// SAFETY:` comment at `env::remove_var`, ~line 248–254
- `// SAFETY:` comment at `setrlimit(RLIMIT_CORE)`, ~line 281–284
- `// SAFETY:` comment at `setrlimit(RLIMIT_CPU)`, ~line 298–299
- `// SAFETY:` comment at `setrlimit(RLIMIT_AS)`, ~line 313–314
- `EVOLUTION.md` (the entry acknowledging the missing-ADR gap)

**MUST DO**:
- Append `See docs/decisions/007-process-hardening-unsafe.md` to the module `# Safety` section and to **each** of the four `// SAFETY:` comments.
- Update the `EVOLUTION.md` gap entry to state the gap is resolved by ADR-007 (do not delete the lesson; mark it resolved).

**MUST NOT DO**:
- Do NOT change any `unsafe` block body, the `rlimit` values, the env-var list, or any control flow.
- Do NOT add/remove dependencies. Do NOT touch tests.

**Acceptance Criteria**:
- [ ] All four production `unsafe` SAFETY comments and the module `# Safety` note cite ADR-007
- [ ] `EVOLUTION.md` gap entry marked resolved (references ADR-007)
- [ ] `git diff` shows comment/doc lines only — no executable line changed
- [ ] `cargo test --workspace` exits 0

**Depends on**: ADR-007
**Estimate**: S
**Risk gate**: sandbox change → security review (AGENTS.md #5), though comment-only

### #ARCH-S2: Deprecate zero-security `Agent::new()` (escape vector, P1)

**Description**: `Agent::new()` (`crates/talos-agent/src/lib.rs:141–150`) constructs an agent with
`permission_engine: None` and `sandbox: None` — a fully **un-gated** agent. This is a silent escape
vector: a caller who uses `new()` instead of `with_security()` (`lib.rs:195–211`) gets no permission
checks and no sandbox. Make the insecure path loud and ensure no production run path uses it.

**Files & anchors**:
- `crates/talos-agent/src/lib.rs:141–150` (`Agent::new`)
- `crates/talos-agent/src/lib.rs:195–211` (`Agent::with_security` — the correct constructor)
- `crates/talos-cli/src/main.rs` run paths: `run_print_mode` (~261), `run_tui_mode` (~396), `run_interactive_mode` (~476)

**MUST DO**:
- Add `#[deprecated(note = "Agent::new() has NO permission engine and NO sandbox; use Agent::with_security(). See docs/decisions/007 and ARCH review.")]` to `Agent::new`.
- Expand the `///` doc on `Agent::new` to state explicitly it is unsafe-by-policy (no gating) and is intended for tests only.
- Audit all three CLI run paths; confirm each builds its agent via `with_security(...)`. If any uses `new()`, switch it to `with_security(...)`.
- If `#[cfg(test)]` code triggers the deprecation lint and the workspace denies warnings, scope a narrow `#[allow(deprecated)]` **only** on the specific test items, each with a one-line comment explaining why.

**MUST NOT DO**:
- Do NOT delete `Agent::new` (unit tests legitimately use a no-security agent).
- Do NOT change `with_security` behavior or signatures.
- Do NOT blanket-`#[allow(deprecated)]` at crate/module scope to silence the warning globally.
- Do NOT add `as any`-style suppressions or alter the permission/sandbox defaults.

**Acceptance Criteria**:
- [ ] `Agent::new` carries `#[deprecated(note = …)]` pointing callers to `with_security`
- [ ] All three production run paths (`print`/`tui`/`interactive`) construct the agent via `with_security`
- [ ] No `#[allow(deprecated)]` exists outside `#[cfg(test)]`
- [ ] `cargo build --workspace` produces no deprecation warnings from production (non-test) code
- [ ] `cargo test --workspace` exits 0

**Depends on**: none
**Estimate**: S
**Risk gate**: permission change → security review (AGENTS.md #5)

### #ARCH-S3: Wire `ProcessHardening` into child execution — closes #I004-S5 false-complete (P2)

**Description**: `#I004-S5` was marked complete, but `ProcessHardening` (`crates/talos-sandbox/src/hardening.rs`)
is **defined and unit-tested yet never invoked** by any production execution path — a *security
illusion*. `apply()` exists; nothing calls it. Hardening must be applied to the **child** bash
subprocess (via `Command::pre_exec` on Unix), **not** to the parent CLI process. Applying `RLIMIT_AS`
/ `RLIMIT_CPU` to the parent would cripple Talos itself.

**Files & anchors**:
- `crates/talos-sandbox/src/hardening.rs` (`ProcessHardening::apply`, `apply_rlimits`, `sanitize_env_vars_internal`)
- Bash subprocess spawn site (the `tokio::process::Command` for the bash tool) reached from `crates/talos-agent/src/lib.rs:489` (`execute_single_tool`) → bash sandbox branch ~`lib.rs:521–527`; confirm whether the actual spawn lives in `talos-tools` (bash tool) or `talos-sandbox` and harden at the real spawn point.

**MUST DO**:
- Apply hardening to the **child** bash process: on Unix, set env scrubbing + `setrlimit` inside `Command::pre_exec` (or the platform sandbox's own child-spawn), so limits bind the subprocess, not the parent.
- Ensure dangerous env vars (`LD_PRELOAD`, `DYLD_*`, …) are absent from the child's environment.
- Add an **integration test** proving, in the child: (a) a set `LD_PRELOAD` is not visible, and (b) a resource limit is in effect (e.g., `RLIMIT_CORE`/`RLIMIT_CPU` observable via the child).
- Keep the `pre_exec` closure async-signal-safe (no allocation/locking/arbitrary Rust) per ADR-007.
- Any `unsafe` introduced (`pre_exec`) MUST carry a `// SAFETY:` comment referencing ADR-007.

**MUST NOT DO**:
- Do NOT call `ProcessHardening::apply()` on the parent CLI process.
- Do NOT introduce `unsafe` not covered by ADR-007 (update ADR-007 first if scope grows).
- Do NOT change the default limit values in `ProcessHardening::new()`.
- Do NOT weaken existing sandbox behavior or its graceful-degradation fallback.

**Acceptance Criteria**:
- [ ] Bash subprocesses run with env sanitization + resource limits applied **in the child**
- [ ] Integration test demonstrates `LD_PRELOAD` stripped and a resource limit active in the child
- [ ] Parent CLI process is unaffected (no rlimit applied to self)
- [ ] `#I004-S5` acceptance criteria are now genuinely satisfied at runtime, not just in unit tests
- [ ] Any new `unsafe` cites ADR-007; `cargo test --workspace` exits 0

**Depends on**: ADR-007, #I004-S5
**Estimate**: M
**Risk gate**: process-hardening change → security review (AGENTS.md #5) — review against escape vectors

### #ARCH-S4: Unify the duplicated `ApprovalChoice` definitions (P3)

**Description**: `ApprovalChoice` is defined **three times** with drifting copies:
`crates/talos-cli/src/approval.rs:20` (live), `crates/talos-tui/src/lib.rs:228` (live), and
`crates/talos-cli/src/event_loop.rs:27` (dead). Three sources of truth for an approval (y/a/n)
decision invites divergence. Consolidate the two **live** definitions onto one canonical type.

**Files & anchors**:
- `crates/talos-cli/src/approval.rs:20` (live definition + usage)
- `crates/talos-tui/src/lib.rs:228` (live definition + usage)
- `crates/talos-cli/src/event_loop.rs:27` (DEAD copy — see MUST NOT DO)

**MUST DO**:
- Define **one** canonical `ApprovalChoice` and have `approval.rs` and the TUI both use it.
- Place the canonical type in `talos-core` (depends on nothing; every crate may depend on it — no circular-dependency risk). Only choose `talos-permission` instead if (and only if) `talos-tui` already depends on it; verify the dependency edge before deciding, and keep `talos-core` as the default.
- Give the canonical type `///` doc comments on the type and each variant (AGENTS.md doc rule).
- Update all live call sites to the canonical type; keep behavior identical (same variants/semantics).

**MUST NOT DO**:
- Do NOT delete or modify the dead `event_loop.rs:27` copy — its removal is owned by **#I010-S7** (ADR-005 phased migration). Touching dead `event_loop.rs` variants here is out of scope.
- Do NOT introduce a circular dependency (verify `cargo check` after placing the type).
- Do NOT change the approval semantics (y = approve once, a = always, n = deny).

**Acceptance Criteria**:
- [ ] Exactly one **live** `ApprovalChoice` definition; `approval.rs` and TUI both import it
- [ ] Canonical type lives in `talos-core` (or `talos-permission` only if justified), with `///` docs
- [ ] No new circular dependencies; `cargo check --workspace` clean
- [ ] Dead `event_loop.rs:27` copy left untouched (deferred to #I010-S7)
- [ ] `cargo test --workspace` exits 0

**Depends on**: none (coordinates with #I010-S7 for the dead-copy deletion)
**Estimate**: S

### #ARCH-S5: Keep the SQLite session index current on normal turns (P2)

**Description**: The I006 SQLite index is treated as the fast path for `talos --search` and
`talos -r`, but normal interactive turns append to JSONL without refreshing the index. Fork paths
and tests call `SessionIndex::index_session(...)`; the ordinary "user prompt -> assistant response"
path does not. Because `SessionIndex::search(...)` and `SessionIndex::list_recent(...)` query only
SQLite, search/list can miss recently created or updated sessions even though JSONL remains the
source of truth.

**Files & anchors**:
- `crates/talos-cli/src/event_loop.rs:461` (`SessionEntry::user(...)` append during normal turns)
- `crates/talos-session/src/lib.rs:661` (`SessionIndex::search`)
- `crates/talos-session/src/lib.rs:670` (`SessionIndex::list_recent`)
- `crates/talos-session/src/lib.rs` (`SessionManager::update_index`)

**MUST DO**:
- Refresh the SQLite index after normal session writes that should be visible to search/list.
- Preserve JSONL as the source of truth; SQLite stays a derived index.
- Ensure injected/custom session directories also use their colocated `index.db` from ADR-008 follow-up work.
- Add a regression test proving a newly written normal session appears in `list_recent` and `search`
  without a manual reindex command.

**MUST NOT DO**:
- Do NOT move session source-of-truth data from JSONL into SQLite.
- Do NOT introduce a storage trait abstraction; ADR-002 still says direct `rusqlite` calls until a
  concrete second engine exists.
- Do NOT solve this by scanning all JSONL files on every search/list call unless there is an explicit
  bounded backfill path.

**Acceptance Criteria**:
- [ ] A normal completed turn updates the session's SQLite metadata/content index
- [ ] `talos -r` / `list_recent` includes the just-updated session
- [ ] `talos --search <term>` finds content from the just-updated session
- [ ] Custom session roots remain isolated from `$HOME/.talos`
- [ ] Regression tests cover the stale-index failure mode; `cargo test -p talos-session` exits 0

**Depends on**: #I006-S3, #I006-S4, ADR-002, ADR-008
**Estimate**: S

### #ARCH-S6: Repair interactive fork session identity and continuation (P2)

**Description**: Interactive `/fork` creates a new session file path and UUID, but the cloned
`Session` value still carries the source session identity/path when it is indexed. The event loop
also records `ForkCompleted` by setting `branch_id` only; it does not switch the active session to
the fork. The result can be a fork file with copied entries, stale index metadata for the source
session, and later turns continuing on the original session rather than the fork.

**Files & anchors**:
- `crates/talos-cli/src/event_loop.rs:326` (`let mut forked = session.clone()`)
- `crates/talos-cli/src/event_loop.rs:334` / `:335` (new fork id/path creation)
- `crates/talos-cli/src/event_loop.rs:358` (`index.index_session(&forked)`)
- `crates/talos-cli/src/event_loop.rs:241` (`AppEvent::ForkCompleted` handler)
- `crates/talos-session/src/lib.rs` (`Session` identity/path fields and fork helpers)

**MUST DO**:
- Ensure the forked `Session` carries the new session id, parent/root metadata, and JSONL path before
  it is indexed.
- Update the active interactive session after `/fork` so subsequent turns append to the fork, not the
  source session.
- Keep fork history independent after the fork point while preserving parent/branch metadata.
- Add a regression test for "fork, then append another turn" proving the new turn lands in the fork
  file and the SQLite index points at the fork id/path.

**MUST NOT DO**:
- Do NOT redesign the full ADR-005 run-path architecture in this story.
- Do NOT duplicate fork logic independently across CLI/TUI if an existing `talos-session` helper can
  own the identity/path mutation.
- Do NOT change the JSONL entry format without a migration note.

**Acceptance Criteria**:
- [ ] `/fork` creates a session with a distinct id and path in both JSONL and SQLite metadata
- [ ] The active interactive session switches to the fork after `ForkCompleted`
- [ ] Subsequent turns append to the forked session only
- [ ] `talos -c <fork-id>` resumes the fork
- [ ] Regression tests cover fork metadata, index metadata, and post-fork append behavior

**Depends on**: #I006-S2, #I006-S3, #I006-S5
**Estimate**: M

### #ARCH-S7: Fix CLI search highlight output leaking literal `BOLD` (P4)

**Description**: Search snippets replace `<b>` with an ANSI-color string that includes the literal
text `BOLD`, so highlighted search output can show `BOLDmatched text` instead of only applying the
bold terminal style. The existing test checks for ANSI output and removed HTML tags, but not for
absence of the marker text.

**Files & anchors**:
- `crates/talos-cli/src/main.rs:662` (search snippet `<b>` replacement)
- `crates/talos-cli/src/main.rs:797` (highlight rendering tests)

**MUST DO**:
- Replace `<b>` with the ANSI bold escape sequence only; do not emit literal marker text.
- Extend the existing search snippet test to assert the output does not contain `BOLD`.

**MUST NOT DO**:
- Do NOT change search ranking, FTS query behavior, or snippet generation.
- Do NOT remove color output support.

**Acceptance Criteria**:
- [ ] Highlighted search output applies bold styling without printing `BOLD`
- [ ] Regression test fails on the current marker leak and passes after the fix
- [ ] `cargo test -p talos-cli search` exits 0

**Depends on**: #I006-S4
**Estimate**: XS

## Iteration I011: Open Providers

**Status**: Paused after S1. `#I011-S1` is implemented and verified; `#I011-S2` remains valid
backlog work but is deferred while R1 closes I008/I009 review drift and I010 R2 prepares to
become the next mainline implementation slice.

**Theme**: Decouple `talos` from hard-coded provider lists so it can talk to any
OpenAI-compatible gateway (DashScope, Bailian, Z.ai, self-hosted vLLM) without code
changes. Foundation for the longer-term **provider plugin architecture** (see
[docs/proposals/provider-plugin-architecture.md](../proposals/provider-plugin-architecture.md)).

### #I011-S1: OpenAI-compatible `base_url` override

**Status**: Done. See `docs/iterations/I011-open-providers.md` for verification evidence and the
2026-06-02 gateway-root bugfix.

**Description**: `talos` currently hard-codes the Anthropic and OpenAI base URLs in
`talos-provider`. Many production gateways (Alibaba Cloud Bailian / DashScope, Z.ai,
self-hosted vLLM, LocalAI, etc.) expose the OpenAI chat-completions protocol at a
different URL. Allow users to override the base URL via `~/.talos/config.toml` without
touching code or env vars beyond the auth token.

**Files & anchors**:
- `crates/talos-config/src/lib.rs:50-72` (`Config` struct)
- `crates/talos-config/src/lib.rs:172-190` (`api_key()` resolution)
- `crates/talos-cli/src/main.rs:822-832` (`build_provider()`)

**MUST DO**:
- Add `base_url: Option<String>` to `Config` (`#[serde(default)]` so existing configs
  load unchanged).
- Add `Config::base_url() -> Option<&str>` getter.
- Extend `Config::api_key()` resolution chain for the OpenAI provider with a
  `OPENAI_COMPAT_API_KEY` env var fallback, checked **after** `OPENAI_API_KEY` so
  existing setups are unaffected.
- Wire `config.base_url()` into `talos-cli::build_provider()` for `Provider::OpenAI`
  only (Anthropic provider keeps its hard-coded URL for this iteration).
- Add unit tests in `talos-config` for the new field, the new env var, and the
  precedence (`config.api_key` > `OPENAI_API_KEY` > `OPENAI_COMPAT_API_KEY`).
- Update the missing-key error message to mention `OPENAI_COMPAT_API_KEY` when
  applicable.

**MUST NOT DO**:
- Do NOT add a new `Provider::*` variant. `provider = "openai"` is the gateway for all
  OpenAI-compatible endpoints. Adding a new variant per gateway is the scope of
  #I011-S2 / provider-plugin-architecture.
- Do NOT add a `--base-url` CLI flag. The config file is the canonical override
  surface; CLI flag is out of scope.
- Do NOT touch Anthropic provider. The hard-coded `https://api.anthropic.com/...` is
  fine.
- Do NOT add support for `thinking` / `reasoning` / `budgetTokens` fields. Out of scope
  for this iteration; tracked separately (see proposal docs).
- Do NOT add new env vars other than `OPENAI_COMPAT_API_KEY`. Naming: "compat" signals
  "OpenAI-compatible protocol" without committing to a specific gateway.

**Acceptance Criteria**:
- [x] `~/.talos/config.toml` accepts `base_url = "https://..."`; loaded config exposes
      it via `Config::base_url()`.
- [x] `Provider::OpenAI` with `base_url` set sends requests to that URL.
- [x] `Provider::OpenAI` with no `base_url` keeps the hard-coded OpenAI URL.
- [x] `OPENAI_API_KEY` is checked before `OPENAI_COMPAT_API_KEY`; the latter is a
      fallback only.
- [x] `Provider::Anthropic` never reads `OPENAI_COMPAT_API_KEY` (or any openai env var).
- [x] Missing-key error message for the OpenAI provider mentions both
      `OPENAI_API_KEY` and `OPENAI_COMPAT_API_KEY`.
- [x] `cargo test --workspace` exits 0; 6 new tests in `talos-config`.
- [x] `cargo clippy -p talos-cli --bin talos -p talos-config -- -D warnings` clean.

**Reference (well-known OpenAI-compatible gateways this enables)**:

The following public endpoints are known to follow the OpenAI chat-completions protocol
and work with this story out of the box. The list is illustrative, not exhaustive — any
endpoint that follows the same protocol works.

| Gateway | Base URL (compatible mode) | Notable models |
|---|---|---|
| Alibaba Cloud Bailian (token plan) | `https://token-plan.cn-beijing.maas.aliyuncs.com/compatible-mode/v1` | `glm-5`, `qwen3.6-plus`, `MiniMax-M2.5`, `deepseek-v3.2` |
| Alibaba Cloud DashScope | `https://dashscope.aliyuncs.com/compatible-mode/v1` | `qwen-plus`, `qwen-coder-plus`, `qwen3.5-plus` |
| Z.ai (Zhipu) coding plan | (OpenAI-compatible; URL published by the vendor) | `glm-4.7`, `kimi-k2.5` |
| Self-hosted vLLM | depends on deployment | any |

Example (Bailian):
```toml
# ~/.talos/config.toml
provider = "openai"
model = "glm-5"
base_url = "https://token-plan.cn-beijing.maas.aliyuncs.com/compatible-mode/v1"
```
```bash
export OPENAI_COMPAT_API_KEY="<your key from the gateway's dashboard>"
cargo run -p talos-cli -- -p "用中文回答:1+1=?"
```
The key never appears in the repo, config file, or git history.

**Depends on**: none (no blocker; standalone)
**Estimate**: S

### #I011-S2: Provider plugin architecture (foundation, not full plugin system)

**Description**: A long-term direction. Today each new provider requires a Rust
compile-time addition to `talos-provider` and a hard-coded `Provider::*` enum variant
in `talos-config`. The plugin architecture would let users register providers at
runtime via a `~/.talos/providers.toml` (or similar) that mirrors the opencode config
schema (`options.baseURL`, `models.{name,options,limit}`, auth via env var per provider).
This story is the **first slice** of that direction: capture the opencode config
schema as a Rust type and write a one-way migration from opencode's `provider` block
into `talos`'s `Config` shape. Full dynamic loading (a fresh binary that can hot-load a
provider written in another language) is out of scope.

**Status**: Deferred backlog work. No code yet. See
[docs/proposals/provider-plugin-architecture.md](../proposals/provider-plugin-architecture.md)
for the design sketch. Resume after R1/I010 or an explicit priority-change update.

**Depends on**: #I011-S1 (provides the runtime base_url wiring that S2 will surface via
config)

**Estimate**: L

---

## I012: "Portable Tools"

**Theme**: Reduce Talos' dependency on the host shell environment by shipping a small,
Rust-native POSIX-style tool subset, and make that tool set embeddable through the same
tool/plugin surfaces used by MCP and future local tool extensions.

### #I012-S1: Built-in POSIX basic tools subset

**Description**: Implement a conservative subset of common POSIX/coreutils-like commands
as native `AgentTool` implementations instead of always relying on `bash` and host
executables. This is not a shell replacement. The goal is dependable, cross-platform
building blocks for common agent actions when the external environment is minimal,
locked down, or inconsistent.

**Initial command set**:
- Read-only: `pwd`, `ls`, `cat`, `head`, `tail`, `wc`, `grep`
- Write-capable: `mkdir`, `cp`, `mv`, `rm`

**MUST DO**:
- Implement each command as a structured tool with explicit JSON parameters, not by
  passing a command string through a shell.
- Route every write-capable command through the existing permission pipeline.
- Reuse workspace path validation rules from existing file tools where possible.
- Mark read-only commands as `is_read_only() == true`; write-capable commands must be
  serialised like other write tools.
- Keep semantics intentionally small and documented; unsupported POSIX flags should
  return a clear error instead of silently falling back to host commands.

**MUST NOT DO**:
- Do NOT implement a general shell parser, pipelines, redirects, glob expansion, or
  environment-variable expansion in this story.
- Do NOT bypass sandbox/permission policy because a command is "built-in".
- Do NOT remove the existing `bash` tool; native POSIX tools are the portable default
  building blocks, while `bash` remains the escape hatch for user-approved commands.

**Acceptance Criteria**:
- [ ] ADR recorded before implementation if `ToolPack`, `ToolProvenance`, `AgentTool`, config,
      MCP/RPC listing, or other public boundaries change.
- [ ] `agent.list_tools` exposes the native POSIX subset with stable tool names and schemas.
- [ ] Read-only tools can run without host `ls`, `cat`, `grep`, etc. being present.
- [ ] Write-capable tools require approval under the default permission policy.
- [ ] Tool outputs are deterministic enough for model consumption and session replay.
- [ ] Unit tests cover each command's happy path, path escape rejection, and unsupported
      option errors.
- [ ] `cargo test --workspace` exits 0.

**Depends on**: #I003-S1, #I004-S1, #I004-S6
**Estimate**: L

### #I012-S2: Embeddable tool pack interface

**Description**: Define how built-in native tool packs are registered and exposed so the
same mechanism can support future local plugin-provided tools. The POSIX subset should
be packaged as a first-class tool pack that can be enabled, disabled, listed, and
surfaced through MCP/RPC without special-case code in the agent loop.

**Acceptance Criteria**:
- [ ] ADR records the native tool-pack boundary before public API/provenance/config changes land.
- [ ] A `ToolPack` or equivalent lightweight registration abstraction can register a
      named group of `AgentTool`s into `ToolRegistry`.
- [ ] The POSIX tool pack can be enabled by default and disabled through config for
      constrained deployments.
- [ ] MCP server and JSON-RPC tool listing expose tool provenance so clients can
      distinguish native built-in, native tool-pack, and MCP-remote tools.
- [ ] Tool-pack registration does not introduce dynamic language runtimes or FFI.
- [ ] Plugin-provided tools use the same registration path in the future; no agent-loop
      changes are required for a new local tool pack.

**Depends on**: #I009-S3, #I009-S4, #I009-S5, #I012-S1
**Estimate**: M
