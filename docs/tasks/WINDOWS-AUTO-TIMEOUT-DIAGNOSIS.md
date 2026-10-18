# Windows Auto Timeout: Bounded CI Diagnosis

Date: 2026-10-11. Baseline: `444a8137f054b33802f26bffdb55fb0f83a7a861`.

Current stage (2026-10-16): add bounded event600 provider-lifecycle observations before engine availability; native execution pending.
Earlier matrices, pending statements and tool availability below are preserved staged history;
the latest checkpoint defines the current fixed 120-process / 2220-test experiment.

## Authorization And Scope

The maintainer requested deeper GitHub CI diagnosis of the intermittent Windows
Desktop Auto allow-once timeout. This single-PR experiment follows the bounded
CI/fixture maintenance exception in `docs/sop/AGENT-COLLABORATION.md`.
It does not change production behavior, permission authority, public APIs,
dependencies, release authorization or any existing owner delivery state.
I290/I291/AUTO-UX-001 and Issue #590 stay Complete/Closed. I296's historical
variance evidence is preserved. No overlapping open PR was found at intake.

The experiment has one deliverable: exact-head baseline/instrumented Windows
logs and a bounded serial/parallel comparison. It makes no root-cause repair or
product completion claim. If the evidence requires production changes or a
multi-PR implementation, establish a separately governed claim before that work.

## Existing Evidence And Unknowns

- CI3109/run `38032860943`, job `114157265192`, head `66d420f3`:
  AutoDecision 577.9406 ms; tool request 584.5438 ms; no tool result before
  the unchanged 15-second test watchdog. 105 tests passed, one failed.
- Same-head full-job rerun `114164241652`: 106 Desktop tests passed in 5.27 s.
  Both used Windows image `20260925.250.1`. Retry success does not prove repair.
- ToolStarted projects a request, not proof of process creation. Windows
  foreground PowerShell uses the Job Object launcher; completion waits for
  process exit plus both pipe readers. Command timeout starts after launch and
  defaults to 120 seconds, independently of the 15-second fixture watchdog.
- PowerShell startup, process lifetime, pipe EOF, blocking reader scheduling and
  event delivery remain candidates, not established causes.

## Experiment And Acceptance

Use the existing `ci.yml`; no one-off workflow. An independent opt-in Windows job
checks out the exact PR head. Mandatory workspace/feature/security checks retain
their original source, commands and gates on separate runners.

Run the same Desktop test executable in fresh processes: five isolated allow-once
runs, three full-suite serial runs and five full-suite 16-thread runs. Then repeat
that matrix with temporary stderr probes. Record process creation/assignment/
resume, wait entry/return, each pipe read/EOF, joined readers and test heartbeat/
outcome (heartbeat observes the test executor, not proof that the separate host
executor is live). No environment dump, credential, script-content or tool-output logging
is added by probes. Existing tests may print their own synthetic failure data.

All attempts are retained; a failed attempt remains a failing job even if later
attempts pass. There are no ignored tests, softened assertions, deadline changes
or retry-to-green. An outer 90-second process watchdog stops a stalled experiment
and terminates only its exact owned test PID/tree. Existing 15-second flow and
10-second cleanup watchdogs are unchanged. Temporary source probes are restored
byte-for-byte in `finally` and their build cache is not saved. A 35-minute total
script budget leaves five minutes before the enclosing CI step deadline for
restoration and artifact handling; insufficient budget fails explicitly. Cleanup
errors are retained with the timed-out attempt. The exact temporary overlay is
included in the evidence artifact.

Baseline/instrumented comparisons can reveal observer effects; a green matrix
does not prove absence of a low-frequency race. An inconclusive run must report
its limits rather than claim reliability. Windows execution remains pending.

## Local Validation

Probe self-test, all 14 classifier tests, Python syntax, workflow YAML and diff
checks pass. Collaboration Claim validator passes with zero warnings.
Full local governance stops at SQLite consumer metadata because Cargo is absent
after workspace maintenance; delivery fixtures pass (12 cases), and no check is
waived. The unchanged full CI route supplies pinned-Cargo governance validation.
Independent static pre-review requested bounded-budget/cleanup evidence fixes;
both have been incorporated before remote submission.
Local Windows execution is unavailable; real evidence must come from Actions.

## First CI Evidence And Controlled Follow-up

Exact head `a7618651d9bc669bdcfe705718c6e74ab6254d77`, CI3132/run
`38134743253`, diagnostic job `114456206963`: all 26 attempts succeed (1706
test executions), same Windows image `20260925.250.1` as the historical failure.
The instrumented Auto result arrives in 0.335263–3.0302286 seconds. Win32 creation
is 1.8928–150.1078 ms; assignment/resume to process wait return is
238.6799–1125.4849 ms; wait return to both readers joined is 0.2194–1.1384 ms.
These use in-process SystemTime intervals (100 ns), not Actions timestamps:
the runner script replays log lines after each test process finishes.

No historical timeout was reproduced and no root cause is established. Evidence
artifact `11700882437` retains all 31 files until 2026-10-25, SHA256
`dfb0076a547c00ac9bb690ec975d176697e2cfd2208c53111edd533b9c1ad61b`.
Public checkpoint: PR #705 comment `6108531033`. Agent exact-head review
`6108425990` approved the initial diagnostic design, not a production repair.

The next stable diagnostic candidate adds two runs of a temporary Windows tools
fixture: a real PowerShell leader starts an owned child in the same window and
exits; the child emits readiness then holds inherited output handles. The
existing launcher must stay pending for a bounded observation window. Explicit
acceptance also requires the native leader wait to report WAIT_OBJECT_0 and
exit code zero before the cleanup-begin probe. Missing that observation fails
the diagnostic as inconclusive even if the Rust fixture itself passes. Explicit
termination of its exact Job Object must unblock pipe readers and return a
terminal event within ten seconds. Cleanup always precedes assertions.
No new unsafe site, environment mutation, persistent Rust patch, permission
change or original-test deadline change is introduced. This synthetic fixture
tests the EOF-wait/cleanup mechanism only; it cannot establish that the original
incident involved an inherited writer. Its compilation/execution is pending CI.

## Second CI Evidence And Harness Correction

Head `9f66e76d158cc6eee0b2fee09f1022d32aad1f67`, CI3138/run
`38136080740`, job `114460107420`: all 26 original attempts pass again.
Both controlled invocations run zero tests (40 filtered); the script rejects
them with diagnostic 125 and the job fails. This is a harness feature-selection
error, not an observed pipe/cleanup failure: `talos-tools` defaults to file-read
and search, while `process_boundary` requires the explicit `shell` feature.
Artifact `11701690661`, SHA256
`c068ec2003e89e9967b0fb00d8757d3925aaa1396ff6204e3cef6880249ad278`,
expires 2026-10-25. Failed head and skipped fixture evidence remain retained;
checkpoint comment `6108702060`. No historical root cause or repair is proven.

The corrected tools build explicitly enables `shell`; an exact test-inventory
check must identify one test before either controlled attempt runs. The latest
opt-in CI invocation uses `--controlled-only`: two fresh controlled processes,
not another 26-case matrix. The manifest records this mode; prior matrices on
the two earlier heads supply 52 processes/3412 original test executions, not
new-head matrix coverage. The default script invocation still runs the full
matrix. Production Rust source is unchanged across these candidates and normal
mandatory full CI continues independently. Corrected Windows execution is pending.

## Controlled Result And Next Original-Test Capture

CI3140/run `38137043755`, exact head `01cf6bac86569c3cd1a82fc524a5c89e16e9ecef`:
all seven jobs SUCCESS. Exact fixture inventory and two actual executions pass.
Leader native wait returns exit zero before descendant readiness; completion stays
pending until owned-Job cleanup. Cleanup-to-both-pipes-joined is 2.3380/2.0474 ms
from native 100 ns SystemTime intervals. Independent raw-result review agrees.
Artifact `11701707026`, SHA256
`fd83bd6d23a36c289d6ed369d212ff046e281f4d3e0136cec508f04047b2e691`,
expires 2026-10-25; durable checkpoint comment `6108803418`.
This proves a synthetic mechanism, not the historical `pwd` timeout.

The next optional invocation uses `--stress`: per baseline/instrumented phase,
50 fresh isolated original allow-once runs and ten full-suite runs at 32 test
threads. Planned total: 120 test processes, 2220 test executions if each full
suite retains 106 tests. Actual counts and every failure must be reported;
budget exhaustion means incomplete, never success. This phase skips synthetic
fixture execution because its mechanism is already characterized.

A temporary host-runtime heartbeat is added inside the existing executor's
async root, with native time, PID and host thread ID matching launcher probes.
It runs every second and ends when that executor shuts down; it has no command,
workspace, environment or output contents. Continued test heartbeat alone cannot
prove host progress. Lost host ticks while test ticks continue can support host
blocking/scheduling investigation, but cannot by itself establish a cause.
The heartbeat adds an observer effect, so the unchanged baseline remains required.
All three original Rust files are restored byte-for-byte. Production Rust remains
unchanged in Git; original deadlines, total budget and mandatory gates remain.
Main advanced to `7e817ab` without changing the existing boundary/Auto-test files;
no stale-main integration or root-cause completion is claimed.

## Stress Result And Fresh-Runner First-Suite Comparison

CI3143/run `38193977394`, head `71b0c85cd63c99895d27f1a55d5f608217618534`,
job `114632522478`: all 120 planned processes/2220 actual executions pass.
Artifact `11722209825` (125 files), SHA256
`048ab54c5a2a8206a8d52d12c6db15b54c382a1902f6817728aa00d06625bd9c`,
expires 2026-10-26; checkpoint comment `6116510438`.
All 60 instrumented cases have exactly one launcher and matching host ticks.
Request-to-result is 0.1969686–1.1551254 s; CreateProcess 1.6095–69.7418 ms;
wait-return-to-readers-joined 0.1722–0.3896 ms. Max host tick gap is 3155.7716 ms
in a successful case, with Auto decision/request at 3.1884095/3.2478974 s and
result at 3.8599155 s. This does not match the historical early request followed
by a long post-request stall. No root cause or repair is established.

The next fixed two-entry job matrix isolates baseline and instrumented phases
on separate fresh Windows jobs. Each invokes `--fresh-phase`: first execute the
full 32-thread suite, then 50 isolated original calls and nine more full suites
(60 processes/1110 expected executions per phase if suite106). Instrumented
sources are overlaid before its first build; baseline never writes probes.
This removes prior Talos-test warmup before the first full-suite call. It does
not guarantee OS/Defender/cache coldness; setup actions and VM variability remain.
Manifest records phase and CPU count. Compare first-suite separately from later
calls; cross-runner differences alone do not prove a probe effect or root cause.
Matrix fail-fast is disabled to retain both outcomes; artifact names include
phase to avoid overwriting. Each runner retains its own unchanged 35/40/45-minute
limits, original assertions, source restoration and owned-process cleanup.
Only the optional experiment becomes two jobs; normal mandatory gates are untouched.

## Actual Failure And Native Wait Status Follow-up

CI3145/run `38195009504`, head `d9b7849cf8597dfea487d51608834395b9236cf0`:
both diagnostic jobs FAIL, all120processes complete, 2188passed/32failed/0ignored
test executions (2220 total). Baseline later parallel5/8 fail13/3 host tests;
instrumented first suite fails the original Auto watchdog, later parallel8/9
fail14/1 tests including broader host startup/shutdown paths. Neither phase's
50 isolated calls fails. Later passes do not replace these failures.

First instrumented failure: AutoDecision2.6739847s/request2.7748133s,
allow_once/configured-model, no ToolResult by15s. Unique child8252, testPID3864,
hostThread62/handle712; CreateProcess2.0962ms, assignment/resume proven. Native
waiter147 enters, but completion is not observed before watchdog; matching host
ticks continue through14s (postlaunch max gap1012.7936ms). This rules out a
continuous whole-host stall for this sample, not waiter scheduling delays.
Wait return0/exit0 and stderrEOF appear only after watchdog/cancellation; no
stdout return or joined readers before then. Do not infer natural pre-cleanup
exit or label all broader failures PowerShell. Specific underlying cause remains
unproven; independent raw-evidence review agrees. Checkpoint `6116704602`.
Artifacts (64 files each, expires2026-10-26): baseline11723446776 SHA256
`b512cbe8c75fcb483193c763d242a5074a027e47c55616b6984165345911993d`;
instrumented11723317188 SHA256
`e3473a3fb26b52d7ef26c361ee5f1377369db727ff5c48230ad3268ca9b261c1`.

The next temporary probe replaces the infinite native wait inside the existing
unsafe waiter with one-second waits. WAIT_TIMEOUT continues waiting; any other
result follows original terminal handling. Each observation records native time,
wait result and GetExitCodeProcess success/code using the same owned handle.
GetProcessTimes success and cumulative kernel/user CPU ticks (100 ns) are also
recorded; failed queries must not be treated as zero CPU activity. No memory,
command line, environment, process enumeration or stack contents are read.
STILL_ACTIVE alone is not enough; pair it with the wait status. No poll causes
termination or changes the existing command/fixture deadlines. This is an
observer change, so the separate original baseline remains required.

Also mark process-handle and owned-Job close entry/return. The legacy
`process_wait_return` was emitted after final exit query and handle close;
`native_wait_observed` now separates the actual API observation from that work.
These probes can distinguish unsignaled child, waiter/log scheduling and handle
release. Same handles/close sites and existing unsafe regions; no new process,
privilege, ownership or unsafe block. Production Git Rust remains unchanged.
Fresh two-runner order, bounded counts/budgets, source restoration and failure
retention stay intact. Corrected probe compilation/execution remains pending CI.

Primary Win32 semantics references: [WaitForSingleObject](https://learn.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-waitforsingleobject),
[GetExitCodeProcess](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getexitcodeprocess),
[GetProcessTimes](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getprocesstimes).

## Actual Child-State Evidence And Host Preparation Follow-up

CI3147/run `38196854374`, head `837fefa15f4892d693fb89e956630d26c9695733`:
all six ordinary mandatory gates SUCCESS. Baseline60processes/1110tests PASS;
instrumented60processes/1096passed14failed, failures only in first full suite.
Combined2206passed14failed0ignored/2220 executions. Both50isolated-call samples
and later nine suites pass; this does not replace the first-suite failure.

Original Auto watchdog: AutoDecision10.6479664s/request10.7125676s, no result
by15s. TestPID6636/hostThread69/child1200/handle596/waiterThread145;
CreateProcess2.8189ms, assigned/resumed. Matching host's initial heartbeat gap
is about10.6s before launch; postlaunch ticks continue about every second.
Four native wait polls return258 with successful exit queries259 before
cancellation. All CPU queries succeed; cumulative kernel+user CPU progresses
.0625/.09375/.1875/.453125 seconds. The child is unsignaled/still-active at
those sampled times and the waiter runs; this does not identify IO, Defender,
PowerShell initialization or the preparation-delay cause. All65 CPU queries
across this instrumented matrix succeed, validating the new API compilation
and execution. Earlier probe-compilation-pending text is now superseded.

After watchdog, Job close takes.2163ms; pipes reach EOF; wait reports0/exit0;
process handle close takes.3089ms. Cleanup exit0 is not natural-exit proof.
This sample does not show a continuous waiter/close stall or an observed
leader-exited-before-EOF hold. Its late request differs from the historical
early-request timeout, so do not assume all samples share one cause. Other13
host startup/write/cancellation failures are not all PowerShell failures.
Independent raw-artifact review confirms these limits. Checkpoint6117073163.
Artifacts (64 files each, expires2026-10-26): baseline11724360591 SHA256
`e48748189bc0792aafe467a7bd8bfbf3c7c5d61c5ca097c0fa0bcd967fa96a66`;
instrumented11723941599 SHA256
`9185fec26584e95bef20d2f246df50bc5e4cf5c11a0cde1380425c9d87f27449`.

Next temporary metadata-only host probes mark task entry, provider entry/return,
session setup, builder setup/build, initial work projection, command-loop ready,
Submit receive and submit entry/return. Native time/PID/thread allow correlation
to the existing launcher without logging requests, workspace/session identifiers,
messages or provider data. Brackets preserve the original evaluation order and
error paths. They measure coarse preparation spans, not every provider/runtime
internal operation; a fast Submit return cannot prove subsequent planning fast.
No new task, await, process, timeout or shutdown policy is added. The original
baseline, fixed two fresh runners, counts/budgets, retained failures, byte-for-byte
restoration and mandatory gates remain unchanged. New bracket compilation and
actual spans require new-head CI; no root-cause repair/merge/closure is claimed.

## Actual Preparation Spans And Post-Submit Follow-up

CI3149/run `38199125087`, exact `a39bfa24956db8a3e02e7bb23a15d3e4b2ae7e2c`:
all six ordinary gates SUCCESS. Baseline60processes1097passed13failed, only
parallel1 fails broader host fixtures; no original Auto watchdog failure in
this round. Instrumented60processes1110passed0failed. Combined2207passed13failed
0ignored/2220tests; failures retained. Checkpoint6119744793. Same image/4CPU,
original source hashes and restoration verified; new brackets actually compiled.
Artifacts64files each, expires2026-10-26: baseline11725210221 SHA256
`78db4cfbdab9c96735765ff68312d93701936e70f680b12b88e5099eb19f57bf`;
instrumented11725165085 SHA256
`773a5cb20aa0ab7214388703c4fab6c0c31a49cd585dca60cb6555b6ba0642b1`.

Matching the Auto launcher PID/thread yields60 cases. Provider pairs59/60
intact (one entry interleaved with concurrent output), max19.9971ms; session
setup max.1645ms, builder setup3.8717ms, builder build64.4404ms, projection
.2183ms, submit.6304ms. Submit return-to-launch66.6265ms–3.4728618s. Largest
single22: decision3.481371s/request3.4864706s/result3.6942738s. First-suite
target succeeds: request2.192847s/result6.6985246s. These successful samples
put the largest measured preparation interval after submission, not inside
measured factory/build spans. They do not assign the failing837 sample's10.6s
to a particular interval or prove any underlying cause.

Next temporary probe brackets the existing AssessmentProvider conversation
stream call, Auto-review stream call and host Auto-report publication. Same
time/PID/thread/stage helper; no message/context/response/digest/outcome content.
Original stream calls/awaits occur once in the original order; existing test
assertions and returned receivers are preserved. Stream return means receiver
creation, not completion of asynchronous stream consumption; the subsequent
return-to-review/report interval includes consumption and request/permission
preparation. Reports may also describe fallback cases, so correlate only the
matching original Auto launcher thread and its fixture outcome. No new async
task, deadline, permanent Rust or fourth source file; budgets, fixed two runners,
original baseline, restoration and failed retention unchanged. Compilation and
actual split intervals await new exact-head CI; no repair/merge/closure claim.

## Actual Pre-Model Gap And Turn Preparation Follow-up

CI3150/run `38218198772`, exact `27c59c086da328324dfb388027a64cf5132c136b`:
all six ordinary mandatory gates eventually SUCCESS. Both diagnostic jobs FAIL,
all120processes/2220tests complete: baseline1101passed9failed, instrumented
1108passed2failed, combined2209passed11failed0ignored. Only baselineparallel1
and instrumentedfirstsuite fail broader host fixtures. Original Auto PASSES at
14.9891291s, a near-miss under the unchanged15s watchdog, not another reproduced
Auto failure. Independent raw verification agrees; checkpoints6119969281 and
6119976824 retain every failure and distinguish ordinary gates from diagnosis.

First target testPID10180/hostThread78/child4540/handle608/waiter147:
build10.686ms, submit69.9us; submit return-to-first conversational MockProvider
stream entry9.5494836s. Conversation stream108.5us, return-to-review entry
27.3626ms, review stream295.4us, review-return-to-report86.0368ms, report
publication78.9us, report-return-to-launch64.6462ms. These stream returns create
receivers, not completed consumption. Decision9.6971733s/request9.8116985s/
result14.9891291s. Launch-return-to-child-signaled5.1745251s, five258/259 polls,
CPU progresses to.453125s. Signal0/exit0 and reader joins occur BEFORE ownedJob
close here, distinct from cancellation cleanup in the prior837failure. This
sample's long prelaunch interval is before the first model invocation, not the
measured mock Auto-review call. No claim that older failures share its cause.

Artifacts64files each, same image/4CPU/original3SHA and restoration, expires
2026-10-26: baseline11733647194 SHA256
`a97c2d68c313ba6f872f806eae53f3efcfe49ffc13762579ca2127f04427b54f`;
instrumented11732933280 SHA256
`808f3e87f44845163c93af7c02854ac4f20a42eaa7baadfebdb0c51176548b68`.

Next bounded overlay adds a fourth temporary source,
`crates/talos-agent/src/request_plan.rs`, original canonical SHA256
`ee966b222561b85a7bce44b6ebf842a54a9627d1a94bd277f68d7ee7f82d08bc`.
It brackets session preparation/structured inputs, turn entry, tool definitions,
protocol selection, prompt build, TurnStart hook and initial request seal. Only
native time/PID/thread/stage; no prompt/tool definitions/input/content/IDs are
printed. A prepare result temporary preserves the original single await and
returned result. Original hooks/protocol/seal/error order stays intact; early
errors may omit a success-return marker and must not become invented durations.
The session entry boundary also distinguishes time waiting for turn preparation
from time inside it; an unmarked later interval remains unresolved.

Independent pre-edit scope review permits this fourth observation source under
the same single-PR mechanically bounded CI-maintenance exception: no Search
delivery, owner-state change, permanent Rust/API/permission/production behavior
or deadline change. Actual scheduling/budget/context/protocol/performance repair
requires an effective governed claim first. Four originals are now captured in
the manifest/overlay and restored byte-for-byte in finally; baseline stays
original. Self-test checks unique anchors and unchanged await count. Fixed two
fresh runners,120cases/budgets/failed retention/mandatory gates unchanged. New
compilation and actual preparation spans await new-head CI. PR remains open,
unmerged; root cause and risk closure remain unclaimed, #590 stays complete.

## Actual Pre-Prepare Failure And Actor Follow-up

CI3151/run `38221117778`, exact `462a8b1d3ecfb51da96898e7517d946b089b81fb`:
both diagnostic jobs FAIL, all120cases/2220tests complete, baseline1076passed
34failed, instrumented1109passed1failed, combined2185passed35failed0ignored.
Instrumented first suite fails only original Auto allow-once watchdog, decision
4.5338539s/request4.5409772s/no result by15s. Baselineparallel1 has16 broader
host failures; parallel2 has18 including all3Auto with stages[]/assessments0.
These pre-stage failures are distinct from instrumented post-request failure.
Every failed sample retained. Checkpoint6120405296, independent raw review agrees.

Actual targetPID4208/thread64/child4244/handle408/waiter146: Submit-return-to-
prepare_session entry4.4746035s; complete prepare7.934ms (definitions3.879ms,
protocol57.5us/prompt1.1729ms/TurnStart67.1us/seal1.8923ms). Long interval lies
BEFORE prepare; cannot blame measured protocol/prompt/hook/seal in this sample.
Ten native wait258/success259 polls, postlaunch host ticks~1s. First7 CPU reads
flat kernel.03125s/user0, later.203125s before cancellation. This is unsignaled
at sampled times, not a diagnosis of IO/Defender/suspension/scheduler. After
watchdog: Jobclose166.6us, then reader EOF and wait0/exit0, finalCPU.25s,
processclose124.6us. Cleanup exit0 is not natural completion proof. Four-source
probes compiled, restored and manifest verified. Ordinary gates are tracked
separately; prior3150success is not current coverage.

Artifacts64files each, same4SHA/image/4CPU, expires2026-10-26:
baseline11734319284 SHA256
`30193bd475521d918fc1ad6c7256c5e9be7fbf392777fc3a5e1f54353a6b1e5b`;
instrumented11733904294 SHA256
`477c7395c1d59b6cd7b173256d3f50db035f0d99cf5be7e756d092b2b0121438`.

Next overlay adds a fifth temporary original, `crates/talos-agent/src/session.rs`,
canonical SHA256 `5d877d723974db086fceabc62c1ada3a44af54e4d1ea625b996fe7093b557c39`.
Native time/PID/thread/stage brackets actor entry/recovery/restore, plain Submit
receive/normalize/process, pending pop, start commit, start_submission, compaction
check and pre-prepare boundary. Submit-processed marks return from the original
accept attempt, not a claimed successful admission; start-return can also mean
rejection. Missing markers/errors never become invented spans. No message,
command, session/submission/turn identifier or persistent-store contents printed.
Original conditions, select/await/task/order/error/permission/queue behavior are
preserved. No new async point; self-test checks unchanged await count and anchors.
All five bytes are captured in manifest/overlay and restored in finally; original
baseline/fixed2runners/120cases/budgets/failed retention/mandatory gates unchanged.
Independent pre-edit review permits same boundedCI maintenance, not production
actor/scheduling/budget repair; such repair requires an effective claim first.
New compilation/spans need new-head CI. Prior failures stay unresolved; no merge
or risk closure and no #590 reopening.

## Actual Actor Recovery Span And Store Follow-up

CI3152/run `38223234443`, exact `383f83a645701710c198c8b037336d78e4526422`:
both diagnostic jobs SUCCESS, baseline1110passed/instrumented1110passed,
120processes/2220tests/0failed/0ignored. New sample pass does not repair or replace
any prior failure. Five originals restored, metadata compiled and manifests/raw
ZIPs independently verified. Checkpoint6120769770. Ordinary gates are separate.

Matching60 original launcher PID/thread pairs: largest successful target
parallel5 PID9060/thread72, Submit-return-to-receive5.2138949s, actor-entry-to-
reconcile-return4.5720454s, subsequent recover63.7013ms/restore574.9591ms,
Submit-to-prepare5.2147522s, prepare7.2336ms, result6.3385167s. Parallel6/3
also have4.3507831/4.3735812s entry-to-reconcile.59 complete actor/reconcile
pairs (one interleaved record); normalize/accept/commit afterward submillisecond.
This locates a successful sample's main startup span in synchronous initial
reconcile, not post-accept queue processing; marker spans include scheduling and
observation/logging, and do not assign older failure gaps to the same operation.

Source call path: reconcile_running_submissions -> pending_store.recover_running
-> recover_states (mutex, connection, schema, query and record collection).
Even no-configured-durable-session fixtures construct a workspace sidecar store.
Connection includes mkdir/open/busy-timeout/WAL+FULL PRAGMAs; current evidence
does not distinguish these operations, drops, scheduling or filesystem causes.
Artifacts64files each, same5SHA/image/4CPU, expires2026-10-26:
baseline11735780513 SHA256
`a426cb1b74f859548c2568cdb258580e508535cef36c6aff90bb07f7cae48e45`;
instrumented11735079349 SHA256
`06ea6edc75f2adfe34dce40d0137f0b7c065840ee97cb24bd35c5c323ddbfb4f`.

Next sixth temporary original: `crates/talos-session/src/pending_submission.rs`,
canonical SHA256 `8c9c2b92f418cfc3c74d613a37464ba26441f9af01e6b60bad63899573508e62`.
Constant metadata stages measure guard, connection, schema, statement prepare,
query creation/record collection and directory/open/busy-config/PRAGMA operations.
Each Instant elapsed is captured before its own logging, excludes that log call,
but remains wall time including scheduling/lock/OS waits, not CPU or pure I/O.
Nested connection timings include internal stage logging; a total closure keeps
original locals/drop order inside and reports after their original drops, so
total includes nested logging and cleanup. No eager iterator replay, extra query,
explicit changed drop order, altered retry/schema/SQL/WAL/guard or error policy.
Early errors retain original conversions/returns and may omit completion markers.
Actor reconcile also captures elapsed before its return log, excluding its own
boundary logging but still including nested observations. No path/SQL/row/input/
session/command contents logged; only constant stage, time/PID/thread/elapsed_ns.
Six originals now captured and restored byte-for-byte, same baseline/fixed120
cases/budgets/deadlines/failed retention/ordinary gates. Independent pre-edit scope
review permits metadata-only same-PR exception; actual storage or scheduling
repair requires effective governed claim. New compile/durations await new-head
CI; older Auto/native-child failures remain unresolved and PR stays unmerged.


## Governed Repair And Next Owned-Child Probe — 2026-10-12

The storage finding now has its own effective RUNTIME-010 / I301 claim through #710 merge
`42b86112`, followed by implementation #711 corrected exact `98fa04cb`. That candidate preserves existing
journal/path/custody/error/WAL/FULL behavior, including the tested ParentDir conservative route.
Its full exact-head CI and merge gates remain pending; this diagnostic owner is not Complete.
The current six-source overlays were checked in memory against that candidate: the missing-sidecar
return stays inside the typed Result recovery closure and retains total timing. This is an anchor
check only, not Windows compilation or post-repair timing evidence.

Next temporary metadata probe: GetProcessIoCounters on the same live owned process handle in the
existing native wait/query site under ADR-068. It uses the already-enabled windows-sys Threading
binding and explicitly initializes its six u64 fields. BOOL success and immediate GetLastError
are logged with time/PID/thread/handle, cumulative operation/byte counts only. A failed read's
zero buffer is not an observed zero; no new process/handle/privilege, memory read, file/command/
environment logging, unsafe block, dependency/feature or production source change is introduced.
The Microsoft contract is [GetProcessIoCounters](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-getprocessiocounters).

Counters describe cumulative I/O accounting, not a blocked-I/O reason. Flat values do not rule out
mapped paging, scheduling, suspension or other waits; progress does not identify Defender/disk or
assign earlier failures. Existing CPU/wait/exit observations and cancellation interpretation remain.
Preserve fresh-first baseline, original 15s/10s/120s deadlines, fixed120process/2220test matrix,
35minute phase budget, byte restoration and all failures. This delta requires independent native
ABI/security review and actual exact-head Windows CI; it has not been run or published yet.

Local preparation checkpoint: all six temporary overlays parse with pinned Rust 1.99 rustfmt;
actual original sources are untouched. All exact anchors and unsafe-block counts remain valid
against the corrected #711 candidate. The candidate pending_submission normalized SHA256 is
`0fddb124c11b9ccb7969ff37febbc379026e528029cd21564bbda8eee30ac52a`; the other five
source hashes remain unchanged from the last diagnostic source. These are pre-merge anchor
checks, not successful Windows compilation, restored-source verification or post-fix timing.

## Post-Repair Production Baseline — 2026-10-12

Implementation #711 merged as `9cad344b8f7adef855ca6b858e6ccc7723103767` after exact
`98fa04cb` CI3161/run38283574722 succeeded in all six jobs, independent review6129237320
and merge-time CAS6129422889. Windows202 session unit tests (including both ParentDir cases)
and real Actor passed. macOS full preflight passed the unchanged Unix-socket fixture; its
permission gate actually observed PermissionDenied. Failed CI3160 remains retained.
I301/RUNTIME-010 stay Review/Claimed pending their published startup-comparison acceptance.

This diagnostic branch now integrates that actual main merge. Relative to production main,
only ci.yml, the temporary diagnostic script and this owner differ. The six original Rust
files, dependencies, lockfiles and 15s/10s/120s deadlines are production-identical. The pending
store normalized source SHA256 is `0fddb124c11b9ccb7969ff37febbc379026e528029cd21564bbda8eee30ac52a`;
the other five normalized original hashes remain identical to CI3153.

The new I/O probe received independent local ABI/security approval. All six temporary overlays
retain exact anchors, parse under Rust1.99 and keep the same unsafe-block count. A missing-sidecar
early return exits the typed Result closure only: total recovery timing still executes. These
local checks do not establish Windows compilation, actual finally-restoration or timing.

Run both fresh-first phases with all original attempts retained. Compare successful
pending_recover_total / actor_reconcile timings with preserved pre-repair CI3153. For the absent
fast path, connection/schema/PRAGMA markers should not execute; do not represent absent markers
as measured zero. Structural regression evidence supports avoided initialization; cross-runner
latency differences, scheduling and nested observers prevent a controlled causal benchmark.
Actual new-head manifests, original-source hashes, attempt counts, failed samples and restoration
markers must be inspected before recording acceptance. No post-repair Windows result is yet claimed.

Native active-child low-CPU delays, the historical early-request/no-result failure, broader host
waits and existing-journal initialization cost remain independent #705 residuals. Successful
new samples do not assign those old failures to this storage repair or establish universal speed.
No release or reopening of #590/I290/I291; I301 closeout follows the measured comparison and
records the already-existing implementation merge SHA, not a status-only commit.

Local stable-stage preparation passed probe self-test, classifier14, workflow/Python syntax,
diff checks and both governance validators with zero warnings. Initial project validation
failed because this shallow clone lacked the already-existing #701 completion object
`690926759229818fcec92b1487d0cffef1bf5fcb`; GitHub verified it, fetching that real object
resolved the failure. No unrelated owner or validation rule was changed.

## Retained CI3163 Failure And Shutdown Follow-up — 2026-10-14

Exact48ed767e CI3163/run38286315560 completed FAILURE. Six ordinary full platform/governance
jobs passed; the two diagnostic phases failed. Both60-attempt matrices completed: baseline1108
passed/2failed and instrumented1097passed/13failed, total120processes/2220executions with2205
passed/15failed/0ignored. Baseline fresh-first failed original Auto after decision151.6643ms /
request159.5418ms with no result before15s; its other failure was paused evaluation. Instrumented
parallel9 failed3Auto shutdown10s checks and10other host tests. Its original Auto inner flow
completed with ToolResult898.8776ms, child natural exit0 and both pipes joined before that result.
Overlay line165 maps to original line151 shutdown expect, not the flow watchdog.

Matched PID5048/thread62 initial recovery69.1622ms / Actor reconcile69.2491ms; subsequent reads
211.8us/204.3us. Across60 matched original Auto targets, first recovery median0.2307ms and
max69.1622ms; reconcile median0.2905ms/max69.2491ms. All63 child I/O reads succeeded. The first
successful instrumented child read57635->60399->172322->2117645bytes before natural exit;
initial flat CPU did not imply absent I/O. These successful counters do not explain the failed
uninstrumented first call, and cross-VM/observer timings are not a controlled causal benchmark.

Post-result target-thread PRAGMA5.8695867s supports a shutdown storage hypothesis, not exact
caller/full-cause proof. pause_unstarted always opens/initializes a journal, including absence;
Actor Shutdown/SQ disconnect/interrupt and custody-error paths call it. The new temporary
observation adds five constant caller begin/return pairs, Shutdown receipt, normal Actor loop /
finalization boundary, RuntimeHandle.shutdown, executor teardown and terminal publication spans.
Terminal publication markers apply to Error as well as Stopped; finalization boundary markers
also surround paths where managed Runtime owns cleanup, and do not assert a finalizer ran.
The pause Result<usize> closure preserves original conditions, SQL, transaction, success/error
and local drop order. It records total including original drops and successful guard/connection/
immediate/schema/update/commit spans; a failing operation may omit its completion marker.
No new await/unsafe/native API/handle/rights/permission/production change or deadline is introduced.

The executor is available again. Old artifacts have now been independently downloaded and SHA256
verified: instrumented11763952124 b7621405aee6ca38212a6c367551b41462ddd1bd6cc8d9240e8e5b8051bb4911;
baseline11763671960 7d13b9767120e2121ff5ffa3ae662740ea2a10cc2399bdcd38143a1c72a7f804. Both
manifests have head48ed767e, matching six normalized sources,4CPUs/image20260925.250.1 and60
attempts with one failed attempt each. Both job logs retain the finally restoration marker;
there is still no independent post-restore source readback hash. Checkpoints6129849703/6129921698.

This stage integrates main f4313237, retaining Search #712-715 and effective #718 / I302 / TEST-003
self-handle claim. That owner's self-only PSS observation excludes #705 takeover; this stage adds
no PSS or launcher-ownership change and leaves #716/#717 and other owners intact. Only the same
three diagnostic files differ from production main. The original fixed fresh-first120/2220
matrix, budgets, no ignored tests and failed-attempt retention remain unchanged. A local SQL-anchor
escape and heartbeat-await-count assertion were corrected before publication; final self-test and
all six generated overlays parse under checksum-verified official Rust1.99. No new Windows result
is claimed. Required independent storage/ABI/scope review and fresh exact-head CI remain.

I301/RUNTIME-010 remain Review/Claimed. A production absent-journal pause no-op changes shutdown
state reconciliation and requires a new effective Work Slice or formal governance scope expansion,
with admission/race/custody/error/WAL review; it cannot enter this bounded diagnostic PR. Original
PowerShell/host failures remain open, with no release, deadline increase, retry-to-green or #590
reopening. Next: correlate actual caller/pause/terminal spans before choosing a production fix.

Stable-stage local verification: both governance validators passed0warnings, classifier14,
YAML/Python syntax and six Rust overlays passed. Locked Linux check of the generated Session/Agent
overlays passed; an isolated worktree then restored both originals and compared actual readback
bytes. This local restoration check is not a retroactive readback proof for old Windows CI.
Independent storage/native/security/scope review found no blocker; new exact remote review and
Windows execution remain required. No product test, full preflight or Windows execution is inferred
from the focused Linux typecheck.

## Retained CI3182 And Actual Pause-Repair Baseline — 2026-10-16

CI3182/run38528907908 exact55dbe63aa40cb0cfc0e172604dfee61966432f53 remains FAILED:
six ordinary jobs succeeded; instrumented1110pass/0fail, baseline1109pass/1fail,0ignored.
All120processes/2220executions completed (2219pass/1fail). Evidence6184082259 and independent
completed-log/source review6184141769 retain the original counts and limitations. Oct15 ZIP
reverification: instrumented11877915517 SHA256c3d0b7060a9fd8a9ec3e4e1985b85316737073e796dd683f9ad625c3cb08b9be;
baseline11876894822 SHA256efa31584406af5470a75c28b7198f9f9fed28cb98487ea0d8feef0566cf14998,
retained untilOct28. No unchanged-head rerun replaces that failed evidence.

The only failure was baseline first-suite paused_evaluation_does_not_block_interrupt_or_shutdown:
outer5s spans Evaluate send, assessor start, Interrupt-or-Shutdown, assessor drop, cancellation
receipt and Stopped, over two branches. The original30s expect wording names the evaluator
budget; baseline cannot identify which branch/substage stalled. All60 matched instrumented
original Auto outcomes succeeded. Largest successful sample parallel3 PID7412/thread71/handle740:
pause1.5210834s, caller pause_shutdown1.5214184s, RuntimeHandle.shutdown1.5225249s; executor394.4us/
terminal69.6us. Connection431.9367ms (open166.3619ms/PRAGMA264.7039ms), BEGIN197.0203ms/schema973us/
update24us/commit485.2577ms. This links caller cost in successful samples, not the cause of the
baseline5s or older10s/15s failures. Totals include observation/scheduling/drops; unexplained
remainder is not assigned entirely to physical I/O.

The separately governed repair is now merged: #726 headaa1c477f1a457dc44ad039cc72bf9ed19842bf83,
main merge c869a9afc2871d82c7fe0a18d1bfddf903831b59, tree1a239eee60205d7a421472b4c66b5b6e8b21d4de.
Effective claim #722/23e659e2 owns RUNTIME-011/I304. Exact CI3203/run38755790909 passed all6jobs
on attempt1; independent exact-remote storage/custody/API APPROVE6195119208 and CAS6205402481
precede merge. Completed macOS logs verify full preflight including the unchanged Unixsocket
case, PermissionDenied fixture and both SDK modes. Windows logs verify new Actor/Runtime tests,
Plugin admission and both governance validators0warnings. Local failures remain in I304/#726;
capable native success is not a waiver or erasure of those failures. Shared Agent identity is
not another natural person's formal approval. I304 and I301 stay Review/Claimed pending their
own acceptance; no #590 reopening, release, Search/Git/PSS/Skill takeover or #705 merge.

This diagnostic candidate integrates that actual production merge. Only the same three diagnostic
files differ from main; all six Rust originals are production-identical. Five are unchanged from
55dbe63; pending_submission normalized SHA256 is now
e80131fd58cb0e9b14ff69dcebdc8ad73427b435e60383061613d8505c30c7fd through #726. Other production
changes since CI3182, VM variation and observation prevent a controlled causal latency benchmark.
The unchanged workflow retains two fresh-first runners,60attempts/1110expected executions each,
120/2220 total if suite106, all original assertions, Auto15s/cleanup10s/tool120s/evaluator5s,
35/40/45minute enclosing bounds, source restoration and every failed attempt. Baseline never
overlays probes. No new Windows comparison outcome is claimed before actual artifacts.

Temporary metadata-only additions under the same bounded CI/fixture-maintenance exception:
- Pause inspection executes once under the original guard, emitting constant absent/existing
  classification and elapsed captured before logging. Early zero still exits the typed Result
  closure; original guard drops and pending_pause_total remain observable. SQL/errors/order are
  unchanged, and absent later markers are not measured zero durations.
- The paused-evaluator fixture emits branch/wait/enqueue/receipt stages and assessor entry/drop.
  inspect_err observes the outer5s Result before the unchanged expect. No new await/task/native
  call, deadline, assertion or authority is introduced. Drop is not a durable receipt. Missing or
  interleaved markers cannot establish cause, and uninstrumented failures remain phase-unknown.
  Added stderr may perturb scheduling, which is why the unchanged baseline remains mandatory.
- Finally restores the same six raw snapshots, reads them back and writes restoration.json with
  expected/restored raw SHA256 and exact byte-match flags (distinct from normalized manifest
  hashes). Mismatch/read/write/artifact errors fail the job. Earlier Windows printouts are not
  retroactive readback proof. Disposable CRLF/non-UTF8 fixtures and an injected mismatch exercise
  fail-closed verification without modifying checkout sources.

Previous local preparation on the identical production tree passed self-test, pinned1.99 syntax
parsing of all six overlays and cargo check --locked --workspace --tests; every source was then
restored with all six raw readbacks matching. Independent Agent local review found no blocking
scope/storage/native/ABI/security issue. Workspace maintenance removed that unpublished checkout
and its raw logs. Reconstructed script blob40a44187adddf5fccf2c1190ab827b2ab1dacda5 matches the
recorded prior40a44187 prefix; self-test passed again. These are retained preparation facts, not
fresh Windows execution or a full-preflight pass. Current validation is appended below before
publication. The six existing owned-handle wait/CPU/I/O probes, initialized buffers, ABI/rights
and unsafe sites remain unchanged; no PSS/otherPID observation is added.

Fresh exact-remote review, both validators and full native CI are required. Native-child latency,
old early-request/no-result and10s failures, existing-journal costs, previous Plugin uncertainty
and evaluator phase attribution remain unresolved. A green sample cannot prove universal speed;
diagnostic success alone cannot complete I304/I301 or authorize merging this temporary PR.


### Fresh Reconstruction Validation — 2026-10-16

The reconstructed candidate passes diagnostic self-test (including raw restoration/mismatch
fixtures), Python AST, workflow YAML parsing, all14 CI classifier tests, pinned Rust1.99
workspace formatting and syntax parsing of all six generated overlays outside the checkout.
Independent byte comparison confirms all six production sources remained unchanged. The first
standalone rustfmt preview attempt could not resolve host child modules in its disposable
directory; rerunning with skip_children=true parses each of the six previews successfully.
This is syntax validation, not a fresh workspace typecheck or Windows execution. The prior
same-tree locked workspace/test typecheck above remains historical evidence.

Fresh local independent Agent review APPROVE covers script40a44187adddf5fccf2c1190ab827b2ab1dacda5,
workflow455c97d8b86495a40a9d907248345e935559907f and the owner before this validation appendix.
It found no blocking storage-order/native/ABI/security issue. Published-head review remains due;
shared Agent/GitHub identity is not another natural person's formal approval.

The freshly installed pinned compiler initially could not load LLVM. The installed LLVM file
was67108864bytes while the same1.99 official compiler archive member was208094144bytes.
The official rustc archive SHA256 was verified as
77171ba2a0345fdf2abc4fedda55d6de078dae7a68527c28be8c77dcc9604bd5 before restoring its three
compiler-library members into the same toolchain. rustc -Vv and pinned rustfmt then succeeded.
The installation truncation cause is unknown and does not establish the cause of an earlier
GNU SIGBUS. No toolchain version, dependency, lockfile or production source changed.

Collaboration claim validation passes with0warnings. The first project-governance invocation
failed because cargo was absent from that command's PATH; the corrected explicit pinned
Cargo/Rustup environment then passes full project governance with0warnings, SQLite consumer
fixtures17cases and delivery fixtures12cases. Both governance validators therefore pass;
all initial failures are retained. Final staged diff/whitespace review passes with exactly the
three diagnostic files relative to actual mainc869a9af. No local full-preflight or Windows
execution is claimed. Next: exact-remote review and all eight CI jobs on this stable candidate.


## Owned-Child Lifetime Versus Observation — 2026-10-16

Current production main is3bc38d780804799133f20cfdede583901fb5ada4 (#728). I301 and I304 are
Complete/Closed in their own bounded source owners; this continuation does not reopen them.
The baseline comparison at diagnosticff84ec7 completed CI3206/run38832452061:120processes,
2220pass/0fail. Retain CI3163's2205pass/15fail and CI3182's2219pass/1fail as failed evidence.
Comments6211820088,6209483437 and6211905052 preserve closeout and raw timing interpretation.
Search #716 remains independently owned; no Git/PSS/Skill work is transferred.

Offline round verified the CI3206 artifact ZIP SHA256 against GitHub digests:
baseline12017369938=1faed5ecf6261634cb26b97f82a1360d61d80dedb86260855470f7a7fd412f8e;
instrumented12018356958=2fcf78709cd7614c4f5f3e004a4c316506121ec122ab54dbfa85a599d52b36cb.
All60 child samples have unique assigned/resumed, wait-return and pipe-join endpoints and an
ordered first stdout-read return paired with positive byte count for the same log/PID/handle.
Assigned-to-positive-read median181.3484ms/max4846.9981ms; positive-read-to-wait-return
median16.4012ms/max65.2970ms; wait-return-to-pipe-join median0.18025ms/max0.2497ms.
The slowest successful sample spends98.6707435% of its4912.2951ms assigned-to-wait window before
that positive read. This is parent-observed read completion, not child write/script-entry time.
The optional waiter-entry timestamp in parallel-7 was interleaved; no timestamp was reconstructed.
Eighteen of20 evaluator branches have intact intermediate timing markers, with maximum
host-ready-to-Stopped328.0706ms. Two branches are incomplete due to interleaving. This proxy span
is not the exact outer5s timer; the old uninstrumented failure remains phase-unknown.

Hypothesis for the next native round: a material part of observed assigned-to-wait time could be
parent observation delay after the child has already exited. The existing GetProcessTimes call
already fills initialized creation/exit FILETIMEs but only CPU fields were emitted. The sole
probe increment emits those two existing values with process handle/PID/thread, only when the
existing wait returns WAIT_OBJECT_0 and GetProcessTimes succeeds. No extra native call, unsafe
site, access right, other PID, PSS, child command, await or timer is introduced. Failure/timeout
emits no lifetime record; absence is unknown, never zero. Exit code259 alone is not used to infer
liveness. Existing CPU/IO/wait records and their order are retained.

Primary API contract:
https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getprocesstimes
Creation/exit FILETIMEs are wall-clock points in100ns units since1601; exit is undefined while
running. Successful read plus a signaled owned-process wait gates use of exit. In analysis, require
matching identity/lifetime, valid ordering and intact records before subtracting. Preserve raw
values and reject negative/inconsistent spans rather than clamping. Wall-clock adjustments,
resolution and logging/scheduling overhead limit attribution; these are not monotonic spans.
Compare recorded exit with native_wait_observed and process_wait_return; the latter also includes
sampling/logging/CloseHandle overhead. Compare creation-to-exit separately from assigned-to-exit:
creation precedes assignment/resume. An exit substantially before observation supports an
observation-delay component; an exit close to observation leaves the long child lifecycle to be
explained. Neither result proves Defender, disk or any particular startup/execution cause.

The bounded CI/fixture-maintenance exception remains applicable. Fresh-first baseline/instrumented
runners, all original assertions, Rust1.99.0, dependencies/lockfile, Auto15s/cleanup10s/tool120s/
evaluator5s, process counts and enclosing budgets remain unchanged. The unchanged workflow runs
on this substantively new diagnostic head, not an unchanged-head retry to replace old failures.
The six raw production snapshots must still restore with exact readback. No prewarm, skip,
threshold relaxation, retry-to-green, release, production repair or merge of #705 is authorized.
Old10s/15s failures, evaluator5s, existing-journal cost and Plugin uncertainty remain open.
Local and exact-remote review/validation evidence follows; native results must be recorded after
execution, without promoting a successful sample to universal reliability or root-cause closure.


Local preparation for this delta: diagnostic self-test including restoration/mismatch fixtures
passed; pinned Rust1.99 rustfmt parsed all six generated overlays without changing production
sources. A disposable Rust harness executes the actual generated lifetime guard over nine
wait/success combinations (signaled/timeout/failed times zero/positive/negative BOOL), emitting
only two signaled-success records; high-word FILETIME conversion also passes. This is mocked
value verification, not native execution. Classifier14, text boundary23crates, whitespace and
both governance validators pass with0warnings (SQLite17 and delivery12 fixture cases). Actual
locked Cargo metadata used the pinned environment. No new Windows or local full-preflight pass
is claimed; the exact-head native workflow remains required. Incoming main closeout files are
preserved byte-for-byte and only the same three diagnostic files differ from current main.


## CI3211 Retained Failure And Next Engine-Event Observation — 2026-10-16

CI3211/run38878126240 at exact819c5f1222a77b5e677d4ba1109f608c98e90c26, attempt1, is FAILED.
Seven jobs succeeded; baseline first-call diagnostics failed. All120processes completed:
baseline1109pass/1fail; instrumented1110pass/0fail; total2219pass/1fail/0ignored.
Baseline-suite-first-1 original Auto allow-once hits the unchanged15s watchdog after decision
111.7885ms and tool-request139.8818ms, with no ToolResult. It has no native instrumentation;
the exact stalled phase and cause remain unknown. Later passing processes do not replace it.

Artifacts (65files/CRC each, retained untilOct30) verified against GitHub digests:
baseline12037036276 SHA2563960c8bc77a73ced606d754dd868081e3522a295ce8be1cdeea2bba4848e03ef;
instrumented12036218858 SHA2567c13f8133d71d79c999f58b855eacf9160d346512c320a54dae776fd65204cde.
All six normalized source hashes match main3bc38d78; all12expected WindowsCRLF/raw source
restoration receipts match. Independent Agent completed-evidence review confirms these facts
and the bounded arithmetic below; shared account is not distinct human approval. Evidence and
review: comments6213194395 and6212170280. Main and other owners remain unchanged.

A near-threshold successful original sample, instrumented-suite-first-1 PID8892/child3400/
handle608/waiter135, has ToolResult14.8179905s and recorded child lifecycle13736.1122ms.
Exit-to-native-observation8.1979ms, exit-to-wait-return8.7137ms, later pipe-join0.2387ms;
first positive stdout13629.3446ms after creation; final cumulative child CPU0.765625s.
All60unique lifetime records match intact owned identities and parent observation endpoints.
Exit-to-native-observation median8.43405/max12.786ms; exit-to-wait median8.97335/max13.2955ms;
join median0.23825/max0.4125ms. Only59assignment records are intact: the slow sample's assignment
header is interleaved, so no assigned-to-exit time is reconstructed for it. An initial extraction
requiring assignment omitted that case; creation/exit FILETIMEs independently retain it for
lifecycle analysis. Several early counter plateaus followed by progress are visible, without
identifying a wait reason, physical I/O, Defender or scheduler cause. This tests and limits the
parent-observation-delay hypothesis for these samples; it cannot explain the uninstrumented
failure or distinguish engine startup from command execution/formatting.

Next hypothesis: existing PowerShell lifecycle-event timestamps may divide the first owned
child's recorded lifecycle around an engine-state observation. The collector runs only AFTER
all original instrumented attempts and successful raw source restoration, in the existing
finally block. Baseline never invokes it. No child command, cold-first order, Rust probe, native
call/rights/unsafe, assertion, deadline, dependency, lockfile, toolchain or workflow is changed.

It requires one intact creation-return/adjacent child-PID receipt and one lifetime record from
instrumented-suite-first-1 (8MiB source-read cap), not its damaged assignment marker. PID/handle/
parent must match; lifetime must be positive and at most the existing90s owned-test cap, with
supported2000..2100 UTC bounds. No identity is inferred from time proximity or partial lines.
It then uses the explicit temporary host-utility bridge wevtutil.exe from SystemRoot/System32,
qe only, fixed Windows PowerShell channel/PowerShell provider/IDs400+403, exact child PID and
creation-through-exit UTC window. No enabling/configuring/clearing/exporting event logs, security
settings or tracing, no discovery across logs, other-PID queries, PSS or unrelated task is added.
The replacement/removal trigger is completion or rejection of this bounded diagnostic experiment;
no product dependency on wevtutil is introduced.

Primary utility contract:
https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/wevtutil
https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.diagnostics/get-winevent?view=powershell-5.1
The latter documents the Windows PowerShell event400 engine-Available example. This is an event
observation, not proof of exact script-entry or exclusive startup cost. Classic-provider PID and
seven-digit timestamp availability on this runner are unverified assumptions. Query absence,
access failure, unsupported schema, missing identity or ambiguous transitions remain unknown;
no matching record is not zero initialization time, and no trace policy is changed to force one.

Read-only query caps:17events requested to detect overflow beyond16,131072actual bytes accepted,
one capped pipe reader, query30s/kill-wait5s/reader-join5s. It launches only with45s left in the
existing35minute experiment budget. No Popen context manager or main-thread close of an active
reader pipe can introduce an unbounded cleanup wait. A thread-start failure kills/reaps the owned
utility within5s; failed reaping writes unavailable evidence and raises a constant failure.
Reader is daemon-owned and closes its own pipe; utility output is never sent through execute(),
printed or written. Stderr is discarded. Raw XML/EventData/rendered messages/command payloads
remain transient and are not persisted. Only eventID, childPID and canonical UTC timestamp survive
in first-child-engine-events.json, with fixed status and the selected child lifetime metadata.
All returned records are independently rechecked for fixed namespace/root/provider/channel/ID/
PID/time bounds; malformed XML/DTD/entities, byte/count overflow and inconsistent records are
rejected as a whole. Duplicate transitions are retained as ambiguous records, not auto-selected.

The unchanged original result codes remain authoritative; optional collection cannot turn an
original failure into success. No retry, warmup or new test skipping. CI3163(2205/15),CI3182
(2219/1),CI3211(2219/1) remain failed evidence, CI3206(2220/0) successful observation only.
Old10s/15s, evaluator5s, Plugin uncertainty and existing-journal costs stay open. I301/I304 stay
Complete/Closed for their separate slices, #705OPEN/unmerged. The same bounded CI/fixture
maintenance exception applies. Local/independent review and fresh native evidence follow.


Local preparation: new scalar-only event/identity/query self-tests pass, including mocked query
success/timeout/reap-failure/thread-start-failure, actual100ns conversion, malformed/interleaved
identity rejection, wrongPID/ID/window/DTD/byte/count caps and command-content sentinel exclusion.
No native utility is invoked by self-test. The first independent design review REQUEST CHANGES
identified Popen context cleanup as potentially unbounded; that unpublished form was replaced
by the explicit bounded wait/reap/reader paths above, and failed-cleanup fixtures now pass.
All six generated Rust overlays compare byte-for-byte equal to prior819c5f12's equivalents,
and all six originals compare byte-for-byte to actualmain3bc38d78. Workflow, dependencies,
Cargo.lock and toolchain have no delta. Classifier14, text boundary23crates, Python parsing,
whitespace and both governance validators pass0warnings (SQLite17/delivery12 fixtures). No new
Rust build, full-preflight or actual Windows event/schema success is claimed before CI.

A second design finding required an explicit fresh-matrix-completed gate before the collector;
that gate is now set only after matrix returns, and successful restoration precedes collection.
Three disposable main-flow fixtures verify completed60with an original failure retains exit1
and collects once, build-abort and matrix-watchdog-abort collect zero times, and all six raw
snapshots restore in every path. These are mocked flow checks, not new native passing tests.


## CI3212 Harness Failure And CRLF Parsing Repair — 2026-10-16

CI3212/run38886936675, exact505a51bb9f379813ac1f82c18317c8929264d75f, attempt1, completed
FAILED. The six ordinary jobs succeeded. BOTH diagnostic jobs failed in the added lifecycle
self-test at line223, checking receipt.status equals records for the normal mock query. Their
original test matrices did not start; no baseline/instrumented diagnostic artifact exists.
The separate i170walkthrough artifact is not a diagnostic matrix. This is a diagnostic harness
failure, not a new original Auto/evaluator timeout, and contributes no new120/2220execution count.
CI3211's2219pass/1fail and all preceding failed evidence remain unchanged and authoritative.

Determined failure chain: Path.write_text(source) uses platform newline translation, producing
CRLF on Windows. The collector reads raw bytes and decodes UTF8 without text-mode universal newline
conversion. first_child_window's intact-line regular expressions required LF, so a CR before the
end anchor made the lifetime record unmatched. The collector correctly returned unavailable,
but the self-test correctly expected records and failed. Primary Python3.12 newline contract:
https://docs.python.org/3.12/library/functions.html#open
On Linux, executing the EXACT previous lifecycle_self_test with Path.write_text forced to Windows
newline translation reproduces its same assertion at223. The proposed version passes the SAME
simulation. This determines the diagnostic fixture/parser failure, not the underlying intermittent
Windows timeout. It is not native query availability or event-schema evidence.

Small correction: normalize only CRLF pairs to LF inside first_child_window before the existing
strict identity/adjacent-record matches. No trimming, arbitrary whitespace acceptance, joining
interleaved lines, substitute PID, weakened assertion or error suppression. Existing LF logs
keep the same tuple; CRLF versions produce that identical tuple. Duplicate receipts, invalidPID
and interleaved adjacency are explicitly rejected in BOTH encodings. The mock first-suite file
now writes deterministic UTF8 CRLF bytes even on Linux, making this Windows path part of every
self-test rather than hiding it behind the host platform's default newline. Collector query,
record/status/privacy/caps/reaping logic and all original Rust overlays/workflow/sources,
dependencies/lockfile/toolchain/deadlines/assertions/first-call order remain unchanged.

The earlier transient worktree had reverted to the pre-collector snapshot and the CI3211 local
ZIPs were absent on resumption. Retrieved both exact505a51bb remote blobs again and verified Git
blob identities(script41b1d085b543fa80ad1bec4e364257d36f285f39 and ownerc3ca7a30c304a2ef84a04b769166f0aa51dcea85).
Reconstructed tree2ae6889283dfd11653afd188e857be1107777ba6 exactly before applying this delta;
prior native/evidence statements above remain historical records, not fresh local revalidation.

Local correction checks: exact old self-test FAIL223 under Windows newline simulation, fixed
same simulation PASS; diagnostic self-test, LF/CRLF strict-identity regressions, classifier14,
text boundary23crates and whitespace PASS. Original assertions remain exercised. No rerun of
unchanged505a51bb to replace CI3212, no Windows query success or timeout-root closure claimed.
Fresh independent review/governance and a new native run remain required. #705 staysOPEN/unmerged;
I301/I304Complete/Closed, old10s15s/evaluator5s/Pluginunknown and other-task owners unchanged.

Both governance validators now PASS0warnings with actual pinned1.99/locked metadata
(SQLite17/delivery12 fixture cases). Six generated Rust overlays byte-compare identical to
exact505a51bb's reconstructed tree; workflow/production/deps/lock/toolchain have no delta.
The independent causal review repeats oldFAIL/newPASS and confirms the parsing fix preserves
strict identities. Final exact-local and exact-remote publication reviews follow separately.
CI3212 failure/diagnostic classification is also recorded in comment6217759937.


## CI3214 Final Evidence And Provider-Lifecycle Observation — 2026-10-16

CI3214/run38916833365 exact0d5cc0bc712a651c056cf88bb875e1903f8a095f attempt1 is finally FAILED:
seven jobs SUCCESS, Windows workspace117026425095 FAILED in original partial-setup process-total
handle assertion before147/after154 (talos-tools440pass/1fail). This is separate from BOTH successful
diagnostic jobs: baseline60processes/1110pass0fail and instrumented60/1110pass0fail, total120/2220.
Native CRLF self-tests pass. Do not promote passing diagnostics to whole-run success or claim a
proven product handle leak from a process-total delta. No rerun/threshold/assertion replacement.

Verified GitHub artifact digests,CRC and extracted bytes: baseline12052878529 SHA256
2357d397e47dd213aaa71a2c3da35aa5a42bbc22eb4e2fd74459779598a280cc (65files);instrumented12053096821
SHA25675ecf5d3dd88ecdc3924d1f1250baca3d0dac98e5cd1f99451325f025bf0206b (66files),expireOct30.
Workspacefailure12053327443 SHA2563b7abe80641ce65a7191789b30f916fe23783ac4363eb34c1ac2eb14acb0d3a8
(2files),expireOct19. Both manifests match exacthead,Windowsimage20261007.264.1/X64/4CPU;
sixnormalizedsources and all12rawWindows restoration receipts match actualmain3bc38d78.
Independent Agent evidence review APPROVE verifies these facts, counts, identities and arithmetic;
shared account is not distinct human approval. Checkpoints6218154137 and6218160205 preserve the
raw facts and review; earlier in-progress statements are historical, not the final state.

The first-owned-child collector returns records for child1476,parent8564,handle612,waiter119:
created134366352538045017; event400134366352558003561;event403134366352564470349;
exit134366352564665476. Lifetime2662.0459ms splits into creation-to4001995.8544ms,
400-to403646.6788ms and403-to-exit19.5127ms. Native observation12.3181ms afterexit,
waitreturn12.6398ms afterexit, then pipejoin0.3731ms. First positive stdout is
134366352564177663; ToolResult2.7650226s. The timestamp-only receipt passes fixed provider/
channel/PID/lifetime checks. Review verifies that receipt and collector source, not discarded
raw XML or a retrospective query on the departed runner. These are engine-event observations,
not exact script-entry/exclusive initialization cost or a cause of CI3211's13.736s child/failed
baseline, older10s15s, evaluator5s or Plugin uncertainty. Event logging timing/clock limitations
and successful-sample versus historical-failure distinctions remain.

Main has since advanced to c81782b6d922ec1b8bfa62e9c4a2914d746680bb through governance-only#730.
It makes the independent I305/TEST004 claim and ADR091 amendment effective, without implementing
any native/test change. Incoming nine documents are integrated byte-for-byte. The current +2
assertion is therefore still present and preserved. Its implementation belongs to that separate
owner/#716/#729; #705 does not take it over, change acceptance or mark it Complete. I301/I304
remain terminal. This refresh adds no production-source/workflow/dependency/lock/toolchain delta.

Next testable hypothesis: the observed creation-to-event400 wait may precede the earliest
PowerShell internal provider-lifecycle event, or may occur among those lifecycle observations.
Extend the SAME fixed post-test query and scalar-only parser allowlist with event600. No new
channel/provider/PID/window/query process or access is added; /c:17,accepted16events,128KiB,
30/5/5s bounds,45s remaining-margin check, completed-matrix and successful-restoration gates
remain unchanged. Original cold-first commands/120processes/2220executions/guard deadlines and
all assertions, baseline without collection and six Rust overlays remain unchanged.

Primary event600 reference (official example of Provider Started events before engine400):
https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.utility/show-command?view=powershell-7.5
Only eventID/PID/UTC is retained; internal provider names, Started/Stopped payload, EventData,
commands, XML and rendered errors are NOT saved. Therefore a600record is a provider-lifecycle
observation, not asserted to be a particular provider or transition. Do not automatically label
all600s as startup, select a unique provider, or infer an exclusive cost. If multiple600records
precede an unambiguous400, compare the earliest/latest such observations to creation and400
while retaining all timestamps; none of those intervals is exact code-execution time. If600s
are missing, timestamps are ambiguous/inconsistent, the16record cap overflows or native query
fails, evidence remains unknown; do not increase caps or fetch partial replacements to force it.
Prior CI3214's raw event XML is not retained and cannot be retroactively expanded to600s.

This is a substantive bounded metadata experiment, not an unchanged-head retry for a green run.
No enabling logs, prewarm, skip, relaxed threshold, retry-to-green, native call/unsafe/rights,
PSS/otherPID observation or product repair. Previous CI3163/3182/3211 runtime failures,
CI3212 harness failure, CI3214 workspace failure and CI3206/3214 passing diagnostic samples are
all retained. #705 staysOPEN/unmerged. Fresh local/independent review and native evidence follow.

Local preparation: unchanged diagnostic self-test plus exact scalar600acceptance and fixed-query
ID-set assertions PASS; all unknown-ID/PID/time/DTD/count/byte/privacy/CRLF/cleanup regressions
remain. All six generated Rust overlays compare byte-identical to previous0d5cc0bc equivalents;
all production sources/lock/toolchain compare identical to currentmainc81782b6; workflow unchanged.
Classifier14/text23/diff and both governors PASS0warnings with actual pinned1.99 locked metadata
(SQLite17/delivery12 fixtures). This is not a new native600record or full-preflight outcome.

## CI3220 Final Evidence And Console-Startup Hypothesis — 2026-10-17

CI3220/run39005526212 exact0e5c89c5f05862994de09ec5f2dda2119a44a6ef attempt1 completed
SUCCESS: all8jobs, baseline60processes/1110passed and instrumented60processes/1110passed,
combined120processes/2220passed/0failed/0ignored. Both ZIP CRCs and digests were verified:

- baseline artifact12090564330 SHA256 f851c91e0c336d9b02fcc1777c18153aa9bfecda9f46bc0e3d3ee3fbfd9f203e;
- instrumented artifact12091197189 SHA256 545b1654116a4909700337aa30e360194f1343f26131186e4eb891d900a79be9.

Both expire2026-10-31. Manifests identify exacthead/image20261007.264.1/4CPUs; all6normalized
source hashes and12raw Windows CRLF restoration receipts match reviewed production sources.
First sample parent9612/child3900/handle460/waiter143 has one intact lifetime and8accepted
scalar records (six600,400,403): creation134366881763898916, exit134366881825159083.

| Recorded interval | ms |
|---|---:|
| Creation to exit | 6126.0167 |
| Creation to earliest logged600 | 4681.1891 |
| Earliest to latest logged600 | 38.0995 |
| Latest600 to400 | 171.8343 |
| 400 to403 | 1201.9257 |
| 403 to exit | 32.9681 |

76.4148929% of this successful lifetime precedes earliest logged600. FinalCPU0.609375s;
CPU/IO samples advance. Exit to native observation11.961ms, exit to waitreturn12.3896ms,
subsequent pipejoin0.2701ms; originalToolResult6.7965185s. This is not firstactual provider
work, provider/state identity, exclusive.NET cost or evidence of Defender/disk/scheduler cause.
Wallclock/event recording limits remain. RawXML is discarded intentionally and cannot be
reparsed. Independent Agent read-only evidence review APPROVE verified head, hashes/CRCs,
counts, identities/restoration and arithmetic; shared account is not distinct human approval.
Evidence/review receipt: #705 comment6233133393. Earlier failures and unknowns remain intact:
CI3163(2205/15),CI3182(2219/1),CI3211(2219/1),CI3212 prematrix CRLF harness failure,
CI3214 original workspace handle147to154 failure (its diagnostic2220/0), evaluator composite5s,
old10s/15s and Plugin uncertainty. No overall timeout closure; I301/I304 remain Complete/Closed.
Main c81782b6d922ec1b8bfa62e9c4a2914d746680bb and separate #716 owner/head198cb751 checked;
I305 implementation remains separately owned. #705 stays OPEN/UNMERGED.

Next bounded hypothesis: existing console-start observations may subdivide creation to
firstlogged600. The Windows5.1 reference enum assigns40961/40962 to
PerformanceTrackConsoleStartupStart/Stop:
https://learn.microsoft.com/en-us/dotnet/api/system.management.automation.tracing.powershelltraceevent?view=powershellsdk-1.1.0
The current upstream manifest independently places these IDs on its Operational channel;
that Core manifest is not proof of the installed Windows5.1 channel configuration:
https://github.com/PowerShell/PowerShell/blob/master/src/PowerShell.Core.Instrumentation/PowerShell.Core.Instrumentation.man

Candidate adds one sequential postmatrix/postrestoration read-only wevtutil query of the fixed
Microsoft-Windows-PowerShell/Operational channel, fixed Microsoft-Windows-PowerShell provider,
IDs40961/40962, and the SAME first owned childPID and creation/exit bounds. Availability and
actual image contract remain a native hypothesis; no logging/policy is enabled or changed.
Each query independently requires45s left in the existing35min script budget and retains
30s query/5s reap/5s readerjoin,131072byte/16accepted event caps (query17 detects overflow),
8MiB source cap and exact owned utility cleanup. No sharedbudget extension. Insufficient
margin skips the second query as unavailable. Unconfirmed reap remains an explicit failure.
Only eventID/PID/UTC and selected child lifetime survive in first-child-console-events.json;
no command/event body/XML/provider state is saved. The existing classic receipt stays separate.
Missing/incompatible/ambiguous/overflow records remain unknown; even valid markers do not
identify the whole historical failure or exclusive internal initialization cost. Baseline has
no collector. All original attempts and six source restorations precede BOTH collectors.
No Rustoverlay/nativeAPI/production/workflow/toolchain/dependency/lock/timeout/assertion change;
no prewarm, ignore, threshold relaxation or retry-to-green. Same3diagnosticfiles versusmain.

Local validation: collector selftest covers both IDs, exact provider/channel separation,
wrongPID/outoflifetime rejection, privacy sentinel, console argv/receipt, exhaustedbudget
without Popen; inherited caps/timeout/reap/threadfailure fixtures remain. All six generated
Rust overlays must remain byteidentical to prior0e5 candidate. Local Rust toolchain is absent;
no fresh pinnedRust/fullpreflight claim; remote exacthead checks remain required before any
acceptance. This is diagnostic publication only, never merge/rootclosure authorization.

Continuation checks: selftest PASS including sequential classic-success/console-budget-skip;
classifier14PASS; textboundary23crates/0errors; collaborationclaims0warnings; diff whitespacePASS;
all6generated Rust overlays compared byteforbyte IDENTICAL to exact0e5 predecessor. Fresh
projectgovernance attempt is BLOCKED/FAILED1error because cargo is absent when its SQLite
consumer validator invokes metadata; delivery12fixtures pass. Prior tree's pinned metadata
success is retained historical evidence, not relabeled as this candidate's fresh success.
No merge until required exacthead CI/governance/security gates pass.

## CI3222 Final Evidence And Intact-Identity Receipt — 2026-10-17

CI3222/run39027455520 exact94e8531273e751eb2e119970d3a7673baeaa296b attempt1 SUCCESS:
all8jobs, baseline60processes/1110passed/0failed/0ignored and instrumented60/1110/0/0,
combined120processes/2220passed/0failed/0ignored. Artifact SHA256/CRC/extracted bytes verified:

- baseline12099770323 (65files):143df807db4c6ff7992091fd3221784c1f53ef6291f5491e0cc319d35c256563;
- instrumented12099192017 (67files):e293e7be9811dcba0e552dafb7a70b3a87d536206e6604e5fa8d320f5a4050c5.

Both expire2026-10-31. Exacthead/image20261007.264.1/4CPU manifests match; all6normalized
production source hashes and12rawWindows CRLF restoration receipts independently verified.
Both event receipts are unavailable/events[] WITHOUT a selected child tuple. First-suite
creation-return header is fragmented by parallel libtest stdout; first_child_window independently
reproduces ambiguous-child rejection. Separate intact PID/lifetime lines MUST NOT be joined
retroactively. Failure occurs before event query, so console-channel availability and the
previous CI3220 creation-to-firstlogged600 interval remain untested by this run. No zero-event,
zero-duration, provider-state or historical-cause conclusion is justified. Independent Agent
read-only review APPROVE bounded completed evidence; receipt #705 comment6233549123. Shared
account/Agent review is not distinct human approval.

Next hypothesis: reducing cross-stream formatting fragments can allow a fresh intact identity
record to reach the existing postmortem collector. The diagnostic overlay now emits one
self-contained child_created receipt containing time/childPID/handle/parentPID/thread at the
existing creation-return position, using a preformatted String with leading/trailing newline
fences and one stderr-lock write_all. The successful postexit lifetime receipt uses the same
writer. Lock lives only for that write; no new native API, policy, file, dependency or production
behavior. One write_all is NOT an atomicity guarantee against stdout/OS writes; damaged records
remain unavailable. No old log is repaired and no guessed tuple is queried.

Parser requires exactly one complete new identity receipt in the source, one intact successful
lifetime record, matching handle/parentPID, valid childPID, and receipt time within creation/exit.
Duplicate records (including conflicting owner), fragments, wrongPID/handle, invalid time and
old adjacent or fragmented schema are rejected. CRLF normalization remains the sole normalization.
Newline-fenced records surrounded by unrelated test output pass. Existing classic and console
query identities, allowlists, count/byte/time/owned-cleanup bounds and completion/restoration
ordering stay unchanged. No retries, prewarm, ignored tests, threshold relaxation or changed
Auto15s/cleanup10s/tool120s/evaluator5s/assertions. Rust1.99.0/dependencies/lock unchanged.

Only the diagnostic boundary overlay changes (two receipt emissions and one small writer);
other5generated Rust overlays are byteidentical to94e8531. Actual production sources and workflow
are unchanged; same3diagnosticfiles versus main. Currentmainc817 and #716head198cb751 unchanged;
I305 implementation remains separate owner, I301/I304 stay Complete/Closed. All prior failures,
CI3222's unavailable-event outcome, evaluator/old10s/15s/Plugin uncertainty stay retained.
No merge, release or overall timeout-root closure authorization. Exactcandidate independent
technical/security review and native CI remain required. Local Cargo is still absent; no fresh
pinnedRust/fullpreflight success claimed. Prior freshgovernance blocked at SQLite metadata remains
an explicit local environment limitation, never relabeled as passing by the green CI result.

Before publication, concurrent #716 merged main forward to
b5b8241e2d8fa72d6e469bd96ccbdf54ff851c69. Synchronize that already-reviewed main wholesale,
preserving its separately owned Search transport, I302/PSS/I305 tests, native source inventory,
governance and dependency-feature state. This is baseline synchronization, not #705 authorship
or takeover; #705 does not modify those implementations, acceptance predicates or new native
sites. Historical CI3222 used oldc817 and is not relabeled as testing newmain. Old handle-count
failures/assertions remain historical evidence even though the separate I305 owner has changed
main's acceptance. No claim that all old main assertions still exist after that other PR.

Threeway workflow merge preserves incoming installed-module evidence collection and keeps
only the existing #705 job additions versus newmain. All27incoming changed files except the
necessarily combined workflow must byte-match newmain. Both previous and current diagnostic
probes apply to new production sources successfully; only the same boundary receipt writer and
two emissions differ between those overlays; the other5overlays are identical. Fresh matrices
compile/run Desktop tests; talos-tools cfg(test) native-inventory tests are not bypassed/edited
there and remain covered on ORIGINAL restored/uninstrumented sources by the separate Windows
workspace job. No fingerprint baseline is refreshed for a temporary probe. Native exacthead CI
must confirm this compatibility; local selftest/classifier14/text23/claims0warnings and diffcheck
pass do not constitute a new Windows execution. Local pinnedRust remains unavailable.
An unfiltered instrumented talos-tools --lib run would intentionally fail the native source
inventory because the temporary boundary differs; the optional exact controlled-fixture mode
filters tests and proves no inventory acceptance. Do not bypass/update that guard. Existing
fresh-phase Desktop matrices and separate uninstrumented full-workspace job remain the required
path. Independent incoming-compatibility review confirms these cfg(test)/anchor limits.

## CI3224 And Later-Sample Observation — 2026-10-18

CI3224/run39030619999 exact0b86ac3b9dce750283ea07aa391e3961f87804b6 attempt1 SUCCESS:
all8jobs, both60process matrices each1110passed/0failed/0ignored; combined120/2220/0/0.
All6normalized source hashes/12rawWindows restoration receipts and artifact SHA256/CRC/extracted
bytes independently verified. Baseline12100646363 (65files) SHA256
878246e86996ad8aef9e92b0ec2d3da19a7922e205afe22cf237c00b3f902c73; instrumented12100730324
(67files) SHA256273f137f6680cf6d0e08eec205fc1712bd9a66f2097fc9c1d8e4f7c5b2b3d0e8.
Expiration2026-10-31. Both event receipts match parent3652/child7868/handle480/waiter146,
creation134367121760719007/exit134367121782616966; all60 new identity/lifetime pairs intact.

| Recorded interval | ms |
|---|---:|
| Creation to exit | 2189.7959 |
| Creation to logged40961 | 985.4130 |
| Logged40961 to40962 | 754.9076 |
| Logged40962 to exit | 449.4753 |

Six600s/400/403 match the same lifetime; first600at1680.0342ms aftercreation,400at1733.8262ms,
403at2156.3634ms. These are recorded marker intervals, not exact first executed code, exclusive
initialization costs or historical-failure cause. Independent Agent read-only review APPROVE
bounded evidence; #705 comment6234327173. CI3222 unavailable receipts remain separate evidence.

Offline crossrun review (#705 comment6237240831) found CI3214 later single23 lifecycle1978.9712ms,
single24at1029.3306ms despite first2662.0459ms. Single23 parent1428/child5320/handle328/waiter4:
assignment toexit1821.6983ms, exit toobservation10.4224ms, waitreturn topipejoin0.1547ms,
CPU0.1875to0.296875s with read counters advancing. No associated event query selected that child.
CI3214 has60/60 intact lifetimes, median172.40175ms; CI3220 has59/60, median224.5244ms,
parallel2 fragmented, so first6126.0167ms is only the longest OBSERVED intact lifetime.
CI3224 first2189.7959ms is longest of60, median168.38505ms, second487.997ms. Thus later children
can also be slow; first-only startup attribution is unsupported. Shared Agent review is not
distinct human approval. All old failures/evaluator5s/old10s/15s/Plugin unknowns remain retained.

Next explicit hypothesis: a long later child's recorded delay may precede40961, fall between
console/provider/engine markers, or follow them. Existing first-only evidence cannot distinguish
these alternatives. Preserve first-child queries and additionally select the longest valid
OBSERVED later lifetime from fixed50single+9parallel logs after the original full matrix and
successful six-source restoration. The selected case is not retried. Results/exit codes and
all original assertions remain unchanged. No extra test process or prewarm is introduced.

Selection scans only59 literal matrix paths, <=8MiB per source and <=16MiB aggregate plus one
overflow-detection byte. Each iteration requires45s left in the existing35min script budget.
Missing, fragmented, invalid or overflowing FILETIME records count as invalid; no exit is inferred.
Receipt later-child-selection.json reports expected59,valid,invalid and complete/partial coverage,
the selected fixed case/PID/lifetime, or unavailable on budget/byte exhaustion/no valid records.
Ties retain fixed matrix order. Partial coverage establishes only the maximum among valid records,
never the missing cases or timed-out children without usable exit receipts. Valid exit metadata
can follow cleanup/cancellation and is not proof of natural exit.

Up to two additional postrestoration queries reuse the existing fixed channels/providers/IDs,
ownedPID/lifetime bounds,16accepted events/131072bytes,30squery/5sreap/5sreaderjoin and45s remaining
gate EACH. Same35minscript/40minstep/45minjob budgets; no added time. Collector re-reads the fixed
selected source and requires the exact selected tuple before launching. Arbitrary source paths
are rejected. Separate later-child-engine-events.json/later-child-console-events.json preserve
first receipts; unavailable selection writes explicit unavailable event receipts without launch.
Only fixed case label/scalar identity/time/counts survive; no event body/command/XML/error text.
No logging/policy enablement, otherPID observation, native API, Rustoverlay or production change.

Selftests cover complete/tied and partial58/1 ranking, all59missing, LF/CRLF, parser FILETIME
overflow as invalid, source-path/window mismatch with noPopen, scan byte/budget exhaustion,
actual wrapper dual-channel receipts and first-receipt preservation, sequential remaining-margin
skip after scan+classic query. Existing cleanup/thread/reap/caps/privacy fixtures remain.
All6generated Rust overlays must match exact0b86ac3 byteforbyte against the same current sources.
Selftest/classifier14/textboundary23/claims0warnings/diffcheck pass; localCargo remains absent,
so no fresh pinnedRust/fullpreflight success claim. Exactcandidate independent review/nativeCI
remain required, not acceptance from these local checks.
Fresh projectgovernance attempt FAILED1error/0warnings: SQLite consumer metadata cannot invoke
missing cargo; delivery12fixtures pass. Keep this environment failure explicit.

Currentmain8fa2a50016143b7c12f4fe7652d75078ab30f006 incorporates separate #731 I305 closeout and
#732 dormant Search ownership. Synchronize main wholesale; no takeover or changes to their
production/native/acceptance/governance/dependency/lock content. Workflow preserves incoming
installed-module collection and unchanged37line705job; only same3diagnosticfiles differ main.
I301/I304/I305 closed slices stay closed; #717/Search residuals retain their owners. Rust1.99.0,
Auto15s/cleanup10s/tool120s/evaluator5s and all current-main assertions stay. No retry-to-green,
ignored test, threshold relaxation, merge/release or overall timeout-root closure authorization.

## CI3229 Two Failures And Evaluator-Entry Probes — 2026-10-18

CI3229/run39096530310 exact1cfdc8b5c4ecaec402570a8ccfbdfb881bda573d attempt1 FAILED.
Six ordinary jobs passed; BOTH diagnostic jobs failed. Each60process phase completed with
1109passed/1failed/0ignored, combined120processes/2218passed/2failed. No retry replaces either.
All6normalized source hashes/12rawWindows restoration receipts match reviewed production.
Artifacts expire2026-11-01; ZIP SHA256/CRC/extracted bytes independently verified:

- baseline12125251606 (65files):8120ffa00c22a5cb28beb743ff1327ac2ef65866fd53493a3f54c5d1e5d80a55;
- instrumented12125255614 (70files):0741a6c0b77275de4c2ac8508ee02bd5fb0fe4ea50abdfd6dfc98197bc549fe8.

Failure1: baseline-suite-first-1 original Auto15s watchdog; decision127.6654ms/request170.825ms,
assessment/report present, no result/approval/explanation. Original auto_tests.rs152 assertion
fails; baseline provides no owned-child timing attribution.
Failure2: instrumented-suite-parallel-5 production_evaluation_uses_authorized_artifacts_and_rejects_stale_or_superseded_results
fails actual observations reach assessor:Elapsed(()). Source verifies the EXISTING10s
entered.notified() assertion (original runtime_host.rs2573–2575; instrumented2616). This is NOT
the historical evaluator composite5s timeout or evaluator30s deadline. Preserve all three facts;
no threshold is changed or conflated.

All60 identity/lifetime pairs are intact; later selector ranks59/59valid/0invalid and selects
parallel5 parent2032/child9024/handle592/waiter144. Both later event receipts match creation
134367709597017054/exit134367709642751378. Its Auto test SUCCEEDS at5.7671577s. Same process
does not make that child part of the failed evaluator test's execution chain.

| Recorded later-child interval | ms |
|---|---:|
| Creation to exit | 4573.4324 |
| Creation to console40961 | 52.5242 |
| Console40961 to first600 | 4448.7875 |
| Six600 span | 1.1222 |
| Last600 to400 | 6.5160 |
| 400 to40962 | 1.0413 |
| 40962 to403 | 59.5547 |
| 403 to exit | 3.8865 |

Exit to native observation7.5488ms; exit to waitreturn7.8211ms; subsequent pipejoin0.1316ms.
Four early CPU samples remain0.09375s/read96753bytes; otherops894to931 initially advance then
plateau. FinalCPU0.25s/read2137062bytes advance. Thus most observed child lifetime is between
logged console markers; this does not identify the CPU-off cause, scheduler, Defender, disk,
exclusive initialization work or the evaluator failure. Firstchild9380 remains valid at
2302.2469ms: creation to40961 1105.8259ms,40961tofirst600695.5073ms. Distinct phase distributions
must not be collapsed into one startup cause. Independent Agent review APPROVE bounded evidence,
#705 comment6245924181; shared-account review is not distinct human approval.

ObservedArtifactAssessor calls notify_one after its artifact assertions. An unsatisfied entered
gate does not establish a lost notification. Evidence preparation can return unavailable before
assessor dispatch; existing probes lack scenario/phase attribution. Next explicit hypothesis:
the10s failure either precedes dispatch (delayed phase or unavailable result), enters an assessor
but fails before notify, or reaches notification yet the observer gate still times out.

Add temporary HOST-overlay markers only: fixed current/stale/new-submit/restart scenario,
Evaluate sent/received/busy/missing-service/future-polled, revision enter/return/unavailable,
context unavailable, artifact capture enter/return/unavailable, service enter/report/failure,
assessor entry BEFORE original assertions, notification return, gate begin/satisfied/timeout.
Target fixture uses current_thread; correlate intact fixed-scenario markers by PID/Rust thread
and sequence, never process identity alone. Missing/interleaved or migrating-thread markers
remain ambiguous; no scenario is reconstructed. No workspace, artifact, request/claim IDs,
goal, content, error body or verdict data is logged. Fixed-stage strings plus time/PID/thread
use a preformatted newline-fenced stderr write; no OS atomicity guarantee follows.
Service report/failure marks service return BEFORE later subject validation; it is not final
delivery success. Some early-return/overflow/output-drop paths remain unmarked, so absent
downstream markers do not identify a unique cause.

No await, branch result, existing assertion, notifier call or time budget is added/removed by
these markers. Await-count guard compares before/after this increment (existing heartbeat
already adds one await). Exactprior-overlay comparison preserves all assert/expect counts and
Duration secs/millis tokens. Other5generated Rust overlays remain byteidentical1cfdc8b against
the SAME current sources. Existing sampling/selection/queries/caps/restoration stay unchanged;
scalar-only diagnostic logging has observation overhead and is not an uninstrumented result.

Local selftest/classifier14/text23/claims0warnings/whitespace pass. Local Cargo is absent; no fresh
pinnedRust/fullpreflight success. Exactcandidate independent technical/security/evidence review
and native CI remain required. Currentmain dea3edccb163420806e08ddeef51c65f977bac59 includes
separate Search#733; preserve all eight incoming files exactly. Same3diagnosticfiles versusmain;
no Search/PSS/I305 takeover, native API, dependency/lock/toolchain, production behavior, ignored
test, prewarm, retry-to-green or relaxed threshold. Rust1.99.0 and all original current-main
assertions/budgets remain, including evaluator composite5s, assessor gate10s, Auto15s,
cleanup10s/tool120s. All earlier failures/Plugin unknowns and closed I301/I304/I305 slices remain.
#705 OPEN/UNMERGED; no merge/release or overall timeout-root closure authorization.
Fresh projectgovernance FAILED1error/0warnings because missingcargo prevents SQLite metadata;
delivery12fixtures passed. Retain this local environment failure explicitly.
