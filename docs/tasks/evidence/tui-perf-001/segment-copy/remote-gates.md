# Remote gate observation

Candidate cef024a11d02607afe86da73682f2fe66f71b9b5, PR929, CI43341349202.
Remote reconciliation job131610165834 failed because matrix still lists closed
search Issue901. Current main f2e30d99 contains search closeout; this is unrelated
matrix drift, advisory for implementation per ADR-071. Owner is global issue matrix
workflow, not a waived performance/native gate. Linux/Windows code jobs still running.
Raw job log: /tmp/talos-perf-929-reconciliation.log. Do not rebase/retest solely
for this unrelated metadata issue. Recheck target/claim/integration before merge.

Update: primary code job131610217702 FAILED, actual runner macOS (despite earlier
informal Linux label). Checkout cef024a1 exact head. Agent test
cooperative_recovered_sources_bind_long_history_and_refuse_foreign_or_expired
assertion committed_source Some(Some(_)) failed at session/tests.rs7185;
672 other Agent tests passed. This is a required code gate blocker, not advisory
matrix drift. Raw log /tmp/talos-perf-929-linux.log. Independent failure review
requested; no rerun or unrelated source patch yet. Windows workspace still running.

Independent read-only failure review: likely completed-turn binding deadline (2s)
for 8,300 entries; recovery in the same test uses 10s. Per-entry cooperative
scans may exhaust the production deadline under macOS Debug contention. Logs do
not identify the None branch, so this remains a hypothesis, not a proven root
cause. Reviewed recovery/storage branches do not change this path. Next action:
add failure-reason/timing diagnostics and reproduce before changing scheduling;
do not merely increase timeout, skip assertion or relabel this gate as advisory.
No Agent/session runtime source was changed in this performance follow-up.
