# Talos

[![Release](https://github.com/wjhuang88/talos/actions/workflows/release.yml/badge.svg)](https://github.com/wjhuang88/talos/actions/workflows/release.yml)
[![Latest Release](https://img.shields.io/github/v/release/wjhuang88/talos?include_prereleases)](https://github.com/wjhuang88/talos/releases)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.95%2B-orange)](https://www.rust-lang.org/)

[中文文档](README.zh-CN.md)

Talos is a Rust-native local coding agent for developers who want a safety-first runtime they can
inspect, extend, and operate from their own machine. It combines a terminal UI, provider adapters,
session history, built-in coding tools, explicit permissions, runtime Skills, MCP/RPC integration,
and project-governance support while keeping the default core local and auditable.

Talos has published its first stable pre-1.0 release line. The current release version in this
workspace is `v0.2.0`. It is usable for local coding workflows, but still pre-1.0: APIs, command
surfaces, and storage formats may change as the product hardens. This README describes shipped
user-facing behavior; research tracks such as the embedded web control surface, dotagents shared
Skills, WASM plugins, and advanced document ingestion are tracked separately under
[Project Status](#project-status).

## Highlights

- **Local-first coding agent**: interactive TUI, inline mode, and print mode for scripts and smoke tests.
- **Safety-first tool runtime**: file writes, deletes, Git writes, shell execution, network actions, and MCP tools route through explicit permission boundaries.
- **Rust-native core**: workspace-oriented crates with minimal runtime assumptions and no Node/Python runtime dependency.
- **Embeddable Rust runtime**: an initial `talos-runtime` facade lets Rust projects construct a safe in-process agent runtime without depending on Talos CLI/TUI crates.
- **Auditable internals**: oversized memory, config, CLI/TUI, and agent compaction modules are split into focused Rust modules with behavior-preserving gates.
- **Built-in coding tools**: file, search, edit, shell, symbol, directory tree, diff/stat, Git, HTTP request, and web search operations.
- **Durable sessions and memory**: SQLite-backed session history, search, branch/fork support, export, semantic memory consolidation, and retention previews.
- **Progressive context**: runtime Skill discovery plus explicit Skill body/reference activation without dumping hidden content into visible history.
- **Extensible surface**: MCP tools, hooks, JSON-RPC, and governance-aware project status are implemented; plugin/WASM and browser control surfaces remain research tracks.

## Current Release Boundary

`v0.2.0` is suitable for local developer use where the operator reviews tool actions and keeps
configuration local. It is not yet a remote multi-user service, marketplace runtime, browser
dashboard, or autonomous background daemon.

Currently shipped:

- TUI, inline, and print execution modes.
- Local provider configuration with masked secrets.
- Built-in coding tools with permission gating.
- Session storage, search, cleanup, maintenance, memory consolidation, and exploration ingestion.
- Runtime Skills from `.talos/skills/`, `~/.talos/skills/`, and inherited parent `.talos/skills/`.
- MCP stdio tools and JSON-RPC infrastructure.
- Initial Rust embedding facade in the `talos-runtime` crate.

Not shipped yet:

- Stable 1.0 SDK guarantees for the embedded runtime facade.
- `~/.agents/skills/` discovery from the dotagents shared directory.
- Embedded browser/web control surface.
- WASM plugin runtime and plugin marketplace.
- PDF/Office document extraction beyond the current web/fetch foundations.
- Remote or P2P session control.

## Install

### Download A Release

Install the latest release on macOS or Linux:

```bash
curl -fsSL https://raw.githubusercontent.com/wjhuang88/talos/main/install/install.sh | sh
```

Install the latest Windows x86_64 release from PowerShell:

```powershell
iex (irm https://raw.githubusercontent.com/wjhuang88/talos/main/install/install.ps1)
```

Installers live under `install/` because they are user-facing release entrypoints. Development and
governance scripts live under `scripts/`; the old `scripts/install.*` paths are intentionally not
kept after the pre-1.0 installer layout cleanup.

Or download the archive for your platform from
[GitHub Releases](https://github.com/wjhuang88/talos/releases), then unpack it:

```bash
tar -xzf talos-aarch64-darwin.tar.gz
chmod +x talos
./talos --help
```

Published archive names:

| Platform | Archive |
|---|---|
| Linux x86_64 | `talos-x86_64-linux.tar.gz` |
| Linux ARM64 | `talos-aarch64-linux.tar.gz` |
| macOS Intel | `talos-x86_64-darwin.tar.gz` |
| macOS Apple Silicon | `talos-aarch64-darwin.tar.gz` |
| Windows x86_64 | `talos-x86_64-windows.zip` |

Windows ARM64 artifacts are not published yet.

### First-Run Setup

When you start Talos without a model configured, the TUI opens with a model
picker instead of failing. Choose a model to get started. If the provider
needs credentials, Talos shows instructions for setting the API key.

To skip the wizard in CI or non-interactive environments:

```bash
talos --no-init -p "summarize this repo"
```

### Configuration Management

View and edit configuration without hand-editing TOML:

```bash
talos --config-list                          # print all settings (secrets masked)
talos --config-get model                     # get a single value
talos --config-set model=claude-sonnet-4-20250514  # set and persist
talos --config-set providers.anthropic.api_key_env=ANTHROPIC_API_KEY

# Subcommand form (equivalent to the flags above):
talos config list                                # print all settings (secrets masked)
talos config get model                           # get a single value
talos config set model=claude-sonnet-4-20250514  # set and persist
```

## Development

### Build From Source

Requirements:

- Rust 1.95 or newer
- Cargo

```bash
cargo build --release -p talos-cli
./target/release/talos --help
```

To build all release artifacts locally:

```bash
./build.sh
```

The multi-platform build writes archives and checksums to `dist/`.

## Configure A Provider

Talos reads configuration from `~/.talos/config.toml`. Secrets can be stored
inline (`api_key`) or via environment variables (`api_key_env`). Inline keys
are persisted in the config file (chmod 600 recommended) and masked in all
display output (`talos config list`, `talos config get`, debug logs). See
[ADR-023](docs/decisions/023-inline-api-key-boundary.md) for the full boundary.

Anthropic example (env-var mode):

```toml
provider = "anthropic"
model = "claude-sonnet-4-20250514"

[providers.anthropic]
api_key_env = "ANTHROPIC_API_KEY"
```

Anthropic example (inline key):

```toml
provider = "anthropic"
model = "claude-sonnet-4-20250514"

[providers.anthropic]
api_key = "sk-ant-..."
```

OpenAI-compatible gateway example:

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

Set the matching environment variable before starting Talos:

```bash
export ANTHROPIC_API_KEY="..."
```

## Run Talos

Start the interactive TUI in the current directory:

```bash
talos "inspect this repository"
```

Run a one-shot prompt in print mode:

```bash
talos -p "summarize this repository"
```

Choose a workspace explicitly:

```bash
talos --workspace /path/to/project "analyze the current architecture"
```

Use the mock provider for deterministic local smoke tests:

```bash
talos -p --mock "/mock-request summarize this repository"
```

### Manage Local Storage

Check local storage usage (read-only):

```bash
talos storage status
```

Preview sessions that would be cleaned up (dry-run, no deletion):

```bash
talos storage cleanup --max-sessions 20
talos storage cleanup --max-age-days 30 --workspace /path/to/project
```

Delete old sessions with explicit apply and active-session protection:

```bash
talos storage cleanup --apply --max-age-days 90 --protect-session <active-uuid>
```

Run SQLite maintenance:

```bash
talos storage maintenance --checkpoint --vacuum --reconcile
```

### Memory

Consolidate session episodes into semantic memory:

```bash
talos memory consolidate --session <session-uuid>
talos memory consolidate                  # latest workspace session
```

Check memory store status (counts and sizes, no content exposed):

```bash
talos memory status
```

Preview memory retention candidates (dry-run, no deletion):

```bash
talos memory retention --min-confidence 0.5
talos memory retention --max-age-days 90 --unreinforced-only
```

### Exploration Library

Ingest local files into a searchable research library:

```bash
talos explore ingest --file README.md --title "Project README"
```

Search ingested sources:

```bash
talos explore search --query "session management" --limit 10
```

### Interactive Commands

In the interactive TUI, type `/` at the start of the composer to open the command menu. Continue
typing to filter commands, use `Up`/`Down` to move the selection, and press `Enter` or `Tab` to
complete it. `Backspace` edits the filter and `Esc` closes the menu without clearing the composer.
Use `/help` to list the commands available in the current session.

## Built-In Capabilities

Talos ships with built-in tools for common coding-agent work:

- Files and directories: `read`, `write`, `edit`, `delete`, `ls`, `tree`, `glob`
- Search and inspection: `grep`, `diff`, `stat`
- Code intelligence: `find_symbol`, `find_references`, `list_symbols`, `list_imports`
- Git: `git_status`, `git_diff`, `git_log`, `git_show`, `git_branch_list`, `git_add`, `git_commit`, `git_push`, `git_pull`, `git_checkout`
- Network: `fetch_url` (bounded URL context), `http_request` (advanced HTTP/API inspection, disclosed on demand), `web_search` (DuckDuckGo + Tavily + SearXNG + Wikipedia)
- Document extraction: `document_extract` (read-only bounded text extraction from local text/HTML/JSON/CSV/Markdown/XML files)
- Shell escape hatch: `bash`

The default prompt asks models to prefer built-in tools and use shell commands as a fallback when a
native tool cannot cover the task. It also emphasizes accuracy over approval: do not flatter,
fabricate citations, or hide uncertainty when evidence is missing.

## Slash Commands

Type `/` in the TUI to access these commands. The Skill commands are also available in inline
mode.

| Command | Description |
|---|---|
| `/help` | Show available commands |
| `/quit`, `/exit` | Exit Talos |
| `/status` | Show session info (model, token usage) |
| `/plugins` | List observed tool provenance and MCP server status |
| `/skills` | List available runtime skills and active state |
| `/skills activate <name>` | Activate one Skill body for subsequent provider requests |
| `/skills reference <path>` | Load a bounded reference file for the active Skill |
| `/copy last` | Copy the last assistant message to clipboard |
| `/copy all` | Copy the full transcript to clipboard |
| `/export <path>` | Export transcript to a file (permission-gated) |
| `/new` | Start a fresh session (preserves old session) |
| `/resume` | List resumable workspace sessions; `/resume <N>` selects by number |
| `/fork` | Fork the active session (clones history into a child session) |
| `/delete` | Open the session picker (excluding the active session); choose a row to remove it |
| `/model` | Open the model picker to browse and switch models at runtime |

## Skills

Talos discovers `SKILL.md` files at session startup and injects Level 0 metadata
(skill name, description, and triggers) into the system prompt before the first
model turn.

Skill search paths, in priority order:

- `.talos/skills/` in the active workspace
- `~/.talos/skills/`
- parent `.talos/skills/` directories up to the Git root
- `~/.agents/skills/` (shared, opt-in via `[skills] discover_shared = true` in config)

Use `/skills` in the TUI or inline mode to list runtime-discovered skills. Use
`/skills activate <name>` to explicitly load one Skill body into provider
context for subsequent turns. After a Skill is active, use
`/skills reference <relative-path>` to load a bounded reference file from that
Skill directory.

Activated Skill bodies and references are added to provider context only. Talos
does not print the full content into scrollback command output or transcript
history, and reference paths must stay inside the active Skill directory.

## MCP Tools

Configure local stdio MCP servers in `~/.talos/config.toml`:

```toml
[[mcp.servers]]
name = "filesystem"
transport = "stdio"
command = "/path/to/mcp-server"
args = ["/path/to/workspace"]
env = {}
```

Talos starts configured servers and discovers their tools before the first model turn in TUI,
print, inline, interactive, and RPC modes. Tool names use the
`mcp:<server>:<tool>` form. Read-only annotations are honored; other MCP tools use the normal
approval path and are denied when interactive approval is unavailable. Startup failures are
reported without aborting the session, and each MCP request has a bounded timeout. Use `/plugins`
in the TUI to inspect the startup connection snapshot and observed tool provenance.

The MCP tool set is fixed for the lifetime of a session so the model-visible tool definitions and
prompt cache prefix remain stable. Restart the session after changing MCP configuration. Only
local stdio transport is currently supported.

## Embedding Talos In Rust

Rust applications can depend on the `talos-runtime` crate to embed the core agent loop without
linking Talos CLI or TUI crates. The initial pre-1.0 facade exposes `RuntimeBuilder` and
`RuntimeHandle` for provider/tool injection, typed event streaming, interruption, shutdown, and
explicit request previews. Embedders can also provide approval handlers and customize or append the
runtime system prompt through `RuntimeBuilder`.

Registered tools are permission-wrapped by default. In headless embedding, unresolved `Ask`
decisions are denied unless the embedder supplies narrower allow-list rules.

This is not a stable 1.0 SDK guarantee yet. The public embedding surface is `talos-runtime`
plus the protocol and trait types it re-exports from `talos-core`; lower-level `talos-agent`
constructors remain implementation surface unless documented otherwise.

## Safety Model

- Read-only workspace tools can run without approval.
- File writes, deletes, Git writes, and shell execution are routed through permissions.
- Tool display focuses on key arguments instead of raw JSON where the tool definition provides summary fields.
- Local secrets should live in environment variables or private config files, never in source.
- Talos does not auto-commit changes. Git commits happen only through explicit tool/user action.

## Contributing And Local Checks

Common checks:

```bash
cargo check --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --all
```

Release tags drive the GitHub release workflow:

- Stable release: `v0.1.0`
- Pre-release: `v0.1.0-alpha.1`, `v0.1.0-beta.1`, `v0.1.0-rc.1`, `v0.1.0-pre.1`, `v0.1.0-dev.1`

The release workflow builds Linux, macOS, and Windows artifacts from a macOS runner.

## Project Status

Talos is moving from core runtime implementation toward product hardening and differentiated
developer experience. The next research priorities are:

- `AGENT-002-B`: dotagents `~/.agents/skills/` compatibility.
- `TOOL-004`: search engine direction before broader tool-set redesign.
- `TOOL-007`: holistic tool-set audit, including WEBFETCH Phase 2+ planning.
- `WEB-001`: embedded local web control surface as a product differentiation track.

For current engineering status, use the project governance docs instead of this README:

- [Board](docs/BOARD.md): active, review, and next work
- [Implementation Roadmap](docs/roadmap/IMPLEMENTATION-ROADMAP.md): planned phases
- [Product Backlog](docs/backlog/PRODUCT-BACKLOG.md): story inventory
- [Iterations](docs/iterations/): iteration records and completion evidence

## Documentation

| Topic | Document |
| --- | --- |
| Architecture | [docs/reference/ARCHITECTURE.md](docs/reference/ARCHITECTURE.md) |
| Reference projects | [docs/reference/REFERENCE-PROJECTS.md](docs/reference/REFERENCE-PROJECTS.md) |
| Decisions | [docs/decisions/](docs/decisions/) |
| Local development | [docs/sop/LOCAL-DEV.md](docs/sop/LOCAL-DEV.md) |
| Testing | [docs/sop/TESTING.md](docs/sop/TESTING.md) |
| Git workflow | [docs/sop/GIT-WORKFLOW.md](docs/sop/GIT-WORKFLOW.md) |
| Public product site | [https://talos.hwj.zone](https://talos.hwj.zone) &mdash; static GitHub Pages site (source under [`site/`](site/)) |

## License

Talos is licensed under the [Apache License 2.0](LICENSE).
