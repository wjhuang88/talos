# Talos

A safety-first, minimal-core agent runtime in Rust. Talos starts as a CLI coding
assistant and is converging toward a full agent runtime with self-evolution,
extension surfaces, portable tools, and a terminal experience that feels native to
the command line.

English | **[中文](README.zh-CN.md)**

## Current Status

| Area | State | Notes |
|------|-------|-------|
| Runtime | Active | Event-driven TUI: `talos-conversation` crate separates business logic from UI rendering. Two-loop design (Agent → ConversationEngine → UI) via typed async channels. Codex-style single-row history insertion, stream-based content delivery, single-directional flow. Styled scrollback with Nord bg for user messages, 3-column line padding, multiline paste blocks, single-row preview with Markdown block classification and conservative styled Markdown rendering, animated braille spinner, native cursor sync. 114 focused TUI+conversation tests pass (61 TUI + 53 conversation). |
| R1 Review Closure | Complete | I008/I009 closed. I009 TUI consumer work deferred to #I009-S6. I010 R3 product polish complete. |
| I008 Learning Agent | Complete | `EvolutionHookHandler` wired into all run paths; runtime evidence recorded. |
| I009 Extensible Agent | Complete | Hooks, MCP client/server, JSON-RPC, and `ToolProvenance` producers shipped. TUI markers shipped in I014. |
| I010 Polished Agent | Active (R3 complete) | R2 AppServerSession convergence; R3 Nord theme, markdown rendering, diff display, steering queues, slash commands. |
| I011/I015 Providers | Active | Named provider/model schema landed for built-in and OpenAI-compatible providers; dynamic provider loading remains deferred. |
| I013 Boundary Control | Complete | Guardian/exec/provider ADRs recorded; logging R1 centralized. |
| I014 TUI Completion | Complete | Tool provenance markers + `/plugins` (#I009-S6) and `/copy` + `/export` with OSC 52 + pbcopy + permission gating (#I010-S9). 652 tests pass. |
| I015-I017 Follow-up Plan | Planned | Provider schema, portable file/search tools, and embedded Git tools. |
| I018-I020 Architecture Plan | Planned | Bounded logs, embedded prompt assets, layered memory, and local research library. |
| I021 Evolution Realignment | Complete | Root-cause fix for the 5MB knowledge.db bloat and `400 Bad Request` loop. 5 atomic commits realigned `talos-evolution` with the MenteDB blueprint; 7470ac5 byte-cap stays as defense-in-depth. |
| I022 TUI Inline-by-Default | Complete | Codex-style inline-by-default TUI: fixed viewport, real-time scrollback flush, status bar tips with TTL. 127 TUI tests pass. |
| I023 TUI State Model | Complete | Event-driven architecture: `talos-conversation` crate owns business logic, `talos-tui` owns pure UI state. Two-loop design with typed async channels. Codex-style single-row history insertion with styled scrollback, 3-column line padding, Nord-themed multiline user message blocks with top/bottom padding, single-row preview with Markdown block classification and conservative styled Markdown rendering, animated braille spinner, native cursor sync. Non-lossy mpsc delivery, agent abort-on-cancel, SIGINT fallback. 114 focused TUI+conversation tests pass. |
| I026 Approval UX + Git + Prompt Optimization | Review | Approval/tool-call ordering fixed in the streamed UI event flow. Built-in Git tools delivered. Dynamic prompt template slots and Anthropic cache-control emission landed. |

Recent remediation work closed R0 architecture findings around permission safety,
session index correctness, fork identity, search highlighting, and process hardening.
See [R0 remediation](docs/iterations/R0-remediation-gate.md).

## Quick Start

```bash
cargo build -p talos-cli
```

Configure a provider token — either via environment variable or directly in your local
`~/.talos/config.toml`:

```bash
# Environment variable (recommended for shared/CI environments)
export ANTHROPIC_API_KEY="<your key>"
# or
export OPENAI_API_KEY="<your key>"
```

```toml
# ~/.talos/config.toml — inline api_key is also supported
provider = "anthropic"
model = "claude-sonnet-4-20250514"

[providers.anthropic]
api_key = "<your key>"   # not echoed back when config is re-serialized
```

When both are set, the inline `api_key` takes precedence. Set
`chmod 600 ~/.talos/config.toml` if you store credentials there.

Run the default TUI:

```bash
cargo run -p talos-cli -- "help me inspect this repository"
```

On startup Talos prints a branded splash to the terminal scrollback: a block-letter
`TALOS` wordmark with a Nord Frost gradient, the `⬡ The watchman never sleeps`
tagline, capability badges, and the version line. It stays inline (no alt-screen),
so the splash remains in your scrollback as the conversation begins. Narrow
terminals (< 80 columns) automatically fall back to a compact wordmark.

Use print mode for shell-style output:

```bash
cargo run -p talos-cli -- -p "summarize the project status"
```

Use mock mode for local request inspection without making a provider call:

```bash
cargo run -p talos-cli -- -p --mock "/mock-request summarize this repository"
```

This prints the provider request snapshot Talos would send, including method,
URL, headers, and JSON body. The `/mock-request` wrapper is stripped from the
snapshot body, and credential headers are redacted in the output. The same
`/mock-request <prompt>` wrapper also works from interactive/TUI input when
Talos is running with `--mock`.

Set an explicit workspace root when launching Talos from another directory:

```bash
cargo run -p talos-cli -- --workspace /path/to/project "inspect this repository"
```

Use an OpenAI-compatible gateway:

```toml
# ~/.talos/config.toml
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

```bash
export OPENAI_COMPAT_API_KEY="<your gateway key>"
cargo run -p talos-cli -- -p "用中文回答: 1+1=?"
```

## What Works

- Safe file and shell operations through the permission pipeline.
- 25 built-in tools: bash, read (with offset/limit), write, edit, delete, grep, glob, ls (with long format), diff, stat, tree, find_symbol, find_references, list_symbols, list_imports, git_status, git_diff, git_log, git_show, git_branch_list, git_add, git_commit, git_push, git_pull, git_checkout.
- Workspace-aware permissions: read-only tools auto-allowed in workspace; write/execute require approval.
- System prompt guides the LLM to prefer dedicated tools over bash for file operations.
- Session storage with JSONL source-of-truth and bundled SQLite search/indexing.
- Skills via `SKILL.md`, progressive disclosure, and prompt integration.
- Multi-provider support with named Anthropic, OpenAI, and OpenAI-compatible gateway configs.
- Configurable tool calling protocol per provider (native, talos-strict, compat).
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
| I010 | Polished Agent | Active (R3 complete) | R2 AppServerSession convergence + inline mode; R3 Nord theme, markdown, diff display, steering queues, slash commands. |
| I011/I015 | Providers | Active | Named provider/model schema for built-in and OpenAI-compatible gateways; provider plugin architecture deferred. |
| I013 | Boundary Control | Complete | Guardian/exec/provider ADRs plus centralized logging R1. |
| I014 | TUI Completion | Complete | Tool provenance markers + `/plugins` (#I009-S6) and `/copy` + `/export` with OSC 52 + pbcopy + permission gating (#I010-S9). |
| I015-I017 | Follow-up Plan | Planned | Provider schema, portable file/search tools, embedded Git tools. |
| I018-I020 | Memory/Research Plan | Planned | Log retention, prompt assets, layered memory foundation, exploration library. |
| I021 | Evolution MenteDB Realignment | Complete | Root-cause fix for the 5MB knowledge.db bloat / `400 Bad Request` loop. Realigns `Signal.context` semantics, `TurnObservation` schema, and `Pattern` provenance per the MenteDB blueprint; defense layer from `7470ac5` stays as belt-and-suspenders. |
| I022 | TUI Inline-by-Default | Complete | Codex-style inline-by-default TUI: fixed 4-line viewport, real-time scrollback flush, status bar tips with TTL. 127 TUI tests pass. |
| I023 | TUI State Model | Complete | Event-driven `talos-conversation` + `talos-tui` state model, styled multiline scrollback, Markdown block classification and conservative styled rendering, spinner, cursor sync. Non-lossy mpsc delivery, agent abort-on-cancel, SIGINT fallback. |
| I025 | Tool Pipeline Completion | Complete | TOOL-002 P1-P2 residual (schema validation, dedup), TOOL-003 P1 (diff, stat), fence info-string fix, Mermaid code block rendering (mermaid-text), ToolNature attribute replacing name-based permission matching. |
| I026 | Approval UX + Git + Prompt Optimization | Review | Approval/tool-call ordering fixed in streamed UI events; read/write Git tools delivered; dynamic prompt template slots and Anthropic cache-control emission landed; active owner docs synchronized. |

Implementation follows vertical slices: every iteration should produce a runnable,
testable `talos` binary. Requirement closure is tracked in
[Requirement Convergence](docs/roadmap/REQUIREMENT-CONVERGENCE.md).

## Architecture

Talos follows a simple core, flexible extensions design:

- **Core crates**: config, provider, agent, CLI, and shared protocol/types.
- **Extension crates**: tools, session, sandbox, permissions, conversation, TUI, skills,
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
- **Bounded local observability**: file logs must rotate and clean up in-process. See [ADR-014](docs/decisions/014-log-retention-and-rotation.md).
- **Embedded prompt assets**: built-in prompts are standalone files embedded at compile time. See [ADR-015](docs/decisions/015-embedded-prompt-assets.md).
- **Layered memory**: working, episodic, semantic, and procedural memory are separate and consolidated explicitly. See [ADR-016](docs/decisions/016-layered-memory-architecture.md).
- **Exploration library**: research artifacts persist locally with source/claim/synthesis provenance; vector/graph stores are Spike-gated. See [ADR-017](docs/decisions/017-exploration-library-storage.md).

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
