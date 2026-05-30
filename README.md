# Talos

A safety-first, minimal-core agent runtime in Rust. CLI-first, evolving into a full agent platform with built-in self-evolution.

English | **[中文](README.zh-CN.md)**

## Status

**I004 complete.** 167 tests passing. Agent can perform file and shell operations safely with permission gating. Implementation follows an agile vertical-slice roadmap — each iteration produces a runnable, testable `talos` binary.

## Roadmap

| Iteration | Codename | User can... |
|-----------|----------|-------------|
| ~~I001~~ | ~~Project Scaffold~~ | ~~`cargo check --workspace` passes~~ ✅ |
| ~~I002~~ | ~~Hello Agent~~ | ~~`talos "What is 2+2?" -p` and get an LLM response~~ ✅ |
| ~~I003~~ | ~~Tool User~~ | ~~Ask the agent to perform file and shell operations~~ ✅ |
| ~~I004~~ | ~~Safe Agent~~ | ~~Dangerous operations get intercepted by permissions~~ ✅ |
| ~~I005~~ | ~~Smart Agent~~ | ~~Mock LLM + basic TUI + context compaction + caching~~ ✅ |
| I006 | Data Agent | TUI tool display + approval + session branching + SQLite search |
| I007 | Skilled Agent | TUI skill display + SKILL.md + multi-provider support |
| I008 | Learning Agent | TUI evolution display + self-evolution engine |
| I009 | Extensible Agent | TUI MCP display + Hook system + MCP + JSON-RPC |
| I010 | Polished Agent | Full TUI polish (Nord theme + markdown + advanced features) |

## Architecture

Talos follows a **simple core, flexible extensions** design philosophy:

- **Core** (5 crates): Minimal turn loop — config, provider, agent, CLI, and foundation types.
- **Extensions** (10 crates): Introduced on demand — tools, session, sandbox, permissions, TUI, skills, evolution, plugins, MCP, RPC.

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
```

### Key Design Decisions

- **Streaming-first**: All LLM communication via SSE streaming. Dual-channel async (SQ/EQ).
- **Safety at every layer**: Permission pipeline, sandboxed tool execution, crash-safe session storage.
- **Built-in self-evolution**: Runtime-level learning loop (Observe → Accumulate → Extract → Apply), not a skill feature. [ADR-001](docs/decisions/001-runtime-self-evolution.md).
- **Progressive storage**: Pure files (I001–I005) → SQLite index (I006) → SQLite evolution tables (I008). [ADR-002](docs/decisions/002-local-storage-architecture.md).
- **File-based by default**: Config (TOML), skills (Markdown), sessions (JSONL). Human-editable, git-friendly.

## Documentation

| Path | Purpose |
|------|---------|
| [AGENTS.md](AGENTS.md) | Agent coding guide, hard constraints, task router |
| [docs/reference/ARCHITECTURE.md](docs/reference/ARCHITECTURE.md) | Full architecture reference |
| [docs/roadmap/IMPLEMENTATION-ROADMAP.md](docs/roadmap/IMPLEMENTATION-ROADMAP.md) | Iteration-by-iteration plan |
| [docs/backlog/PRODUCT-BACKLOG.md](docs/backlog/PRODUCT-BACKLOG.md) | User stories and acceptance criteria |
| [docs/decisions/](docs/decisions/) | Architecture Decision Records |
| [docs/reference/REFERENCE-PROJECTS.md](docs/reference/REFERENCE-PROJECTS.md) | Reference project patterns and source links |

## Tech Stack

- **Language**: Rust (stable, edition 2024)
- **Async**: tokio
- **Serialization**: serde + schemars
- **Errors**: thiserror (libraries), anyhow (CLI)
- **Storage**: JSONL (sessions), TOML (config), SQLite via rusqlite bundled (index, evolution)
- **TUI**: ratatui + crossterm (I005+, evolving progressively, Nord theme)

## License

Licensed under the [Apache License 2.0](LICENSE).
