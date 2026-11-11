# SOP: Local Development

## Purpose

Set up and maintain a local development environment for Talos.

## Prerequisites

- Rust toolchain: repository-pinned `rust-toolchain.toml`, edition 2024
- OS: macOS, Linux, or WSL2

## Setup

### 1. Install Rust Toolchain

```bash
rustup default stable
rustup update
```

### 2. Clone and Build

```bash
git clone <repo-url> talos
cd talos
cargo build --workspace
```

### 3. Verify

```bash
cargo check --locked --workspace
cargo test --locked --workspace
cargo clippy --locked --workspace -- -D warnings
```

All three must exit 0 before starting work.

## Development Commands

| Command | Purpose |
| --- | --- |
| `cargo build --workspace` | Build all crates |
| `cargo check --locked --workspace` | Fast type-check all crates with the committed lockfile |
| `cargo test --locked --workspace` | Run all tests with the committed lockfile |
| `cargo clippy --locked --workspace -- -D warnings` | Lint shipped workspace targets with CI's warning policy |
| `./scripts/release_preflight.sh vX.Y.Z` | Validate a release candidate using the exact CI flow |
| `cargo doc --workspace --no-deps` | Generate documentation |
| `cargo run -p talos-cli` | Run the CLI |
| `cargo test -p talos-{crate}` | Test a specific crate |
| `cargo test -p talos-{crate} test_name` | Run a specific test |

## Windows Cross-Compilation From macOS/Linux

Use `cargo xwin` with the MSVC target for Windows cross-compilation. Run from
this repository so `rust-toolchain.toml` selects the pinned toolchain:

```bash
rustup target add x86_64-pc-windows-msvc
cargo xwin check --locked -p talos-tools --all-features --target x86_64-pc-windows-msvc
```

Install `cargo-xwin` if unavailable. It uses a local Windows SDK/CRT cache and
LLVM tools; allow its cache setup when the execution sandbox blocks it. Do not
substitute a GNU/MinGW target and report missing MinGW as a project blocker.
Cross-compilation proves build compatibility only; run lifecycle, permission,
and process ownership tests on native Windows CI before claiming Windows delivery.

## Project Structure

```
talos/
├── Cargo.toml              # Workspace root
├── crates/
│   ├── talos-core/         # Core types and traits
│   ├── talos-agent/        # Agent loop
│   ├── talos-tools/        # Tool registry and built-ins
│   ├── talos-sandbox/      # Platform sandboxes
│   ├── talos-permission/   # Permission engine
│   ├── talos-provider/     # LLM providers
│   ├── talos-session/      # Session storage
│   ├── talos-skill/        # Skill system
│   ├── talos-plugin/       # Plugin runtime
│   ├── talos-mcp/          # MCP integration
│   ├── talos-config/       # Configuration
│   ├── talos-cli/          # CLI entry point
│   └── talos-rpc/          # JSON-RPC server
├── docs/                   # Governance and reference
├── tests/                  # Integration tests
└── AGENTS.md               # Agent coding guide
```

## Useful Tools

- `cargo-expand` — Inspect macro expansions
- `cargo-tree` — Visualize dependency tree
- `cargo-outdated` — Check for outdated dependencies
- `cargo-audit` — Security vulnerability check
- `rust-analyzer` — IDE support (VS Code, Neovim)

## TUI rendering diagnostics (TUI-PERF-001, P0 candidate)

Set `TALOS_TUI_METRICS=/absolute/path/to/unique-report.json` before starting the
TUI to opt into aggregate rendering measurements. Unset it for normal operation.
The report is written when the UI loop exits (also on an error return); a killed
process may not produce it. An unwritable destination silently loses diagnostics,
so verify that a fresh report exists. Use a unique path per process/run.

Counters cover draw, layout planning, history rebuild/cache hit, Git scan, diff,
flush, input/output events and timer wakes; changed cells are accumulated. Timings
use bounded 32-bucket log2 nanosecond histograms with a saturating overflow bucket.
Durations are nested and must not be summed as exclusive CPU cost. No input,
prompt, tool payload, clipboard data or rendered text is recorded. The counters
in `stages` are process cumulative and include startup. Schema 2 additionally
reports `steady_stages` after `TALOS_TUI_METRICS_WARMUP_MS` (bounded to one hour,
default zero), measured from Tui::run entry; durations crossing the warmup
boundary are excluded. The PTY sampler has a separate ready-state origin, so
report both actual spans and do not assume exact boundary alignment. Instrumentation overhead still needs comparison
with the switch disabled before interpreting baseline results.

The offline example `tui_perf_replay` drives the real TUI without a provider or
executing tools. The developer-only Linux script `scripts/measure_tui_perf.py`
uses Python's PTY and procfs access to measure CPU/PSS/output; it is not a runtime
dependency or proof of native terminal appearance. Its report explicitly lists
currently missing child/terminal accounting and latency/allocation measurements.

The sampler's `--input-probe` types and erases a synthetic character every 500 ms.
`input_to_submit` measures the first received event of a batch through the next
successful frame submission; cancelled/exiting batches without a submitted frame
are not samples. Histogram quantiles are bounds, not exact latency values. Resize
and stream wakes have separate counters. The sampler also supports `approval`,
`scroll` and `resize` scenarios; these are display-only fixtures and cannot grant
permission or execute tools. Every report records the sampled binary SHA-256 and
OS identity. Record source revision, build profile, toolchain and host power mode
with the eventual baseline report; a binary digest alone does not prove those.

For a serial scenario matrix, use `scripts/replay_tui_perf_matrix.py` with
`--binary baseline=/absolute/binary --output /new/report/directory --profile debug`.
The default covers six scenarios, three history sizes and three rounds. Split by
`--histories 100` (then 1000 and 10000) for shorter checkpoints. Add a second
`--binary candidate=/absolute/binary` to alternate baseline/candidate within each
case; never benchmark two processes simultaneously. `--metrics off` supports an
instrumentation-overhead comparison. Output directories must be new; manifests
retain completed cases on failure, and binaries are checked for changes before
each launch. Do not compile concurrently. Profile is caller-supplied metadata;
retain the actual build command and source-to-binary binding with the report.

Matrix and standalone sampling both reject premature successful exits and gaps
above 750 ms between steady samples. Matrix additionally parses metrics, checks
histogram/count consistency, and requires steady scroll/resize state changes
matching injected actions within one boundary event. Idle and approval waiting
may legitimately have zero steady frames after scheduling optimization. Context
switch counters currently describe the main thread; child-process CPU and
per-thread totals remain unavailable rather than implicitly included.
