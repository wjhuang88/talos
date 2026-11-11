# TUI-PERF-001 acceptance evidence tracker

This is a derived evidence checklist. The owner remains
[`2026-11-10-tui-render-performance.md`](../../2026-11-10-tui-render-performance.md).
No row below closes that owner or substitutes for missing platform evidence.
Measured runtime source: `539adfd2`; candidate build receipt: `candidate-build/receipt.json`.
Current source: `df7411d2` on main `55617ad7`; old measurements are not
new-baseline acceptance. Six rebased patches are unchanged according to range-diff.

| Requirement | Current evidence | Remaining verification |
|---|---|---|
| P0 reproducible debug/release baseline | Baseline archives, SHA receipts, six scenarios × three histories × three rounds | Allocation volume and terminal emulator resources; release switch baseline now source-bound in switch-release-df7411d2 |
| Idle does not poll rendering | Paused-clock minute-idle loop test; release paired matrix passes zero steady frame checks across three rounds | Release and Debug metrics-off CPU/PSS passed; native terminal pending |
| Visible animation only | Tiny-window hide/restore test; release matrix passes zero layout/history rebuild and deadline P99 budget | Changed-cell/ANSI coverage |
| Tip/reconnect/drag deadlines | Actual-loop paused-clock tests; drag stops after release | Native terminal behavior |
| Ordered output and cancellation | 1000-delta actual-loop test retains every token and submits final token; cancellation and <=4 draws | High-rate candidate Debug absolute input and Release paired gates pass; native cancellation remains separate |
| Async Git ownership | Controlled blocked worker crosses timeout and workspace switch; input/Exit responsive; global slot survives observer drop | Git fixture and Debug/Release switch matrices pass; new local actual-loop tests cover external clean/dirty/clean publication and successful obsolete scan suppression |
| Git unknown/freshness | Per-session snapshot tests, exact expiry, unchanged refresh, worker completion timestamp | Frame-weighted age distributions recorded for clean/large fixtures; external-change freshness and native status display remain |
| Cache correctness | Full-plan preview oracle and 28-step full-cell/style/cursor comparison; width/revision tests | Native terminal/ANSI equivalence; inspect remaining static preparation after measurements |
| P4 decision | Baseline resize dominated by width reflow; stream fixture has few transcript commits | Candidate comparison and repeated transcript-commit evidence before deciding |
| Bounded memory | Single-entry caches and one actual scan slot by construction | Long repeated replay/PSS stability; allocation evidence |
| Input latency | Release and corrected Debug paired probes: 36 cases each; all P95/P99 budgets passed | 200Hz output/50ms input: candidate Debug and paired Release budgets pass; Debug baseline timed out; native visible-pixel acceptance remains |
| Build and tests | Full locked preflight passed after local behavior-oracle additions; 677 TUI unit tests, integration/doc tests; xwin MSVC tests/examples passed | Fresh exact-head CI and prescribed independent review |
| Native platforms | User reports no available Windows environment | Linux Wayland+niri+Ghostty, another supported terminal, native Windows; unavailable stays Review |
| Delivery | Rebased P1 `9065db99`, P2 `3835fbd4`, P3 `5b25d891`; diagnostics `df7411d2` | Final evidence/doc commits, stable candidate push/PR, review, owner/Board synchronization |

The paired release experiment completed all 108 cases; validated reports are in
`release-paired/`. The equivalent debug matrix completed but failed its stable-animation gate;
the captured candidate was confirmed to contain stale P2 code from the shared
target directory. Raw reports remain in `debug-paired/` for audit and are excluded
from P3 acceptance. A forced current-library rebuild contains the P3 cache symbols;
a short animation smoke records 53 steady draws with zero layout/history rebuilds.
The corrected full matrix passed all evaluated checks; archive: `debug-paired-rebuilt/`.
Its raw experiment directory is
`/tmp/talos-perf-debug-paired-rebuilt-539adfd2`, with provenance in
`candidate-build/debug-rebuilt-receipt.json`. Its default stream rate is 20 Hz, below
the 33 ms display-coalescing ceiling. Consequently similar stream draw counts
are expected; measure stable preparation work separately from frame suppression.
High-rate burst tests cover a different workload and must not be substituted for
this matching baseline/candidate comparison.

Formal budget evaluation must report missing samples as unavailable or failed,
never as zero latency. Stage durations are nested wall-clock observations, not
exclusive CPU attribution. PSS is not allocation volume. Synthetic PTY completion
is not native visible-pixel acceptance. Keep all these limits in the final report.

Pre-rebase non-repository Debug experiment: `git-nonrepo-debug/` preserves all six
samples. Its conservative deadline P99 gate fails in round 2; exact P99 and
deadline type are unknown. Animation acceptance remains open. See archive README.

New-version non-repository Debug matrix `nonrepo-deadline-df7411d2/` passes its
evaluated checks, including 602 specific animation observations with none over
50 ms. The old failure remains unexplained and preserved. Full preflight and
xwin MSVC tests/examples passed for the diagnostic refinement; remote/native
and the remaining workload gates are still open.

Debug and Release workspace-switch matrices now pass evaluated CPU/PSS/input
gates; archives: `switch-debug-df7411d2/` and `switch-release-df7411d2/`. Exact
pre-rebase source commits remain recoverable through `source-history/pre-rebase.bundle`.
