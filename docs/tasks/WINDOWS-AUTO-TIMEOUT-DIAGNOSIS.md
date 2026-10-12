# Windows Auto Timeout: Bounded CI Diagnosis

Date: 2026-10-11. Baseline: `444a8137f054b33802f26bffdb55fb0f83a7a861`.

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
