# Iteration I004: Safe Agent

## Scope

Agent 执行工具时受到权限控制和沙箱隔离。实现权限引擎、审批提示、Bubblewrap/Seatbelt 沙箱、进程加固，以及完整的工具执行管线集成。

## Selected Stories

- [x] #I004-S1: Permission rules engine
- [x] #I004-S2: Interactive approval prompt
- [x] #I004-S3: Bubblewrap sandbox (Linux)
- [x] #I004-S4: sandbox-exec (macOS)
- [x] #I004-S5: Process hardening basics
- [x] #I004-S6: Tool execution pipeline integration

## Acceptance Criteria

- [x] Permission engine evaluates tool calls against rules (allow/deny/ask)
- [x] Interactive approval prompt for `Ask` decisions
- [x] Bubblewrap sandbox restricts filesystem and network on Linux
- [x] Seatbelt sandbox restricts filesystem and network on macOS
- [x] Process hardening: env sanitization, resource limits, core dump prevention
- [x] Tool execution pipeline: permission → sandbox → execute
- [x] `cargo test --workspace` exits 0 (167 tests)
- [x] `cargo clippy --workspace` has no warnings

## Risks

- **沙箱不可用**: bwrap/sandbox-exec 可能未安装。Mitigation: 优雅降级，不强制要求沙箱。
- **macOS 路径符号链接**: `/tmp` 是 `/private/tmp` 的符号链接。Mitigation: 使用 canonical path。
- **权限规则优先级**: 自定义规则需要优先于默认规则。Mitigation: `load_from_config` 前置自定义规则。

## Execution Results

### I004-S1: Permission rules engine
- `talos-permission` crate: `PermissionEngine`, `PermissionRule`, `PermissionDecision`
- 默认规则集: read/list → Allow, write/edit/bash → Ask
- glob 路径模式匹配, 首匹配胜出
- 自定义 serde 反序列化支持灵活的配置 JSON 格式
- 24 tests passing

### I004-S2: Interactive approval prompt
- `ApprovalPrompt` in `talos-cli`: 格式化提示 + 单字符输入 (y/a/n)
- 'a' (always approve) 自动添加 Allow 规则到引擎
- print mode 下 Ask 默认 Deny
- 9 tests passing

### I004-S3+S4: Sandbox (Linux + macOS)
- `talos-sandbox` crate: `SandboxProvider` trait + 平台实现
- Linux: Bubblewrap (`bwrap`) — ro-bind /, bind workspace, unshare-net
- macOS: Seatbelt — 动态生成 profile, canonical paths, process* 权限
- 优雅降级: 沙箱不可用时返回 `NotAvailable`
- 26 tests passing

### I004-S5: Process hardening
- `ProcessHardening` struct: CPU/内存限制, 核心转储禁用, 环境变量清理
- 移除危险 env vars: LD_PRELOAD, DYLD_INSERT_LIBRARIES 等
- Unix rlimit 调用 (带 unsafe 文档)
- 12 tests passing

### I004-S6: Pipeline integration
- `Agent::with_security()` 构造函数接受权限引擎和沙箱
- 工具执行管线: permission.evaluate() → sandbox (bash only) → execute
- Ask 在 agent 层默认 Deny, CLI 层处理交互审批
- 27 tests passing

### Retrospective
- 并行委派效果良好: S1 和 S3+S4 并行, S2 和 S5 并行
- macOS 沙箱调试耗时: `/tmp` 符号链接和 `process*` vs `process-exec*` 问题
- 总测试数从 133 增长到 167 (+34)
