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
workspace is `v0.9.0`. It is usable for local coding workflows, but still pre-1.0: APIs, command
surfaces, and storage formats may change as the product hardens. This README describes shipped
user-facing behavior; research tracks such as web control expansion beyond the read-only loopback
dashboard, broader dotagents compatibility, plugin carriers, and advanced document ingestion are tracked separately under
[Project Status](#project-status).

## Highlights

- **Local-first coding agent**: interactive TUI, inline mode, and print mode for scripts and smoke tests.
- **Configurable providers and models**: use the parameterless `/connect` and `/model` pickers to add an OpenAI- or Anthropic-compatible provider, discover its models, and switch the live session without command-string parsing.
- **Truthful provider progress**: model requests show `Connecting...` initially and
  `Reconnecting... (attempt n/m)` during provider-reported retry dispatch or backoff; the values
  come from typed provider events and do not change retry policy or infer progress from timers.
- **Explicit vision attachments**: attach PNG, JPEG, GIF, or WebP images with `/attach` (or print-mode `--attach`) only after capability, permission, format, size, pixel, and replacement checks; image paths are never auto-read from ordinary text.
- **Safety-first tool runtime**: file writes, deletes, Git writes, shell execution, network actions, and MCP tools route through explicit permission boundaries.
- **Rust-native core**: workspace-oriented crates with minimal runtime assumptions and no Node/Python runtime dependency.
- **Embeddable Rust runtime**: an initial `talos-runtime` facade lets Rust projects construct a safe in-process agent runtime without depending on Talos CLI/TUI crates.
- **Auditable internals**: oversized memory, config, CLI/TUI, and agent compaction modules are split into focused Rust modules with behavior-preserving gates.
- **One ordered runtime flow**: TUI, inline, print, embedded, and RPC modes share the session actor's sequenced turn lifecycle; live text, thinking, and tool output preserve FIFO order.
- **Built-in coding tools**: file, search, edit, shell, symbol, directory tree, diff/stat, Git, HTTP request, and web search operations.
- **Durable sessions and memory**: SQLite-backed session history, search, branch/fork support, export, semantic memory consolidation, and retention previews.
- **Progressive context**: runtime Skill discovery plus explicit Skill body/reference activation without dumping hidden content into visible history.
- **Extensible surface**: MCP tools, hooks, JSON-RPC, governance-aware project status, and explicit local read-only WASM packages are implemented; remote plugin distribution and browser control remain bounded separately.

## Current Release Boundary

`v0.9.0` is suitable for local developer use where the operator reviews tool actions and keeps
configuration local. It is not yet a remote multi-user service, marketplace runtime, browser
automation surface, or autonomous background daemon.

Currently shipped:

- TUI, inline, and print execution modes.
- Read-only loopback dashboard in TUI mode; successful startup shows the complete local URL as a
  copyable plain-text entry in the display-only TUI Logo region. It occupies one row at ordinary
  widths and wraps without truncation at narrow widths. Startup failures remain transient error
  notices. In a browser, `/`, `/status`, `/history`, `/governance`, `/config`, and `/extensions`
  share a light, compact Nord-derived read-only shell with keyboard-visible navigation, responsive
  layouts, and deterministic empty states. Explicit `Accept: text/html` selects the rendered
  representation for all five data routes; requests without explicit HTML acceptance keep the
  existing JSON/plain-text API. `/activity` adds a GET/read-only live workspace over bounded SSE:
  it shows the current Session/model/Turn, allowlisted semantic activity, authoritative usage, and
  a secondary filtered view of successfully written, re-redacted runtime logs. Prompt/message text,
  thinking/reasoning, approval arguments, raw tool inputs/results, credentials, and unknown event
  variants are not projected. Reconnect uses process-local IDs and bounded replay/reset; it is not
  durable Session history. The Dashboard UI is English-only even though the user
  documentation is bilingual. Dynamic content is HTML-escaped and redacted before presentation,
  and configuration remains masked. The server binds to `127.0.0.1` and registers no write/action
  route; the per-process bearer token is off by default. Set `[dashboard] loopback_only = false` to
  require the token; the Logo entry then shows the token-free base URL plus
  `authentication required` and never displays or logs the credential. The current TUI emits no
  terminal hyperlink escape sequence. The default loopback mode is the supported browser-live path;
  auth-required mode remains explicit-header-only and adds no browser token-delivery mechanism.
- Local provider configuration with masked secrets.
- Parameterless provider/model pickers, custom compatible-provider registration, bounded model discovery, and structured session switching.
- Explicit local-image attachments for catalog-confirmed vision-capable models, with exact-path authorization and safe history summaries. Anthropic-compatible wire behavior is covered by fixtures; live-provider validation depends on operator credentials.
- Built-in coding tools with permission gating.
- Session storage, search, cleanup, maintenance, memory consolidation, and exploration ingestion.
- Runtime Skills from `.talos/skills/`, `~/.talos/skills/`, inherited parent `.talos/skills/`, and shared `~/.agents/skills/` (enabled by default; set `[skills] discover_shared = false` to disable). Symbolic-link traversal remains disabled by default and is governed separately by `SkillDiscoveryPolicy`.
- MCP tools via stdio, SSE, and Streamable HTTP transports.
- Explicit local read-only WASM packages via repeatable `--plugin DIRECTORY`; packages remain
  confined, permission-wrapped, provenance-bearing, and absent unless selected by the operator.
- Initial Rust embedding facade in the `talos-runtime` crate.

Not shipped yet:

- Stable 1.0 SDK guarantees for the embedded runtime facade.
- Remote web control, browser automation, web approvals, and web write/action routes.
- Plugin marketplace, remote install, automatic discovery, host calls, and write-capable plugins.
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

Install or roll back to a specific release by using its complete Git tag:

```bash
curl -fsSL https://raw.githubusercontent.com/wjhuang88/talos/main/install/install.sh \
  | TALOS_VERSION=v0.9.0 sh
~/.talos/bin/talos --version
```

```powershell
$env:TALOS_VERSION = 'v0.9.0'
iex (irm https://raw.githubusercontent.com/wjhuang88/talos/main/install/install.ps1)
& "$env:USERPROFILE\.talos\bin\talos.exe" --version
Remove-Item Env:TALOS_VERSION
```

Replace `v0.9.0` with a tag listed on
[GitHub Releases](https://github.com/wjhuang88/talos/releases). The installer overwrites the Talos
binary in the selected install directory; it does not roll back configuration or session data.
Because Talos is pre-1.0, back up `~/.talos` before running an older binary, or test it in an
isolated directory by also setting `TALOS_INSTALL_DIR`. To return to the newest release, clear
`TALOS_VERSION` and run the normal latest-install command again. A missing tag or missing platform
archive fails the installation instead of silently falling back to another version.

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
talos config set auto.enabled false               # disable bounded auto-assistance attempts

# Remove a provider entry or clear a credential (--confirm required):
talos config unset providers.my-gateway --confirm             # remove entire custom provider
talos config unset providers.anthropic --confirm              # clear builtin provider credentials
talos config unset providers.my-gateway.api_key --confirm     # clear only the API key
```

`config unset` removes a `[providers.<name>]` entry or clears a single `api_key`
field. Without `--confirm`, the command refuses to modify config and the file is
left byte-identical. Removing a custom provider deletes it entirely; removing a
builtin-backed provider (e.g. `anthropic`) clears the user-saved credential and
overrides but the provider remains available via the builtin catalog. Removing
the active provider does not crash — the next session start or `/model` re-opens
the model picker. Environment variables you set yourself are never touched.

The `[auto] enabled` setting defaults to `true` as an attempted bounded-assistance mode, never an
unconditional permission. `/auto`, `/auto on`, and `/auto off` inspect or override the active
session without writing configuration or transcript state.

When auto assistance evaluates a foreground shell request, the configured model receives a bounded
copy of the exact command as untrusted data plus structural risk facts, the current user instruction,
canonical workspace/cwd bindings, environment variable names and an opaque environment digest. A
configured or explicit `Ask` rule bypasses the assessor. A valid high-confidence `read_only` result
may admit that one invocation only. Mutating, network, privileged, ambiguous, secret-bearing,
malformed, timed-out, or stale requests remain human-required or denied by the authoritative
permission and sandbox gates. The assessor has no tools, never sees raw environment values or
secret-like assignments, and cannot create a permanent grant.

Auto assistance is available only when the active surface supplies an interactive approval
resolver (Goal and interactive CLI/TUI). Headless CLI, embedded Runtime and standalone MCP keep
their existing fail-closed behavior for unresolved `Ask` decisions; enabling `auto` in config does
not silently grant those surfaces model authority. Disabling auto always returns an interactive
surface to human approval and a headless surface to denial.

## Development

### Build From Source

Requirements:

- Rust 1.95 or newer
- Cargo

```bash
cargo build --release -p talos-cli
./target/release/talos --help
```

### Embedded Durable Runtime Session

An embedding host can keep Talos model history in a host-selected directory without using
`~/.talos`. The host persists only its stable logical ID; Talos maps it to a UUID-named TLOG and
keeps the binding/index under the supplied directory.

```rust
use std::sync::Arc;
use talos_runtime::RuntimeBuilder;
use talos_session::SessionManager;

let manager = SessionManager::with_dir(app_data_dir.join("messages"));
let session = manager.create_or_open_session("assistant:account-uuid")?;
let runtime = RuntimeBuilder::new()
    .provider(provider)
    .durable_session(session)
    .build()?;
```

Durable runtime turns are finalized atomically with a Success, Error, or Cancelled outcome.
Rebuilding with the same external ID restores model context automatically. When an error or
interruption follows a completed tool exchange, Talos retains the latest closed, display-safe
prefix exactly once and records that the turn did not complete normally; cancellation before any
persistable fact records only hidden outcome evidence. The transcript excludes unfinished
assistant fragments, raw provider responses, private tool data, and authentication material; host
UI state, approval audits, artifacts, and provider conversation IDs remain host-owned.

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

If the provider reaches its output-token limit, Talos preserves the partial answer and prints an explicit truncation warning. Unsupported terminal reasons, transport failures, and streams that close without a protocol terminal signal fail instead of being reported as normal completion. See [`docs/reference/PROVIDER-TERMINAL-OUTCOMES.md`](docs/reference/PROVIDER-TERMINAL-OUTCOMES.md).

In the interactive TUI, a model request starts with `Connecting...`. Built-in OpenAI-compatible
and Anthropic providers replace it with `Reconnecting... (attempt n/m)` during an actual bounded
retry, using the provider's configured retry ordinal and ceiling. The activity is transient and
clears on content, failure, cancellation, or completion; providers that do not implement typed
progress retain the compatible static `Connecting...` display. A fast retry still leaves the
initial `Connecting...` phase visible for one activity frame before the truthful reconnect state.

Attach one or more local images to a print-mode prompt (requires a vision-capable model):

```bash
talos -p "describe these screenshots" --attach shot-1.png --attach shot-2.png
# shorthand: -a
talos -p "describe this" -a diagram.png
```

`--attach` enforces the same fail-closed capability gate, MIME/byte/pixel
limits, decoder panic containment, and TOCTOU guard as the TUI `/attach`
flow. `--attach` is refused before any file read when the active model's
catalog metadata does not confirm `image_input = true`. `--attach` is
print-mode-only; TUI and inline modes reject it with a pointer to the
`/attach` slash command.

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
installing permission authority:

```bash
talos permissions preflight \
  --operation 'bash={"command":"cat Cargo.toml"}' \
  --operation 'bash={"command":"cargo test approval"}'

talos permissions preflight --json \
  --operation 'bash={"command":"rm generated.txt"}'
```

The preflight packet uses the real tool permission profile and the same compiler used by approval
surfaces. It shows the reusable Session scope that would be offered later: file writes remain exact
paths, safe shell operations may reuse an audited template descriptor, and high-risk shell commands
remain exact. Preflight neither executes a tool nor installs a grant. Configured deny rules always
win.

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

The composer wraps long input to the terminal width. `Shift+Enter` inserts a newline while bare
`Enter` submits. Talos probes for the progressive keyboard protocol before enabling complete
modified-key reporting; `Ctrl+J` is the portable newline fallback for terminals or multiplexers
that do not support the protocol.

Press `Esc` during an active turn to request cancellation. `Ctrl+C` clears the
composer locally; with an empty idle composer, press `Ctrl+C` twice to exit
Talos. A graceful exit restores terminal input and display modes. If Talos is unavoidably hard
killed and the shell remains in a raw or mouse-reporting state, run `reset` or open a fresh terminal.

If you type a message while the model is still processing, it queues automatically and is sent
FIFO after the current turn completes. The TUI shows a compact preview of queued messages
above the composer (up to 6 lines; longer queues show a `+N more` summary). The preview clears
when the queue empties. If `Esc` cancels the active turn, already-accepted queued input becomes the
next turn in the same session automatically. Model/provider switching remains blocked until that
queued input is accepted by the session and the queue is drained.
An idle first submission dispatches directly and does not show the queued-message hint.

Use `/model` to switch among models whose providers are already configured. The picker
uses **three-level navigation**: Level 1 lists recent models (when available, persisted
at `~/.talos/recent_models.json`) and providers; selecting a provider enters Level 2,
which lists that provider's models; selecting a model with declared invocation variants
(for example `high-reasoning` or `low-reasoning` on reasoning-capable models) enters
Level 3 to pick a variant, while selecting a variant-less model switches immediately.
`Esc` closes the picker at any level. Both `/model` and `/connect` are **strict
no-argument commands**: typing `/model gpt-4o` or `/connect openai` does not switch a
model or start credential setup — it shows a brief correction and opens the relevant
picker so you can search and select from the panel. Use `/connect` to add or update
provider credentials. `/connect` shows provider setup choices from the
packaged offline `models.toml`, asks for an API key, then offers an optional custom endpoint
(`base_url`) for gateway-compatible providers. Standard providers whose catalog metadata supplies a
default endpoint submit after the API key without prompting for a URL; custom providers (or any row
without a built-in endpoint) still require a non-empty `base_url`. The `/connect` picker also has an
**Add custom provider** entry that opens a cancel-safe wizard: enter a provider name (1–64 char
slug), choose a protocol (`openai-chat` or `anthropic-messages`), enter an HTTPS base URL (HTTP only
for loopback), enter an API key (masked), and confirm. The wizard saves all fields as one atomic
config update — cancellation or any validation error leaves your existing config unchanged. A fresh
install does not need a manual catalog initialization step: Talos does not create a runtime
`catalog.db` for model metadata. Model/provider metadata updates are build-time only through
`BUILD_MODELS=1`; the legacy `--import-models` flag is kept as a no-op compatibility notice.

## Built-In Capabilities

Talos ships with built-in tools for common coding-agent work:

- Files and directories: `read`, `write`, `edit`, `delete`, `ls`, `tree`, `glob`
- Search and inspection: `grep`, `diff`, `stat`
- Code intelligence: `find_symbol`, `find_references`, `list_symbols`, `list_imports`
- Git: `git_status`, `git_diff`, `git_log`, `git_show`, `git_branch_list`, `git_add`, `git_commit`, `git_push`, `git_pull`, `git_checkout`
- Network: `fetch_url` (bounded URL context — public pages, HTML extraction, JSON), `http_request` (advanced HTTP/API inspection — custom methods/headers/bodies, disclosed on demand via continuation), `save_url` (download URL to local file — dual network+write permission), `web_search` (DuckDuckGo + Tavily + SearXNG + Wikipedia)
- Document extraction: `document_extract` (read-only bounded text extraction from local text/HTML/JSON/CSV/Markdown/XML files)
- Image reading: `read_image` (model-invoked image read for vision-capable models; validates PNG/JPEG/GIF/WebP through shared ingestion; exact-path authorization; returns a safe summary and a one-shot provider continuation artifact per ADR-051; only presented when the active model's catalog metadata confirms `image_input = true`)
- Process execution: `exec` (argv-only single process, no shell parsing), plus the
  permission-gated platform shell escape hatch: `bash` through `sh -c` on Unix and `powershell`
  through `powershell.exe -NoProfile -NonInteractive` on Windows. Command tools are
  non-interactive: stdin is closed unless an `exec` pipe supplies data, and Unix children are
  detached from the TUI controlling terminal, so password prompts fail instead of reading from
  the composer.
- Session planning: `todo_create`, `todo_update_status`, `todo_update`, `todo_delete`,
  `todo_add_dependency`, `todo_remove_dependency`, `todo_query` (session-scoped todo state)
- Session scheduling: `delay` (schedule a one-shot delayed follow-up message; session-scoped, dies on process exit; Execute/Ask permission), `schedule` (schedule a recurring follow-up at a bounded interval 5–3600s; MissedTickBehavior::Delay, no catch-up burst; Execute/Ask permission), `list_scheduled_tasks` (read-only list of active scheduled tasks; shows task ID, kind, and timing only — does not expose message content; Read/Allow), `cancel_scheduled_task` (cancel a scheduled task by ID; Execute/Ask permission)

The default prompt asks models to prefer built-in tools and use shell commands as a fallback when a
native tool cannot cover the task. It also emphasizes accuracy over approval: do not flatter,
fabricate citations, or hide uncertainty when evidence is missing.

In normal CLI/TUI/inline composition, `read` gives the active model compact two-hex line anchors
backed by a bounded in-memory file snapshot. `edit` can use that snapshot for stale-read-resistant
atomic line edits while retaining its legacy string-replacement input. Snapshot handles and hashes
are transient model coordination data: they are omitted from hooks, approval presentation, visible
history, exports, transcripts, and TLOG. Display/history retain ordinary sanitized file content; a
resumed or rebuilt Runtime must read the file again, and every write still passes the current
permission policy.

File tools remain workspace-confined by default. When an interactive permission-aware mode requests
an external path, Talos asks the operator; approval carries only the selected tool operation and
normalized path. Denial, missing headless approval, path reuse, operation reuse, and changed
symlink targets fail closed.

Approval choices are explicit authority classes. Approve once is consumed by one official adapter
invocation and is never stored. Always approve creates an in-memory grant for the active Talos
Session only; it is not written into permission policy or session history, does not survive
new/resume/fork/runtime replacement, and never overrides a configured deny. A different path, tool
provenance, provider identity, or uncovered permission facet still requires approval.

Approval outcomes in the TUI retain the associated tool name, and the temporary `Calling tools...`
compatibility marker is held until a real structured tool call is confirmed. Direct result or
approval events without a matching call keep the marker visible rather than creating a false
correlation.

Load a confined local read-only plugin package explicitly:

```bash
talos --plugin /path/to/plugin-package
```

Repeat `--plugin` for multiple packages. `/plugins` reports the packages that successfully loaded
and their registered capabilities; with no flag, existing behavior is unchanged.

## TUI Text Selection

In the interactive TUI, drag with the primary mouse button to select any visible cells, including
partial transcript lines, tool output, panels, composer text, and the status row. Releasing the
button copies the highlighted text through the same clipboard backends used by `/copy`; no Shift
modifier is required. Dragging at the top or bottom of the history viewport scrolls history while
extending the selection. Terminal resize clamps the selection to the resized frame instead of
clearing it. Selection reads only the rendered frame and never exposes hidden transcript, tool, or
credential data.

Keyboard PageUp/PageDown and Ctrl+Home/Ctrl+End remain the reliable history-navigation controls.
`/copy last` and `/copy all` remain available for semantic message/transcript copies.

## Slash Commands

Type `/` in the TUI to access these commands. The Skill commands are also available in inline
mode.

| Command | Description |
|---|---|
| `/help` | Show available commands |
| `/quit`, `/exit` | Exit Talos |
| `/status` | Show session info (model, token usage) |
| `/auto`, `/auto on`, `/auto off` | Show or change bounded auto-assistance for this session only |
| `/plugins` | Show explicitly loaded local plugin packages and registered capabilities |
| `/mcp` | Show MCP server status and observed tool provenance |
| `/skills` | List available runtime skills and active state |
| `/skills activate <name>` | Activate one Skill body for subsequent provider requests |
| `/skills reference <path>` | Load a bounded reference file for the active Skill |
| `/copy last` | Copy the last assistant message to clipboard |
| `/copy all` | Copy the full transcript to clipboard |
| `/export <path>` | Export transcript to a file (permission-gated) |
| `/new` | Start a fresh session (preserves old session) |
| `/resume` | List other non-empty resumable workspace sessions; `/resume <N>` selects by number, while `/resume <UUID>` accepts only an entry from the same filtered workspace list |
| `/fork` | Fork the active session (clones history into a child session) |
| `/delete` | Open the session picker (excluding the active session); choose a row to remove it |
| `/model` | Open the model picker — three-level navigation: Level 1 lists Recent (≤5 most-recently-used models, persisted) and providers; selecting a provider enters Level 2 (that provider's models); selecting a model with declared variants enters Level 3 (variant list), while variant-less models switch immediately; `Esc` closes the picker. No-argument only: `/model gpt-4o` shows a correction and opens the picker |
| `/connect` | Open the provider picker to connect a new provider (credential and optional custom endpoint/`base_url`). No-argument only: `/connect openai` shows a correction and opens the picker |
| `/todo`, `/todo list`, `/todo show <id>`, `/todo stats`, `/todo export [json|markdown]` | View or export active-session todos (read-only) |
| `/todo delete <id> --confirm` | Delete a session todo item by short-ID or full UUID; requires `--confirm` |
| `/hooks` | Show configured hook diagnostics (declared paths, presence, validation status) without executing hooks |
| `/agile [status]` | Show read-only governance status: board disposition, open iterations, manifest, and Rust validation findings |
| `/attach <path>` | Attach a local image (PNG/JPEG/GIF/WebP) to the next message. Validates file type, size, MIME, and pixel dimensions before attachment. Requires a vision-capable model — `/attach` is rejected before any file read when the active model's `image_input` capability is `Unknown` or `Unsupported`. Use `/attachments` to list queued images and `/detach <index\|all>` to remove one |
| `/attachments` (`/imgs`) | List pending image attachments with their index, byte size, and MIME type. Attachments are queued for the next user submit |
| `/detach <index\|all>` | Remove a pending image attachment by 1-based index, or clear all pending attachments with `all`. The status line reflects the new count immediately |

## Skills

Talos discovers `SKILL.md` files at session startup and injects Level 0 metadata
(skill name, description, and triggers) into the system prompt before the first
model turn.

In YAML frontmatter, `name` and `description` are required. `triggers` is optional: omitting it has
the same result as `triggers: []`. Explicit empty and non-empty lists retain their existing
behavior, and malformed scalar, mapping, or mapping-entry shapes are rejected as YAML parse errors.
Skill authors should write trigger entries as strings; Talos preserves the YAML scalar coercion
behavior accepted by earlier releases.

Skill search paths, in priority order:

- `.talos/skills/` in the active workspace
- `~/.talos/skills/`
- parent `.talos/skills/` directories up to the Git root
- `~/.agents/skills/` (shared, enabled by default; set `[skills] discover_shared = false` to disable)

Talos application configuration defaults to shared discovery. Low-level `SkillLoader`
constructors do not implicitly opt embedders into HOME-based shared discovery unless the
caller passes the application configuration. Symbolic-link following remains disabled by
default; linked targets are controlled separately by `SkillDiscoveryPolicy`.

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

Runtime shutdown is bounded and shareable. `RuntimeHandle::shutdown_controller()` returns a
cloneable shutdown-only controller; `ShutdownOptions` selects `FinishCurrent` or `Interrupt` under
one total monotonic deadline, and every caller receives the same redacted `ShutdownReport`. Once the
shutdown fence closes, new SDK submissions return typed `RuntimeClosing` without enqueueing. The
existing consuming `RuntimeHandle::shutdown()` remains available and returns
`ShutdownIncomplete` instead of reporting incomplete cleanup as success.

After actor-owned durable reconciliation, shutdown runs a build-time frozen set of Talos-owned
resource finalizers once in fixed order under the same total deadline. `ShutdownReport::finalizers()`
contains only fixed identifiers and typed outcomes; arbitrary embedder callbacks and caller-provided
report text are not supported. The current default composition installs no resource finalizers.

Registered tools are permission-wrapped by default. In headless embedding, unresolved `Ask`
decisions are denied unless the embedder supplies policy or an approval handler. `AlwaysApprove`
from an SDK handler installs only an in-memory grant owned by that `RuntimeHandle`; each new runtime
starts with an empty grant store, and current policy denies are rechecked before every admission.

This is not a stable 1.0 SDK guarantee yet. The public embedding surface is `talos-runtime`
plus the protocol and trait types it re-exports from `talos-core`; lower-level `talos-agent`
constructors remain implementation surface unless documented otherwise.

`talos-runtime` is not yet published as an SDK crate in the current release gate. It remains
manifest-ready but blocked by dependency closure; see
[RUNTIME-SDK-CONTRACT](docs/reference/RUNTIME-SDK-CONTRACT.md) and the
[publish gate packet](docs/reference/PUBLISH-GATE-PACKET-2026-07-02.md).

Direct source consumers of `talos-tools` get local `file-read` and `search` capabilities by
default. Enable individual Cargo features for write, document, shell, Git, network, image, or code
intelligence support. The Talos CLI explicitly enables `coding`, so the normal workspace
`cargo build` and `cargo run -p talos-cli` product tool inventory remains unchanged.

## Safety Model

- Read-only workspace tools can run without approval.
- File writes, deletes, Git writes, and shell execution are routed through permissions.
- External file paths require an exact interactive authorization; unresolved headless requests are denied.
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
talos diagnostics status --json    # JSON diagnostics (iterations, gates, trust, redacted)
talos --governance-status          # governance manifest and board disposition
```

All diagnostic commands mask secrets. `config list` replaces `api_key` values with `***` while
preserving `api_key_env` variable names so you can share output safely. `diagnostics status --json`
emits valid JSON via `serde_json` with dynamic iteration state from `docs/iterations/README.md`,
typed residual gates, and bounded `unavailable` diagnostics when governance sources are missing or
malformed.

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
- Scheduled follow-ups (`delay`, `schedule`, `list_scheduled_tasks`, `cancel_scheduled_task`) are
  session-scoped only: tasks die when the process exits, are never persisted, and cannot survive a
  restart. No cron, calendar, or background daemon is planned for v1.

## Contributing And Local Checks

For governed implementation started after the adoption of PR `#83`, follow
[Agent Collaboration And Task Claiming](docs/sop/AGENT-COLLABORATION.md): finalize the
owner-document claim with the actual claim PR number, merge it through an authorized path, and
create the implementation branch from that target-branch claim. An open claim PR does not reserve
work.

Existing pre-adoption work is grandfathered. Wording-only documentation fixes, broken links,
formatting, reviewer follow-ups within the same scope, and mechanically bounded CI/fixture
maintenance may use one PR when they do not change behavior, API, security, dependencies, release
authorization, persistent data, or owner status. Time-critical incident and security response may
use the documented emergency override, with governance reconciled within two business days.

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

Before creating a tag, run the same preflight used by CI and the release workflow:

```bash
./scripts/release_preflight.sh v0.9.0
```

The repository pins the Rust/Clippy toolchain in `rust-toolchain.toml`; do not tag a release from
a different toolchain.

The release workflow builds Linux, macOS, and Windows artifacts from a macOS runner.

The post-v0.2.0 hardening notes that fed the pre-0.3 release line are collected in
[RELEASE-NOTES-DRAFT-2026-07-02](docs/reference/RELEASE-NOTES-DRAFT-2026-07-02.md). GitHub Releases
is the source of truth for the published `v0.9.0` release announcement and downloads.

## Project Status

Talos is moving from core runtime implementation toward product hardening and differentiated
developer experience. The next research priorities are:

- `AGENT-002-B`: broader dotagents compatibility beyond the shipped shared Skills directory.
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
| Agent collaboration and task claiming | [docs/sop/AGENT-COLLABORATION.md](docs/sop/AGENT-COLLABORATION.md) |
| Git workflow | [docs/sop/GIT-WORKFLOW.md](docs/sop/GIT-WORKFLOW.md) |
| Public product site | [https://talos.hwj.zone](https://talos.hwj.zone) &mdash; static GitHub Pages site (source under [`site/`](site/)) |

## License

Talos is licensed under the [Apache License 2.0](LICENSE).
