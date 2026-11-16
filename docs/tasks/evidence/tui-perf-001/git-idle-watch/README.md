# Native idle Git scan follow-up (2026-11-16)

Status: experimental implementation, not delivered or accepted as a whole.

The maintainer confirmed the native Talos process was waiting in its input box.
Binary `/proc/2083228/exe` SHA-256:
`f9c41216eeaa8b2c580a493ee4aada20803a4bd456bf712a7794dbcbc8961f4c`.
This was the user's dirty main-checkout Debug build, not a source-attested build
of the earlier published performance candidate. Do not attribute it to that commit.

Two 30-second samples observed 18.24% and 16.87% of one CPU core. The second
sample's persistent threads accounted for approximately 0.1%, motivating sampling
of short-lived threads. A subsequent 20.36-second, 50ms observer captured ten
`talos-git-statu` workers approximately two seconds apart with observed maximum
lifetime CPU ticks of 28–36 (100 ticks/s). Their final CPU ticks may be missed;
this is lower-bound attribution, not exhaustive accounting. Ghostty measurements
include other windows and are not attributable exclusively to Talos.

Upstream gix 0.90.0 `src/status/iter/types.rs` documents that its serial iterator
collects all results before yielding. The parallel iterator was tested then
rejected: `src/util.rs::parallel_iter_drop` drops JoinHandles rather than joining,
which could release Talos's scan slot before internal workers finish. The candidate
uses synchronous plumbing and a single tracked-file worker limit instead.

Read-only alternating Debug scan timings against the live main workspace:

| Round | Original seconds | Synchronous early-stop seconds | Both dirty |
|---|---:|---:|---|
| 0 | 0.326941145 | 0.186758660 | Yes |
| 1 | 0.326906923 | 0.186479311 | Yes |
| 2 | 0.326662022 | 0.186433586 | Yes |

This finite comparison shows roughly 43% lower wall time, not complete idle
acceptance. The live workspace could change during measurements.

Native watcher-only probe: setup 0.083149999s, three notifications in 30 seconds;
systemd service wall 30.104s, CPU 170ms, peak memory 2.3MiB, swap zero. This includes
startup and registry maintenance, excludes compilation, and does not include TUI
rendering or Git scans. Other sessions were active: no claim that notifications
were spurious or that the repository was unchanged. An earlier compile-plus-probe
run is excluded from runtime CPU attribution.

All probes used CPUQuota=100%, CPUAffinity=0, MemoryHigh=2G, MemoryMax=3G,
MemorySwapMax=0, TasksMax=128, Nice=10. The native user process was observed, not
moved into this resource group. No running binary or other worktree was replaced.

Validation to date: final local TUI library suite 695 passed (zero ignored), including an
actual-loop test: after 3s initialization, no additional scan launched
through 6s of quiet native-watcher operation. This does not prove the 30s fallback,
all watch failures, Windows runtime semantics or final whole-TUI CPU budget.

Current remaining: explicit Git freshness-budget decision, final exact-commit
review/CI and publication, plus native platform acceptance. Full workspace and
MSVC checks and one integrated resource sample are recorded below; that sample
does not establish the full performance budget. The original 479ms tail and
other native acceptance gaps remain.

Probe sources are archived alongside this report as `.rs.txt` evidence, not shipped
Cargo examples. To reproduce, copy to `crates/talos-tui/examples/` with a `.rs`
extension and run the named example against a disposable or read-only workspace.
The watcher probe includes the current implementation via its relative source path.

Platform review: notify 8.2.0 `src/kqueue.rs` re-registers directories with
`add_watch(path, true)` after directory events, bypassing the caller's nonrecursive
watch budget. This is upstream-source evidence, not a macOS runtime measurement.
The candidate returns before creating a watcher on macOS, retaining two-second
polling. Bounded change-driven macOS refresh remains unimplemented. The final
local review found the bypass unreachable and both notification-drain test loops
bounded; exact-commit review remains pending.

Full standard preflight attempt (2026-11-16): workspace check and Clippy passed.
The tools suite reported 1160 passed and one failure in
`hasconfig_non_path_key_still_triggers_url_prohibition`, whose assertion matches
English native Git stderr. The validation unit had omitted LOCAL-DEV's required
`LC_ALL=C` / `LANG=C`. Re-running the identical compiled test binary with that
locale passed (one test, 25ms service wall). No tools implementation was changed.
The full attempt remains failed: 46m15.531s wall, 25m59.150s CPU, 2GiB peak, zero
swap. The subsequent standard preflight with the prescribed locale reused
the existing check, Clippy and test artifacts in under one second per Cargo phase.
An intervening package-only Cargo retry was intentionally terminated after it
started a different feature build; SIGTERM is not recorded as a test pass.

Final locale-correct standard preflight completed successfully (exit 0): locked
workspace check, Clippy, all workspace tests and both independent SDK fixture
modes passed. Service wall 13m39.795s, CPU 8m4.924s, peak 2GiB, swap zero.
The governance validator reported no warnings; claim validation passed with one
pre-existing unrelated SEARCH-001-D3 documentation warning. No code was changed
to accommodate the locale failure.

Final MSVC cross-check: `cargo xwin check --locked -p talos-tui --tests --target
x86_64-pc-windows-msvc` passed, service 1.525s wall / 1.175s CPU / 926MiB peak /
zero swap. This is not native Windows acceptance.

Integrated Debug TUI probe (10,000 history entries, actual main workspace,
120x40 synthetic PTY, 5s warmup + 60s nominal sample, metrics disabled): validated
59.842s steady coverage, CPU 1.504% of one core, peak PSS 49,531KiB, zero steady
output bytes, maximum five threads and no direct child processes. Service wall
67.579s, CPU 2.419s, peak 422.6MiB (includes loading/startup/sampler), swap zero.
`build-receipt.json` binds current frozen source hashes to the measured binary;
this is not a source assertion for the maintainer's running native Talos.

A subsequent metrics-enabled diagnostic observed CPU 2.249%, peak PSS 48,994KiB,
zero steady output. Its metrics warmup was not set: the entire 65s UI run includes
startup (three draws) and eight Git scans totaling 1.622786s, maximum 269.849ms.
Do not report those cumulative counts as the sampler's post-warmup counts or
compare this run as instrumentation-overhead evidence. Other sessions changed
the repository during these sequential runs; neither is a controlled quiet-repo
baseline or a three-round budget pass. The earlier native 17–18% samples used a
different workload/binary. No precise savings ratio or full idle acceptance is
claimed. Raw samples and metrics are retained alongside this report.

Acceptance adjustment authorized on 2026-11-16: the maintainer explicitly replied
“接受30秒”. Healthy notification mode uses a 30-second reconciliation interval and
35-second snapshot lifetime instead of the original five-second freshness budget.
Notifications still trigger coalesced refreshes. Thirty seconds is the scan
scheduling interval, not a completion guarantee when scanning stalls; expiry marks
an old snapshot unknown. Failed/unavailable watchers retain two-second polling and
the five-second lifetime. Remaining full-task acceptance is unchanged.

Stage published through PR #960, merge `3b1b1f67`, implementation `4c9bf99d`.
Exact-head independent Agent review and all applicable CI code/platform checks
passed in run `43861152146`; macOS remote preflight and Linux local preflight are
separate evidence. The global missing-Issue-#957 reconciliation check remains an
advisory failure. Full task/native acceptance remains open in the owner and #923.
