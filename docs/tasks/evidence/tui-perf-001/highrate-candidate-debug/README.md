# Candidate-only high-rate Debug validation

Candidate df7411d2, 200Hz configured stream, history10000, input probes at50ms,
35s nominal fixture with5s warmup, three rounds. Fixed probe deadlines skip rather
than burst overdue probes and explicitly count skips. All runs completed; steady
output event counts6034/6028/6033. Input P95/P99 upper bounds by round:
15.935096/12.489444/12.259310ms; all satisfy Debug20/35ms budgets.
CPU27.395888/24.934457/29.201506%; peakPSS61242/61358/59858KiB.
Total injected probes691/684/711 with zero skipped deadlines; counts include
post-window drain time and must not be called steady-window rates. Software
input latency measures event reception to submission, not OS queue or pixel time.

Baseline200Hz Debug did not complete before65s watchdog (separate preserved
failure). No paired CPU/PSS acceptance or improvement ratio can be claimed.
This candidate-only evidence is additional to normal20Hz paired matrices.
Two prior candidate samples completed, but the third report hit/tmp quota;
that incomplete experiment remains separately archived. Binary copies were
moved to system disk with bytewise hashes preserved and original symlink paths.
No concurrent builds/tests/other experiments launched during final sampling.
Final token/cancellation behavior is established by separate actual-loop tests,
not inferred from this process exit. Native and longrun/allocation gates remain.
