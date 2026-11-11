# Repeated transcript commits — Release

Three alternating baseline/candidate pairs, 10000 initial history entries,
one complete short Unicode assistant message per second, 35s with 5s warmup,
500ms input probes, synthetic 120x40 PTY. See raw manifest for exact environment.
Both binaries use the identical fixture; baseline runtime is bytewise verified
against 15ef42c1, candidate e42b9c9c plus the archived fixture-only patch.
Build receipts include binary hashes and Cargo artifact/profile records.

All evaluated CPU/PSS/input budgets pass. CPU medians 5.099875% -> 2.791949%;
candidate peak PSS 68918 KiB; worst candidate input P99 upper 594405 ns.
Candidate each round: 94 steady output events, 31 history rebuilds; baseline
62 rebuilds. Candidate history rebuild totals 717.08/601.81/955.92 ms versus
whole draw totals 880.17/743.41/1159.89 ms (nested timers, not additive).
History rebuild represents about 81–82% of draw duration in this workload.
This supports evaluating P4; passing aggregate budgets does not remove the hotspot.

Reproduce with scripts/replay_tui_perf_matrix.py, the two receipt-bound binaries,
--profile release-with-symbols --scenarios transcript-commit --histories 10000
--rounds 3 --seconds 35 --warmup 5 --process-tree --input-probe.
Run scripts/compare_tui_perf.py on extracted raw reports.

Limitations: fixed baseline-first ordering and dependency/host drift limit causal
attribution. Software submit latency is not pixel latency. A 35s growing transcript
is not bounded long-run memory evidence. No allocation or native-terminal claim.
