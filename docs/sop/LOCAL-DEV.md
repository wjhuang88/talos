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

## Resource limits on a shared Linux workstation

Cargo build jobs only limit compilation concurrency; they do not cap memory,
linker threads, test threads or concurrent work from other sessions. After an
OOM/resource-pressure incident, stop the task's heavy work and establish verified
process-tree limits before resuming. Do not raise limits merely to obtain a pass.
On a systemd/cgroup-v2 host, run one validation unit at a time, for example:

```sh
systemd-run --user --wait --pipe --collect \
  -p CPUQuota=100% -p MemoryHigh=2G -p MemoryMax=3G \
  -p MemorySwapMax=0 -p TasksMax=128 -p Nice=10 \
  --working-directory="$PWD" \
  --setenv=CARGO_BUILD_JOBS=1 --setenv=RUST_TEST_THREADS=1 \
  --setenv=LC_ALL=C --setenv=LANG=C \
  /usr/bin/cargo test --locked -p talos-tui
```

Verify the actual Cargo path, pinned toolchain, writable disk-backed target and
cgroup controller values first. Pass the selected `CARGO_TARGET_DIR` explicitly
when the worktree lives on tmpfs. Keep resource peak/exit evidence from systemd;
if user systemd or controllers are unavailable, do not silently fall back to an
unrestricted heavy command. These example limits bound this job, not the whole
workstation. CPU-throttled validation is not an unthrottled performance baseline;
record caps and throttling when interpreting any measurements.
Also inspect `memory.events` and host memory/I/O pressure during a long run:
repeated `high` events can cause reclaim stalls without an OOM. Stop the task
if it degrades interactive use; a memory cap alone does not prove low impact.
After an intentional stop, require the command's completion marker and test
summary before recording a pass. `systemd-run --wait` or service `Result=success`
can report success for a service terminated by the requested SIGTERM.
The user service manager can inherit a different locale from the invoking
shell. Pin diagnostic locale as above for tests that compare host-tool messages;
this does not replace separate localized UI acceptance.

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
flush, input/output events and timer wakes; changed cells are accumulated.
`history_rebuild` counts projection refreshes, including incremental tail refreshes;
it does not imply every historical entry was rebuilt. Timings
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

Use `python3 scripts/summarize_tui_perf.py /completed/matrix` to emit a JSON
summary after sampling ends. It validates the expected case identities and binary
hashes, reports CPU/PSS/output medians and ranges, and preserves per-round stage
counts and P95/P99 latency bounds. Empty latency histograms produce `null`, not
zero latency. The overflow bucket uses the observed maximum as its upper bound.
This report cannot replace raw samples, native terminal acceptance, or a profiler.

Prepare isolated Git inputs with
`python3 scripts/create_tui_perf_git_fixtures.py /new/fixture/directory`.
This developer-only preparation uses host Git to create a one-file clean repository
and one with 10,000 untracked files, plus an empty non-repository. If Git is absent,
preparation fails explicitly; production TUI Git reads still use the Rust library.
Replacement trigger: adopt a shared Rust fixture builder if these inputs become
part of shipped runtime behavior or automated cross-platform acceptance.
For release profiling, use both `CARGO_PROFILE_RELEASE_DEBUG=1` and
`CARGO_PROFILE_RELEASE_STRIP=false`; retain a source-bound binary copy and verify
debug sections before measurement. The workspace release profile strips symbols
by default, so setting debug information alone is insufficient.

`--scenarios workspace-switch --workspace /fixture/one --alternate-workspace
/fixture/two` alternates the displayed workspace every second. Its metrics must
contain actual `workspace_change` events; sending status updates alone is not
acceptance evidence. `--cgroup` optionally records leaf cgroup-v2 CPU throttling,
CPU/memory/I/O pressure and memory events at each sample. Both the sampler and
matrix runner accept it; default is off. It adds procfs/cgroup reads and includes
the local cgroup path in the report, so review paths before sharing. Missing
observations are marked unavailable. Counters cover the whole leaf group, not
individual frames; ancestor limits are not captured. Use matching options for
baseline/candidate and do not infer causality from coincident pressure alone.
`--process-tree` optionally records bounded live process-tree
snapshots (up to 256 processes), gathering children from every live thread and
identifying processes by PID and start ticks. Missing/racing reads and truncation
are explicit. This adds sampler overhead and must be identical on both sides of
a comparison. Top-level CPU/PSS remain main-process values. Short-lived children
and exited threads can escape polling; do not subtract live-tree CPU sums across
changing membership or call live-thread context-switch sums lifetime totals.

For focused high-rate stream regressions, set `TALOS_REPLAY_STREAM_HZ=200`
(default 20, allowed 1–1000). The fixture retains token order and emits a final
Content::End; sampler and matrix reports record the requested rate. A higher
rate also increases total text, so compare identical rates and durations. A
successful process exit and reduced draw count alone do not prove final transcript
content, cancellation fairness, or native visual correctness.

The optional `git_snapshot_age` histogram samples successful snapshot age when a
frame reads Git status. Collection time comes from worker completion, not UI
receipt. These are frame-weighted age samples, not scan durations: idle unchanged
snapshots produce no samples and unknown state has no age. Age observations use
the observation's warmup window; they do not subtract the age from that window.

For a completed alternating matrix whose binary labels are exactly `baseline`
and `candidate`, `python3 scripts/compare_tui_perf.py /completed/matrix` reports
per-case checks against TUI-PERF-001's frozen budgets. It checks active CPU median,
peak PSS, idle/approval frame suppression, stable animation planning and animation
deadline P99; input P95/P99 is checked only when input probes were enabled. Empty
applicable latency histograms fail the check. Metrics-off idle CPU is evaluated
separately from metrics-on frame counts. Its `evaluated_checks_pass` field covers
only listed checks, not whole-task acceptance. Retain source-bound binary receipts,
raw reports and the native/long-run/allocation evidence separately.

The additional `animation_deadline_lateness` stage observes only a visible
animation whose own cadence is due. `deadline_lateness` remains the combined
timer-wakeup metric (including Git, stream and expiry work). Each metric also
contains a bounded `over_50ms_count`; the animation comparator uses that count
to test the frozen P99 <= 50 ms threshold exactly without pretending to know an
exact percentile. Legacy reports retain their conservative mixed-deadline
histogram check and are labeled accordingly. Neither observation proves visible
pixel timing or identifies the cause of host scheduling delays.

For sustained input probes, add `--input-probe-interval-ms 50` to the sampler or
matrix alongside `--input-probe` (default500ms, allowed10..10000ms). Probes follow
fixed deadlines and skip overdue slots without catch-up bursts. Reports include
the interval and skipped count; total probes include post-window drain, so they
are not steady-window event rates. Retain the original incomplete report if a
watchdog or write failure prevents completion. A configured high stream rate
must be supported by both replay binaries; old fixed20Hz fixtures need an
explicit recorded fixture-only patch before use in high-rate comparisons.
