# P4 versus P3 transcript commits — Release

Three alternating pairs, 10000 initial history entries, one complete message/s,
35s with5s warmup,500ms input probes,100x30 synthetic PTY, process-tree sampling.
Both source-bound binaries use e42b9c9c and identical replay fixture/dependencies;
baseline is P3 plus fixture, candidate adds the archived P4 source patch.
Receipts contain exact binary/source/patch hashes and Cargo artifact metadata.
The P4 patch is gzip-compressed unchanged; its receipt hash applies after decompression.

All evaluated budgets pass. CPU medians2.828760% ->0.067683%; candidate peak
PSS33532KiB. Candidate history projection refreshes cost0.232/0.276/0.291ms
per steady window; draw totals33.953/23.139/24.173ms. First baseline refresh
cost746.820ms vs draw907.459ms. Output counts94 on both sides per round.

Candidate worst input P99 upper10.413224ms in round1 (remaining rounds under
0.45ms), within20ms budget; this long tail is retained, cause undetermined.
CPU near procfs tick resolution, host drift and baseline-first ordering constrain
precision. Software input-submit is not pixel latency; no native acceptance.
This35s growing transcript does not establish long-run heap stability.

Reproduce scripts/replay_tui_perf_matrix.py using receipt-bound binaries:
--profile release-with-symbols --scenarios transcript-commit --histories 10000
--rounds 3 --seconds 35 --warmup 5
--process-tree --input-probe. Compare extracted raw reports with
scripts/compare_tui_perf.py. history_rebuild now includes incremental refreshes;
its count is not the number of full-history reconstructions.
