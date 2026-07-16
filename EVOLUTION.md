# Evolution Lessons

## Purpose

Record reusable lessons from failures, corrections, and unexpected successes. Check here before
repeating known mistakes.

## Lesson Index

| # | Area | Lesson | Added |
|---|------|--------|-------|
| 1 | Terminal | Cooked mode Ctrl+C 需要 `exit_token` 模式 | I005 |
| 2 | Terminal | Raw mode 手动处理 ANSI 转义序列是错误路径 | I003 |
| 3 | Testing | 环境变量测试必须用 Mutex 防止并行干扰 | I004 |
| 4 | macOS | `/tmp` 是 `/private/tmp` 的符号链接 | I004 |
| 5 | Architecture | Mock LLM 是测试基础设施，应尽早实现 | I005 |
| 6 | Delegation | 子代理适合独立模块，复杂集成需主代理协调 | I003-I005 |
| 7 | Security | `setrlimit` 需要 unsafe，必须有文档说明 | I004 |
| 8 | TUI | UTF-8 字符边界问题需要字符索引而非字节索引 | I008 |
| 9 | TUI | EventStream 需要定期重绘间隔防止界面冻结 | I008 |
| 10 | SQLite | 多 crate 共享 SQLite 需要统一 rusqlite 版本 | I008 |
| 11 | Process | "单测全过 + 勾选验收" ≠ 完成；需端到端运行时证据 | I008 |
| 12 | Storage | 自包含 SQLite 需要 ADR 明确例外与运行时边界 | I008 |
| 13 | Testing | 自定义存储目录必须隔离派生索引文件 | I008 |
| 14 | Process | 并行委派时 `git stash` 会吞掉兄弟任务的未提交改动 | R0 |
| 22 | Git | Agent 提交的 commit 必须带 `[model:<model-id>]` 后缀 (AGENTS.md Git Rule 2); 缺失是 governance drift | I041 |
| 15 | Delegation | 并行委派代理会"顺手"实现兄弟任务范围，需用 marker 协议隔离 | I009 |
| 16 | Planning | `task()` 调用必须二选一：`category` 或 `subagent_type`，不可同时给 | I009 |
| 17 | TUI | visual-engineering 任务在 R0 级别并行委派下 30 分钟不够（结构+消费两个 scope） | I009 |
| 18 | Governance | 更新 skill 后必须重新跑 governance validator 并修复 conformant 漂移 | I013 |
| 19 | Evolution | 持久化任何"用户输入上下文"前必须 byte-cap，dedup 必须含内容指纹 | I008/I015 |
| 20 | Evolution | `Signal.context` 字段语义 = signal 周围短窗口,不是整条 user message; 7470ac5 是 defense layer,真治根在 I021 | I008/I021 |
| 21 | Process | 不要擅自更改已经与用户确认的设计决策 | I023 |
| 22 | Code Quality | 库 crate 必须用 thiserror 而非 anyhow；clippy -D warnings 必须通过 | I023 |
| 23 | Code Style | Rust 2024 let-chains (`if let X && let Y`) 替代嵌套 if-let 金字塔 | I023 |
| 24 | TUI | 流内容应按完整 block 渲染（积累 buffer → finalize 时一次性输出），不要逐行分割 | I023 |
| 25 | Safety | 外部 C/Native 依赖的 panic 必须 `catch_unwind` 捕获 + 降级，不能静默崩溃 | TUI-006/CODE-001 |
| 26 | Governance | Partial 状态不能成为静默扩展已发布迭代基线的理由 | I033 |
| 27 | Safety | 工具装饰器必须透传安全与来源元数据，权限判断不能退化为名称推断 | I034 |
| 28 | Process | 并行审计是 closeout 前的必要闸门；自我比较 bug 能穿透单元测试 | I043/I044 |
| 29 | Config | 机密字段必须显示层屏蔽，不得在持久化层 skip_serializing；否则用户写进文件的 key 会被静默擦除 | I045 |
| 30 | Session | 生命周期操作必须从全新启动路径追溯——Model switch 需要 ensure_persisted() 处理首轮无 session 的边界情况 | I045 |
| 31 | Build | `ring` + `cargo-xwin` Windows ARM64 交叉编译失败, musl 替代方案 | I046 |
| 32 | Governance | 共享数据集变更后必须重新验证, closeout 证据必须反映最终提交状态 | I046 |
| 33 | Agent | Agent 收到新需求时不应立即中断当前任务, 通过 todo 工具捕获并规划执行节奏 | I045 |
| 34 | Config | 静态目录数据必须对照上游实况核验；"格式看起来合理"的条目 ID 不是证据 | 2026-07-03 |
| 35 | Testing | 触及 `Config::save()`/`$HOME` 的测试必须从第一次编写起就重定向 HOME，且必须用跨模块共享的单一 Mutex，不能各模块各建私有锁 | 2026-07-03 |
| 36 | Config | `Config::load()` 不应做"可执行性"校验；否则损坏的磁盘配置会挡住向导/`config set` 自我修复路径 | 2026-07-03 |
| 37 | Governance | ADR 冲突是 change-control gate，不是 Agent 永久拒绝用户产品需求的授权 | 2026-07-10 |
| 38 | Architecture | 单消费者 channel 拓扑不等于单数据流；必须审计排序域、生命周期权威和持久化写者 | I115 |

## Lessons

### 38. 2026-07-11 - Single-consumer topology does not prove semantic single flow

- Trigger: ARCH-032 reported zero broadcast channels and no deviations, but the user later observed
  dropped content/turns around thinking and tool output.
- Symptom: Text deltas lived in a nested `StreamMessage` receiver while tool/status/reasoning used
  `UiOutput`; provider `TurnEnd` and session `TurnCompleted` both drove terminal behavior; CLI modes
  also persisted or reconstructed the same turn differently.
- Root cause: The audit counted producers/consumers but did not identify independent ordering
  domains, authoritative lifecycle ownership, or durable write ownership. A later FIFO event could
  close a receiver that still held earlier text even though every individual channel had one
  consumer.
- Fix: ADR-039/ARCH-033/I115 introduced ordered `TurnEvent` envelopes, flattened live content onto
  one `UiOutput` FIFO, made session completion authoritative, moved successful turn persistence to
  the actor, and converged CLI/RPC surfaces on the session protocol.
- Prevention: Architecture audits must prove (1) one ordering domain per causal stream, (2) one
  lifecycle authority, (3) one durable writer, and (4) replay/surface parity; channel counts alone
  are insufficient.
- Promoted to rule/check: ADR-039, ARCH-033 acceptance tests, and the semantic follow-up in
  `docs/reference/ARCHITECTURE.md`.

### 37. 2026-07-10 - ADR conflict routes to change control, not permanent product rejection

- Trigger: TUI-029 / GitHub #26 requested that already-visible thinking text enter static history.
  An external agent formally rejected the requirement because ADR-034 v3 kept display transient;
  the maintainer then explicitly stated that the feature should be implemented.
- Symptom: The owner doc changed from decision-required to Rejected and claimed there was no user
  evidence, even though the issue itself was a concrete user request. The implementation gate was
  correctly identified, but the product decision was overreached.
- Root cause: The agent conflated two different conclusions: "do not implement before revising the
  ADR" and "the maintainer does not want this feature." ADRs record current decisions and reversal
  triggers; they do not grant an executor authority to permanently reject a new maintainer-directed
  acceptance target.
- Fix: Preserve the original rejection as history, apply `CHANGE-CONTROL.md`, revise ADR-034 to v4,
  append a supersession note to completed TUI-020, and move TUI-029 to Ready for Implementation with
  explicit security, persistence, export, semver, test, and runtime gates.
- Prevention: When a requested feature conflicts with an ADR, stop production implementation and
  produce an impact/revision packet. Use Rejected only when the maintainer rejects the product
  outcome or the request violates an immutable hard constraint. Treat explicit maintainer direction
  as new evidence for the ADR's reversal trigger.
- Promoted to rule/check: `docs/sop/CHANGE-CONTROL.md`, ADR-034 v4, and TUI-029 activation gate.

### 36. 2026-07-03 - `Config::load()` 混淆了"可解析"和"可执行"两层校验

- Trigger: 修复 I085 Stage 2 gap-fix 引入的测试数据污染真实 `~/.talos/config.toml`
  后，用户运行 `talos` 命中 "invalid configuration: 'model' is required" 硬崩溃，
  质疑"没有 model 难道不该打开交互向导？"
- Symptom: 三个模式入口（TUI/print/RPC）在 `Config::load()` 之后都各自写好了
  `config.model.is_empty()` 的优雅处理（首次设置向导 / 友好提示），但只要磁盘上的
  `config.toml` 文件存在且 `model` 为空，这些逻辑永远无法触达——`Config::load()`
  内部无条件调用 `self.validate()?`，在返回给调用方之前就已经因为 model 为空而
  `Err`。文件不存在时才会走 `Config::default()` 跳过校验，因此该缺陷只在"文件
  存在但不完整"这一具体状态下才会现形。更严重的是，`talos config set` 修复命令
  本身也先调用 `Config::load()`，所以用户一旦落入这个状态，连命令行都无法自救，
  只能手动改文件或删除它。
- Root cause: 把"这段 TOML 能否解析成 Config 结构体"（`load()` 的职责）和"这份
  配置能否被用来跑一次真实会话"（`validate()` 的职责）合并成了一个不可分割的
  调用链，而调用方的分级处理逻辑假设了前者一定成功。
- Fix: `Config::load()` 不再调用 `self.validate()`；三个模式入口既有的
  `needs_model_setup`/`needs_api_key` 检查保持不变、开始生效；`talos config set`
  在应用编辑后仍显式调用 `.validate()` 作为保存前的把关点。
- Prevention: 任何"加载/解析"函数如果被多个调用方在校验失败后要做不同的降级
  处理，就不能在内部预先做硬性校验——校验应该留给真正需要"完整可用"保证的那个
  调用点（这里是保存前）。新增了两个回归测试锁定这一行为边界。
- Promoted to rule/check: `crates/talos-config/src/tests.rs`
  (`test_load_existing_file_with_empty_model_succeeds`,
  `test_load_then_set_model_recovers_from_empty_model_on_disk`).

### 35. 2026-07-03 - HOME 隔离测试需要跨模块共享同一把锁，而不是各建私有锁

- Trigger: 为 I085 Stage 2 `/connect` 新增的 `handle_connect_with_credential_*`
  测试调用了会执行 `Config::save()` 的生产函数；最初没有做任何 HOME 隔离，直接
  把测试固件数据写进了开发机真实的 `~/.talos/config.toml`，覆盖掉了用户的真实
  配置（含一个看起来有效的真实 API key）。
- Symptom: 补上"重定向 HOME 到临时目录 + 本模块私有 `Mutex`"的隔离方案后，单独
  跑该测试文件没问题，但跑整个 `talos-cli` 二进制的完整测试集时，`init_wizard.rs`
  里另一组同样会重定向 `HOME` 的既有测试开始随机失败，报的是 `PoisonError` 或
  "config.model 不该为空"这类看似无关的断言失败。
  该 crate 早已有 lesson #3（"环境变量测试必须用 Mutex 防止并行干扰"），但这次
  的坑不是"忘了加锁"，而是"两处各自都加了锁，却是两把不同的锁"。
- Root cause: `HOME` 是进程级全局状态，`cargo test` 默认在同一进程的多个线程里
  并行跑测试。`init_wizard.rs` 的既有 `ENV_MUTEX` 和新增 `mode_runners.rs` 的
  `HOME_MUTEX` 是两个完全独立的 `static Mutex` 实例——各自内部确实互斥，但两把
  锁之间毫无关联，因此一个模块的测试线程可以在另一个模块的测试线程"以为自己
  独占 HOME"期间把 `HOME` 改到别的临时目录，造成读写目标错位。
- Fix: 新增 `crates/talos-cli/src/test_support.rs`，导出唯一一个
  `pub(crate) static HOME_ENV_MUTEX`；`init_wizard.rs` 和 `mode_runners.rs`
  的所有 HOME 重定向测试改为锁这一把共享实例。
- Prevention: 任何测试如果要修改进程级全局状态（环境变量、当前目录等），必须
  在整个二进制（而不是模块）范围内使用同一把锁；引入新的 HOME/env 变更测试前，
  先搜索 crate 内是否已有类似锁，复用而不是新建。
- Promoted to rule/check: `crates/talos-cli/src/test_support.rs` (doc comment
  states the shared-mutex requirement explicitly for future test authors).

### 27. 2026-06-19 - 工具装饰器必须透传安全与来源元数据

- Trigger: I034 将 MCP 工具接入所有会话模式时审计现有审批包装器。
- Symptom: 包装后的 MCP 工具丢失 `ToolProvenance`，且包装器按工具名推断权限；print
  模式还把未包装的写型 MCP 工具交给只记录 `Ask`、不执行交互批准的 Agent 路径。
- Root cause: `AgentTool` 装饰器只代理了执行和 schema，没有完整代理 `nature()`、
  `provenance()`、`summary_fields()` 等策略元数据；权限层把名称启发式当成类型事实。
- Fix: 两类审批包装器统一按 `evaluate_with_nature(...)` 判定并透传 provenance/summary；
  所有 MCP 工具先经过对应模式的包装器，headless `Ask` 明确拒绝。
- Prevention: 新增或修改任何 `AgentTool` 装饰器时，测试必须覆盖执行、`ToolNature`、
  `ToolProvenance`、summary 字段和 headless `Ask` 行为；不得只测试方法转发。
- Promoted to rule/check: `crates/talos-cli/src/registry.rs` wrapper tests.

### 26. 2026-06-19 - Partial 状态不能成为静默扩展已发布迭代基线的理由

- Trigger: 用户要求继续开发时，准备把 I033 从 Level 0 Skill 接入直接扩展到 Level 1/2
  执行；重新加载治理 Skill 后复查已发布迭代基线。
- Symptom: I033 的所有已发布 Story 和验收均已完成，却因 Level 1/2 产品能力仍未实现而
  标记 Partial，导致后续工作看起来可以继续写入同一迭代。
- Root cause: 需求父目标、迭代 MVP 和后续产品扩展没有分开；状态描述把“产品还有后续”
  错当成“本迭代范围未完成”。
- Fix: 保留 I033 的 Level 0/gate 基线并转 Review；Level 1/2 使用独立需求和迭代；补齐
  AGENTS、START-ITERATION、CHANGE-CONTROL、DOC-CHECK 和迭代模板中的基线规则。
- Prevention: 迭代状态只评价已发布基线。相同子系统但不同可观察结果或验收目标必须使用
  新迭代 ID；开始新工作前盘点并处置所有非终态迭代。
- Promoted to rule/check: `AGENTS.md`; `docs/sop/START-ITERATION.md`;
  `docs/sop/CHANGE-CONTROL.md`; `docs/sop/DOC-CHECK.md`; `docs/iterations/TEMPLATE.md`.

### 21. 2026-06-10 - 不要擅自更改已经与用户确认的设计决策

- Trigger: 用户指出"贴底启动时 logo 块没了"，我擅自把已确认的 `println!` + `print_banner()` 方案改成 `pending_scrollback` 方案，又改回 `println!` + `print_splash_scrollback()`，反复多次。
- Symptom: 用户多次纠正"不要自说自话的就改掉已经做好决策的东西"。
- Root cause: 发现问题时，没有先确认方案方向是否应该变更，而是自作主张换了一个与 TUI-005 需求文档设计相矛盾的实现路径。TUI-005 明确要求 Phase 1 在 raw mode 之前用 crossterm ANSI 输出 splash，`pending_scrollback` + `insert_history` 方案与此冲突。
- Fix: 恢复 `println!` + `print_splash_scrollback()` 方案，贴底看不到 logo 的问题留给 TUI-005 实施（Phase 3 viewport 内 splash status 保证内容可见）。
- Prevention: 遇到已确认方案的问题时，先向用户说明问题和可选方案，等待用户确认后再改。不要自作主张推翻已确认的架构决策。需求文档（如 TUI-005）中的设计约束优先于临时修复。

### 22. 2026-06-11 - 库 crate 必须用 thiserror 而非 anyhow

- Trigger: `cargo clippy --workspace -- -D warnings` 在 CI 中失败。
- Symptom: `talos-evolution` 使用 `anyhow` 作为错误类型，违反了 AGENTS.md 的 "Use `thiserror` for library crates, `anyhow` for binary crates only" 规则。
- Root cause: I021 在 `talos-evolution` 中引入 `anyhow` 时没有遵守已有的错误处理策略。
- Fix: 将 `talos-evolution` 从 `anyhow` 迁移到 `thiserror`：定义 `EvolutionError` 枚举（`Io` + `Store` 变体），创建 `EvolutionResult<T>` 类型别名，移除 `Cargo.toml` 中的 `anyhow` 依赖。
- Prevention: 引入新依赖前检查 AGENTS.md 的 crate 约束。CI `clippy -D warnings` 会阻止违规。

### 23. 2026-06-11 - Rust 2024 let-chains 替代嵌套 if-let

- Trigger: Clippy 和代码审查发现多层嵌套 `if let` / `if condition` 金字塔。
- Symptom: 代码缩进到 5-6 层深，如 `session.rs` 的旧消息格式解析。
- Root cause: 使用 Rust 2021 风格的嵌套 guard，没有利用 edition 2024 的 `let-chains` 特性。
- Fix: 将 `if let X { if let Y { ... } }` 模式统一替换为 `if let X && let Y { ... }`。跨多个 crate 应用：`talos-config`、`talos-agent`、`talos-session`、`talos-sandbox`、`talos-provider`、`talos-rpc`、`talos-cli`。
- Prevention: 使用 edition 2024 时优先使用 let-chains 减少嵌套。CI clippy 会标记不必要的复杂度。

### 24. 2026-06-11 - TUI 流内容应按完整 block 渲染

- Trigger: 我最初实现的 `consume_stream_chunk` 逐行分割流内容并逐行推入 scrollback，导致多行消息行间间距不一致。
- Symptom: 用户消息中如果包含换行，每行单独处理会导致 padding、背景色、行间距不一致。
- Root cause: 流式内容到达时逐 `\n` 分割并逐行 flush，无法对整个消息块做统一的渲染处理（如上下 padding、背景色）。
- Fix: 外部修改重构为 block-based 渲染：`consume_stream_chunk` 只积累 buffer，`finalize_active_stream` 一次性用 `render_stream_block_lines` 渲染整个 block，统一添加 top/bottom padding 和背景色。
- Prevention: 对于需要统一格式化（padding、背景色、分组）的内容，积累后批量渲染优于逐条流式渲染。预览组件仍然实时显示 streaming 内容（最多 6 行），但 scrollback 应在 block 完成后一次性写入。

### 18. 2026-06-05 - 更新 skill 后必须重新跑 governance validator

- Trigger: 用户提示 skill 已更新并要求重新载入、纠偏。
- Symptom: 项目刚提交治理/迭代更新后，重新加载 `agent-project-governance` skill 并运行 validator，发现 `docs/sop/EVOLUTION-FEEDBACK.md` 缺失、AGENTS 未路由 lesson feedback、Git Rules 对 `[model:...]` 的要求不符合新版措辞。
- Root cause: 上一轮治理更新前使用了旧版 skill 认知，没有在最终完成声明前运行新版 governance validator，也没有同步 manifest 中 conformant capability 的必需文件。
- Fix: 新增 `docs/sop/EVOLUTION-FEEDBACK.md`，更新 AGENTS 路由和 Git Rules，刷新 manifest 状态，并登记 backlog compaction 债务。
- Prevention: 用户提示 skill 更新、治理规则更新、或任何 governance artifact 变更后，必须重新读取 skill 并运行 `validate_project_governance.sh`，通过后才能声称治理闭环完成。
- Promoted to rule/check: `docs/sop/EVOLUTION-FEEDBACK.md`; governance validator; AGENTS.md Session End Checklist.

### 1. Cooked mode Ctrl+C 需要 `exit_token` 模式 (I005)

**Symptom**: 交互式模式下双击 Ctrl+C 后进程不退出，需要再按任意键。

**Cause**: `std::process::exit(0)` 暴力退出跳过了 tokio runtime 清理和 Drop trait。而 `return Ok(())` 后 `lines.next_line()` 仍阻塞在 stdin 读取上。

**Remedy**: 使用 `CancellationToken` 作为退出信号，在 `tokio::select!` 中用 `biased` 确保退出信号优先触发，循环退出后正常清理。

**Prevention**: 交互式 CLI 的退出逻辑必须考虑所有异步任务的清理顺序，不能依赖暴力退出。

---

### 2. Raw mode 手动处理 ANSI 转义序列是错误路径 (I003)

**Symptom**: 尝试用 raw mode + 手动 ANSI 转义序列实现交互模式，遇到光标位置、行编辑、历史记录等大量问题。

**Cause**: Raw mode 禁用了终端的行编辑功能，需要手动实现所有交互逻辑（光标移动、删除、历史等），复杂度远超预期。

**Remedy**: 使用 cooked mode（默认模式），让终端处理行编辑，只捕获 Ctrl+C 信号。后续 TUI 模式再用 raw mode + ratatui。

**Prevention**: 不要过早优化。先用最简单的方案（cooked mode），验证核心功能，再考虑高级交互。

---

### 3. 环境变量测试必须用 Mutex 防止并行干扰 (I004)

**Symptom**: `test_env_sanitization_removes_dangerous_vars` 测试偶尔失败，断言 `env::var("LD_PRELOAD").is_ok()` 不成立。

**Cause**: Rust 测试默认并行执行，多个测试同时修改环境变量导致竞态条件。

**Remedy**: 使用 `std::sync::Mutex` 保护环境变量修改，确保测试串行执行。

**Prevention**: 任何修改全局状态（环境变量、文件系统、网络端口）的测试都必须用锁保护。

---

### 4. macOS `/tmp` 是 `/private/tmp` 的符号链接 (I004)

**Symptom**: Seatbelt sandbox 配置中允许写入 `/tmp`，但实际写入失败。

**Cause**: macOS 的 `/tmp` 是指向 `/private/tmp` 的符号链接，Seatbelt 路径匹配不解析符号链接。

**Remedy**: 在生成 sandbox 配置时，使用 `std::fs::canonicalize()` 解析所有路径。

**Prevention**: macOS 路径处理必须考虑符号链接，尤其是 `/tmp`、`/var`、`/etc` 等系统目录。

---

### 5. Mock LLM 是测试基础设施，应尽早实现 (I005)

**Symptom**: I002-I004 的测试依赖真实 API 调用或简单的 mock，无法覆盖复杂场景（长对话、工具调用链、错误恢复）。

**Cause**: 没有统一的 Mock LLM 基础设施，每个测试自己实现简单的 mock，导致测试不完整。

**Remedy**: I005-S1 实现完整的 `MockProvider`，支持预设响应序列、工具调用、错误模拟、流式输出。

**Prevention**: 测试基础设施（mock、fixture、helper）应在项目早期实现，作为后续迭代的 foundation。

---

### 6. 子代理适合独立模块，复杂集成需主代理协调 (I003-I005)

**Symptom**: 子代理实现独立 crate（如 `talos-permission`、`talos-sandbox`）效果好，但集成到 `talos-agent` 时出现接口不匹配、依赖冲突等问题。

**Cause**: 子代理只看到局部上下文，无法预见集成时的约束（如 trait 签名、错误类型、生命周期）。

**Remedy**: 独立模块（新 crate、独立功能）委派给子代理，集成工作（修改现有代码、跨 crate 协调）由主代理完成。

**Prevention**: 委派任务时明确边界：子代理负责 "what"（实现功能），主代理负责 "how"（集成方式）。

---

### 7. `setrlimit` 需要 unsafe，必须有文档说明 (I004)

**Symptom**: `ProcessHardening::apply()` 调用 `libc::setrlimit()` 需要 `unsafe` 块，clippy 警告。

**Cause**: `setrlimit` 是 C 库函数，Rust 无法验证其安全性（可能影响进程资源限制）。

**Remedy**: 使用 `unsafe` 块包裹调用，并添加详细注释说明为什么这是安全的（参数验证、错误处理）。

**Prevention**: 任何 `unsafe` 代码必须有：(1) 注释说明安全性保证，(2) 参数验证，(3) 错误处理。考虑是否可以用 safe wrapper crate（如 `rlimit`）。

**Resolution**: Resolved 2026-06-01 by ADR-007 (`docs/decisions/007-process-hardening-unsafe.md`). The four production `unsafe` sites in `crates/talos-sandbox/src/hardening.rs` now carry `// See docs/decisions/007-…` annotations next to each `// SAFETY:` comment, and the module `# Safety` section links the ADR. Closes the compliance gap recorded by this lesson.

---

### 8. UTF-8 字符边界问题需要字符索引而非字节索引 (I008)

**Symptom**: TUI 输入中文字符时崩溃，错误信息 `end byte index 1 is not a char boundary; it is inside '你' (bytes 0..3)`。

**Cause**: `String::insert()` 和 `String::remove()` 使用字节索引，但 `cursor_pos` 是字符索引。中文字符占 3 字节，字节索引 1 不是字符边界。

**Remedy**: 使用 `char_indices()` 将字符索引转换为字节索引后再操作字符串。输入缓冲区长度使用 `chars().count()` 而非 `len()`。

**Prevention**: 处理可能包含多字节字符的字符串时，始终使用字符索引而非字节索引。

---

### 9. EventStream 需要定期重绘间隔防止界面冻结 (I008)

**Symptom**: TUI 第二次输入后屏幕不再更新，界面冻结。

**Cause**: `tokio::select!` 中 `EventStream::next()` 在无事件时阻塞，导致整个循环无法继续，界面无法重绘。

**Remedy**: 添加 `render_interval.tick()` 作为 select 分支，每 50ms 强制触发一次重绘。

**Prevention**: TUI 事件循环必须有定期重绘机制，不能完全依赖事件驱动。

---

### 10. SQLite 多 crate 共享需要统一 rusqlite 版本 (I008)

**Symptom**: 编译错误 `package 'libsqlite3-sys' links to the native library 'sqlite3', but it conflicts with a previous package`。

**Cause**: `talos-session` 使用 `rusqlite 0.37`，`talos-evolution` 使用 `rusqlite 0.31`，导致两个版本的 `libsqlite3-sys` 链接冲突。

**Remedy**: 统一所有 crate 的 `rusqlite` 版本为 `0.37`。

**Prevention**: 多个 crate 使用同一个原生库时，必须在 workspace 级别统一版本。

---

### 11. "单测全过 + 勾选验收" ≠ 完成；需端到端运行时证据 (I008)

**Symptom**: I008 自进化引擎被标记 COMPLETE（7/7 故事打勾，467 测试通过），但事后审计发现该能力在真实二进制中是 no-op：`TurnObserver`/`BehaviorAdapter` 从未在真实 turn loop 中被调用，TUI `render()` 收到 `evolution_panel` 却从不绘制，`--learned` 因无写入永远为空。

**Cause**: 验收门只有 `cargo test --workspace`。单元测试隔离测试库代码，覆盖不到"库是否被接进运行路径"。`never used` / `never constructed` 警告正是这种脱节的信号，却被忽略。

**Remedy**: 将 I008 状态降级为 REVIEW，登记残留工作 R1-R4，并在 `docs/sop/ITERATION-WORKFLOW.md` 增加强制的"端到端运行时验收门"(§3a)，新建 `docs/sop/DOC-CHECK.md` 防止文档状态漂移。

**Prevention**: 任何改变可观察行为的故事，必须有"通过真实二进制驱动该功能并断言用户可见结果"的证据(测试或手动记录)才能标记 Done。功能核心类型上的 `never used` 警告 = 验收失败。

---

### 12. 自包含 SQLite 需要 ADR 明确例外与运行时边界 (I008)

**Symptom**: 诊断发现 `talos-session` 和 `talos-evolution` 使用 `rusqlite/bundled`。技术上它能把 SQLite 编进二进制、避免依赖系统 SQLite，但 AGENTS.md 的 "No C/C++ bindings" 字面约束没有说明这个例外。

**Cause**: ADR-002 说明了为什么引入 SQLite，但没有说明 `rusqlite/bundled` 与硬约束 #1 的关系，也没有区分 "SQLite 自包含" 和 "完全静态二进制"。

**Remedy**: 新增 ADR-008，明确 `rusqlite/bundled` 是仅限本地存储的例外：SQLite 静态链接进 Talos，运行时不需要系统 SQLite；但最终二进制仍可能链接平台系统库。同步更新 AGENTS.md、README 和架构文档。

**Prevention**: 任何引入原生库、FFI 或 bundled C 源码的依赖，都必须在合并前有 ADR 说明范围、运行时依赖边界、替代方案和回退触发条件。

---

### 13. 自定义存储目录必须隔离派生索引文件 (I008)

**Symptom**: `cargo test --workspace` 中 `session_manager_list_recent_empty_index` 失败。测试创建了临时 `SessionManager::with_dir(...)`，但 `list_recent()` 仍读取真实 `$HOME/.talos/sessions/index.db`。

**Cause**: `SessionManager` 的 JSONL 目录支持注入，但 `get_or_create_index()` 硬编码 `$HOME/.talos/sessions/index.db`，导致测试和自定义运行目录没有完全隔离。

**Remedy**: SQLite session index 改为 `self.sessions_dir.join("index.db")`。默认运行路径不变，自定义目录和测试目录获得独立 index。

**Prevention**: 任何可注入的 storage root 都必须约束所有派生文件（索引、锁文件、缓存、临时文件），不能只约束主数据文件。

---

### 14. 并行委派时 `git stash` 会吞掉兄弟任务的未提交改动 (R0)

**Symptom**: R0 同时启动了 6 个并行委派实现 #ARCH-S1…#ARCH-S7。#ARCH-S4 的代理为 "验证编译错误是否为预先存在" 跑了 `git stash` / `git stash pop` 来回切两次。stash 收走了同时段另一个代理 (#ARCH-S1) 的未提交注释改动，pop 之后那些改动没回来，事后 `git status` 才发现 #ARCH-S1 整个丢失，必须人工重做一遍。

**Cause**: 并行委派下，工作树是多个代理共享的"并发写"区域。`git stash` 会把全部 untracked/modified 改动打包——包括兄弟代理尚未 commit 的工作。`stash pop` 假设 stash 当时的工作树是干净的，但并行场景下不是。

**Remedy**: R0 的 #ARCH-S1 注释工作最终由主代理手工重做 (4 处 `// SAFETY:` + 模块 # Safety 段 + `EVOLUTION.md` Lesson #7 解决标记)。事后在 `docs/iterations/R0-remediation-gate.md` 的 Execution Results 段落记录全过程，并在本表新增 Lesson #14。

**Prevention**:
1. 委派给子代理的 prompt 必须显式禁止 `git stash` / `git reset --hard` / `git checkout --` 等会改动工作树的命令——验证预先存在性应用 `git diff HEAD` / `git show HEAD:<file>`，而不是污染工作树。
2. 多个并行委派之间，每个代理的改动应**先 commit 再开始下一个委托**，让工作树不被多个 WIP 同时占据。
3. 主代理在并行委派结束、收齐结果后，应跑 `git status --short` + `git diff --stat` 校验每个故事的关键文件是否仍在；如果某故事的关键文件 (如 `hardening.rs`) 缺失，要么要求该代理重做，要么主代理自己补。

---

### 15. 并行委派代理会"顺手"实现兄弟任务范围，需用 marker 协议隔离 (I009)

**Symptom**: I009 的 Wave 2 同时启动了 S3 (MCP client)、S4 (MCP server)、S5 (JSON-RPC) 三个并行 deep 任务。S3 代理本应只交付 client 范围，却额外在 `main.rs` 里写好了 MCP server 的 dispatch 路径；S5 代理则在 JSON-RPC server 里预留了 MCP server 的处理钩子。结果三个文件交叠，最终必须用 `// I009-S{n} begin/end` 标记 + 手工剥离 + 多个 Python 脚本（`/tmp/i009-split/` 下的 `build_s3_config.py`、`fix_s3_main_v3.py` 等）来拆分 commit。

**Cause**: 委派 prompt 里只写了"实现 S3 范围"，但代理看到的是整个仓库状态，会从"系统完整可工作"的视角出发去补全关联代码。缺少机器可校验的边界标记时，代理无法判断哪些代码属于"兄弟任务"。

**Remedy**: 主代理最终手动重做拆分：
- 在每个 story 的 start 位置加 `// I009-S{N} begin` 标记；
- 用 `strip_markers.sh` 剥除不属于本 story 的 marker 块；
- 用 Python 脚本逐行分析 `main.rs` 块，按 marker 范围重新分文件；
- 逐个 commit 后跑 `cargo test --workspace` 验证范围。

**Prevention**:
1. **Marker 协议**：每个 story 必须有显式的 `// IXXX-S{N} begin/end` 标记，划定"本故事独占代码块"。Marker 之外区域即使逻辑上相关也不动。
2. **Plan-first pre-stage**：主代理应在并行委派之前，预先在主入口（`main.rs`、`Config`、dispatch 表）写入**空的** marker 块和 stub 实现，让代理只填 marker 内部；不在 marker 内的代码改动 = 越界。
3. **子代理 prompt 显式禁止**："不要修改本 marker 块以外的代码。如需跨 story 协作，把改动写到 TODO 注释，由主代理后续分配。"
4. **主代理验收时硬性规则**：收齐结果后 `git diff --stat` 比对 marker 范围，标记外的改动一律 revert。

---

### 16. `task()` 调用必须二选一：`category` 或 `subagent_type`，不可同时给 (I009)

**Symptom**: I009 plan agent 输出的 prompt 里写了 `task(subagent_type="general", category="ultrabrain", ...)` 这种"同时给两个"的形态。第一次 `task()` 调度就失败、plan 内容丢失，必须重跑。

**Cause**: Sisyphus-Junior 的 `task()` 工具签名是 `category XOR subagent_type`。同时给两个时框架按 `category` 走但参数校验失败，导致整个任务被丢弃没有任何回执。Momus 第一次审稿时也撞上同样的格式错误被拒。

**Remedy**: I009 后半段把"同时给两个"的调用全部改为纯 `category` 形式（`task(category="ultrabrain", ...)` / `task(category="deep", ...)` / `task(category="visual-engineering", ...)`）。Momus 必须以 `task(prompt=".sisyphus/plans/*.md")` 形式调用，路径作为**唯一** prompt。

**Prevention**:
1. 在 `.opencode/agents/Sisyphus-Junior.md` 或全局 prompt 里加一条硬性规则："`task()` 调用必须 EITHER `category` OR `subagent_type`, NEVER BOTH"。
2. Plan agent 在输出 prompt 时应自动 lint 这条规则。
3. 主代理在 plan agent 跑完、Momus 审完之后、肉眼 / 脚本扫一遍所有 `task(` 调用，确认没有同时给两个参数。

---

### 17. visual-engineering 任务在 R0 级别并行委派下 30 分钟不够（结构+消费两个 scope） (I009)

**Symptom**: I009-S1 委派给 `visual-engineering` 任务跑 30 分钟超时。主代理在超时后看到部分工作遗留：TUI 状态机改了一半、`agent_ext.rs` 成了孤儿、consumer 侧 (`/plugins` 命令、provenance marker 渲染) 还没接上。

**Cause**: I009-S1 实际上有两个不同 scope：
- **结构 scope**：`talos-core` 加 `ToolProvenance`、`AgentEvent` 加字段、`AgentTool` trait 加默认方法 — 这是 `ultrabrain` 范畴。
- **消费 scope**：TUI 状态机加 provenance 渲染、`/plugins` 斜杠命令 — 这是 `visual-engineering` 范畴。

`visual-engineering` 30 分钟内同时完成"理解现有 TUI 状态机 + 调整渲染函数 + 写新命令 + 验证不破坏现有交互"，对 1667 行的 `talos-tui/src/lib.rs` 来说太紧。

**Remedy**: 主代理在超时后做了一次"分离式补完"：
1. 撤销消费侧未完成 / 中间状态代码（`set_plugin_status_cb`、`handle_plugins_command`、`agent_ext.rs`）；
2. 保留并完成结构侧（`ToolProvenance` 枚举、`AgentEvent` 字段、MCP adapter override、ADR-009）；
3. 把消费侧 TUI marker + `/plugins` 命令**显式记录到 ADR-009 "Out of Scope"**，作为独立 follow-up story。

**Prevention**:
1. visual-engineering 任务在 ≥1000 行的 TUI 状态机上预计需要 ≥45 分钟。R0 级别（涉及结构变更）应单独预留 60 分钟预算。
2. 当一个 story 同时跨越 core 结构和 UI 消费时，应**先**委派结构给 `ultrabrain`、验证编译后再**再**委派消费给 `visual-engineering`——两阶段串行而非一阶段并行。
3. 必须在 plan agent 输出里就标注 "预计时间 > 30 分钟？→ 不要并行委派给 visual-engineering，串行两阶段"。

## When to Write a Lesson

- A bug was caused by an incorrect assumption about the codebase.
- A test caught something that would have been a production issue.
- A pattern from a reference project didn't translate well to Rust.
- A security concern was discovered during implementation.
- A crate boundary or API design caused unexpected coupling.
- Build, test, or CI behavior surprised you.

---

### 19. 持久化任何"用户输入上下文"前必须 byte-cap，dedup 必须含内容指纹 (I008/I015)

**Symptom**: 用户的 `~/.talos/evolution/knowledge.db` 涨到 241MB；首次 `cargo run -- -p "你好"` 立刻收到 provider `400 Bad Request: Range of input length should be [1, 202752]`。Debug 后发现 system_prompt 是 5,151,386 字节（5MB），user message 是 5,146,164 字节 — 都比 context_limit 大 25 倍。

**Cause**: 三段逻辑 bug 串联成指数膨胀循环，每轮 turn 都让 prompt 变大、pattern 变大、再次注入后下一轮变得更大：
1. **Hook 捕获整条 user message 当 context**。`EvolutionHookHandler` 在 `BeforeProviderCall` 抓 `Message::User.content`，但 agent 早已把 system_prompt 拼进了 user.content（5MB），所以每次"用户说了一句话"实际存进去的是 5MB。
2. **Pattern 提取原样保留 context**。`extract_correction_pattern` 把 `instruction` 设为 `"Remember: " + context`，pattern.instruction 直接变成 5MB。
3. **Dedup 只看 category+description**。每轮 turn 拼接出的 description 都略有不同（system_prompt 每次增长 25MB），40 个 pattern 都是 5MB 级。
4. **BehaviorAdapter 只有数量上限没有字节上限**。`max_patterns=5` × 5MB = 25MB 注入 system_prompt。

**Remedy**: 4 个修复（commit `7470ac5`）：
1. `EvolutionConfig.max_context_bytes` (default 4096) + `TurnObserver::truncate_context` — 写入前 byte-cap，截断带 marker。
2. `EvolutionConfig.max_output_bytes` (default 8192) + `BehaviorAdapter::get_evolution_context` — 输出 byte-cap，超大单条 pattern 丢弃并 warn。
3. `patterns.content_hash` 列 + `DefaultHasher(category + first 1KB instruction)` — dedup 键包含内容指纹，防止近重复累积。
4. `KnowledgeStore::delete_oversized_patterns` 在 `open()` 一次性迁移 — 把现有 30 个 5MB pattern 标记 `active=0`，下次启动输出 `purged oversized patterns on open, count: 30`。

**Prevention**:
1. **任何把"用户输入片段"持久化到本地存储的代码路径**（evolution、log、session 摘要），都必须在写之前 byte-cap，cap 默认值 ≤ 8KB。理由：用户输入理论上可以包含任何上游 prompt 拼进来的内容，size 是对手（用户、provider、设计变更）能制造的维度。
2. **Dedup 键必须包含内容指纹**（hash、embedding、normalized window），不能只看用户可控字段（category、description）。理由：用户可控字段会被内容膨胀污染。
3. **任何"注入到 prompt 的内容"在源头和出口都要有 byte-cap**。源头（cap observation）、中段（cap pattern extraction）、出口（cap adapter output）三层都加，单层失守另两层兜底。
4. **持久化层的迁移不只是 schema**。加新字段/约束时，也要写一次性数据迁移（`open()` / `migrate()` 之后跑），把已存在的脏数据标 `active=0` 而非 DELETE，保留审计轨迹。SQLite 不会自动 VACUUM，241MB 文件大小不会变，但新增数据不会再膨胀。
5. **涉及 system prompt 注入的运行时回路要有运行时证据**。这次发现 5MB 的过程是用户实际跑 `cargo run -- -p "你好"` 才暴露的 — 单测全过 + 单元/集成测试 = 看似完成，但 system_prompt 大小是端到端运行才能看出的属性。Lesson #11 早就提过"端到端运行时证据"，这次仍走了 4 轮 commit 才暴露问题。

> **Note (2026-06-06)**: 本 lesson 的 prevention 规则 1 已被 lesson #20 部分修正 — byte-cap 是 defense layer,不是 root-cause fix。真治根已在 I021 落地 (commit: see `git log --oneline -- crates/talos-evolution/src/`)。byte-cap 仍是 defense-in-depth,但已不再是唯一防线。这条 lesson 保留作 defense-in-depth 的依据,但**不要把 byte-cap 当成完整修复**。

---

### 20. `Signal.context` 字段语义 = signal 周围短窗口,不是整条 user message (I008/I021)

**Symptom**: 实施 7470ac5 之后,knowledge.db 从 241MB 降到 13MB,system_prompt 从 5MB 降到 13KB,模型能正常响应。但是 — 翻看 active patterns 列表(共 9 个),其中 1 个 `preference` 类 pattern 的 instruction 内容是 `Remember: # Identity\nYou are Talos, an AI coding assistant. You help users with programming tasks by using tools to read, write, and execute code.\n# Tools\nNo tools available.\n# Skills...` (整整 4KB,全是 system_prompt 头)。这条 pattern 没有任何用户偏好信号,纯噪声。

**Cause**: 7470ac5 的 `truncate_context` 实现是 `context[..truncate_at]` — 保留**前 4KB**。但根据 MenteDB 原始设计(`docs/reference/REFERENCE-PROJECTS.md` §17),`Signal.context` 字段的语义是"the user showed the correct behavior" 的那一句 + 短窗口,通常 < 500 字节,不是整条 5MB user message。`Signal.context` 的语义在实现时偏离了 MenteDB 蓝图:
- 期望: signal 周围短窗口(marker 居中,前后 200 字节)
- 实际: 整条 5MB user.content
- 7470ac5 折中: 整条 5MB 截到前 4KB → 仍是 system_prompt 头,无用户信号

7470ac5 是 **defense layer**(防 storage 暴涨、注入溢出),**不是 root-cause fix**(治不了字段语义错用)。Lesson #19 的 prevention 规则 1 ("byte-cap 默认 ≤ 8KB") 在此失效 — 因为正确的字段值本就 < 500 字节,任何 cap 都只是兜底。

**Remedy**: 新 iteration `I021-evolution-mentedb-realignment.md`,5 个 story:
- #I021-S1: `Observation` → `TurnObservation` (parent) + `Signal` (child),Signal 用 MenteDB 字段
- #I021-S2: Hook 捕获改用 `find_marker + capture_window(text, marker_pos, 200)`,marker 居中
- #I021-S3: `Pattern` 加 `key`/`value`/`contradicting_count`/`source_sessions`
- #I021-S4: `knowledge.db` 一次性硬重置(schema 不兼容,无法软迁移)
- #I021-S5: 保留 7470ac5 的 byte-cap 作为 defense-in-depth,文档说明真治根在 I021

**Remedy** (已落地 2026-06-06):
- #I021-S1..S5: Signal/TurnObservation/Pattern schema alignment with MenteDB (data structure root-cause fix, landed 2026-06-06)

**Prevention**:
1. **字段语义对齐参考设计的优先级 > 防御层**。7470ac5 防住了 storage 暴涨,但掩盖了"字段语义错"这个更深的 bug — 后续 review 容易误以为已经修好。规则:**先修字段语义,再加 cap**。如果字段本来就只存 < 500 字节,cap 是不必要的复杂度。
2. **Defense-in-depth 修复必须在文档里标注 "this is not the root cause"**。7470ac5 的 commit message 和 lesson #19 都没清楚区分 "防 storage 暴涨" 和 "治字段语义",导致这次 evidence-driven 复查才发现问题。规则:**defense layer 修复的 commit message 和对应 lesson 必须包含 "real fix is in <future iteration>" 的明确指向**。
3. **数据结构的"实现 vs 设计"差距应该定期审查**。MenteDB reference 文档(`docs/reference/REFERENCE-PROJECTS.md` §17) 写了 `Signal.context: String` 是短窗口,但实现时把它当成 5MB 容器用 — 这种 reference-vs-impl drift 应该在每次 evolution-engine 相关 PR 时 check 一次。规则:**任何修改 `talos-evolution` 的 PR 必须在 PR 描述里对照 `docs/reference/REFERENCE-PROJECTS.md` §17 列出字段语义是否保持**。
4. **`EVOLUTION.md` lesson 应该能被指向,而不是只被阅读**。Lesson #19 现在被 I021 README "Required Reads" 引用,以后任何相关 PR 都能找到 — 这条 lesson 之前的预防规则 1 已经被 #20 修正("byte-cap 不是治根"),不要盲信旧规则。

---

### 25. 2026-06-15 - 外部 C/Native 依赖的 panic 必须捕获 + 降级，不能静默崩溃

- **Trigger**: `arborium::Highlighter::highlight_spans()` 调用时进程静默崩溃（无错误信息，直接退出），原因是内部 tree-sitter C 运行时 panic。
- **Symptom**: TUI 在渲染 `receiving code block...` 预览时立刻退出，终端回退到 shell，无任何 panic 信息或错误输出。用户反馈"直接闪退了"。
- **Root cause**: `highlight_spans()` 调用的是 C 运行时，`?`/`.ok()` 只能捕获 `Result::Err`，无法捕获 `panic!` 或 C 级别的 abort。Arborium 的 C 语法解析器在某类输入上直接终止了进程。
- **Fix**: 在 `HighlightEngine::highlight()` 中对外部依赖调用包裹 `std::panic::catch_unwind(AssertUnwindSafe(|| { ... }))`，将 panic 转为 `None` 返回，让上层降级为纯文本渲染。
- **Prevention**:
  1. **任何调用外部 C/Native 依赖的边界必须包裹 `catch_unwind`**。包括但不限于：tree-sitter、SQLite、libc、子进程启动。
  2. **降级路径必须是同功能的无依赖纯 Rust 实现**。语法高亮失败 → 纯色字符渲染；SQLite 崩溃 → JSONL 文件直接读取。
  3. **此约束已写入 AGENTS.md Hard Constraint #9**。

---

### 26. 2026-06-22 - Agent 提交必须带 `[model:<id>]` 后缀；缺失是 governance drift，需要特批才能修

- **Trigger**: 提交 `bf4dca4` (依赖升级) 和 `a1943c5` (I040 关闭 + I041 启动) 时漏掉了 `[model:MiniMax-M3]` 后缀，被用户提醒后用 force-push 修复。
- **Symptom**: 两次 commit 的 subject line 末尾没有 `[model:<id>]`，与 AGENTS.md Git Rule 2 冲突。EVOLUTION.md lesson index 和 commit 历史都无法追溯到具体模型。
- **Root cause**: 我（agent）在 commit 时只关注了 conventional commit type 和 scope，没有把 `[model:...]` 当作强制字段对待；提交后没有自检 commit message 格式。
- **Fix**:
  1. 修复：`GIT_EDITOR=/tmp/talos_add_model_tag.sh GIT_SEQUENCE_EDITOR="sed -i '' 's/^pick/reword/'" git rebase -i HEAD~2`，脚本在每条 commit message 第一行末尾追加 `[model:MiniMax-M3]`。
  2. `git push --force-with-lease` 完成 force-push。
- **Prevention**:
  1. **Agent 提交的 commit message 必须包含 `[model:<id>]` 后缀**。提交前自检：subject 末尾是否带模型标记；不带则补上再 commit。
  2. **把 `[model:...]` 当作 conventional commit 的一部分写进 commit 命令模板**。例如：`git commit -m "feat(scope): description [model:MiniMax-M3]"`。
  3. **Force-push 修复历史是 governance exception**，需要用户明确特批（AGENTS.md Git Rule 5: "Never force-push to main"）。默认走 forward-only 路径：缺失的 tag 在后续 commit 中补上，不回头修。
  4. **本条 lesson index 引用 #22**。未来任何 agent commit 自检脚本都应包含 model tag 检查。

---

### 28. 2026-06-23 - 并行审计是 closeout 前的必要闸门；`a.id.cmp(&a.id)` 类自我比较 bug 能穿透单元测试

- **Trigger**: I043/I044 关闭前启动了两个并行 explore agent 做最终逻辑审计。审计发现 4 个真实问题 + 2 个 cosmetic gap,其中 1 个是 sort tiebreaker `a.id.cmp(&a.id)` (与自身比较,永远是 Equal,no-op)。
- **Symptom**: `/delete` picker 列表对相同 timestamp 的 session 排序不确定,会随 HashMap 迭代顺序变化。单元测试因为只有 1-2 个 session 没触发。
- **Root cause**:
  1. 写 sort tiebreaker 时手误,把 `b` 写成了 `a`。Rust 编译器不会报错 (`a.id` 在闭包里可见),clippy 也没抓到。
  2. 单元测试用的数据集太小 (1-2 个 session),即使 tiebreaker 是 no-op,排序结果仍然"看起来正确"。
- **Fix**:
  1. `a.id.cmp(&a.id)` → `a.id.cmp(&b.id)`。
  2. 同时审计发现的其他 3 个问题 (silent bridge send failure, fork file copy race, /delete arg_hint) 也一并修复。
- **Prevention**:
   1. **关闭任何迭代前,启动至少 2 个并行 explore/oracle agent 做最终逻辑审计**。审计 prompt 必须明确列出 acceptance criteria 让 agent 逐条核对。
   2. **Sort tiebreaker 写完后,自检两个比较变量是否不同**。`a.id.cmp(&a.id)` 是典型的复制粘贴或手误 bug,编译器和 clippy 都不会抓。
   3. **单元测试的排序测试至少覆盖 3+ 个元素 + 制造 tie 场景**。1-2 个元素的排序测试无法发现 tiebreaker bug。
   4. **并行审计发现的每个 issue 都要 trace 回 acceptance criteria**:这个 bug 是否影响 acceptance?如果影响,必须在 closeout 前修;如果不影响,记录为 residual。

---

## #29: `#[serde(skip_serializing)]` on `api_key` causes silent data loss

- **Area**: Config
- **Added**: I045 (2026-06-24)
- **Symptom**: User manually added `api_key` to `~/.talos/config.toml`, ran talos, and the key was silently erased on the next `Config::save()`.
- **Root cause**: `#[serde(skip_serializing)]` on `ProviderConfig::api_key` means serde never serializes the field back to TOML. The key is loaded correctly from the file (deserialization works fine), but on save, the key disappears because it was never written back. The design intention was to redirect keys to a separate `credentials.toml`, but the redirect was implemented as "shred and forget" rather than "extract on load, write to separate file."
- **Fix**: Reverted `#[serde(skip_serializing)]`. `api_key` is now serialized in `config.toml` like any other field. Display masking is handled at the CLI layer (`--config-list` replaces `api_key = "..."` with `api_key = ***`), not the serializer.
- **Prevention**:
  1. **Never use `#[serde(skip_serializing)]` on data that the user wrote to a file.** The field round-trips well in memory, but `save()` → `load()` silently loses the original value.
  2. **Secrets masking must be at the display boundary, not the persistence boundary.** The serializer should faithfully write what's in memory. The CLI (display) is responsible for masking.
  3. **test that save → read-back preserves the field.** Our regression test `test_save_writes_api_key_in_config_toml` now verifies that the key is present in the file after save.

## #30: Model switch needs `ensure_persisted()` — session may not exist on first turn

- **Area**: Session / Model Lifecycle
- **Added**: I045 (2026-06-24)
- **Symptom**: Running `/model` on a fresh TUI (no session yet) panicked with "session file not found."
- **Root cause**: `handle_session_model` calls `session_watch_rx.borrow().clone()` to get the current session, then tries to read its messages. But on the very first turn before any user input, the session hasn't been persisted to disk yet — `ensure_persisted()` was never called. The session exists in-memory (created by the engine startup) but has no backing file.
- **Fix**: Added `current_session.ensure_persisted()` at the start of the model switch handler, before any message reads. This creates the backing file if it doesn't exist, matching the same pattern used by `/new`, `/resume`, and `/fork`.
- **Prevention**:
  1. **Any code that reads session messages from disk must call `ensure_persisted()` first.** The session may only exist in-memory at engine startup.
  2. **File-backed state requires explicit creation.** In-memory-only sessions are valid until the first persistence operation.
  3. **When adding lifecycle operations, trace the complete execution path from fresh startup.** The first-turn edge case is easy to miss if you only test on existing sessions.

## #31: `ring` + `cargo-xwin` cross-compilation fails for Windows ARM64

- **Area**: Build / Release
- **Added**: I046 (2026-06-25)
- **Trigger**: v0.1.1 release build for `aarch64-pc-windows-msvc`
- **Symptom**: `ring` crate assembly compilation fails: `clang: error: no such file or directory: '/imsvc'`
- **Root cause**: `cargo-xwin` injects MSVC-style `/imsvc` include flags via `CFLAGS_<target>`. `ring`'s build script resolves `CC_<target>=clang-cl` through cc-rs but still invokes `clang` (GCC driver) for the actual assembly/C compilation — `clang` (GCC mode) doesn't understand `/imsvc` (MSVC mode only). Setting bare `CC=clang-cl` also doesn't help because `ring` doesn't propagate it to its internal compiler invocation. `aws-lc-rs` is not a viable alternative — it uses CMake + NASM and has even worse cargo-xwin compatibility (static CRT `dllimport` linkage conflicts).
- **Fix**: Skipped `aarch64-pc-windows-msvc` in `build.sh`. Switched Linux targets from `gnu` to `musl` for fully static binaries (tested both arches locally, zero issues). Windows ARM64 users can use the x86_64 build via emulation.
- **Prevention**:
  1. **Don't attempt `aarch64-pc-windows-msvc` cross-builds until `ring` is replaced or the TLS backend switches to `native-tls`.** Documented in `build.sh`.
  2. **For static Linux binaries, prefer `musl` targets via `cargo-zigbuild`.** They compile cleanly and avoid glibc version dependencies.
  3. **Future: migrate `reqwest` to `native-tls`** (uses OS TLS — Schannel on Windows, no C compilation). Requires replacing or feature-gating `rust-websearch` which pulls `reqwest 0.12` with its own `rustls`+`ring`.

## #32: Stale validation evidence after shared-dataset changes

- **Area**: Governance / Validation
- **Added**: I046 (2026-06-25)
- **Trigger**: I046 pre-planning discovered `cargo test --workspace` was failing at I045 closeout
- **Symptom**: I045 iteration doc claimed "✅ `cargo test --workspace` passes" but two tests were actually broken: `test_model_limits_from_builtin_and_custom_providers` (stale `gpt-4.1` after catalog commit `0734eae`) and `test_session_picker_accept_resume_default_command` (lost `/resume` fallback in I045 `PanelItemAction` refactor `a8cd614`).
- **Root cause**: The catalog expansion commit (`0734eae`) and the `PanelItemAction` refactor (`a8cd614`) landed AFTER I045's closeout verification. The closeout check passed at an intermediate commit but the subsequent commits broke tests without re-verification.
- **Fix**: I046-S1 fixed both tests. I045 evidence corrected with a post-closeout correction note appended (not rewritten) per baseline-preservation rules.
- **Prevention**:
  1. **Re-run `cargo test --workspace` after ANY commit that touches shared datasets** (`models.toml`, protocol types, public API signatures) — not just after the feature commit.
  2. **Closeout verification must reflect the final committed state.** If commits land after the verification run, re-verify before marking Complete.
  3. **Prefer `cargo test --workspace` over `cargo test -p <crate>` for closeout.** The targeted test hid the cross-crate breakage.

## #33: Agent 收到新需求时不应立即中断当前任务，通过 todo 工具捕获并规划执行节奏

- **Area**: Agent / Prompt Engineering
- **Added**: I045 (2026-07-01)
- **Trigger**: 用户报告模型切换导致 todo 列表丢失，Agent 立即跳转到代码库调查 EVOLUTION.md、I045 文档等，用户指出发生了多次话题跳跃。
- **Symptom**: 用户在同一个会话中提出多个需求时，Agent 每次立即中断当前工作去追逐新话题，导致上下文漂移，没有任务被完成闭环。
- **Root cause**: Agent 系统提示词（`identity.txt`）中没有"收到新需求时先捕获再规划"的行为规则。Agent 默认行为是立即响应用户最新消息，不管手头是否还有其他未完成的事情。
- **Fix**: 在 `crates/talos-agent/prompts/identity.txt` 中新增 "Task Interruption and Planning" 节：当用户提出新需求时，Agent 应使用 `todo_create` 捕获为待办项，在当前任务到达检查点后再评估优先级和规划执行顺序，与用户确认后再切换。
- **Prevention**:
  1. **系统提示词是 Agent 行为的第一防线。** 任何关于执行节奏、任务规划的期望行为必须在 identity 模板中明确化。
  2. **Agent 的默认"立即响应"倾向与多任务场景冲突。** 需要显式规则覆盖。
  3. **todo 工具不仅是记录工具，也是中断缓冲区。** 新需求先进入 todo 队列，避免直接丢掉当前上下文。

## #34: 静态目录数据必须对照上游实况核验；"格式看起来合理"的条目 ID 不是证据

- **Area**: Config / Data Integrity
- **Added**: 2026-07-03
- **Trigger**: 对 `models.toml` 做全量更新时，将每个条目与 models.dev 实时 api.json 逐一比对。
- **Symptom**: 目录中存在三个从未存在过的 OpenAI 模型 ID（`gpt-4.1-2025-04-14`、`o3-2025-04-16`、`o4-mini-2025-04-16`）——形态完全符合 OpenAI 的日期后缀惯例，且有配套测试"验证"它们，掩盖了数据是编造的事实。另有 MiniMax-M3 context、Google/Zhipu 发布日期、OpenRouter 条目 ID 记法等多处与实况不符。
- **Root cause**: 早前会话按参数化知识中的"惯例格式"生成条目 ID，未对照上游数据源核验；后续 I046 修测试时又以该编造目录为基准修正测试（lesson #32 的姊妹问题：测试与数据互相"证明"，双双错误）。
- **Fix**: 2026-07-03 全量刷新（commit `071449b`）：直接拉取 models.dev api.json，逐条目核验后重写；错误 ID 替换为真实规范 ID，测试同步修正。
- **Prevention**:
  1. **目录/数据集类文件的任何条目必须能溯源到上游实况**（实时 API 响应或官方文档），"符合命名惯例"不构成存在性证据。
  2. **测试断言目录数据时，测试本身不是真相来源。** 测试与数据同源于编造时会互相掩护。
  3. **机制化替代人工核验**：I085/MC101 的 build.rs 门控刷新落地后，此类漂移应由管道消除。已在 I085 执行记录中登记。

## 2026-07-16 - Secret absence tests must isolate nondeterministic fields

- Trigger: I134 targeted crate validation intermittently failed `record_excludes_credentials`.
- Symptom: The test searched the complete serialized record for the short fixture value `abc`; a
  random UUID record ID can independently contain that substring and produce a false failure.
- Root cause: A security assertion used an unscoped low-entropy substring across deterministic and
  nondeterministic fields.
- Fix: Assert credential absence only in the URL/origin fields that can carry the credential while
  retaining whole-record checks for semantic secret-key markers.
- Prevention: Secret-leak fixtures must use distinctive values and scope assertions to fields or
  projections that could contain the source secret; random IDs must not participate in equality or
  substring absence claims.
- Promoted to rule/check: `browser_page::tests::record_excludes_credentials` regression test.
