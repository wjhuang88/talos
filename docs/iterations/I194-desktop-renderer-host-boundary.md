# Iteration I194: Desktop Renderer, Host, And Repository Boundary

> Document status: Complete
> Published plan date: 2026-08-13
> Planned objective: decide the Desktop renderer/dependency/host/repository boundary and produce an independently reviewable ADR/security packet without adding Desktop production code or renderer dependencies.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: one accepted decision packet that makes a later mock-only visual/i18n slice authorizable without implying real Mission/runtime binding.
> Activation rule: this iteration is not implementation authority until its finalized Collaboration Claim exists on `main` and the current three-track overlap/CAS gate passes.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-5.6 Sol Desktop governance session 2026-08-13 |
| Work Slice | DESKTOP-001-D0 / I194 only: decide and document the Desktop renderer/dependency/host/repository boundary, current GPUI-or-alternative evidence, native/unsafe/security implications, localization selection criteria, and the later mock-only authorization gate. No production UI, dependency, runtime/domain, persistence, session, permission or P0-P4 implementation. |
| Claimed At | 2026-08-13 |
| Source Issue | #29 |
| Governance Claim PR | #211 |
| Authorization Mode | Independent review |
| Authorization Evidence | Independent natural-person exact-head review `5277513378`, exact-head CI `31678604823`, both governance validators and merge-time CAS passed for PR #211 head `fb8a2b67`; claim merged to `main` as `f778543c`. Authorization remains limited to this decision-only D0 slice. |
| Implementation PR | #215 (decision-only D0 packet; no Desktop implementation) |
| Last Updated | 2026-08-13 |
| Handoff / Release Condition | None — I194 decision packet is complete; later visual or renderer work requires a separate governed child. |

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `DESKTOP-001-D0` | `DESKTOP-001` | Complete; claim effective in merge `f778543c` | DESKTOP-001 design baseline; ADR-042; ADR-052; repository hard constraints | Auditable renderer/dependency/host/repository ADR and security-review packet; no renderer implementation |

### Start Here

Read in order:

1. `AGENTS.md`
2. `docs/sop/AGENT-COLLABORATION.md`
3. `docs/sop/REQUIREMENT-INTAKE.md`
4. `docs/sop/START-ITERATION.md`
5. `docs/sop/ITERATION-WORKFLOW.md`
6. `docs/sop/GIT-WORKFLOW.md`
7. `docs/sop/TESTING.md`
8. `docs/tasks/2026-08-13-three-track-development-baseline.md`
9. `docs/backlog/active/DESKTOP-001-D0-renderer-host-boundary.md`
10. `docs/backlog/active/DESKTOP-001-desktop-product-direction.md`
11. Desktop proposal/design/i18n documents and all Required Reads named by the selected Story
12. the exact current `talos-runtime`, `talos-session` and `talos-conversation` boundaries

The selected Story owns D0 scope and acceptance. This iteration owns activation, exact-base
provenance, three-track coordination, execution evidence, variance and completion state.

## Claim-Preparation Base And Three-Track Inventory

The governance branch for this Planned iteration was created from exact target-branch SHA:

`c4bd9606c8bae63cb9bf11becd45846bf0805982`

The common three-track base remains:

`23e4174bcfb036602ce2145026b872ec5c517289`

Pre-claim inventory immediately before governance branch creation:

| Iteration / Work | State | Disposition |
|---|---|---|
| I159 | Blocked | Keep blocked; TUI-037 disposition gate remains authoritative. |
| I160 | Blocked | Keep blocked; requires I159 Complete. |
| I161 | Blocked | Keep blocked; requires I160 Complete and a security-review plan. |
| I162 | Blocked | Keep blocked; requires I161 Complete and readiness authorization. |
| I164 | Paused | Keep paused; superseded startup-inline target is not resumed. |
| I188 | Planned / Claimed | Keep unactivated; TOOL-024-A decision scope remains independent. |
| I189 | Planned / Claimed | Keep unactivated; PERM-006-A scope remains independent. |
| I193 / PR #210 | Planned / Claimed on target branch through merge `fb5a1f62` | Keep unactivated and unchanged; Desktop intentionally uses I194 to avoid cross-track iteration-ID collision. |
| PR #120 / #121 | Archival recovery Draft PRs | Do not touch or treat as implementation authority. |

There was no Active or Review iteration on the observed `main`. The three-track baseline explicitly
permits parallel non-overlapping Dashboard, Desktop and mainline-foundation work, so unrelated lane
activity does not by itself block I194. It does require I194 to refresh the complete inventory and
open-PR/branch overlap before activation, before implementation PR submission and again at merge-time
CAS. Target-branch owner truth always wins over this preparation snapshot.

## Authorized Scope

Only the decision work defined by `DESKTOP-001-D0` is in scope:

- current primary-source renderer/dependency evidence sufficient to accept or reverse the GPUI
  direction;
- material native/FFI/build-script/platform/unsafe/panic-boundary inventory;
- host integration responsibilities for macOS, Windows and Linux;
- repository placement and dependency-direction decision reconciled with existing Talos crates;
- Chinese IME, text/CJK, accessibility and reduced-motion implications relevant to renderer choice;
- localization mechanism/library selection criteria without adding the dependency;
- a Proposed ADR and security-review matrix;
- explicit exclusion/gate mapping for the later mock-only visual/i18n slice and real P0-P4 binding.

## Forbidden Changes

- No `talos-desktop`, `talos-work`, second runtime/agent engine, alternate durable Mission/session/
  permission/work store or new speculative shared presentation crate.
- No GPUI/i18n/native dependency, `Cargo.toml`, `Cargo.lock`, workspace member, build script, FFI or
  `unsafe` implementation.
- No visual mock, fixture UI, window/widget code, localization catalog or renderer implementation.
- No real Mission execution, durable persistence, Completion, Evaluation, Approval, Artifact,
  Delivery, recovery, reconnect or multi-client behavior.
- No I188/I189/SESSION-008-B activation or boundary change.
- No SESSION-008, RUNTIME-005 or ARCH-034-R04 boundary/status change.
- No animation implementation; motion is recorded only as a renderer/host acceptance constraint.
- No PR #120/#121 or Issues #45/#49/#59 modification/closure.

## Implementation Slices

After the effective claim exists, D0 execution remains deliberately small:

1. **Fresh-base / overlap CAS**
   - refresh `main` and record exact implementation base SHA;
   - repeat all Active/Review/Planned/Blocked dispositions;
   - check open claim/implementation PRs and branches across all three lanes;
   - verify DESKTOP-001 remains Deferred/Unclaimed and this child claim remains effective.
2. **Current evidence**
   - collect current primary-source evidence only for the minimum renderer/localization candidates
     needed to validate the design direction;
   - record versions/dates and distinguish product preference from implementation authorization.
3. **Boundary trace**
   - trace material dependency/native/unsafe/platform implications;
   - reconcile actual workspace dependency direction with runtime/session/conversation ownership.
4. **Decision packet**
   - write one Proposed ADR plus a security/repository/host matrix;
   - state selected/rejected direction, containment duties, exclusions and reversal triggers.
5. **Validation/review**
   - run governance and repository-required locked checks for the exact diff;
   - obtain independent natural-person exact-head architecture/security review;
   - repeat merge-time CAS and compare reviewed head with the content that will merge.
6. **Closure after merge only**
   - record the already-existing implementation/decision merge SHA as `Completion Commit:`;
   - synchronize owner first, then iteration/backlog/Board/Issue state.

## Acceptance

All D0 acceptance items in `DESKTOP-001-D0` must be satisfied. In particular:

- a design-level GPUI preference is not enough; the D0 ADR must explicitly authorize or reject the
  concrete renderer dependency boundary from current evidence;
- native/unsafe/platform and failure-containment implications must be auditable;
- repository/dependency arrows must prevent renderer code from becoming runtime/core authority;
- i18n/IME/bilingual/reduced-motion requirements must influence the decision without adding the
  future implementation dependency in D0;
- no production UI/runtime/domain behavior may be claimed.

## Planned Validation

Claim/governance preparation:

- `git diff --check`
- `scripts/validate_project_governance.sh .`
- `bash scripts/validate_collaboration_claims.sh .`
- exact-head CI for the governance-only diff

D0 implementation/decision PR after effective claim:

- `cargo fmt --all --check`
- `cargo check --workspace --locked`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo test --workspace --locked`
- `scripts/validate_project_governance.sh .`
- `bash scripts/validate_collaboration_claims.sh .`
- `./scripts/release_preflight.sh`
- `git diff --check`
- independent natural-person exact-head architecture/security review
- merge-time CAS against the then-current `main`

A mechanically reduced documentation CI route does not reduce this iteration's acceptance standard.
If the execution environment cannot run a required command, the missing local evidence must remain
explicit and exact-head CI/reviewer evidence must not be represented as a substitute unless the
repository SOP permits it.

## Documentation To Update During D0

Expected decision PR targets:

- `docs/backlog/active/DESKTOP-001-D0-renderer-host-boundary.md`
- `docs/iterations/I194-desktop-renderer-host-boundary.md`
- one new Desktop renderer/host ADR under `docs/decisions/`
- `docs/decisions/README.md`
- a focused security/dependency matrix under `docs/reference/` only if the ADR would otherwise become
  unwieldy
- directly affected Desktop proposal/design/i18n wording only when the accepted D0 decision narrows
  or reverses the current design direction
- `docs/iterations/README.md`, `docs/backlog/PRODUCT-BACKLOG.md` and `docs/BOARD.md` for truthful
  status synchronization

README/user docs remain unchanged because D0 adds no shipped behavior.

## Risks And Rollback

- Risk: treating the documented GPUI direction as already authorized could bypass dependency/native
  review. Mitigation: D0 requires an explicit accepted decision before any renderer dependency.
- Risk: parallel lanes may consume the same iteration ID or change shared boundaries while D0 is in
  flight. Mitigation: I194 was chosen after observing PR #210's I193 proposal, and every activation/
  merge gate refreshes mainline/PR/branch owner truth.
- Risk: a GUI dependency can hide native/unsafe/process-exit behavior behind transitive crates.
  Mitigation: trace material transitive ownership and define integration containment before
  acceptance.
- Risk: inventing a shared UI abstraction for a second renderer could duplicate
  `talos-conversation` or prematurely stabilize it. Mitigation: extraction requires concrete
  dependency-direction evidence and is outside D0 unless a decision proves it unavoidable.
- Rollback: reject/defer the D0 ADR and leave DESKTOP-001 Deferred; no production code or dependency
  has changed.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-08-13 | Selection | `DESKTOP-001-D0` selected as Planned I194 governance/decision work from `main@c4bd9606c8bae63cb9bf11becd45846bf0805982`. I193 was deliberately not used because PR #210 proposed it for SESSION-008-B. PR #211 carried the finalized claim and subsequently merged to establish the effective D0 claim. |
| 2026-08-13 | Baseline refresh | Refreshed against `main@0459b8afb1626783f21b54dbaf55a0ef84393cd7` after PR #210 merged as `fb5a1f62`. I193 is now Planned / Claimed and remains unactivated; derived governance files preserve the Runtime/Session lane alongside the proposed I194 Desktop lane. |
| 2026-08-13 | Activation | I194 activated from `main@f778543c7ceeb2a099eb3863fc8259da68d02195` in worktree `/private/tmp/talos-i194` on `feat/desktop-I194-d0-boundary`; merge target `main`. D0 remains decision-only. |

## Verification Evidence

- Implementation/decision worktree base: `f778543c7ceeb2a099eb3863fc8259da68d02195`.
- Governance claim PR: #211.
- Target-branch claim is effective through PR #211 merge `f778543c7ceeb2a099eb3863fc8259da68d02195`.
- Independent worktree evidence: `/private/tmp/talos-i194` on
  `feat/desktop-I194-d0-boundary`, based on `f778543c7ceeb2a099eb3863fc8259da68d02195`.
- Governance validators and merge-time CAS for the claim passed before merge; D0 ADR/security review
  and implementation-slice validation remain pending.
- `cargo check --locked --workspace`: passed.
- `cargo clippy --locked --workspace --all-targets -- -D warnings`: passed.
- `cargo test --locked --workspace`: 325 passed; 16 existing `talos-cli` provider-discovery tests
  failed with sandbox `PermissionDenied` while creating their local HTTP fixture. No D0 test or
  source code was involved; rerun with an approved writable test environment remains required.
- `scripts/validate_project_governance.sh .`: passed with 0 warnings.
- `bash scripts/validate_collaboration_claims.sh .`: passed with 0 warnings.
- `git diff --check`: passed.
- Primary-source snapshots for GPUI and the minimum Iced comparison were retrieved on 2026-08-13
  and pinned by immutable commit in `docs/reference/DESKTOP-I194-DEPENDENCY-SECURITY-MATRIX.md`.
  They establish candidate capability/risk facts only; selected-release lock closure, SBOM/license
  review, platform tests, panic containment and motion benchmarks remain open Review residuals.
- Crates.io metadata confirmed `gpui 0.2.2` and `iced 0.14.0`; disposable full-graph resolution was
  attempted outside the Talos worktree and stopped when the registry proxy became unavailable. No
  probe manifest, Cargo change or dependency was added to Talos.

## Completion Evidence

Completion Commit: `0a47208ce6fad23c706ebede8b3d07111b9303dc`

- PR #215 merged to `main` as `1beaca68b98b56aaff42d952b7dbbc7519304740`.
- Independent natural-person approval comment `5278769979` binds the exact completion head.
- Exact-head CI `31687636396` passed all applicable jobs; both governance validators reported 0
  warnings and `git diff --check` was clean.
- The packet is decision-only. ADR-059 remains Proposed; renderer dependency, platform execution,
  panic containment, SBOM and motion benchmark gates remain future implementation requirements.

## Variance And Residuals

- DESKTOP-001 remains Deferred / Unclaimed / Selected Iteration None / Implementation PR Not started.
- A later mock-only visual/i18n child remains separate from I194 and requires its own owner,
  iteration and effective claim after D0 acceptance.
- P0-P4 shared Work Graph/evaluation prerequisites remain separate and gate real runtime binding.
- SESSION-009 remains the later multi-client/reconnect gate.

## Retrospective

I194 separated renderer direction from implementation authorization and recorded motion as an
input-first, semantic, cancellable and reduced-motion-equivalent quality boundary. Primary-source
evidence was pinned without importing dependencies; failed transitive metadata resolution remains
negative evidence. Later renderer work requires a separate governed child.
