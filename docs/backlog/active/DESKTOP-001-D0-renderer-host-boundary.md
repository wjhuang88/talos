# DESKTOP-001-D0: Desktop Renderer, Host, And Repository Boundary

| Field | Value |
|---|---|
| Story ID | DESKTOP-001-D0 |
| Type | Architecture / Governance Spike |
| Priority | P1 |
| Status | Complete — decision-only D0 packet accepted; renderer implementation and dependency authorization remain closed |
| Parent | DESKTOP-001 |
| Source | GitHub Issue #29; three-track development baseline |
| Selected Iteration | I194 |
| Depends On | DESKTOP-001 refined design baseline; ADR-042; ADR-052; repository hard constraints |

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
| Handoff / Release Condition | None — D0 is complete; any visual or renderer implementation requires a separate governed child, iteration, claim and worktree. |

## Identity / Goal / Value

Create the smallest auditable Desktop implementation prerequisite: one decision-only D0 slice that
turns the existing product direction into an explicit renderer/dependency/host/repository contract
without adding Desktop production code or treating design approval as implementation authorization.

The output must let a later mock-only visual/i18n slice answer, from accepted evidence rather than
assumption:

- which renderer and host dependency boundary is authorized;
- where a future Desktop host may live in the repository;
- which existing Talos crates it may consume and which dependency directions are forbidden;
- what native, FFI, build-script, platform, panic and `unsafe` implications require containment or
  separate review;
- how Chinese IME, bilingual text/layout, accessibility and reduced-motion requirements affect the
  renderer choice;
- which localization mechanism class is acceptable without yet adding an i18n dependency;
- what remains blocked on the shared P0-P4 Work Graph/evaluation chain or SESSION-009.

## Scope

D0 is documentation/decision work only.

- Re-evaluate the current GPUI direction against then-current primary-source ecosystem evidence at
  execution time; GPUI remains a design direction until this D0 decision is accepted.
- Compare only the minimum credible renderer/host alternatives needed to validate or reverse that
  direction; avoid an open-ended GUI framework survey.
- Inventory direct and material transitive dependency implications relevant to Talos hard
  constraints: native libraries, C/C++/Objective-C/system APIs, build scripts, FFI, generated code,
  platform SDK assumptions, panic boundaries and any visible `unsafe` ownership.
- Define the future host integration boundary for macOS, Windows and Linux, including window/event
  loop ownership, process lifetime, shutdown, input/IME, accessibility, reduced-motion and failure
  containment responsibilities at the level needed for an ADR/security review.
- Decide repository placement and dependency direction. The decision must reconcile existing
  `talos-runtime`, `talos-session` and `talos-conversation` ownership before proposing any new
  presentation/host crate.
- Define the localization dependency selection criteria required by `I18N.md`, including stable
  keys/catalogs, interpolation/plural/count support, deterministic English fallback and runtime
  locale switching feasibility. D0 may select a mechanism/library direction but does not add the
  dependency.
- Produce a Proposed Desktop renderer/host ADR and security-review input matrix during the later
  D0 implementation PR.
- Record explicit reversal triggers, excluded capabilities and the gate from D0 into the first
  mock-only visual/i18n slice.
- Treat motion as an interaction-quality constraint: state-semantic, interruptible, input-first,
  reduced-motion aware and performance-budgeted; do not add decorative looping motion or motion that
  competes with the current Goal/Work state.

## Exclusions

D0 must not:

- create `talos-desktop`, `talos-work`, a second runtime, a second Agent engine, or a parallel
  Mission/Goal/Work/session/permission store;
- add GPUI, an i18n crate, native dependencies, build scripts, FFI, or `unsafe` code;
- modify workspace membership, `Cargo.toml`, `Cargo.lock`, production crates, workflows or release
  packaging;
- implement a window, renderer, widget, localization catalog, fixture UI or visual mock;
- execute a real Mission or bind any real Work Graph/evaluation state;
- add durable Desktop persistence, Completion, Evaluation, Approval, Artifact or Delivery behavior;
- implement session recovery, attach/detach, reconnect, multi-client or multi-window session
  semantics;
- activate or change I188, I189, SESSION-008-B, RUNTIME-005, SESSION-008 or ARCH-034-R04;
- modify or close Issues #45, #49 or #59;
- modify PR #120 or PR #121.

## Dependencies And Existing Foundations

D0 consumes the existing boundaries as facts; it does not reopen them.

- `RUNTIME-001` / `talos-runtime` is the existing embeddable Agent runtime facade. Desktop remains a
  host/client above this runtime, never a replacement runtime.
- ADR-042 and `talos-session` own durable runtime transcript/session state. D0 cannot invent a
  Desktop-owned durable session authority.
- ADR-052 keeps `talos-runtime` as the supported embedding facade and keeps
  `talos-conversation` experimental/product-oriented rather than promising a generic UI SDK.
- Existing Todo semantics in `talos-session` are migration input for the future shared Work Graph;
  D0 does not implement that migration.
- `VALIDATION-001` remains the shared validation/evidence producer; D0 does not turn validation into
  independent Goal/Mission evaluation.
- `SESSION-009` remains the later gate for attach/reconnect/multi-client behavior, but it does not
  block a future local mock-only single-window visual slice.
- The DESKTOP-001 P0-P4 shared chain remains the gate for real Mission/Work Graph/Evaluation binding.

## Three-Track Coordination And Base Record

This owner was prepared from:

- target branch: `main`;
- observed `main` base: `c4bd9606c8bae63cb9bf11becd45846bf0805982`;
- common three-track base: `23e4174bcfb036602ce2145026b872ec5c517289`.

Observed non-terminal iteration disposition before creating this governance branch:

| Iteration / Work | Observed State | D0 Disposition |
|---|---|---|
| I159 | Blocked | Keep blocked; no overlap. |
| I160 | Blocked | Keep blocked; no overlap. |
| I161 | Blocked | Keep blocked; no overlap. |
| I162 | Blocked | Keep blocked; no overlap. |
| I164 | Paused | Keep paused; do not resume through Desktop. |
| I188 | Planned / Claimed | Keep unactivated and unchanged. |
| I189 | Planned / Claimed | Keep unactivated and unchanged. |
| I193 / PR #210 | Planned / Claimed on `main` through merge `fb5a1f62` | Keep unactivated and unchanged; do not touch or reuse I193. Desktop uses I194 to avoid cross-track ID collision. |
| PR #120 / #121 | Archival recovery Draft PRs | Immutable and excluded. |

There was no Active or Review iteration on the observed target branch. The three-track baseline
explicitly permits non-overlapping Dashboard, Desktop and mainline-foundation lanes, but every D0
write/merge gate must refresh `main`, current non-terminal owners, open claims/implementation PRs and
branch overlap. A newer compatible `main` may become the eventual implementation base; the SHA above
is claim-preparation provenance, not permission to ignore later mainline changes.

Merge-CAS refresh on 2026-08-13 observed `main@0459b8afb1626783f21b54dbaf55a0ef84393cd7`,
including effective I193 claim merge `fb5a1f62`. The refreshed derived views retain the
Runtime/Session lane alongside this proposed Desktop lane.

Implementation base and worktree refresh on 2026-08-13 observed
`main@f778543c7ceeb2a099eb3863fc8259da68d02195` after PR #211 merge. The implementation branch is
`feat/desktop-I194-d0-boundary` in `/private/tmp/talos-i194`, with merge target `main`.

## Decision Links And Constraints

- `AGENTS.md`: Rust first; no arbitrary native/FFI expansion; no `unsafe` without ADR; external
  native/panic-prone dependency calls must fail safely at the integration boundary.
- DESKTOP-001 and its proposal/design/i18n baseline define product intent but explicitly do not
  authorize implementation dependencies.
- ADR-042: durable session ownership remains in `talos-session`.
- ADR-052: `talos-runtime` remains the primary SDK facade; do not create a speculative shared UI SDK
  or composition crate merely for Desktop.
- Renderer dependencies must never flow into `talos-core` or `talos-runtime`.
- TUI and Desktop remain independent renderers; neither may depend on the other.
- Locale is presentation state. Locale choice or display text must never become Mission, Goal,
  WorkUnit, Evaluation, Evidence, Artifact, Delivery, protocol, persistence, command or enum
  identity.
- D0 cannot convert the proposed Work Graph/evaluation model into canonical implemented state.

## Uncertainty And Validation Path

The GPUI direction and concrete localization library are intentionally not treated as already
accepted implementation choices.

During the later D0 implementation PR:

1. refresh current `main` and repeat the complete non-terminal/overlap inventory;
2. collect current primary-source evidence for candidate renderer and localization dependencies;
3. trace material native/unsafe/build/platform implications and failure boundaries;
4. reconcile repository/dependency direction with the actual current workspace graph;
5. publish a Proposed ADR plus a security-review matrix with explicit exclusions and reversal
   triggers;
6. run repository governance and locked validation required by the exact diff;
7. obtain independent natural-person exact-head architecture/security review;
8. repeat merge-time CAS and prove the merged content equals the reviewed head.

If dependency evidence is insufficient, platform/IME requirements are not demonstrable, or a
candidate conflicts with repository hard constraints, D0 remains Review/Blocked and no renderer
implementation dependency is authorized.

## State / Status Owners

- D0 Story scope, acceptance and claim: this file.
- D0 activation/execution evidence: `docs/iterations/I194-desktop-renderer-host-boundary.md`.
- Parent product direction: `docs/backlog/active/DESKTOP-001-desktop-product-direction.md`.
- Product/architecture baseline: `docs/proposals/talos-desktop-goal-oriented-workspace.md`.
- Visual baseline: `docs/design/talos-desktop/DESIGN.md`.
- Internationalization baseline: `docs/design/talos-desktop/I18N.md`.
- Runtime/session ownership: their existing Story/ADR/crate owners.
- Current operating view: `docs/BOARD.md`.
- Compact backlog selection view: `docs/backlog/PRODUCT-BACKLOG.md`.
- Source discussion: GitHub Issue #29.

## User-Facing Documentation

D0 adds no user-visible Desktop behavior. README/user documentation must not claim Desktop,
GPUI-based UI, bilingual Desktop behavior or Mission/runtime binding as shipped or implemented.

## Required Reads

- `AGENTS.md`
- `docs/sop/AGENT-COLLABORATION.md`
- `docs/sop/REQUIREMENT-INTAKE.md`
- `docs/sop/START-ITERATION.md`
- `docs/sop/ITERATION-WORKFLOW.md`
- `docs/sop/GIT-WORKFLOW.md`
- `docs/sop/TESTING.md`
- `docs/backlog/active/DESKTOP-001-desktop-product-direction.md`
- `docs/proposals/talos-desktop-goal-oriented-workspace.md`
- `docs/proposals/talos-desktop.md`
- `docs/design/talos-desktop/DESIGN.md`
- `docs/design/talos-desktop/I18N.md`
- `docs/reference/DESKTOP-I194-DEPENDENCY-SECURITY-MATRIX.md`
- `docs/backlog/active/RUNTIME-001-embeddable-agent-runtime-api.md`
- `docs/backlog/active/SESSION-009-multi-client-session-architecture.md`
- `docs/backlog/active/TODO-001-session-todo-list.md`
- `docs/backlog/active/TODO-002-todo-mutation-reliability.md`
- `docs/backlog/active/VALIDATION-001-internal-validation-service.md`
- `docs/decisions/042-embedded-durable-runtime-session-boundary.md`
- `docs/decisions/052-sdk-publication-and-composition-boundary.md`
- `crates/talos-runtime/`
- `crates/talos-session/`
- `crates/talos-conversation/`

## Acceptance For D0 Governance / Decision Work

- [x] Current renderer/dependency evidence is recorded from primary sources and is dated to the D0
      execution window rather than copied from the earlier product proposal.
- [x] The ADR makes an explicit authorize / do-not-authorize decision for the selected renderer and
      states that the earlier GPUI design direction alone was not implementation authority.
- [x] Direct/material transitive native, FFI, build-script, platform, panic and `unsafe` implications
      are auditable, with containment/security-review ownership identified.
- [x] Repository placement and dependency arrows are explicit and prove no renderer dependency can
      flow into `talos-core`, `talos-runtime` or a second runtime/session/work authority.
- [x] Host integration responsibilities cover macOS, Windows and Linux at the boundary needed for a
      later mock-only slice, including shutdown/failure containment, text/input/Chinese IME,
      accessibility and reduced-motion implications.
- [x] Localization mechanism selection criteria cover `zh-CN`, `en-US`, deterministic English
      fallback, stable message identity, interpolation/count behavior and locale-neutral domain
      identity without adding an i18n dependency in D0.
- [x] The ADR/security packet names all exclusions and the exact later gate for the mock-only visual
      slice and real P0-P4/runtime binding.
- [x] No Cargo/workspace/production-code/dependency/fixture/UI implementation change exists in D0.
- [x] Exact-head CI, both governance validators, applicable locked checks, independent natural-person
      exact-head review and merge-time CAS are recorded before acceptance.

Current evidence status: primary-source snapshots for GPUI and the minimum Iced comparison were
retrieved and pinned by commit on 2026-08-13. They establish candidate capability/risk facts only;
full selected-release lock closure, SBOM/license review, platform tests, panic containment and motion
benchmarks remain Review residuals. Renderer/dependency authorization stays closed.

The crates.io metadata refresh confirmed published candidates `gpui 0.2.2` (Apache-2.0; default
font-kit/Wayland/X11/Windows-manifest features) and `iced 0.14.0` (MIT; Rust 1.88; default
wgpu/tiny-skia/X11/Wayland features). A disposable `cargo metadata` probe could not complete the
full graph because the registry proxy became unavailable while resolving transitive packages. This
is negative evidence for authorization: no lockfile, SBOM, build-script or license-closure claim is
made from the partial probe.

## Residual Destination

After D0 is accepted, a separate child owner/iteration/claim may authorize the first mock-only
visual/i18n slice. That later slice may use presentation-local ephemeral state or fixtures only and
must independently prove `zh-CN`, `en-US`, deterministic fallback, reduced-motion, Chinese IME,
bilingual layout and locale-neutral canonical identity. Real Mission/runtime/work/evaluation binding
remains blocked by the relevant P0-P4 shared contracts and APIs.

## Completion Evidence

Completion Commit: `0a47208ce6fad23c706ebede8b3d07111b9303dc`

- PR #215 merged to `main` as `1beaca68b98b56aaff42d952b7dbbc7519304740`.
- Independent natural-person approval comment `5278769979` binds the exact completion head.
- Exact-head CI `31687636396` passed all applicable jobs; both governance validators reported 0
  warnings and `git diff --check` was clean.
- The net D0 change is eight documentation files only, with no Cargo, crate, dependency, native,
  FFI, `unsafe`, UI or implementation artifact.
- ADR-059 remains Proposed. Full lock/SBOM closure, platform execution, panic containment and motion
  measurements remain later renderer-implementation gates and are not authorized by this closeout.
