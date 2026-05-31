# Iteration I006: Data Agent

## Scope

生产级事件循环架构 + TUI 工具可视化 + 审批覆盖层 + 会话分支 + SQLite 搜索。

## Selected Stories

- [x] #I006-S0: Production-grade event loop architecture (ADR-004)
- [x] #I006-S1: TUI tool call bubbles + approval overlay
- [x] #I006-S2: JSONL tree-branching sessions
- [x] #I006-S3: SQLite session index with FTS5
- [x] #I006-S4: Session search and resume commands
- [x] #I006-S5: Session fork command

## Execution Plan

1. S0 (Event loop) — 基础，所有后续 story 依赖
2. S1 (TUI 工具气泡) + S2 (会话分支) — 并行，互不依赖
3. S3 (SQLite) — 依赖 S2
4. S4 (搜索恢复) + S5 (fork 命令) — 并行，依赖 S3

## Acceptance Criteria

- [x] 事件循环架构实现 (ADR-004)
- [x] 双击 Ctrl+C 立即退出，无挂起
- [x] TUI 显示工具调用气泡和审批覆盖层
- [x] 会话分支、搜索、恢复、fork 功能完整
- [x] `cargo test --workspace` exits 0 (352 tests)
- [x] `cargo clippy --workspace` has no warnings (4 dead_code warnings for unused approval types)

## Execution Results

### I006-S0: Production-grade event loop architecture
- `AppEvent` enum: UserInput, UserInterrupt, AgentTextDelta, AgentToolCall, AgentToolResult, AgentCompleted, AgentError
- `AppState` enum: WaitingForInput → AgentRunning → ShuttingDown
- Single `mpsc::unbounded` event channel
- stdin via `std::thread` (not tokio::spawn)
- Layered cancellation: turn → agent
- `render(&state)` called after every state transition
- 12 unit tests for state machine transitions

### I006-S1: TUI tool call bubbles + approval overlay
- `ToolCallBubble` widget: tool name (nord8), arguments (dimmed), result status (✓/✗)
- `ApprovalOverlay` widget: semi-transparent overlay, y/a/n options
- `ApprovalState` enum: Hidden/Visible with selected choice
- `ApprovalChoice` enum: ApproveOnce/AlwaysApprove/Deny
- 8 unit tests for rendering and key handling

### I006-S2: JSONL tree-branching sessions
- `SessionEntry` with id/parent_id for tree structure
- `SessionBranch` with root_id and entries
- `Session::fork(from_entry_id)` creates new branch
- `list_sessions()` scans `~/.talos/sessions/`
- `resume_session(session_id)` loads existing session
- CLI flags: `-c/--continue`, `-r/--resume`, `--session <id>`
- 15 unit tests for branching and session management

### I006-S3: SQLite session index with FTS5
- `SessionIndex` wrapping `rusqlite::Connection`
- `sessions` table: id, project, created_at, updated_at, message_count
- `messages_fts` FTS5 virtual table for full-text search
- `index_session()`, `search()`, `list_recent()`, `get_session_info()`
- `SearchResult` with session_id, project, snippet, timestamp, rank
- 15 unit tests for indexing and search

### I006-S4: Session search and resume commands
- `--search <query>` flag for full-text search
- `--list` flag to list recent sessions (default limit 20)
- `--limit <n>` flag to customize result count
- Nord theme colors: session ID (nord8), project (nord14), timestamp (nord3)
- Snippet highlighting with nord13 for matched terms
- `highlight_snippet()` converts FTS5 `<b>` markers to ANSI colors
- 6 unit tests for formatting and parsing

### I006-S5: Session fork command
- `--fork <session-id>` CLI flag
- `/fork` command in TUI (parse from user input)
- `AppEvent::ForkSession` and `ForkCompleted` variants
- `ForkInfo` struct: forked_session_id, fork_entry_id, forked_at
- `forks` table in SQLite: source_session_id, forked_session_id, fork_entry_id, forked_at
- `SessionIndex::get_forks()` returns fork relationships
- Status bar shows current branch ID
- 8 unit tests for fork operations and metadata

### Summary
- **Total tests**: 352 (up from 315 in I005, +37)
- **New modules**: `event_loop.rs`, `sqlite.rs`, `colors` module
- **New dependencies**: `rusqlite` with bundled and fts5 features
- **Key achievement**: Production-grade event loop (ADR-004) enables clean Ctrl+C handling and extensible architecture for future TUI features
