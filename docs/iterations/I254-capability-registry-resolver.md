# Iteration I254: Capability Registry And Resolver

> Document status: Complete / Closed
> Published plan date: 2026-09-10
> Planned objective: Deliver the deterministic, offline Capability/Provider registry and resolver defined by CAP-001-B / Issue #512.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: A UI-neutral registry/resolver with typed outcomes, deterministic provider selection, bounded cancellation/deadline behavior, and conformance tests.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex mainline execution Agent |
| Work Slice | Capability identity lookup, registry ownership and bounded resolver contract only. |
| Claimed At | 2026-09-10 |
| Source Issue | #512 (parent #466) |
| Governance Claim PR | #523 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer authorized single-developer unattended completion of #499 then #466. Claim #523 is effective on main at 4cd7b42e958e73235597a984b6805219ea1f2256. Shared GitHub identity does not prove human separation; each stable implementation candidate requires fresh exact-head CI and independent Agent-role API review. |
| Implementation PR | #524 and #525 (merged) |
| Last Updated | 2026-09-10 |
| Handoff / Release Condition | Implementation starts only after this claim reaches main; no Plugin, Bundle, permission, release or product-surface authority. |

Claim #523 established ownership and activation on main. Follow `docs/sop/AGENT-COLLABORATION.md`
for local convergence, fresh stable-candidate evidence and final owner-first closure.

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
- Changed-file inventory: `crates/talos-core/src/capability.rs`, `crates/talos-core/src/lib.rs`,
  and this owner document only; all are within I254's UI-neutral core/API scope.

## Atomic Claim Candidate (2026-09-10)

PR #523 proposes Active / Claimed for I254 and CAP-001-B together. The selection inventory above
is the pre-claim checkpoint, not a competing current state. Implementation starts at the claim
merge or a later main commit. Local work uses the existing checkout on a short-lived branch;
no additional worktree, release, publication, dependency upgrade or destructive cleanup is needed.
Converge implementation, tests, API docs and owner Review state locally before the stable push.
Then obtain exact-head CI and applicable independent API/security review, execute CAS, merge and
close using pre-existing implementation evidence. All intermediate fixes remain under #512.

## Completion Evidence

- Completion Commit: d03da4940dfd0a749fa7b3ffebb2dcf3a60c5464, 63328e53e3e4759d99826c4dcc1a48bf7088f235.
- Both implementation commits exist on main before this state-only closeout; this closeout
  is not its own completion evidence.

## Variance And Residuals

- Claim #523 is effective. CAP-001-C and all domain children remain separately governed;
  this library-only slice does not claim runtime integration or completion of parent #466.

## Retrospective

- Outcome: offline library registry/resolver and public conformance acceptance completed.
- Documentation: public rustdoc, architecture boundary, owners and derived views synchronized.
- Lessons: verify callback failure and selection-return cancellation explicitly; keep intermediate
  fixes local, and never treat a healthy Windows job's duration as failure evidence.

## Post-Merge Acceptance Checkpoint (2026-09-10)

This checkpoint supersedes earlier candidate/pending statements as current execution truth;
the Published Baseline and dated planning records above remain historical evidence.

- PR #524 merged as `d7a836dee37a003a8f36476c124ba63cbc5decf2`, with head
  `d03da4940dfd0a749fa7b3ffebb2dcf3a60c5464` and base
  `4cd7b42e958e73235597a984b6805219ea1f2256`. CI `34428178830` attempt 2 and
  independent Agent-role approval `5612024153` apply only to that head.
- Acceptance hardening remains under I254/#512: callback unwind containment, checks within
  capability scans and before returning a selected provider, invalid request identity rejection,
  deterministic distinct-provider selection, and atomic validation of same-ID replacement.
- A synchronous host callback must return promptly. Deadlines are cooperative, not a mechanism
  to preempt a blocking callback; aborting panics cannot be recovered. No execution permission,
  installation, trust decision, tool/schema disclosure or persisted format changes are introduced.
- Public integration fixture `i254_capability_contract` drives unavailable -> registration ->
  available via exported APIs and preserves descriptor serialization and existing ToolRegistry
  names/schemas. This is the planned library-only acceptance, not CLI/runtime binding.
- Local validation: 90 core unit tests and 5 integration tests passed. Full
  `COLLABORATION_VALIDATION_BASE=origin/main CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 ./scripts/release_preflight.sh`
  passed with Rust 1.97.0 and main at `d7a836de` (fmt/check/Clippy/workspace tests/doctests).
  The first sandboxed attempt failed when an existing skill test created
  `~/.agents/skills/dedup-test`; the authorized unsandboxed rerun passed, with no fixture change.
  Core rustdoc built with seven pre-existing unrelated broken-link warnings, not a warning-free claim.
- Follow-up branch `fix/i254-resolver-conformance` starts at the #524 merge. The verified code
  retains the locally verified behavior. #512 remains open; Completion Commit
  remains pending until the follow-up is merged and acceptance is closed.
- Complete follow-up inventory: `crates/talos-core/src/capability.rs`,
  `crates/talos-core/tests/i254_capability_contract.rs`, this iteration owner,
  `docs/backlog/active/CAP-001-B-capability-registry-resolver.md`, CAP-001 parent,
  `docs/reference/ARCHITECTURE.md`, `docs/BOARD.md`, `docs/backlog/PRODUCT-BACKLOG.md`,
  `docs/iterations/README.md`, `docs/reference/ISSUE-DOC-CODE-STATUS-2026-08-31.md`,
  and `.agent-governance/manifest.yaml`.
  The two Rust files implement/test I254; remaining files document its actual boundary and
  mirror owner state. No Cargo, dependency, permission, Dashboard, release or unrelated owner edits.
- Resume: finish local governance/staged-diff checks, submit one stable follow-up, obtain new
  exact-head CI/API review and perform merge-time CAS. Then close owner-first with existing
  implementation evidence. Do not activate another #466 child before this acceptance gate.

## Acceptance Closeout (2026-09-10)

This checkpoint supersedes earlier pending execution/recovery instructions, not the Published
Baseline. PR #525 merged as `80b6678bbaf52e0bfb5ebace55f3c143a18e4f18` from exact head
`63328e53e3e4759d99826c4dcc1a48bf7088f235` and base
`d7a836dee37a003a8f36476c124ba63cbc5decf2`. CI `34441380091` passed all five jobs,
including Windows workspace (14m14s). Independent Agent-role API APPROVE is
[5613682105](https://github.com/wjhuang88/talos/pull/525#issuecomment-5613682105);
merge-time CAS is [5613808695](https://github.com/wjhuang88/talos/pull/525#issuecomment-5613808695).
Review and implementation roles are separate; shared GitHub identity does not establish
natural-person separation. No substantive candidate change followed these checks.

| Published acceptance | Evidence on main | Result |
|---|---|---|
| Deterministic compatible selection | `registry_selection_is_independent_of_distinct_provider_registration_order`; capability/provider major mismatch test | Pass |
| Typed unavailable/incompatible/invalid outcomes | Core resolver fixtures, invalid identity and descriptor tests | Pass |
| Cancellation, deadline and failure fail closed | Entry/deadline, nested scan, final-return cancellation and callback panic fixtures | Pass within documented cooperative callback contract |
| No tool/schema disclosure | Public `i254_capability_contract` compares ToolRegistry names/schema before and after registration/resolution | Pass |
| Offline startup | In-memory registry API has no I/O, install or network call; public fixture resolves host-registered descriptor offline | Pass |

No binary runtime integration is claimed: the Published Baseline explicitly selects library-only
conformance. CAP-001-C owns Plugin lifecycle/Carrier integration; BUNDLE/DIST own installation.
The synchronous callback must return promptly; aborting panics and blocking host callbacks are
outside recovery guarantees. No dependency, permission, persisted format or product UI changed.
The existing home-directory skill fixture limitation remains under TEST-001/#316, not I254.

Resume after this closeout reaches main: synchronize and close #512, inventory non-terminal
iterations, then prepare the next #466 child with its own effective claim. Do not infer a claim
for CAP-001-C or completion of CAP-001/#466 from this closeout.
