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

## Lessons

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

## When to Write a Lesson

- A bug was caused by an incorrect assumption about the codebase.
- A test caught something that would have been a production issue.
- A pattern from a reference project didn't translate well to Rust.
- A security concern was discovered during implementation.
- A crate boundary or API design caused unexpected coupling.
- Build, test, or CI behavior surprised you.
