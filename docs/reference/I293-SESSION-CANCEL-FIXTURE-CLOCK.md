# Offline Session Cancellation Fixture Clock

This is bounded fixture maintenance under `docs/sop/AGENT-COLLABORATION.md`, not a new Search
implementation or claim. I293 / SEARCH-001-C and #643 retain their existing owner/status; #705
Windows diagnostics remain separate. No product behavior, public API, permission policy,
dependency resolution, CI routing, release authorization or persistent data changes.

## Observed failure

Data PR #775 at `b65ab9cd7d2885bf1b461f9ac50bdacd5b0463fd` failed CI3463 attempt 1,
run 40360214366, Windows job 121703396044. Its only failed target was
`talos-agent --test i293_caller_cancellation_characterization` (2 passed / 1 failed).
`session_interrupt_releases_owned_tool_before_cancelled_terminal_event` never reached tool
execution: the diagnostic was `Permission denied: approval deadline exceeded`. The four catalog
snapshot cases and Responses terminal regressions passed in that same Windows run. macOS full
preflight and Linux Desktop passed. Code PR #749 / CI3462 passed all six jobs.

The fixture helper sets a 10ms approval budget for all three cases. The Session Interrupt case
used a real clock, although its intended observation is cleanup before the matching cancelled
terminal event. The permission pipeline starts a Tokio Instant deadline before state evaluation
and checks it again after resolver completion. Filesystem/policy work or OS scheduling can consume
the fixture's real-clock budget before the owned future starts. The CI log establishes the early
rejection; it does not locate the exact original delay or establish a production cancellation bug.

## Correction boundary

Run only the offline Session Interrupt case with `#[tokio::test(start_paused = true)]`, matching
the existing approval-versus-execution characterization case. Keep the 10ms approval budget,
5-second logical wait bounds and every assertion: one approval/start, owned future dropped before
the matching cancelled terminal, zero completed work, no returned/delayed ToolResult, and actor
shutdown/join plus drain. The no-resolver denial case and all production clocks stay unchanged.

Tokio's paused clock holds Tokio Instant stable while runnable work proceeds, and auto-advances
to the next timer when idle; logical timeout bounds remain active. It does not freeze std Instant.
Source checked for pinned Tokio 1.53.1: https://docs.rs/tokio/latest/tokio/time/fn.pause.html and
https://tokio.rs/tokio/topics/testing. The source uses Tokio Instant, not std Instant. No upstream
code was copied. The test performs no external HTTP/DNS; this is not a transport-cancellation,
search-budget or real Windows scheduling proof.

## Reproduction and validation

On main `7ca67d9ecd2aa89ef3c66293ef4adebd01c022d0`, a temporary test-only 30ms blocking
sleep in ApproveOnce reproduced the same premature rejection with the original real clock:
exit 101, 0 passed / 1 failed / 2 filtered, denial `approval deadline exceeded`. Changing only the
Session test to start_paused while retaining the injected delay passed: exit 0, 1 passed / 2
filtered. This establishes a deterministic real-budget failure mechanism; it does not locate the
exact delay in the original Windows attempt. The temporary sleep was removed before final checks.
The final code diff is only the test macro and its rationale comment. Final uninstrumented source
validation (pinned Rust 1.99.0 / locked dependencies; four build jobs, zero debug info,
incremental disabled):

- The three-case characterization target passed with zero failed/ignored/filtered.
- Complete `cargo test --locked -p talos-agent`: **538 passed across 16 groups**, zero
  failed/ignored/filtered, exit 0.
- Strict target Clippy (`-p talos-agent --test i293_caller_cancellation_characterization -- -D
  warnings`), format and diff checks passed.
- Standard `release_preflight.sh` passed installer/static checks, both governance validators,
  format, workspace check and strict workspace Clippy. Its unfiltered workspace test phase then
  stopped at inherited MCP `git_prepared`: **26 passed / 21 failed**, reporting socket creation
  denial and `Git maintenance executable differs from the bound Git binary`. Exit 101 is retained;
  no full local workspace/preflight success is claimed and no guard/assertion was modified.

An exact source comparison to the main version confirms that removing the rationale comment and
restoring the original test attribute yields identical bytes: approval helper/budget, wait bounds,
all assertions and the other two cases are unchanged. The existing I293 record describes this offline Agent/Session characterization and
already uses paused Tokio time for its approval/execution case; no owner status or work slice changes.
Open PR/branch inventory found no overlapping fixture maintenance: #776 changes Search manifest
files, #777 changes CI, #773 retains I308 and #705 retains Windows diagnostics.

The original Windows attempt is retained, and one failed-job-only exact-head retry
of #775 was requested without changing that PR's source. No merge or root-cause closure is claimed
from a retry.
