# Debug baseline, source 52263c62

All 54 cases completed with exit 0: six scenarios, three history sizes,
three rounds; 35 seconds per run including five seconds warmup. The source-bound
binary is identified by the sibling `baseline-build/receipt.json`. The runner
was at `15ef42c1`; its checkout revision is not the binary's source revision.

```sh
python3 scripts/replay_tui_perf_matrix.py \
  --binary baseline=/tmp/talos-p0-binaries-52263c62/debug \
  --output /tmp/talos-p0-debug-full --profile debug
python3 scripts/summarize_tui_perf.py /tmp/talos-p0-debug-full
```

`raw-reports.tar.gz` contains the manifest and all process/metrics/coverage
reports; `summary.json` retains rounds and aggregates. CPU percentages use one
logical core as 100%. These are pre-optimization results, not improvements.

| History | Idle median (range) | Stream median (range) | Resize median (range) |
|---|---|---|---|
| 100 | 1.936% (1.770–1.936) | 10.906% (10.471–13.064) | 2.142% (1.972–2.274) |
| 1000 | 1.436% (1.403–1.770) | 12.382% (10.017–14.040) | 4.232% (3.814–4.301) |
| 10000 | 1.579% (1.479–1.648) | 11.937% (11.937–12.602) | 14.580% (14.540–15.147) |

No builds, tests or competing measurements were started during this batch.
Lightweight report reading/source inspection occurred. The synthetic Linux PTY
does not measure terminal emulator cost or native visual behavior. Main-process
CPU/PSS and main-thread context switches do not imply full process-tree lifetime
accounting. Input probes and process-tree collection were disabled; empty
histograms are unavailable data, not zero latency. Workspace is empty, so Git
scans are not covered. Release and debug batches were sequential, not interleaved;
their differences alone cannot isolate compiler optimization from host drift.

Full P0 still needs Git/switch, input probes, instrumentation-overhead measurements
and budget freeze. No P1–P3 runtime improvement is represented by this report.
