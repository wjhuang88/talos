# Iteration I196: Canonical Work Domain Decision And Migration Contract

> Document status: Active
> Published plan date: 2026-08-14
> Planned objective: establish the P0 canonical work-state architecture decision, current Todo
> compatibility inventory, migration/rollback contract and separately governed P1-P4 boundaries
> before any public or persisted work-state behavior changes.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: a maintainer can run the recorded inventory and governance checks and use one
> independently reviewed decision packet to determine the allowed canonical work-state boundary,
> every existing Todo compatibility obligation, migration failure/rollback behavior and the exact
> gate into P1 without inspecting or accepting implementation code.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline governance session 2026-08-14 |
| Work Slice | WORK-001-A / I196 P0 only: inventory current Todo/runtime/session/projection obligations; decide the canonical work-state ownership and dependency boundary; define stable identity/revision plus migration, compatibility, rollback and P1-P4 contracts. No Work Graph, Evaluation, Evaluator, persistence, public API, Rust/Cargo, Desktop, Dashboard, TUI product or later-child implementation. |
| Claimed At | 2026-08-14 |
| Source Issue | #29 |
| Governance Claim PR | #226 |
| Authorization Mode | Independent review |
| Authorization Evidence | PR #226 exact head `8d0cce3230b4030aab946fb0757da705dcfa4e26` passed CI `31781768908` and independent approval comment `5291072895`, then merged to `main` as `453d1fba97470639835468664c58397770db384c`. The claim is effective. A separate independent exact-head architecture review is still required for P0 decision acceptance. |
| Implementation PR | Not started |
| Last Updated | 2026-08-18 |
| Handoff / Release Condition | Activation is effective from current `main@b59912e3`; complete only the P0 decision/documentation slice, then obtain independent exact-head architecture review, exact-head CI and owner-first closeout. |

The claim is effective through PR #226 merge `453d1fba97470639835468664c58397770db384c`.
It authorizes only the frozen decision/documentation slice and does not by itself activate I196.

## Published Baseline

### Selected Story

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| WORK-001-A | WORK-001 | Ready / proposed claim | RUNTIME-001, TODO-001, TODO-002 and VALIDATION-001 Complete; current-main inventory | One independently reviewed P0 decision and migration packet with no production or persistence behavior change |

### Target-Branch Baseline At Planning

- Target branch: `main`.
- Exact `main` and `origin/main` at governance-branch creation:
  `556b5a4319085bf5250bccf4920e0dec0c6646c8`.
- Governance branch: `docs/mainline-I196-work-domain-p0-claim`.
- Governance claim PR: #226 (proposed claim; ineffective before target-branch merge).
- I195 is reserved by Dashboard PR #212; that proposed claim remains ineffective until merged.
- The eventual P0 implementation base is the effective I196 claim merge or a later current `main`,
  after a fresh dependency/overlap inventory and merge-time CAS.

### Non-Terminal Iteration Inventory And Disposition

| Iteration | Target-Branch State | Disposition For I196 Planning |
|---|---|---|
| I159 | Blocked | Keep blocked until TUI-037 has a recorded disposition. |
| I160 | Blocked | Keep blocked until I159 is Complete. |
| I161 | Blocked | Keep blocked until I160 is Complete and a security-review plan exists. |
| I162 | Blocked | Keep blocked until I161 is Complete and release-readiness authorization exists. |
| I164 | Paused | Preserve superseded history; do not resume. |
| I188 | Planned / Claimed | Keep unactivated; its TOOL-024-A decision scope remains independent. |
| I189 | Planned / Claimed | Keep unactivated; its PERM-006-A protected scope remains independent. |
| I195 | Planned / Claimed | Keep unactivated for this P0; its WEB-001-A read-only Dashboard shell claim is effective on `main` through merge `f123e534`, but its implementation scope remains independent of WORK-001-A. |

There is no Active or Review iteration on the target branch. I195 is now a target-branch
`Planned / Claimed` iteration through the merged Dashboard governance claim PR #212 at
`f123e534`; it has ownership on `main` but no scope overlap or implementation authorization in
WORK-001-A. ARCH-034-R04 remains Partial;
RUNTIME-005 remains Refinement/Unclaimed; DESKTOP-001 remains Deferred/Unclaimed; ADR-059 remains
Proposed. PRs #120/#121 remain archival, and #222/#225 remain closed historical replay PRs outside
this owner scope.

### Current Exact-Base Inventory Checkpoint

The original planning baseline above remains historical evidence. At the synchronization checkpoint
for the current `main@bfb167d81c4fc320c5cc532cde08d81a29d17113`, the complete non-terminal inventory is
I159/I160/I161/I162 (Blocked), I164 (Paused), I188 (Planned / Claimed), I189 (Planned / Claimed),
and I195 (Planned / Claimed). No iteration is Active or Review. This checkpoint supersedes the
earlier statement that I195 existed only on an open PR; it does not activate I195 or import its
Dashboard implementation scope into I196.

### Scope

- Produce exactly WORK-001-A's current-state inventory, canonical boundary ADR,
  migration/compatibility/rollback contract and P1-P4 boundary map.
- Add discoverable WORK-001 downstream references to RUNTIME-001 and SESSION-009 without changing
  their status, completed evidence or selection state.
- Record deterministic commands and structural evidence that prove the packet matches the current
  repository and changes no implementation behavior.

### Non-Goals

- No Work Graph, Completion Claim, Evaluation, Evaluator or Mission gate implementation.
- No new crate, Rust source, Cargo/lockfile, schema, persistence or public API change.
- No Desktop/Dashboard/TUI product work, GPUI, localization or native dependency.
- No activation or implementation of I188, I189, RUNTIME-005 or SESSION-009.
- No repair of unrelated historical I193/manifest/Board wording drift inside this owner scope.

### Execution Sequence After Claim Becomes Effective

1. Refresh `main`; repeat iteration, issue, PR, branch and owner overlap inventory.
2. Create the P0 implementation branch from the claim merge or later exact `main`.
3. Inventory current Todo storage, tools, permission profiles, commands, prompt and projections.
4. Write the minimum decision ADR and migration/compatibility/rollback contract.
5. Review P1-P4 boundaries against the proposal, current crate ownership and public API constraints.
6. Run structural, governance and unchanged-path validation.
7. Submit one decision-only implementation PR for independent exact-head architecture review.

### Acceptance

All WORK-001-A acceptance items must be satisfied. The iteration must remain Review or Partial if
the decision packet leaves a breaking API, persistence migration, compatibility, rollback or
evaluator-isolation decision unresolved for P1.

### Planned Validation

- Focused inventory commands over Todo/runtime/session/conversation/CLI/TUI ownership.
- Existing Todo regression tests recorded as baseline without modification.
- `scripts/validate_project_governance.sh .`
- `bash scripts/validate_collaboration_claims.sh .`
- `git diff --check`
- Changed-path assertion proving no `*.rs`, `Cargo.toml`, `Cargo.lock`, schema/migration or Desktop
  implementation asset is changed.
- Independent natural-person exact-head architecture review with shared-account identity disclosure
  where applicable.

### Documentation To Update During P0 Implementation

- WORK-001-A and this iteration with decision/validation evidence.
- WORK-001 child summary if a boundary changes.
- One new Proposed canonical work-domain ADR and `docs/decisions/README.md`.
- One current-state and migration/compatibility/rollback contract under `docs/reference/`.
- `docs/reference/ARCHITECTURE.md` only for accepted shared-boundary truth.
- Product Backlog, iterations index and Board only after owner state changes.
- No user feature claim in README or release notes because P0 changes no observable behavior.

### Risks And Rollback

- Risk: treating the candidate `talos-work` name as a settled boundary without repository evidence.
- Risk: promising Todo compatibility without a complete mechanical P1 regression matrix.
- Risk: defining revision semantics that cannot support later exact-subject Evaluation staleness.
- Rollback: P0 changes documentation/decision artifacts only; reject or supersede the Proposed ADR
  before P1 and preserve all current code, schema and user behavior.

## Actual Activation And Execution

Append execution facts only after the finalized claim is effective on `main`; do not rewrite the
published baseline.

### 2026-08-14 Pre-Activation Priority Hold

PR #226 merged the effective I196 claim at
`453d1fba97470639835468664c58397770db384c`. Before an implementation branch was created, the
maintainer selected the v0.8.0 GitHub-first/Cargo-second publication sequence as the next mainline
work. I196 remains Planned / Claimed and unactivated; its scope and acceptance are unchanged. After
the release task closes, I196 must refresh to then-current `main`, repeat every dependency and
overlap check, and obtain independent exact-head review for the P0 decision implementation.

### 2026-08-18 Activation Checkpoint

I196 is Active from `main@b59912e36025088e4e3fa76b7b5b4e2aa7a1396c` as the mainline P0
decision/documentation slice. The existing non-overlap authorization for Dashboard I195 is recorded
by activation PR #288. Inventory at activation: I159/I160/I161/I162/I188 Complete; I164
Paused; I189 Planned/Claimed; I195 Active/Claimed in the separately authorized Dashboard lane; no
other iteration is Review. The I195/I196 scopes are explicitly non-overlapping. This activation
does not authorize Rust, Cargo, persistence, public API, Desktop, Dashboard or TUI implementation.

## Verification Evidence

Pending P0 implementation after claim merge.

## Completion Evidence

Completion Commit: Pending. A status-only commit cannot serve as its own evidence.

## Variance And Residuals

- WORK-001-B through WORK-001-E remain separate blocked future slices.
- Historical I193/manifest/Board closeout wording drift remains outside this claim and requires an
  independent consistency correction or explicit residual disposition.

## Retrospective

Pending execution.
