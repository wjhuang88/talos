# Frontline Handoff Prompt — Four-Month Unattended Reliability Program

Copy the prompt below verbatim when assigning the goal.

---

你负责在 Talos 仓库中一次性、无值守执行完整的“四个月可靠性、扩展性与记忆质量计划”。这不是自由探索任务；必须严格按已发布基线连续完成全部阶段。不得在任何正常阶段边界停下来等待验收，只有 N200-N250 全部完成并推送后，才向维护者提交一次最终验收请求。

工作目录：`/Users/GHuang/WorkSpace/RustProjects/talos`

开始前完整阅读：

1. `AGENTS.md`
2. `docs/sop/LONG-RUNNING-TASK.md`
3. `docs/sop/START-ITERATION.md`
4. `docs/sop/ITERATION-WORKFLOW.md`
5. `docs/sop/GIT-WORKFLOW.md`
6. `docs/sop/DOC-CHECK.md`
7. `docs/tasks/2026-07-16-four-month-reliability-extensibility-plan.md`
8. `docs/tasks/2026-07-16-reliability-extensibility-execution-package.md`
9. 当前阶段对应的 I135-I139 owner doc 及其中列出的 backlog/ADR/code owners

目标：依次完成 N200-N250：

- N200：Start Gate 与全部非终态迭代盘点；
- N210 / I135：修复 SESSION-006，但不得破坏 ADR-042 durable failed-turn abort 语义；
- N220 / I136：收口本地显式只读 WASM 插件与 `/plugins` 诊断；
- N230 / I137：离线确定性 MEM-009 基准，给出预声明规则下的 Go/No-Go；
- N240 / I138：严格应用 I137 决定，证据不足必须 No-Go 且不改生产行为；
- N250 / I139：独立复验、文档/Issue/残留同步和 pre-1.0 发布就绪报告。

执行权限：这是覆盖 N200-N250 全周期的一次性合并授权。你可以在本仓库内编辑代码、测试、fixture 和文档；运行计划规定的本地命令；在每个阶段通过全部 gate 后创建常规提交并 fast-forward 推送 `origin/main`；对明确映射的 GitHub Issue 发布事实状态评论；仅当 SESSION-006 Complete 时关闭 Issue #36。阶段完成后无需再次申请授权或等待验收，应写入 checkpoint 并立即继续下一阶段。

明确禁止：不得创建或推送 tag，不得发 GitHub Release、crates.io、部署或迁移；不得 force-push；不得新增 dependency、`unsafe`、破坏公共 API、改变 session/TLOG 格式、权限默认、审批语义、sandbox 语义、凭证边界或事件顺序；不得实现远程/自动发现/可写插件、desktop、自动健康恢复、持久任务引擎、多 Agent 或多实例通信；不得声称 REL-002 或 v1.0 达标。任何上述需要都必须停止并记录 blocker，不能自行扩大权限。

工作纪律：

1. 一次只能有一个 Active iteration。前一阶段未 Complete，不得激活下一阶段。
2. 每阶段开始前重新盘点所有 Active、Review、Planned、Blocked 迭代并写入 execution package。
3. 先读真实 API 和现有测试，再修改；不得假设 owner doc 中的期望 API 已存在。
4. 行为变更先建立失败测试；现有行为已经满足时优先补证据/状态，不做无关重构。
5. 每阶段都必须产生独立、可审计的提交并推送。一个逻辑变更一个 commit；提交信息遵循：`type(scope): description (#Ixxx或#story) [model:<实际模型>]`。阶段内可有多个逻辑提交，但不得把两个阶段混在同一提交。
6. 每次 commit 前运行 `git diff --cached`、`git diff --cached --check` 并检查凭证；测试不绿不得提交完成状态或推送阶段完成。
7. 每阶段至少执行 focused tests、真实 runtime/fixture 证据，以及：

```bash
cargo fmt --all -- --check
cargo check --workspace --locked
cargo clippy --workspace --locked -- -D warnings
cargo test --workspace --locked
./scripts/release_preflight.sh
scripts/validate_project_governance.sh .
git diff --check
```

8. owner doc 先更新，Board/README/index 等 derived views 后更新。每个阶段结束把实际命令结果、commit SHA、push 结果、风险、下一精确动作和恢复命令追加到 execution package，并将 checkpoint 提交推送。
9. GitHub 暂时不可用时，代码与本地验证可继续，但必须记录待同步 Issue；不得伪造远端结果。
10. 测试基础设施瞬时失败最多重试两次并记录首次结果；确定性失败必须修复。相同外部 blocker 连续三次且无安全 fallback 时，按 SOP 标记 Blocked 并停止依赖阶段。
11. 不得发送阶段性验收请求、不得在阶段边界等待维护者回复。普通代码缺陷、测试失败、文档漂移或 MEM-009 的 No-Go 都应按既定 fallback 自主闭环并继续。只有 execution package 中列出的硬停止条件可以提前终止整轮；提前终止时只能提交一份包含证据和精确恢复步骤的 Partial/Blocked 报告。

关键判断默认值：

- SESSION-006 只允许保存“已完整形成且规范有效”的交互式消息前缀；不得保存半条 assistant，不得伪造 tool result；I128 durable Runtime 失败 Turn 仍 abort。
- 插件闭环只允许本地显式、只读、无 host call、受 fuel/timeout/output bound 和正常 permission/provenance 管线约束。
- MEM-009 先冻结 fixtures、指标、阈值和 decision rule，再跑最终结果；并列、波动或证据不足一律 No-Go。
- 遇到多种可行实现时选择最小、兼容、可回滚方案；格式/API/dependency/permission/security 语义不清时停止，不得猜测授权。

现在直接从 N200 开始。先确认 `origin/main` 是否已包含本计划；如果共享工作树只有本次规划/状态同步变更且无其他用户改动，先完整复核、运行治理与 diff check，创建独立 docs 规划提交并推送，作为 publication baseline。然后记录 `git status -sb`、最近提交、`origin/main` 同步状态、固定 Rust 版本、locked metadata、治理验证和 release preflight 的实际结果；确认没有被绕过的非终态 owner 后，仅激活 I135。不要先实现 I136-I139，也不要等待日历月份；前一 gate 通过即可继续下一阶段。

最终交付必须在四个月任务全部实现、验证、提交并推送后一次性给出，包含：每阶段 commits 与 push 证据、逐项 acceptance 映射、真实 runtime/fixture 结果、完整 locked 验证、所有 owner/Issue 同步、残留项及恢复说明。最终只给出 release-readiness 建议，不执行发版。

---
