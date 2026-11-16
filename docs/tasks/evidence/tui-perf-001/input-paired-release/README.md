# Release input-probe comparison

36 cases completed: six scenarios, history 10000, three rounds, baseline/candidate.
All evaluated CPU/PSS/input latency and applicable animation checks passed.
Baseline source 52263c62; candidate source 539adfd2. See sibling build receipts.
35s runs, 5s warmup, input probes and process-tree accounting enabled.

| Candidate scenario | Worst round P95 upper bound (ms) | Worst round P99 upper bound (ms) |
|---|---:|---:|
| candidate-idle-10000 | 0.653 | 0.653 |
| candidate-animation-10000 | 0.524 | 0.841 |
| candidate-stream-10000 | 0.754 | 0.754 |
| candidate-approval-10000 | 0.457 | 0.457 |
| candidate-scroll-10000 | 1.106 | 1.189 |
| candidate-resize-10000 | 66.975 | 66.975 |

These are histogram bounds for input-to-submit, not native visible-pixel latency.
Metrics on, no workspace. Fixed baseline-first order can retain host drift.
No builds/tests/other experiments ran during sampling. Whole-task acceptance remains open.
