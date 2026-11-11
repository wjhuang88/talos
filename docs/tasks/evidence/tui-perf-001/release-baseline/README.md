# Release baseline, source 52263c62

This is the pre-optimization release baseline, not a performance improvement
claim or completion of P0. All 54 cases passed: six scenarios, three history sizes
(100/1000/10000), three rounds. Each run requests 35 seconds including 5 seconds
warmup. Actual sampled spans are retained in the raw coverage reports.

Reproduction (run from the performance worktree):

```sh
python3 scripts/replay_tui_perf_matrix.py \
  --binary baseline=/tmp/talos-p0-binaries-52263c62/release \
  --output /tmp/talos-p0-release-full --profile release-with-symbols
python3 scripts/summarize_tui_perf.py /tmp/talos-p0-release-full
```

`raw-reports.tar.gz` contains the complete matrix manifest, process samples,
schema-2 metrics and coverage checks. Extract it into a new directory before
running the summary command. `summary.json` records per-round measurements and
aggregate medians/ranges. The sibling `baseline-build/receipt.json` binds the
immutable binary to source SHA, toolchain and release settings. The binary was
not rebuilt during the matrix. Developer source edits made during measurement
are not part of this binary. No builds, tests or competing benchmark runs were
started during the batch; lightweight source inspection/editing did occur.

Selected CPU results (100% = one logical core):

| History | Idle median (range) | Stream median (range) | Resize median (range) |
|---|---|---|---|
| 100 | 0.336% (0.270–0.337) | 1.937% (1.876–2.074) | 0.571% (0.504–0.571) |
| 1000 | 0.334% (0.303–0.370) | 2.144% (2.027–2.278) | 1.276% (1.136–1.445) |
| 10000 | 0.370% (0.369–0.370) | 2.077% (1.980–2.416) | 10.004% (9.770–10.134) |

Idle still submits approximately 200 draws in the steady window, despite no
terminal output changes. Large-history resize is materially more expensive and
requires examination of history rebuild timing; this alone does not justify an
incremental-history rewrite. CPU clock granularity is procfs clock ticks, so
small absolute differences should not be overinterpreted.

Limitations: synthetic Linux PTY, no terminal emulator cost or native visual
acceptance; main-process CPU/PSS, main-thread context switches, sampled direct
child count only. No input probes were enabled in this matrix, so absent input
latency samples are not zero latency. No allocation profiling or flamegraph.
Workspace is empty, so these are not Git-status performance measurements.
Debug, Git/switch, input-probe and instrumentation-overhead matrices remain
necessary before freezing the full P0 budget. Windows native acceptance remains
pending because no environment is available.
