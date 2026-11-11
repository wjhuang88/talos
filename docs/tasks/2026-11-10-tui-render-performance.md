# TUI-PERF-001：TUI 事件驱动渲染与资源优化

> Document status: Active / Claimed (proposed in #910; ineffective until target-branch merge)
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
| Authorization Evidence | Proposed only; target-branch merge and independent review pending |
| Implementation PR | Not started |
| Last Updated | 2026-11-11 |
| Handoff / Release Condition | Full published acceptance including native terminal evidence; incomplete platforms remain Review |
