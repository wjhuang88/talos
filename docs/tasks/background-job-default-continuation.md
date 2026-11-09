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

## Source integration checkpoint (candidate tree)

Inspection of `crates/talos-agent/src/session.rs` confirms the dispatch loop gates
pending turns on current turn, fork, manual compaction and pause state (lines
515–519), then calls runtime `commit_start` before `start_submission` (534–546).
`start_submission` currently takes a public `StructuredSubmission` (1251–1256)
and prepares messages from its items (1285–1295). Thus private completion input
cannot simply call this function unchanged without misrepresenting provenance.
Implementation must extract/reuse private preparation and finalization machinery,
not fabricate a user submission or add a parallel unguarded dispatch loop.
These are candidate-source observations, not evidence of implemented behavior or
proof that all target-branch integration seams have been audited.

Remote main decision-directory inspection currently ends at ADR-099; neither
competing ADR-100 is merged. A unique renumbering remains necessary before
activation. No number reservation or other PR modification has been performed.

## ADR namespace conflict discovered during follow-up

Open-PR file inspection found #895 (`fix/clipboard-image-input`) adds
`docs/decisions/100-native-clipboard-image-input.md` and edits the ADR index.
#893 independently proposes ADR-100. Therefore the provisional number is NOT
cleared for activation. Coordinate a unique successor number and update references
before proposing acceptance; recheck target and competing PRs at merge time.
Do not modify #895 or assume either candidate owns the number from PR order alone.

## Follow-up readiness checkpoint

User reports independent remediation re-review passed for e1c77bc2. This is
user-supplied review evidence; GitHub reviews API currently returns no reviews.
Read-only remote verification confirms PR #893 OPEN at
`e1c77bc20a169e99f00b2e9811225f330380820b`, base
`4646c02ad839c8e45fc487c8a99d1bffb8131159`. Exact-head Rust
Format + Check + Clippy + Test, scope classification and Windows installer fixture
passed; Windows workspace and Desktop jobs were skipped. Remote reconciliation
still failed; attribution to #891 remains reviewer-provided, not independently
revalidated here. No claim effectiveness or ADR acceptance follows from this check.

Candidate-tree iteration header inventory was repeated: I310 Active retained;
I277, I302 and I318 Review retained with their existing acceptance gates; I249
Planned deferred. I162 is Complete with a recorded historical Review outcome,
not reopened. TEMPLATE is not an iteration. No Blocked owner header matched.
This is a candidate-tree checkpoint, not yet a complete current-target inventory:
target-tree owner comparison and open-PR file/claim overlap remain activation gates.

ADR-100 already selects private Actor input without extending public exhaustive
enums, requires shared admission/finalization and specifies provenance, cancellation,
consumption and recovery fencing. Verify those integration seams against current
target source before replacing the historical Design readiness entry with a final
activation decision. Keep this candidate inactive until these checks and successor
ADR authorization are complete. Do not reinterpret remediation approval as approval
of a changed activation candidate.

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