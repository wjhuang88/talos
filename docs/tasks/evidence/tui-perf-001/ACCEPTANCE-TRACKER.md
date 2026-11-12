# TUI-PERF-001 acceptance evidence tracker

This is a derived evidence checklist. The owner remains
[`2026-11-10-tui-render-performance.md`](../../2026-11-10-tui-render-performance.md).
No row below closes that owner or substitutes for missing platform evidence.
Measured runtime source: `539adfd2`; candidate build receipt: `candidate-build/receipt.json`.
Current implementation source: `43305a58` on base `6c81390a`. Each archive retains
its own source receipt; earlier measurements do not automatically cover later changes.

| Requirement | Current evidence | Remaining verification |
|---|---|---|
| P0 reproducible debug/release baseline | Baseline archives, SHA receipts, six scenarios × three histories × three rounds | Allocation profiles now archived for animation, transcript and P4; terminal emulator resources remain missing |
| Idle does not poll rendering | Paused-clock minute-idle loop test; release paired matrix passes zero steady frame checks across three rounds | Release and Debug metrics-off CPU/PSS passed; native terminal pending |
| Visible animation only | Tiny-window hide/restore test; release matrix passes zero layout/history rebuild and deadline P99 budget | Changed-cell bounds and shared-submission ANSI oracle passed; native display pending |
| Tip/reconnect/drag deadlines | Actual-loop paused-clock tests; drag stops after release | Native terminal behavior |
| Ordered output and cancellation | 1000-delta actual-loop test retains every token and submits final token; cancellation and <=4 draws. Input EOF/key exit fix `a91039be` and isolated four-case regression `18965ba2` passed the focused test | Final stable-batch gates and native cancellation remain separate |
| Async Git ownership | Controlled blocked worker crosses timeout and workspace switch; input/Exit responsive; global slot survives observer drop | Git fixture and Debug/Release switch matrices pass; new local actual-loop tests cover external clean/dirty/clean publication and successful obsolete scan suppression |
| Git unknown/freshness | Per-session snapshot tests, exact expiry, unchanged refresh, worker completion timestamp | Frame-weighted age distributions recorded for clean/large fixtures; external-change freshness and native status display remain |
| Cache correctness | Full-plan preview oracle and 28-step full-cell/style/cursor comparison; width/revision tests | ANSI equivalence and static preparation audit recorded in owner; native terminal remains |
| P4 decision | Repeated transcript commits identified full projection hotspot; P4 implemented in `cabc7a37`, Release comparison archived | P4 Debug transcript input passes; capped Debug resize samples exceed P95/P99 budgets. Rejected fitting-line shortcut is archived, not implemented. Final-source acceptance remains |
| Bounded memory | Single-entry caches, fixed-history125s×3 PSS, P4 allocation profiles,128-cycle obsolete-snapshot lifetime test | Growing-history/resize heap stability; finite PSS and ownership checks are not leak proofs |
| Input latency | Release and corrected Debug paired probes: 36 cases each; all P95/P99 budgets passed | 200Hz output/50ms input: candidate Debug and paired Release budgets pass; Debug baseline timed out; native visible-pixel acceptance remains |
| Build and tests | Implementation `6f558bf9` full preflight passed:5316 tests,0 failed/ignored; updated xwin MSVC tests/examples passed; final-preflight receipts and logs; independent increment review approved | Exact-head remote CI and final review remain. Prior failed attempts retained |
| Native platforms | User reports no available Windows environment | Linux Wayland+niri+Ghostty, another supported terminal, native Windows; unavailable stays Review |
| Delivery | Local candidate through `43305a58`; owner convergence table records current gaps | Final evidence/doc commits, stable candidate push/PR, review, owner/Board synchronization |

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
