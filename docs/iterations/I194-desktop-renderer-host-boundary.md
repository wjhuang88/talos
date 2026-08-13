# Iteration I194: Desktop Renderer, Host, And Repository Boundary

> Document status: Planned
> Published plan date: 2026-08-13
> Planned objective: decide the Desktop renderer/dependency/host/repository boundary and produce an independently reviewable ADR/security packet without adding Desktop production code or renderer dependencies.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: one accepted decision packet that makes a later mock-only visual/i18n slice authorizable without implying real Mission/runtime binding.
> Activation rule: this iteration is not implementation authority until its finalized Collaboration Claim exists on `main` and the current three-track overlap/CAS gate passes.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-5.6 Sol Desktop governance session 2026-08-13 |
| Work Slice | DESKTOP-001-D0 / I194 only: decide and document the Desktop renderer/dependency/host/repository boundary, current GPUI-or-alternative evidence, native/unsafe/security implications, localization selection criteria, and the later mock-only authorization gate. No production UI, dependency, runtime/domain, persistence, session, permission or P0-P4 implementation. |
| Claimed At | 2026-08-13 |
| Source Issue | #29 |
| Governance Claim PR | #211 |
| Authorization Mode | Independent review |
| Authorization Evidence | Independent natural-person exact-head architecture/security review is required on finalized PR #211 before merge; exact-head CI, both governance validators and merge-time CAS remain mandatory. Repository operations use shared GitHub account `wjhuang88`; a reviewer using the same account must explicitly disclose distinct natural-person identity. This proposed `Claimed` record is ineffective until merged to `main`. |
| Implementation PR | Not started |
| Last Updated | 2026-08-13 |
| Handoff / Release Condition | Merge finalized claim PR #211 to `main` after the required exact-head evidence, then refresh target-branch truth and create a real independent worktree from the claim merge or later compatible `main` before creating `feat/desktop-I194-d0-boundary`. |

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `DESKTOP-001-D0` | `DESKTOP-001` | Ready; proposed claim in PR #211 | DESKTOP-001 design baseline; ADR-042; ADR-052; repository hard constraints | Auditable renderer/dependency/host/repository ADR and security-review packet; no renderer implementation |

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
| I193 proposal / PR #210 | Draft claim proposal for SESSION-008-B; not effective on target branch | Do not touch; Desktop intentionally uses I194 to avoid cross-track iteration-ID collision. |
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
| 2026-08-13 | Selection | `DESKTOP-001-D0` selected as Planned I194 governance/decision work from `main@c4bd9606c8bae63cb9bf11becd45846bf0805982`. I193 was deliberately not used because open Draft PR #210 proposes it for SESSION-008-B. PR #211 now carries the finalized proposed claim, but ownership remains ineffective until target-branch merge. |

## Verification Evidence

- Governance preparation branch base: `c4bd9606c8bae63cb9bf11becd45846bf0805982`.
- Governance claim PR: #211.
- Current proposed-claim branch state records `Claimed`, but target-branch ownership remains unchanged until merge.
- Local checkout/worktree evidence is unavailable in the current execution environment because the
  container cannot resolve GitHub; this claim preparation uses the connected GitHub repository
  interface. A real independent worktree remains mandatory before any D0 implementation branch is
  created.
- Governance validators, exact-head CI, independent natural-person review and merge-time CAS remain
  pending on the finalized claim head.

## Completion Evidence

No completion evidence. A claim/governance/status commit cannot certify this iteration.

## Variance And Residuals

- DESKTOP-001 remains Deferred / Unclaimed / Selected Iteration None / Implementation PR Not started.
- A later mock-only visual/i18n child remains separate from I194 and requires its own owner,
  iteration and effective claim after D0 acceptance.
- P0-P4 shared Work Graph/evaluation prerequisites remain separate and gate real runtime binding.
- SESSION-009 remains the later multi-client/reconnect gate.

## Retrospective

Pending activation and execution.
