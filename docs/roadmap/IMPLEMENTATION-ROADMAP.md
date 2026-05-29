# Talos Implementation Roadmap

## Design Principle

Each iteration is a **vertical slice**: it adds end-to-end functionality and produces a runnable,
testable `talos` binary. No iteration leaves the project in a "foundation-only" state. Every
iteration delivers something a user can actually run and verify.

```
I001 "Hello Agent"      talos "What is 2+2?" -p               能对话
I002 "Tool User"        talos "list files here"               会调工具
I003 "Safe Agent"       talos "rm -rf /"                      会被拦住
I004 "Smart Agent"      talos 长对话50轮不炸上下文              能压缩
I005 "Skilled Agent"    talos 加载SKILL.md自动遵循             会学习
I006 "Extensible Agent" talos 加载WASM插件 + MCP工具           可扩展
I007 "Polished Agent"   talos 全功能TUI交互 + 多模式           可发布
```

## I001: "Hello Agent" (MVP)

**User can**: Run `talos "hello" -p` and get an LLM response streamed to stdout.

**Scope**:
- Cargo workspace with 5 crates: core, config, provider, agent, cli
- `talos-core`: Message types (UserMessage, AssistantMessage, AgentEvent), Error hierarchy
- `talos-config`: Minimal config: API key from env, model name, provider selection
- `talos-provider`: Streaming trait + Anthropic Messages API (SSE)
- `talos-agent`: Basic turn loop (no tools), async with CancellationToken
- `talos-cli`: Print mode (`-p`), stdin pipe, streaming output to stdout

**Not in scope**: Tools, sessions, permissions, sandbox, TUI, multiple providers

**Verification**:
```bash
ANTHROPIC_API_KEY=sk-... cargo run -p talos-cli -- "Explain Rust ownership" -p
# Expected: streaming text response to stdout
echo "What is 2+2?" | cargo run -p talos-cli -- -p
# Expected: "4" (or a short response)
```

## I002: "Tool User"

**User can**: Ask the agent to perform file and shell operations.

**Scope**:
- `talos-tools`: AgentTool trait, ToolRegistry, read/write/edit/bash tools
- `talos-agent`: Turn loop with tool execution, concurrent read tools, serial write tools
- `talos-session`: JSONL append-only session log (no branching yet)
- `talos-cli`: Interactive mode (basic readline loop, no TUI)

**Not in scope**: Sandbox, permissions, compaction, skills, plugins

**Verification**:
```bash
talos "List all .rs files in the current directory"
# Expected: agent calls bash tool, returns file list
talos "Create a file called hello.txt with content 'world'"
# Expected: agent calls write tool, file created
```

## I003: "Safe Agent"

**User can**: Trust that dangerous operations are caught and contained.

**Scope**:
- `talos-sandbox`: Bubblewrap (Linux), sandbox-exec (macOS), restricted filesystem
- `talos-permission`: Permission rules engine, allow/deny patterns, interactive approval prompt
- Process hardening basics (PR_SET_NO_NEW_PRIVS equivalent, env sanitization)
- Approval pipeline: rule match -> ask user -> execute or deny

**Not in scope**: Guardian AI auto-approval, full DSL rules, network sandboxing

**Verification**:
```bash
talos "Delete all files in /tmp"
# Expected: permission prompt appears, user must approve
talos "Read /etc/shadow"
# Expected: sandbox blocks access outside workspace
# Agent cannot escape workspace root via symlinks or ../
```

## I004: "Smart Agent"

**User can**: Have long conversations without hitting context limits. Search across session history.

**Scope**:
- 5-layer context compaction: budget -> trim -> microcompact -> collapse -> summarize
- JSONL tree-branching sessions (`/fork`, session resume with `-c`)
- **SQLite introduction** (ADR-002): `rusqlite` bundled, session metadata index + FTS5 full-text search
- `SessionStore` trait for storage abstraction (future engine migration path)
- Token estimation and budget tracking per turn
- Prompt caching strategy (stable system prompt prefix)
- Basic context files: load `AGENTS.md` from project root

**Not in scope**: Skills, plugins, MCP, TUI, evolution

**Verification**:
```bash
# 50-turn conversation stays within context window
for i in $(seq 1 50); do echo "Turn $i: what is $i + $i?"; done | talos
# Expected: no context overflow errors, compaction kicks in

talos -c  # Resume last session
# Expected: previous conversation context restored

talos --search "authentication error"
# Expected: FTS5 returns matching sessions
```

## I005: "Learning Agent"

**User can**: Define skills in SKILL.md files; agent adapts its behavior across sessions via
built-in evolution with cognitive feedback (ADR-001).

**Scope**:
- `talos-skill`: SKILL.md parser, progressive disclosure (3 levels)
- `talos-evolution` (new crate, per ADR-001): 4-phase learning loop with cognitive signals
  - TurnObserver: captures signals (error, correction, satisfaction, inefficiency) with intensity
  - PatternExtractor: rule-based + optional LLM, with contradiction detection
  - KnowledgeStore: SQLite extension (same DB from I004), observations + patterns + conflicts tables
  - BehaviorAdapter: injects high-confidence patterns into system prompt
  - Cognitive feedback: confidence scoring, evidence counting, 70-day half-life time decay
  - Signal-driven extraction triggers (high-pain, conflict, threshold, session end, `/learn`)
- Evolution data: user preferences, project patterns, error patterns, tool efficiency
- File-based skill discovery: `.talos/skills/**/SKILL.md`, global `~/.talos/skills/`
- Skill materialization: stable patterns (confidence > 0.8) can become SKILL.md files
- OpenAI provider added (second provider)
- System prompt assembly: identity + evolution context + skills index + context files

**Not in scope**: LLM-assisted pattern extraction tuning (defer to I006), skill hub, WASM plugins, MCP

**Verification**:
```bash
# Skills work
mkdir -p .talos/skills/code-review
cat > .talos/skills/code-review/SKILL.md << 'EOF'
# Code Review
Use this skill when asked to review code.
Focus on: security, performance, correctness.
EOF
talos "Review src/main.rs"
# Expected: agent loads skill, follows review instructions

# Evolution learns user preference
# Session 1: user says "no, use functional style"
# Session 2: agent automatically prefers functional patterns
talos -c
# Expected: agent behavior adapted based on learned preferences

# Evolution transparency — inspect cognitive feedback
talos /learned
# Expected: lists patterns with confidence scores, evidence counts, and last-reinforced dates

# Evolution data inspection
ls ~/.talos/
# Expected: index.db (SQLite with session + evolution tables), sessions/ directory
```

## I006: "Extensible Agent"

**User can**: Extend Talos via WASM plugins and MCP servers.

**Scope**:
- `talos-plugin`: WASM runtime (wasmtime), sandboxed execution
- Hook system: 20+ extension points (before/after tool call, message transform, etc.)
- `talos-mcp`: MCP client (connect to external servers) + MCP server (expose Talos tools)
- `talos-rpc`: JSON-RPC over stdio (basic, no WebSocket yet)
- File-based plugin discovery: `.talos/plugins/*.wasm`

**Not in scope**: MCP OAuth, WebSocket RPC, plugin marketplace

**Verification**:
```bash
# WASM plugin changes agent behavior
talos --plugin ./my-plugin.wasm "hello"
# Expected: plugin hooks fire, custom behavior observed

# MCP server provides external tools
talos --mcp-config mcp.json "Search the web for Rust 2024 edition changes"
# Expected: agent uses MCP-provided web search tool

# JSON-RPC control
echo '{"method":"thread/start","params":{"prompt":"hello"}}' | talos --mode rpc
# Expected: JSON response with agent output
```

## I007: "Polished Agent"

**User can**: Use Talos as a daily coding companion with full TUI.

**Scope**:
- **TUI layout design** (#I007-S0): Design document for layout, components, interaction model, keymaps
  before implementation (reference: Codex `codex-rs/tui/src/` 80+ modules)
- **Full TUI** (#I007-S1): ratatui-based, HistoryCell rendering, ChatComposer, status bar, approval overlay,
  diff display, `--no-alt-screen` inline mode, frame rate limiting, markdown rendering
- **Steering + follow-up** message queues with ChatComposer queue mode
- **Slash commands**: 10+ commands with fuzzy filtering (`/model`, `/new`, `/resume`, `/fork`, `/compact`,
  `/diff`, `/status`, `/vim`, `/help`, `/quit`)
- **Three execution modes**: interactive (TUI), headless (`exec`), SDK (library)
  via AppServerSession abstraction (Codex pattern: TUI never calls agent loop directly)
- Session management commands
- Guardian AI sub-agent for auto-approval (with circuit breaker)
- Full exec policy DSL rules in `.talos/rules/`

**Not in scope**: Desktop app, web UI, mobile, multi-agent side threads (future)

**Verification**:
```bash
talos  # Launch full TUI
# Expected: interactive terminal UI with streaming output
# Type messages, see tool execution in real-time
# Ctrl+C to cancel, /help for commands

talos --no-alt-screen  # Inline mode preserving scrollback
# Expected: same UI but in terminal scroll buffer

# Headless mode for CI
talos exec "run tests and fix failures" --max-turns 20
# Expected: autonomous execution with test results

# SDK mode
# Rust code can embed Talos as a library
```

## Iteration Transition Rules

1. Each iteration must produce `cargo build -p talos-cli` that runs.
2. Each iteration must pass `cargo test --workspace`.
3. No iteration may break functionality from a previous iteration.
4. If an iteration exceeds its timebox, cut scope (stories), not quality (tests).
5. Architecture docs update after each iteration, not during planning.
6. `README.md` must be updated to reflect changes from each iteration (living document).

## Crate Introduction Schedule

Not all 14 crates exist from day one. Crates are created when first needed:

| Iteration | New Crates | Cumulative |
|-----------|-----------|------------|
| I001 | core, config, provider, agent, cli | 5 |
| I002 | tools, session | 7 |
| I003 | sandbox, permission | 9 |
| I004 | (none new; rusqlite dependency introduced) | 9 |
| I005 | skill, evolution | 11 |
| I006 | plugin, mcp, rpc | 14 |
| I007 | (none new) | 14 |

## Storage Introduction Timeline

Storage complexity grows with each iteration's needs (ADR-002):

| Iteration | Storage Technology | Data Domains |
|-----------|-------------------|--------------|
| I001–I003 | Pure files (zero DB) | JSONL sessions, TOML config |
| I004 | + SQLite (rusqlite bundled) | Session metadata index, FTS5 search |
| I005 | + SQLite extension | Evolution observations, patterns, conflicts |
| Future | Possible Turso migration | Same traits, different engine |
