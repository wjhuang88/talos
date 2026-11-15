# Debug non-repository workspace: acceptance not established

All six samples completed. The conservative animation deadline P99 check failed
in candidate round 2. CPU, peak PSS, stable layout/history, and input budgets pass.
CPU medians: baseline 1.346838%, candidate 1.277243%.

Round 2 has 213 steady deadline observations, four in the 33.554432–67.108863 ms
bucket and a measured maximum of 62.753716 ms. The P99 upper bound therefore
exceeds 50 ms; the exact P99 is unknown. This is insufficient to establish the
budget, not proof that the exact P99 exceeds 50 ms. Do not discard or replace
this run with a favorable retry.

Source inspection: app.rs records the minimum of animation, Git refresh/timeout,
stream, tip and reconnect deadlines into the same deadline_lateness metric.
This archive cannot identify the late event type or establish host scheduling
as the cause. Round 2 draw maximum is 22.722324 ms and Git scan maximum is
0.142383 ms; these aggregate maxima do not establish temporal causality.

Baseline source 52263c62, candidate source 539adfd2 (rebuilt Debug receipt in
../candidate-build/debug-rebuilt-receipt.json). These are pre-rebase binaries.
Three paired rounds, animation/history 10000, input probes and process-tree
sampling, 35 seconds including 5 seconds warmup. No concurrent experiments.
Follow-up: separate animation observations and collect bounded threshold counts
before attributing the failure or claiming acceptance on the rebased candidate.
