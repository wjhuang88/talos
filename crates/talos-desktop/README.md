# Talos Desktop development host

I282-I285 and their H1/H6 follow-ups form a validated local development candidate.
These commands describe the development host, not a released Desktop product.

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
The task page scrolls when its contents exceed the available window area. Output grows with
its contents and shares that page scroll; there is no nested vertical output scroller.
Choose the original workspace before opening **Recent tasks**, then select **Resume**.

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
  The local approval panel displays actual JSON arguments alongside the scope and review
  explanation. Oversized requests are denied rather than shown incompletely for approval.
  Session grants are scoped and do not survive restarting the host.
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
  evidence is unavailable after restart. Evaluation is available only when the
  Runtime can assemble authoritative session/revision-bound criteria and evidence;
  otherwise Evaluation and Delivery fail closed. A workspace fingerprint alone does not
  validate task completion. I287's ephemeral evidence contract does not require a new persistent
  evaluation store, and cannot restore an old PASS from conversation history after restart.
  Explicit Evaluate requests can supply a separate Runtime-produced observation of authorized,
  successful built-in file operations: up to 32 artifacts, 64 KiB per UTF-8 file and 256 KiB total.
  These current file contents are sent to the evaluator, unlike the metadata-only evidence panel.
  Content is complete and preserves whitespace/newlines; oversized files are rejected, not truncated.
  Unregistered workspace files are not sent. Observations can support content-based Behavior
  criteria, but cannot prove tests passed or unobserved execution succeeded. Missing, unsupported
  or stale evidence fails closed; a Goal verdict alone does not authorize Mission Delivery.
  Evaluate assesses the current Goal input; Send starts a task turn instead. For a content-only
  check, enter the expected file state and click Evaluate without Send. Evaluation displays
  criterion verdicts in claim order and bounded model-authored findings, or explicitly states
  that no detailed findings were supplied. These explanations are not independent evidence.
  Malformed or incomplete model reports remain failures; they are never repaired into PASS.
  Evaluation has a 30-second total deadline covering model dispatch and response generation;
  timeout invalidates the current evaluation and keeps Delivery blocked. Cancel remains available.
  Resumed transcript entries are marked as restored
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
对话；关闭窗口会先请求清理 Runtime。内容超过窗口时整页滚动，输出区随内容增高，
不再嵌套纵向滚动。恢复历史前先选择原工作区，再进入“最近任务”点击“恢复”。
审批面板显示实际 JSON 参数；会话授权不跨执行服务重启保留。

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
没有权威 Runtime 评估来源时，Evaluation 与 Delivery 保持不可用；工作区指纹不能证明任务完成。
I287 的临时证据合同不要求新增持久化评估存储，重启后也不能从对话历史恢复旧的 PASS。
显式评估会把当前会话中已授权、成功执行的内置文件工具产物交给评估模型：最多 32 个产物，
每个 UTF-8 文件最多 64 KiB，合计最多 256 KiB。此路径会发送受限的实际文件内容，
与只显示元数据的证据面板不同；未登记的工作区文件不会发送。内容观察只能支持相应的行为标准，
不能证明测试通过或未观察到的执行成功。证据缺失、不支持或过期时保持不可用；
单个 Goal 的评估结论不能代替 Mission 的交付门禁。
文件内容完整保留空格和换行；超限文件会被拒绝，不会截断后交给评估模型。
Evaluate 评估当前 Goal 输入框中的目标；Send 则启动任务执行。只验收文件内容时，
填写预期文件状态后直接点击 Evaluate，不点击 Send。结果按验收标准原顺序显示，
并展示有长度限制的模型说明；模型未给出详细说明时会明确提示。这些说明本身不是证据。
格式错误或未完整返回的报告保持失败，不会自动补成 PASS。
评估的总时限为 30 秒，包含模型请求派发与响应生成；超时使当前评估失效，交付保持阻止，期间仍可取消。
本轮 H1-H6 原生人工验收与相应代码门禁已记录在四周任务中；此前延期的
VoiceOver、Windows/Linux 原生交互等 I277 项仍未验收。这个本地候选也尚未实现
后续 Mission-first 完整产品流程，不能把截图或单元测试通过视为完整桌面产品交付。
