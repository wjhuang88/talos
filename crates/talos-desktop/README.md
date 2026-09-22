# Talos Desktop development host

I282 is under local development. These commands describe the implementation in
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
- No executable tools are registered in I282. A tool request is shown as
  unavailable/not executed. Auto and permission UI belong to I283.
- Conversations are not persisted in this stage; restart/resume belongs to I284.
  Fixture presets do not configure live execution.
- Cancel requests a Runtime interrupt. Tests cover a pending provider connection,
  an open paused stream, and provider-backed history compaction; the latter cancels
  the committed turn without waiting for the compactor provider.
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

本阶段不执行工具、不接入 Auto、不保存会话；示例预设不会影响真实请求。
重启会丢失当前内存会话。历史压缩阶段的取消、原生人工验收和后续阶段仍未闭环，
不能把截图或单元测试通过视为完整桌面交付。
