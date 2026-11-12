# Diagnostic overhead comparison

24 cases passed: three rounds, release/debug, metrics off/on, idle/stream,
10,000 history entries. Each run lasted 35 seconds including five seconds warmup.
Within each round/profile, off idle/stream ran before on idle/stream. Immutable
source-52263c62 binaries were used; no builds/tests ran during sampling.
No input probes or process-tree sampling were enabled.

The raw archive retains all 12 two-case matrices. `paired-cpu.json` records
CPU percentage values and on-minus-off differences for matching round/profile/
scenario. One logical core is 100%. Median paired differences:

| Profile | Idle | Stream |
|---|---:|---:|
| Release | approximately 0.000 percentage points | -0.142 percentage points |
| Debug | +0.169 percentage points | +1.024 percentage points |

Negative differences are measurement/host variation, not a speedup caused by
diagnostics. Three repetitions and procfs clock resolution do not establish
zero release overhead or a confidence interval. Debug instrumentation has a
visible observer effect in this sample. Candidate comparisons must keep the
switch identical; final CPU budgets should also be checked with diagnostics off.
This comparison does not measure output-report serialization in the steady window.

Reproduce for each profile, mode and round using a new directory:

```sh
python3 scripts/replay_tui_perf_matrix.py \
  --binary baseline=/tmp/talos-p0-binaries-52263c62/release \
  --output /tmp/talos-p0-overhead-release-off-r1 \
  --profile release-with-symbols --scenarios idle stream \
  --histories 10000 --rounds 1 --metrics off
```

Use `on` for the matching second matrix; repeat three times, then repeat with
the debug binary/profile. These are diagnostic overhead results, not P1–P3
optimization evidence.
