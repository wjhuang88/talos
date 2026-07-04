# TOOL-017: exec 多命令/并行/串行/管道支持

| Field | Value |
|-------|-------|
| Story ID | TOOL-017 |
| Priority | P2 |
| Status | Refinement |
| Source | Maintainer request 2026-08-05 — bash 授权频率不可接受，需要让 exec 覆盖大部分 bash 场景 |
| Depends on | TOOL-016 (exec 基础实现) |
| Blocks | — |

## 问题

当前 `exec` 工具（TOOL-016）只能执行单条命令。以下日常操作必须回退到 `bash` 工具：

| 场景 | 示例 | 当前方式 |
|------|------|---------|
| 多步串行 | `mkdir -p target && cargo build` | bash (shell `&&`) |
| 并行执行 | 同时测试多个 crate | bash (shell `&` + `wait`) |
| 管道 | `ps aux \| grep talos` | bash (shell pipe) |

bash 工具每次调用都触发 Execute 权限弹窗。PERM-002 的 "always" 规则按 (命令+cwd+env) 精确指纹匹配，日常开发中目录和命令频繁变化，实际无法消除重复弹窗。用户体验不可接受。

## 目标

让 `exec` 工具原生支持多命令、并行/串行策略和管道，覆盖 80%+ 的日常 bash 场景。`bash` 退化为仅复杂 shell 脚本、glob 展开、文件重定向场景下的兜底工具。

## 范围

- `ExecInput` 新增 `steps: Vec<ExecStep>`、`pipes: Vec<PipeSpec>`、`mode: ExecMode` 字段
- `ExecMode::Sequential`（默认）：按顺序执行，前一步失败时后续步骤行为可配置
- `ExecMode::Parallel`：通过 `tokio::join!` 同时启动所有步骤
- 管道：通过 `stdout→stdin` pipe 连接同一 `PipeSpec` 内的步骤
- 完全向后兼容：现有单命令 `command + args` 用法不受影响
- 权限管线不变：仍走 `ToolNature::Execute`

## 非目标

- 不引入 shell 解析、glob 展开、文件重定向（`>`、`2>&1`）
- 不引入条件执行语法（`&&` / `||` 语义由模型在步骤级别处理）
- 不改变 bash 工具本身
- 不改变权限模型

## 实现阶段

| 阶段 | 内容 | 预计工作量 |
|------|------|-----------|
| M1 | `ExecInput` 扩展 + `steps` 串行支持 + 向后兼容测试 | 小 |
| M2 | `mode: "parallel"` 并发执行 | 小 |
| M3 | `pipes` 管道链 | 中 |
| M4 | 权限策略对齐：`always` 规则对多步骤语义 | 中 |

## Required Reads

- `docs/proposals/exec-multi-command-parallel-pipe.md` — 完整设计提案
- `docs/backlog/active/TOOL-016-direct-exec-tool.md` — 当前 exec 实现
- `docs/backlog/active/TOOL-005-bash-streaming-output.md` — bash 工具
- `docs/reference/EXEC-TOOL-PERMISSION-POLICY-2026-07-02.md` — exec 权限策略
- `docs/backlog/active/PERM-002-operation-scoped-permissions.md` — 操作级别权限
- `crates/talos-tools/src/exec_tool.rs` — 当前实现