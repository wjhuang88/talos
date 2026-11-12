# TUI-PERF-001：TUI 事件驱动渲染与资源优化

> Document status: Active / Claimed (#910 merged as d355dd68)
> Created: 2026-11-10
> Owner: 本文；Board 仅为派生入口。
> 本轮交付为开发任务及实施指导。运行时代码尚未修改，性能收益尚未验证。

## 1. 目标与交付边界

用户在同时运行 Talos 与 Codex 时观察到资源差异，要求解释重绘原因、评估局部动画绘制，
并将优化独立登记，供开发者落地。本任务的可运行交付物是：空闲时不持续绘制、动画只使
必要计算失效、Git 状态扫描不阻塞交互的 CLI/TUI；同时保留现有显示、输入和审批语义。

实施顺序：P0 基线 → P1 调度 → P2 Git 快照 → P3 布局/组件缓存。
P4 历史增量优化仅在测量证明必要时进入范围，不能自动扩张为渲染框架重写。
各阶段可以形成独立稳定候选，但整体任务不能用某一阶段完成代替全部验收。

排除：Desktop 渲染、全局事件总线、替换终端库、后台任务业务调度、自动审核策略、
剪贴板功能、前缀字形垂直对齐、主机终端配置修改。这些功能的已有修复应保持有效。

### 约束分类

- Hard：保留权限/审批边界；业务事件不能因合并绘制丢失；不新增未经 ADR 批准的 unsafe
  或原生依赖；公共 API 破坏性变更需 ADR 和迁移方案；保持单一终端提交路径。
- Hard：使用仓库固定工具链与 Cargo.lock；合并前完整测试及对应平台验证通过。
- Soft：动画继续采用现有 150 ms 节奏；Git 后备刷新周期和性能预算由 P0 数据确定。
  调整时记录理由，不能以无限陈旧状态换取漂亮 CPU 数据。
- Assumption：无效帧和同步 Git 扫描可能贡献明显成本；目前未有剖析数据证明占比。
  区域持久缓冲、历史增量重排是否值得引入也须测量。

## 2. 开始实施前

Required Reads：

- [AGENTS.md](../../AGENTS.md)
- [协作与认领](../sop/AGENT-COLLABORATION.md)、[本地开发](../sop/LOCAL-DEV.md)、[测试](../sop/TESTING.md)
- [架构](../reference/ARCHITECTURE.md)、[事件架构边界](../decisions/006-event-architecture-boundary.md)
- [I114 视觉稳定性](../iterations/I114-tui-runtime-visual-stability.md)
- [I023 状态模型](../iterations/I023-tui-state-model.md)
- [工具结果截断](2026-11-08-tui-result-ellipsis.md)、[后台轮询归集](2026-11-08-process-poll-grouping.md)
- [剪贴板任务](2026-11-09-clipboard-image-input.md)、[Board](../BOARD.md)

当前为独立待办，不激活迭代、不声称已有 Collaboration Claim。
实际开发者须先盘点全部 Active / Review / Planned / Blocked 迭代及相关任务，记录冲突
处置，再按协作 SOP 在目标分支建立有效认领；需要新迭代时分配新 ID。
尤其核对其他任务对 app、frame、history、输入处理的并行修改，不覆盖其工作树。
认领记录应明确 Work Slice、负责人、授权路径、实现 PR 和交接条件。
已发布的本文计划作为基线保留，执行结果追加到末尾。

## 3. 已确认事实与证据边界

以下定位基于调研时主分支 `4e8d209ea209d3f89d28b00eeaec515fea9eb6a6`；
落地前重新核对函数，不依赖易变行号。

| 位置 | 当前行为 | 优化含义 |
| --- | --- | --- |
| `crates/talos-tui/src/app.rs` / `run` | 常驻 150 ms interval；循环中先提交 transcript、draw，再 select；部分输出分支还主动 draw | 约 6.7 次/秒只是定时器频率，并非总帧率上限；存在重复绘制机会 |
| `crates/talos-tui/src/app/frame.rs` / `draw_frame` | 计算预览、组件高度、布局、历史投影、模态和选择；部分模式切换重新规划 | 需要区分布局失效和仅样式/动画变化 |
| `crates/talos-tui/src/inline_terminal.rs` / `draw_inner` | 当前 buffer reset 后完整绘制；双缓冲 diff，仅输出变化单元格 | 已有终端增量输出；只跳过 widget 会把未画区域清空 |
| `crates/talos-tui/src/history_projection.rs` | revision+width 缓存；全量 rows 和 logical_lines 使用 Arc；可见 rows 仍复制 | 缓存命中没有深拷贝全部历史；优先检查可见切片复制和重复查询 |
| `crates/talos-tui/src/scrollback_status.rs` | 标准宽度至少 80 且工作区非空时读取 Git 状态 | 不可见时不应为了展示扫描 |
| `crates/talos-tui/src/scrollback_status_git.rs` | Mutex 快照 TTL 500 ms；miss 同步执行 gix discover/status，含 untracked 检查 | 扫描在绘制路径内，有阻塞风险，但成本占比未证明 |
| `crates/talos-tui/src/app/output.rs` | transcript 提交；processing frame 推进关联重连提示、动画及拖选滚动 | 删除常驻 tick 前必须拆出隐含时序依赖 |

历史提交 `c68fd08b6d31072cf5fa81fe2998018844760a5c`（2026-07-10，I114）
把 50 ms 改为 150 ms，动画推进从 draw 移到 timer，并跳过积压 tick。
该变更解决动画被输入/流式输出加速的问题。没有证据说明持续空闲重绘是经过性能研究的
必要取舍。它保留了多种定时行为共用一个节拍的结构；不能只删 timer 就认为修复完成。
Git 缓存注释中的旧 50 ms 描述也应随相关修改校正。

本会话观察过 debug Talos：PSS 约 168.7 MiB，两个 10 秒 CPU 样本约 7.88% / 3.7%。
独立 Codex CLI 样本约 118.2 MiB / 0.1%；另一组 Codex CLI、app-server 及辅助进程
合计约 527.6 MiB / 7.7–7.9%。100% CPU 表示一个逻辑核。
这些样本的构建、历史、工作负载和进程架构不一致，仅为调查线索，不是公平基准，
不能推出 Talos 普遍更耗资源、内存泄漏或具体热点。尚未采集火焰图。

## 4. P0：建立可复现剖析基线

交付：可重复执行的本地场景/回放工具、可选计数与计时、基线报告。
复用现有测试/模拟 provider 能力，避免真实服务网络波动；尚无能力时补最小回放夹具。

1. 记录 commit、Rust 版本、构建 profile、终端及版本、窗口行列、系统、电源模式、
   工作区文件数、历史条目/字符数、输入回放 seed。固定环境与数据。
2. 分别测 debug 和带符号的 release。可用以下命令准备分析构建；执行前核对 binary 名：

   ```sh
   cargo build --locked -p talos-cli
   CARGO_PROFILE_RELEASE_DEBUG=1 cargo build --locked --release -p talos-cli
   ```

3. 场景每轮预热后采集 30–60 秒，至少三轮；基线与修改后交替运行。记录中位数及范围，
   长尾输入延迟另报 P95/P99。固定历史规模至少覆盖 100、1,000、10,000 条。
4. 空闲、等待模型动画、固定速率流式输出、审批等待、长历史滚动、窗口缩放分别测量。
   Git 场景包含非仓库、小仓库、大量 untracked 文件、工作区切换。
5. 测量 Talos 主进程与其子进程；终端模拟器单列。用 CPU 时间差/墙钟计算 CPU%，
   用 PSS 比较驻留内存，避免直接累加共享页 RSS。重复场景观察堆是否稳定。
6. 给帧标记原因：输入、流、业务输出、动画、到期、Git 结果、resize、初始绘制。
   统计 wake、draw、layout、history rebuild/cache hit、Git scan、diff、flush 次数，
   各阶段耗时、变化单元格、输出字节、分配量、上下文切换和输入到显示延迟。
7. 输入到显示延迟先以事件接收到终端提交为软件指标；真实终端显示验收单列，不能将
   flush 完成解释为屏幕像素已经显示。记录 timer 到期延迟与 Git 快照年龄。

分析记录默认关闭，不写提示词、工具内容、凭据或剪贴板数据；比较开关前后开销。
直方图/计数器必须有界，不能为每帧无限积累记录。性能报告需说明缺失指标与采集误差。

Linux 如可用，可对固定 PID 采集：

```sh
perf stat -p <PID> -- sleep 30
perf record -F 99 -g -p <PID> -- sleep 30
perf report
```

调研环境未发现 perf，`perf_event_paranoid=2`；执行者应先检查工具与权限，不擅自
修改系统安全设置。不可用时记录限制，使用应用内分段计时和进程 CPU/PSS 差值，不能
伪称完成采样剖析。必要时另用分配剖析工具，记录其开销及运行条件。

P0 出口：附原始统计和复现命令，明确主要成本、误差、后续预算。
在开始 P1 前填写性能预算：空闲 CPU、各场景输入 P95/P99、Git 最大陈旧时间、
动画到期偏差、峰值内存及允许回归范围。预算以测量为依据，不预承诺降耗百分比。

## 5. P1：事件驱动调度与显式截止时间

交付：无可见变化不绘制；无业务/定时需求时不靠渲染 tick 唤醒。

- 在 app 层维护待绘制状态和原因；处理事件产生状态变更后统一请求帧。
  保留业务事件顺序及每次状态更新，只合并展示，不能丢流式 token、审批或完成事件。
- 初始帧只绘制一次；消除分支主动绘制后下一轮无条件再画的路径。
- 为可见动画、tip 失效、延迟重连提示、拖选边缘自动滚动分别计算下一截止时间。
  动画不可见时不设动画 deadline；到期时仅更新相关状态。
- `select!` 等待输入、业务事件、必要截止时间和 Git 结果；没有 deadline 时禁用
  timer 分支。到期前不空转，跨过多个节拍时跳到当前状态，避免补画历史帧。
- 批量处理事件有数量或时间上限；持续输出下输入、取消、resize 不得饥饿。
  审批、错误、取消、最终输出优先及时提交；不为凑批次拖延安全交互。
- 正常流式输出可合并到有限展示节奏；最后一帧必须提交。动画速率不得随事件速率变化。

建议控制流（描述约束，不要求照搬接口）：

```text
等待一个事件或最近截止时间
按顺序应用状态变更，并在有限预算内排空已就绪事件
提交 pending transcript，累积区域失效
更新到期状态及下一截止时间
若存在可见失效：规划必要布局，绘制一次，提交 diff，清除已消费失效
```

用可控时钟测试无输入下的 tip/重连/拖选行为、无 deadline 时无 draw、突发事件合并、
持续流公平性和最终帧；不要用不稳定的真实 sleep 断言。错误路径也必须释放调度状态。

## 6. P2：Git 状态脱离绘制路径

交付：状态栏格式化只读快照，扫描由有生命周期的受控后台工作完成。
这里的后台是实际与交互并行的工作，不是把普通顺序操作一律后台化。

1. 定义工作区标识、generation、结果、采集时间、进行中状态。保持依赖 panic 的降级边界。
2. 首次显示、工作区切换、相关工具完成、支持时的焦点恢复触发刷新；增加低频后备刷新
   以捕获外部修改。不要假定所有终端都会发送焦点事件。
3. 同时最多一个扫描；请求合并，限制排队；返回结果绑定工作区/generation，旧结果丢弃。
   不可见（如窄窗口）时停止仅为状态栏服务的请求；再次显示时按快照年龄刷新。
4. 快照内容不变不请求绘制。失败时明确选择保留旧值并标记陈旧，或显示未知；不得把
   错误解释为 clean。将最大陈旧预算与后备周期写入 P0 表并测试。
5. `spawn_blocking` 外层 timeout 不会终止正在运行的扫描。必须规定单任务上限、
   超时后的结果忽略、会话切换和退出时的生命周期；不能超时后不断启动新线程。
   若需要硬终止保证，应另评估隔离方案，不能在文档中承诺未实现的取消。
6. 第一阶段不要求目录 watcher。只有后备刷新成本或时效证据支持时再单独评估。

测试慢扫描不阻塞输入、扫描失败/恐慌降级、切换工作区不串结果、退出无无界任务增长、
相同结果无 draw、外部修改在预算内可见。模拟结果用可控通道注入。

## 7. P3：按依赖失效，缓存布局与静态组件

交付：纯动画更新不重复历史、Git 和稳定布局计算，仍通过现有终端 diff 提交。
优先复用计算结果，测量后再决定是否值得减少 buffer 构建及全 buffer diff。

| 变化 | 必要失效 | 必须检查的传播 |
| --- | --- | --- |
| 窗口宽高 | Layout + 所有相关组件 | 宽度变化重排历史；旧区域清除；滚动锚点 |
| transcript revision / 展开折叠 | History | 可见高度变化触发布局；分组尾部可能改变 |
| 流式预览内容 | Preview | 行数变化传播 Layout；高度稳定可只更新内容 |
| spinner / ripple | 对应动画组件 | 高度稳定时不得使 History/Layout 失效 |
| composer 编辑/附件 | Composer、Cursor | 换行/附件高度变化传播 Layout |
| panel / modal / queue / tip | 相应组件 | 开关或高度变化传播 Layout；遮挡旧区域清除 |
| Git 快照/状态文字 | Status | 同尺寸仅样式/内容更新；文本布局约束保持 |
| 选择变化/滚动 | History、选择覆盖层、Cursor | 跨区覆盖、拖选滚动及命中映射一致 |
| follow-tail/锚定模式切换 | History、必要时 Layout | 处理当前重新规划分支，避免递归失控 |

区域标记可保持为 app 私有简单类型，不引入通用响应式框架。
缓存键包含尺寸、组件可见版本、布局模式/高度、主题等实际依赖；明确每个 key 的生产者。
缓存失效以正确性为先：无法证明稳定时进行完整规划，不能用“看起来没变”猜测。

第一步仍每帧构建完整 buffer，但使用缓存的静态内容和布局。
若 P0/P3 结果证明 buffer/diff 成为主要成本，才追加区域保留方案：必须在 draw 生命周期
保留未失效单元格，跟踪旧/新区域并清除腾空区域，处理宽字符的相邻单元格及覆盖层。
不能在当前 `buffer.reset()` 后直接省略静态 widget。
区域优化与完整路径共享组件输出和提交逻辑，不维护两套业务渲染算法。

光标位置未变化也可能需要在 backend 写入后恢复，不能仅按坐标相等省略控制序列。
验证中文、emoji、组合字符、零宽字符、换行、背景色、选区、模态遮挡、窗口缩放和输入法。
用测试中的完整渲染结果作为 oracle，对相同状态序列比较最终 cells、样式、光标；
另外校验 ANSI 输出/真实终端，因为内存 buffer 一致不能覆盖全部终端行为。

## 8. P4：有证据才做长历史增量处理

先验证低复杂度改进：读取总行数不复制可见窗口、每帧复用一次投影、减少可见 rows clone。
全历史已有 Arc 共享，不能以“每帧深拷贝全部历史”为前提重写。

若 revision 更新导致全量投影成为实测热点，再设计分条目缓存与可回退的尾部重排：
连续后台轮询会归集上一段、展开状态会改变投影、tool call/result 有关联，不能只 append
最后一行。记录脏起点与回溯边界；宽度变化允许全重排。对不同分组、展开和乱序到达结果
比较增量输出与完整投影，保留滚动锚点和复制的 logical_lines 一致性。

本阶段默认待评估。决定不实施时必须附 P0/P3 证据与理由；若仍有性能缺口，将剩余项
登记独立 owner，不得用“可选”掩盖未达整体验收。

## 9. 验收矩阵与交付门槛

| 场景 | 必须满足 |
| --- | --- |
| 空闲，无输入/业务事件/可见 deadline | draw、diff、flush 为零；允许独立低频 Git 唤醒，但不变结果不绘制 |
| 仅动画 | 不扫描 Git、不重建历史、不重算稳定布局；变化单元格限于预期动画区域（控制序列另计） |
| tip / 重连 / 拖选 | 不靠额外按键仍按截止时间更新，取消后停止唤醒 |
| 大量流式事件 | 输入/取消无饥饿，顺序完整，动画速度稳定，最终帧不遗漏 |
| 审批与模态 | 可见状态及时更新，快捷键/重复键/取消语义不变，不触发额外授权 |
| Git 慢/错/切换 | UI 保持响应，无旧工作区结果覆盖，无并发扫描增长，陈旧度符合预算 |
| 历史与缩放 | 归集/展开/锚点/选区/复制/省略号/显式换行正确，无残留和半个宽字符 |
| 输入框 | 附件、背景、光标、IME、重复输入及焦点切换回归通过 |
| 长时重复回放 | 堆与缓存有界，无扫描或计时记录积压；CPU/延迟符合 P0 冻结预算 |

真实终端至少覆盖 Linux Wayland + niri + Ghostty（用户环境），另一个受支持终端，
以及原生 Windows 终端。记录 OS/终端版本、构建 SHA、行列和具体操作结果；交叉编译
不能代替原生执行。平台不可用则记录为 Review/待验收，不写“全部通过”。

实现阶段按修改范围执行聚焦测试，稳定候选再执行完整门槛：

```sh
cargo test --locked -p talos-tui
./scripts/release_preflight.sh
cargo xwin check --locked -p talos-tui --tests --target x86_64-pc-windows-msvc
./scripts/validate_project_governance.sh .
bash scripts/validate_collaboration_claims.sh .
git diff --check
```

先阅读最新 LOCAL-DEV；涉及 CLI 或其他 crate 时扩大 xwin 范围。完整 preflight 必须
包括 locked workspace 测试。保留精确命令、退出码、报告位置、实现 SHA、exact-head CI
及规定的独立评审证据。本文任务登记不执行或声称通过运行时代码测试。

用户文档同步目标：README 的 CLI/TUI 使用入口及排障说明；如分析开关暴露给开发者，
在 LOCAL-DEV 增加启用、采集、开销与隐私说明。不要把内部 invalidation 术语放入普通交互。

## 10. 开发检查点、回退与续作

- [ ] 认领前盘点与冲突处置，目标分支认领生效。
- [ ] P0：回放夹具、原始基线、可选指标、预算表与热点结论。
- [ ] P1：调度实现、deadline 清单、可控时钟/公平性测试。
- [ ] P2：Git 快照、并发/生命周期约束、慢/错/切换测试。
- [ ] P3：失效矩阵实现、缓存键审查、buffer/真实终端等价验证。
- [ ] P4：依据测量决定实施或附证据暂不实施。
- [ ] 对照报告、原生平台验收、文档、完整本地门槛、CI 与评审。
- [ ] owner 证据及状态更新，再同步 Board；遗留项各有明确 owner。

每个阶段先局部收敛设计、代码、测试、文档及 staged diff，再推送稳定候选。
调度、Git 和缓存保持可分别回退的逻辑提交；出现显示回归先撤销引入问题的阶段，
保留剖析与回归夹具。不可用禁用审批、丢弃输出、降低正确性或无限暂停扫描来回避缺陷。
不为回退长期维护第二套 renderer；临时诊断开关需有删除条件。

每次中断在下表追加已做工作、验证证据、未解决问题和下一条可执行步骤。
Complete 需要真实存在且已推送的实现 SHA 写入 `Completion Commit:`，并满足全部必要
验收；文档创建提交不能作为运行时完成证据。当前没有实现 Completion Commit。

| 日期 | 阶段/状态 | 证据与问题 | 下一步 |
| --- | --- | --- | --- |
| 2026-11-10 | 任务登记 / Planned / Unclaimed | 完成源码路径与历史原因梳理；尚无公平基准或运行时优化 | 开发者按第 2 节认领，再完成 P0 |

本次文档检查：仓库 `validate_project_governance.sh`、`validate_collaboration_claims.sh`
及 agent-project-governance 技能 validator 均 exit 0、0 warning；`git diff --check`
通过。检查仅证明任务登记和文档结构有效，不是运行时性能或显示验收证据。

### P0 待填写预算与结果

| 指标 | 基线（构建/场景/三轮范围） | 接受预算及理由 | 修改后 | 证据位置 |
| --- | --- | --- | --- | --- |
| 空闲 CPU / wake / draw | 待测 | 待冻结；无事件无 draw 为硬验收 | 待测 | 待填 |
| 动画 CPU / layout / history rebuild | 待测 | 待冻结；纯动画不重算稳定历史/布局 | 待测 | 待填 |
| 输入提交 P95/P99 | 待测 | 待冻结 | 待测 | 待填 |
| deadline lateness / Git staleness | 待测 | 待冻结 | 待测 | 待填 |
| PSS / 峰值堆 / 分配 / 输出量 | 待测 | 待冻结 | 待测 | 待填 |

## Execution intake — 2026-11-11

The maintainer requested implementation closure of this complete published plan.
No P0–P3 acceptance is removed; P4 remains evidence-dependent. This is a task,
not a new iteration. The published baseline above remains unchanged.

Target snapshot: `a320b7a8`. Current original worktree has unrelated uncommitted
background-continuation changes and approval/tool-summary fixes. Preserve them;
performance work uses an isolated worktree. The approval follow-up may later
require explicit integration and revalidation; it is not performance evidence.

Nonterminal iteration inventory (owner document status, not historical index prose):

| Owner | Current state | Disposition |
|---|---|---|
| I249 | Planned | Dependency upgrade deferred; no scope transfer |
| I277 | Review | Desktop device acceptance; excluded |
| I302 | Review | Windows owned-handle acceptance; excluded |
| I310 | Active | Gemini provider, PR #838; preserve |
| I318 | Review | Key repeat; preserve batching/release semantics and rerun regression |
| I319 | Active | Search retirement, PR #907; no overlapping TUI source |
| I320 | Review | Host composition; no activation or scope transfer |
| I321 | Active | Storage foundation, PR #909; no overlapping TUI source |

Open PR #861 also retains its search compatibility owner. No open PR in the
2026-11-11 inventory claims TUI render scheduling or status snapshot caching.
Related TUI approval, ellipsis, polling aggregation, clipboard and visual-stability
owners retain their original scope and pending device acceptance. This task must
regress those behaviors rather than declare them completed by association.

Verified source still has a permanent 150 ms interval and unconditional loop draw
in App::run, plus additional tool-output draws. This establishes work remains,
not a measured cost allocation. Next: finalize claim, implement P0 replay/metrics,
measure baseline and freeze budgets before P1.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / gpt-6 |
| Work Slice | TUI-PERF-001 P0 measurements, P1 event scheduling, P2 Git snapshots, P3 dependency caches, evidence-based P4 decision and full acceptance |
| Claimed At | 2026-11-11 |
| Source Issue | None |
| Governance Claim PR | #910 |
| Authorization Mode | Independent review |
| Authorization Evidence | #910 merged as d355dd68; exact-head CI 42933711478 success; independent agent review APPROVE at 6c3e23df |
| Implementation PR | #922 (Draft) |
| Last Updated | 2026-11-12 |
| Handoff / Release Condition | Full published acceptance including native terminal evidence; incomplete platforms remain Review |

### P0 execution checkpoint — 2026-11-11

Claim #910 is effective at `d355dd68bb2892ddcd9f042b4d83fd9b71fb65ae`.
Implementation branch: `perf/tui-event-render`, isolated from the user's original
worktree. The initial real-TUI offline replay supports deterministic 100/1000/10000
message history, idle, animation and 20 Hz streaming. Locked example build passes.
The Linux synthetic PTY sampler collects process CPU time, PSS and output bytes;
its explicit ready signal excludes terminal startup from warmup/measurement.
Python is developer-only measurement tooling, not a Talos runtime dependency.
Without Linux procfs/PTY this sampler is unavailable; native platform acceptance
remains independently required. Do not treat smoke tests as the three-round baseline.

Still missing before P0 exit: approval/scroll/resize/Git fixture matrix, stage and
latency instrumentation, child/terminal accounting, release-symbol builds, repeated
raw statistics and frozen budgets. No scheduling or caching benefit claimed.

P0 instrumentation checkpoint: opt-in bounded aggregate counters/timers now cover
draw, layout rectangle planning, history rebuild/hit, Git scan, buffer diff,
flush and input/output/timer wakes. `cargo test --locked -p talos-tui --lib`
passed 652 tests; replay build and diagnostic smoke passed. Three-second idle
smoke observed 23 draws/layouts/diffs, 20 timer wakes, 2 initial history rebuilds
and 43 history cache hits, with no Git scan. This validates instrumentation and
the repeated idle-draw path, not a representative performance budget. Reports:
`/tmp/talos-perf-metrics-smoke.json`, `/tmp/talos-perf-metrics-sampler-smoke.json`.
These ephemeral reports must be replaced by committed full baseline artifacts
before P0 exit. Layout timing currently measures rectangle planning, not all
component height calculations; input events are counts, not latency evidence.

P0 follow-up: software input-to-submit latency and resize/legacy-stream wake
counters added. Eight synthetic input batches produced eight submitted latency
samples in the focused PTY smoke; this is not physical display latency. The TUI
suite now passes 653 tests, including histogram overflow/saturation boundaries.
Replay scenarios now include display-only approval, alternating PageUp/PageDown
and alternating 80x30/120x40 resize. Build SHA-256 and OS identity are captured;
full source/build/environment provenance and repeated baseline matrix still remain.

### Scheduler dependency audit before P1

Source inspection at the P0 candidate found two additional wake dependencies:

- `retire_closed_approval` currently detects a dropped approval receiver only at
  the next loop iteration. Removing the recurring tick requires explicitly
  awaiting sender closure; an expired modal must not remain indefinitely.
- `next_stream_chunk` mutates transcript state on legacy stream EOF, then returns
  None; the select pattern `Some(chunk)` discards that branch's completion. The
  timer currently supplies the later draw. P1 must represent EOF as a handled
  event and submit final content without relying on another input or deadline.

Keep the existing tip expiry, deferred reconnect and drag-edge deadlines as
separate producers. Add controlled regressions for approval closure and legacy
EOF as well as canonical FIFO Content::End. These are audit findings, not fixes.

Initial three-round debug idle data and its limitations are retained in
[evidence/tui-perf-001/initial-idle](evidence/tui-perf-001/initial-idle/README.md).
CPU median 2.371%, range 2.270–2.404%; steady terminal bytes zero, but all three
runs made 237 draw/layout/diff calls including startup and 234 timer wakes.
This confirms repeated computation, not P0 completion or a final budget.
Perf is unavailable; no sampled flamegraph is claimed.

### Long-task execution contract

Work mode: Standard. Deferred validation tracker: None; native terminal gates
remain required in this owner. The user authorized full implementation closure;
#910 established the effective claim. Outcome, scope, exclusions, dependencies,
ordered P0–P4 items, gates and fallback remain the published sections above.

Worktree `/tmp/talos-tui-perf-claim`, branch `perf/tui-event-render`. Preserve original
worktree changes. Ordinary source edits, offline tests and requested PR delivery
are authorized; no system security changes, external messages, destructive cleanup
or release publication are implied. Raw reports and manifests live under the
explicit experiment output directory and are retained with the evidence report.

Measurement batches are serial, 35 seconds per scenario including 5-second
warmup, 3 rounds per case, grouped by history size. Each process has a bounded
startup/exit timeout; one failure stops the batch and retains successful samples.
Do not silently resume using a changed binary, overwrite an output directory,
or run builds/tests during timed samples. Inspect live process handles before
retrying. No repeated CI submission or speculative benchmark rerun is authorized
without new changes, a failed check, or an identified unresolved measurement issue.
All incomplete rows stay here; this contract does not relax native acceptance or
mark P0 complete. No deadline or token/cost budget was imposed by the maintainer.

P0 independent static review requested changes before baseline acceptance:
validate actual sample-window duration, parse/validate metrics rather than only
checking file existence, and distinguish injected actions from actual scroll/size
changes. Window/schema validators and their two regression tests now pass. Existing
completed matrix samples pass the retrospective window/schema checks, but old
schema-1 scroll/resize reports lack actual-state evidence and remain preliminary.
Schema 2 adds actual history-scroll and observed screen-size counters, and counts
every consumed input/resize event in a batch. Source edits are not rebuilt while
the schema-1 batch is running, preserving its sampled binary. Full retest and
replacement acceptance evidence are pending. Context switches currently describe
the main thread, not all process threads.

Native-platform constraint confirmed by maintainer: no Windows environment is
available for physical terminal acceptance (2026-11-11). Keep that row pending;
xwin and Windows CI are not substitutes for native terminal behavior. Linux
implementation and independent automated gates continue. Overall Complete remains
unavailable until the required device evidence exists; no scope waiver was given.

The initial schema-1 debug history100 matrix completed all 18 cases. The corrected
steady-window/schema-2 fixture smoke passed all six scenarios; actual scroll and
observed terminal-size counters were nonzero in the corresponding cases. Full
corrected baseline, release matrix and final gates remain pending.

Steady-window diagnostics and explicit deadline-lateness/frame-preparation timers
now compile; 653 TUI tests and MSVC xwin tests/examples check pass. Standalone
and matrix sampling share duration/gap validation, and strict matrix acceptance
checks steady actions rather than cumulative prewarm activity. Pure idle and
approval waiting permit zero steady draws by design for the target scheduler.

P0 build checkpoint: both source-bound builds at `52263c6279c3fb068332d63edb6a3770081c9ce2`
exit 0. Release explicitly uses `CARGO_PROFILE_RELEASE_DEBUG=1` and
`CARGO_PROFILE_RELEASE_STRIP=false`; `readelf -S` confirms `.debug_info`,
`.debug_line` and `.symtab`. Binary hashes and build settings are retained in
[baseline-build/receipt.json](evidence/tui-perf-001/baseline-build/receipt.json).
The release six-scenario / three-history / three-round matrix is running at
`/tmp/talos-p0-release-full`; only its final validated manifest can establish
completion. Do not rebuild or run tests during this measurement. Debug matrix,
Git cases, input probes, instrumentation overhead and budget freeze remain pending.

Developer Git fixtures now contain a non-repository, a clean one-file repository,
and a repository with exactly 10,000 untracked files (verified by `git ls-files`).
The new summary script reports histogram bounds for P95/P99, including overflow,
rather than treating bucket boundaries as exact latency. Its initial smoke and
boundary checks passed before measurement began; subsequent identity-validation
changes and committed regression tests still require execution after sampling.
P1–P3 remain unimplemented. Native Windows acceptance remains pending because the
maintainer has no available Windows environment; this does not block Linux work.

### P1 implementation notes from the baseline source audit

The following choices are implementation guidance, not delivered behavior:

- Select into a private event value before mutating `self`. The current
  `next_stream_chunk(&mut self)` borrows the whole TUI across `select!`; waiting
  for approval sender closure in the same select requires splitting the stream
  and pending-response field borrows. Receive legacy chunks without side effects,
  then handle `None` as an explicit EOF event and finalize outside the select.
- Use an optional nearest deadline. Tip expiry is `created_at + ttl`, deferred
  reconnect already carries its own deadline, and animation/drag cadence must
  advance from their previous deadline rather than reset after every input.
  Otherwise continuous input postpones animation indefinitely. Missed intervals
  advance state without submitting catch-up frames.
- Retain the current bounded input batch and release filtering. Drain business
  outputs with a separate finite budget; do not drain through an approval barrier.
  Queued outputs currently flush via an unbounded loop, so removing unconditional
  draws must also preserve cancellation/input fairness during queue release.
- Never count layout-cache hits as avoided work while still recomputing all
  component heights. Cache the preparation inputs/results needed by the published
  P3 dependency matrix, and keep full-buffer rendering until a retained-buffer
  design has independent evidence. The current history-cap lookup constructs a
  viewport solely to read total rows; a count-only cache accessor is a candidate
  low-complexity improvement, subject to measured P4 justification.

These notes preserve the published P0-before-P1 order. Only the optional
workspace-switch replay was added during the immutable release baseline run;
that fixture change has not been built or tested yet and is not part of the
running binary. The active matrix manifest and build receipt retain the original
`52263c62` source identity. Follow-up Git-switch measurements need a new build
receipt rather than relabeling this binary.

### P0 release matrix completed; remaining baseline work

The source-bound release matrix exited 0 with all 54 cases validated. Raw reports,
per-round metrics and summary are retained in
[release-baseline](evidence/tui-perf-001/release-baseline/README.md).
Idle CPU medians range from 0.334% to 0.370%; 10,000-entry resize has a 10.004%
median (9.770–10.134%). These are pre-optimization measurements, not savings.
The large-history resize cost requires inspection before the P4 decision.

After sampling ended, all six Python regression tests passed, the TUI library
suite passed 653 tests, and `cargo xwin check --locked -p talos-tui --tests
--examples --target x86_64-pc-windows-msvc` passed. Workspace-switch smoke passed
with five actual steady workspace changes and eight input-submission samples;
all 27 optional process-tree snapshots were complete. This five-second smoke
does not replace three-round Git/input measurements. Host Git fixtures now fix
the branch/template and suppress hooks/signing; regenerated fixtures succeeded.

Still pending before P0 exit: debug matrix, three-round Git/switch and input
probes, instrumentation overhead, full evidence limitations and frozen budgets.
Runtime event scheduling, asynchronous Git snapshots and preparation caches
remain unimplemented. Full local preflight, implementation CI/review and native
terminal acceptance also remain pending.

### P2 dependency audit: cancellation is not hard termination

Inspected the locally pinned `gix 0.90.0` source before choosing the worker
lifecycle. Upstream `src/status/iter/types.rs` (Parallel Operation) explicitly
states that without `parallel`, status is computed in advance and the iterator
is non-interruptible. `src/status/platform.rs` exposes `should_interrupt_owned`,
but this does not establish hard cancellation for every execution path.
`src/status/iter/mod.rs` also documents non-interruptible index work even on the
parallel path. Primary source: https://github.com/GitoxideLabs/gitoxide/tree/gix-v0.90.0/gix/src/status
(local Cargo.lock version and registry source were used for this inspection).

The TUI manifest disables default features and does not directly request
`parallel`; feature unification must be checked for the final CLI build before
making feature-specific claims. Do not enable dependency parallelism merely to
claim cancellability. P2 must keep one bounded scan ownership slot occupied until
the underlying work actually returns, even if its result times out or its
workspace generation becomes obsolete. Cooperative cancellation can supplement
this invariant but cannot replace it. Shutdown must document any surviving work
and avoid joining a stalled scanner on the UI thread. This is a verified upstream
constraint and design requirement, not an implemented worker or cancellation test.

### Frozen implementation budgets before P1

Both six-scenario/three-history/three-round matrices, 36 input-probe samples,
18 static Git samples, 24 diagnostic-overhead samples and three debug workspace
switch samples passed. Raw reports are in `evidence/tui-perf-001/` under
`debug-baseline`, `release-baseline`, `input-baseline`, `git-baseline`,
`metrics-overhead`, and `workspace-switch`. The latter uses source `15ef42c1`;
other baseline binaries use `52263c62`. Measurements justify starting P1 with
the following fixed acceptance budgets, without claiming full P0 completion:

| Metric | Budget | Basis |
|---|---|---|
| No-event idle | Zero steady draw/diff/flush; CPU <=0.10% release, <=0.20% debug without diagnostics | Baselines ~0.33–0.37% release, ~1.44–1.94% debug; zero unnecessary frames is the primary invariant |
| Input P95/P99, excluding resize | <=10/20 ms release; <=20/35 ms debug | Baseline P99 upper <=1.64 ms release, <=8.74 ms debug; headroom for host scheduling and bounded batching |
| Resize input P95/P99 at 10000 history | <=100/150 ms release; <=150/200 ms debug | Baseline P99 upper <=70.56/122.69 ms respectively; full width reflow remains required |
| Animation cadence | 150 ms; P99 deadline lateness <=50 ms except resize; no catch-up frame bursts | Preserve established cadence; explicit bounded delay target for scheduler tests and replay |
| Git freshness | Refresh every 2 s while visible; old results become explicitly stale/unknown at 5 s; scan timeout 2 s | 59 scans/30 s baseline is wasteful; bounded freshness independent of failed or stalled scans |
| Active CPU/PSS regressions | No >20% median CPU regression in matching active scenarios; PSS peak <=baseline peak*1.20+8 MiB | Three-round variability and fixed-size caches need headroom; compare interleaved baseline/candidate |
| Scanner lifetime | At most one actual scan at a time; timeout never frees the slot before worker return | Upstream cancellation limits and published lifecycle requirement |

Remaining measurement limitations stay open: no allocation-volume profiler or
native-terminal resource/visual evidence; process-tree polling misses short-lived
work; workspace-switch release profile is not yet sampled; precise Git snapshot
age only becomes meaningful with P2 snapshots. These residuals remain part of
overall acceptance, not waived by the budget freeze. P1 implementation may now
proceed using the measured baselines; P0 is Partial until its remaining evidence
is resolved or explicitly scoped by the maintainer. Do not relabel P1–P3 complete.

### P1 first local implementation checkpoint (uncommitted)

The loop now selects explicit input/output/stream/approval-close/deadline events,
draws only when dirty, removes duplicate initial/tool draws, and handles legacy
stream EOF outside the select. Input batches retain release filtering. Queued
business events are consumed one per select to preserve FIFO while allowing
input to compete. Cadence deadlines survive input traffic and skip elapsed frames
without catch-up renders; a deterministic time-arithmetic test covers this.

The TUI suite passed 654 tests. Six three-second PTY smoke cases passed at
`/tmp/talos-p1-first-smoke`: steady idle and approval draw/timer counts were zero;
scroll/resize each drew four frames with zero timer wakes; animation drew 13
frames with 13 timer wakes in the two-second steady window. These are smoke
results, not the full comparison or native visual acceptance.

P1 remains Partial: dedicated loop regressions for closed approvals, legacy EOF,
tip/reconnect/drag deadlines, fairness and final stream frame are still needed;
bounded stream display coalescing remains pending. Visible processing currently
uses status.is_processing as a conservative animation trigger. A transitional
500 ms Git deadline still draws when a workspace/status is visible; P2 must
replace it with asynchronous snapshot delivery and unchanged-result suppression.
P3 caches and P4 decision are not implemented. No implementation completion or
push is claimed by this checkpoint.

### P1–P3 local implementation checkpoint (supersedes earlier pending notes)

All changes below remain uncommitted on `perf/tui-event-render` in the isolated
`/tmp/talos-tui-perf-claim` worktree. The original project worktree is unchanged.

- P1 now coalesces streaming frames at 33 ms, observes approval closure and legacy
  EOF explicitly, and submits pending final content before Exit. Ordered 1000-token
  Exit-without-End regression passes. A 200 Hz PTY smoke observed 802 output events,
  119 frames and nine input latency samples (maximum 5.99 ms) in its steady window;
  this does not prove final terminal contents or cancellation fairness.
- P2 runs Git scanning off the UI thread with a process-wide single actual-worker
  slot. Timeout/drop suppress publication without releasing ownership before return;
  panic containment and abandoned-worker ownership tests pass. Workspace generation
  checks reject obsolete results. Refresh/timeout/staleness are 2/2/5 seconds.
  Unchanged snapshots do not redraw; a five-second steady smoke observed three
  scans and zero draw/diff/flush calls. A spawn failure now immediately submits its
  changed unknown status instead of waiting for another event.
- P3 caches splash, layout and composer height, and computes history row count
  without preparing a visible projection. The compressed preview plan is shared
  between height calculation and drawing for ordinary and tool previews, avoiding
  duplicate wrapping within a frame. This is not a cross-frame preview cache.
  Unicode/newline/width/cursor oracle tests cover existing caches.

The TUI suite passed 664 tests after the preview-plan change. MSVC xwin checking
of TUI tests/examples passed at that checkpoint; the subsequent spawn-failure
scheduling correction still requires the next checks. Full workspace preflight,
final-frame/cancellation loop oracles, slow/switch Git integration, cache buffer
and cursor equivalence, the formal interleaved performance comparison, P4 decision,
implementation review and remote gates remain outstanding. P1–P3 remain Partial.
The user explicitly has no native Windows environment; xwin is build evidence
only and native Windows terminal acceptance remains unavailable, not passed.

### Lifecycle and equivalence follow-up (local, uncommitted)

The display snapshot is now owned by each TUI rather than a process-global cache;
only actual Git scanner ownership is global. Tests cover two sessions sharing a
workspace, workspace mismatch, exact expiry and unchanged-result freshness renewal.
The final CLI dependency graph (`cargo tree --locked -p talos-cli -e features -i gix`)
confirms gix 0.90 has no `parallel` feature, retaining the cancellation limitation.

The production event loop now accepts a private input-stream seam. An actual-loop
regression verifies cancellation amid 1000 queued ordered deltas, retention of all
tokens in order, and submission of the last token to the terminal buffer before
Exit without End. This strengthens the earlier handler-only evidence; it is not a
native display or latency measurement. Animation scheduling uses the final visible
preview rows, with a regression covering compression to a one-row window and
restoration. A 28-step cache oracle compares full cells/styles/cursor/visibility
against forced fresh planning across Unicode, explicit newlines, cursor moves,
approval open/close, scrolling and resize.

The full preflight reached 668 passing TUI unit tests, then failed the I174 source
layout test because it named the removed `next_stream_chunk` method. That structural
assertion has been migrated to the private scheduling module and its actual EOF
wait call, preserving module encapsulation checks. Full preflight must be rerun;
the failed run is not a passed workspace gate. Latest logs are
`/tmp/talos-perf-preflight.log`, `/tmp/talos-perf-final-frame.log` and
`/tmp/talos-perf-frame-oracle.log`. Other acceptance gaps listed above remain open.

### Deterministic scheduling and preview cache checkpoint (local)

P3 now has two single-entry preview geometry caches (natural/compressed height).
Keys include width, bounded padding width, text, height cap, spinner presence and
thinking-layout presence. Animated glyph/color/frame changes are excluded from
geometry; the actual first-row prefix is restored for each frame. Tests compare
cached plans with the full planner across empty/thinking/long/Unicode text,
widths 0/1/2/3/7/80, caps 0/1/4/10/11, active/inactive states and animation frames;
unchanged animation must not rebuild. The full-cell oracle also resets these
caches on its fresh-planning path. This avoids wrap computation, not all per-frame
buffer construction or bounded visible-row copies.

The loop now uses Tokio's monotonic clock (converted to the existing Instant type)
for its deadlines. Test-only Tokio clock control verifies: one initial draw in a
minute of no-event idle; independent tip expiry and deferred reconnect without
visible animation; three drag scroll ticks before mouse release and no later
frames; at most four submitted frames for 1000 ready deltas plus cancellation and
final Exit. No real sleeps are used to assert these scheduling outcomes.

A controlled blocked-scanner integration test pauses the UI clock after worker
entry, crosses the two-second timeout, switches workspace/generation, and confirms
input submission and Exit remain responsive. Refreshes cannot create another
actual worker; even a following session cannot acquire its slot until the blocked
worker is explicitly released. Worker completion now supplies the snapshot
collection timestamp. Optional frame-weighted Git age metrics are documented in
LOCAL-DEV; no native display freshness claim follows from them.

`cargo test --locked -p talos-tui` passes 673 unit tests, 2 structure integration
tests and 2 doctests at this checkpoint. Full preflight, current xwin and final
formal performance comparisons still need to run for this source. Earlier
limitations concerning native Windows/terminal acceptance and profiling remain.

Validation follow-up: `./scripts/release_preflight.sh` exited 0 on the complete
local implementation above (`/tmp/talos-perf-preflight2.log`), including locked
workspace check/Clippy/tests, governance checks and external runtime SDK fixture.
`cargo xwin check --locked -p talos-tui --tests --examples --target
x86_64-pc-windows-msvc` also exited 0 (`/tmp/talos-perf-cache-clock-xwin.log`).
P0 baseline archives and summary-accounting changes are committed locally as
`97764675`; runtime changes remain uncommitted. Neither local checks nor this
baseline commit establish final performance/native acceptance or remote gates.

### Local stage commits and candidate measurement boundary

Runtime work is now separated into three reviewable local commits:

- `61f2c322`: P1 event/deadline scheduling and deterministic loop regressions.
- `1e574cd1`: P2 asynchronous Git worker, per-session snapshots and lifecycle tests.
- `539adfd2`: P3 bounded layout/preview caches and complete-buffer equivalence tests.

The exact P1 and P2 staged trees were exported without changing the working tree
and tested independently (`cargo test --locked -p talos-tui`): respectively
663 and 667 unit tests, plus 2 integration and 2 doc tests each. The final combined
source matches the previously validated 673-test implementation; strict all-target
TUI Clippy also passed (`/tmp/talos-perf-stage-clippy.log`). All commits remain
local, not pushed; no phase is marked Complete on this basis.

Candidate debug/release replay binaries are being built from `539adfd2`, with
Rust build inputs verified clean against that commit. Release retains debug
symbols (`CARGO_PROFILE_RELEASE_DEBUG=1`, `CARGO_PROFILE_RELEASE_STRIP=false`),
with two build jobs. Immutable copies and SHA-256 receipt will live under
`/tmp/talos-candidate-539adfd2`. Formal paired sampling starts only after both
builds complete; use the existing source-52263c62 immutable baseline binaries and
identical replay parameters. Do not compile or run tests during sampling.

### Formal paired release sampling started

Both candidate builds completed successfully at source `539adfd2`; the receipt is
archived in `evidence/tui-perf-001/candidate-build/receipt.json`. `readelf -S`
confirmed `.debug_info`, `.debug_line` and `.symtab` in the release binary.
The formal release matrix is running at `/tmp/talos-perf-release-paired-539adfd2`:

```sh
python3 scripts/replay_tui_perf_matrix.py \
  --binary baseline=/tmp/talos-p0-binaries-52263c62/release \
  --binary candidate=/tmp/talos-candidate-539adfd2/release \
  --output /tmp/talos-perf-release-paired-539adfd2 \
  --profile release-with-symbols
```

Defaults retain six scenarios, histories 100/1000/10000, three rounds and 35-second
runs including five seconds warmup, totaling 108 alternating samples. No builds
or tests run during sampling. The first candidate idle/100 sample has zero steady
draw/diff/flush/timer counts; this is preliminary evidence only, not a completed
matrix or CPU-budget conclusion. New `compare_tui_perf.py` and its focused budget
tests are prepared but unvalidated until sampling ends; they evaluate only their
explicit CPU/PSS/idle/stable-layout checks and cannot close overall acceptance.

### Release paired matrix completed; debug comparison started

All 108 release cases completed with exit 0. Raw reports and validated derived
summary/comparison are archived under `evidence/tui-perf-001/release-paired/`.
Twelve Python evidence tests passed. Every explicitly evaluated release check
passed (CPU/PSS budgets, idle/approval frame suppression, stable animation
planning and animation deadline P99). Stream/10000 CPU median increased from
1.440% to 1.491% (~3.5%, within the frozen +20% budget); do not claim universal
speedup. Input probes were off and workspace empty, so input/Git/native/metrics-off
budgets remain unverified by this matrix.

The equivalent 108-case debug comparison is now running at
`/tmp/talos-perf-debug-paired-539adfd2`, using baseline/candidate debug binaries
from the same source-bound receipt directories and `--profile debug`. No builds
or tests run during sampling. The task remains Active/Partial pending remaining
measurement, acceptance and delivery gates.

### Debug paired experiment: failed evidence and provenance investigation

All 108 cases completed, but the stable animation layout gate failed for all
three history sizes. Reports are preserved in `evidence/tui-perf-001/debug-paired/`.
Other evaluated CPU/PSS/idle/approval/deadline gates passed; this is not acceptance.
The captured debug executable lacks LayoutCache implementation symbols while
containing earlier-stage code; its original build reused the shared target in
0.27 seconds. The receipt therefore does not sufficiently prove source identity.
An isolated target rebuild with Cargo JSON artifact paths is in progress.
Do not attribute the failed comparison to current source until provenance is
resolved, or use the old debug receipt as verified candidate evidence.

### Debug provenance correction verified

The old executable embeds `/tmp/talos-perf-p2-check`; a forced current library
rebuild restores P3 cache symbols. A ten-second animation smoke (two-second
warmup) records 53 steady draws, zero layouts and zero history rebuilds.
No runtime source change was needed. The corrected receipt is
`evidence/tui-perf-001/candidate-build/debug-rebuilt-receipt.json`.
An attempted isolated target under /tmp exhausted tmpfs quota and was removed;
project-disk shared dependencies were reused with forced current library rebuild
and Cargo artifact-path verification. Full corrected debug comparison follows.

### Corrected Debug paired comparison completed

All 108 samples completed and all evaluated checks passed; raw and derived
reports are archived in `evidence/tui-perf-001/debug-paired-rebuilt/`.
Stream CPU medians fell from 11.249/11.279/11.016% to 7.825/8.260/8.089%
for histories 100/1000/10000. Idle and approval steady draw/diff/flush counts
are zero; stable animation layout/history rebuild counts are zero.
Input probes, Git, metrics-off idle, allocation, long-run and native acceptance
remain separate outstanding requirements. No whole-task completion is claimed.

### Release input-probe comparison completed

All 36 cases passed evaluated checks; archive: `evidence/tui-perf-001/input-paired-release/`.
Worst-round ordinary input P99 upper bound is 1.190ms; resize with 10000 history
entries is 66.975ms, within frozen release budgets. This is synthetic input-to-submit
evidence, not native pixel latency. Debug input comparison follows with the corrected binary.

### Debug input-probe comparison completed

All 36 cases passed evaluated checks; archive: `evidence/tui-perf-001/input-paired-debug/`.
Worst ordinary input P99 upper bound4.848ms; resize110.863ms, within frozen
debug budgets. Uses corrected P3 binary. Native terminal, Git freshness,
metrics-off idle, allocation and sustained/long-run workloads remain outstanding.

### Release metrics-off idle completed

All18 samples passed CPU/PSS budgets, with measured candidate CPU medians0.000%
across all history sizes (procfs resolution limited). Archive:
`evidence/tui-perf-001/idle-off-release/`. Debug metrics-off verification follows.

### Debug metrics-off idle completed

All 18 samples passed CPU/PSS budgets; measured CPU medians 0.000% across
all history sizes (procfs resolution limited). Archive:
`evidence/tui-perf-001/idle-off-debug/`. Both profiles now have metrics-off
idle evidence; Git and native-terminal behavior remain separate gates.

### Release large Git fixture measured

Six paired cases with10000 untracked files pass evaluated gates. CPU median
2.267%→0.976%;15 scans per30s steady window, max scan20.490ms, observed
frame-weighted snapshot age max1.994s. Evidence: `evidence/tui-perf-001/git-many-release/`.
Other fixtures, debug, workspace switch and external-change behavior remain open.

### Debug large Git fixture measured

Six cases passed evaluated gates: CPU median10.404%→4.471%,15 scans per30s,
max scan66.404ms and observed snapshot age max2.005s. Archive:
`evidence/tui-perf-001/git-many-debug/`. External-change/switch gates remain open.

### Release clean Git fixture measured

Six paired cases passed evaluated gates; CPU median0.438%→0.337%,15 scans/30s,
max scan1.150ms and observed age max2.001s. Archive:
`evidence/tui-perf-001/git-clean-release/`. Debug clean fixture follows.

### Debug clean Git fixture measured

Six paired cases passed evaluated gates: CPU median 1.960% to 1.858%,
15 scans/30s, max scan 7.364ms and observed age max 2.003s. Archive:
`evidence/tui-perf-001/git-clean-debug/`. Non-repository and switch coverage follows.

### Release non-repository fixture measured

Six cases passed evaluated gates. CPU median0.304% to0.334% (+10.0%,
within20% budget; not a speedup). Archive: `evidence/tui-perf-001/git-nonrepo-release/`.

### Debug non-repository gate remains unverified; main rebase

The completed six-case experiment is preserved in
`evidence/tui-perf-001/git-nonrepo-debug/`. Candidate round 2 fails the
conservative deadline P99 upper-bound check (62.754 ms maximum); exact P99 and
late deadline type are unavailable. CPU/PSS/input/stable preparation checks pass.
Mixed deadline instrumentation requires diagnosis; no favorable rerun supersedes
this failure. Task remains Active with acceptance incomplete.

Rebased all six local commits onto origin/main `55617ad7`, new tip `5b25d891`.
`git range-diff d355dd68..539adfd2 origin/main..HEAD` reports all six patches
unchanged; `git diff --check` and unmerged-path checks pass. Uncommitted evidence
and docs restored from stash `cf940fc8`; all untracked contents verified bytewise.
Existing binary receipts and measurements retain their original source SHAs;
new-baseline build/test and performance evidence are required before delivery.

### Animation deadline diagnostic refinement — local commit df7411d2

Added a separate due-visible-animation observation and bounded >50ms counter.
The original combined deadline histogram is retained. Summary/comparator tests
cover exact threshold ranks, no samples, malformed counts, and mixed/specific
evidence disagreement; all 14 Python evidence tests passed. This refines
measurement, not the rendering algorithm, and does not resolve the earlier
failed run or establish new performance acceptance.

Rebase workspace preflight: check and Clippy passed; tests failed with temporary
filesystem quota errors. Removed only this turn's mistakenly created 1.3GB
worktree target after approval. Redirecting TMPDIR inside the repository then
caused the non-repository fixture to discover its ancestor Git repository;
that isolated environment failure is retained in the test log. Subsequent
tests use the restored default temporary directory and the disk-backed shared
Cargo target. Full fresh preflight remains required.

Validation after diagnostic refinement: full `./scripts/release_preflight.sh`
passed (exit 0; `/tmp/talos-perf-deadline-preflight.log`), including locked
workspace check/Clippy/tests and SDK fixture. TUI has 674 unit tests plus two
integration and two doc tests. `cargo xwin check --locked -p talos-tui --tests
--examples --target x86_64-pc-windows-msvc` passed (exit 0;
`/tmp/talos-perf-deadline-xwin.log`). Fourteen Python evidence tests passed.
The old failed experiment remains failed under the updated comparator.

New source-bound Debug candidate: `df7411d2`, binary SHA-256
`b5ab0cc7176b3b6bc502aec2ff407346d22ead3e03b6dfdf525b46f27d6de41c`.
Receipt: `/tmp/talos-candidate-df7411d2/receipt.json`. A new non-repository
three-round paired animation experiment is running in
`/tmp/talos-perf-nonrepo-deadline-df7411d2`; no results claimed yet.

### Rebased candidate specific animation measurement

The new six-case non-repository Debug matrix completed successfully; all evaluated
checks pass. Archive: `evidence/tui-perf-001/nonrepo-deadline-df7411d2/`, including
source-bound binary receipt and raw reports. Across three candidate rounds,
602 visible-animation observations have zero >50ms samples; maximum 4.399247ms.
CPU median baseline1.747760% / candidate1.517662%; candidate peakPSS46312KiB.
Input P95/P99 and zero stable layout/history rebuild checks pass.

This is independent evidence for df7411d2, not a retrospective pass for the old
failed run. The old delay's cause remains unknown; dependency and host drift
limit attribution across rebases. Next: workspace-switch matrices, Git external
freshness/stale completion oracle, high-rate and long-run evidence, P4 decision,
ANSI/native acceptance, final source-bound CI/review and delivery.

### Debug workspace-switch matrix completed

Six cases at history10000 pass CPU/PSS/input gates; baseline15ef42c1 and
candidate df7411d2 alternate 10000-untracked and clean workspaces every second.
CPU medians6.765962% to3.488965%; peak candidatePSS61084KiB. Archive and both
source receipts: `evidence/tui-perf-001/switch-debug-df7411d2/`. This establishes
observed switching and latency, not the separate stale-result or freshness
oracles. Release switch baseline must still be built from its fixture commit.

### Release workspace-switch matrix completed

Six Release cases also pass CPU/PSS/input gates, source-bound baseline15ef42c1
and candidate df7411d2. CPU medians1.509232% to0.605813%; peak candidatePSS40170KiB.
Each candidate round observes31 steady workspace changes. Archive:
`evidence/tui-perf-001/switch-release-df7411d2/`. Both profiles now have formal
switch measurements; external mutation freshness and stale-result behavior
remain separate acceptance requirements. Runtime JSON confirms all new metric
fields despite an inconclusive optimized-binary literal search.

### Evidence recoverability after rebase

`evidence/tui-perf-001/source-history/pre-rebase.bundle` preserves all six original
commit objects referenced by old binary receipts. `git bundle verify` passes;
the prerequisite d355dd68 is on main. Export instructions and SHA-256 are
archived beside the bundle. No additional remote branch was created.

### Actual-loop EOF and Git behavior oracles (local, uncommitted)

Added test-only frame timestamp/observer hooks; no production rendering or
scheduling changes. Three focused tests passed:

- Legacy EOF: 1000 tokens each appear exactly once in order; final token reaches
  the submitted buffer before a delayed Exit, with no input or explicit End.
- External Git freshness: real gix-created repository, external untracked-file
  creation/removal, submitted clean/dirty/clean frames without input. Each
  external mutation publishes within the frozen5s freshness limit.
- Successful obsolete scan: hold old worker until new-workspace frame is
  submitted, then let it return a successful old branch result. Observe every
  subsequent submitted frame until new scan publishes; no old branch appears.
  Exactly two actual scans launch and the new snapshot remains current.

Full TUI suite passes677 unit tests plus2integration/2doc tests; log
`/tmp/talos-perf-behavior-tests.log`. These are software-buffer and lifecycle
oracles, not native pixel/IME or long-run evidence. New Windows test cross-check
and stable-candidate full preflight remain to be recorded before commit.

Behavior-oracle validation follow-up: `cargo xwin check --locked -p talos-tui
--tests --target x86_64-pc-windows-msvc` passed (exit0,
`/tmp/talos-perf-behavior-xwin.log`). Full `./scripts/release_preflight.sh`
passed (exit0, `/tmp/talos-perf-behavior-preflight.log`), including governance,
locked workspace check/Clippy/tests and SDK fixture. All runtime changes in
this checkpoint are behind cfg(test); performance candidate df7411d2 remains
the production source used by the archived measurements.

### Sustained high-rate Debug evidence and failed baseline

Original P0 fixtures are fixed20Hz. Prepared exact15ef42c1 source with an explicit
fixture-only200Hz rate patch; all TUI runtime sources verified bytewise unchanged.
Baseline200Hz plus50ms input probes reached65s watchdog at~97.37% CPU and was
terminated; preserve incomplete evidence in `evidence/tui-perf-001/highrate-baseline-timeout/`.
No valid CPU/PSS paired comparison exists for this workload yet.

Candidate initially completed two samples but third report write exceeded/tmp
user quota; archive `evidence/tui-perf-001/highrate-candidate-quota/` includes the
truncated failure. Moved only this task's six binary-copy directories (~2.8GB)
to system-disk target/tui-perf-artifacts, retained original symlinks and verified
all SHA-256 digests. No artifacts were discarded.

Sampler now supports explicit input interval(default500ms), fixed deadlines and
skipped-probe count; default interval preserved. Final candidate-only three-round
200Hz/50ms Debug matrix completed, steady output6034/6028/6033 events, no skipped
input probes, P95/P99 upper bounds15.935096/12.489444/12.259310ms (within20/35ms).
Total probe counts include drain time, not solely the steady window. Archive:
`evidence/tui-perf-001/highrate-candidate-debug/`. This is absolute input-budget
evidence, not paired CPU/PSS or final-pixel acceptance. Release highrate, longrun,
allocation and P4 evidence remain open.

### Release high-rate matrix completed

All six paired cases pass CPU/PSS/input budgets at200Hz configured output and
50ms fixed-deadline input probes. CPU medians34.657680% ->7.066650%; candidate
peakPSS53965KiB, worst inputP99 upper5.095588ms. Baseline/candidate each process
~6010steady output events per round; zero skipped input probe deadlines. Archive
`evidence/tui-perf-001/highrate-release/` includes fixture-only baseline patch,
both build receipts and raw reports. Debug baseline timeout is not superseded.

Sixteen Python evidence tests pass, including an actual PTY child test verifying
20ms probe delivery is not throttled by sampler polling and invalid interval
rejection before launch. No production Rust changes in this sampling checkpoint.
Longrun/allocations, repeated transcript commits/P4, ANSI/native and remote gates
remain open.

### Actual allocator profiling established

Isolated Heaptrack1.5.0 CLI from verified Arch package collected three Release
animation/history10000 pairs, baseline52263c62 vs candidate df7411d2. Archive:
`evidence/tui-perf-001/heaptrack-animation/`, including raw profiles, allocator
histograms, stacks, massif snapshots, exact runner and binary receipts.
Median whole-run allocation calls851478 ->816698; requested bytes127667737
->124076306. Both peaks31.32MB (rounded), so peak heap did not improve.
Histogram counts were checked against profiler totals. Whole-run includes
initial history construction; stack evidence includes history projection and
ScrollbackLine cloning but does not isolate steady frame cost. No P4 conclusion
is made until repeated transcript commits are measured. Profiler overhead and
shutdown-live memory are explicitly distinguished from ordinary CPU/leaks.

### Latest-main rebase and repeated transcript evidence

Rebased ten local patches onto origin/main6c81390a; range-diff reports all
unchanged, uncommitted work restored. New local head e42b9c9c; no push yet.
Added identical transcript-commit fixture to baseline15ef42c1 source export
and current candidate. Three Release pairs at10000 history/1complete message
per second pass CPU/PSS/input budgets. Evidence:
`evidence/tui-perf-001/transcript-release/`. CPU median5.099875% ->2.791949%,
peak candidatePSS68918KiB, worst inputP99upper0.594405ms. Candidate31history
rebuilds vs baseline62 per steady window, both94output events.

History rebuild consumes81–82% of candidate draw duration (nested timers),
so repeated revision changes establish a P4 hotspot despite aggregate budgets
passing. P4 remains unimplemented; next allocator profile and conservative
incremental projection design must preserve grouping/tail mutation, anchors,
selection/logical lines and full-projection oracle. This35s growing workload
is not long-run bounded-memory acceptance.

Seventeen Python evidence tests and locked TUI replay example check pass after
rebase. Release fixture binaries built with pinned toolchain and symbols;
source/patch/binary receipts archived. Native terminal, longrun, ANSI equivalence,
final full gates, exact-head CI and independent review remain open.

### Transcript revision allocator profile completed

Three Heaptrack Release pairs complete; archive
`evidence/tui-perf-001/heaptrack-transcript/`. Whole-run median allocation calls
26139343 ->13386688, requested bytes3793180204 ->1947615179. Histogram totals
match profiler counts. All peak heap values43.56M rounded, no peak improvement.
Totals include initialization/teardown and profiler overhead excludes CPU/PSS
from normal budgets. This corroborates revision-dependent allocation cost;
P4 implementation/oracles and bounded longrun stability remain outstanding.

### P4 incremental projection — local implementation under validation

Repeated transcript timing and allocator evidence above activates P4. The cache
now retains a stable prefix and reprojects from the beginning of the final
projection unit, including its incoming pending-call count. This backtracks over
an entire final poll group, including a pending poll subsequently completed or
failed. Width changes and any expansion/reasoning edit conservatively rebuild
all history. No per-revision journal or second formatting algorithm is added.

Rows/logical lines use Arc<Vec>; live old snapshots remain immutable via COW.
At frame preparation, the consumed previous hit-map snapshot is released before
updating the cache, avoiding deep copies of the stable prefix in the normal path.
Cache capacity follows retained transcript size; this is not a fixed memory cap
on conversation history. Existing history_rebuild metrics now count projection
refreshes (full or tail), not exclusively full-history reconstruction.

The full projection remains the oracle:256 successive state comparisons across
four widths cover pending/adjacent/parallel polls, terminal observations,
expansion edits, anchored views, copy text and retained old snapshots. A separate
prefix-pointer test verifies append refresh does not clone stable text. Full
TUI679unit+2integration+2doc tests pass; xwin MSVC --tests --examples passes.
Logs:/tmp/talos-p4-tui-tests.log,/tmp/talos-p4-xwin.log. Release source-bound
before/after measurement is next; no speedup or P4 acceptance claimed yet.

### P4 Release paired result

Three paired P3/P4 runs with identical fixture/dependencies pass listed
CPU/PSS/input gates; archive `evidence/tui-perf-001/transcript-p4-release/`.
CPU median2.828760% ->0.067683%,candidate peakPSS33532KiB. Three steady history
refresh totals0.232/0.276/0.291ms, draw33.953/23.139/24.173ms; both process94
output events. Worst candidate inputP99upper10.413224ms in first round, within
20ms budget but retained as an unexplained long tail; remaining rounds<0.45ms.
No pixel, longrun or whole-task acceptance claim. Full workspace preflight is
running; evidence archives contain complete binary receipts and source patches.

### Full-gate fixture failure and bounded repair

First P4 preflight exited101 in talos-config's existing
store_unset_builtin_provider_clears_credentials (272passed/1failed), with
"builtin provider must not have resurrected credential". Source inspection
shows an unlocked environment-reading api_key assertion while other tests set
ANTHROPIC_API_KEY under ENV_MUTEX. Persisted removal does not prohibit host
credential fallback, so the assertion tested the wrong boundary.

Bounded test-only repair checks active_provider_config.api_key absence and
removal of the last-key credentials file, without changing runtime behavior.
SOP collaboration local-test-fix/bounded-fixture rules apply; keep this logical
change separate in commits. The first attempted exact filter ran zero tests
and is not evidence; a subsequent file-read assertion failed because last-key
removal deletes the file. Corrected fully qualified isolated test executes1and
passes. Logs:/tmp/talos-p4-preflight.log,/tmp/talos-p4-config-isolated-fixed2.log.
Fresh fullpreflight follows in /tmp/talos-p4-preflight-fixed.log; failed evidence
is retained. Evolution lesson3 updated with the persisted/environment boundary.

Full P4 preflight after the test-only repair passed(exit0), including locked
workspace check/Clippy/tests and external SDK fixtures:
/tmp/talos-p4-preflight-fixed.log. Config Windows test cross-check also passes
/tmp/talos-p4-config-xwin.log. TUI Windows --tests --examples passed earlier
against unchanged P4 runtime source. Test repair committed18e9e117;
transcript/allocator fixture evidence committed3bedbe23. These are local commits,
not pushed completion evidence. P4 allocator follow-up, Debug/longrun coverage,
changed-cell/ANSI/native checks and final remote gates remain open.

### P4 fixed-history repeated replay

Three125s candidate runs(5s warmup),10000fixed history, animation+500ms input
probes complete. Archive `evidence/tui-perf-001/p4-bounded-longrun/`. First/last30s
median PSS deltas-3165/-1/+1KiB; no sustained upward trend observed, not a heap
leak proof. CPU0.2504/0.2593/0.2590%; inputP99upper worst0.503696ms. All steady
history rebuild/layout/Git counts0. Animation deadline799/800/800samples have
5/3/1over50ms: P99 passes; individual long tails retained. Resizing/heap/growing
history/native evidence still needed. Corrected two prior transcript report
READMEs from erroneous100x30 to sampler's actual120x40 dimensions; raw data unchanged.

### Workstation resource incident and guarded ANSI validation

User reported severe workstation CPU/memory pressure. Kernel logs confirm a
2026-11-11 global OOM killing rustc and WeChatAppEx; several rustc/linker processes
were resident. Attribution to this session alone is unproven. Build jobs=2 did
not bound aggregate host load. All known task processes were confirmed terminal
before further work; no unguarded build/test or sampling is permitted after this
incident. Future heavy validation uses a dedicated user systemd service with
CPUQuota=100%,MemoryHigh=2G,MemoryMax=3G,MemorySwapMax=0,TasksMax=128,Nice=10,
CARGO_BUILD_JOBS=1,RUST_TEST_THREADS=1. Actual cgroup files verified100000/100000,
2147483648,3221225472,0,128 respectively. Never increase limits to force a pass.
Resource-capped execution is validation evidence, not comparable performance data.

Actual terminal submission is now shared with a capturing Crossterm backend.
Cached and complete planning paths emit identical ANSI bytes across the existing
28-state transition oracle. Animation cell differences remain inside preview
across4widths/2modes/63frames; layout/history/cursor remain stable. Unchanged
frames emit no bytes; an injected write failure still ends synchronized output,
flushes and propagates the error. This is byte-level evidence, not native pixels.

Initial new idle fixture erroneously omitted text in its second frame, and the
failure test used an unavailable backend.writer accessor; both corrected.
Guarded fullTUI test passes682unit+2integration+2doc, exit0,40.067s wall,
31.198s CPU,~2GiB peak,0swap. Log:/tmp/talos-ansi-guarded-tests.log.
Current submission extraction remains uncommitted; fresh xwin/fullworkspace
and independent review still required. P4 allocation/Debug follow-up, heap and
native terminal acceptance and remote delivery remain open.

### Guarded validation follow-up and static preparation audit

Guarded xwin MSVC check with `--tests --examples` passed (exit0; log
`/tmp/talos-ansi-guarded-xwin.log`). Fresh project and collaboration validators
passed with zero warnings; `git diff --check` passed.
The full preflight passed governance/check/Clippy but was deliberately stopped
during workspace test compilation: cgroup `memory.events` recorded623556 high
events, no max/OOM events, while host I/O pressure was elevated. This does not
prove the service was the sole source of host pressure. Unit is confirmed
inactive; runtime17min41.417s, CPU17min28.445s, peak2GiB, swap0. Log
`/tmp/talos-ansi-guarded-preflight.log` records SIGTERM. Wrapper exit0 and service
success are not evidence that preflight passed. Current full-workspace gate
remains incomplete; do not raise limits or restart the same expensive command
without addressing repeated reclaim.

Static inspection of `app/frame.rs` confirms cached splash lines, composer
natural height, preview plans, layout and history projection are reused.
Status formatting reads the Git snapshot. Full buffer construction/diff remains
intentional; tool-preview planning, panel/queue height queries, composer painting
and cursor calculation can still execute per drawn frame. No claim is made that
all static component work or frame allocations have been eliminated. Existing
region/ANSI tests cover observable equivalence; allocation and native evidence
remain required before closing the task.

### Projection snapshot lifecycle regression

Added `repeated_reflow_releases_obsolete_snapshots_after_view_is_dropped`:
128 cycles alternate widths7/80/120/1, expand existing reasoning and append
history. Every new projection matches the full oracle; weak references verify
the previous rows/logical-text snapshots survive while a consumer holds them
and are deallocated after that consumer drops them. The cache retains only its
current snapshot. This covers ownership retention, not allocator fragmentation
or whole-process heap stability, and does not replace the remaining heap probe.

Focused locked TUI test passed1/1 (682 filtered) in a dedicated resource guard:
CPUQuota100%, MemoryHigh2G, MemoryMax3G, swap0, tasks128, jobs1/testthreads1.
Service exited normally0, runtime15.990s, CPU14.472s, peak2GiB, swap0. No full
workspace pass is implied; Windows cross-check of this new test remains pending.

### Delivery convergence checkpoint (supersedes pending statements above)

Current implementation tip: `a04ab9e0` (local, not pushed). New snapshot test MSVC
`cargo xwin check --locked -p talos-tui --tests --examples --target x86_64-pc-windows-msvc`
passed under the same guard: normal exit0,2.202s, peak837.2M, swap0.

P4 allocation follow-up is now recorded in
[heaptrack-p4-guarded](evidence/tui-perf-001/heaptrack-p4-guarded/README.md).
Three serial alternating P3/P4 pairs: median allocation calls13386688→765612;
reported peak heap43.56M→26.11M. Six normal exits0, total3min50.941s, guard peak
453.8M/swap0. These whole-run capped profiler observations are not CPU/latency
comparisons or leak proofs. The sampled output window differs by one update in
two P3 cases; exact business-event counts were not captured. Source receipts bind
the binaries, with P4 preceding the shared ANSI submission extraction.

The user identified excessive validation time. Consolidate remaining work into
the following delivery gates; do not add experiments without an uncovered
requirement, regression, or review finding. Reuse existing evidence within its
actual source/platform/workload scope; do not erase the original acceptance.

| Gate | Existing evidence | Remaining action |
| --- | --- | --- |
| Runtime implementation P1–P4 | Local commits, scheduling/Git/projection oracles, ANSI equivalence and snapshot lifecycle tests | Independent review of final candidate; repair findings only |
| Performance and memory | Debug/Release P0–P3 pairs, P4 Release transcript pairs, capped P4 allocation comparison, fixed-history repeated replay | P4 Debug follow-up and growing-history/resize heap stability remain unverified; combine required coverage instead of expanding matrix |
| Current-source local checks | Full TUI682+2+2 before test-only addition, new test1/1, current MSVC tests/examples, check/Clippy before test-only addition | Full workspace preflight was interrupted during linking; solve bounded resource execution, then one final preflight |
| Native terminal behavior/resources | Byte/cell oracles and synthetic PTY only | Ghostty on Wayland+niri plus another supported terminal; native Windows unavailable; emulator accounting, IME/clipboard/modal/input acceptance remain open |
| Documentation/delivery | README responsiveness entry and LOCAL-DEV profiling/resource instructions; this owner and evidence archives | Push stable candidate, implementation PR, exact-head CI/independent review, then owner/Board/Issue synchronization; never mark Complete without required gates |

No test/profile process remains live at this checkpoint. Keep task Active and
claim effective; these gaps are not a completion claim or an approved reduction
of scope. Native unavailability must be reported as pending acceptance rather
than compensated by more synthetic tests.

### Workspace validation failure diagnosis (2026-11-12)

Single-CPU affinity was added without raising resource limits. The first run
failed rustfmt on the new snapshot assertion; formatting corrected. The second
passed formatting/check/Clippy and completed linking, but failed CLI
`i255_plugin_e2e`: the actual tool message was `execution timed out after 250ms`.
Full log: `/tmp/talos-preflight-affinity-fixed.log`, exit101. Runtime6min40.189s,
peak2GiB/swap0; memory.high events still occurred. Affinity is not proof that
reclaim pressure is solved.

Targeted Cargo retry passed2/2 but rebuilt dependencies due to a different
feature selection (5min12.284s); do not call this a cache-only retry. Running the
previous workspace test executable directly then passed2/2 in1.722s, peak69.9M,
swap0. The CLI binary path can have been overwritten by the targeted build, so
this is not an immutable exact-binary reproduction of the original failure.
No runtime timeout, permission rule or assertion was relaxed. Failure is retained
as an unreproduced timeout, not declared fixed; full workspace gate remains open.
All three service handles are terminal; attempted stops arrived after completion.

### Service locale correction

Full preflight on `43305a58` exited101 after8min13.525s. Tools1146/1147 passed;
`hasconfig_non_path_key_still_triggers_url_prohibition` expected English Git
stderr while the user service manager supplied `LANG=zh_CN.utf8`. A minimal
native Git2.56.0 reproduction rejected the same config in both languages, but
Chinese diagnostics did not contain the English assertion substring.
The exact existing tools test executable passed the failing test1/1 with
`LC_ALL=C LANG=C` under the unchanged resource limits (46ms, peak4.9M, swap0).
No product behavior or test assertion changed. LOCAL-DEV now pins diagnostic
locale in its guarded command example. A full rerun is in progress; this focused
result does not complete the workspace gate.

### Current implementation full local gate passed

`./scripts/release_preflight.sh` completed normally with exit0 and its explicit
`release preflight: passed` marker. Archive: `evidence/tui-perf-001/local-preflight/`.
Implementation source43305a58; intervening6c51db7e changes documentation only.
148 Rust test-result summaries:5315 passed,0 failed,0 ignored; formatting,
workspace check/Clippy, governance/claims and external Runtime SDK fixture passed.
Guard: CPU100%/affinity0, memory high2G/max3G, swap0, tasks128, jobs1/testthreads1,
Nice10, LC_ALL=C/LANG=C. Runtime8min28.678s, CPU4min7.841s, reported peak2G/swap0.
The previously failing Git diagnostic assertion passed in this full run.
The earlier plugin timeout remains retained evidence, not a claimed product fix.

This satisfies the current local workspace gate; do not repeat it solely for
status prose. Remaining performance/native/remote requirements in the delivery
convergence table still apply. No validation process remains live.

### Current-source guarded Debug result

Archive `evidence/tui-perf-001/p4-debug-guarded/` binds source57603793 and its
Debug binary. Six serial transcript/resize runs completed normally; no other
heavy task ran. Transcript input P99 upper worst9.295705ms fits Debug budget.
Resize round1 P99 upper202.267295ms exceeds200ms; round2 P95 upper176.078983ms
exceeds150ms. Retain failures; CPU-capped candidate-only data cannot isolate a
runtime regression from scheduling. Debug follow-up is measured but resize
acceptance remains unproven. Do not rerun until a specific diagnostic question
and comparable conditions are established. Growth/resize heap proof, native
terminals and remote delivery still remain. No live processes at checkpoint.

Existing-data resize attribution narrows the issue to full-width history reflow:
about97% of nested draw wall time is history rebuild, versus0.07–0.11ms layout
per round. `p4-debug-guarded/resize-attribution.json` records the calculation.
This is wall-time attribution, not evidence separating scheduling from CPU.
Keep width reflow correctness and existing oracles; further work should target
that path rather than unrelated animation/Git scheduling. No new run was needed.

### Rejected fitting-line fast path

A bounded single-segment/no-fill fast path passed42 projection tests, but three
same-cap alternating Debug resize pairs did not demonstrate benefit: median
history rebuild7430.947→8033.446ms; candidate P95 upper169.451–180.064ms still
misses150ms. Baseline misses too, and host/order limitations remain. Removed the
uncommitted runtime/test patch; retained it and raw results under
`evidence/tui-perf-001/fit-path-evaluation/`. Runtime is back to the source covered
by the passing full preflight. No new full test run is warranted for this revert
to that exact source. Resize acceptance remains unresolved; do not claim a fix.

### Draft stage candidate published

Implementation branch pushed through8afbbe4c; draft PR#922:
https://github.com/wjhuang88/talos/pull/922 . This provides a reviewable stage,
not completion or merge authorization. Resize-budget misses, growing/resize heap
acceptance, native terminals and exact-head CI/review remain open. User was asked
to arrange Ghostty acceptance; no result has been received at this checkpoint.

### Remote stage gate observation

At draft head8afbbe4c, CI run43134873925 has passed Linux Desktop, Windows
installer and change classification; Linux workspace and Windows jobs remain
running at this observation. Remote reconciliation job130907712181 failed because
`docs/reference/ISSUE-DOC-CODE-STATUS-2026-09-30.md` omits open Issue#919
(GIT-COMPAT-001, gix-native Git preflight compatibility). This is global Issue
matrix drift, not a TUI claim failure. Per ADR-071 and collaboration SOP, retain
it as a governance reconciliation residual owned by that matrix/workflow; do not
widen this implementation slice to Git compatibility. This does not waive
exact-head code CI or independent review. No merge is authorized by this note.

Existing P4 heaptrack massif data was summarized without another replay.
Three candidate growing-history runs show identical24250-byte median live-heap
increase between elapsed5–10s and25–30s; evidence and reproduction script are in
`heaptrack-p4-guarded/`. Ongoing history growth and missing exact event alignment
prevent interpreting the delta as leak/non-leak proof. Repeated-resize heap
coverage remains absent. Both Linux and Windows Desktop CI jobs have now passed;
workspace jobs are still running at this observation.

### Independent review follow-up: input exit flush (2026-11-12)

Independent Agent review of published `8afbbe4c` against target `f61a15cd`
requested changes: input EOF and exit-key paths bypassed the final dirty-frame
submission while stream content was waiting for its coalescing deadline. The
working candidate routes both paths through the existing `UiOutput::Exit`
finalization and loop-top submission. A paused-clock regression covers ordered
and legacy streams, each with input EOF and Ctrl+C at 1 ms before the stream
frame deadline. This fix is not yet validated or delivered.

The guarded focused test (`cargo test --locked --workspace --lib
input_exit_submits_coalesced_content_before_restore`) exited 101 before tests:
the new test omitted the qualification of `std::time::Duration`; corrected
locally. Runtime 2m18.166s, CPU 1m29.616s, peak 2 GiB, swap 0. During compilation,
`memory.events high` reached 272811 and host I/O pressure was high. A stop was
requested, but the service had already exited with that compiler error. No
successful test result is claimed; no immediate rebuild or raised resource cap.
The review also explicitly retains Debug resize budget, growing/resize heap,
and native-terminal acceptance gaps. Shared GitHub identity means independent
Agent review is not independent human authorization.

Independent Agent follow-up reviewed the working diff against `905a05c9` and
accepted the input-exit repair statically: both paths finalize legacy/ordered
streams, then `should_exit` overrides the coalescing deadline for submission.
The reviewer confirmed the four 1 ms regression scenarios target the original
defect. This is source review only, not successful test execution or final-head
review. At the next resource check, host I/O PSI remained high (some avg10 89.81,
full avg10 81.19); compilation was not restarted. The derived acceptance tracker
now explicitly separates the old passing implementation from this unvalidated
fix and records the P4 Debug resize budget misses.

### Input exit regression passed (2026-11-12)

Source `18965ba2` strengthens key-exit isolation: input stays open after Ctrl+C,
so EOF cannot rescue a broken key path; a 100 ms paused-clock timeout guards
liveness. The guarded focused command above then completed normally with exit 0:
`app::app_tests::input_exit_submits_coalesced_content_before_restore` passed,
covering all four ordered/legacy × EOF/key cases in 0.02 s (683 TUI tests filtered).
Total service runtime 3m53.166s, CPU 2m26.339s, peak 2 GiB, swap 0. Compilation
also rebuilt provider/MCP library test targets despite the name filter. Memory
high events rose from 9927 to 55773; a stop was requested, but the service had
already completed normally before it arrived. This result proves the focused
regression only, not the full workspace suite or final performance budgets.
No immediate repeat is needed. Keep the stable-batch full gate and MSVC checks
outstanding for changed code; native acceptance and resize/heap gaps remain.

### Resize diagnosis with resource counters (2026-11-12)

Sampler `8994abc7` adds opt-in leaf cgroup-v2 observations with unavailable-data
handling; three fixture tests and two PTY probe tests passed. Developer usage,
overhead, path privacy and attribution limits are documented in LOCAL-DEV.
A single existing-binary run is archived in `resize-cgroup-diagnostic/`: Debug
source57603793, normal exit0,37.953s runtime,5.221s CPU,424.7MiB peak/no swap.
No sampled leaf throttling, memory-high events or I/O pressure increase; maximum
input-to-submit82.538561ms. The earlier150/200ms misses were not reproduced in
this same binary. This does not identify their cause or satisfy final-source,
repeated-round acceptance. Preserve previous failures; no speculative additional
runtime optimization is justified by this observation alone.
