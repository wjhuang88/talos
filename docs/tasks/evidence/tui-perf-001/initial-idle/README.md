# Initial idle measurement — TUI-PERF-001

This is an initial debug diagnostic baseline, not P0 completion or a frozen budget.
Source base: d355dd68; local P0 instrumentation/replay candidate. The exact sampled
binary SHA-256 is recorded in each JSON. Source commit attribution and the full
scenario/platform/build matrix remain pending. No scheduler optimization exists.

Linux synthetic PTY, 120x40, 100 deterministic assistant history entries,
no workspace/Git, 35 seconds after ready, first 5 seconds excluded from process
statistics. Instrumentation is enabled and includes startup in its aggregate
counters; do not treat aggregate counters as steady-window-only measurements.
Three sequential runs; no compilation/tests were launched during the samples.
Host power mode and unrelated host load were not controlled. No terminal emulator
was involved. PSS is process residency, not heap size. Child/terminal accounting,
allocation statistics, and instrumentation overhead remain unmeasured here.

Reproduction (from repository, after building the replay example):

```sh
CARGO_TARGET_DIR=/home/wjhuang/WorkSpace/personal/talos/target cargo build --locked -p talos-tui --example tui_perf_replay
TALOS_TUI_METRICS=/tmp/idle-metrics.json python3 scripts/measure_tui_perf.py /home/wjhuang/WorkSpace/personal/talos/target/debug/examples/tui_perf_replay --seconds 35 --warmup 5 --output /tmp/idle.json
```

| Round | CPU % (one core = 100%) | Peak PSS KiB | Draw | Layout | Diff | History rebuild | Timer wakes |
|---|---:|---:|---:|---:|---:|---:|---:|
| 1 | 2.404 | 11269 | 237 | 237 | 237 | 2 | 234 |
| 2 | 2.371 | 11386 | 237 | 237 | 237 | 2 | 234 |
| 3 | 2.270 | 10906 | 237 | 237 | 237 | 2 | 234 |

CPU median 2.371%, range 2.270–2.404%. All steady output byte counts were zero.
The repeated draws/layouts/diffs despite zero steady bytes confirm avoidable
computation. History rebuild counts include initialization; stable-history cache
hits mean this evidence does not support a full-history rewrite. No claim about
relative cost against Codex or final CPU savings follows from these samples.

Environment verification: pinned `rustc 1.99.0 (b940084d7 2026-09-28)`,
Linux `7.2.14-arch1-1`. `perf` is not on PATH and perf_event_paranoid is 2.
No kernel setting was changed and no sampling flamegraph was captured.

Aggregate timing (startup included, milliseconds; nested, not additive):

| Round | Draw | Layout rectangle planning | History rebuild | Diff | Flush |
|---|---:|---:|---:|---:|---:|
| 1 | 790.541 | 1.243 | 2.005 | 159.572 | 0.012 |
| 2 | 786.460 | 1.269 | 1.940 | 159.072 | 0.014 |
| 3 | 754.152 | 1.305 | 1.836 | 149.593 | 0.009 |
