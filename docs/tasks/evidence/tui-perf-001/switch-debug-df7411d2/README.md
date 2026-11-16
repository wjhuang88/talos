# Debug workspace switching

All six cases pass evaluated CPU/PSS/input gates. Baseline 15ef42c1 (P0 with
workspace-switch fixture), candidate df7411d2; both source receipts included.
Three baseline-first pairs, 35s including 5s warmup, history 10000, input probes
and process-tree sampling. Switch every second between the 10000-untracked
fixture and the small clean fixture. CPU medians 6.765962% to 3.488965%; peak
candidate PSS 61084 KiB. Both profiles' native pixel behavior, external Git
mutation freshness and successful stale-completion races remain separate gates.
No builds/tests/other experiments launched by this task during sampling.
Dependency changes and host drift limit causal attribution across the rebase.

Observed steady workspace-change counters:
{
  "baseline-workspace-switch-10000": [
    31,
    31,
    31
  ],
  "candidate-workspace-switch-10000": [
    31,
    31,
    31
  ]
}
