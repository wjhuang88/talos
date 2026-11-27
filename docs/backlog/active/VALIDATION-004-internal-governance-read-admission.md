# VALIDATION-004 · validate_project 内部治理检查的 Read 权限

状态：Refinement / Unclaimed。实施范围为 talos-tools 的 validate_project 权限分类与相应用户文档。

当前 main 在 run 模式对治理检查和 Cargo 检查均生成 Execute facet。用户运行固定的进程内治理检查时会收到命令执行权限请求。期望内部只读检查走常规 Read 规则，主机命令继续按其精确 Execute 资源准入。

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Internal governance validation permission classification |
| Claimed At | Not applicable |
| Source Issue | None |
| Governance Claim PR | Pending |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-11-26 |
| Handoff / Release Condition | Independent permission review and locked tool tests |

Required reads include [VALIDATION-003](VALIDATION-003-model-validation-agile-tools.md) and the executable validation-tool permission tests.

## 验收

1. 构造含治理文档的工作区，运行 governance profile。用计数 host 验证主机命令调用为零，检查每个 facet 为 Read。
2. 验证正常 Read 放行、显式工具 Deny 及 Read Deny；确认拒绝时检查不执行。
3. 覆盖 workspace 和 i076 的主机检查、无效输入和不可用工作区。保留 prepared invocation 的身份绑定、输入匹配、漂移复验和清理行为。
4. 通过 locked 权限与工具测试及独立权限审查；在用户文档说明触发 Execute 的 profile。

原现场已有候选测试。目标分支上的实现、验证和审批依据此任务独立交付。
