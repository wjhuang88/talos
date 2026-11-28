# VALIDATION-004 · validate_project 内部治理检查的 Read 权限

状态：Active / Claimed（#1050 合入 main 后生效）。实施范围为 talos-tools 的 validate_project 权限分类与相应用户文档。

当前 main 在 run 模式对治理检查和 Cargo 检查均生成 Execute facet。用户运行固定的进程内治理检查时会收到命令执行权限请求。期望内部只读检查走常规 Read 规则，主机命令继续按其精确 Execute 资源准入。

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex |
| Work Slice | Internal governance validation permission classification |
| Claimed At | 2026-11-28 |
| Source Issue | None |
| Governance Claim PR | #1050 |
| Authorization Mode | Independent review |
| Authorization Evidence | Maintainer requested recovery of the validation change; independent permission review required |
| Implementation PR | #1051 |
| Last Updated | 2026-11-28 |
| Handoff / Release Condition | Independent permission review and locked tool tests |

Required reads include [VALIDATION-003](VALIDATION-003-model-validation-agile-tools.md) and the executable validation-tool permission tests.

## 验收

1. 构造含治理文档的工作区，运行 governance profile。用计数 host 验证主机命令调用为零，检查每个 facet 为 Read。
2. 验证正常 Read 放行、显式工具 Deny 及 Read Deny；确认拒绝时检查不执行。
3. 覆盖 workspace 和 i076 的主机检查、无效输入和不可用工作区。保留 prepared invocation 的身份绑定、输入匹配、漂移复验和清理行为。
4. 通过 locked 权限与工具测试及独立权限审查；在用户文档说明触发 Execute 的 profile。

原现场已有候选测试。目标分支上的实现、验证和审批依据此任务独立交付。

## 2026-11-28 范围变更

维护者将模型工具限定为跨语言的治理元数据检查。以治理标记发现工具，plan/run 均仅处理 governance，旧 workspace 和 i076 请求返回参数错误。模型入口使用进程内检查，无主机命令执行器。原计划中的 Cargo 命令准入验收由旧参数拒绝和仅有 Cargo.toml 时的发现测试替代。独立权限审查继续核实元数据读取边界。
