# Native idle checkpoint — 2026-11-17

User confirmed composer idle, then launched a separate fresh mock TUI in an empty
non-repository directory. Ghostty1.4.0-arch1, niri26.04(8ed0da4), Linux7.3.4-arch1-2.
Running executable SHA2565347488615ef9be965fa5a156dde541dca9eea098084698577d10941ed39c49c
matches the installed-candidate receipt for source b2a7d471, dev profile. This is
receipt-based provenance from the storage session, not a reconstructed build.
Window geometry and history size were not captured.

Continued real session: one30s CPU2.6667%, PSS213456KiB,swap0. Later separate
thread window totals approximately0.7%; Git watcher118.25ms, notify25.52ms,
Tokio worker45.36ms, main5.33ms runtime. Endpoint snapshots can miss transient
threads. These separate windows do not explain the earlier CPU result.

Fresh mock/no-context empty-workspace instance: three30s windows CPU0.1000%,
0.1667%,0.1667%, each below the frozen debug0.20% threshold. Final PSS140321,
140330,140334KiB;swap0. This proves only these windows meet the CPU threshold,
not zero draw/diff/flush, fixed-size memory/leak behavior, three independent
launches, baseline regression or long-history acceptance. History, provider and
workspace differ from the real session, so no Git-only causal conclusion follows.
Ghostty samples include other tabs; emulator CPU is not attributable to Talos.

Source-bound isolated watcher diagnostic: five reads produce0notifications;
five untracked ignored-file writes5; tracked ignored-file writes7; ordinary writes6.
Counts need not equal writes. Exact current watcher source was compiled with pinned
Rust against preserved dependency artifacts selected by Cargo fingerprints.
This is not a Cargo workspace validation or proof of live CPU causality. No
production filter change was made. See watch-decision.md for correctness risks.

Full owner remains Review/Claimed. Original479ms causal attribution, complete
native operation/source/environment records and native Windows terminal acceptance
remain open. Second terminal remains explicitly waived, not passed. No repeated
unchanged performance matrix or budget relaxation is authorized by these results.
