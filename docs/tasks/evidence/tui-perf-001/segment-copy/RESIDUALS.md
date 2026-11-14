# Remaining acceptance after fitting-segment correction

Owner: [TUI-PERF-001](../../../2026-11-10-tui-render-performance.md); tracker #923.
This ledger preserves original acceptance; it does not replace the owner.

| Requirement | Current evidence | Remaining action |
|---|---|---|
| Debug resize input P95/P99 | Three fixed-patch rounds pass150/200ms | Synthetic gate satisfied; native display evidence remains separate |
| Release resize and other-workload regression | Candidate three Release resize/input rounds pass100/150ms | Bounded293ce0a2 segment and Release workload checks pass; Debug transcript479ms tail remains unresolved |
| Fixed-history live heap | Corrected sampler570/570 effective changes; nine heap windows constant | Finite allocation-path evidence independently reviewed; preserve limits, no universal leak-freedom claim |
| Native Ghostty operations | Maintainer-reported basic passes | Exact binary/environment, resource observations and deferred manual operations |
| Second supported terminal | No execution evidence | Native operation/resource record |
| Native Windows | User has no environment | Keep Review until native execution; xwin/CI are not substitutes |
| Full final-source checks | Check/Clippy/governance/xwin passed; initial tests failed one cleanup race, isolated rerun passed | Final-head4d9e5cfc code CI43360750041 passes; preserve prior failures and unknown macOS failure cause |
| Publication | PR #929 merged as a8dc7fb0; exact candidate4d9e5cfc | Stage delivered; owner/#923 remain Review for original tail and native gaps |

Do not repeat existing timing matrices without a new diagnostic question. Preserve
single-core/memory limits and serial heavy jobs. Do not mark whole task Complete
while any required row remains unverified.
