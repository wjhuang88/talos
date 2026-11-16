# Release workspace switching

All six cases pass evaluated CPU/PSS/input gates. Source-bound Release binaries
with debug symbols: baseline 15ef42c1, candidate df7411d2. Both receipts included.
Three baseline-first pairs, history 10000, 35s with 5s warmup, input probes and
process-tree sampling. Alternate large untracked and clean workspaces each second.
Candidate reports record 31 steady workspace-change events in each round.
CPU medians 1.509232% to 0.605813%; candidate peak PSS 40170 KiB.

Runtime reports confirm animation_deadline_lateness and over_50ms_count fields.
A literal byte search for the latter failed in the optimized binary; compiled
artifact provenance and actual JSON establish its presence, not that search.
No concurrent builds/tests/experiments were launched by this task. Baseline and
candidate differ in main dependencies after rebase; host drift also limits causal
attribution. This does not substitute for freshness/race/native acceptance.
