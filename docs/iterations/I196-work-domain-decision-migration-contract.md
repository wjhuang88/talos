# Iteration I196: Canonical Work Domain Decision And Migration Contract

> Document status: Planned
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
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned — proposed WORK-001-A / I196 P0 scope is defined below |
| Claimed At | Not applicable |
| Source Issue | #29 |
| Governance Claim PR | Pending |
| Authorization Mode | Not applicable until the Draft claim PR is finalized |
| Authorization Evidence | Not applicable until the Draft claim PR is finalized |
| Implementation PR | Not started |
| Last Updated | 2026-08-14 |
| Handoff / Release Condition | Finalize the proposed claim with its actual PR number, pass exact-head governance/CI checks and independent review, repeat merge-time CAS, and merge it to main before creating the P0 implementation branch. |

This governance branch is not an implementation branch. No claim is effective until the finalized
record reaches `main`.

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
- Governance claim PR: Pending.
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

There is no Active or Review iteration on the target branch. I195 exists only on open Dashboard
claim PR #212 and has no target-branch ownership effect. ARCH-034-R04 remains Partial;
RUNTIME-005 remains Refinement/Unclaimed; DESKTOP-001 remains Deferred/Unclaimed; ADR-059 remains
Proposed. PRs #120/#121 remain archival, and #222/#225 remain unrelated temporary replay PRs.

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

No activation has occurred. Append execution facts only after the finalized claim is effective on
`main`; do not rewrite the published baseline.

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
