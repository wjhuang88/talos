# Input-probe baseline, source 52263c62

Release and debug each passed 18 cases: six scenarios, 10,000 history entries,
three rounds, 35 seconds including five seconds warmup. Both use the immutable
binaries recorded in `../baseline-build/receipt.json`. The runner source is
`15ef42c1`; summaries additionally include the process-lifetime accounting change
whose seven Python regression tests passed after measurement ended.

Reproduce once per profile, using a new output directory:

```sh
python3 scripts/replay_tui_perf_matrix.py \
  --binary baseline=/tmp/talos-p0-binaries-52263c62/release \
  --output /tmp/talos-p0-release-input --profile release-with-symbols \
  --histories 10000 --input-probe --process-tree
python3 scripts/replay_tui_perf_matrix.py \
  --binary baseline=/tmp/talos-p0-binaries-52263c62/debug \
  --output /tmp/talos-p0-debug-input --profile debug \
  --histories 10000 --input-probe --process-tree
```

`raw-reports.tar.gz` retains both complete matrices. Summaries validate identity,
coverage, metrics and nonempty submitted latency evidence. Each probe injects
`x` then backspace every 500 ms. The metric starts at first event receipt in a
batch and ends at successful terminal submission. It excludes queueing before
receipt and physical pixel latency. Coalescing can combine events into one sample.

P99 upper bounds in milliseconds, shown as the range across three rounds:

| Scenario | Release | Debug |
|---|---:|---:|
| Idle | 0.536–0.539 | 3.284–3.924 |
| Animation | 0.514–0.546 | 3.339–3.978 |
| Stream | 0.932–1.637 | 6.853–8.735 |
| Approval waiting | 0.428–0.467 | 2.636–2.844 |
| Scroll | 1.062–1.131 | 4.425–5.512 |
| Resize | 60.585–70.556 | 85.464–122.689 |

These are histogram bounds capped by the observed maximum, not exact quantile
samples. P95, per-round counts and both bounds remain in the summaries. Approval
is display-only; these input probes do not validate authorization decisions.

Optional process-tree snapshots identify lifetimes by PID/start ticks and account
CPU differences only between adjacent observations of that lifetime. First/last
unobserved intervals, short-lived processes and missing reads are excluded. This
is a lower-bound observation, not complete lifetime resource accounting. PSS is
summed for simultaneously observed processes; it is not heap allocation volume.
The terminal emulator is absent from this synthetic PTY benchmark. Process-tree
sampling adds observer overhead; use identical settings for candidate comparison.

No builds/tests or competing measurements ran during the batch. Light report
inspection and source edits occurred. Input probes are extra work, so CPU results
must not be compared directly with idle-without-input samples as optimization
evidence. Workspace is empty; Git measurements are separate. No runtime scheduling
or cache optimization is represented here; full P0 budgets remain pending.
