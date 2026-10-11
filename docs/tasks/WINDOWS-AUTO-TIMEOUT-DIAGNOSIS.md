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
