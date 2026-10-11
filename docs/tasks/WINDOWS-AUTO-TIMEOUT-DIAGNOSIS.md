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
