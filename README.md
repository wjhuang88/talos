# Talos

[![Release](https://github.com/wjhuang88/talos/actions/workflows/release.yml/badge.svg)](https://github.com/wjhuang88/talos/actions/workflows/release.yml)
[![Latest Release](https://img.shields.io/github/v/release/wjhuang88/talos?include_prereleases)](https://github.com/wjhuang88/talos/releases)
[![License](https://img.shields.io/github/license/wjhuang88/talos)](LICENSE)
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

Talos reads configuration from `~/.talos/config.toml`. Prefer environment variables for secrets.

Anthropic example:

```toml
provider = "anthropic"
model = "claude-sonnet-4-20250514"

[providers.anthropic]
api_key_env = "ANTHROPIC_API_KEY"
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

## Built-In Capabilities

Talos ships with built-in tools for common coding-agent work:

- Files and directories: `read`, `write`, `edit`, `delete`, `ls`, `tree`, `glob`
- Search and inspection: `grep`, `diff`, `stat`
- Code intelligence: `find_symbol`, `find_references`, `list_symbols`, `list_imports`
- Git: `git_status`, `git_diff`, `git_log`, `git_show`, `git_branch_list`, `git_add`, `git_commit`, `git_push`, `git_pull`, `git_checkout`
- Shell escape hatch: `bash`

The default prompt asks models to prefer built-in tools and use shell commands as a fallback when a native tool cannot cover the task.

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
