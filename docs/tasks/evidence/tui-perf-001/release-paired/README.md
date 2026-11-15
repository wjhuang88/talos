# Release paired comparison

All 108 cases completed with exit 0: baseline source `52263c62`, candidate
`539adfd2`, six scenarios, three histories, three rounds, alternating baseline
and candidate per case. Each run includes five seconds warmup and thirty seconds
steady sampling. Source-bound receipts are in sibling baseline-build/candidate-build.

All explicitly evaluated checks passed: active CPU median within +20%, peak PSS
within baseline peak*1.2+8MiB, zero steady idle/approval draws/diffs/flushes,
zero stable animation layout/history rebuilds, no Git without workspace, and
animation deadline P99 <=50ms. Twelve evidence-script tests passed before report
creation. Raw sample coverage, identity and histogram validation passed.

| Scenario/history | Baseline CPU median | Candidate CPU median |
|---|---:|---:|
| idle-100 | 0.202% | 0.000% |
| idle-1000 | 0.267% | 0.000% |
| idle-10000 | 0.236% | 0.000% |
| animation-100 | 0.269% | 0.236% |
| animation-1000 | 0.269% | 0.235% |
| animation-10000 | 0.271% | 0.236% |
| stream-100 | 1.472% | 1.077% |
| stream-1000 | 1.479% | 1.240% |
| stream-10000 | 1.440% | 1.491% |
| approval-100 | 0.167% | 0.000% |
| approval-1000 | 0.201% | 0.000% |
| approval-10000 | 0.200% | 0.000% |
| scroll-100 | 0.337% | 0.101% |
| scroll-1000 | 0.337% | 0.101% |
| scroll-10000 | 0.470% | 0.168% |
| resize-100 | 0.370% | 0.135% |
| resize-1000 | 0.970% | 0.637% |
| resize-10000 | 5.735% | 5.310% |

The stream/10000 CPU median regressed about 3.5%; this remains under the frozen
20% budget and is not presented as a speedup. Zero observed CPU is limited by
procfs tick resolution, not proof of literally zero instructions. Metrics were
on, input probes off, workspace empty. No input-latency, Git, metrics-off CPU,
allocation or native-terminal acceptance follows from this matrix. No builds or
tests ran during sampling. Binary ordering was fixed baseline-first within each
pair; host drift remains possible. Stage timings overlap and are not exclusive CPU.

Reproduce using the exact command in the owner under “Formal paired release
sampling started”. Summarize with `scripts/summarize_tui_perf.py`; evaluate listed
budgets with `scripts/compare_tui_perf.py`. Keep raw reports and receipts alongside
these derived JSON files. This report does not close whole-task acceptance.
