# SESSION-CRASH-001: Resume After Interrupted Execution

> Document status: Complete / Closed
> Deliverable: a session interrupted by process/power loss can accept new user work after safe recovery, without replaying ambiguous tools.

Completion Commit: c7a4b28eac0c0d985c6e24406165638e60aa87e5

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / gpt-6 |
| Work Slice | Interrupted Running-custody recovery, runtime ownership exclusion, durable terminal recovery evidence and resume regression tests; excludes tool display, clipping, theme and permission policy |
| Claimed At | 2026-11-08 |
| Source Issue | None |
| Governance Claim PR | #882 |
| Authorization Mode | Independent review |
| Authorization Evidence | Independent Agent safety/API APPROVE at 2b386161c64643f8b2918b70983c1c29314f14bd against 0adc34519490b85769cc46863420b888bf014e2a; CI42450412604 all successful; merged #884 |
| Implementation PR | #884 |
| Last Updated | 2026-11-08 |
| Handoff / Release Condition | None; scoped acceptance passed and implementation merged |

## Evidence and acceptance

Screenshot: session 4404450a-443c-479d-8153-f530d10faa21; read-only journal
inspection confirms turn `turn_2407837_1_1` (screenshot spacing was ambiguous),
Running submission steering:64b3c82b-a89b-4917-b410-da0bb8986ca1:batch:1 has
no terminal transcript proof; startup rejects all new input with SessionClosed.
Code explicitly preserves this state indefinitely. Previous AGENT-PAUSE-001
addresses a different live-output defect and does not recover this interruption.

Preserve transcript, accepted identities and user input. Never infer success
from partial messages or replay the interrupted submission/tools. Resolve an
abandoned turn as interrupted only with exclusive recovery/runtime ownership;
preserve existing terminal evidence and fail closed on corrupt/unreadable data.
Durably record recovery before releasing Running custody, make recovery idempotent
across another crash, and explain interruption to the user. New explicit user
input must execute normally after recovery. Pending work must retain its existing
pause/admission rules. Concurrent actors must not cancel each other's work.

Hard: ADR-056 no ambiguous replay, no permission relaxation, no secrets or unsafe.
Any refined recovery/ownership contract must be recorded in a decision addendum.
Required reads: ADR-056, Session custody and persistence, runtime transitions,
artifact lifecycle, I169 crash/terminal recovery tests, testing/local-dev SOPs.
Docs: README.zh-CN.md, recovery decision, EVOLUTION.md, this owner; Board mirrors.

## Inventory and closure ledger

Initial investigation base 1f537340; current claim candidate base 7a439172;
implementation starts from claim merge or later. I310 Active;
I318/I302/I277 Review; I249 Planned deferred; I164 Paused superseded; I162 Complete;
no Blocked iteration. Preserve these dispositions. AGENT-PAUSE-001 closes
Complete / Closed in this candidate using merged #881. VALIDATION-DISPLAY-001 remains separately claimed in this
governance PR. TUI-ELLIPSIS-001 and original dirty files remain untouched.
Other open #838/#849/#861 do not own this scope. Existing profile unchanged.

Validate missing terminal with/without partial history, persisted outcomes,
second crash during recovery, simultaneous ownership, errors and new submissions.
Use independent safety review, standard locked preflight and cargo-xwin MSVC
--tests; native CI required. Run both governance validators, staged review and
merge-time CAS. No Source Issue. Residual destination: this owner. Complete only
after existing merged implementation SHA and all required evidence are recorded.

## Implementation and validation ledger

Implementation branch `fix/session-crash-resume` starts at claim merge 0adc3451.
SessionRuntimeLease uses the pinned Rust standard-library file lock; permanent
anchors and canonical parent identity prevent cooperating cleanup/alias paths
from splitting ownership. Actor, Agent, event-forwarder, preview and manual
compaction tasks retain ownership. Entire deletion/index cleanup holds one lease.
Existing older binaries and external filesystem mutation remain outside this
cooperative protocol; documentation requires exiting old clients before upgrade.

Strict host validation streams chunks with a deadline and keeps only unfinished
record data; it accepts supported legacy and large native records, refusing
corrupt/truncated bytes without rewriting them. Recovery pauses unstarted work,
durably records an interrupted Error with one assistant explanation, syncs even
on already-visible terminal retries, and only then finalizes Running custody.
The current model history receives the same deduplicated note. Original turn and
submission identities remain; exact retries never re-execute old tools.

Read-only user journal inspection returned SQLite integrity `ok` and one Running
row. User data was not edited. Original repo dirty files are preserved. Power loss
also left two zero-byte Git files; they were isolated in
`/tmp/talos-git-powerloss-backup` and the remote tracking ref restored by fetch.

Initial full Agent regression passed 662 unit tests on the host. Repeated sandbox
runs reused PID-derived `/tmp` fixture identities and caused unrelated timeouts;
the host run passed those tests. A new model-history assertion caught a System
note filtered from provider history; the note is now Assistant and all six initial
terminal recovery tests pass (`/tmp/talos-crash-terminal-final.log`). Session suite
passed (`/tmp/talos-crash-session-final.log`). Later added second-crash, generation,
sync-failure and cleanup-race tests require final validation below. Preliminary
MSVC all-feature test compilation passed (`/tmp/talos-crash-xwin.log`), with the
existing unrelated checkpoint-history test-only dead-code warning. Full locked
preflight, final xwin, independent review and exact-head native CI remain pending.

Final focused validation: 8 terminal-outcome integration tests and 9 runtime-lease
tests passed; final cargo xwin check --locked -p talos-session -p talos-agent
-p talos-cli --all-features --tests --target x86_64-pc-windows-msvc passed.
A consistent copy of the affected user transcript/journal recovered its one
Running row as TerminalError with original identities, original transcript bytes
retained as a prefix, and no model/tool call. The private fixture probe was removed
after execution and is not shipped. Original user data was not manually edited.

Actual user acceptance on 2026-11-08: after synchronizing the recovery source into
the user's main working directory and building there, the user reran cargo run
-- -c and explicitly confirmed "可以继续了" (new input can continue the original
session). An earlier shared-target binary-only delivery was insufficient because
cargo run rebuilt the old main source. This is recorded in EVOLUTION.md.
Shared-target preflights reached passing ordinary workspace tests but failed
rustdoc against stale dependency artifacts; standalone Agent doctests passed
12/12. Copying that cache to an isolated target retained the stale Session library
and reproduced the failure. After targeted cargo clean of Session/Agent in the
isolated target, all workspace doctests passed. Final preflight uses this cleaned,
isolated target so the user's main cargo run and independent worktrees cannot
overwrite validation artifacts. This cache diagnosis does not substitute for the
required complete preflight below.

Final local checkpoint passed: pinned 1.99.0 locked release_preflight.sh, including
workspace check, Clippy, all unit/integration/doc tests and both external SDK
feature modes (/tmp/talos-crash-preflight-clean.log). Both repository governance
validators, current skill validator and scale assessment passed; profile remains
high-risk/release-managed with worktrees required. Final exact-head independent
review, native CI and merge/closure remain pending.

## Closure

PR #884 merged as the Completion Commit above after independent exact-head Agent
safety/API/technical APPROVE with no blockers, both governance validators and
CI42450412604 all successful, including native Windows workspace/process tests
and Windows/Linux Desktop checks. Agent review shares the GitHub identity and
does not represent a separate human approval. Merge-time CAS confirmed the owner
unchanged, no overlapping claim/implementation, and the reviewed head mergeable
on the latest target (the intervening #883 presentation change was independent).

The user confirmed original-session continuation via cargo run -- -c. Follow-up
read-only verification found the original interrupted turn terminal_error and
SQLite integrity ok. Recovery was performed by the application, not a manual
journal edit. Temporary private-session fixture and probe were removed. Git
connectivity check passed after the earlier power-loss artifact recovery.
Documented cooperative-lock and existing whole-file-reader limits remain design
boundaries, not omitted acceptance. No residual implementation within this scope.
