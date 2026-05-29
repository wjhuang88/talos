# Talos

A safety-first, minimal-core agent runtime in Rust. CLI-first, evolving into a full agent platform with built-in self-evolution.

English | **[中文](README.zh-CN.md)**

## Status

**Pre-implementation.** Architecture design and governance are complete. Implementation follows an agile vertical-slice roadmap — each iteration produces a runnable, testable `talos` binary.

## Roadmap

| Iteration | Codename | User can... |
|-----------|----------|-------------|
| I001 | Hello Agent | `talos "What is 2+2?" -p` and get an LLM response |
| I002 | Tool User | Ask the agent to perform file and shell operations |
| I003 | Safe Agent | Dangerous operations get intercepted by permissions |
| I004 | Smart Agent | Long conversations (50+ turns) without context overflow |
| I005 | Skilled Agent | Load SKILL.md, self-evolve from experience |
| I006 | Extensible Agent | Hook system, MCP protocol, plugin runtime |
| I007 | Polished Agent | Full TUI, multi-mode interaction, release-ready |

## Architecture

Talos follows a **simple core, flexible extensions** design philosophy:

- **Core** (5 crates): Minimal turn loop — config, provider, agent, CLI, and foundation types.
- **Extensions** (9 crates): Introduced on demand — tools, session, sandbox, permissions, skills, evolution, plugins, MCP, RPC.

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
- **Progressive storage**: Pure files (I001–I003) → SQLite index (I004) → SQLite evolution tables (I005). [ADR-002](docs/decisions/002-local-storage-architecture.md).
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
- **TUI**: ratatui (I007)

## License

Licensed under the [Apache License 2.0](LICENSE).
