# Desktop Four-Week Delivery Task

> Document status: Planned
> Plan date: 2026-09-22
> Target window: 2026-09-22 through 2026-10-19 (four calendar weeks; approximately 20 working days)
> Execution state: planning only; no new implementation claim is effective.

## Outcome

Deliver a usable local single-client Desktop candidate: create a task in an explicit workspace,
run a real configured model and tools, observe activity, approve or deny requests, cancel safely,
resume persisted conversations, and inspect actual work/evaluation evidence. Preserve the accepted
GPUI visual direction and Chinese/English interaction. This is a development candidate, not a
promise to ship the entire Desktop product or publish signed installers within four weeks.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Planning container for DESKTOP-001-D4 through D7 / I282-I285; implementation authority belongs to serial child claims |
| Claimed At | Not applicable |
| Source Issue | #29 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Maintainer requested a roughly four-week Desktop task and iteration plan on 2026-09-22; prior single-maintainer and Agent-role review preferences retained |
| Implementation PR | Not started |
| Last Updated | 2026-09-22 |
| Handoff / Release Condition | Activate only a ready child with an effective target-branch claim; no release or migration authority implied by dates |

## Published Baseline

### In Scope

- Existing GPUI surfaces and design references, real Runtime composition and provider configuration.
- One local client; one executing task at a time, with explicit workspace and session identity.
- Streamed content, reasoning/tool activity where exposed, real errors and terminal outcomes.
- Existing permission and Auto behavior with visible explanations; explicit scoped approval,
  denial, cancellation and shutdown; no new permission policy.
- Shared durable Session resume, recent-session navigation, and actual work/evidence projections.
- Read-only artifact/change inspection when backed by actual execution/workspace evidence.
- Existing independent evaluation contracts exposed honestly; no fabricated PASS or Delivery.
- Bilingual user documentation, deterministic integration tests and a guided final native walkthrough.
- Final stabilization and a repeatable local candidate build.

### Out Of Scope

- Full #308 Preset/Session Environment and MODEL-012 role routing; existing single-model config is
  sufficient for this cycle. Mock presets must not silently configure real execution.
- Multi-window, concurrent clients, remote attach/reconnect and SESSION-009 implementation.
- A new work engine, Desktop-owned business store, new permission authority, global event bus,
  general scheduling, or unattended autonomous Mission orchestration.
- A new durable Mission/Evaluation schema or data migration without a separate accepted contract.
- Signed installers, auto-update, store submissions, release tags, GitHub/crates publication.
- #502 libc/advisory research, MODEL-007 catalog expansion, #590 language inference, and unrelated
  dependency upgrades. Follow up on the existing owners; do not erase these requirements.

### Work Mode And Validation Tracker

Work mode: Deferred Human Validation.
Deferred validation tracker: existing GitHub Issue #29; final cleanup iteration I285.
Reuse the existing tracker rather than creating another Issue or one Issue per task. This follows
the maintainer's explicit instruction to keep local loops local and the already accepted #29
deferred-validation queue; it is a scoped scheduling choice over the SOP default of creating a
new tracker at activation. Before I282 activation, add one cycle acceptance ledger to #29 and link
it here. Until that update, this remains a plan rather than active unattended execution.

Required human rows for this cycle: real-provider task; real allow/deny/cancel; restart/resume;
Chinese IME and English/Chinese layout; coherent final integrated flow. Each row records source
owner, exact build/head, device and result. Automated tests and Agent security review cannot be
deferred. Source owners stay Review while their required human rows are outstanding. Following
serial implementation may proceed after the predecessor's implementation is merged and its
technical gates pass, with the outstanding human rows explicitly carried forward.

| Row | Source | Human scenario | Evidence state |
|---|---|---|---|
| H1 | I282 | Launch with configured provider, submit one real task, observe content and provider-error handling | Not run; head/build/device pending |
| H2 | I283 | Observe existing Auto decision, approve one bounded request, deny another and verify actual workspace effects | Not run; head/build/device pending |
| H3 | I283 | Cancel while streaming, while a tool runs and while approval is pending; verify stopped state and no stale approval | Not run; head/build/device pending |
| H4 | I284 | Exit/relaunch, resume the selected session, switch between two saved sessions and verify isolation/no repeated write | Not run; head/build/device pending |
| H5 | I282/I285 | Chinese multiline IME, English/Chinese Settings, focus, scroll and layout during live output | Not run; head/build/device pending |
| H6 | I284/I285 | Integrated real task through result/change inspection; distinguish missing/stale/current evaluation and Delivery eligibility | Not run; head/build/device pending |

At I285, retest interacting behavior at the final integrated head. Record application faults and
provider nondeterminism separately; a scripted fixture is not evidence of real-provider acceptance.

I277's already deferred VoiceOver, reduced-motion, Windows/Linux native interaction and physical
display measurements remain separate #29 rows. They are not retroactively waived or repeated as
passed. Regressions caused by this cycle must be corrected; unavailable pre-existing device rows
remain owned residuals and prevent claims of universal accessibility/platform acceptance.

### Ordered Task Items

| ID | Window / iteration | Expected output | Depends on | Completion gate | Fallback | Status |
|---|---|---|---|---|---|---|
| T1 | Week 1, Sep 22-28 / I282 | Launch a live Desktop task using configured provider; responsive streaming, errors and cancellation; tools unavailable in this initial slice | WORK-001 P0-P4, I280, ADR-059; API map and effective claim | Mock-provider E2E plus real-provider native row; no UI blocking or fictitious results | Keep existing mock explicitly separate; unresolved facade contract becomes a named blocker, never a copied engine | In Progress — claim/activation PR #598 |
| T2 | Week 2, Sep 29-Oct 5 / I283 | Real file/shell activity, visible existing Auto review, scoped approval and safe cancellation | T1 merged and technical gates passed | Allow/Deny/Once/Session, stale approval, cancel and shutdown matrix; independent security/API review | Tool execution remains disabled until gates pass; do not ship an auto-allow workaround | Complete — PR #603 merged as `93c8b357`; H2/H3 human residual |
| T3 | Week 3, Oct 6-12 / I284 | Recent tasks, durable transcript resume, actual work/evaluation and read-only change/evidence views | T2 merged; storage/projection compatibility verified | Restart and session-isolation tests; revision/staleness/Delivery gates; no tool replay | Preserve transcript-only recovery if richer projection cannot persist; explicitly show unavailable evidence and keep unmet acceptance open | Planned |
| T4 | Week 4, Oct 13-19 / I285 | Reproducible integrated candidate, fixes, user guide and consolidated acceptance report | T1-T3 merged with technical gates passed | Integrated E2E, current-head CI/review, required native rows, documented residuals and clean handoff | Deliver Partial with exact remaining blockers; no fake Complete or unrequested release | Planned |

Reserve roughly three days in Week 4 for fixes/retesting and two for acceptance/documentation.
The dates are planning targets, not autonomous wall-clock scheduling or a guarantee. Each weekly
checkpoint compares accepted behavior against the baseline; do not count PRs or documents as
feature progress. Report slips with their impact. Do not silently drop required behavior to fit dates.

### Dependencies, Readiness And Decisions

Confirmed: RuntimeHandle exposes submit, interrupt, next_event and structured shutdown;
RuntimeBuilder accepts a provider, approval handler and durable Session. WORK-001 P0-P4 and I280
are complete. These facts support planning but do not prove the entire Desktop adapter exists.
I282 must map event ownership/backpressure, provider/config loading and host lifetime before coding.
I284 must inspect the persistence and shared evaluation facade rather than assume storage-neutral
P4 provides a durable Mission store.

Hard: ADR-059 UI/Tokio separation; runtime/session/work/permission authority; no secrets; protected
security review; public API compatibility and migration decision where required.
Soft: four-week target, macOS-first native acceptance using the available environment, serial WIP=1.
Assumptions: existing facade supports a thin adapter; shared projections can be consumed without
new durable schemas. Resolve in the owning iteration before implementation; changes outside
accepted decisions require a bounded decision record, not speculative implementation.

### Existing Iteration Inventory And Priority Disposition

Checked current iteration status declarations and the iteration index on 2026-09-22, before
creating I282-I285. No pre-existing Active or Blocked execution iteration was identified.

| Owner | Current truth | Disposition |
|---|---|---|
| I277 | Review / Claimed; mock implementation merged, human/device rows deferred | Preserve scope and evidence; #29 carries deferred checks; no mock-scope expansion |
| I249 | Planned dependency pilot | Remains deferred; #502 investigation is separate and not in this cycle |
| I164 | Paused, superseded layout target | Retain historical baseline; not reactivated |
| I162 | Terminal predecessor; completion belongs to its existing owner | The word Review in its recorded outcome is not an open iteration |
| I280 / I281 | Terminal predecessors; completion belongs to their existing owners | Reuse delivered SDK and permission behavior; do not reopen |
| MODEL-007 follow-up | Refinement / Unclaimed, previously next priority | Maintainer's Desktop request now takes priority; deferred, not cancelled |
| DESKTOP-002 / #308 | Blocked / Unclaimed | Preserve model-role/environment dependencies; not a blocker for one configured model |
| #502 / DEPENDENCY-003 | Refinement / Unclaimed | Dedicated research handoff already posted; outside Desktop scope |

Recheck non-terminal owners, open PRs and branch/worktree ownership before each activation.
At planning time no open PR was found. New unrelated issue reconciliation must not become an
edit-by-edit implementation gate.

### Artifacts And State Owners To Update

- Child story owners DESKTOP-001-D4/D5/D6/D7 and iteration owners I282/I283/I284/I285.
- Parent DESKTOP-001, this task, iterations README, compact backlog, Board and manifest selection.
- User-facing Desktop README (to be created in I282), bilingual root README/site when delivered
  behavior changes, and changelog at candidate handoff. Do not advertise planned behavior as shipped.
- ADR-059 and applicable API/migration records only if a real decision change is needed.
- Existing #29 holds stage evidence and consolidated manual results; #308 retains preset scope.

### Validation And Acceptance Evidence

Inner loop: focused locked tests for changed crates, deterministic fake providers, controlled
clocks/barriers and explicit timeout diagnostics; no real sleeps as ordering assertions.
Stage gate for code: repository-pinned toolchain, focused Desktop/Runtime/permission/session tests
as affected, full release_preflight without a version/tag argument, both governance validators,
staged-diff/secret review and independent Agent-role review for technical/security/API scope.
CI is queried from the PR; retain exact head/base/job links. Windows-only behavior uses GitHub
Actions where no native Windows runner is available. Compile success is not native UI acceptance.
Docs-only planning/status changes use governance/link/diff checks, not Rust workspace builds.

### Branch, Worktree And Checkpoint Plan

Single-maintainer, one active implementation branch/worktree at a time. Work locally until the
weekly deliverable converges; normally one stable implementation PR per iteration, not per test,
UI adjustment or review finding. Batch same-slice corrections in that PR. Required claim and
owner-first evidence closure use the existing SOP; combine compatible status work rather than
inventing extra checkpoint PRs. Do not pre-activate dependent iterations to bypass claim gates.
Keep merged predecessor code on main before starting the next child; preserve uncommitted work.

Append a checkpoint after each stage and before stopping: actual changes, checks, head/base,
remaining acceptance, next command/gate and resume owner. Completion Commit must name existing
implementation evidence, never the status-only commit itself.

### Allowed Permissions And External Actions

Current request authorizes creating and aligning this plan. Established single-maintainer
workflow and subagent review preferences carry forward; no new spending, credentials, deployment,
release or destructive database operation is authorized. At activation record concrete claim/PR
authority and use repository gates. Do not send messages to external researchers.
Real-provider acceptance uses the maintainer's configured provider only with its existing
authorization; automated regression suites use fixtures, not paid requests.
No dependency is added speculatively; check then-current upstream versions and compatibility if needed.

### Time, Cost And Resource Limits

Target four weeks; WIP=1. Reuse build caches, avoid parallel full workspace builds and extra
checkouts. Check disk space before large GPUI builds; remove only verified expendable task-owned
artifacts, preserving current acceptance binaries until replaced. No benchmark or token-cost
promises without measurement. No external service purchases or unbounded model retry loops.

### Failure, Retry And Fallback Policy

Diagnose each failure before retrying. At most two unchanged transient retries; deterministic
failures return to local correction. CI running longer than a short polling interval is expected,
not stalled; poll every 60 seconds and inspect job/step progress before intervention. Cancel only
superseded or demonstrably stuck task-owned runs, never a valid green candidate.
Missing device access defers its row; security/API blockers stop the affected integration, not
unrelated local work. If a week slips, update forecast and preserve acceptance.

### Default Decisions And Residual Destination

Preserve existing visuals, use Settings rather than reintroducing language popovers, reuse current
single-model configuration and selected workspace, fail closed on permissions and unknown results,
and keep optional features out. #29 and owning child retain Desktop gaps; #308 owns real presets;
#502 owns dependency research; MODEL-007 and #590 retain their existing owners.
No unrelated cleanup, automatic dependency upgrade, new per-subtask Issue or release.

## Execution Checkpoint — 2026-09-23 (historical checkpoint, superseded)

Completed task items: none; I282 implementation is locally converging.
Current state: I282/D4 implementation is merged to `main` as `aaa4c015` through #599. I282/D4
remain Review because H1/H5 native acceptance is outstanding in #29. I283-I285 remain Planned /
Unclaimed and are not activated. I282 now has the live Runtime host, configured provider
adapter, typed output/completion/error handling, bounded presentation buffering, explicit no-tools
behavior, shutdown receipts, and cancellation during provider-backed history compaction.
Focused evidence: Desktop binary tests 61 passed; Runtime interrupt tests 2 passed. The stable
candidate #599 passed exact-head CI `35766202595`, independent Agent-role review found no
technical/API/security blocker, and merge-time CAS produced `aaa4c015`. H1/H5 native rows remain
deferred in #29.
Next item: record this owner-first Review closeout, then activate I283 only through its own
effective claim; do not mark the four-week task complete yet.
Resume: read this task and I282, retain the current branch/worktree, inspect the full diff, and
preserve I277 deferred rows.
Validation: workspace tests, affected checks, release preflight, governance validators and
PR #599 exact-head CI passed. Human H1/H5 acceptance remains pending.
Independent Agent-role planning review approved after H1-H6 numbering was added; this is not a
natural-person review, implementation security approval or proof of the four-week estimate.
No Rust tests were run for this governance-only activation candidate. Implementation remains
unstarted until #598 reaches main.
Completion Commit: pending.

## Submission Checkpoint — 2026-09-22

The maintainer authorized committing/pushing this plan and preparing a clean development starting
point. Deliver through one documentation PR using the existing single-maintainer path; this is
planning publication, not I282 implementation activation. Preserve the separate open #596
DEPENDENCY-003 research claim and its branch. Shared Board/backlog edits use union semantics;
recheck target main at merge time. No new per-subtask Issue, code build, release or dependency
change is part of this submission. Next work remains I282 readiness mapping and effective claim.

## Local Review Correction Checkpoint — 2026-09-23 (historical checkpoint, superseded)

This checkpoint supersedes earlier next-action descriptions without rewriting the published
baseline. I282 implementation is merged as `aaa4c015`; its H1/H5 native rows remain outstanding.
I283 is complete after implementation PR #603 merged as `93c8b357`; H2/H3 remain explicit human
acceptance residuals in #29. I284 and I285 remain Planned and are not activated by this closeout.

I283 candidate `cbb156433f9119166150988264a84945cf003726` passed CI, but local review corrections
are uncommitted and are not covered by that CI. The I283 owner records the actual tool/approval
and Auto integration tests, manual-fallback preservation, and workspace-cwd correction. Runtime
API documentation now explains explicit Interactive mode and unchanged Headless defaults.
Neither a green older candidate nor pending-approval cancellation proves running-tool cleanup.

Next required decision: proposed ADR-082 defines Unix sandbox process ownership and cancellation.
Maintainer acceptance is still pending; no new sandbox unsafe supervisor is implemented or
authorized by automatic task continuation. The decision must distinguish ordinary owned-group
cleanup from enforced containment of descendants deliberately leaving the group. Keep this
acceptance open in I283/#29; do not silently narrow it or claim whole-tree cleanup.

Resume on the existing branch, preserving all local corrections. After the decision, finish
running-tool cancellation and its shutdown receipt, converge tests and protected review locally,
then push one stable correction candidate to #603. No new iteration, release or separate
subtask Issue is activated. H1-H6 remain unverified human rows; the four-week task is incomplete.

## Cancellation Decision Accepted — 2026-09-23 (historical checkpoint, superseded)

The maintainer explicitly accepted ADR-082's bounded owned-group cancellation guarantee.
The decision gate above is resolved. I283 now implements and validates the supervisor,
cleanup receipt and Runtime/host shutdown integration locally. Deliberate process-group
escape containment remains unimplemented under I283/#29, not silently completed.
Independent protected review and exact-head CI subsequently passed for `9163bf82`; PR #603 merged
to `main` as `93c8b357`. H2/H3 remain unverified human rows; I284/I285 remain unactivated.

## I283 Technical Closeout — 2026-09-23

Implementation PR #603 merged to `main` as `93c8b357` after CAS. Exact-head CI run
`35825870138` passed all six jobs for head `9163bf823579c7111a0a0e3214fdb46e30345d43`.
Independent Agent-role security/API review conditionally approved the same head and disclosed
shared-workspace identity limits. I283/DESKTOP-001-D5 are technically complete; H2/H3 remain
the only known human acceptance residuals. I284 and I285 stay Planned/Unclaimed.

The local stable candidate subsequently passed the full pinned preflight on 2026-09-23:
Sandbox 40/40 plus 2 doctests, Runtime 41/41, Desktop 76/76, talos-skill 81/81,
governance validators with 0 warnings, full workspace Clippy/tests, and the external
Runtime SDK fixture. The formerly pending exact-head remote validation and fresh review were
subsequently completed by CI run `35825870138` and the independent review bound to `9163bf82`;
I283 then merged as `93c8b357`.
