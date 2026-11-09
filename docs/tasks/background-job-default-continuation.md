# BG-CONTINUE-001: Default Background Completion Continuation

> Document status: Refinement / Unclaimed (inactive; #893 is a design candidate only)
> Deliverable: background success, failure and timeout resume the originating live Session without another user message or opt-in flag.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Actor-owned terminal admission and default continuation, cancellation/consumption deduplication, bounded context, tests and docs |
| Claimed At | Not applicable |
| Source Issue | None |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Review of ba402e6d requested changes. This candidate is explicitly inactive; no implementation authorization is asserted |
| Implementation PR | Not started |
| Last Updated | 2026-11-09 |
| Handoff / Release Condition | Accepted successor decision, effective target claim, tested implementation and exact-head review/CI |

## Goal and scope

User explicitly requests default continuation and rejects per-job opt-in. Preserve
ADR-060 historical evidence; adopt a successor decision for its product boundary.
Actor owns admission and dispatch, never TUI. Retain permissions, live Session
identity/generation, shutdown, bounded output and queues. No process launcher,
unsafe, new dependency, process restart recovery, spinner or steering UI work.

## Required Reads

- docs/decisions/100-background-completion-continuation.md (Proposed; not effective)

- docs/proposals/background-continuation-actor-design.md
- docs/decisions/060-supervised-background-command-jobs.md
- docs/backlog/active/TOOL-024-background-command-jobs.md
- docs/sop/AGENT-COLLABORATION.md and docs/sop/TESTING.md
- crates/talos-agent/src/background_jobs.rs, scheduler.rs and session.rs
- crates/talos-core/src/submission.rs and session.rs

## Acceptance

1. Idle Session + authorized success/failure/timeout: mock provider receives bounded
   attributable completion context and continues without another user submission.
2. Busy Session: queue completion until current turn commits; never run concurrent
   provider turns or interrupt the active tool batch.
3. Duplicate notifications/uncertain receipt do not duplicate consumption. A terminal
   result already delivered to the model by process does not cause redundant work.
   Host reads alone are not proof of model consumption.
4. Explicit cancellation, stale generation and shutdown suppress pending wakeups,
   including notifications arriving after cancellation and before dispatch.
5. Queue saturation/output expiry have visible bounded outcomes, no silent loss or
   unbounded retries. Provider errors must not create automatic retry loops.
6. Completion is not user authority, privileged instruction or a second Tool result
   for the launch call. Subsequent tools retain permission checks.
7. Synchronous launch failure already returned does not generate a duplicate turn;
   explicitly decide and test asynchronous monitor failure.
8. Existing recovery fencing, runtime admission, manual compaction and pause states
   remain authoritative. Completion must not bypass unresolved Running custody.
9. Actor-only mock-provider tests and a real CLI smoke test demonstrate continuation;
   foreground behavior and cleanup regressions remain green.

## Selection and dependency checkpoint

Preparation base: 4adefbf4 (#887); isolated branch docs/bg-continuation-claim created
with user approval. Header inventory: I310 Active retained; I277/I302/I318 Review
retained; I249 Planned deferred; I162 Complete with historical Review wording not
reopened. No Blocked header matched; comprehensive owner inventory must be repeated
before activation. Remote open PRs #861/#849/#838 concern search and Gemini; inspect
actual file/claim overlap before merge rather than relying on titles alone.
TUI-ELLIPSIS-001 remains Review for deferred visual acceptance.

## Design readiness

Public SubmissionSource/SubmissionKind are exhaustive enums. Choose private Actor
input versus a reviewed versioned extension and migration before activation. Define
context provenance, cancel epoch, consumed-terminal tracking and admission recovery.
A private inbox alone must not bypass the existing custody/recovery fencing.
The proposal is not an accepted ADR and this owner does not activate implementation.

## Verification and documentation

Pinned toolchain and locked focused agent/core/CLI tests, standard release preflight,
both governance validators, exact-head CI and independent runtime/security review.
README.md, README.zh-CN.md and applicable runtime/tool references explain default
continuation and additional provider usage. Residual destination: this owner.

## Review remediation

User supplied independent REQUEST CHANGES for head ba402e6d against main 4646c02a.
P2 slot lifecycle: ADR-100 now defines exactly-once retirement for successful
continuation without process, terminal provider failure, cancellation, worker
failure and pre-start/teardown paths, with capacity-exhaustion regression targets.
P2 activation contradiction: this candidate is Refinement/Unclaimed, ADR Proposed.
Merging this version would NOT activate implementation. Complete readiness and
atomic claim/activation in this same open PR before seeking activation approval;
do not claim the prior inventory is refreshed or create a redundant activation PR.
The reviewer attributed remote reconciliation failure to global matrix omission
#891, not this PR's runtime behavior. This is reviewer-provided evidence; retain
that separate residual without editing unrelated owners here. Both findings still
require independent re-review of the corrected head. No runtime tests were run.

## Execution ledger

Draft PR #893 publishes design commit c57f8f7245dddd9d4645d7c1c7172ccc1283f454.
Remote checkpoint: Issue/owner reconciliation failed; Rust validation still running;
no reviews/comments returned. Workflow marks remote reconciliation continue-on-error.
Cause remains unknown: full run logs were unavailable while the run was active;
annotations only reported exit 1. Track this residual here until scoped attribution
is known; do not infer it is caused by this slice or repair unrelated owners.
Local governance and collaboration validators passed with zero warnings.
This commit is design evidence only, not implementation completion evidence.

Only source characterization and intake/design drafts exist. No effective claim,
production changes, runtime tests, successor ADR approval or completion evidence.
Next: resolve design, prepare atomic claim/activation and successor decision in a
governance PR, obtain required authorization, implement from that merge or later.