# Proposal: exec 多命令/并行/串行/管道支持

**创建时间**: 2026-08-05
**状态**: 提案阶段（待设计评审和 ADR）
**优先级**: P2 — 减少对 bash 工具的依赖，降低权限摩擦

## 动机

当前 `exec` 工具（TOOL-016）只能执行单条命令。任何需要多步操作（`mkdir && cd && cargo init`）、并行执行（同时编译+测试）、或管道（`ps aux | grep talos`）的场景都必须回退到 `bash` 工具。

`bash` 工具的问题：
- 通过 shell 解析（`sh -c`），增加了注入风险面
- 每次调用触发 Execute 权限弹窗（PERM-002 的 "always" 规则粒度太细，实际无法消除重复弹窗）
- 用户每次 cd、每次 cargo check、每次 git status 都要授权一次，完全不可用

**目标**：让 `exec` 工具能覆盖 80%+ 的日常 bash 使用场景，使 `bash` 退化为仅在复杂 shell 脚本、glob 展开、重定向等少数场景下才需要的"兜底"工具。

## 设计方向

### 新增输入结构

```rust
/// 单个执行单元
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ExecStep {
    /// Program name or path.
    pub command: String,
    /// Arguments passed as argv elements. No shell parsing.
    #[serde(default)]
    pub args: Vec<String>,
    /// Optional working directory override for this step.
    #[serde(default)]
    pub cwd: Option<String>,
    /// Optional environment additions for this step.
    #[serde(default)]
    pub env: BTreeMap<String, String>,
}

/// 管道定义：按顺序将前一步的 stdout 通过管道传递给下一步的 stdin
#[derive(Debug, Deserialize, JsonSchema)]
pub struct PipeSpec {
    /// Ordered steps forming the pipe chain: step[0] stdout → step[1] stdin → ...
    pub steps: Vec<ExecStep>,
}

/// 批量执行请求（向后兼容：command 字段仍在顶层，单命令用法不变）
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ExecInput {
    // --- 单命令（向后兼容，保持不变）---
    /// 单命令模式：命令名或路径
    #[serde(default)]
    pub command: Option<String>,
    /// 单命令模式：参数
    #[serde(default)]
    pub args: Vec<String>,
    /// 单命令模式：工作目录
    #[serde(default)]
    pub cwd: Option<String>,
    /// 单命令模式：环境变量
    #[serde(default)]
    pub env: BTreeMap<String, String>,

    // --- 多命令扩展 ---
    /// 批量步骤。当提供时，`command` 字段应被忽略。
    #[serde(default)]
    pub steps: Vec<ExecStep>,
    /// 管道链，每条链内的步骤通过管道连接，链之间可串行或并行
    #[serde(default)]
    pub pipes: Vec<PipeSpec>,

    // --- 执行策略 ---
    /// "sequential"（默认）：按顺序执行，"parallel"：同时启动所有步骤
    #[serde(default = "default_mode")]
    pub mode: ExecMode,

    /// 全局超时。覆盖各步骤独立超时。
    #[serde(default)]
    #[schemars(range(min = 1, max = 600))]
    pub timeout_secs: Option<u64>,
}

#[derive(Debug, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "lowercase")]
pub enum ExecMode {
    #[default]
    Sequential,
    Parallel,
}
```

### 输出结构

```rust
#[derive(Debug, Serialize)]
pub struct ExecResult {
    /// 执行模式
    pub mode: ExecMode,
    /// 每条步骤的结果
    pub results: Vec<StepResult>,
    /// 总耗时
    pub duration_ms: u64,
    /// 汇总退出码（第一个非零码 or 最后一个零码）
    pub exit_code: i32,
}

#[derive(Debug, Serialize)]
pub struct StepResult {
    pub index: usize,
    pub command: String,
    pub args: Vec<String>,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
    pub killed: bool,
}
```

### 使用示例

**串行执行：**
```json
{
  "mode": "sequential",
  "steps": [
    {"command": "mkdir", "args": ["-p", "target/debug"]},
    {"command": "cargo", "args": ["build"], "cwd": "crates/talos-core"}
  ]
}
```

**并行执行：**
```json
{
  "mode": "parallel",
  "steps": [
    {"command": "cargo", "args": ["test", "-p", "talos-core"]},
    {"command": "cargo", "args": ["test", "-p", "talos-config"]}
  ]
}
```

**管道：**
```json
{
  "pipes": [
    {
      "steps": [
        {"command": "ps", "args": ["aux"]},
        {"command": "grep", "args": ["talos"]}
      ]
    }
  ]
}
```

**混合（串行 + 管道）：**
```json
{
  "mode": "sequential",
  "steps": [
    {"command": "echo", "args": ["building..."]},
    {"command": "cargo", "args": ["build"]}
  ],
  "pipes": [
    {
      "steps": [
        {"command": "find", "args": ["src", "-name", "*.rs"]},
        {"command": "wc", "args": ["-l"]}
      ]
    }
  ]
}
```

## 权限边界

- **不改变权限模型**：`exec` 仍然走 `ToolNature::Execute` 权限管线
- 多步骤共享同一个权限决策——如果用户对 `steps` 整体授权一次 `always`，该组合可以在同一资源/目录复用
- 单命令向后兼容路径保持不变
- 管道中的每步仍然是独立的 `tokio::process::Command`，不会引入 shell 解析

## 非目标

- 不引入 shell 解析、glob 展开、重定向（`>`、`2>&1`）
- 不引入条件执行（`&&`、`||` 语义由模型在步骤级别处理）
- 不引入后台任务或持久进程
- 不改变 bash 工具本身
- 不改变 `ExecInput` 的单命令字段——完全向后兼容

## 与 bash 的关系

最终目标状态：

| 场景 | 使用工具 |
|------|---------|
| 单条命令（cargo、git、mkdir 等） | `exec` |
| 多步串行（build → test → lint） | `exec`（sequential） |
| 并行任务（同时测试多个 crate） | `exec`（parallel） |
| 管道（ps → grep、find → wc） | `exec`（pipes） |
| 复杂 shell 脚本、if/for/while、heredoc、glob 展开 | `bash`（兜底） |
| 重定向到文件 | `bash`（兜底） |

## 实现阶段

| 阶段 | 内容 | 依赖 |
|------|------|------|
| M1 | `ExecInput` 扩展、`steps` 串行支持、向后兼容 | TOOL-016 |
| M2 | `mode: "parallel"` 并发执行（tokio::join!） | M1 |
| M3 | `pipes` 管道链（stdout→stdin 连接） | M1 |
| M4 | 权限策略对齐：多步骤 PermissionRule 语义 | PERM-001/PERM-002 |

## 关联项

- `docs/backlog/active/TOOL-016-direct-exec-tool.md` — 当前 exec 实现
- `docs/backlog/active/TOOL-005-bash-streaming-output.md` — bash 工具演进
- `docs/reference/EXEC-TOOL-PERMISSION-POLICY-2026-07-02.md` — exec 权限策略
- `docs/backlog/active/PERM-002-operation-scoped-permissions.md` — 操作级别权限