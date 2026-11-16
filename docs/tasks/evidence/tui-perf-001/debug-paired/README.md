# Debug paired comparison — failed gate

All 108 samples completed (exit 0). Budget evaluation did not pass:
all three animation history sizes fail stable layout/history preparation checks.
CPU, peak PSS, idle/approval frame and animation deadline checks passed.

Candidate receipt claims source 539adfd2. Binary/source consistency and the
unexpected repeated layout computations require investigation before acceptance.
Do not discard this failed evidence or treat release results as debug acceptance.

| Case | Baseline CPU median | Candidate CPU median |
|---|---:|---:|
| idle-100 | 1.370% | 0.000% |
| idle-1000 | 1.436% | 0.000% |
| idle-10000 | 1.413% | 0.000% |
| animation-100 | 1.520% | 1.277% |
| animation-1000 | 1.445% | 1.588% |
| animation-10000 | 1.378% | 1.575% |
| stream-100 | 9.714% | 10.454% |
| stream-1000 | 11.042% | 12.205% |
| stream-10000 | 10.505% | 12.364% |
| approval-100 | 1.246% | 0.000% |
| approval-1000 | 1.136% | 0.000% |
| approval-10000 | 0.976% | 0.000% |
| scroll-100 | 1.879% | 0.470% |
| scroll-1000 | 1.846% | 0.436% |
| scroll-10000 | 2.092% | 0.573% |
| resize-100 | 1.873% | 0.670% |
| resize-1000 | 3.585% | 2.444% |
| resize-10000 | 14.540% | 13.540% |

Confirmed stale artifact: original debug binary contains `/tmp/talos-perf-p2-check`.
Forced library rebuild contains P3 cache symbols; ten-second animation smoke
records 53 steady draws and zero layout/history rebuilds. This old matrix is
excluded from candidate acceptance; full corrected matrix must be rerun.
