# Talos

[![Release](https://github.com/wjhuang88/talos/actions/workflows/release.yml/badge.svg)](https://github.com/wjhuang88/talos/actions/workflows/release.yml)
[![Latest Release](https://img.shields.io/github/v/release/wjhuang88/talos?include_prereleases)](https://github.com/wjhuang88/talos/releases)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.95%2B-orange)](https://www.rust-lang.org/)

[English](README.md)

Talos 是一个 Rust 原生的本地编码 Agent 运行时。它提供终端 UI、模型提供商适配、会话、内置工具、权限控制、Skills、MCP/RPC 集成和自进化钩子，同时保持核心边界清晰、默认安全。

Talos 仍处于 1.0 之前的活跃开发阶段。核心 CLI/TUI、工具管线、Git 工具、会话、Skills、MCP/RPC 服务、模型配置、提示词缓存和工程治理流程已经可用。工程进度请查看[项目状态](#项目状态)中的链接。

## 主要能力

- **终端优先的 Agent 体验**：提供交互式 TUI，也支持脚本友好的 print 模式。
- **Rust 原生核心**：基于 Cargo workspace 的小 crate 架构，边界明确。
- **内置工具**：覆盖文件、搜索、编辑、Shell、符号索引、目录树、diff/stat 和 Git 操作。
- **写操作权限控制**：文件写入、删除、Git 写操作和命令执行都会进入批准流程。
- **模型提供商适配**：支持 Anthropic Messages、OpenAI Chat、OpenAI Responses 和 OpenAI 兼容网关。
- **会话记忆**：基于 SQLite 的会话、搜索、摘要、分支和导出能力。
- **可扩展性**：支持 Skills、Hooks、MCP 工具、RPC 服务和面向协议的扩展设计。

## 安装

### 下载 Release

从 [GitHub Releases](https://github.com/wjhuang88/talos/releases) 下载对应平台的压缩包，然后解压：

```bash
tar -xzf talos-aarch64-apple-darwin.tar.gz
chmod +x talos
./talos --help
```

Windows 发布包是 `.zip`，macOS 和 Linux 发布包是 `.tar.gz`。

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
```

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

Talos 从 `~/.talos/config.toml` 读取配置。密钥建议放在环境变量中。

Anthropic 示例：

```toml
provider = "anthropic"
model = "claude-sonnet-4-20250514"

[providers.anthropic]
api_key_env = "ANTHROPIC_API_KEY"
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

### 交互命令

在交互式 TUI 中，于输入区开头键入 `/` 即可打开命令菜单。继续输入可筛选命令，使用
`Up`/`Down` 移动选项，按 `Enter` 或 `Tab` 完成命令。`Backspace` 编辑筛选内容，`Esc`
关闭菜单但保留输入区文本。使用 `/help` 可以查看当前会话可用的命令。

TUI 中可用的斜杠命令：

| 命令 | 说明 |
|---|---|
| `/help` | 显示可用命令 |
| `/quit`、`/exit` | 退出 Talos |
| `/status` | 显示会话信息（模型、token 用量） |
| `/plugins` | 列出已观察的工具来源和 MCP 服务状态 |
| `/skills` | 列出运行时发现的技能（Level 0 元数据） |
| `/copy last` | 复制上一条助手消息到剪贴板 |
| `/copy all` | 复制完整对话记录到剪贴板 |
| `/export <path>` | 导出对话记录到文件（需权限批准） |
| `/new` | 开始新会话（保留旧会话） |
| `/resume` | 列出可恢复的工作区会话；`/resume <N>` 按序号选择 |
| `/fork` | 分叉当前会话（将历史记录克隆到子会话） |
| `/delete` | 打开会话选择器（排除当前会话）；选择一行进行删除 |
| `/model` | 打开模型选择器，在运行时浏览和切换模型 |

## 内置能力

Talos 自带一组面向编码 Agent 工作流的工具：

- 文件和目录：`read`、`write`、`edit`、`delete`、`ls`、`tree`、`glob`
- 搜索和检查：`grep`、`diff`、`stat`
- 代码智能：`find_symbol`、`find_references`、`list_symbols`、`list_imports`
- Git：`git_status`、`git_diff`、`git_log`、`git_show`、`git_branch_list`、`git_add`、`git_commit`、`git_push`、`git_pull`、`git_checkout`
- Shell 兜底：`bash`

默认提示词会要求模型优先使用内置工具，只有在原生工具无法覆盖任务时才使用 Shell 命令兜底。

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
MCP 请求也有超时上限。TUI 中可使用 `/plugins` 查看启动连接快照和本会话已观察到的
工具来源。

会话期间 MCP 工具集保持不变，以维持模型可见工具定义和提示词缓存前缀稳定。修改 MCP
配置后需要重启会话。当前仅支持本地 stdio transport。

## 安全模型

- 只读工作区工具可以免批准执行。
- 文件写入、删除、Git 写操作和 Shell 执行会进入权限流程。
- 工具展示优先显示工具定义声明的关键参数，而不是原始 JSON。
- 本地密钥应放在环境变量或私有配置文件中，不应进入源码。
- Talos 不会自动提交代码。Git 提交只会在用户或工具显式请求时发生。

## 开发

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

## 项目状态

Talos 正从核心运行时实现阶段进入产品化加固阶段。当前工程状态不再放在 README 中维护，请查看项目治理文档：

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

## License

Talos 使用 [Apache License 2.0](LICENSE) 授权。
