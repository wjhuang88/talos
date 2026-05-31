# Talos

安全优先、精简内核的 Rust Agent 运行时。从 CLI 起步，逐步演化为完整的 Agent 平台，内置自进化能力。

**[English](README.md)** | 中文

---

## 项目状态

**I004 已完成。** 167 个测试通过。Agent 可以安全地执行文件和 Shell 操作，并带有权限门控。开发遵循敏捷垂直切片路线图——每个迭代交付可运行、可测试的 `talos` 二进制文件。

## 路线图

| 迭代 | 代号 | 用户可以... |
|------|------|------------|
| ~~I001~~ | ~~Project Scaffold~~ | ~~`cargo check --workspace` 通过~~ ✅ |
| ~~I002~~ | ~~Hello Agent~~ | ~~`talos "What is 2+2?" -p` 获得流式 LLM 响应~~ ✅ |
| ~~I003~~ | ~~Tool User~~ | ~~让 Agent 执行文件和 Shell 操作~~ ✅ |
| ~~I004~~ | ~~Safe Agent~~ | ~~危险操作被权限系统拦截~~ ✅ |
| ~~I005~~ | ~~Smart Agent~~ | ~~Mock LLM + 基础 TUI + 上下文压缩 + 缓存~~ ✅ |
| ~~I006~~ | ~~Data Agent~~ | ~~TUI 工具展示 + 审批 + 会话分支 + SQLite 搜索~~ ✅ |
| ~~I007~~ | ~~Skilled Agent~~ | ~~TUI 技能展示 + SKILL.md + 多模型支持~~ ✅ |
| ~~I008~~ | ~~Learning Agent~~ | ~~TUI 进化展示 + 自进化引擎~~ ✅ |
| I009 | Extensible Agent | TUI MCP 展示 + Hook 系统 + MCP + JSON-RPC |
| I010 | Polished Agent | 完整 TUI 打磨（Nord 主题 + Markdown + 高级功能） |

## 架构

Talos 遵循**简单内核、灵活扩展**的设计哲学：

- **核心**（5 个 crate）：最小化 turn loop — 配置、模型提供者、Agent、CLI 和基础类型。
- **扩展**（11 个 crate）：按需引入 — 工具、会话、沙箱、权限、TUI、技能、自进化、插件、MCP、RPC。

```
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

### 核心设计决策

- **流式优先**：所有 LLM 通信基于 SSE 流式传输。双通道异步架构（SQ/EQ）。
- **全链路安全**：权限管线、沙箱化工具执行、崩溃安全的会话存储。
- **内置自进化**：运行时级别学习循环（观察 → 积累 → 提取 → 应用），而非技能系统功能。[ADR-001](docs/decisions/001-runtime-self-evolution.md)。
- **渐进式存储**：纯文件（I001–I005）→ SQLite 索引（I006）→ SQLite 演化表（I008）。[ADR-002](docs/decisions/002-local-storage-architecture.md)。
- **默认文件驱动**：配置（TOML）、技能（Markdown）、会话（JSONL）。人类可编辑、git 友好。

## 文档

| 路径 | 说明 |
|------|------|
| [AGENTS.md](AGENTS.md) | Agent 编码指南、硬性约束、任务路由 |
| [docs/reference/ARCHITECTURE.md](docs/reference/ARCHITECTURE.md) | 完整架构参考 |
| [docs/roadmap/IMPLEMENTATION-ROADMAP.md](docs/roadmap/IMPLEMENTATION-ROADMAP.md) | 逐迭代实现计划 |
| [docs/backlog/PRODUCT-BACKLOG.md](docs/backlog/PRODUCT-BACKLOG.md) | 用户故事与验收标准 |
| [docs/decisions/](docs/decisions/) | 架构决策记录（ADR） |
| [docs/reference/REFERENCE-PROJECTS.md](docs/reference/REFERENCE-PROJECTS.md) | 参考项目模式与源码链接 |

## 技术栈

- **语言**：Rust（stable，edition 2024）
- **异步**：tokio
- **序列化**：serde + schemars
- **错误处理**：thiserror（库）、anyhow（CLI）
- **存储**：JSONL（会话）、TOML（配置）、SQLite via rusqlite bundled（索引、演化）
- **TUI**：ratatui + crossterm（I005+，逐步演进，Nord 主题）

## 许可证

基于 [Apache License 2.0](LICENSE) 许可。
