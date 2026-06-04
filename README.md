# Talos

A safety-first, minimal-core agent runtime in Rust. Talos starts as a CLI coding
assistant and is converging toward a full agent runtime with self-evolution,
extension surfaces, portable tools, and a terminal experience that feels native to
the command line.

English | **[中文](README.zh-CN.md)**

## Current Status

| Area | State | Notes |
|------|-------|-------|
| Runtime | Active | 532 tests passing across 12 crates. TTY launches the Nord-themed TUI by default; `--repl` keeps the legacy readline loop. |
| R1 Review Closure | Complete | I008/I009 closed. I009 TUI consumer work deferred to #I009-S6. I010 R2 is complete; R3 product polish is next. |
| I008 Learning Agent | Complete | `EvolutionHookHandler` wired into all run paths; runtime evidence recorded. |
| I009 Extensible Agent | Complete | Hooks, MCP client/server, JSON-RPC, and `ToolProvenance` producers shipped. TUI markers deferred to #I009-S6. |
| I010 Polished Agent | Active (R3 planned) | R2 AppServerSession convergence and inline mode complete; R3 covers TUI polish, markdown, diff display, and slash commands. |
| I011 Open Providers | Paused | S1 OpenAI-compatible `base_url` override shipped; S2 provider plugin architecture deferred. |
| I012 Portable Tools | Planned | Rust-native POSIX-style tool subset plus embeddable tool-pack registration to reduce host environment dependency. |

Recent remediation work closed R0 architecture findings around permission safety,
session index correctness, fork identity, search highlighting, and process hardening.
See [R0 remediation](docs/iterations/R0-remediation-gate.md).

## Quick Start

```bash
cargo build -p talos-cli
```

Configure a provider token:

```bash
export ANTHROPIC_API_KEY="<your key>"
# or
export OPENAI_API_KEY="<your key>"
```

Run the default TUI:

```bash
cargo run -p talos-cli -- "help me inspect this repository"
```

Use print mode for shell-style output:

```bash
cargo run -p talos-cli -- -p "summarize the project status"
```

Use an OpenAI-compatible gateway:

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

## What Works

- Safe file and shell operations through the permission pipeline.
- Session storage with JSONL source-of-truth and bundled SQLite search/indexing.
- Skills via `SKILL.md`, progressive disclosure, and prompt integration.
- Multi-provider support with Anthropic, OpenAI, and OpenAI-compatible gateways.
- Runtime self-evolution through Observe -> Accumulate -> Extract -> Apply.
- Extension surfaces: hooks, MCP client/server, stdio JSON-RPC, typed tool provenance.

## Roadmap

| Iteration | Codename | Status | Outcome |
|-----------|----------|--------|---------|
| I001-I007 | Foundation through Skilled Agent | Complete | CLI, tools, permissions, TUI base, sessions, SQLite search, skills, multi-provider support. |
| R0 | Remediation Gate | Complete | Architecture/security/session correctness findings closed. |
| R1 | Review Closure | Complete | I008/I009 closed; I009 TUI consumer work deferred to #I009-S6. |
| I008 | Learning Agent | Complete | Runtime self-evolution via hook-based `EvolutionHookHandler` across all paths. |
| I009 | Extensible Agent | Complete | Hooks, MCP client/server, JSON-RPC, provenance producers shipped. |
| I010 | Polished Agent | Active (R3 planned) | R2 Codex-like terminal UX complete; R3 targets release-grade TUI workflows. |
| I011 | Open Providers | Paused | Configurable OpenAI-compatible gateway support shipped; provider plugin architecture deferred. |
| I012 | Portable Tools | Planned | Built-in POSIX-style tools and tool-pack embedding. |

Implementation follows vertical slices: every iteration should produce a runnable,
testable `talos` binary. Requirement closure is tracked in
[Requirement Convergence](docs/roadmap/REQUIREMENT-CONVERGENCE.md).

## Architecture

Talos follows a simple core, flexible extensions design:

- **Core crates**: config, provider, agent, CLI, and shared protocol/types.
- **Extension crates**: tools, session, sandbox, permissions, TUI, skills,
  evolution, plugins, MCP, and RPC.

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

## Design Decisions

- **Streaming-first**: LLM communication is built around SSE streaming and dual-channel async flow.
- **Safety at every layer**: Tool calls pass through permissions, sandboxing, and approval policy.
- **Self-evolution is runtime-level**: learning is a first-class runtime primitive, not a skill feature. See [ADR-001](docs/decisions/001-runtime-self-evolution.md).
- **Progressive storage**: JSONL first, SQLite when FTS/index/query behavior is needed. See [ADR-002](docs/decisions/002-local-storage-architecture.md).
- **Bundled SQLite**: `rusqlite/bundled` is an approved storage exception; Talos does not require system SQLite. See [ADR-008](docs/decisions/008-sqlite-bundled-storage.md).
- **Tool provenance**: native and MCP-remote tools carry typed provenance for future TUI/plugin/RPC consumers. See [ADR-009](docs/decisions/009-tool-provenance.md).

## Documentation

Project governance is guided by the
[agent-project-governance](https://github.com/wjhuang88/agent-project-governance)
skill.

| Path | Purpose |
|------|---------|
| [AGENTS.md](AGENTS.md) | Agent coding guide, hard constraints, task router |
| [docs/README.md](docs/README.md) | Documentation map |
| [docs/roadmap/REQUIREMENT-CONVERGENCE.md](docs/roadmap/REQUIREMENT-CONVERGENCE.md) | Requirement-to-implementation closure tracking |
| [docs/roadmap/IMPLEMENTATION-ROADMAP.md](docs/roadmap/IMPLEMENTATION-ROADMAP.md) | Iteration plan and execution sequence |
| [docs/backlog/PRODUCT-BACKLOG.md](docs/backlog/PRODUCT-BACKLOG.md) | Stories, acceptance criteria, and planned work |
| [docs/iterations/](docs/iterations/) | Iteration plans, status, and execution evidence |
| [docs/decisions/](docs/decisions/) | Architecture Decision Records |
| [docs/reference/ARCHITECTURE.md](docs/reference/ARCHITECTURE.md) | Full architecture reference |

## Tech Stack

| Layer | Choice |
|-------|--------|
| Language | Rust stable, edition 2024 |
| Async | tokio |
| Serialization | serde + schemars |
| Errors | thiserror for libraries, anyhow for CLI |
| Storage | JSONL, TOML, SQLite via `rusqlite/bundled` |
| TUI | ratatui + crossterm |

## License

Licensed under the [Apache License 2.0](LICENSE).
