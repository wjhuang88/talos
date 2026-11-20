# 使用 gix 对旧版 Git 不支持的预检动作进行局部兼容

> 当前范围：2026-11-19 维护者更正，覆盖 2026-11-12“完全替换 Git CLI 前置分析”的旧意见。
> 来源：[Issue #919](https://github.com/wjhuang88/talos/issues/919)、[ADR-089 限定修订提案](adr-089-gix-native-preflight-amendment.md)。
> **Status: Proposed / not implementation authority.** 本方案保留实际系统 Git 写入与原有权限安全模型。

## 背景与实证

原始 Windows Git 2.36.1.windows.1（Scoop）中，`git var GIT_ATTR_SYSTEM` 与 `git var GIT_ATTR_GLOBAL` 直接返回 usage，属性路径发现阻塞 Prepared Checkout/Commit。2026-11-17 测试：纯 gix 候选 12/12；全套原环境 272 pass / 113 fail；仅移除测试进程中的 `GIT_INSTALL_ROOT` 时 305 pass / 80 fail。17 MiB 真实 Checkout/Commit 验收失败，不能据此宣布兼容。见 [原始证据](https://github.com/wjhuang88/talos/blob/fb5ee9809ddd4c1c63684157f6b5d35444df8173/docs/reference/WINDOWS-VALIDATION-2026-11-17-I324.md)。

**这些结果只能证明属性查询确实不兼容，不证明整个 Git CLI 预检不兼容。** 配置路径失效 5 项、URL rewrite/tracking 3 项及其他接收端、测试夹具错误必须独立判因。

## 目标与保留行为

- **保留** 现有且被旧版支持的受控 Git CLI 预检（如经批准的 `git config`、`git --exec-path`、`git check-attr`、ref/branch 查询；具体支持性须逐项验证），不改变已有单次审批和确定性审计边界。
- **替换** 明确不被旧版支持的具体调用：首批只关注 `git var GIT_ATTR_SYSTEM/GLOBAL`。可为此类查询统一使用经验证的 gix/Talos 原生实现，以避免脆弱的版本判断；并非要求重写其他已支持命令。
- **仍然由系统 Git 执行写入**。保留配置/hooks/filters/transport/维护辅助程序可能的全部操作效果；保留 Path/Command/Remote 精确授权、文件/可执行文件身份、失效重验和失败关闭。
- 如果替代操作无法可信重现被选 Git 的属性路径编译默认值，返回该操作明确的 Unsupported；**不要**因为采用 gix 就绕过配置源或放宽 Read。仅替代动作确实需要的可信来源才构成该动作的审批阻塞，不是整个预检的统一强制先决条件。

## 最小变更策略

| 预检动作 | 处理 |
|---|---|
| `git var GIT_ATTR_SYSTEM/GLOBAL` | 2.36 已证实不支持：优先设计单一、受限的替代属性路径求值 |
| `git var GIT_CONFIG_SYSTEM/GLOBAL` | 有配置路径失败证据，但需单独记录精准命令及原因后再决定 |
| `git check-attr --source=<target>` | Git 2.40 才加入该选项；Git 2.36 无法可靠执行目标树查询 | 仅替换 target-tree 这一查询为获准的 gix/字节解析，必须证明树对象与路径来源、优先级和 effect；保留普通与 `--cached` CLI，不允许拿当前索引冒充目标提交 |
| 其余已接受且兼容的 CLI 命令 | 保留；不改变固定 argv、授权、预算和重验 |
| gix/Talos 替代 API | 只能处理已经获准的输入/内存字节；严格禁止隐藏的未授权读取、执行、路径猜测及进程全局状态 |
| Windows `GIT_INSTALL_ROOT` | 与查询替代分开评估，不 blanket-allow `GIT_*` |

## 测试与交付门槛

- 增加针对每一条 CLI 的 Git 2.36.1 / 新版 Git 原生兼容矩阵；旧版本不支持的命令只按证据扩范围，不根据安装目录或硬编码版本推断。
- 用 recording sentinel 证明**新增替代路径**没有未获授权的 Git 或其他子进程；已获批准且保留的 Git CLI 调用是正常预检行为，不属于零子进程断言。
- 检查 system/global attributes、相对与缺失路径、空源、filter 预测、HOME/XDG、不同安装、Unicode/空格、替换/重验、>16MiB Prepared Checkout/Commit。
- 保留 ADR-089 Accepted 合同，拟变更的命令权限和新增路径必须单独安全/API 审查、维护者明确批准。更新 Rust locked、Windows native 和精确提交 CI 证据后才可宣布 #919 完成。

## 历史范围说明

2026-11-12 曾记录“gix 完全替换 Git CLI preflight，且所有预检零 Git 子进程”。该方向已被 2026-11-19 的维护者更正**明确取代**。此前的 gix 源码审计和纯解析候选测试保留为技术证据，但不意味着需移除所有已经正常工作的 CLI 命令。
