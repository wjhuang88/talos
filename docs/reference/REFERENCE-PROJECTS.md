# Reference Projects Source Guide

> Implementation reference organized by Talos feature. Each entry links directly to the relevant
> source code so you can study the concrete pattern during implementation.

## Quick Index

| Talos Crate / Feature | Primary Reference | Secondary Reference |
|---|---|---|
| `talos-core` message types | [Codex protocol](https://github.com/openai/codex/tree/main/codex-rs/protocol/src) | [Pi types](https://github.com/earendil-works/pi/tree/main/packages/agent/src) |
| `talos-agent` turn loop | [Codex session/turn](https://github.com/openai/codex/tree/main/codex-rs/core/src/session) | [Claude Code query](https://github.com/yasasbanukaofficial/claude-code/tree/master/src) |
| `talos-provider` streaming | [Codex sampling](https://github.com/openai/codex/tree/main/codex-rs/core/src) | [OpenCode provider](https://github.com/anomalyco/opencode/tree/main/packages/opencode/src/provider) |
| `talos-tools` registry | [OpenCode tool](https://github.com/anomalyco/opencode/tree/main/packages/opencode/src/tool) | [Claude Code tools](https://github.com/yasasbanukaofficial/claude-code/tree/master/src/tools) |
| `talos-sandbox` isolation | [Codex sandboxing](https://github.com/openai/codex/tree/main/codex-rs/sandboxing) | [Codex linux-sandbox](https://github.com/openai/codex/tree/main/codex-rs/linux-sandbox) |
| `talos-permission` rules | [Codex execpolicy](https://github.com/openai/codex/tree/main/codex-rs/execpolicy) | [Claude Code permissions](https://github.com/yasasbanukaofficial/claude-code/tree/master/src/tools/BashTool) |
| `talos-session` storage | [Pi session](https://github.com/earendil-works/pi/tree/main/packages/agent/src/harness/session) | [Hermes state](https://github.com/NousResearch/hermes-agent/tree/main/core) |
| `talos-skill` system | [Pi skills](https://github.com/earendil-works/pi/tree/main/packages/coding-agent/docs) | [Hermes skills](https://github.com/NousResearch/hermes-agent/tree/main/skills) |
| `talos-plugin` extensions | [Pi extensions](https://github.com/earendil-works/pi/tree/main/packages/coding-agent/src/core/extensions) | [OpenCode plugin](https://github.com/anomalyco/opencode/tree/main/packages/plugin/src) |
| `talos-mcp` integration | [OpenCode mcp](https://github.com/anomalyco/opencode/tree/main/packages/opencode/src/mcp) | [Hermes mcp](https://github.com/NousResearch/hermes-agent/tree/main/mcp) |
| `talos-config` loading | [OpenCode config](https://github.com/anomalyco/opencode/tree/main/packages/opencode/src/config) | [Pi settings](https://github.com/earendil-works/pi/tree/main/packages/coding-agent/docs) |
| `talos-cli` interface | [Codex cli](https://github.com/openai/codex/tree/main/codex-rs/cli) | [OpenCode TUI](https://github.com/anomalyco/opencode/tree/main/packages/opencode/src) |
| `talos-rpc` server | [Codex app-server](https://github.com/openai/codex/tree/main/codex-rs/app-server) | [Pi RPC docs](https://github.com/earendil-works/pi/tree/main/packages/coding-agent/docs) |
| Context compaction | [Claude Code query](https://github.com/yasasbanukaofficial/claude-code/tree/master/src) | [Hermes compressor](https://github.com/NousResearch/hermes-agent/tree/main/core) |
| Guardian auto-approval | [Codex guardian](https://github.com/openai/codex/tree/main/codex-rs/core/src/guardian) | |
| Self-evolution | [Hermes learning loop](https://github.com/NousResearch/hermes-agent/tree/main/core) | |
| Multi-agent delegation | [Codex agent control](https://github.com/openai/codex/tree/main/codex-rs/core/src/agent) | [Claude Code AgentTool](https://github.com/yasasbanukaofficial/claude-code/tree/master/src/tools/AgentTool) |

---

## Project Repositories

| Project | Repo | Language | Commit (studied) |
|---|---|---|---|
| **Pi** | <https://github.com/earendil-works/pi> | TypeScript | `main` branch |
| **Claude Code** | <https://github.com/yasasbanukaofficial/claude-code> | TypeScript / Bun | `master` branch |
| **Codex** | <https://github.com/openai/codex> | Rust (96%) | `main` branch, `9f42c89c` |
| **OpenCode** | <https://github.com/anomalyco/opencode> | TypeScript / Effect | `main` branch, `16cae9a3` |
| **Hermes** | <https://github.com/NousResearch/hermes-agent> | Python | `main` branch |
| **Hermes Rust** | <https://github.com/Lumio-Research/hermes-agent-rs> | Rust (community) | `main` branch |

---

## 1. Message Types and Event Protocol

### What to study: How each project defines the message vocabulary exchanged between LLM, tools, and UI.

#### Codex (Rust, direct port)

```
https://github.com/openai/codex/blob/main/codex-rs/protocol/src/protocol.rs
```

- `Op` enum: Submission variants (UserInput, UserTurn, Interrupt, ExecApproval, Compact, Shutdown)
- `EventMsg` enum: Event variants (TurnStarted, AgentMessageDelta, ExecCommandBegin/End, TurnComplete, Error)
- `Submission` struct: Wraps Op with metadata
- Separation: protocol crate defines types only, no logic

```
https://github.com/openai/codex/blob/main/codex-rs/protocol/src/permissions.rs
```

- `AskForApproval` enum: UnlessTrusted, OnRequest, OnFailure, Never, Granular
- `SandboxPolicy` struct: writable roots, network access mode, permission profile

#### Pi (TypeScript, adapt pattern)

```
https://github.com/earendil-works/pi/blob/main/packages/agent/src/types.ts
```

- `AgentMessage` union: extensible via TypeScript declaration merging
- `AgentEvent` discriminated union: agent_start/end, turn_start/end, message_start/update/end, tool_execution_start/update/end
- `AgentTool` interface: label, execute(), executionMode (sequential/parallel)
- `StopReason`: stop, length, toolUse, error, aborted

```
https://github.com/earendil-works/pi/blob/main/packages/ai/src/types.ts
```

- `Message` union: UserMessage, AssistantMessage, ToolResultMessage
- `AssistantMessageEvent` streaming events: start, text_delta, thinking_delta, toolcall_delta, done, error

#### Claude Code (TypeScript, adapt pattern)

```
https://github.com/yasasbanukaofficial/claude-code/blob/master/src/Tool.ts
```

- `Tool<Input, Output, Progress>` type: 30+ fields including checkPermissions, isReadOnly, isConcurrencySafe, isDestructive
- `buildTool(def)` factory: fills safe defaults (isEnabled=true, isConcurrencySafe=false, isReadOnly=false)

---

## 2. Agent Turn Loop

### What to study: The core loop that orchestrates LLM calls and tool execution.

#### Codex (Rust, adopt directly)

```
https://github.com/openai/codex/blob/main/codex-rs/core/src/session/turn.rs
```

- `run_turn()` function: the core turn loop
- Pre-sampling compaction check
- Skills/plugins injection into prompt
- Inner loop: drain pending input -> build prompt -> run_sampling_request -> handle tool calls -> loop or break
- Auto-compact if token limit reached mid-turn
- `SamplingRequestResult { needs_follow_up }` controls loop continuation

```
https://github.com/openai/codex/blob/main/codex-rs/core/src/session/mod.rs
```

- `Codex` struct: holds `tx_sub: Sender<Submission>` and `rx_event: Receiver<Event>`
- SQ/EQ pattern: bounded submission channel (cap=512), unbounded event channel
- Session lifecycle management

#### Pi (TypeScript, adapt dual-loop)

```
https://github.com/earendil-works/pi/blob/main/packages/agent/src/agent-loop.ts
```

- Dual-loop structure: outer loop (follow-up messages) + inner loop (tool calls + steering)
- `agentLoop()` returns `EventStream<AgentEvent, AgentMessage[]>`
- `agentLoopContinue()` for retry without re-adding messages
- Tool execution modes: sequential vs parallel based on `executionMode`

```
https://github.com/earendil-works/pi/blob/main/packages/agent/src/agent.ts
```

- `Agent` class: stateful wrapper with steeringQueue + followUpQueue
- Configurable hooks: convertToLlm, transformContext, beforeToolCall, afterToolCall, prepareNextTurn
- `steer()` injects mid-run, `followUp()` injects after agent stops
- Queue drain modes: "all" or "one-at-a-time"

#### Claude Code (TypeScript, adapt streaming executor)

```
https://github.com/yasasbanukaofficial/claude-code/blob/master/src/query.ts
```

- `queryLoop()`: single async generator, ~1730 lines
- Context pipeline runs before every API call (5 layers)
- `StreamingToolExecutor`: starts executing tools as tool_use blocks arrive during streaming
- `partitionToolCalls()`: consecutive read-only tools batched concurrent, write tools serial
- Terminal reasons: end_turn, max_turns, max_budget, error_during_execution

---

## 3. Tool System

### What to study: Tool trait definition, registry, and execution patterns.

#### Codex (Rust, adopt directly)

```
https://github.com/openai/codex/blob/main/codex-rs/core/src/tools/orchestrator.rs
```

- `ToolOrchestrator`: approval check -> sandbox selection -> execute -> retry on denial
- `run()` method: generic over tool type, handles approval pipeline
- On sandbox denial, retries with escalated permissions

```
https://github.com/openai/codex/blob/main/codex-rs/core/src/tools/parallel.rs
```

- Parallel tool execution with tokio::FuturesOrdered
- Read lock for concurrent execution, write lock for exclusive
- Cancellation tokens per tool invocation

```
https://github.com/openai/codex/blob/main/codex-rs/core/src/tools/runtimes/
```

- `ShellRuntime`: shell command execution
- `UnifiedExecRuntime`: PTY-based execution
- `ApplyPatchRuntime`: verified code patch application

#### OpenCode (TypeScript, adapt registry)

```
https://github.com/anomalyco/opencode/blob/main/packages/opencode/src/tool/tool.ts
```

- `Tool.define(id, init)` factory pattern
- Parameters defined with Effect Schema
- Execute returns `{ title, output, metadata }`

```
https://github.com/anomalyco/opencode/blob/main/packages/opencode/src/tool/registry.ts
```

- Registry with built-in tools: bash, read, write, edit, patch, glob, grep, task, todo, webfetch, skill, question, lsp
- Tool discovery from `{tool,tools}/*.{js,ts}` in `.opencode/` directories
- Tool filtering per-request based on provider capabilities

#### Claude Code (TypeScript, adapt concurrency)

```
https://github.com/yasasbanukaofficial/claude-code/blob/master/src/services/tools/toolOrchestration.ts
```

- `partitionToolCalls()`: batch consecutive read-only tools for concurrent execution
- `runToolsConcurrently()` with max 10 concurrent
- `runToolsSerially()` for write operations
- Context modifiers from concurrent tools queued and applied after batch

```
https://github.com/yasasbanukaofficial/claude-code/blob/master/src/tools/
```

- 42+ tools organized by category: File, Execution, Agent, Web, Planning, Tasks, Team, MCP, IDE, Utility
- Each tool in its own directory with dedicated permissions logic

---

## 4. Sandboxing and Process Isolation

### What to study: OS-level isolation for tool execution.

#### Codex (Rust, adopt directly)

```
https://github.com/openai/codex/blob/main/codex-rs/linux-sandbox/src/bwrap.rs
```

- Bubblewrap sandbox: `--ro-bind / /` (read-only root), `--bind <workspace>` (writable workspace)
- `--unshare-user`, `--unshare-pid`, `--unshare-net` for namespace isolation
- Network modes: Isolated, ProxyOnly (managed TCP proxy), FullAccess
- Seccomp filters applied after bridge setup

```
https://github.com/openai/codex/blob/main/codex-rs/linux-sandbox/README.md
```

- Architecture overview: bwrap for namespace isolation, landlock for filesystem rules
- Protected paths: `.git`, `.codex`, `.agents` always read-only

```
https://github.com/openai/codex/blob/main/codex-rs/process-hardening/
```

- `PR_SET_NO_NEW_PRIVS` applied
- Core dumps disabled
- `ptrace` attach blocked
- Dangerous env vars removed: `LD_PRELOAD`, `DYLD_*`

```
https://github.com/openai/codex/blob/main/codex-rs/sandboxing/src/
```

- Platform-agnostic `SandboxManager` trait
- macOS: sandbox-exec with Seatbelt SBPL profiles
- Windows: Restricted Tokens + ACLs
- Sandbox type selection based on approval policy

#### Hermes (Python, reference for security model)

```
https://github.com/NousResearch/hermes-agent/tree/main/tools/approval.py
```

- `DANGEROUS_PATTERNS` regex list: `rm -rf`, `mkfs`, `DROP TABLE`, `curl | sh`
- Smart approval: auxiliary LLM can auto-approve low-risk matches
- Approval tracking per session, permanent allowlist in config

---

## 5. Permission and Approval System

### What to study: How tools are gated before execution.

#### Codex (Rust, adopt directly)

```
https://github.com/openai/codex/blob/main/codex-rs/core/src/exec_policy.rs
```

- DSL-based rule engine: rules in `~/.codex/rules/*.rules`
- Commands matched against rules to determine approval
- Complex shell features (pipes, redirects, substitution) bypass rules and require explicit approval

```
https://github.com/openai/codex/blob/main/codex-rs/core/src/guardian/review_session.rs
```

- Guardian sub-agent: independent Codex thread reviews tool executions
- Collects conversation history via `GuardianTranscriptCursor`
- Returns `ReviewDecision`: Approved, Denied, TimedOut
- Circuit breaker: blocks after 3 consecutive denials or 10 total recent denials

#### Claude Code (TypeScript, adapt security depth)

```
https://github.com/yasasbanukaofficial/claude-code/blob/master/src/tools/BashTool/bashPermissions.ts
```

- 4-way permission pipeline: stripSafeWrappers -> AST parsing -> checkSemantics -> classifier
- ~20 validators: incomplete commands, shell metacharacters, dangerous variables, command substitution
- Safe env var whitelist: `NODE_ENV`, `GOOS`, `RUST_BACKTRACE`, etc.
- Explicitly excludes: `PATH`, `LD_PRELOAD`, `PYTHONPATH`, `NODE_OPTIONS`

```
https://github.com/yasasbanukaofficial/claude-code/blob/master/src/tools/BashTool/bashSecurity.ts
```

- Tree-sitter AST parsing for semantic analysis
- Pattern detection for injection vectors

#### OpenCode (TypeScript, adapt ruleset model)

```
https://github.com/anomalyco/opencode/blob/main/packages/opencode/src/permission/index.ts
```

- 3 actions: ask, allow, deny
- Wildcard matching with last-rule-wins precedence
- `evaluate(permission, pattern, ...rulesets)` function
- Rulesets merged: defaults -> user config -> agent config -> session config
- Subagent permission inheritance: parent deny rules forwarded

```
https://github.com/anomalyco/opencode/blob/main/packages/opencode/src/config/permission.ts
```

- Permission rule schema definition

---

## 6. Context Compaction

### What to study: How to manage context windows for long conversations.

#### Claude Code (TypeScript, adopt 5-layer pipeline)

```
https://github.com/yasasbanukaofficial/claude-code/blob/master/src/query.ts
```

Look for the context pipeline section in `queryLoop()`:

- **Layer 1: `applyToolResultBudget()`** - Replace large tool results with file references
- **Layer 2: `snipCompactIfNeeded()`** - Remove zombie messages, truncate for headless
- **Layer 3: `microcompact()`** - Strip old tool results by tool_use_id (cache-aware)
- **Layer 4: `contextCollapse()`** - Collapse completed sub-conversations into summaries
- **Layer 5: `autocompact()`** - Fork conversation for LLM-based summarization, circuit breaker at 3 failures

Key insight: light operations first (removal), heavy operations last (summarization). If collapse gets under threshold, autocompact is a no-op.

#### Hermes (Python, adopt iterative compression)

```
https://github.com/NousResearch/hermes-agent/tree/main/core/
```

Look for `ContextCompressor`:

- 4-phase algorithm: prune old results -> determine head/middle/tail boundaries -> generate structured summary -> assemble compressed messages
- **Iterative re-compression**: previous summary passed to LLM with "update" instructions, prevents signal degradation
- Summary template: Goal / Constraints / Progress / Key Decisions / Relevant Files / Next Steps

#### Pi (TypeScript, reference for compaction hooks)

```
https://github.com/earendil-works/pi/blob/main/packages/coding-agent/src/core/compaction/compaction.ts
```

- `estimateContextTokens()` / `calculateContextTokens()` for token estimation
- `prepareCompaction()` decides what to summarize vs keep
- `generateBranchSummary()` for condensed history
- Extensions can customize compaction behavior

---

## 7. Session Storage

### What to study: How conversations are persisted and resumed.

#### Pi (TypeScript, adopt JSONL tree)

```
https://github.com/earendil-works/pi/blob/main/packages/agent/src/harness/session/session.ts
```

- JSONL append-only storage with tree structure
- `SessionEntryBase`: type, id (8-char hex), parentId, timestamp
- `SessionManager`: newSession(), switchSession(), fork(), getEntries(), getLeafId()
- Tree branching via id/parentId linking, in-place branching without new files

```
https://github.com/earendil-works/pi/blob/main/packages/coding-agent/src/core/session-manager.ts
```

- High-level session orchestration
- `NewSessionOptions`: id, parentSession for branching
- Sessions stored in `~/.pi/agent/sessions/` organized by working directory

#### Hermes (Python, adopt SQLite + FTS5)

```
https://github.com/NousResearch/hermes-agent/tree/main/core/
```

- SQLite WAL mode with tables: sessions, messages, messages_fts, messages_fts_trigram
- FTS5 full-text search for cross-session recall
- Write contention: short timeout (1s), retry with jitter (20-150ms, up to 15 retries)
- Session lineage: compression creates child sessions via `parent_session_id`

---

## 8. Skill System

### What to study: How capabilities are defined, loaded, and evolved.

#### Pi (TypeScript, adopt progressive disclosure)

```
https://github.com/earendil-works/pi/blob/main/packages/coding-agent/docs/skills.md
```

- SKILL.md format: Markdown with frontmatter (name, trigger) + instructions body
- Progressive disclosure: Level 0 (name+description in system prompt) -> Level 1 (full content on demand) -> Level 2 (specific reference files)
- Discovery: `~/.pi/agent/skills/`, `.pi/skills/`, parent directories, pi packages
- Agent uses `read` tool to load full SKILL.md when task matches

#### Hermes (Python, adopt self-evolution)

```
https://github.com/NousResearch/hermes-agent/tree/main/skills/
```

- Closed learning loop: complex task (5+ tool calls) -> observe workflow -> abstract into SKILL.md -> index -> auto-load next time
- SKILL.md with YAML frontmatter: name, description, category, version, platform restrictions, fallback skills
- Skill bundles: YAML files grouping multiple skills under one command
- 9 discovery sources: official, skills.sh, well-known endpoints, GitHub, custom taps

```
https://github.com/NousResearch/hermes-agent/tree/main/core/
```

- Skill creation trigger: 5+ tool call threshold
- Generated skills follow agentskills.io open standard
- Performance claim: 40% faster on repeated workflows after 20+ self-created skills

---

## 9. Plugin and Extension System

### What to study: How third-party code extends the runtime.

#### Pi (TypeScript, adopt factory pattern)

```
https://github.com/earendil-works/pi/blob/main/packages/coding-agent/src/core/extensions/types.ts
```

- `ExtensionAPI` interface: registerTool, registerCommand, registerProvider, on(event), events (EventBus)
- Extension events: resources_discover, session_start, session_shutdown, agent_event, tool_call, model_change
- `ExtensionContext`: ui (notify, confirm, select, input, editor, custom), sessionManager, shutdown, reload
- Tool call interception: handlers can return `{ block: true }` to prevent execution

```
https://github.com/earendil-works/pi/blob/main/packages/coding-agent/src/core/extensions/loader.ts
```

- Discovery: `~/.pi/agent/extensions/*.ts`, `.pi/extensions/*.ts`
- Loaded via jiti (TypeScript without compilation)
- `export default function(pi: ExtensionAPI)` pattern

#### OpenCode (TypeScript, adopt hook points)

```
https://github.com/anomalyco/opencode/blob/main/packages/plugin/src/index.ts
```

- 20+ hook extension points:
  - `tool` - register custom tools
  - `chat.message` - intercept messages
  - `chat.params` - modify LLM parameters
  - `chat.headers` - add custom headers
  - `permission.ask` - intercept permission requests
  - `tool.execute.before` / `tool.execute.after` - pre/post execution
  - `tool.definition` - modify tool definitions sent to LLM
  - `shell.env` - inject environment variables
  - `experimental.chat.messages.transform` - transform history
  - `experimental.chat.system.transform` - transform system prompts
  - `event` - listen to all bus events
  - `dispose` - cleanup on shutdown

```
https://github.com/anomalyco/opencode/blob/main/packages/opencode/src/plugin/loader.ts
```

- Plugin loading: spec resolution -> npm install -> compatibility check -> dynamic import -> hook registration
- Auto-installs npm plugins on demand

---

## 10. LLM Provider Abstraction

### What to study: How multiple LLM providers are unified behind a common interface.

#### Codex (Rust, adopt directly)

```
https://github.com/openai/codex/blob/main/codex-rs/core/src/
```

- WebSocket V2 streaming with persistent connections and sticky routing
- SSE fallback within same turn on WebSocket failure
- Prewarm: best-effort WebSocket prewarm before first stream to minimize TTFT
- Client-side state: full conversation history sent with each request

#### OpenCode (TypeScript, adapt provider model)

```
https://github.com/anomalyco/opencode/blob/main/packages/opencode/src/provider/provider.ts
```

- 20+ bundled providers: Anthropic, OpenAI, Azure, Google, Bedrock, OpenRouter, xAI, etc.
- Custom provider loading: npm package with `@ai-sdk/openai-compatible`
- Provider config in `opencode.json`: npm package, options, model list with cost/limits
- Dynamic model discovery via provider loaders

```
https://github.com/anomalyco/opencode/blob/main/packages/opencode/src/provider/schema.ts
```

- Model identification: `provider/model` format
- Model metadata: id, name, cost (input/output per token), limit (context, output), capabilities (tool_call, reasoning)

#### Hermes (Python, adopt fallback chains)

```
https://github.com/NousResearch/hermes-agent/tree/main/core/
```

- 3 API modes with priority resolution: chat_completions, codex_responses, anthropic_messages
- Fallback model: on 429/5xx/401/403, tries `fallback_providers` list in order
- Auxiliary tasks (vision, compression) have independent fallback chains
- Per-call provider/model overrides for subagents

---

## 11. Configuration System

### What to study: How configuration is loaded, merged, and validated.

#### OpenCode (TypeScript, adopt layered merge)

```
https://github.com/anomalyco/opencode/blob/main/packages/opencode/src/config/config.ts
```

- Layered merge with precedence: remote configs -> global (`~/.config/opencode/`) -> CLI override -> project (walking up from cwd) -> `.opencode/` directories -> env variable -> managed preferences
- Arrays concatenated across layers, objects deep-merged
- `${ENV_VAR}` substitution via `ConfigVariable.substitute()`
- Schema validation with Effect Schema

```
https://github.com/anomalyco/opencode/blob/main/packages/opencode/src/config/mcp.ts
```

- MCP server config: type (local/remote), command, environment, timeout, URL, headers, OAuth
- Local MCP: command + args (spawns process)
- Remote MCP: URL + headers + OAuth (Dynamic Client Registration RFC 7591)

#### Pi (TypeScript, reference for context files)

```
https://github.com/earendil-works/pi/blob/main/packages/coding-agent/docs/settings.md
```

- Settings locations: `~/.pi/agent/settings.json` (global), `.pi/settings.json` (project)
- Context files: `AGENTS.md` loaded from `~/.pi/agent/`, parent directories, cwd
- System prompt customization: `.pi/SYSTEM.md` (replace), `APPEND_SYSTEM.md` (append)

---

## 12. MCP Integration

### What to study: Model Context Protocol for external tool and resource access.

#### OpenCode (TypeScript, adopt client implementation)

```
https://github.com/anomalyco/opencode/blob/main/packages/opencode/src/mcp/index.ts
```

- `MCP.Service`: client lifecycle, tool conversion (MCP -> AI SDK Tool), prompt exposure, resource access
- OAuth flow: Dynamic Client Registration (RFC 7591), callback server on port 19876
- Token persistence in `mcp-auth.json`
- MCP prompts become slash commands in TUI

#### Hermes (Python, adopt bidirectional MCP)

```
https://github.com/NousResearch/hermes-agent/tree/main/mcp/
```

- Bidirectional: Hermes is both MCP client AND server
- Other agents (Claude Code, Cursor) can delegate long-running tasks to Hermes via MCP
- Tool and resource exposure to external MCP hosts

---

## 13. Multi-Agent Delegation

### What to study: How sub-agents are spawned and managed.

#### Codex (Rust, adopt directly)

```
https://github.com/openai/codex/blob/main/codex-rs/core/src/agent/control.rs
```

- `AgentControl`: spawns sub-agents, manages inter-agent communication
- Sub-agent config: FullHistory fork or LastNTurns truncation
- Depth limit: max 1 level of sub-agents
- Parallel limit: max 6 concurrent agent threads
- `InterAgentCommunication`: structured messages via `Op::InterAgentCommunication`
- Sub-agents inherit parent's effective configuration (provider, approval, sandbox, cwd)

#### Claude Code (TypeScript, adapt isolation model)

```
https://github.com/yasasbanukaofficial/claude-code/tree/master/src/tools/AgentTool/
```

- Sub-agents: independent tool pool, model, working directory, permissions
- Fresh context window: no inherited conversation history, only task string
- Depth limit: 1 (sub-agents cannot spawn more)
- Built-in agents: general, explore, plan, verification, guide

#### OpenCode (TypeScript, adapt subagent permissions)

```
https://github.com/anomalyco/opencode/blob/main/packages/opencode/src/tool/task.ts
```

- Task tool: subagent_type (general, explore, scout, custom), background mode
- Child session with parentID
- Restricted permissions: parent deny rules forwarded, `task` and `todowrite` denied by default

```
https://github.com/anomalyco/opencode/blob/main/packages/opencode/src/agent/subagent-permissions.ts
```

- Permission inheritance logic for subagents

---

## 14. JSON-RPC Server

### What to study: External client interface for IDE and tool integration.

#### Codex (Rust, adopt directly)

```
https://github.com/openai/codex/blob/main/codex-rs/app-server/README.md
```

- Transports: stdio (stable), WebSocket (experimental), Unix Socket (stable)
- Core primitives: Thread (conversation session), Turn (one interaction), Item (atomic unit)
- Key methods: `thread/start`, `turn/start`, `turn/interrupt`, `config/read`, `config/write`

#### Pi (TypeScript, reference for RPC protocol)

```
https://github.com/earendil-works/pi/blob/main/packages/coding-agent/docs/rpc.md
```

- JSON-over-stdin/stdout with LF-delimited JSONL framing
- Commands -> stdin, Responses + Events -> stdout
- Extension UI request/response sub-protocol

---

## 15. CLI and TUI

### What to study: Terminal user interface patterns, event-driven rendering, full-screen TUI architecture.

#### Codex (Rust, adopt directly — PRIMARY REFERENCE for TUI)

**Architecture**: **Inline-by-default** ratatui TUI with a custom terminal
wrapper (Codex forks `ratatui::Terminal` from MIT-licensed source to add
inline-viewport support; see `codex-rs/tui/src/custom_terminal.rs`). The
viewport sits at the cursor's current y; finalized chat turns are
appended to the scrollback above the viewport via `insert_history_lines`
(`codex-rs/tui/src/insert_history.rs`) using
`SetScrollRegion(1..viewport.top())`. The terminal's native scrollback **is
the transcript**. Alt-screen is opt-in for full-screen sub-views only
(model picker, theme picker, keymap remapper, plugin browser, onboarding)
and is gated by `alt_screen_enabled: bool` (default `true` in sub-views).
80+ source modules in `codex-rs/tui/src/`.

> **Authoritative technical reference for the verified 2026-06-06 source
> read**: `docs/reference/codex-tui-architecture.md` (11 sections,
> file:line evidence, gap analysis, Talos mapping table).

```
https://github.com/openai/codex/tree/main/codex-rs/tui/src/
```

**Core structure**:
- `app.rs` — Core `App` struct, `tokio::select!` event loop
- `app_event.rs` — `AppEvent` enum with 100+ variants (internal message bus)
- `chatwidget.rs` — Main conversation interface (ChatWidget + HistoryCell)
- `bottom_pane/` — Interactive footer: ChatComposer, ApprovalOverlay, SlashCommands
- `history_cell/` — Per-type session record renderers (messages, exec, approvals, patches, MCP, plans). Each cell produces `Vec<Line<'static>>` (lines as data, not widgets); the cell stream is push-appended to scrollback by `ChatWidget`.
- `tui/` — Terminal backend: custom `Terminal` (inline-viewport), `insert_history` (SetScrollRegion push-append), `EventBroker` (stdin pause/resume for `$EDITOR` handoff), `FrameRequester` (actor-style rate-limited redraw), `FrameRateLimiter` (120 FPS cap), `JobControl` (SIGTSTP with alt-screen awareness), `KeyboardModes` (enhancement flags).
- `keymap.rs` — Context-aware keymaps with Vim mode
- `markdown_render.rs` — Markdown → ratatui Lines conversion
- `diff_render.rs` — Git-style diff display

**Key patterns to adopt**:

| Pattern | Implementation | Why it matters |
|---|---|---|
| **AppEvent enum** | 100+ variants, single `mpsc` channel | Validates our `AgentEvent` + broadcast design — no multi-layer bus needed |
| **tokio::select! event loop** | Multiplexes terminal input, frame requests, app server | Clean separation of event sources |
| **TUI never calls agent loop** | Communicates via `AppServerSession` | UI stays responsive during long agent turns |
| **EventBroker for stdin** | `pause/resume` model for external editor | Handles stdin conflict with $EDITOR |
| **FrameRequester** | Rate-limited redraw scheduling | Prevents wasteful renders during streaming |
| **HistoryCell types** | Per-cell-type renderers (messages, exec, approvals, patches, MCP) | Clean rendering abstraction |
| **BottomPane view stack** | Nested overlays (slash command → model picker) | Flexible modal interaction |
| **ApprovalOverlay** | Replaces editor when permission requested | UX pattern for permission flow |
| **ChatComposer** | Multiline input, @-mention file search, $/-mention apps | Rich input experience |
| **Vim mode** | Full modal editing in composer | Power user expectation |
| **`--no-alt-screen`** | Inline mode preserving terminal scrollback | Practical fallback mode |
| **Multi-agent threads** | `/side` side conversations, thread switching with `[`/`]` | Future extensibility |
| **Keymap system** | Context-aware (App/Chat/Composer/Editor/Vim/Pager/List/Approval) | Professional keyboard UX |
| **Slash commands** | 20+ commands with inline args, fuzzy filtering | Rich command interface |
| **Diff display** | Git-style patch rendering in TUI | Essential for file change review |
| **Session resume** | UUID/name/last/interactive selector | Session lifecycle management |

**Critical files**:
```
https://github.com/openai/codex/blob/main/codex-rs/tui/src/app_event.rs          # AppEvent 100+ variants
https://github.com/openai/codex/blob/main/codex-rs/tui/src/app.rs                 # Core event loop
https://github.com/openai/codex/blob/main/codex-rs/tui/src/chatwidget.rs          # Chat UI
https://github.com/openai/codex/blob/main/codex-rs/tui/src/bottom_pane/mod.rs     # Footer pane
https://github.com/openai/codex/blob/main/codex-rs/tui/src/tui/event_stream.rs    # EventBroker stdin
https://github.com/openai/codex/blob/main/codex-rs/tui/src/tui/frame_requester.rs # Frame rate limit
https://github.com/openai/codex/blob/main/codex-rs/tui/src/keymap.rs              # Keymap system
https://github.com/openai/codex/blob/main/codex-rs/tui/src/slash_command.rs       # Slash commands
https://github.com/openai/codex/blob/main/codex-rs/tui/src/markdown_render.rs     # Markdown rendering
https://github.com/openai/codex/blob/main/codex-rs/tui/src/diff_render.rs         # Diff display
https://github.com/openai/codex/blob/main/codex-rs/tui/src/history_cell/mod.rs    # History cell types
https://github.com/openai/codex/blob/main/codex-rs/tui/src/history_cell/exec.rs   # Tool exec rendering
https://github.com/openai/codex/blob/main/codex-rs/tui/src/history_cell/approvals.rs # Approval rendering
https://github.com/openai/codex/blob/main/codex-rs/tui/src/history_cell/patches.rs  # Patch/diff rendering
```

#### OpenCode — Event Bus Architecture (TypeScript, reference for event system design)

**Architecture**: Three-layer event system — Bus (instance Pub/Sub) + GlobalBus (cross-instance) + SyncEvent (event sourcing).

```
https://github.com/anomalyco/opencode/tree/main/packages/opencode/src/bus/
```

**Key patterns to reference**:
- **Bus**: Effect-TS PubSub, typed channels per event type, Instance-scoped lifecycle
  - `bus/index.ts` — Publish, subscribe, subscribeAll with auto-cleanup
  - `bus/bus-event.ts` — `define()` for schema-validated event types
- **GlobalBus**: Node.js EventEmitter singleton for cross-instance bridging
  - `bus/global.ts` — Cross-process event forwarding
- **SyncEvent**: Versioned, persisted events with projectors (CQRS-like)
  - `sync/index.ts` — Event sourcing with SQLite persistence
- **EventV2 + V2Bridge**: Migration layer from old Bus to new core events
  - `event-v2-bridge.ts` — Temporary bridge during migration

**Key lessons (what to avoid)**:
- Three layers + migration bridge caused 7+ regression bugs in v1.15.0-v1.15.12
- Subscription race conditions required special "immediate subscription" fix
- Project-scoped event routing leaked between instances
- Over-engineering for multi-frontend (TUI + Desktop + VS Code + Web) complexity

**Critical files**:
```
https://github.com/anomalyco/opencode/blob/main/packages/opencode/src/bus/index.ts
https://github.com/anomalyco/opencode/blob/main/packages/opencode/src/bus/bus-event.ts
https://github.com/anomalyco/opencode/blob/main/packages/opencode/src/bus/global.ts
https://github.com/anomalyco/opencode/blob/main/packages/opencode/src/sync/index.ts
```

#### OpenCode (TypeScript, reference for UX patterns)

```
https://github.com/anomalyco/opencode/blob/main/packages/opencode/src/session/processor.ts
```

- Doom loop detection: same tool 3+ times with identical args triggers warning
- Step boundaries for agentic iteration tracking
- Token/cost tracking per-step stored in SQLite

---

## 16. Self-Evolution (Hermes Unique)

### What to study: How an agent improves its own capabilities.

```
https://github.com/NousResearch/hermes-agent/tree/main/core/
```

- Closed learning loop: experience -> observe -> abstract -> SKILL.md -> index -> reuse
- Feedback signal system: explicit (5+ tool calls) + implicit (LLM metacognition, user corrections)
- Performance claim: 40% faster after 20+ self-created skills
- Frozen snapshot pattern: memory injected once at session start, mid-session writes update disk but not cached prompt

---

## 17. Cognitive Feedback for Evolution Engine (MenteDB-Inspired Reference Design)

### What to study: Lightweight cognitive feedback mechanisms for I005 evolution engine implementation.

These concrete design patterns were derived from MenteDB's cognitive memory research and evaluated
during ADR-001. They are preserved here as **reference designs for I005**, not current commitments.
The actual implementation may differ based on usage data from I001-I004.

### Signal System

TurnObservation captures nuanced signals beyond binary success/failure:

```rust
// REFERENCE DESIGN — not committed, for I005 implementation reference only
struct TurnObservation {
    tools_used: Vec<ToolUsage>,
    duration_ms: u64,
    outcome: TurnOutcome,  // Success / PartialSuccess / Failed / UserAbandoned
    signals: Vec<Signal>,
}

enum TurnOutcome {
    Success,
    PartialSuccess,
    Failed,
    UserAbandoned,
}

enum SignalKind {
    Error,              // Tool execution error
    UserCorrection,     // User said "no, do it this way"
    Retry,              // Same tool called again (first attempt insufficient)
    Inefficiency,       // 10 steps for a 2-step task
    UserSatisfaction,   // User expressed approval
    TokenWaste,         // Excessive tokens on ineffective operations
}

struct Signal {
    kind: SignalKind,
    intensity: f32,     // 0.0–1.0
    context: String,
    tool_name: Option<String>,
}
```

Signals are the primary learning input. `UserCorrection` and `Error` signals with high intensity
are the most valuable — they indicate the system did something wrong and the user showed the
correct behavior.

### Confidence and Evidence Counting

```rust
// REFERENCE DESIGN — not committed, for I005 implementation reference only
struct Pattern {
    pattern_type: PatternType,
    key: String,
    value: serde_json::Value,
    confidence: f32,              // 0.0–1.0
    evidence_count: u32,          // Supporting observations
    contradicting_count: u32,     // Contradicting observations
    last_reinforced: DateTime<Utc>,
    source_sessions: Vec<Uuid>,   // Traceability
}
```

Confidence computed as:
```
confidence = supporting_evidence / (supporting_evidence + contradicting_evidence)
```

Simple Bayesian-inspired ratio. Avoids complexity of full probabilistic models while providing
a principled way to handle uncertain knowledge.

### Time Decay (Confidence Half-Life)

```rust
// REFERENCE DESIGN — not committed, for I005 implementation reference only
impl Pattern {
    /// Half-life of ~70 days. Unreinforced patterns lose relevance gradually.
    fn effective_confidence(&self, now: DateTime<Utc>) -> f32 {
        let days = (now - self.last_reinforced).num_days().max(0);
        let decay = (-0.01 * days as f64).exp();
        self.confidence * decay as f32
    }
}
```

A preference not seen for 6 months has ~16% of its original confidence. If the user re-expresses
the same preference, confidence is restored quickly via reinforcement. Decay rate should be tuned
with real data during I005.

### Contradiction Detection

```rust
// REFERENCE DESIGN — not committed, for I005 implementation reference only
enum ConflictResolution {
    Override,                // New evidence clearly supersedes old
    KeepBoth,                // Context-dependent (e.g., different project types)
    IncreaseUncertainty,     // Lower confidence, don't change value
    AskUser,                 // Severe conflict, surface to user
}
```

Evidence-strength-based resolution heuristics:
- New evidence count > 3× existing → Override
- Context differs (different project/language) → KeepBoth
- Evidence counts are close → IncreaseUncertainty
- Confidence both > 0.8 but values contradict → AskUser

### Extraction Triggers

```rust
// REFERENCE DESIGN — not committed, for I005 implementation reference only
enum ExtractionTrigger {
    SessionEnd,
    ObservationThreshold(u32),       // N observations accumulated
    HighPainSignal(f32),             // Signal intensity exceeds threshold
    PatternConflictDetected,         // Contradiction during accumulation
    ExplicitUserRequest,             // /learn command
}
```

`HighPainSignal` enables fast learning from failures rather than waiting for session end.

### Source

These patterns were synthesized from:
- MenteDB cognitive memory architecture (confidence scoring, temporal decay, contradiction handling)
- Hermes feedback signal system (explicit + implicit learning triggers)
- ADR-001 design discussion (evaluated and approved in principle, details deferred to I005)

---

## 18. TUI Framework Selection (I009)

### Decision: ratatui + crossterm

Talos TUI will use **ratatui** (v0.30+) as the rendering framework with **crossterm** as the terminal backend.

### Comparison

| Framework | Status | Language | Verdict |
|-----------|--------|----------|---------|
| **ratatui + crossterm** | Active (v0.30, Dec 2025), 20.8k stars, 15.1k dependents | Rust | **Selected** |
| tui-rs | Archived (Aug 2023) | Rust | Dead, don't use |
| termion | Slow maintenance, UNIX-only | Rust | Not cross-platform |
| OpenTUI | Active (v0.3, May 2026) | Zig + TypeScript | Not Rust, unofficial Rust port incomplete |
| Textual | Active | Python | Not applicable |

### Why ratatui + crossterm

1. **Proven in production for exactly this use case**: Codex CLI and Claude Code both use ratatui + crossterm for their chat TUI
2. **Complete widget ecosystem for agent CLI requirements**:

| Agent CLI Component | Ratatui Solution |
|---|---|
| Chat viewport | `Paragraph` + `tui-scrollview` + `ansi-to-tui` |
| Input area | `ratatui-textarea` or `tui-input` |
| Tool call display | `List` / `Table` with custom styling |
| Approval overlay | `tui-overlay` / `tui-popup` |
| Status bar | `Block` widget at bottom of layout |
| Syntax highlighting | `ratatui-code-editor` (Tree-sitter) or `ansi-to-tui` |
| Markdown rendering | `ratatui-markdown` |
| Streaming updates | Immediate-mode re-render on each token |

3. **Pure Rust, no FFI** — satisfies Hard Constraint #1
4. **tokio compatible** — works with async event loop

### Reference: Pi's TUI Architecture

Pi (TypeScript) uses a **fully custom-built TUI** with no third-party framework. Key patterns worth studying:

- **3-layer input pipeline**: `ProcessTerminal` (raw mode + Kitty protocol) → `StdinBuffer` (escape sequence buffering) → `TUI.handleInput()` (middleware chain → focused component)
- **Differential rendering**: Compares previous/new lines, only re-renders changed lines, 16ms frame cap
- **Overlay system**: Stack-based modal overlays with anchor positioning
- **Hardware cursor for IME**: Components embed cursor markers in render output

Pi hand-rolled its TUI because TypeScript lacks a mature equivalent to ratatui. **We don't need to** — ratatui provides all these capabilities out of the box.

### Suggested Dependencies

```toml
[dependencies]
ratatui = "0.30"
crossterm = { version = "0.29", features = ["event-stream"] }
ratatui-textarea = "0.3"
ansi-to-tui = "7"
tui-scrollview = "0.5"
tui-overlay = "0.4"
```

---

## 19. TUI Color Scheme — Nord Theme

### Design Philosophy

Talos TUI uses the **Nord** color palette — an arctic, north-bluish color palette designed by Arctic Ice Studio.
Nord emphasizes a calm, clean, and minimal aesthetic with high contrast for readability.

Official reference: https://www.nordtheme.com/docs/colors-and-palettes

### Color Palette

#### Polar Night (Dark Backgrounds — nord0 to nord3)

| Name | Hex | RGB | Purpose |
|------|-----|-----|---------|
| `nord0` | `#2E3440` | `46,52,64` | Main background, primary area coloring |
| `nord1` | `#3B4252` | `59,66,82` | Status bar, panels, modals, popups, borders |
| `nord2` | `#434C5E` | `67,76,94` | Active line highlight, selection, elevated panels |
| `nord3` | `#4C566A` | `76,86,106` | Indent guides, wrap markers, comments |

#### Snow Storm (Light Text — nord4 to nord6)

| Name | Hex | RGB | Purpose |
|------|-----|-----|---------|
| `nord4` | `#D8DEE9` | `216,222,233` | Elevated UI text, caret, variables, constants |
| `nord5` | `#E5E9F0` | `229,233,240` | Subtle text, hover/active state animations |
| `nord6` | `#ECEFF4` | `236,239,244` | Primary text color, plain text, reserved syntax chars |

#### Frost (Blue Accent — nord7 to nord10)

| Name | Hex | RGB | Purpose |
|------|-----|-----|---------|
| `nord7` | `#8FBCBB` | `143,188,187` | Secondary accent, classes, types, focused elements |
| `nord8` | `#88C0D0` | `136,192,208` | **Primary accent** — function calls, links, active tabs |
| `nord9` | `#81A1C1` | `129,161,193` | Keywords, operators, tags, secondary UI highlights |
| `nord10` | `#5E81AC` | `94,129,172` | Tertiary accents, pragmas, pre-processor |

#### Aurora (Semantic Colors — nord11 to nord15)

| Name | Hex | RGB | Purpose |
|------|-----|-----|---------|
| `nord11` | `#BF616A` | `191,97,106` | **Error** — linter errors, diff deletions, error states |
| `nord12` | `#D08770` | `208,135,112` | Advanced/danger functionality, annotations, decorators |
| `nord13` | `#EBCB8B` | `235,203,139` | **Warning** — linter warnings, diff modifications |
| `nord14` | `#A3BE8C` | `163,190,140` | **Success** — success states, diff additions, strings |
| `nord15` | `#B48EAD` | `180,142,173` | Uncommon functionality, numbers |

### Talos TUI Component Mapping

| Component | Background | Text | Accent | Border |
|-----------|-----------|------|--------|--------|
| **Main background** | `nord0` | — | — | — |
| **Status bar** | `nord1` | `nord4` | `nord8` | `nord2` (top) |
| **Chat viewport** | `nord0` | `nord6` | — | — |
| **User message** | `nord1` | `nord6` | — | `nord2` (left) |
| **Assistant message** | `nord0` | `nord4` | — | — |
| **Tool call indicator** | `nord2` | `nord8` | `nord8` | `nord9` |
| **Tool result (ok)** | `nord0` | `nord14` | `nord14` | `nord14` (left) |
| **Tool result (error)** | `nord0` | `nord11` | `nord11` | `nord11` (left) |
| **Input area** | `nord1` | `nord6` | `nord8` | `nord2` (top) |
| **Input prompt `>`** | — | `nord8` | — | — |
| **Approval overlay** | `nord1` (90% opacity) | `nord6` | `nord9` | `nord9` |
| **Approval allow button** | `nord14` | `nord0` | — | — |
| **Approval deny button** | `nord11` | `nord6` | — | — |
| **Selection highlight** | `nord2` | `nord6` | — | — |
| **Scrollbar** | — | `nord3` | `nord8` (thumb) | — |
| **Diff additions** | — | `nord14` | — | — |
| **Diff deletions** | — | `nord11` | — | — |
| **Diff modifications** | — | `nord13` | — | — |
| **Spinner / loading** | — | `nord8` | — | — |
| **Session ID** | — | `nord3` | — | — |
| **Key hints** | — | `nord3` | `nord9` (key) | — |

### Ratatui Color Constants

```rust
// Talos TUI color scheme — Nord theme
// Reference: https://www.nordtheme.com/docs/colors-and-palettes

use ratatui::style::Color;

/// Polar Night — dark backgrounds
mod polar_night {
    use super::Color;
    pub const NORD0: Color = Color::Rgb(46, 52, 64);   // #2E3440 — main background
    pub const NORD1: Color = Color::Rgb(59, 66, 82);   // #3B4252 — status bar, panels
    pub const NORD2: Color = Color::Rgb(67, 76, 94);   // #434C5E — active line, selection
    pub const NORD3: Color = Color::Rgb(76, 86, 106);  // #4C566A — indent guides, comments
}

/// Snow Storm — light text
mod snow_storm {
    use super::Color;
    pub const NORD4: Color = Color::Rgb(216, 222, 233); // #D8DEE9 — elevated text
    pub const NORD5: Color = Color::Rgb(229, 233, 240); // #E5E9F0 — subtle text, hover
    pub const NORD6: Color = Color::Rgb(236, 239, 244); // #ECEFF4 — primary text
}

/// Frost — blue accent
mod frost {
    use super::Color;
    pub const NORD7: Color = Color::Rgb(143, 188, 187); // #8FBCBB — secondary accent
    pub const NORD8: Color = Color::Rgb(136, 192, 208); // #88C0D0 — primary accent
    pub const NORD9: Color = Color::Rgb(129, 161, 193); // #81A1C1 — keywords, operators
    pub const NORD10: Color = Color::Rgb(94, 129, 172); // #5E81AC — tertiary accent
}

/// Aurora — semantic colors
mod aurora {
    use super::Color;
    pub const NORD11: Color = Color::Rgb(191, 97, 106);  // #BF616A — error
    pub const NORD12: Color = Color::Rgb(208, 135, 112); // #D08770 — danger/advanced
    pub const NORD13: Color = Color::Rgb(235, 203, 139); // #EBCB8B — warning
    pub const NORD14: Color = Color::Rgb(163, 190, 140); // #A3BE8C — success
    pub const NORD15: Color = Color::Rgb(180, 142, 173); // #B48EAD — uncommon
}
```

### Design Principles

1. **Calm, not flashy**: Nord's muted blues create a focused, distraction-free environment
2. **High contrast for readability**: Light text on dark backgrounds with sufficient contrast ratios
3. **Semantic colors are restrained**: Errors/warnings/success use Aurora colors sparingly
4. **Consistent with developer tools**: Nord is widely adopted in terminals, editors, and IDEs
5. **Accessible**: All text/background combinations meet WCAG AA contrast requirements

---

## Usage During Implementation

When implementing a specific Talos feature:

1. Find the relevant section above by Talos crate/feature name.
2. Click the primary reference link to study the source pattern.
3. For Rust implementations, prefer Codex links (same language, direct port).
4. For TypeScript/Python patterns, read for design intent, then reimplement idiomatically in Rust.
5. Record any significant deviations in `docs/decisions/`.
