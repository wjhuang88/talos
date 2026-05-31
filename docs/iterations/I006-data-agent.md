# Iteration I006: Data Agent

## Scope

生产级事件循环架构 + TUI 工具可视化 + 审批覆盖层 + 会话分支 + SQLite 搜索。

## Selected Stories

- [ ] #I006-S0: Production-grade event loop architecture (ADR-004)
- [ ] #I006-S1: TUI tool call bubbles + approval overlay
- [ ] #I006-S2: JSONL tree-branching sessions
- [ ] #I006-S3: SQLite session index with FTS5
- [ ] #I006-S4: Session search and resume commands
- [ ] #I006-S5: Session fork command

## Execution Plan

1. S0 (Event loop) — 基础，所有后续 story 依赖
2. S1 (TUI 工具气泡) + S2 (会话分支) — 并行，互不依赖
3. S3 (SQLite) — 依赖 S2
4. S4 (搜索恢复) + S5 (fork 命令) — 并行，依赖 S3

## Acceptance Criteria

- [ ] 事件循环架构实现 (ADR-004)
- [ ] 双击 Ctrl+C 立即退出，无挂起
- [ ] TUI 显示工具调用气泡和审批覆盖层
- [ ] 会话分支、搜索、恢复、fork 功能完整
- [ ] `cargo test --workspace` exits 0
- [ ] `cargo clippy --workspace` has no warnings

## Execution Results

(To be filled after completion)
