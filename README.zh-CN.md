# Talos

安全优先、精简内核的 Rust Agent 运行时。Talos 从 CLI 编码助手起步，正在收敛为完整的 Agent runtime：具备自进化、扩展接口、便携工具，以及更接近普通命令行的终端体验。

**[English](README.md)** | 中文

## 当前状态

| 范围 | 状态 | 说明 |
|------|------|------|
| Runtime | Active | 12 个 crate 共 515 个测试通过。TTY 默认启动 Nord 主题 TUI；`--repl` 保留旧 readline 模式。 |
| R1 Review Closure | Active | 当前执行轮：收口 I008/I009 review 漂移，暂停 I011 S2，然后进入 I010 R2。 |
| I008 Learning Agent | Review | `EvolutionHookHandler` 已接入 print、TUI、interactive、RPC 四条路径；剩余最终 review 证据和状态同步。 |
| I009 Extensible Agent | Review | Hook、MCP client/server、JSON-RPC、`ToolProvenance` producer 已实现；TUI provenance marker 与 `/plugins` 仍是 follow-up。 |
| I010 Polished Agent | Planned | Codex-like inline terminal mode、AppServerSession 收敛、TUI 打磨、Markdown、diff、slash commands。 |
| I011 Open Providers | Paused | S1 OpenAI-compatible `base_url` override 已落地；S2 provider plugin architecture 暂缓。 |
| I012 Portable Tools | Planned | Rust-native POSIX 基本工具子集 + 工具包嵌入接口，降低外部环境依赖。 |

R0 已关闭权限安全、session index、fork identity、搜索高亮、process hardening 等架构修复项。详见 [R0 remediation](docs/iterations/R0-remediation-gate.md)。

## 快速开始

```bash
cargo build -p talos-cli
```

配置 provider token：

```bash
export ANTHROPIC_API_KEY="<your key>"
# 或
export OPENAI_API_KEY="<your key>"
```

运行默认 TUI：

```bash
cargo run -p talos-cli -- "help me inspect this repository"
```

使用 print mode 获得更接近普通命令行的输出：

```bash
cargo run -p talos-cli -- -p "summarize the project status"
```

使用 OpenAI-compatible gateway：

```toml
# ~/.talos/config.toml
provider = "openai"
model = "your-model"
base_url = "https://your-gateway.example.com/v1"
```

```bash
export OPENAI_COMPAT_API_KEY="<your gateway key>"
cargo run -p talos-cli -- -p "用中文回答: 1+1=?"
```

## 已具备能力

- 文件和 Shell 操作通过权限管线执行。
- JSONL 会话源数据 + bundled SQLite 搜索/索引。
- `SKILL.md` 技能系统、渐进式披露和 prompt 集成。
- Anthropic、OpenAI、OpenAI-compatible gateway 多模型支持。
- 运行时自进化：观察 -> 积累 -> 提取 -> 应用。
- 扩展接口：hook、MCP client/server、stdio JSON-RPC、typed tool provenance。

## 路线图

| 迭代 | 代号 | 状态 | 结果 |
|------|------|------|------|
| I001-I007 | Foundation through Skilled Agent | Complete | CLI、工具、权限、TUI 基础、会话、SQLite 搜索、技能、多 provider。 |
| R0 | Remediation Gate | Complete | 架构、安全、会话正确性问题关闭。 |
| R1 | Review Closure | Active | 在启动 I010 R2 前收口 I008/I009 review 漂移。 |
| I008 | Learning Agent | Review | 运行时学习已实现，等待最终 review 证据。 |
| I009 | Extensible Agent | Review | 后端/runtime 扩展能力已实现；TUI consumer 工作待完成。 |
| I010 | Polished Agent | Planned | Codex-like 终端体验和发布级 TUI 工作流。 |
| I011 | Open Providers | Paused | 可配置 OpenAI-compatible gateway 已交付；Provider 插件架构暂缓。 |
| I012 | Portable Tools | Planned | 内置 POSIX-style 工具和工具包嵌入。 |

项目按垂直切片推进：每轮迭代都应交付可运行、可测试的 `talos` 二进制。需求闭环见 [Requirement Convergence](docs/roadmap/REQUIREMENT-CONVERGENCE.md)。

## 架构

Talos 遵循简单内核、灵活扩展的设计：

- **核心 crates**：config、provider、agent、CLI、共享协议/类型。
- **扩展 crates**：tools、session、sandbox、permissions、TUI、skills、evolution、plugins、MCP、RPC。

```text
[ talos-cli / talos-rpc ]
          |
          v
    [ talos-agent ]
    /     |     \
   v      v      v
[tools][session][provider][permission][skill][plugin][mcp]
   \      |      /           |           |      /     /
    \     v     /            v           v     /     /
     [ talos-core ] <-------------------------------'
                      |
                      v
               [ talos-evolution ]
```

## 关键设计决策

- **流式优先**：LLM 通信围绕 SSE streaming 和双通道异步流构建。
- **全链路安全**：工具调用必须经过权限、沙箱和审批策略。
- **自进化是运行时能力**：学习是 runtime primitive，不是技能系统功能。见 [ADR-001](docs/decisions/001-runtime-self-evolution.md)。
- **渐进式存储**：先 JSONL，需要 FTS/index/query 行为时引入 SQLite。见 [ADR-002](docs/decisions/002-local-storage-architecture.md)。
- **内置 SQLite**：`rusqlite/bundled` 是已批准的存储例外；Talos 不依赖系统 SQLite。见 [ADR-008](docs/decisions/008-sqlite-bundled-storage.md)。
- **工具来源追踪**：native 和 MCP-remote 工具有 typed provenance，服务后续 TUI/plugin/RPC consumer。见 [ADR-009](docs/decisions/009-tool-provenance.md)。

## 文档

项目治理由 [agent-project-governance](https://github.com/wjhuang88/agent-project-governance)
skill 辅助建立和审计。

| 路径 | 用途 |
|------|------|
| [AGENTS.md](AGENTS.md) | Agent 编码指南、硬性约束、任务路由 |
| [docs/README.md](docs/README.md) | 文档地图 |
| [docs/roadmap/REQUIREMENT-CONVERGENCE.md](docs/roadmap/REQUIREMENT-CONVERGENCE.md) | 需求到实现的闭环追踪 |
| [docs/roadmap/IMPLEMENTATION-ROADMAP.md](docs/roadmap/IMPLEMENTATION-ROADMAP.md) | 迭代计划和执行顺序 |
| [docs/backlog/PRODUCT-BACKLOG.md](docs/backlog/PRODUCT-BACKLOG.md) | 用户故事、验收标准、计划工作 |
| [docs/iterations/](docs/iterations/) | 迭代计划、状态、执行证据 |
| [docs/decisions/](docs/decisions/) | 架构决策记录（ADR） |
| [docs/reference/ARCHITECTURE.md](docs/reference/ARCHITECTURE.md) | 完整架构参考 |

## 技术栈

| 层 | 选择 |
|----|------|
| 语言 | Rust stable, edition 2024 |
| 异步 | tokio |
| 序列化 | serde + schemars |
| 错误处理 | library 使用 thiserror，CLI 使用 anyhow |
| 存储 | JSONL、TOML、SQLite via `rusqlite/bundled` |
| TUI | ratatui + crossterm |

## 许可证

基于 [Apache License 2.0](LICENSE) 许可。
