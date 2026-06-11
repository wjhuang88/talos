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

## Lessons

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
