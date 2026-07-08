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
workspace is `v0.3.0`. It is usable for local coding workflows, but still pre-1.0: APIs, command
surfaces, and storage formats may change as the product hardens. This README describes shipped
user-facing behavior; research tracks such as web control expansion beyond the read-only loopback
dashboard, dotagents shared Skills, WASM plugins, and advanced document ingestion are tracked separately under
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

`v0.3.0` is suitable for local developer use where the operator reviews tool actions and keeps
configuration local. It is not yet a remote multi-user service, marketplace runtime, browser
automation surface, or autonomous background daemon.

Currently shipped:

- TUI, inline, and print execution modes.
- Read-only loopback dashboard in TUI mode; startup prints the local URL. Binds to
  `127.0.0.1`; the per-process bearer token is off by default. Set
  `[dashboard] loopback_only = false` to re-enable the token.
- Local provider configuration with masked secrets.
- Built-in coding tools with permission gating.
- Session storage, search, cleanup, maintenance, memory consolidation, and exploration ingestion.
- Runtime Skills from `.talos/skills/`, `~/.talos/skills/`, and inherited parent `.talos/skills/`.
- MCP tools via stdio, SSE, and Streamable HTTP transports.
- Initial Rust embedding facade in the `talos-runtime` crate.

Not shipped yet:

- Stable 1.0 SDK guarantees for the embedded runtime facade.
- `~/.agents/skills/` discovery from the dotagents shared directory.
- Remote web control, browser automation, web approvals, and web write/action routes.
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

### Cargo Install Status

`cargo install talos-cli --bin talos` is the planned crates.io binary-install shape, but it is not
published yet. For now, use the release installers/archives above or build from source with
`cargo build --release -p talos-cli`. A local source checkout can be installed with Cargo for
testing:

```bash
cargo install --path crates/talos-cli --bin talos --locked
```

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
talos config set model claude-sonnet-4-20250514  # set and persist
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

### Validation Plans

Preview the commands Talos expects for a validation profile without running them:

```bash
talos validate plan --profile workspace
talos validate plan --profile i076
talos validate plan --profile governance --json
```

The validation plan surface is read-only. It lists required checks and missing prerequisites, but it
does not execute commands, install dependencies, edit files, push, publish, or tag releases.

Execute an allowlisted validation profile and emit evidence:

```bash
talos validate run --profile governance --json
talos validate run --profile workspace
```

Validation execution accepts only built-in profiles. Each evidence record includes the command,
exit status, stdout/stderr summaries, and the allowlisted-profile permission decision. It does not
accept arbitrary commands, edit repository files, push, publish, or tag releases.

### Permission Preflight

Preview permission scopes for expected long-task tool operations without executing tools or
installing allow rules:

```bash
talos permissions preflight \
  --operation 'bash={"command":"cat Cargo.toml"}' \
  --operation 'bash={"command":"cargo test approval"}'

talos permissions preflight --json \
  --operation 'bash={"command":"rm generated.txt"}'
```

The preflight packet uses the real tool permission profile and shows the reusable `always` scope
that would be offered later. Configured deny rules still win, and high-risk shell commands remain
exact unless the audited bash template policy classifies them as reusable.

### Governance Mutation Preview

Preview a bounded governance owner-doc update before writing it:

```bash
talos governance iteration-record preview \
  --iteration I096 \
  --date 2026-07-04 \
  --record-type validation \
  --record "Recorded validation evidence."
```

Apply the same mutation only after reviewing the preview:

```bash
talos governance iteration-record write \
  --iteration I096 \
  --date 2026-07-04 \
  --record-type validation \
  --record "Recorded validation evidence." \
  --confirm-preview
```

The write path is intentionally narrow: it appends a row to the selected iteration owner doc and
runs governance validation after the write. If validation fails, the file is rolled back.

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

List packaged model metadata without dumping the full catalog by default:

```bash
talos --available-models
talos --available-models --available-models-filter openai/gpt-4
talos --available-models --available-models-all
talos --available-models-browser
```

Model rows are printed as `provider/model` to avoid ambiguity. `--available-models` stays bounded
for scripts and diagnostics; use `--available-models-browser` in a real terminal for a scrollable
catalog view with `j/k`, arrows, `g/G`, `/` search, `Enter` selection/setup, and `q` quit.

### Interactive Commands

In the interactive TUI, type `/` at the start of the composer to open the command menu. Continue
typing to filter commands and use `Up`/`Down` to move the selection. `Enter` runs commands that do
not need inline arguments and fills the composer for commands that need more input. `Tab` always
completes the selected command into the composer. `Backspace` edits the filter and `Esc` closes the
menu without clearing the composer. Use `/help` to list the commands available in the current
session.

Use `/model` to switch among models whose providers are already configured. The picker shows only
usable models, grouped by provider, with the active model pinned in a `Current` group. Use
`/connect` to add or update provider credentials. `/connect` shows provider setup choices from the
packaged offline `models.toml`, asks for an API key, then offers an optional custom endpoint
(`base_url`) for gateway-compatible providers. Standard providers whose catalog metadata supplies a
default endpoint submit after the API key without prompting for a URL; custom providers (or any row
without a built-in endpoint) still require a non-empty `base_url`. A fresh install does not need a
manual catalog initialization step: Talos does not create a runtime `catalog.db` for model
metadata. Model/provider metadata updates are build-time only through `BUILD_MODELS=1`; the legacy
`--import-models` flag is kept as a no-op compatibility notice.

## Built-In Capabilities

Talos ships with built-in tools for common coding-agent work:

- Files and directories: `read`, `write`, `edit`, `delete`, `ls`, `tree`, `glob`
- Search and inspection: `grep`, `diff`, `stat`
- Code intelligence: `find_symbol`, `find_references`, `list_symbols`, `list_imports`
- Git: `git_status`, `git_diff`, `git_log`, `git_show`, `git_branch_list`, `git_add`, `git_commit`, `git_push`, `git_pull`, `git_checkout`
- Network: `fetch_url` (bounded URL context — public pages, HTML extraction, JSON), `http_request` (advanced HTTP/API inspection — custom methods/headers/bodies, disclosed on demand via continuation), `save_url` (download URL to local file — dual network+write permission), `web_search` (DuckDuckGo + Tavily + SearXNG + Wikipedia)
- Document extraction: `document_extract` (read-only bounded text extraction from local text/HTML/JSON/CSV/Markdown/XML files)
- Process execution: `exec` (argv-only single process, no shell parsing), `bash` (shell escape hatch)
- Session planning: `todo_create`, `todo_update_status`, `todo_update`, `todo_delete`,
  `todo_add_dependency`, `todo_remove_dependency`, `todo_query` (session-scoped todo state)

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
| `/plugins` | Plugin packages (not yet available — use /mcp for MCP status) |
| `/mcp` | Show MCP server status and observed tool provenance |
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
| `/model` | Open the model picker to browse and switch models at runtime; models are grouped by provider with the current model in a top group, and typing filters by group |
| `/connect` | Open the provider picker to connect a new provider (credential and optional custom endpoint/`base_url`), or a specific `/connect <provider>` to connect it directly |
| `/todo`, `/todo list`, `/todo show <id>`, `/todo stats`, `/todo export [json|markdown]` | View or export active-session todos (read-only) |
| `/todo delete <id> --confirm` | Delete a session todo item by short-ID or full UUID; requires `--confirm` |
| `/hooks` | Show configured hook diagnostics (declared paths, presence, validation status) without executing hooks |
| `/agile [status]` | Show read-only governance status: board disposition, open iterations, manifest, and Rust validation findings |

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

Configure MCP servers in `~/.talos/config.toml`. Talos supports four transport types: `stdio`
(local process), `sse` (legacy HTTP/SSE), `streamable_http` (Streamable HTTP), and `http` (alias
for `streamable_http`).

Local stdio:

```toml
[[mcp.servers]]
name = "filesystem"
transport = "stdio"
command = "/path/to/mcp-server"
args = ["/path/to/workspace"]
env = {}
```

Streamable HTTP (recommended for remote servers):

```toml
[[mcp.servers]]
name = "remote-streamable"
transport = "streamable_http"
url = "https://mcp.example.com/mcp"
auth_token_env = "REMOTE_MCP_TOKEN"   # sends Authorization: Bearer $REMOTE_MCP_TOKEN
```

Legacy SSE:

```toml
[[mcp.servers]]
name = "remote-sse"
transport = "sse"
url = "https://mcp.example.com/sse"
# sse_post_url is optional; auto-discovered from the endpoint event when omitted
auth_token_env = "REMOTE_MCP_TOKEN"
```

Auth examples prefer `auth_token_env` or `authorization_env` over inline secrets. `auth_token_env`
sends `Authorization: Bearer <token>`; `authorization_env` sends the full `Authorization` header
value. The `headers` field accepts non-secret HTTP headers.

Talos starts configured servers and discovers their tools before the first model turn in TUI,
print, inline, interactive, and RPC modes. Tool names use the
`mcp:<server>:<tool>` form. Read-only annotations are honored; other MCP tools use the normal
approval path and are denied when interactive approval is unavailable. Startup failures are
reported without aborting the session, and each MCP request has a bounded timeout. Per-server
remote startup failures do not affect other servers. Use `/mcp`
in the TUI to inspect the startup connection snapshot and observed tool provenance.

The MCP tool set is fixed for the lifetime of a session so the model-visible tool definitions and
prompt cache prefix remain stable. Restart the session after changing MCP configuration.
Streamable HTTP resumable sessions and long-lived server-to-client notification channels are not
yet supported.

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

`talos-runtime` is not yet published as an SDK crate in the current release gate. It remains
manifest-ready but blocked by dependency closure; see
[RUNTIME-SDK-CONTRACT](docs/reference/RUNTIME-SDK-CONTRACT.md) and the
[publish gate packet](docs/reference/PUBLISH-GATE-PACKET-2026-07-02.md).

## Safety Model

- Read-only workspace tools can run without approval.
- File writes, deletes, Git writes, and shell execution are routed through permissions.
- Tool display focuses on key arguments instead of raw JSON where the tool definition provides summary fields.
- Local secrets should live in environment variables or private config files, never in source.
- Talos does not auto-commit changes. Git commits happen only through explicit tool/user action.

## Troubleshooting And Bug Reports

### Reporting Issues

File bugs and feature requests on [GitHub Issues](https://github.com/wjhuang88/talos/issues).

Include the following diagnostic information in your bug report:

```bash
talos --version                    # version and build info
talos config list                  # redacted config (secrets masked as ***)
talos storage status               # local data directory sizes and session counts
talos --governance-status          # governance manifest and board disposition
```

All diagnostic commands mask secrets. `config list` replaces `api_key` values with `***` while
preserving `api_key_env` variable names so you can share output safely.

### Debug Logging

Talos writes logs to `~/.talos/logs/talos.log`. Check the log directory size with:

```bash
talos storage status
```

Increase log verbosity by setting the `RUST_LOG` environment variable:

```bash
RUST_LOG=talos=debug talos
```

### Common Issues

- **Provider connection fails**: verify `api_key` or `api_key_env` is set. Use `talos config list`
  to confirm the credential source. Standard providers (Anthropic, OpenAI, DeepSeek, etc.) have
  built-in endpoints; custom providers require an explicit `base_url`.
- **Permission prompts repeat**: use `always` scope when approving repeated low-risk operations.
  Deny rules always take precedence over `always` rules.
- **Session not resuming**: ensure the session UUID exists with `talos storage status`. Use
  `talos --continue` to resume the most recent workspace session.
- **Model picker is empty**: unauthenticated providers are omitted from `/model`. Use `/connect`
  to set up credentials first.

### Known Limitations

- Pre-1.0: APIs, command surfaces, and storage formats may change.
- No remote multi-user service, marketplace, or browser automation.
- No WASM plugin runtime or PDF/Office document extraction.
- Self-bootstrap qualification (REL-002) is not yet met; `v1.0` is not claimable.

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

The post-v0.2.0 hardening notes that fed the pre-0.3 release line are collected in
[RELEASE-NOTES-DRAFT-2026-07-02](docs/reference/RELEASE-NOTES-DRAFT-2026-07-02.md). GitHub Releases
is the source of truth for the published `v0.3.0` release announcement and downloads.

## Project Status

Talos is moving from core runtime implementation toward product hardening and differentiated
developer experience. The next research priorities are:

- `AGENT-002-B`: dotagents `~/.agents/skills/` compatibility.
- `TOOL-004`: search engine direction before broader tool-set redesign.
- `TOOL-007`: holistic tool-set audit, including WEBFETCH Phase 2+ planning.
- `WEB-001`: local loopback web surface expansion beyond the read-only dashboard MVP.

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
