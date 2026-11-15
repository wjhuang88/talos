# Release high-rate paired comparison

Three alternating baseline/candidate pairs completed and pass all evaluated
CPU/PSS/input gates. History10000, configured stream200Hz, fixed-deadline input
probes50ms,35s including5s warmup. Baseline15ef42c1 has only the archived fixture
rate patch; TUI source was checked bytewise against that commit. Candidate
runtime df7411d2. Both Release binaries contain debug information; receipts here.

CPU medians34.657680% ->7.066650%; peak candidatePSS53965KiB. Worst candidate
inputP99 upper bound5.095588ms, within Release20ms budget (P95 also passes10ms).
Steady output events by round: baseline6009/6009/6010,candidate6010/6012/6011.
Probe skips allzero; total counts include post-window drain and are not steady
rates. Actual sampled window and process-tree limitations remain in raw reports.

No concurrent builds/tests/experiments launched during sampling. Fixed ordering
and main dependency changes can affect comparisons. This does not close native
pixel latency, allocation/longrun memory, cancellation or final-output correctness
(those have separate behavioral tests). Debug baseline timeout remains a failure;
this successful Release matrix does not replace it.
