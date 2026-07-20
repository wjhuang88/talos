# TUI-026: Queued Steering Message Display

| Field | Value |
|-------|-------|
| Story ID | TUI-026 |
| Priority | P2 |
| Status | Review (I145, 2026-07-20) |
| Source | Maintainer requirement refined 2026-07-20 |
| Depends on | TUI-032 (composer 多行), TUI-004 (state model) |
| Blocks | — |
| Decision Gate | Cleared by ADR-049: bounded snapshots travel through the existing ordered `UiOutput` stream. |

## 问题

Agent 正在处理一个 turn（包括工具调用）时，后续输入已由
`ConversationEngine::steering_queue` 以 FIFO 形式保存，但用户目前只能看到计数和
短暂提示，无法确认具体排了什么、顺序为何。该队列已经支持多条消息；本需求是让
这个事实成为可靠、可读、不会无限挤占 viewport 的交互体验。

## Goal / Value

用户在处理中的 turn 继续提交多条 steering 消息时，能在 composer 上方确认每条已
入队消息的内容、顺序和总数，并确信它们会在当前完整 turn 结束后按 FIFO 处理。

## Scope

1. **Engine-owned truth**：展示的数据只能来自 `ConversationEngine` 的权威 steering
   队列或其有序快照；TUI 不得维护第二个可漂移的消息队列。
2. **Multiple messages**：处理中的 turn 可接受多条消息。每次提交后 composer 清空，
   立即可继续输入；运行时仍只在权威 `TurnCompleted` 后取出下一条，不得在 tool-use
   等中间事件提前 drain。
3. **Bounded queue preview**：在 composer 上方渲染按 FIFO 编号的 queued section。
   每条显示可读的文本摘要（保留换行语义，按 display width 换行，CJK 不切半个
   glyph）；区域最多占 6 个终端行。超出时必须显示精确的隐藏条目数，例如
   `… and 4 earlier queued messages`，并保留总数。
4. **Viewport contract**：preview 行数必须参与现有 viewport 布局计算。composer 的
   `MAX_COMPOSER_LINES = 10`、滚动 offset、光标位置、slash/credential/approval panel
   的优先级都不得被破坏。窄终端下优先保留至少 1 行 composer；无法同时容纳 preview
   与 composer 时，先压缩 preview 至总数摘要，绝不把 viewport 高度扩张到终端外。
5. **Lifecycle reconciliation**：队列快照在 enqueue、下一条 dequeue、turn cancel/error、
   新会话、resume 和 TUI exit 时更新或清空，不能留下已发送或已丢弃的残影。
6. **Accessibility / terminal behavior**：使用现有主题和纯文本符号；不得依赖鼠标、
   alternate screen 或可变历史 viewport。最终消息历史仍仅进入 terminal scrollback，
   符合 ADR-035。

## Explicit Exclusions

- 不改变 steering 的投递时机、并发模型或完整-turn drain 语义。
- 不新增后台并发 turn、全局 event bus、持久化 steering 队列或跨会话排队。
- 不在本故事中提供编辑、删除、重排或取消已排队消息的控制；这些需独立需求。
- 不将 finalized conversation history 移入 ratatui viewport。

## Architecture / Semver Decision

ADR-049 选择 `UiOutput::SteeringQueueSnapshot`：engine 在既有有序 UI 流中发送最多 8 条
FIFO、每条最多 4 KiB 的预览和精确计数；TUI 只读渲染，不维护镜像队列。公开 enum 新
variant 要求下一个 pre-1.0 minor release 和下游 exhaustive-match 迁移说明。

## Acceptance

- Given 一个正在处理的 turn，When 用户依次提交 A、B、C，Then preview 按 A、B、C
  显示内容及总数 3，composer 每次均可继续输入。
- Given A/B/C 已排队且当前 turn 经过 tool-use，When tool-use 结束但 turn 未完成，Then
  A/B/C 仍完整保留，不得发送 A。
- Given A/B/C 已排队，When 当前 turn authoritative completion 到达，Then 仅 A 被取出并
  发起下一 turn，preview 更新为 B/C；后续 completion 同理保持 FIFO。
- Given 含 CJK、显式换行和超过终端宽度的 queued text，When preview 渲染，Then 不切开
  宽字符，且与 TUI-032 的 display-width 约定一致。
- Given 多条长消息，When preview 达到 6 行上限或终端高度紧张，Then viewport 保持在
  屏幕内、composer 至少可编辑一行、隐藏项数准确且无 cursor/scroll offset 漂移。
- Given turn 被取消、失败或会话切换，When TUI 收到权威状态，Then preview 不显示陈旧
  队列内容。
- Given slash menu、credential input 或 approval dialog 打开，When queue 非空，Then 这些
  模态输入优先级和键盘行为与当前实现一致。

## Required Reads

- `docs/decisions/035-tui-history-scrollback-boundary.md`
- `docs/decisions/039-runtime-event-semantic-single-flow.md`
- `docs/decisions/049-steering-queue-projection-boundary.md`
- `docs/backlog/active/TUI-004-state-model.md`
- `docs/backlog/active/TUI-032-composer-multiline-wrap.md`
- `crates/talos-conversation/src/engine.rs`
- `crates/talos-cli/src/tui_bridge.rs`
- `crates/talos-tui/src/scrollback.rs`
- `crates/talos-tui/src/scrollback_input.rs`
- `crates/talos-tui/src/app.rs`

## Minimum Validation

- Engine FIFO and no-drain-before-completion tests.
- TUI layout tests for one / many / CJK / long queued messages at wide and narrow widths.
- Composer cursor, scroll, slash, credential, approval, cancellation and session-switch regressions.
- `cargo fmt --all -- --check`
- `cargo check --workspace --locked`
- `cargo clippy --workspace --locked -- -D warnings`
- `cargo test --workspace --locked`
