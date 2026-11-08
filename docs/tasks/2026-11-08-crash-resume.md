# SESSION-CRASH-001: Resume After Interrupted Execution

> Document status: Active / Claimed (proposed; ineffective until claim #882 merges)
> Deliverable: a session interrupted by process/power loss can accept new user work after safe recovery, without replaying ambiguous tools.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / gpt-6 |
| Work Slice | Interrupted Running-custody recovery, runtime ownership exclusion, durable terminal recovery evidence and resume regression tests; excludes tool display, clipping, theme and permission policy |
| Claimed At | 2026-11-08 |
| Source Issue | None |
| Governance Claim PR | #882 |
| Authorization Mode | Independent review |
| Authorization Evidence | User reports power loss followed by unusable -c recovery and requests a fix; independent safety review and exact-head CI required |
| Implementation PR | Not started |
| Last Updated | 2026-11-08 |
| Handoff / Release Condition | Crash/reopen/new-submission and no-replay tests, concurrent-owner exclusion, locked preflight, MSVC xwin tests and native CI, exact-head review and merged evidence |

## Evidence and acceptance

Screenshot: session 4404450a-443c-479d-8153-f530d10faa21, turn turn24078371_1,
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
