# P4 fixed-history repeated replay

Three candidate-only125s runs,5s warmup,10000 fixed history entries, visible
animation and500ms type/erase probes,120x40 PTY. No history growth, Git workspace,
builds, tests or concurrent experiments. Candidate source/binary is the same P4
receipt as ../transcript-p4-release/ (source patch stored there).

First/last30s median PSS deltas:-3165/-1/+1KiB. No sustained upward PSS trend
observed in these finite runs; PSS depends on shared-page accounting and is not
live-heap evidence or proof of no leak. Raw samples and window calculation are
retained in raw-reports.tar.gz and memory-windows.json.

CPU0.2504/0.2593/0.2590%; no paired CPU improvement claim. Worst input P99 upper
0.503696ms; zero history rebuild/layout/Git scan during each steady window.
Animation deadlines799/800/800 observations, with5/3/1 over50ms respectively:
P99 meets budget, but not every deadline. Do not discard these long tails.

Reproduce scripts/replay_tui_perf_matrix.py with the receipt-bound candidate,
--profile release-with-symbols --scenarios animation --histories 10000
--rounds 3 --seconds 125 --warmup 5 --process-tree --input-probe.
Bounded heap/cache evidence under resizing, tool grouping, growing history and
native terminals remains separate. Synthetic submit latency is not pixel latency.
