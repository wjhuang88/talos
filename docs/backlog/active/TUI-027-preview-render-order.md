# TUI-027: 预览区渲染顺序错乱

| Field | Value |
|-------|-------|
| Story ID | TUI-027 |
| Priority | P1 |
| Status | Refinement |
| Source | Maintainer request 2026-07-06; [GitHub Issue #27](https://github.com/wjhuang88/talos/issues/27) — 预览区偶尔出现残留文本/顺序错乱 |
| Depends on | TUI-004 (state model), TUI-020 (thinking preview) |
| Blocks | — |

## 问题

TUI 预览区（scrollback 的最后一个 block，显示正在流式输出的 Agent 响应）偶尔出现渲染顺序错乱：

2026-07-06 issue sync: #27 adds a concrete cancellation/new-input residue case. The broader
preview/status polish cluster is tracked in `TUI-028`; this story remains responsible for the
ordering/generation guard and stale-preview correctness boundary.

**已观察到的异常表现**：

1. **残留文本**：`System`、`Cancel` 等系统级文字残留在预览区中，不会被新内容替换
2. **历史区/预览区顺序颠倒**：后续任务的预览内容已经出现在历史区（scrollback 上方固化区域），但前面某个消息的预览内容又出现在预览区，造成视觉上的"时间倒流"
3. **竞态嫌疑**：可能涉及多个流式响应的完成顺序与渲染顺序不一致

**推测根因**（待验证）：

- 多个并发流（tool output 流 + assistant 文本流 + thinking 流）的完成事件到达顺序与期望的渲染顺序不同
- 某个较早启动但较晚完成的流，其预览内容在后续流的内容已经固化到历史区之后才进入预览区
- `AppStreamState` 或 `StreamRenderState` 中缺少"该流是否已经过期"的检查

## 待排查的文件

| 文件 | 作用 | 排查重点 |
|------|------|---------|
| `crates/talos-tui/src/app_stream.rs` | 流式渲染状态机 | 多个流的生命周期管理、完成→固化转换 |
| `crates/talos-tui/src/app.rs` | 主渲染循环 | 事件分发顺序、preview block 与 history block 的边界 |
| `crates/talos-tui/src/scrollback.rs` | scrollback 渲染 | 最后一个 block 作为预览区的逻辑 |
| `crates/talos-conversation/src/engine.rs` | 对话引擎 | 流事件产生的顺序和时机 |
| `crates/talos-tui/src/state.rs` | TuiState | 预览区状态字段的更新时机 |

## 排查思路

1. **日志注入**：在 `app_stream.rs` 的完成事件处理中，对每个 stream 加一个单调递增的 `stream_id`，完成时记录 `[stream_id=N] finalized → history`
2. **预览区守卫**：确认"预览区仅显示当前最新的未完成流"这一约束——已完成的流、已被后续流覆盖的旧流不应再写入预览区
3. **System/Cancel 消息路径**：追踪 System 消息和 Cancel 消息是否不应该经过预览区，而应该直接写入历史区
4. **复现条件**：快速连续发送多个请求（或工具调用产生多个流）时更容易触发

## 实现方向（待细化）

初步想法：
- 每个 stream 分配一个单调递增的 generation ID
- 预览区只响应"当前 generation"的流事件——旧 generation 的事件直接丢弃或写入历史区
- System/Cancel 消息跳过预览区，直接写入历史区
- 在完成事件中加一个"如果 generation 已过期，写入历史区而非预览区"的检查

## 非目标

- 不改变流式传输协议
- 不改变 scrollback 的整体渲染架构

## Required Reads

- `crates/talos-tui/src/app_stream.rs`
- `crates/talos-tui/src/app.rs`（主渲染循环）
- `crates/talos-tui/src/scrollback.rs`
- `crates/talos-conversation/src/engine.rs`
- `docs/backlog/active/TUI-004-state-model.md`
- `docs/backlog/active/TUI-020-thinking-preview-not-history.md`
