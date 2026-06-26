# Talos

[![Release](https://github.com/wjhuang88/talos/actions/workflows/release.yml/badge.svg)](https://github.com/wjhuang88/talos/actions/workflows/release.yml)
[![Latest Release](https://img.shields.io/github/v/release/wjhuang88/talos?include_prereleases)](https://github.com/wjhuang88/talos/releases)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.95%2B-orange)](https://www.rust-lang.org/)

[中文文档](README.zh-CN.md)

Talos is a Rust-native agent runtime for local coding workflows. It provides a terminal UI, provider adapters, sessions, built-in tools, permissions, skills, MCP/RPC integration, and self-evolution hooks while keeping the core small and safety-first.

Talos is pre-1.0 and under active development. The core CLI/TUI, tool pipeline, Git tools, sessions, skills, MCP/RPC server, provider configuration, prompt cache support, and governance workflow are already implemented. See [Project Status](#project-status) for where to track the engineering roadmap.

## Highlights

- **Terminal-first agent experience**: interactive TUI plus print mode for scripts and automation.
- **Rust-native core**: workspace-oriented architecture with small crates and explicit boundaries.
- **Built-in tools**: file, search, edit, shell, symbol, directory tree, diff/stat, and Git operations.
- **Permission-gated writes**: write-capable and execute-capable actions go through the approval pipeline.
- **Provider adapters**: Anthropic Messages, OpenAI Chat, OpenAI Responses, and OpenAI-compatible gateways.
- **Session memory**: SQLite-backed sessions, search, summaries, branches, and export support.
- **Extensibility**: skills, hooks, MCP tools, RPC server, and protocol-focused design.

## Install

### Download A Release

Download the archive for your platform from [GitHub Releases](https://github.com/wjhuang88/talos/releases), then unpack it:

```bash
tar -xzf talos-aarch64-apple-darwin.tar.gz
chmod +x talos
./talos --help
```

Windows releases are published as `.zip` archives. macOS and Linux releases are published as `.tar.gz` archives.

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
```

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
- Network: `http_request` (SSRF-protected, permission-gated), `web_search` (DuckDuckGo + Tavily + SearXNG + Wikipedia)
- Shell escape hatch: `bash`

The default prompt asks models to prefer built-in tools and use shell commands as a fallback when a native tool cannot cover the task.

## Slash Commands

Type `/` in the TUI to access these commands:

| Command | Description |
|---|---|
| `/help` | Show available commands |
| `/quit`, `/exit` | Exit Talos |
| `/status` | Show session info (model, token usage) |
| `/plugins` | List observed tool provenance and MCP server status |
| `/skills` | List available runtime skills (Level 0 metadata) |
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

Use `/skills` in the TUI to list the runtime-discovered skills. Full skill body
activation and reference loading are intentionally gated for a later explicit
activation flow, so large skill content is not dumped into the prompt or history
by default.

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

## Safety Model

- Read-only workspace tools can run without approval.
- File writes, deletes, Git writes, and shell execution are routed through permissions.
- Tool display focuses on key arguments instead of raw JSON where the tool definition provides summary fields.
- Local secrets should live in environment variables or private config files, never in source.
- Talos does not auto-commit changes. Git commits happen only through explicit tool/user action.

## Development

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

Talos is moving from core runtime implementation toward product hardening. For current engineering status, use the project governance docs instead of this README:

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

## License

Talos is licensed under the [Apache License 2.0](LICENSE).
