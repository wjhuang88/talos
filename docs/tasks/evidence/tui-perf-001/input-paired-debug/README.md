# Debug input-probe comparison

All 36 cases completed, and all evaluated checks passed. Baseline source52263c62;
candidate539adfd2 uses the corrected binary in ../candidate-build/debug-rebuilt-receipt.json.
Six scenarios, history10000, three rounds,35s with5s warmup,input probes/process-tree on.

| Candidate scenario | Worst round P95 upper bound (ms) | Worst round P99 upper bound (ms) |
|---|---:|---:|
| candidate-idle-10000 | 3.185 | 3.185 |
| candidate-animation-10000 | 3.333 | 3.333 |
| candidate-stream-10000 | 4.127 | 4.127 |
| candidate-approval-10000 | 2.527 | 2.617 |
| candidate-scroll-10000 | 4.673 | 4.848 |
| candidate-resize-10000 | 110.863 | 110.863 |

Histogram bounds measure input-to-submit, not native visible-pixel latency.
Metrics on, no workspace. No concurrent builds/tests/experiments. Fixed baseline-first
ordering may retain host drift. Whole-task acceptance remains open.
