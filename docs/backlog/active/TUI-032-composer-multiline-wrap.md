# TUI-032: 输入框多行支持（自动换行 + Shift+Enter 手动换行）

| Field | Value |
|-------|-------|
| Story ID | TUI-032 |
| Priority | P1 |
| Status | Review (I142 cross-terminal acceptance remediation, 2026-07-20) |
| Source | Maintainer request recorded 2026-07-19 — 单行输入框超出后内容不可见 |
| Depends on | TUI-010 (slash command menu), TUI-002 (composer/keymap) |
| Blocks | — |

## 问题

当前 TUI 输入框（composer）是单行的。当输入内容超过一行宽度时，超出部分完全不可见——用户不知道自己输入了什么。

两个场景：
1. **长 prompt 粘贴**：粘贴一段较长的 prompt 文本，只有最后几个字可见
2. **手动编辑长内容**：需要查看完整内容时只能盲打

## 需求

### 自动换行

- 输入内容宽度超过 composer 可视区域时，自动折行显示
- composer 高度随行数动态增长（设置合理的最大高度，如 10 行）
- 超过最大高度后出现滚动（或者固定底部行可见 + 顶部行被裁剪，类似终端输入）
- 不能遮盖 slash command menu（menu 应该在 composer 上方）

### Shift+Enter 手动换行

- `Enter`：提交（当前行为不变）
- `Shift+Enter`：在当前光标位置插入换行符（不提交）

### 显示约束

- 换行后的内容在提交时作为多行文本发送给 Agent（即模型收到的是真正的多行字符串）
- scrollback 中用户消息的渲染需支持多行显示（已有 `Paragraph` 渲染，可能需要验证）
- 状态栏/cursor 位置计算需要适配多行

## Codex 参考

Codex 的 TUI 输入框：
- 默认单行，自动折行
- `Shift+Enter` 手动换行
- composer 高度动态增长到约 10 行上限
- Enter 提交

## 实现方向

| 阶段 | 内容 | 预计工作量 |
|------|------|-----------|
| M1 | composer 支持多行存储（`Vec<String>` 或带 `\n` 的 String） | 小 |
| M2 | 自动折行渲染 + 动态高度计算 | 中 |
| M3 | Shift+Enter 手动换行 | 小 |
| M4 | scrollback 用户消息多行渲染适配 + cursor 位置修复 | 中 |
| M5 | 与 slash command menu / approval panel 的 layout 交互测试 | 中 |

## 非目标

- 不改变 Enter 提交的语义
- 不改变 Esc / Ctrl+C 的行为
- 不引入富文本编辑

## I142 Acceptance Remediation (2026-07-19)

The original closeout reused the already-assigned `TUI-025` ID. This document was
renumbered to `TUI-032`; the objective and I142 baseline are unchanged.

Maintainer runtime acceptance in Alacritty found two blockers:

1. `Shift+Enter` was indistinguishable from bare Enter because Talos did not enable
   progressive keyboard enhancement at terminal startup.
2. Composer wrapping was not preserved when the submitted user message moved into
   terminal scrollback because finalized history lines relied on implicit terminal wrap.

Acceptance remediation enables modified-key disambiguation with a paired terminal
push/pop boundary, explicitly wraps finalized scrollback rows while preserving styles
and continuation indentation, and corrects the composer effective width to include its
right padding. The story remains Review until a rebuilt binary passes real Alacritty
acceptance.

Follow-up protocol review on 2026-07-20 found that disambiguation alone deliberately
keeps Enter and Shift+Enter identical. Talos now probes protocol support before enabling
`DISAMBIGUATE_ESCAPE_CODES | REPORT_ALL_KEYS_AS_ESCAPE_CODES |
REPORT_ALTERNATE_KEYS`; unsupported terminals and multiplexers retain normal input and
can use `Ctrl+J` as the portable newline fallback.

## Required Reads

- `crates/talos-tui/src/app.rs` — composer rendering, input handling
- `crates/talos-tui/src/state.rs` — `TuiState` composer字段
- `crates/talos-tui/src/scrollback_input.rs` — 输入相关 scrollback 渲染
- `docs/backlog/active/TUI-010-slash-command-menu.md` — menu 在 composer 上方，layout 约束
- `docs/backlog/active/TUI-002-codex-overhaul.md` — Codex 参考
