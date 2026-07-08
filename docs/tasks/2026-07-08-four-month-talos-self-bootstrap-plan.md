# 2026-07-08 Four-Month Talos Self-Bootstrap Plan

**Status**: Planned
**Created**: 2026-07-08
**Timebox**: 16 weeks / roughly 4 months
**Owner boundary**: Talos-primary self-bootstrap attempt; maintainer or senior agent reviews
monthly closeouts and release-gate classifications
**Primary objective**: prove, or honestly fail to prove, REL-002 by assigning the next long-running
development sequence to Talos as the primary runtime.

## Assumptions And Constraint Classification

Hard constraints:

- This plan does not authorize release tags, crate publishing, GitHub Releases, deployments,
  external trial invitations, destructive cleanup, new secrets, or credential/schema changes.
- Push to `main` is allowed only when the specific execution contract for that Talos run grants it;
  otherwise commit locally and record the missing push as a residual.
- No permission allow/deny precedence changes, sandbox relaxation, process-hardening changes,
  session-storage default migration, runtime `catalog.db` restoration, or unreviewed `unsafe`.
- No new runtime dependency unless the responsible owner doc, ADR, and review are complete first.
- Behavior changes require real runtime evidence through the `talos` binary or Talos-owned test
  harnesses, not only unit tests.
- REL-002 qualification requires Talos to be the primary development executor. External-agent code
  or doc edits convert the affected session to partial or non-qualifying evidence unless the edit is
  explicitly limited to review commentary.

Soft constraints:

- Prefer low-risk, reviewable packets that can be completed in one or two commits.
- Prefer existing backlog stories over new requirements.
- Prefer read-only diagnostics, governance evidence, and bounded user-facing polish before
  architecture-sensitive changes.
- Keep each monthly iteration independently closeable even if REL-002 remains unmet.

Assumptions:

- Talos can be invoked locally with a configured provider capable of repository-scale development.
- The maintainer can provide external review, but external review is not allowed to silently become
  the primary executor.
- The current starting point is post-I105 closeout: controlled local trial is GO, `v1.0.0` remains
  NO-GO because I102-I105 were external-runtime primary.

## Outcome

Deliver four Talos-primary monthly iterations that attempt a full self-bootstrap qualification:

1. prove the execution/evidence harness can distinguish qualifying, partial, and non-qualifying
   Talos runs;
2. complete at least one user-facing feature or polish change with Talos as primary executor;
3. complete one architecture-sensitive but bounded change or audit with correct risk routing; and
4. close REL-002 with a concrete v1.0 go/no-go report.

If Talos cannot carry any part as primary executor, the result is still useful only if the owner docs
record exactly where the self-bootstrap attempt failed.

## In Scope

- Talos-primary runbook, checkpoint format, and evidence classification.
- Real binary smoke harness for Talos development sessions.
- Owner-doc and board synchronization performed by Talos-owned commands or Talos-guided edits.
- One low-risk user-facing feature or polish change selected from existing backlog.
- Corrective work for issue-audit findings that were closed inaccurately: #18 request-dispatch
  timeout, #28/#39 dashboard notification behavior, #24/#31 visual/runtime evidence gaps, and #26
  thinking history policy.
- One architecture-sensitive bounded change, audit, or diagnostic selected from existing owner docs.
- REL-002 evidence updates, readiness report, and final go/no-go classification.

## Out Of Scope

- No `v1.0.0` tag, release, publish, or external trial invitation during the plan.
- No permission-policy redesign, sandbox/process hardening, workspace-trust implementation, or
  credential persistence change.
- No session-storage default migration unless a separate SESSION-004 gate is activated.
- No remote plugin marketplace, executable hooks, browser automation, PDF/Office/OCR ingestion, or
  write-capable plugin tools.
- No broad autonomy loop, global event bus, background scheduler, or multi-agent orchestration.
- No claim that partial Talos assistance satisfies REL-002.

## Existing Work Inventory And Disposition

This inventory satisfies `docs/sop/START-ITERATION.md` before introducing I106-I109. It preserves
published baselines and does not silently re-scope older iterations.

| Area | Current State | Disposition For This Plan |
|---|---|---|
| R27 High-Risk Governance Gate | In Progress standing gate | Keep as oversight only; it grants no implementation authority. |
| I018/I028 | Planned historical baselines | Deferred; not part of the self-bootstrap attempt. |
| I081-I083 | Planned superseded remainder | Keep as historical shells; do not activate. |
| I085 | Paused | MC107 walkthrough remains residual; may be used only if selected explicitly by I107. |
| I086-I089 | Planned product-hardening shells | Keep planned; I106-I109 are the new Talos-primary REL-002 attempt. |
| I102-I105 | Complete | Starting evidence baseline; useful controlled-local-trial evidence, non-qualifying for REL-002. |
| RUNTIME-002 / PROVIDER-002 | Partial after 2026-07-08 audit | #18 request-dispatch timeout remains open; select before lower-priority feature polish. |
| TUI-028 | Partial after 2026-07-08 audit | #28 reopened as #39; #24/#31 need visual/runtime evidence; #26 split to TUI-029. |
| TUI-029 | Planned — decision required | #26 thinking content history archive requires ADR-034/TUI-020 policy decision before implementation. |
| TOOL-020 | Planned | Candidate for I107 user-facing/read-only polish if Talos can execute it safely. |
| ARCH-032 | Planned | Candidate for I108 architecture-sensitive audit/routing work. |
| SESSION-004 | Ready for Implementation | Do not activate unless the maintainer explicitly accepts the storage-default migration gate. |
| REL-002 | Planned, not ready | Primary release gate targeted by this plan; no release action is authorized. |

## Four-Month Execution Matrix

| ID | Week | Iteration | Track | Deliverable | Validation | Status |
|---|---:|---|---|---|---|---|
| SBT100 | 1 | I106 | Start gate | Talos-primary execution contract, inventory, and disqualification rules recorded. | governance validation; clean worktree evidence | Planned |
| SBT101 | 1 | I106 | Evidence schema | Checkpoint template classifies qualifying, partial, and non-qualifying sessions. | owner-doc review; sample record | Planned |
| SBT102 | 2 | I106 | Runtime smoke | Repeatable Talos binary smoke harness covers version, validation, governance preview/write dry run, provider failure, and session resume evidence. | real `talos` commands or Talos-owned tests | Planned |
| SBT103 | 3 | I106 | Governance rehearsal | Talos performs a bounded owner-doc update through the accepted governance path and records rollback evidence. | governance validation; diff review | Planned |
| SBT104 | 4 | I106 | Closeout | Month-1 harness result classified for REL-002 without overclaiming. | workspace validation as needed; REL-002 update | Planned |
| SBT110 | 5 | I107 | Selection | Select from the issue-audit corrective queue first: #18 request-dispatch timeout, #39 dashboard transient notification, #24/#31 visual evidence, then TOOL-020/I085 only if higher-priority residuals are closed. | owner-doc activation note | Planned |
| SBT111 | 6 | I107 | Implementation | Talos implements the selected corrective change using existing patterns and permission-gated tools. | targeted tests; real binary evidence | Planned |
| SBT112 | 7 | I107 | Docs sync | Talos updates user docs, backlog, iteration, and board in owner-first order. | governance validation; docs diff review | Planned |
| SBT113 | 8 | I107 | Closeout | First non-trivial Talos-primary feature/polish session classified for REL-002. | workspace tests or recorded fallback | Planned |
| SBT120 | 9 | I108 | Risk routing | Activate ARCH-032 or another bounded architecture-sensitive item with explicit risk classification. | owner-doc activation note | Planned |
| SBT121 | 10 | I108 | Architecture work | Talos performs the audit or bounded diagnostic/change without crossing permission/storage/dependency gates. | architecture evidence; targeted tests if code changes | Planned |
| SBT122 | 11 | I108 | Review gate | External review checks architecture claims without becoming primary executor. | review findings resolved or recorded | Planned |
| SBT123 | 12 | I108 | Closeout | Architecture-sensitive Talos-primary session classified for REL-002. | governance validation; REL-002 update | Planned |
| SBT130 | 13 | I109 | Final session | Complete a third non-trivial Talos-primary session or record why it could not qualify. | targeted tests; runtime evidence | Planned |
| SBT131 | 14 | I109 | Evidence audit | Audit all self-bootstrap sessions against every REL-002 acceptance criterion. | evidence matrix | Planned |
| SBT132 | 15 | I109 | Readiness report | Produce v1.0 go/no-go report with residuals, rollback, and next owners. | docs review; governance validation | Planned |
| SBT133 | 16 | I109 | Final gate | Close the four-month plan with no release overclaim. | workspace validation; maintainer review | Planned |

## Monthly Exit Gates

| Month | Exit Criteria |
|---|---|
| Month 1 / I106 | Talos-primary execution can be measured repeatably, and non-qualifying external intervention cannot be hidden. |
| Month 2 / I107 | Talos closes at least the P0 #18 request-dispatch timeout residual or records a precise blocker; lower-priority polish follows only after that gate. |
| Month 3 / I108 | Talos routes and completes one architecture-sensitive session without bypassing review, or records a precise blocker. |
| Month 4 / I109 | REL-002 has an evidence-backed GO/NO-GO report and no `v1.0` claim unless every acceptance criterion is met. |

## Required Reads

Before starting any task, the Talos executor must read:

1. `AGENTS.md`
2. `docs/sop/LONG-RUNNING-TASK.md`
3. `docs/sop/START-ITERATION.md`
4. `docs/sop/ITERATION-WORKFLOW.md`
5. `docs/sop/GIT-WORKFLOW.md`
6. `docs/sop/DOC-CHECK.md`
7. `docs/BOARD.md`
8. `docs/backlog/PRODUCT-BACKLOG.md`
9. `docs/backlog/active/REL-002-v1-self-bootstrap-release-gate.md`
10. `docs/tasks/2026-07-07-four-month-developer-operating-plan.md`
11. `docs/backlog/active/RUNTIME-002-turn-health-and-stuck-processing.md`
12. `docs/backlog/active/PROVIDER-002-response-reliability-timeout-retry.md`
13. `docs/backlog/active/TUI-028-preview-status-feedback-reliability.md`
14. `docs/backlog/active/TUI-029-thinking-history-archive.md`
15. Candidate owner docs for the selected I107 and I108 work items.

## Operating Rules For Talos

- Work in ID order inside the active monthly iteration.
- Before editing, record the exact files expected to change and the validation commands expected to
  prove the result.
- Use existing repository patterns. Do not introduce broad abstractions to satisfy a single packet.
- For every behavior-facing change, record a real runtime scenario through the Talos binary or a
  Talos-owned harness.
- Update owner docs before `docs/BOARD.md`.
- If external Codex or another runtime edits files, mark the affected session partial or
  non-qualifying in REL-002.
- Never claim a command passed unless it was run in this worktree.

## Validation Matrix

Targeted task gates are listed in I106-I109. Monthly closeout should run:

```sh
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
scripts/validate_project_governance.sh .
git diff --check
```

If a command cannot run, the closeout must record the command, failure summary, likely cause, and
the fallback validation actually run.

## Branch, Worktree, And Checkpoint Plan

- Default worktree: `main` with a clean starting tree.
- Checkpoint after each SBT task with changed files, validation, and next task.
- Commit only after staged diff review and conventional commit message.
- Push only when explicitly authorized for that Talos execution run.
- If a session is interrupted, resume from the lowest-numbered Planned task in the active
  iteration and verify owner docs before editing.

## Allowed Permissions And External Actions

- Local source edits, tests, and governance validation inside the repository are allowed.
- Network use is limited to the configured provider calls needed for Talos execution and any
  maintainer-approved Git operation.
- External senior-agent review may identify defects and request changes.
- External senior-agent implementation makes the affected evidence partial or non-qualifying.

## Destructive Or Irreversible Operations

Not authorized: force-push, reset, tag deletion, release deletion, database deletion, broad cleanup,
credential migration, permission policy migration, or storage default migration.

## Time, Cost, And Resource Limits

- Prefer task packets that can finish within one working day.
- Stop for review if the same blocker recurs after three materially different attempts.
- Do not use paid or remote infrastructure beyond the configured model provider without explicit
  approval.

## Failure, Retry, And Fallback Policy

- Ordinary test failures should be fixed inside the active task scope.
- Scope expansion requires owner-doc update before implementation.
- If Talos cannot perform a required step as primary executor, record the blocker and classify the
  session honestly instead of allowing an external runtime to complete it invisibly.

## Default Decisions For Foreseeable Ambiguity

- For I107, select #18 request-dispatch timeout before TOOL-020, I085 MC107, or TUI polish. A P0
  stuck-processing residual outranks lower-risk feature polish.
- After #18 is closed, prefer #39 dashboard transient notification before #24/#31 visual evidence,
  because it has a concrete open issue and bounded behavior change.
- If TOOL-020 and I085 MC107 are both viable after the corrective queue is closed, prefer TOOL-020
  because it is a bounded, read-only user-facing tool improvement with clearer testability.
- If ARCH-032 and SESSION-004 are both proposed for I108, prefer ARCH-032 unless the maintainer
  explicitly accepts the session-storage default migration gate.
- If a candidate requires a new dependency, permission-default change, sandbox change, or release
  action, defer it and select a lower-risk candidate.

## Residual-Work Destination

- REL-002 qualification gaps: `docs/backlog/active/REL-002-v1-self-bootstrap-release-gate.md`.
- Feature/story residuals: the selected backlog owner doc.
- Monthly execution evidence: I106-I109 iteration docs.
- Cross-cutting ideas outside scope: `docs/proposals/`.

## Execution Log

| Date | Record |
|---|---|
| 2026-07-08 | Created planned Talos-primary self-bootstrap package with I106-I109 shells and REL-002 qualification boundaries. |
| 2026-07-09 | SBT100 activated. Runtime: glm-5.2 via zai-coding-plan (external, not Talos). Baseline: 1791 tests pass, governance 0 warnings, `talos 0.3.0` binary available, clean worktree. Session classified non-qualifying for REL-002. I106 moved to Active. |
| 2026-07-09 | SBT101-SBT104 complete. Evidence schema, smoke harness (`scripts/talos_smoke.sh`), governance rehearsal with rollback all delivered. Pre-existing `bash_tool.rs` fmt violation fixed. Full validation matrix green: fmt, check, 1791 tests, clippy, governance, diff-check. I106 moved to Review. REL-002 classification: non-qualifying (external runtime). I107 pending Review acceptance and Talos-primary runtime. |
