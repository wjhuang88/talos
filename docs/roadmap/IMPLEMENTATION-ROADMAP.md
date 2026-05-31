# Talos Implementation Roadmap

## Design Principle

Each iteration is a **vertical slice**: it adds end-to-end functionality and produces a runnable,
testable `talos` binary. No iteration leaves the project in a "foundation-only" state. Every
iteration delivers something a user can actually run and verify.

```
I001 "Project Scaffold"  cargo check --workspace               能编译 ✅
I002 "Hello Agent"       talos "What is 2+2?" -p               能对话 ✅
I003 "Tool User"         talos "list files here"               会调工具 ✅
I004 "Safe Agent"        talos "rm -rf /"                      会被拦住 ✅
I005 "Smart Agent"       Mock LLM + 基础TUI + 压缩 + caching    能压缩
I006 "Data Agent"        TUI工具调用气泡 + 审批 + 会话分支 + SQLite  能搜索
I007 "Skilled Agent"     TUI技能侧栏 + SKILL.md + OpenAI        会技能
I008 "Learning Agent"    TUI进化洞察面板 + 自进化引擎             会学习 ✅
I009 "Extensible Agent"  TUI MCP标记 + Hook + MCP + JSON-RPC   可扩展
I010 "Polished Agent"    TUI打磨 (Nord + markdown + 高级功能)    可发布
```

## I001: "Project Scaffold"

**User can**: Build the workspace and get a binary that prints version/help.

**Scope**:
- Cargo workspace with 5 crate skeletons: core, config, provider, agent, cli
- `talos-core`: Message types (UserMessage, AssistantMessage, AgentEvent), Error hierarchy, serde round-trips
- Each crate has minimal lib.rs/main.rs that compiles
- No business logic — only type definitions and skeleton structure

**Not in scope**: LLM calls, tools, sessions, permissions, TUI

**Verification**:
```bash
cargo check --workspace
# Expected: exits 0, all crates compile
cargo build -p talos-cli
# Expected: produces binary
./target/debug/talos --version
# Expected: prints version
./target/debug/talos --help
# Expected: prints help text
```

## I002: "Hello Agent" (MVP)

**User can**: Run `talos "hello" -p` and get an LLM response streamed to stdout.

**Scope**:
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

## I003: "Tool User"

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

## I004: "Safe Agent"

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

## I005: "Smart Agent"

**User can**: Have long conversations without hitting context limits. Test agent behavior without real LLM costs.

**Scope**:
- **Mock LLM provider** (`talos-provider`): `#[cfg(test)]` module implementing `LanguageModel` trait, configurable response sequences, simulates tool_use/errors/streaming — enables full agent testing without API keys
- **Basic TUI shell** (`talos-tui`): ratatui + crossterm, chat viewport, input area, status bar, Ctrl+C handling, streaming output display
- 5-layer context compaction: budget -> trim -> microcompact -> collapse -> summarize
- Token estimation and budget tracking per turn
- Prompt caching strategy (stable system prompt prefix)
- Basic context files: load `AGENTS.md` from project root

**Not in scope**: Skills, plugins, MCP, evolution, SQLite, session branching

**Verification**:
```bash
# Mock LLM enables testing without API key
cargo test -p talos-provider -- mock_llm
# Expected: mock provider returns preset responses, simulates tool_use and errors

# 50-turn conversation stays within context window
for i in $(seq 1 50); do echo "Turn $i: what is $i + $i?"; done | talos
# Expected: no context overflow errors, compaction kicks in

# TUI launches without blocking during streaming
talos
# Expected: basic TUI shell appears, streaming output renders smoothly
```

## I006: "Data Agent"

**User can**: See tool calls visually in TUI, approve/deny operations, search and fork sessions.

**Scope**:
- **Production-grade event loop** (ADR-004): Single event channel, explicit state machine (AppState), layered cancellation, stdin via std::thread, render/logic separation — foundational infrastructure for all interactive features
- **TUI tool call bubbles**: Visual rendering of tool calls and results in chat viewport
- **TUI approval overlay**: y/a/n approval UI rendered in TUI (replaces CLI prompt)
- JSONL tree-branching sessions (`/fork`, session resume with `-c`)
- **SQLite introduction** (ADR-002): `rusqlite` bundled, session metadata index + FTS5 full-text search
- Session search and resume commands
- Session fork command

**Not in scope**: Skills, plugins, MCP, evolution

**Verification**:
```bash
# Event loop: double Ctrl+C exits immediately
talos
# Press Ctrl+C twice — Expected: exits without hanging

talos "List all .rs files"
# Expected: TUI shows tool call bubble with bash tool execution

talos "rm -rf /tmp/test"
# Expected: approval overlay appears in TUI with y/a/n options

talos -c  # Resume last session
# Expected: previous conversation context restored

talos --search "authentication error"
# Expected: FTS5 returns matching sessions
```

## I007: "Skilled Agent"

**User can**: Define skills in SKILL.md files; agent loads and follows them. Switch between LLM providers.

**Scope**:
- `talos-skill`: SKILL.md parser, progressive disclosure (3 levels)
- **TUI skill index sidebar**: Visual display of loaded skills
- OpenAI provider added (second provider)
- System prompt assembly: identity + skills index + context files
- File-based skill discovery: `.talos/skills/**/SKILL.md`, global `~/.talos/skills/`
- `/model` command for switching providers in TUI

**Not in scope**: Evolution, WASM plugins, MCP

**Verification**:
```bash
mkdir -p .talos/skills/code-review
cat > .talos/skills/code-review/SKILL.md << 'EOF'
# Code Review
Use this skill when asked to review code.
Focus on: security, performance, correctness.
EOF
talos "Review src/main.rs"
# Expected: TUI shows skill index sidebar, agent loads skill, follows review instructions

talos --provider openai "hello"
# Expected: response from OpenAI provider
```

## I008: "Learning Agent"

**User can**: Agent adapts its behavior across sessions via built-in evolution with cognitive feedback (ADR-001).

**Scope**:
- `talos-evolution` (new crate, per ADR-001): 4-phase learning loop with cognitive signals
  - TurnObserver: captures signals (error, correction, satisfaction, inefficiency) with intensity
  - PatternExtractor: rule-based + optional LLM, with contradiction detection
  - KnowledgeStore: SQLite extension (same DB from I006), observations + patterns + conflicts tables
  - BehaviorAdapter: injects high-confidence patterns into system prompt
  - Cognitive feedback: confidence scoring, evidence counting, 70-day half-life time decay
  - Signal-driven extraction triggers (high-pain, conflict, threshold, session end, `/learn`)
- **TUI evolution insights panel**: Visual display of learned patterns
- `/learned` command for transparency
- Evolution data: user preferences, project patterns, error patterns, tool efficiency
- Skill materialization: stable patterns (confidence > 0.8) can become SKILL.md files
- System prompt assembly: identity + evolution context + skills index + context files

**Not in scope**: LLM-assisted pattern extraction tuning, skill hub, WASM plugins, MCP

**Verification**:
```bash
# Evolution learns user preference
# Session 1: user says "no, use functional style"
# Session 2: agent automatically prefers functional patterns
talos -c
# Expected: agent behavior adapted based on learned preferences

# Evolution transparency — inspect cognitive feedback
talos /learned
# Expected: TUI shows evolution insights panel with patterns, confidence scores, evidence counts

# Evolution data inspection
ls ~/.talos/
# Expected: index.db (SQLite with session + evolution tables), sessions/ directory
```

## I009: "Extensible Agent"

**User can**: Extend Talos via hooks, MCP servers, and plugins.

**Scope**:
- **Hook system** (PRIMARY extension mechanism): 20+ extension points (before/after tool call, message transform, etc.) — pure Rust, no WASM dependency
- `talos-mcp`: MCP client (connect to external servers) + MCP server (expose Talos tools)
- `talos-rpc`: JSON-RPC over stdio (basic, no WebSocket yet)
- `talos-plugin`: Plugin runtime — native hooks first, WASM as optional hosting mechanism
- File-based plugin discovery: `.talos/plugins/`
- **TUI MCP tool markers**: Visual indicators for MCP-provided tools
- **TUI plugin status display**: Show loaded plugins and hook activity

**Not in scope**: MCP OAuth, WebSocket RPC, plugin marketplace

**Verification**:
```bash
# Hook plugin changes agent behavior
talos --plugin ./my-plugin "hello"
# Expected: plugin hooks fire, custom behavior observed

# MCP server provides external tools
talos --mcp-config mcp.json "Search the web for Rust 2024 edition changes"
# Expected: agent uses MCP-provided web search tool, TUI shows MCP tool marker

# JSON-RPC control
echo '{"method":"thread/start","params":{"prompt":"hello"}}' | talos --mode rpc
# Expected: JSON response with agent output
```

## I010: "Polished Agent"

**User can**: Use Talos as a daily coding companion with fully polished TUI.

**Scope**:
- **Nord theme**: Full Nord color scheme application across all TUI components (per REFERENCE-PROJECTS.md §19)
- **Markdown rendering**: Rich markdown display in assistant messages (code blocks, lists, headers, links)
- **Diff display**: Visual diff rendering for file changes in chat viewport
- **Steering + follow-up** message queues with ChatComposer queue mode
- **Slash commands**: 10+ commands with fuzzy filtering (`/model`, `/new`, `/resume`, `/fork`, `/compact`, `/diff`, `/status`, `/vim`, `/help`, `/quit`)
- **Guardian AI** sub-agent for auto-approval (with circuit breaker)
- **Headless mode** (`talos exec`) and **SDK mode** (library embedding) via AppServerSession abstraction
- **Exec policy DSL** rules in `.talos/rules/`

**Not in scope**: Desktop app, web UI, mobile, multi-agent side threads (future)

**Verification**:
```bash
talos  # Launch full TUI
# Expected: Nord-themed interactive terminal UI with streaming output, markdown rendering
# Type messages, see tool execution with diff display in real-time
# Ctrl+C to cancel, /help for commands, fuzzy slash command filtering

talos --no-alt-screen  # Inline mode preserving scrollback
# Expected: same UI but in terminal scroll buffer

# Headless mode for CI
talos exec "run tests and fix failures" --max-turns 20
# Expected: autonomous execution with test results

# SDK mode
# Rust code can embed Talos as a library
```

## TUI Evolution Timeline

The TUI grows progressively from I005. Each iteration adds visualization for the features delivered:

| Iteration | TUI 新增能力 | 验证场景 |
|-----------|-------------|---------|
| I005 | 基础壳：聊天视口 + 输入区 + 状态栏 + Ctrl+C + 流式输出 | Mock LLM 测试压缩时 TUI 不卡顿 |
| I006 | **事件循环架构 (ADR-004)** + 工具调用气泡 + 审批覆盖层 (y/a/n) + 会话列表 | 双击 Ctrl+C 立即退出，权限提示在 TUI 中弹出 |
| I007 | 技能索引侧栏 + /model 切换 | 加载 SKILL.md 后显示技能列表 |
| I008 | 进化洞察面板 + /learned 命令 | 自进化后显示学到的模式 |
| I009 | MCP 工具标记 + 插件状态 + Hook 日志 | MCP 工具有特殊标识 |
| I010 | Nord 主题 + markdown + diff + steering + slash + Guardian + headless + DSL | 发布级打磨 |

## Iteration Transition Rules

1. Each iteration must produce `cargo build -p talos-cli` that runs.
2. Each iteration must pass `cargo test --workspace`.
3. No iteration may break functionality from a previous iteration.
4. If an iteration exceeds its timebox, cut scope (stories), not quality (tests).
5. Architecture docs update after each iteration, not during planning.
6. `README.md` must be updated to reflect changes from each iteration (living document).

## Crate Introduction Schedule

Not all 15 crates exist from day one. Crates are created when first needed:

| Iteration | New Crates | Cumulative |
|-----------|-----------|------------|
| I001 | core, config, provider, agent, cli (skeletons) | 5 |
| I002 | (none new; provider and agent get real implementations) | 5 |
| I003 | tools, session | 7 |
| I004 | permission, sandbox | 9 |
| I005 | tui (basic shell) | 10 |
| I006 | (none new; rusqlite dependency introduced) | 10 |
| I007 | skill | 11 |
| I008 | evolution | 12 |
| I009 | plugin, mcp, rpc | 15 |
| I010 | (none new) | 15 |

## Storage Introduction Timeline

Storage complexity grows with each iteration's needs (ADR-002):

| Iteration | Storage Technology | Data Domains |
|-----------|-------------------|--------------|
| I001–I004 | Pure files (zero DB) | JSONL sessions, TOML config |
| I006 | + SQLite (rusqlite bundled) | Session metadata index, FTS5 search |
| I008 | + SQLite extension | Evolution observations, patterns, conflicts |
| Future | Possible Turso migration | Same traits, different engine
