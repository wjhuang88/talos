# Git status baseline, source 52263c62

All 18 cases passed: release/debug, three workspace fixtures, three rounds.
Each case used idle state with 10,000 history entries, input probes and optional
process-tree snapshots, for 35 seconds including five seconds warmup.
The immutable binaries are recorded in `../baseline-build/receipt.json`.

Fixture preparation:

```sh
python3 scripts/create_tui_perf_git_fixtures.py /tmp/talos-p0-git-fixtures-v2
```

The fixtures are an empty non-repository, a clean repository with one tracked file,
and a repository with one tracked file plus 10,000 untracked files. Substitute each
fixture name and each binary/profile to reproduce in a new output directory:

```sh
python3 scripts/replay_tui_perf_matrix.py \
  --binary baseline=/tmp/talos-p0-binaries-52263c62/release \
  --output /tmp/talos-p0-git-release-many-untracked --profile release-with-symbols \
  --scenarios idle --histories 10000 \
  --workspace /tmp/talos-p0-git-fixtures-v2/many-untracked \
  --input-probe --process-tree
```

The six summaries and raw archive retain case identity, CPU/PSS, stage counts,
latency bounds and tree observations. Every case recorded 59 actual steady Git
scans. Median accumulated scan elapsed time per approximately 30-second window:

| Fixture | Release | Debug |
|---|---:|---:|
| Non-repository | 3.882 ms | 8.037 ms |
| Small clean | 44.795 ms | 154.358 ms |
| 10,000 untracked | 761.630 ms | 3055.535 ms |

The measurements establish that synchronous status scanning can materially add
work to drawing, especially for many untracked files. They do not establish
worst-case repository latency or hard cancellation bounds. The status reader
checks whether any change exists, so it need not consume every untracked entry;
the fixture is not a complete-diff benchmark. Non-repository probing is real work
even though there is no branch to display.

All cases include synthetic input every 500 ms; their CPU cannot be compared
directly with no-input idle as an optimization result. Stage durations are elapsed
wall time, nested inside drawing, not independent CPU attribution. No builds or
tests ran during sampling. Synthetic PTY, process-tree polling and software input
latency limitations are the same as the input baseline. Workspace-switch tests
remain separate; these cases hold one workspace throughout each run.
