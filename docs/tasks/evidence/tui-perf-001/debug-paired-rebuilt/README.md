# Corrected debug paired comparison

All 108 cases completed with exit 0. All explicitly evaluated checks passed:
active CPU median within +20%, peak PSS within baseline peak*1.2+8MiB,
zero steady idle/approval frames, zero stable animation layout/history rebuilds,
no Git without workspace, and animation deadline P99 <=50ms.

Baseline source 52263c62; candidate source 539adfd2. Corrected candidate receipt:
`../candidate-build/debug-rebuilt-receipt.json`. The previous stale-P2 matrix
remains in ../debug-paired for audit and is excluded from P3 acceptance.

| Case | Baseline CPU median | Candidate CPU median |
|---|---:|---:|
| idle-100 | 1.403% | 0.000% |
| idle-1000 | 1.537% | 0.000% |
| idle-10000 | 1.479% | 0.000% |
| animation-100 | 1.596% | 1.505% |
| animation-1000 | 1.708% | 1.588% |
| animation-10000 | 1.508% | 1.412% |
| stream-100 | 11.249% | 7.825% |
| stream-1000 | 11.279% | 8.260% |
| stream-10000 | 11.016% | 8.089% |
| approval-100 | 1.246% | 0.000% |
| approval-1000 | 1.302% | 0.000% |
| approval-10000 | 1.278% | 0.000% |
| scroll-100 | 1.914% | 0.537% |
| scroll-1000 | 2.014% | 0.537% |
| scroll-10000 | 2.190% | 0.640% |
| resize-100 | 2.008% | 0.702% |
| resize-1000 | 3.972% | 2.410% |
| resize-10000 | 16.434% | 15.824% |

Metrics on, input probes off, workspace empty; 35s runs include 5s warmup.
No concurrent builds/tests/experiments. Fixed baseline-first ordering may retain
host drift. Zero CPU is procfs-resolution limited. This does not establish input
latency, Git freshness, metrics-off CPU, allocations, native terminal or whole-task acceptance.
