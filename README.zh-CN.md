# Talos

[![Release](https://github.com/wjhuang88/talos/actions/workflows/release.yml/badge.svg)](https://github.com/wjhuang88/talos/actions/workflows/release.yml)
[![Latest Release](https://img.shields.io/github/v/release/wjhuang88/talos?include_prereleases)](https://github.com/wjhuang88/talos/releases)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.95%2B-orange)](https://www.rust-lang.org/)

[English](README.md)

Talos 是一个 Rust 原生的本地编码 Agent，面向希望在自己机器上运行、审查和扩展 Agent
运行时的开发者。它提供终端 UI、模型提供商适配、会话历史、内置编码工具、显式权限控制、
运行时 Skills、MCP/RPC 集成和项目治理支持，同时保持默认本地、边界清晰、可审计。

Talos 已发布第一条稳定的 pre-1.0 release 线。当前工作区版本是 `v0.2.1`。它已经可以用于
本地编码工作流，但仍然处于 1.0 之前：API、命令界面和存储格式仍可能随着产品化加固继续演进。
本 README 只描述已经发布或当前已实现的用户可见能力；只读 loopback dashboard 之外的 Web
控制面扩展、dotagents shared Skills、WASM 插件和高级文档解析等研究方向请查看[项目状态](#项目状态)。

## 主要能力

- **本地优先的编码 Agent**：支持交互式 TUI、inline 模式和脚本友好的 print 模式。
- **默认安全的工具运行时**：文件写入、删除、Git 写操作、Shell 执行、网络动作和 MCP 工具都经过显式权限边界。
- **Rust 原生核心**：基于 workspace 的小 crate 架构，默认不依赖 Node/Python 运行时。
- **可嵌入 Rust 运行时**：初始 `talos-runtime` facade 允许其他 Rust 项目在不依赖 Talos CLI/TUI crate 的情况下构造安全的进程内 Agent 运行时。
- **可审计内部结构**：记忆、配置、CLI/TUI 和 agent compaction 等超大模块已拆成聚焦的 Rust 模块，并通过行为保持验证。
- **内置编码工具**：覆盖文件、搜索、编辑、Shell、符号索引、目录树、diff/stat、Git、HTTP 请求和 Web 搜索。
- **持久会话与记忆**：SQLite 会话历史、搜索、分支/分叉、导出、语义记忆固化和保留策略预览。
- **渐进式上下文**：运行时 Skill 发现和显式 Skill 主体/引用激活，不把隐藏内容直接倒入可见历史。
- **可扩展表面**：MCP 工具、Hooks、JSON-RPC 和工程治理状态已实现；插件/WASM 与浏览器控制面仍是研究方向。

## 当前 Release 边界

`v0.2.1` 适合本地开发者在自己机器上使用，并由操作者审查工具动作和配置。它还不是远程多用户服务、
插件市场、浏览器自动化控制面或自主后台守护进程。

当前已发布/已实现：

- TUI、inline 和 print 运行模式。
- TUI 模式下默认启动只读 loopback dashboard，并在启动时提示本地 URL 和 bearer token。
- 本地模型配置，密钥显示自动脱敏。
- 带权限控制的内置编码工具。
- 会话存储、搜索、清理、维护、记忆固化和探索库导入。
- 从 `.talos/skills/`、`~/.talos/skills/` 和父级 `.talos/skills/` 发现运行时 Skills。
- MCP stdio 工具和 JSON-RPC 基础设施。
- `talos-runtime` crate 中的初始 Rust 嵌入 facade。

尚未发布：

- 嵌入式运行时 facade 的稳定 1.0 SDK 承诺。
- 从 dotagents shared directory 发现 `~/.agents/skills/`。
- 远程 Web 控制、浏览器自动化、Web 批准和 Web 写操作/动作路由。
- WASM 插件运行时和插件市场。
- 当前 web/fetch 基础能力之外的 PDF/Office 文档解析。
- 远程或 P2P 会话控制。

## 安装

### 下载 Release

macOS 或 Linux 安装最新 release：

```bash
curl -fsSL https://raw.githubusercontent.com/wjhuang88/talos/main/install/install.sh | sh
```

Windows x86_64 在 PowerShell 中安装最新 release：

```powershell
iex (irm https://raw.githubusercontent.com/wjhuang88/talos/main/install/install.ps1)
```

安装脚本放在 `install/`，因为它们是面向用户的 release 入口。开发和治理脚本放在
`scripts/`；pre-1.0 安装器目录清理后，不再保留旧的 `scripts/install.*` 路径。

也可以从 [GitHub Releases](https://github.com/wjhuang88/talos/releases) 下载对应平台的压缩包，然后解压：

```bash
tar -xzf talos-aarch64-darwin.tar.gz
chmod +x talos
./talos --help
```

已发布的压缩包名称：

| 平台 | 压缩包 |
|---|---|
| Linux x86_64 | `talos-x86_64-linux.tar.gz` |
| Linux ARM64 | `talos-aarch64-linux.tar.gz` |
| macOS Intel | `talos-x86_64-darwin.tar.gz` |
| macOS Apple Silicon | `talos-aarch64-darwin.tar.gz` |
| Windows x86_64 | `talos-x86_64-windows.zip` |

Windows ARM64 产物暂未发布。

### Cargo Install 状态

`cargo install talos-cli --bin talos` 是计划中的 crates.io 二进制安装形态，但目前尚未发布。
当前请使用上面的 release 安装器/压缩包，或通过 `cargo build --release -p talos-cli` 从源码构建。
本地源码 checkout 可用于测试 Cargo 安装：

```bash
cargo install --path crates/talos-cli --bin talos --locked
```

### 首次启动设置

未配置模型时启动 Talos，TUI 会打开模型选择器而不是直接报错。选择一个模型即可开始。
如果该模型的提供商需要凭据，Talos 会提示设置 API key。

在 CI 或非交互环境中跳过向导：

```bash
talos --no-init -p "summarize this repo"
```

### 配置管理

无需手动编辑 TOML 即可查看和修改配置：

```bash
talos --config-list                          # 打印所有设置（密钥已脱敏）
talos --config-get model                     # 查询单个值
talos --config-set model=claude-sonnet-4-20250514  # 设置并持久化
talos --config-set providers.anthropic.api_key_env=ANTHROPIC_API_KEY

# 子命令形式（与上面的 flag 等价）：
talos config list
talos config get model
talos config set model=claude-sonnet-4-20250514
```

## 开发

### 从源码构建

环境要求：

- Rust 1.95 或更新版本
- Cargo

```bash
cargo build --release -p talos-cli
./target/release/talos --help
```

如需在本地构建全部发布产物：

```bash
./build.sh
```

多平台构建产物和校验和会写入 `dist/`。

## 配置模型提供商

Talos 从 `~/.talos/config.toml` 读取配置。密钥可以直接写在配置文件中（`api_key`），也可以通过环境变量引用（`api_key_env`）。内联密钥会保存在配置文件中（建议 chmod 600），并在所有显示输出（`talos config list`、`talos config get`、debug 日志）中自动脱敏。详见 [ADR-023](docs/decisions/023-inline-api-key-boundary.md)。

Anthropic 示例（环境变量模式）：

```toml
provider = "anthropic"
model = "claude-sonnet-4-20250514"

[providers.anthropic]
api_key_env = "ANTHROPIC_API_KEY"
```

Anthropic 示例（内联密钥）：

```toml
provider = "anthropic"
model = "claude-sonnet-4-20250514"

[providers.anthropic]
api_key = "sk-ant-..."
```

OpenAI 兼容网关示例：

```toml
provider = "my-gateway"
model = "your-model"

[providers.my-gateway]
protocol = "openai-chat"
base_url = "https://your-gateway.example.com/v1"
api_key_env = "OPENAI_COMPAT_API_KEY"

[providers.my-gateway.models.your-model]
context_limit = 202752
output_limit = 4096
```

启动 Talos 前设置对应的环境变量：

```bash
export ANTHROPIC_API_KEY="..."
```

## 运行 Talos

在当前目录启动交互式 TUI：

```bash
talos "inspect this repository"
```

使用 print 模式执行一次性提示：

```bash
talos -p "summarize this repository"
```

显式指定工作区：

```bash
talos --workspace /path/to/project "analyze the current architecture"
```

使用 mock provider 做确定性的本地冒烟测试：

```bash
talos -p --mock "/mock-request summarize this repository"
```

### 验证计划

预览 Talos 对某个验证 profile 期望执行的命令，但不实际运行它们：

```bash
talos validate plan --profile workspace
talos validate plan --profile i076
talos validate plan --profile governance --json
```

验证计划是只读表面。它只列出必需检查和缺失前置条件，不会执行命令、安装依赖、编辑文件、
push、publish 或打 tag。

### 管理本地存储

查看本地存储用量（只读）：

```bash
talos storage status
```

预览会被清理的会话（dry-run，不删除文件）：

```bash
talos storage cleanup --max-sessions 20
talos storage cleanup --max-age-days 30 --workspace /path/to/project
```

删除旧会话（需显式 apply 并保护当前活动会话）：

```bash
talos storage cleanup --apply --max-age-days 90 --protect-session <active-uuid>
```

运行 SQLite 维护操作：

```bash
talos storage maintenance --checkpoint --vacuum --reconcile
```

### 记忆

将会话情景记忆固化为语义记忆：

```bash
talos memory consolidate --session <session-uuid>
talos memory consolidate                  # 最近的工作区会话
```

查看记忆库状态（仅计数和大小，不暴露内容）：

```bash
talos memory status
```

预览记忆保留候选（dry-run，不删除）：

```bash
talos memory retention --min-confidence 0.5
```

### 探索库

将本地文件导入可搜索的研究库：

```bash
talos explore ingest --file README.md --title "项目说明"
```

搜索已导入的来源：

```bash
talos explore search --query "会话管理" --limit 10
```

### 交互命令

在交互式 TUI 中，于输入区开头键入 `/` 即可打开命令菜单。继续输入可筛选命令，使用
`Up`/`Down` 移动选项，按 `Enter` 或 `Tab` 完成命令。`Backspace` 编辑筛选内容，`Esc`
关闭菜单但保留输入区文本。使用 `/help` 可以查看当前会话可用的命令。Skill 相关命令也可在
inline 模式中使用。

TUI 中可用的斜杠命令：

| 命令 | 说明 |
|---|---|
| `/help` | 显示可用命令 |
| `/quit`、`/exit` | 退出 Talos |
| `/status` | 显示会话信息（模型、token 用量） |
| `/plugins` | 插件包（暂不可用 — MCP 状态请使用 /mcp） |
| `/mcp` | 显示 MCP 服务状态和已观察到的工具来源 |
| `/skills` | 列出运行时可用技能和当前激活状态 |
| `/skills activate <name>` | 激活一个 Skill 主体，加入后续模型请求上下文 |
| `/skills reference <path>` | 为当前激活 Skill 加载一个有大小上限的引用文件 |
| `/copy last` | 复制上一条助手消息到剪贴板 |
| `/copy all` | 复制完整对话记录到剪贴板 |
| `/export <path>` | 导出对话记录到文件（需权限批准） |
| `/new` | 开始新会话（保留旧会话） |
| `/resume` | 列出可恢复的工作区会话；`/resume <N>` 按序号选择 |
| `/fork` | 分叉当前会话（将历史记录克隆到子会话） |
| `/delete` | 打开会话选择器（排除当前会话）；选择一行进行删除 |
| `/model` | 打开模型选择器，在运行时浏览和切换模型 |

## Skills

Talos 会在启动时发现运行时 Skill 元数据，并把可用 Skill 的简要清单加入模型上下文。
发现路径包括：

- `.talos/skills/`（当前工作区）
- `~/.talos/skills/`
- 从当前目录向上到 Git 根目录之间的父级 `.talos/skills/`

在 TUI 或 inline 模式中使用 `/skills` 查看已发现 Skill 和当前激活状态。使用
`/skills activate <name>` 可以显式加载一个 Skill 的 `SKILL.md` 主体到后续模型请求上下文；
使用 `/skills reference <relative-path>` 可以从该 Skill 目录内加载一个受大小限制的引用文件。
Skill 主体和引用内容不会被直接打印到滚动历史、诊断信息或可见对话导出中。引用路径必须是
相对路径，并且不能逃逸出当前激活 Skill 的目录。

## 内置能力

Talos 自带一组面向编码 Agent 工作流的工具：

- 文件和目录：`read`、`write`、`edit`、`delete`、`ls`、`tree`、`glob`
- 搜索和检查：`grep`、`diff`、`stat`
- 代码智能：`find_symbol`、`find_references`、`list_symbols`、`list_imports`
- Git：`git_status`、`git_diff`、`git_log`、`git_show`、`git_branch_list`、`git_add`、`git_commit`、`git_push`、`git_pull`、`git_checkout`
- 网络：`fetch_url`（有边界的 URL 上下文读取 — 公开页面、HTML 提取、JSON）、`http_request`（按需披露的高级 HTTP/API 检查 — 自定义方法/请求头/请求体，通过 continuation 触发）、`save_url`（下载 URL 到本地文件 — 网络+写入双重权限）、`web_search`
- Shell 兜底：`bash`

默认提示词会要求模型优先使用内置工具，只有在原生工具无法覆盖任务时才使用 Shell 命令兜底。
它也强调准确性优先于迎合：不能奉承、编造引用，证据不足时不能隐藏不确定性。

## MCP 工具

在 `~/.talos/config.toml` 中配置本地 stdio MCP 服务：

```toml
[[mcp.servers]]
name = "filesystem"
transport = "stdio"
command = "/path/to/mcp-server"
args = ["/path/to/workspace"]
env = {}
```

Talos 会在 TUI、print、inline、interactive 和 RPC 模式的首个模型回合前启动已配置的
服务并发现工具。工具名称使用 `mcp:<server>:<tool>` 格式。只读标注会被保留，其他 MCP
工具进入正常批准流程；无法交互批准时默认拒绝。单个服务启动失败不会中止会话，每个
MCP 请求也有超时上限。TUI 中可使用 `/mcp` 查看启动连接快照和本会话已观察到的
工具来源。

会话期间 MCP 工具集保持不变，以维持模型可见工具定义和提示词缓存前缀稳定。修改 MCP
配置后需要重启会话。当前仅支持本地 stdio transport。

## 在 Rust 中嵌入 Talos

Rust 应用可以依赖 `talos-runtime` crate，在不链接 Talos CLI 或 TUI crate 的情况下嵌入
核心 Agent 循环。当前 pre-1.0 初始 facade 提供 `RuntimeBuilder` 和 `RuntimeHandle`，
用于注入 provider/tool、接收类型化事件流、中断、关闭和显式 request preview。
嵌入方也可以通过 `RuntimeBuilder` 提供审批处理器，并替换或追加运行时系统提示词。

注册的工具默认会经过权限包装。在 headless 嵌入模式下，未解决的 `Ask` 决策会被拒绝，
除非 embedder 提供更窄的 allow-list 规则。

这还不是稳定的 1.0 SDK 承诺。当前公开嵌入表面是 `talos-runtime` 以及它从
`talos-core` 重新导出的协议和 trait 类型；低层 `talos-agent` 构造器仍视为实现表面，
除非文档另行声明。

当前 release gate 尚未发布 `talos-runtime` SDK crate。它处于 manifest-ready 状态，但仍被
依赖闭包阻塞；详见 [RUNTIME-SDK-CONTRACT](docs/reference/RUNTIME-SDK-CONTRACT.md) 和
[publish gate packet](docs/reference/PUBLISH-GATE-PACKET-2026-07-02.md)。

## 安全模型

- 只读工作区工具可以免批准执行。
- 文件写入、删除、Git 写操作和 Shell 执行会进入权限流程。
- 工具展示优先显示工具定义声明的关键参数，而不是原始 JSON。
- 本地密钥应放在环境变量或私有配置文件中，不应进入源码。
- Talos 不会自动提交代码。Git 提交只会在用户或工具显式请求时发生。

## 贡献与本地检查

常用检查命令：

```bash
cargo check --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --all
```

GitHub Release 工作流由 tag 触发：

- 稳定版本：`v0.1.0`
- 预发布版本：`v0.1.0-alpha.1`、`v0.1.0-beta.1`、`v0.1.0-rc.1`、`v0.1.0-pre.1`、`v0.1.0-dev.1`

Release 工作流在 macOS runner 上构建 Linux、macOS 和 Windows 产物。

post-v0.2.0 加固素材集中在
[RELEASE-NOTES-DRAFT-2026-07-02](docs/reference/RELEASE-NOTES-DRAFT-2026-07-02.md)。已发布的
`v0.2.1` release 公告和下载以 GitHub Releases 为准。

## 项目状态

Talos 正从核心运行时实现阶段进入产品化加固和差异化体验阶段。接下来的研究重点是：

- `AGENT-002-B`：兼容 dotagents `~/.agents/skills/`。
- `TOOL-004`：先确定搜索引擎方向，再做工具集整体重构。
- `TOOL-007`：工具集综合审计，并纳入 WEBFETCH Phase 2+ 规划。
- `WEB-001`：在只读 loopback dashboard MVP 之外继续扩展本地 Web 控制面。

当前工程状态不再放在 README 中维护，请查看项目治理文档：

- [Board](docs/BOARD.md)：当前、Review 和下一步工作
- [Implementation Roadmap](docs/roadmap/IMPLEMENTATION-ROADMAP.md)：阶段规划
- [Product Backlog](docs/backlog/PRODUCT-BACKLOG.md)：需求和故事清单
- [Iterations](docs/iterations/)：迭代记录和验收证据

## 文档

| 主题 | 文档 |
| --- | --- |
| 架构 | [docs/reference/ARCHITECTURE.md](docs/reference/ARCHITECTURE.md) |
| 参考项目 | [docs/reference/REFERENCE-PROJECTS.md](docs/reference/REFERENCE-PROJECTS.md) |
| 决策记录 | [docs/decisions/](docs/decisions/) |
| 本地开发 | [docs/sop/LOCAL-DEV.md](docs/sop/LOCAL-DEV.md) |
| 测试策略 | [docs/sop/TESTING.md](docs/sop/TESTING.md) |
| Git 工作流 | [docs/sop/GIT-WORKFLOW.md](docs/sop/GIT-WORKFLOW.md) |
| 公开产品站 | [https://talos.hwj.zone](https://talos.hwj.zone) &mdash; 静态 GitHub Pages 站点（源码见 [`site/`](site/)） |

## License

Talos 使用 [Apache License 2.0](LICENSE) 授权。
