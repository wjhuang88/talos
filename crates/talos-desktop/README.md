# Talos Desktop development host

I284 is under local development. These commands describe the implementation in
this branch, not a released Desktop product. Native acceptance is still pending.

## Launch

Use the repository-pinned Rust toolchain from the repository root:

```sh
cargo run -p talos-desktop --features desktop-ui --locked --bin talos-desktop-mock -- --live zh-CN
```

Use `en-US` instead for English. The binary retains its existing name; `--live`
explicitly selects the configured-provider path. Without `--live`, it opens the
existing fixture prototype and does not run real tasks:

```sh
cargo run -p talos-desktop --features desktop-ui --locked --bin talos-desktop-mock -- zh-CN
```

Live mode loads the normal Talos configuration and credentials on its host thread.
Configure the provider/model using the existing Talos CLI first. Configuration is
loaded when the host is first started; restart Desktop after changing it. Model
variant resolution is shared with the CLI. No setup failure substitutes fixture
output. Sending a live task calls the configured provider and may incur its normal
usage charges.

Enter an explicit workspace and task, then choose **Send**. The folder button
uses the native directory picker. **Current task** and **Settings** navigate
without starting another turn. Language switching preserves the task and output.
Small windows scroll the task page to reach Send, Cancel and output.

## Current behavior and limits

- One in-memory conversation, one active turn, one workspace per host. Restart
  live mode to change workspace. Output currently displays the latest turn only.
- Text, terminal errors and cancellation are projected from Runtime events.
  The displayed turn ID comes from Runtime, not a fixture identifier.
- Live mode registers shared Runtime tools. Tool calls and results display their
  actual call IDs; execution remains subject to Runtime permission and sandbox checks.
- Auto follows the existing `auto.enabled` configuration. Its reported decision,
  reason and evaluator are displayed; a report does not imply model consultation.
- A pending approval offers Allow once, Allow session and Deny for the displayed
  request. Replies are bound to its request ID; stale or repeated replies cannot
  authorize a later request. Cancelling or closing the approval surface fails closed.
- Configured live mode binds the conversation to a workspace-scoped durable Session under
  `.talos/desktop-sessions`; successful turns are persisted through the shared Runtime contract
  and can be read after a host restart. Pending, failed or cancelled turns are not replayed.
  Tool activity also exposes read-only provenance evidence (`native`, MCP server, or plugin) with
  the exact call ID; missing provenance is rendered as unavailable rather than inferred. The
  Recent tasks page lists existing identities for the selected workspace and offers an explicit
  Resume action. The Work panel reads the shared graph for the exact session when available and
  distinguishes unavailable data from storage errors. The evidence panel lists bounded file
  differences observed around successful built-in `write`, `edit` and `delete` calls, with session,
  turn and tool-call identity. It excludes shell/custom tools, symlinks, paths outside the workspace
  and files larger than 1 MiB; it retains no file contents, does not show a content diff, and prior
  evidence is unavailable after restart. Evaluation and Delivery remain unavailable without a
  supported shared persistent evaluation source. Resumed transcript entries are marked as restored
  history and never re-run tools. Fixture presets do not configure live execution.
- Cancel requests a Runtime interrupt. Tests cover a pending provider connection,
  an open paused stream, and provider-backed history compaction; the latter cancels
  the committed turn without waiting for the compactor provider.
- ADR-082 defines the Unix shell cancellation boundary: ordinary descendants in
  the owned process group are in scope; descendants deliberately leaving that
  group are not contained by this mechanism. Cancel and shutdown wait for the
  managed sandbox's cleanup receipt; unconfirmed cleanup is reported as an error.
  Native integration tests exercise a running shell and descendant; protected
  review and the human acceptance rows remain separate gates.
- Closing the window requests Runtime shutdown and waits asynchronously for its
  result. After the GUI loop exits, the application waits at most 35 seconds
  overall for host completion receipts and reports unconfirmed cleanup as failure.
- Presentation buffering has count/byte limits and reports incomplete output on
  overflow. These are not a hard end-to-end memory bound on Runtime's upstream
  event channel or the rendered transcript.

## Local verification

```sh
cargo test -p talos-desktop --features desktop-ui --locked
cargo check -p talos-cli --locked
cargo build -p talos-desktop --features visual-test --locked
target/debug/talos-desktop-mock --capture-live /tmp/talos-desktop-live-capture-new
```

The screenshot directory must not already exist. Capture uses fixed display data,
hidden native windows, both locales and wide/narrow sizes; it makes no provider
request. A capture validates rendering, not real-provider or human acceptance.
On Linux a usable X11/Wayland display is required.

## 中文操作说明

上面的 `--live zh-CN` 命令打开真实模型模式；不带 `--live` 则仍为示例原型。
先用现有 CLI 配好模型，再填写工作区和任务，点击“发送”。“取消”请求停止当前
对话；关闭窗口会先请求清理 Runtime。窄窗口请向下滚动查看操作按钮与输出。

真实模式使用共享 Runtime 工具和现有权限、沙箱门禁。Auto 遵循 `auto.enabled`
配置并显示实际决策；需要人工批准时，可选择单次允许、会话允许或拒绝。
选择只作用于当前申请，取消或关闭不会批准后续请求。
Unix shell 取消的保证范围是所属进程组内的普通后代；主动脱离进程组的程序
不属于该机制的强隔离保证。取消和退出会等待托管沙箱的清理回执，无法确认
清理时会报告错误。真实 shell 与后代的集成测试不替代安全复核或人工验收。
会话 transcript 通过共享 Runtime 持久化，Recent tasks 可显式恢复已有会话，恢复不会重放工具。
工作面板只读取当前会话的共享工作图；证据面板仅列出成功的内置文件工具调用前后观察到的
受限文件差异，并显示会话、turn 和工具调用身份，不保留文件内容或提供内容 diff。它不覆盖
shell/自定义工具、符号链接、工作区外路径或大于 1 MiB 的文件；重启前的文件证据不可用。
没有共享持久化评估数据时，Evaluation 与 Delivery 保持不可用，不会推断为通过。
工具和权限交互的原生人工验收以及后续阶段仍未闭环，
不能把截图或单元测试通过视为完整桌面交付。
