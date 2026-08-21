# Iteration I214: Bounded Shutdown Contract Decision

> Document status: Complete / Closed
> Published plan date: 2026-08-21
> Planned objective: decide the idempotent, deadline-bounded runtime shutdown, arbitration,
> finalizer and structured-report contract without changing production behavior.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: an independently reviewed ADR and current-path matrix make RUNTIME-005-B and C
> separately runnable and testable while preserving the current binary and SDK behavior.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline governance session 2026-08-21 |
| Work Slice | Decide only RUNTIME-005-A / I214: current-path characterization plus one shutdown policy, arbitration, admission, deadline, finalizer ordering, durable reconciliation, redacted report and compatibility ADR with B/C boundaries. No Rust/Cargo/API/runtime/Session/persistence, TOOL-024, permission, sandbox, product UI, dependency, release, publication or unsafe change. |
| Claimed At | 2026-08-21 |
| Source Issue | #49 |
| Governance Claim PR | #336 |
| Authorization Mode | Independent review |
| Authorization Evidence | Claim PR #336 merged as `7de582a3`. Decision PR #338 exact head `6719c876` passed CI `32449605985`, independent architecture review `5365529351` and merge-time CAS, then merged as `fc70e396`. |
| Implementation PR | #338 |
| Last Updated | 2026-08-21 |
| Handoff / Release Condition | Closed at Completion Commit `6719c876`; RUNTIME-005-B remains separately claimable and unactivated. |

## Published Baseline

### Current-Main Inventory And Disposition

Planning target: `main@5301b8c2bb570687cd3de2eb3ddeaa2ddb811c92`.

| Iteration(s) | State | I214 disposition |
|---|---|---|
| I164 | Paused / superseded | Do not restore; I165 owns its replacement. |
| I189 | Planned / Claimed | Keep unactivated; permission-sensitive work is independent. |
| I197, I198, I201, I210 | Review | Retain corrective-owner dispositions; no authority transfers. |
| I206-I208 | Planned / Unclaimed | Preserve steering order and defer. |
| I213 | Planned / Claimed | Dashboard lane is independent; do not activate or modify it. |
| I214 | Planned / Unclaimed | Select only this decision-only runtime slice for claim preparation. |

Open PRs #120/#121 are immutable archival recovery Drafts. No open PR owns RUNTIME-005-A, I214,
Issue #49 shutdown decisions or this Work Slice.

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| RUNTIME-005-A | RUNTIME-005 | Ready / Unclaimed | SESSION-008-A/B Complete; RUNTIME-001 Complete | Reviewed shutdown/finalizer ADR and current-path matrix; B/C implementation boundaries become explicit. |

### Scope

- Read-only characterization of current runtime/session shutdown and finalization paths.
- One decision covering policy vocabulary, arbitration, admission, deadlines, finalizer ordering,
  durable reconciliation, report privacy and compatibility.
- Explicit B/C implementation split, validation seams, semver/migration triggers and rollback.
- Parent, backlog, iteration, Board, manifest, long-task and Issue #49 synchronization.

### Non-Goals

- No Rust, Cargo, API, runtime, Session, persistence, TOOL-024, permission, sandbox, Desktop,
  Dashboard, TUI, dependency, release, publication or `unsafe` change.
- No implementation or activation of RUNTIME-005-B/C, TOOL-024-B/C/D or I189/PERM-006-A.
- No claim that Issue #59 production work is ready.

### Acceptance

- Given current runtime/session code, when the matrix is reviewed, then every shutdown handoff and
  current limitation is traceable to code without speculative behavior claims.
- Given idle, active, repeated and concurrent shutdown callers, when the ADR is evaluated, then it
  defines one terminal report, deadline semantics, admission result and exactly-once finalizer rule.
- Given partial-turn state or a failing/timed-out finalizer, when shutdown is decided, then durable
  reconciliation and redacted failure reporting preserve ADR-042/058 and fail conservatively.
- Given future TOOL-024 integration, when dependency direction is checked, then TOOL-024 consumes
  completed runtime finalization and never becomes a RUNTIME-005 prerequisite.

### Planned Validation

- `scripts/validate_project_governance.sh .`
- `bash scripts/validate_collaboration_claims.sh .`
- `git diff --check`
- Independent exact-head architecture review of race, privacy, deadline, semver and dependency
  boundaries.
- CI routed from the repository classifier; no Rust validation is claimed as decision evidence.

### Documentation To Update

- `docs/backlog/active/RUNTIME-005-A-shutdown-contract-decision.md`
- `docs/backlog/active/RUNTIME-005-bounded-graceful-shutdown.md`
- a new Proposed ADR and current-path reference during decision execution
- `docs/backlog/PRODUCT-BACKLOG.md`, `docs/iterations/README.md`, `docs/BOARD.md`
- `.agent-governance/manifest.yaml` and the mainline long-task checkpoint

### Risks And Rollback

- Risk: a vague decision pushes incompatible policy choices into implementation or creates a
  RUNTIME-005/TOOL-024 dependency cycle.
- Risk: a structured report accidentally permits sensitive prompts, reasoning or raw tool data.
- Rollback: reject the decision, leave RUNTIME-005-B/C and TOOL-024-B blocked, and preserve current
  shutdown behavior unchanged.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-08-21 | Selection | T10 reassessment found TOOL-024-A and TOOL-023-C complete, but TOOL-024-B still blocked by RUNTIME-005 and PERM-006-C. RUNTIME-005-A is the smallest currently runnable gate-clearing slice; this planning record creates no implementation authority. |
| 2026-08-21 | Claim effective | PR #336 final head `cc99af9e` passed CI `32435705544`, both governance validators, independent claim review `5364050202` and merge-time CAS, then merged as `7de582a3`. |
| 2026-08-21 | Activation | I214 is Active/Claimed from `7de582a3`. Only read-only current-path characterization and the Proposed ADR are authorized. I189 remains unactivated and I213 remains in the independent Dashboard lane. |
| 2026-08-21 | Decision execution | Current-path matrix and Proposed ADR-063 were committed as `648a35d3` from activation merge `14531bbc` and submitted in PR #338. They define independently runnable B/C boundaries without changing Rust, Cargo, APIs, persistence or runtime behavior. |
| 2026-08-21 | Review correction | Architecture review of head `0adcd072` rejected the separate actor closing-bit check and invalid consuming-options contract. ADR-063 now requires one SDK/actor admission-start linearization point, construction-time validated options, borrowing structured shutdown and explicit primary/controller Drop semantics; both findings are batched locally before one new stable #338 head. |
| 2026-08-21 | Decision accepted | Corrected exact head `6719c876` passed CI `32449605985`, independent architecture review `5365529351` and merge-time CAS; PR #338 merged as `fc70e396`. ADR-063 is Accepted and I214 closes without authorizing B/C implementation. |

## Verification Evidence

- Claim exact-head CI `32435705544`, both governance validators, independent review `5364050202`
  and merge-time CAS passed before merge `7de582a3`.
- Current-path evidence: `docs/reference/I214-RUNTIME-SHUTDOWN-CURRENT-PATH.md` at `14531bbc`.
- Decision evidence: Accepted `docs/decisions/063-bounded-runtime-shutdown-finalization.md` at
  exact head `6719c876`; CI `32449605985`, architecture review `5365529351`, CAS and PR #338 merge
  `fc70e396` passed.
- Review correction evidence: PR #338 comment `5364268484`; corrected head `6719c876` then received
  fresh exact-head CI `32449605985` and architecture approval `5365529351` before merge.
- Runtime behavior evidence: not applicable; this decision-only iteration changes no executable
  path and cannot claim B/C behavior.

## Completion Evidence

- Completion Commit: `6719c876fe9f190e47fba5ef62f3263e782d6e8b`
- The Completion Commit predates this status-only closeout and contains the independently reviewed
  correction to ADR-063. Original matrix/decision content is in `648a35d3`.

## Variance And Residuals

- RUNTIME-005-B is Ready/Unclaimed after ADR-063 acceptance; C remains Blocked until B completes.
- PERM-006-A/B/C and TOOL-024-B/C/D remain outside I214.

## Retrospective

- Outcome: the current path and one implementable shutdown/finalizer contract were independently
  reviewed without changing production behavior.
- Documentation: ADR-063, current-path matrix and RUNTIME-005 A/B/C boundaries are synchronized.
- Lesson: an SDK admission bit alone cannot prove actor start exclusion; lifecycle decisions must
  name the final start-commit linearization point and ownership/drop semantics.
