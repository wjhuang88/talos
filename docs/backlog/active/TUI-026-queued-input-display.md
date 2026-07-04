# TUI-026: 执行中排队输入的显示问题

| Field | Value |
|-------|-------|
| Story ID | TUI-026 |
| Priority | P2 |
| Status | Refinement |
| Source | Maintainer request 2026-08-05 — 执行中排队逻辑显示有问题 |
| Depends on | TUI-025 (composer 多行), TUI-004 (state model) |
| Blocks | — |

## 问题

当 Agent 正在执行工具（如 `bash` 或 `exec`）时，用户在 composer 中输入的内容进入"排队"状态，等待当前工具执行完毕后再发送。但当前的排队显示逻辑存在问题——具体表现为排队内容显示不正确或让用户困惑。

**Maintainer 备注**: 具体表现需要进一步讨论和复现后再细化，先记录问题。

## 待讨论的点

1. **排队提示**：应该在 composer 或 status bar 中明确告知用户"等待当前任务完成，输入已排队"吗？
2. **排队期间可编辑**：排队后用户可以继续编辑已排队的内容，还是锁定当前排队内容？
3. **取消排队**：应该支持取消排队吗（如 `Esc` 清除已排队内容）？
4. **显示位置**：排队内容是在 composer 中继续显示，还是移到别处（如状态栏提示）？
5. **与多行输入的交互**：TUI-025 的多行 composer 会让排队逻辑更复杂——换行后哪些行属于排队内容？

## Codex 参考

Codex 在 Agent 运行期间：
- composer 保持可用，用户可以继续输入
- 输入的文本会排队，在当前 Agent 回复完成后自动发送
- 没有显式的"排队中"指示器——输入就是正常显示在 composer 里，等 Agent 完成后自动提交

## 实现方向（待讨论确认）

初步想法：
- composer 保持活跃，不锁定
- 排队内容正常显示在 composer 中
- 状态栏或 composer 边缘加一个小的 pending 指示器（如 `⏳`）
- `Esc` 清除排队内容
- 不需要复杂的排队队列——只保留一个"下一个消息"

## 非目标

- 不改变 Agent 的消息投递机制
- 不引入多消息并发队列

## Required Reads

- `crates/talos-tui/src/app.rs` — 输入事件处理、排队逻辑
- `crates/talos-tui/src/state.rs` — `TuiState`
- `docs/backlog/active/TUI-004-state-model.md` — 状态模型
- `docs/backlog/active/TUI-025-composer-multiline-wrap.md` — 多行输入（依赖项）