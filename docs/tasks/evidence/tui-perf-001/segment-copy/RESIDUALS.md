# Remaining acceptance after fitting-segment correction

Owner: [TUI-PERF-001](../../../2026-11-10-tui-render-performance.md); tracker #923.
This ledger preserves original acceptance; it does not replace the owner.

| Requirement | Current evidence | Remaining action |
|---|---|---|
| Debug resize input P95/P99 | Three fixed-patch rounds pass150/200ms | Bind final source and delivery gates |
| Release resize and other-workload regression | Candidate three Release resize/input rounds pass100/150ms | Wrapped/zero-width long-segment and other-workload regression |
| Fixed-history live heap | Five-minute resize medians constant before patch | Review applicability after change; cache occupancy and growing-history attribution |
| Native Ghostty operations | Maintainer-reported basic passes | Exact binary/environment, resource observations and deferred manual operations |
| Second supported terminal | No execution evidence | Native operation/resource record |
| Native Windows | User has no environment | Keep Review until native execution; xwin/CI are not substitutes |
| Full final-source checks | Check/Clippy/governance/xwin passed; initial tests failed one cleanup race, isolated rerun passed | Workspace rerun and SDK passed; preserve initial failure; macOS source-binding test failed; diagnose before merge |
| Publication | Pushed PR #929 at cef024a1; independent review approved | Resolve required CI blocker, review next stable candidate and authorized merge |

Do not repeat existing timing matrices without a new diagnostic question. Preserve
single-core/memory limits and serial heavy jobs. Do not mark whole task Complete
while any required row remains unverified.
