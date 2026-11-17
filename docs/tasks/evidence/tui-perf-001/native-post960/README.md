# Native Ghostty follow-up after #960

Maintainer confirmed idle input throughout the 30.023-second sample in `idle.json`.
Talos averaged 0.6995% of one CPU core; final PSS142,485KiB and swap0. Ghostty
averaged7.7275%, but includes other tabs/surfaces and is not attributable solely
to Talos. No child-process accounting or repeated controlled comparison is claimed.
Executable SHA256 and path are recorded in the sample. `source-snapshot.json`
hashes45 files at sampling time in the mixed primary tree; it is not build-time
attestation. Native display evidence comes from maintainer replies, not screenshots
or instrumented visible-pixel latency.

`acceptance.json` records external Git clean/dirty/clean and branch changes,
native workspace-switch replay, and display-only approval/reject observations.
The replay binary hash matches the prior git-idle-watch build receipt. Replay
workspace status changes do not establish an interactive CLI switching feature;
approval fixture acceptance does not establish production permission execution.

Second-terminal testing is explicitly waived by the maintainer, not passed.
Original479ms tail, strict native build/source binding and native Windows remain
open in the owner/#923. macOS native notifications remain unavailable.
