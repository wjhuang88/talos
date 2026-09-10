# Iteration I254: Capability Registry And Resolver

> Document status: Active / Claimed (effective on main via PR #523 merge `4cd7b42e`)
> Published plan date: 2026-09-10
> Planned objective: Deliver the deterministic, offline Capability/Provider registry and resolver defined by CAP-001-B / Issue #512.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: A UI-neutral registry/resolver with typed outcomes, deterministic provider selection, bounded cancellation/deadline behavior, and conformance tests.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex mainline execution Agent |
| Work Slice | Capability identity lookup, registry ownership and bounded resolver contract only. |
| Claimed At | 2026-09-10 |
| Source Issue | #512 (parent #466) |
| Governance Claim PR | #523 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer authorized single-developer unattended completion of #499 then #466. No independent human reviewer is available; shared GitHub identity does not prove human separation. PR #523 requires exact-head CI, both validators and merge-time CAS; proposed claim and activation remain ineffective until merge. |
| Implementation PR | Not started |
| Last Updated | 2026-09-10 |
| Handoff / Release Condition | Implementation starts only after this claim reaches main; no Plugin, Bundle, permission, release or product-surface authority. |

Before implementation, follow `docs/sop/AGENT-COLLABORATION.md`. One governance-only PR proposes
both `Claimed` and `Active`; both are ineffective until the finalized record reaches the target
branch. Implementation then converges locally before the first stable stage candidate is pushed.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| CAP-001-B | CAP-001 / #466 | Ready | ADR-072 Accepted; CAP-001-A / I252 Complete | Deterministic offline capability registry/resolver with typed, fail-closed outcomes. |

### Scope

- Add a single UI-neutral registry for validated `CapabilityDescriptor` and `ProviderDescriptor` values.
- Resolve only already-available providers using deterministic identity and major-version compatibility.
- Return typed available, unavailable, incompatible, invalid and resolver-failure outcomes without panics.
- Bound cancellation/deadline handling and fail closed on resolver errors or stale inputs.
- Prove registration does not expose tools or schemas and does not perform network discovery or installation.
- Preserve the existing public descriptor API and persisted configuration formats.
- Infrastructure-only deliverable: public API conformance fixtures exercise registration through
  resolution end to end. This does not claim that CLI tools or Plugin loaders consume the registry.
  Registration records host-declared availability; descriptor provenance is never proof of trust
  or an execution permission grant.

### Non-Goals

- No Plugin loader or Carrier adapter implementation (CAP-001-C).
- No Bundle installation, network discovery, executable download or persisted manifest migration.
- No permission-policy rewrite, language/browser/Desktop/Dashboard work, release or publication.
- No public API rename or alternate registry in CLI, TUI, Dashboard or Desktop.

### Acceptance

- Given validated descriptors registered in any order, when a capability is resolved, then the same compatible provider and result are selected deterministically.
- Given no compatible provider or an invalid descriptor, when resolution runs, then a typed unavailable/incompatible/validation result is returned and the process remains alive.
- Given cancellation, deadline expiry or resolver failure, when resolution runs, then no provider is selected and the result is fail closed.
- Given a successful registration, when the tool registry or schema projection is inspected, then no tool or schema is exposed by registration alone.
- Given offline startup, when the registry is constructed, then no network or installation operation is attempted.

### Planned Validation

- Focused `talos-core` registry/resolver unit and conformance tests with deterministic offline fixtures.
- `cargo check --locked -p talos-core` and affected-workspace locked checks.
- `cargo test --locked -p talos-core` plus the relevant dependent crate tests.
- Governance validators, `git diff --check`, and staged changed-file inventory.
- API documentation build/check proving the public boundary is UI-neutral and unchanged outside this slice.
- `./scripts/release_preflight.sh` before the stable implementation candidate, using the pinned
  toolchain and locked workspace validation. Pure governance preparation does not run Rust tests.

### Documentation To Update

- `docs/backlog/active/CAP-001-B-capability-registry-resolver.md` and CAP-001 parent child map.
- `docs/reference/ARCHITECTURE.md` capability-provider boundary and public API notes.
- `docs/iterations/README.md`, `docs/BOARD.md`, Product Backlog and issue reconciliation after owner activation.
- Public API rustdoc for the registry/resolver types.

### Risks And Rollback

- Risk: nondeterministic provider selection or resolver failure could expose the wrong capability or crash a host.
- Risk: accidental coupling of registration to tool/schema disclosure could widen the model-visible surface.
- Rollback: remove the registry/resolver integration while retaining I252 descriptor types; no persisted schema or startup behavior changes are required.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-09-10 | Claim activation | PR #523 (`d3cfbdd8`) merged to `main` as `4cd7b42e`; claim/activation are effective. Implementation may begin from this merge or later; no implementation PR exists yet. |

### Selection Inventory (2026-09-10)

Verified target: `main` / `origin/main` at `540ad257cd9d7692b62166956ab130b3c5739f06`.
Iteration owner headers, not dated historical checkpoints or Board prose, establish current state.

| Iteration / Work | Current State | Disposition |
|---|---|---|
| Existing Active / Review iterations | None | No active implementation authority overlaps this selection. |
| I164 | Paused, superseded by I165 | Preserve historical layout target; do not resume. |
| I249 | Planned / Unclaimed | Retain unselected dependency pilot; no dependency upgrades in I254. |
| Existing Blocked iterations | None | Unselected blocked backlog owners retain their own gates. |
| I254 / CAP-001-B / #512 | Planned / Unclaimed draft | Select only the registry/resolver; activation requires finalized claim merge. |
| CAP-001-C and other #466 children | Unclaimed | No authority transfer; retain owner-defined dependencies. |
| INTEGRATION-001 / #520 | Intake / Unclaimed | Separate runtime/browser clarification; excluded from I254. |

The checkout was clean before this draft, with one worktree and no stashes. The remote open-PR
list was empty. Retained historical branches are not active work or implementation starting points.
I252 descriptor evidence and I253 governance evidence remain unchanged. I162's header records a
completed readiness review, not an active Review iteration.

Local design, tests, fixes and review corrections stay in this iteration and its candidate PR.
Use existing Issue #512; do not create remote Issues for these local steps.

## Verification Evidence

- Draft `203d1168`: both governance validators passed with 0 warnings; collaboration validation
  used `COLLABORATION_VALIDATION_BASE=origin/main`. Staged diff and whitespace checks passed.
- Finalized claim validation and exact-head CI remain pending; draft evidence does not substitute.
- Runtime evidence: not applicable before implementation authorization.

## Local Convergence Checkpoint (2026-09-10)

- Implementation commit: `2d060655` (local stable candidate; not yet pushed).
- `cargo fmt --all -- --check` passed.
- `cargo check --locked -p talos-core` passed.
- `cargo test --locked -p talos-core` passed: 84 tests, 0 failures.
- `git diff --check` passed; no Dashboard, permission, release or persistence files changed.
- Candidate remains local until the complete affected-workspace and governance checks finish.

## Atomic Claim Candidate (2026-09-10)

PR #523 proposes Active / Claimed for I254 and CAP-001-B together. The selection inventory above
is the pre-claim checkpoint, not a competing current state. Implementation starts at the claim
merge or a later main commit. Local work uses the existing checkout on a short-lived branch;
no additional worktree, release, publication, dependency upgrade or destructive cleanup is needed.
Converge implementation, tests, API docs and owner Review state locally before the stable push.
Then obtain exact-head CI and applicable independent API/security review, execute CAS, merge and
close using pre-existing implementation evidence. All intermediate fixes remain under #512.

## Completion Evidence

- Completion Commit: pending implementation evidence.
- Status-only documentation commits must not cite themselves. Retain `Planned`, `Active`, or `Review` until implementation evidence exists.

## Variance And Residuals

- No variance; implementation remains unauthorized until the claim is effective on `main`. CAP-001-C and all domain children remain separately governed.

## Retrospective

- Outcome: planned.
- Documentation: governance and API documentation targets are listed above.
- Lessons: pending implementation closeout.
