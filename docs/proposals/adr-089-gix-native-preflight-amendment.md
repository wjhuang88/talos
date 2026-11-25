# ADR-089 限定修订提案：旧版 Git 不支持的预检动作兼容替换

> **2026-11-23 publication boundary: non-binding Proposed documentation ONLY.** Merging this document under single-maintainer rules does not accept ADR-089 authority changes, enable any new runtime permission, or satisfy independent Git security/API review for a later security-policy amendment. The accepted ADR remains unchanged.
> 2026-11-19 维护者范围修正。**Status: Proposed — NOT Accepted / NOT implementation authority.**
> Source: [Issue #919](https://github.com/wjhuang88/talos/issues/919); decision text in [ADR-089](../decisions/089-structured-git-effect-authorization.md).
> 本修订**取代** 2026-11-12 的“所有预检都必须 gix 化、彻底禁止 CLI”方案，不修改已经生效的 ADR 条款。

## 当前目标

**保留兼容旧版 Git 的现有受控 Git CLI 预检；仅替换旧版不支持的具体查询动作。** 系统 Git 仍负责最终经过审批的写操作。拒绝自动升级为全量 gix 重写、Git 版本硬编码分支、环境变量放宽或未经审批的新 CLI 探测。

2026-11-21 [Windows 原生报告](https://github.com/wjhuang88/talos/blob/374df343ed4c089e636676a23cc14ac0ff60278f/docs/reference/WINDOWS-VALIDATION-2026-11-21-I324.md) 在官方 SHA-256 固定的 MinGit 2.36.1 与选定的 Scoop Git 2.36.1 上逐命令复现：`git var GIT_ATTR_SYSTEM/GLOBAL`、`GIT_CONFIG_SYSTEM/GLOBAL`、`GIT_SHELL_PATH` **五项均返回 129 / unsupported-variable**，`git check-attr --source=HEAD` 返回 **129 / unsupported-option**。普通 `git check-attr` 和 `--cached` 各自可用，且语义不同。**六项为候选最小替换范围**，不能推导所有 `git var`、`git config`、`git --exec-path`、ref 检查都不可用；11 月 17 日的配置/receiver 失败仍需单独判根因。固定脚本在隔离的 HOME/config 下执行，**不代表**原始 Scoop 环境产品验收；该历史报告中的 17 MiB Checkout/Commit 仍在 `GIT_INSTALL_ROOT` 安全门禁处失败；之后独立的 GIT-ENV-001 / #1007 已在 main 合并，修复了受控 Scoop 环境准入，但原生 Git 2.36.1 仍阻塞于本 Issue 的不兼容属性查询。

## 保留哪些 CLI

- 已经通过 ADR-089 接受的固定 `git config --show-origin --null --list --includes`、`git --exec-path`、`git check-attr`、`git rev-parse` 等命令，在其受支持的版本及既有合法权限边界内继续执行。
- 保留现有独立发现审批、只读命令限制、已批准的 Command/Path/Remote facets、特定可执行文件身份、冻结环境、超时和输出预算、一次性令牌及变更后重验。
- 不将继续执行已批准的 CLI 误判为违规；检查重点是**不兼容命令的替代处理是否新增未授权文件读取或隐藏子进程**。
- 如果其他命令也被原始 Git 2.36 证据证实不支持，再逐一扩充替代范围并补充 ADR 与测试；不能根据版本号和安装路径推测支持情况。


**Added older-Git command gap (Git 2.40 release evidence):** `git check-attr --source=<tree-ish>` was introduced in 2.40. Talos uses `--source=<target.commit>` to predict target-tree filter attributes; it separately queries `--cached` against the current index. Those sources can disagree, so **retain existing compatible normal/cached `git check-attr` CLI but substitute only the unsupported target-tree `--source` option** using explicitly admitted tree and `.gitattributes` bytes, with exact Git precedence and mutation revalidation. [Git 2.40 official release notes](https://github.com/git/git/blob/master/Documentation/RelNotes/2.40.0.adoc). This does not authorize a new temporary-index trick, hidden helper command, `GIT_INDEX_FILE` redirection, source omission, generic filesystem reads or full gix replacement. The Draft #928 supplied-buffer candidate is not native Windows Git 2.36 product acceptance.

## 只替换哪些能力

首先以经审计的 gix/Talos-native 路径或获单独批准的受控替代方案处理已实测不支持的六种具体形式：五个 `git var` 变量查询与目标树 `check-attr --source`。配置根、shell 路径和属性根必须分别设计与审查，保留受支持的 `git check-attr --cached`、`git config --show-origin --null --list --includes`、`git --exec-path`。要求：选定系统 Git 的来源语义一致；系统/全局属性、缺失/空路径、HOME/XDG、include 与工作树覆盖关系均有证据；每个新文件/祖先元数据只在精确 Read/metadata admission 后读取；快照变化必须在写入前拒绝。

高层 gix 初始化/默认路径 API 可能隐式调用 Git、扫描未授权路径或使用进程级缓存，**仍不得**作为隐式授权通道。必要的可信编译默认来源证明仅针对需要它的**替代动作**；不得将它扩散为所有正常 CLI 操作的普遍阻塞条件。如果某项替代无法证明来源，明确报告该项不受支持、拒绝写入，而不是猜路径或跳过 attributes。

`GIT_INSTALL_ROOT` 环境影响另行处理；既有对危险 `GIT_*` 的拒绝不变化。

## 审查与验收

1. 按命令逐项确认旧版失败，而非按整个预检阶段假定不能运行 CLI；记录 native Git 2.36.1 和新版 Git exit code/stdout/stderr。
2. 对**新增替代路径**验证没有未经授权的外部进程、隐式 I/O、权限扩张和源遗漏；不要求保留的合法 CLI 零调用。
3. 比较属性根和过滤器预测，覆盖缺失变出现、配置/可执行文件替换、Unicode/空格、不同安装和 17 MiB Checkout/Commit。
4. **D1 — 架构设计提案与受保护的权限决策分离**：本 #931 只合并 **Proposed / NOT Accepted** 的非生效设计资料，且不修改 Accepted ADR-089。未来任何真正涉及 Path/Command/Remote 等权限边界的 ADR 修订，仍须独立 Git 权限/安全/API 审查、维护者对其精确文本正式接受、精确提交 CI 与治理检查。单维护者流程不能代替受保护安全审查。
5. **D2 — 生产实现验收**：针对每一条替代路径证明选定 writer 的真实来源、精确授权的配置/属性/树对象与祖先 Read、缺失源证据、无隐藏子进程、失效重验和权限效果；原始 Scoop 环境的 Git 2.36.1 与新版 Git 17 MiB Prepared Checkout/Commit、拒绝/过期测试和实施 PR 精确提交 CI 均完成，且获得必要独立审查与最终合并授权后才允许生产启用和关闭 #919。**D2 不是纯架构 ADR 在 D1 阶段的循环先决条件，D1 也不会自动授予任何运行时新权限。**

这是**重新界定的安全决策申请**，不是允许全面替换旧的 Git CLI 探测，也不是已经完成的 Git 2.36 兼容性结果。
