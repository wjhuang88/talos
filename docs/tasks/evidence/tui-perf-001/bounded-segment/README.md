# Bounded fitting-segment follow-up

Owner: TUI-PERF-001 / #923 / PR #929. Runtime correction committed locally as293ce0a2; not pushed or merged.

The independent old implementation (1eabf296) and candidate projection functions,
fill helper and data types were extracted verbatim into a standalone Rust harness.
The only renaming is baseline/candidate function names. Equality compares text,
styles, fills, background, synthetic padding and scalar offsets for widths
0/1/2/7/80/120 across six fixtures. Both before and after candidates passed.

Three alternating-order pairs per fixture run each function for at least150ms.
Single CPU affinity0, CPUQuota100%,MemoryHigh2GiB/Max3GiB,swap0. Prebuilt dependency
libraries and standalone opt-level0/3 are used; these are microbenchmarks, not
Cargo Debug/Release TUI resource acceptance or native terminal measurements.
No provider/network/user content is present. Exact generated source, commands,
receipts, raw samples and median summaries are archived with lossless gzip.

Original f725b74b extra width scan makes 32K combining marks followed by wrapped
ASCII about72% slower at opt0 and61% at opt3. The bounded candidate rejects the
fast-path probe when UTF8 byte length exceeds4*viewport width. This upper-bounds
speculative scanning even for zero-width text and retains the original fallback.
After correction this fixture is about1%/14% above the independent old baseline;
all measured fixture medians remain within20%, with normal short-fit benefit.
This does not substitute for the full active-scenario CPU/PSS regression gate.

43 history-projection tests passed (0failed/ignored), including a new32K-prefix
regression asserting exact wrapped text, style and scalar offsets. Service6.210s,
CPU4.382s,peak2GiB,swap0. Microbenchmark services before/after took12.015/11.846s,
CPU11.519/11.440s,peak361.3/548.9MiB,swap0. Earlier Release/Debug resize evidence
belongs to f725b74b, not this correction. Debug results are recorded below.

## CI source-binding diagnosis (separate unresolved blocker)

CI43341349202 is terminal: Windows workspace succeeded, macOS primary job failed
committed_source assertion, global reconciliation failed separately. An isolated
Linux harness using actual HistorySource source (only failure logging added)
passes at0/8/32/64 competing cooperative tasks in85/364/1062/1867ms. First three
workloads are in result-low-contention.log; main.rs was then changed to64 only
for result.log. No failure was reproduced and no macOS environment was used.
This establishes contention sensitivity, not the cause of CI failure. Production
budget and validation remain unchanged. A fixture instrumentation compile error
was corrected before runs; no runtime integration check is implied. Existing
prebuilt libraries are recorded by path; no final-source Cargo build is claimed.
Next: capture the failure reason on the actual failing platform before changing
production source binding. Do not blindly raise timeout or suppress the test.

Bounded-candidate Debug resize/input replay completed: all three valid rounds
pass150/200ms. Over150ms0/115,3/115,0/111; P99upper148.640179/155.596768/
149.782736ms. Actions58/58/57,probes61/61/60,zero skipped. Combined build and
measurement service115.754s,CPU20.923s,peak2GiB,swap0. Binary and patch hashes
in receipt.json.gz. Independent static review APPROVE; suggested byte-boundary
coverage added to existing segmentation test (test-only, runtime unchanged).
Release and local final-gate results follow.

Boundary fixtures also passed in the43-test rerun:3.428s wall,2.896s CPU,peak1.4GiB,swap0. Release results are recorded below.

Final correction293ce0a2 Release replay passed all three100/150ms budget rounds:
P95/P99 upper32.705288/29.491502/29.130585ms. Each round61 keyboard probes,
zero skipped; coverage and actual resize validation passed. Build43.97s; combined
build+measurement service156.117s,CPU46.274s,peak2GiB,swap0. Same fixed guard.
These results close only synthetic resize/input timing for this correction.
Final preflight and xwin results follow; CI failure, active-workload comparisons,
final-source heap attribution and native/platform residuals remain open.

Final-source standard preflight exit0, 5320 tests passed,0failed/ignored
across148 summaries,1118.070s. Includes formatting, governance, locked
workspace check/Clippy/test and external SDK fixture. MSVC cargo-xwin --tests exit0,
1.542s. Combined1119.686s wall,561.342s CPU,peak2GiB,swap0. Prior failed preflight
and macOS CI remain historical failures; this local pass does not erase them.

### Interleaved transcript workload follow-up

Twelve valid35s samples (5s warmup), previousf725b74b versus bounded293ce0a2,
three alternating pairs each Debug/Release,10,000 history,one committed message/s,
500ms input probes. CPU median ratios0.881Debug/0.667Release pass<=1.20;
candidate peak PSS38,787/33,130KiB passes baseline*1.20+8MiB. Low Release CPU
uses coarse procfs ticks; this is regression evidence, not a precise speedup.
Release input all passes10/20ms, worstP99upper0.489082ms. Debug first bounded
round FAILS35ms P99:479.239045ms; later rounds3.422221/3.461222ms pass. No probes
skipped. Validated coverage does not erase the failed latency budget.
Frame-preparation max476.541768ms while history-rebuild max0.065496ms; IO pressure
observed, but causality remains unproven. See long-tail-diagnosis.md.gz; further
unchanged matrices are not the next diagnostic. Runtime source remains unchanged.
Service449.876s,CPU10.448s,peak810.4MiB (whole measurement service),swap0.
Final-source heap profiling is separate; profiler latency is not acceptance.

Final-source heap attempt exited1 at resize coverage validation:571 driver actions,
569 actual steady resize/rebuild events (required>=570),300.172s steady duration.
Replay itself exited0. The sample is retained for limited diagnostic analysis,
not passed full coverage. Transcript profile was not started. Service309.409s,
CPU126.427s,peak624.4MiB,swap0. No repeat or relaxed threshold was applied.

CI source-binding diagnosis adds test-only failure metadata and distinguishes
not-attempted from refused binding. Independent review approves privacy and cfg
boundary; diagnostic test results follow. Production2s budget and
return behavior unchanged. Expiry observed after refusal alone is not proof of
which internal branch returnedNone. Required macOS failure remains unresolved.

Diagnostic commit34328646 passed the exact long-history Agent test (1pass,
0fail/ignored; test0.68s,build+test192.928s). Independent review APPROVE; production
code remains unaffected by cfg(test). Prior5320-test preflight precedes this test-only
change, so final-head remote validation is still required. This is not a macOS fix.

Existing failed-coverage resize heap analysis completed: all nine30s windows
(t30..300) have identical median25,883,809bytes,delta0; peak26,159,584bytes.
569 actual resize/rebuild events were observed. This supports finite fixed-history
heap stability only, while coverage remains failed. Raw profile and detailed massif
are retained. Analysis5.410s,CPU5.373s,peak62.7MiB,swap0. The previously unexecuted
transcript heap scenario ran independently with a separate manifest.

Final-source transcript heap profile passed coverage:301 actual steady history
updates,300.012s. Nine30s heap-median windows increase by exactly25,020bytes/window,
about834bytes per fixture message; first-to-last delta200,160bytes. No accelerating
growth observed in this finite run. Service308.391s,CPU12.553s,peak488.6MiB
(includes profiler),swap0. Detailed massif tree children incorrectly expose zero
bytes despite nonzero totals, so object/category attribution is NOT established.
Raw profile, text stacks, totals and exporter limitation are archived. All local
heavy jobs are terminal; no extra unchanged timing matrices are scheduled.

Remaining blockers: Debug transcript input long-tail479ms not diagnosed; previous
macOS CI source binding failed (test-only diagnostic34328646 now available);
resize heap coverage mismatch569/571; heap category attribution; exact native
resources/deferred operations and second terminal/native Windows. Task staysReview.
