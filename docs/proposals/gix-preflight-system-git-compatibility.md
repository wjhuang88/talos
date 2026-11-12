# 使用 gix 组织系统 Git 的安全前置分析

> 日期：2026-11-10。状态：设计提案，尚未实现或激活。
> 维护者选择：继续支持 Git 2.36，不以要求升级替代兼容性修复。

## Issue reconciliation

Owner ID: GIT-COMPAT-001. Source Issue: #919.
Status: Proposed / Unclaimed on main. The formal backlog owner and I324 claim
are proposed in PR #920 and remain ineffective until merge. This existing
proposal is the interim main-branch reconciliation source, not an activation.
PR #920 should replace the matrix link with its formal owner when merged.

## Problem

Talos 最终通过系统 `git.exe` 执行结构化 Git 写入，但在执行前额外发现配置、
属性和可执行程序资源，形成权限审批计划。该前置流程属于 Talos 的安全策略，
不是 Git 自身要求的审批行为。

I300 提交 `e7114125` 引入的前置分析使用 `git var GIT_ATTR_SYSTEM/GLOBAL`；
当前代码还查询 `GIT_CONFIG_SYSTEM/GLOBAL` 和传输 shell 路径。
本机 Git 2.36.1.windows.1 不支持属性路径查询，导致 checkout/commit 等操作
在实际写入命令启动前失败。Git 2.42 发布说明记录了新增路径查询能力。
这属于 Talos 对宿主命令行能力的隐含依赖，不能简单归责于用户安装版本。

已发现的原生验证结果见 [Windows 验证记录](../reference/WINDOWS-VALIDATION-2026-11-10.md)。
URL 重写和 tracking 配置差异也须独立解决，不能把全部失败归于路径查询。

## Proposed Approach

由 Talos 编排前置工作，优先复用当前锁定的 gix 0.90.0 的配置解析、路径处理
和属性分析能力；系统 Git 保留实际操作执行职责。安全目标保持不变，
实现不再把新版 `git var` 查询作为 Git 2.36 的必经入口。

| 职责 | 实现边界 |
|---|---|
| 选择执行程序、固定环境和 cwd | Talos 保留实际获批 executable 的绑定和身份复验 |
| 配置解析、包含链和路径计算 | 优先复用 gix；文件读取仍须经过现有分阶段授权 |
| 安装级配置和属性默认路径 | 针对实际执行程序补必要的兼容适配，不猜目录、不遗漏空或缺失文件 |
| 必要的宿主探测 | 使用 Talos 受控执行器，支持旧 Git，保持超时、输出预算、错误处理和进程清理 |
| 生成、展示和执行权限计划 | 继续使用现有 prepared stages、facets 和 admission pipeline |
| checkout、commit 等实际写入 | 执行绑定的系统 Git，并复验前置证据 |

gix 已有 `gix::config_path()`，但不能据此直接认定与选定的系统 Git 等价。
本地 gix-path 0.14.1 源码表明，安装路径发现内部可能通过 PATH 寻找 Git、
调用 `git config -lz --show-origin --no-includes --show-scope --name-only`，
并将结果存入进程级 `LazyLock`。这些行为不能绕过 Talos 的 executable 绑定、
环境快照、调用预算和权限控制。实现前逐个审计所用 API，而非把整个发现过程
交给一个内部可自行启动进程的高层接口。

### 安全与一致性要求

1. 分析对象必须与最终执行程序、cwd、环境、配置优先级和包含链一致。
   gix 的默认环境或进程级缓存不能替代一次 prepared invocation 的证据。
2. 发现路径与读取内容分开；新发现的文件内容须获得对应 Read 授权后才可读取。
   即使文件当前为空或不存在，也要保留相关路径证据，以捕获审批后的出现或变化。
3. 保留 hooks、filters、维护子进程、远端、传输 shell 和写入目标的授权覆盖。
   未知路径不能当作“没有该文件”，也不能静默关闭用户配置或过滤器以让测试通过。
4. 审批后配置、文件身份、执行程序或操作发生变化，应使计划失效，重新发现和审批。
5. gix 调用的 panic 边界、宿主命令的超时/输出限制与安全错误传播须遵循项目约束。
   不新增 unsafe，不放宽全局目录授权，不增加未经审查的外部依赖。
6. 旧版与新版 Git 的语义差异必须显式建模或验证，不能假设 gix 的行为必然等同
   任意系统 Git。无法证明的操作应明确报告尚不支持，不能声称完整兼容已交付。

### 环境变量处理

本机 Scoop 设置的 `GIT_INSTALL_ROOT` 被现有 `GIT_*` 总体拒绝规则拦截。
应核实该变量对实际 Git 和启动包装器的作用后，决定保留或精确允许。
`GIT_PAGER` 已在现有规则中允许，不能把它误报为拒绝原因。
仓库重定向、配置注入、trace 文件输出等变量继续保留安全检查；
不通过删除所有 `GIT_*` 变量来实现产品兼容。

## Validation And Acceptance

- 在本机 Git 2.36.1.windows.1 上，用原始 Scoop 环境重跑 Git-enabled tools 测试；
  对必要环境差异作显式测试，不能依赖测试启动脚本静默清理变量。
- 验证大于 16 MiB 的 checkout/commit 和审批后修改源文件拒绝用例实际成功执行。
- 覆盖 HOME/XDG、自定义属性路径、缺失/空配置、include/includeIf、工作树、
  Unicode/空格路径及多个 Git 安装；新增文件、替换程序和改配置均须使旧计划失效。
- 将前置路径、配置顺序、URL 重写及 tracking 预测与 Git 2.36 的实际行为比较；
  同时覆盖新版 Git，避免兼容修复使已有平台回归。
- 测试未授权内容不被提前读取、hook/filter 不提前执行、预算耗尽不会触发替代探测。
- 使用仓库固定 Rust、`--locked`，完成相关测试、完整 preflight 和原生 Windows gates；
  权限/进程相关变更须通过逃逸向量审查及授权相关独立 review。
- 更新用户侧 Git 支持范围、安装说明和错误提示，再以已有实现提交记录完成证据。
  本提案和当前通过的非 Git 测试不构成兼容性修复的完成证据。

## Alternatives Considered

- 要求升级 Git：维护者未选择此范围；升级不能代替 Git 2.36 兼容实现。
- 忽略探测失败并继续写入：可能漏掉资源和可执行副作用，拒绝。
- 完全猜测系统路径：无法覆盖安装布局和编译默认值，不作为通用方案。
- 直接调用 gix 高层发现接口：只有在绑定执行程序、受控执行和缓存语义均满足要求时
  才能采用，不能仅因 API 名称匹配就替换。
- 完全改用 gix 执行所有写入：超出本提案，不据此改变系统 Git 的 hooks/filters 语义。

## Open Questions And Delivery Gates

实现阶段须证明旧 Git 安装级默认路径的来源，确定 gix 可复用的具体 API，
并决定 URL 重写和 tracking 配置如何按宿主能力匹配。Git 2.36 是验证目标，
目前没有证据声称全部平台、自定义编译版本和所有 Git 操作均已兼容。

本次仅提交说明文档。后续实施需按 requirement intake 建立 owner、验收范围和有效
Collaboration Claim；涉及 ADR-089 的发现命令或命名资源能力变化时先完成相应 ADR 审查。
不在本提案中更改既有 owner 状态、授权或完成条件。

## Dependencies And Sources

- [ADR-089：结构化 Git 效果授权](../decisions/089-structured-git-effect-authorization.md)
- [Git 写入实现](../../crates/talos-tools/src/git_write/prepared_add.rs)
- [Git 2.42 发布说明](https://github.com/git/git/blob/v2.42.0/Documentation/RelNotes/2.42.0.txt)
- [gix 0.90.0 config_path](https://docs.rs/gix/0.90.0/gix/fn.config_path.html)
- [gix-path 0.14.1 源码](https://docs.rs/crate/gix-path/0.14.1/source/src/env/git/mod.rs)

宿主探测是保留系统 Git 行为所需的兼容桥接，不是新增宿主工具依赖。
若 gix 后续提供满足绑定、受控执行和无隐藏 I/O 要求的接口，应替换自有适配。
