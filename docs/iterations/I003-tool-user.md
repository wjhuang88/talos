# Iteration I003: Tool User

## Scope

Agent 能调用工具执行文件和 Shell 操作。实现 AgentTool trait、bash 工具、文件读写编辑工具、
带工具执行的 turn loop、JSONL 会话日志、交互式 readline 循环。

## Selected Stories

- [x] #I003-S1: AgentTool trait and ToolRegistry
- [x] #I003-S2: Bash tool
- [x] #I003-S3: File read/write/edit tools
- [x] #I003-S4: Turn loop with tool execution
- [x] #I003-S5: JSONL session logging
- [x] #I003-S6: Interactive readline loop

## Acceptance Criteria

- [x] Agent can call bash tool and return stdout/stderr
- [x] Agent can read/write/edit files within workspace root
- [x] Turn loop handles tool_use from LLM, executes, feeds back results
- [x] Read-only tools run concurrently (up to 10), write tools run serially
- [x] Every message logged to JSONL session file
- [x] `talos` (no args) starts interactive readline loop
- [x] `cargo test --workspace` exits 0 (95 tests)
- [x] `cargo clippy --workspace` has no warnings

## Risks

- **LLM tool call format**: Anthropic tool_use 格式需要正确解析。Mitigation: 参考 S2 已有的 SSE 解析。
- **并发工具执行**: 读工具并发 + 写工具串行的调度逻辑。Mitigation: 先实现串行，再优化并发。

## Execution Results

### I003-S1: AgentTool trait and ToolRegistry
- `AgentTool` trait in `talos-core`: object-safe async trait with `name`, `description`, `parameters`, `execute`, `is_read_only`
- `ToolRegistry`: dynamic registration, lookup, JSON Schema validation
- `ToolResult` with `success()`/`error()` helpers
- `tool_parameters!` macro for JSON Schema generation from `schemars::JsonSchema` types
- 16 unit tests passing

### I003-S2: Bash tool
- `BashTool` in `talos-tools`: executes shell commands via `tokio::process::Command`
- Supports shell metacharacters (pipes, redirects, globs) via `sh -c`
- Configurable timeout (default 120s)
- Working directory defaults to project root
- 5 unit tests (echo, invalid command, timeout, metacharacters, working dir)

### I003-S3: File read/write/edit tools
- `ReadTool`: file content with optional line range, binary file detection
- `WriteTool`: create/overwrite files, auto-create parent directories
- `EditTool`: string replacement (first occurrence)
- Path security: all tools validate paths stay within workspace root
- 11 unit tests (read, write, edit, path escape, binary detection)

### I003-S4: Turn loop with tool execution
- Extended `Agent` with `ToolRegistry`
- Tool execution loop: collect tool calls → execute (concurrent read / serial write) → feed back → repeat
- Turn budget: max 50 tool calls per turn
- Doom loop detection: same tool+args 3 times triggers warning
- 11 unit tests (tool loop, concurrency, serial, budget, doom loop, error handling)

### I003-S5: JSONL session logging
- `talos-session` crate: `SessionManager`, `Session`
- JSONL format: `{"type":"message","data":{...}}` per line
- Sessions in `~/.talos/sessions/<project>/<uuid>.jsonl`
- Crash-safe: only last line can be corrupted, invalid lines skipped on read
- 12 unit tests (create, append, read, list, invalid JSON, tool calls)

### I003-S6: Interactive readline loop
- `talos` (no args) starts interactive mode with async readline
- Ctrl+C cancels current turn, double Ctrl+C exits
- Session logging integrated: all messages/events logged to JSONL
- Tools registered: bash, read, write, edit
- `tokio::select!` for combining readline and ctrl_c signals

### Retrospective
- Parallel delegation worked well: S1+S5, then S2+S3, then S4, then S6
- All subagents completed within time limits (no timeouts this iteration)
- Total: 95 tests passing, 0 clippy warnings, ~2000+ lines of new code
- New crates: `talos-tools`, `talos-session`
