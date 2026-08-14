# ARCH-031-B: Shared Internal CLI And Runtime Composition

| Field | Value |
|---|---|
| Story ID | ARCH-031-B |
| Type | Architecture / Runtime Composition Story |
| Parent Epic | ARCH-031 |
| Priority | P1 |
| Status | In Progress / I160 Active / Claimed — implementation baseline starts at `main@71faf844` |
| Depends on | ADR-052; ADR-053 Accepted; I158 Complete; I159 Complete |
| Selected Iteration | I160 (Active / Claimed) |
| Value | CLI and SDK use one tested internal composition path without merging their public entrypoints |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | `Codex / GPT-5 mainline session` |
| Work Slice | `ARCH-031-B / I160` only: shared internal CLI/runtime composition with separate public entrypoints and behavior-equivalence evidence; no preset, fallback, version, tag or publication. |
| Claimed At | 2026-08-14 |
| Source Issue | None |
| Governance Claim PR | #238 |
| Authorization Mode | Independent review |
| Authorization Evidence | Maintainer-directed release prerequisite; `@wjhuang88` is the shared GitHub account and natural-person separation is limited. PR #238 exact head `edcbe47f81798480447962048fe4f50bb69fdba1` passed CI `31815122170`, independent approval `5295372157`, and merge-time CAS before merge `71faf8440466668daeef0afd0e779be072978b01` established the claim on `main`. |
| Implementation PR | #240 |
| Last Updated | 2026-08-15 |
| Handoff / Release Condition | Execute only the bounded shared-composition slice from `main@71faf844`; submit exact-head validation and independent review before merge. Release/version/tag/publication remain excluded. |

## Problem

Talos CLI and `talos-runtime` need consistent construction of tools, permissions, sessions, hooks,
skills, sandbox inputs, and product defaults. ADR-052 requires separate public entrypoints sharing
one internal composition, but the shared implementation does not yet exist.

Copying CLI registry construction into `talos-runtime` would preserve duplication and make
`RuntimePreset::coding()` unsafe to implement.

## Goal

Create one internal, explicit composition implementation used by both CLI and runtime adapters while:

- keeping `talos-runtime::RuntimeBuilder` as the supported SDK facade;
- keeping CLI/TUI product behavior intact;
- preserving caller override points;
- not adding a new crate.

## Placement Decision

Start in an existing crate/module chosen by accepted ADR-053 and the I158 responsibility map.

Preferred order:

1. a focused internal module in the crate that owns the accepted composition contract;
2. an internal module in `talos-runtime` only if CLI can depend on it without reversing product/SDK
   boundaries;
3. an internal module in `talos-cli` is NOT acceptable as the final shared owner because the SDK must
   not depend on CLI;
4. a new crate is forbidden in this Story.

The iteration must record the chosen owner before code changes.

## Scope

- Define an internal composition input/configuration structure.
- Compose the authoritative tool contributions from I158.
- Inject only the dependencies needed by selected profiles.
- Apply existing permission wrappers outside tool-owning crates.
- Preserve existing session, hook, skill, plugin, provider, and sandbox wiring.
- Add thin CLI and runtime adapters.
- Add equivalence tests between old/current CLI behavior and the shared path.
- Keep old paths temporarily if needed for side-by-side testing, then remove only proven duplicates.

## Explicit Exclusions

- no public `RuntimePreset` API;
- no `SandboxFallbackPolicy`;
- no change to default permission decisions;
- no change to sandbox availability/fallback behavior;
- no new tool or product capability;
- no new crate;
- no real publish;
- no version bump;
- no TUI renderer refactor;
- no general-purpose UI SDK.

## Responsibility Map

| Responsibility | Owner after Story |
|---|---|
| Generic tool/contribution contracts | `talos-core` per ADR-053 |
| Tool factories | implementing crates |
| Profile/capability selection | shared internal composition |
| Permission wrapper selection | outer composition adapter |
| CLI command/TUI lifecycle | `talos-cli` |
| Supported embedder API | `talos-runtime` |
| Turn loop | `talos-agent` |
| Sandbox implementation | `talos-sandbox` |
| Sandbox fallback policy | still future ARCH-031-C |

## Claim Preparation Checkpoint (2026-08-14)

- I159/ARCH-031-A is complete on current `main` through PR #236 merge
  `f79c1ead1cd3a547797dea3666295f510d88a13d`.
- A dedicated I160 governance claim is proposed in PR #238 from
  `main@1b129c951df22a7de63e14735e02b1e8a79a9cd7`.
- The claim is not effective until its finalized owner record is merged to `main`; no
  implementation branch or Rust/Cargo change is authorized by this checkpoint.

## Required Profiles For Equivalence

The exact existing modes must be inventoried, but at minimum prove:

- print mode;
- interactive/TUI mode;
- MCP mode if currently built by CLI;
- minimal embedded runtime;
- product coding composition internal profile (not yet exposed as `RuntimePreset`).

## Invariants

- CLI and SDK public entrypoints remain separate.
- `talos-runtime` does not depend on `talos-cli` or `talos-tui`.
- tool-owning crates do not learn product mode policy.
- permission and sandbox decisions remain explicit.
- plugin registration remains explicit.
- no tool can become model-executable merely because it is registered but filtered from presentation.
- current mode-specific intentional differences remain documented and tested.
- caller-provided runtime tools continue to work.

## Acceptance

### Structure

- [ ] one shared internal composition owner is recorded.
- [ ] CLI and runtime adapters call the same internal composition primitives.
- [ ] no new crate is added.
- [ ] no product crate becomes a dependency of a reusable SDK/library crate.
- [ ] duplicate registration lists removed only after equivalence proof.
- [ ] architecture docs show the final responsibility map.

### Equivalence

For each mode/profile, tests record:

- tool-name set;
- contribution sources;
- wrapper/permission type or observable decision behavior;
- required construction inputs;
- intentional exclusions.

- [ ] current and new print registries are equivalent.
- [ ] current and new TUI registries are equivalent.
- [ ] current and new MCP registries are equivalent.
- [ ] plugin collision behavior is preserved.
- [ ] model capability filtering is preserved.
- [ ] snapshot-aware file tool grouping remains correct.
- [ ] runtime custom `.tool(...)` additions remain supported.

### Runtime evidence

- [ ] real print-mode command exercises tools through the shared path.
- [ ] real TUI session exercises at least one read and one permission-gated tool.
- [ ] embedded runtime fixture builds and runs through the shared path.
- [ ] no duplicate or missing tool appears.

### Validation

Run focused tests plus full locked validation. Record command output in I160.

## Expected Change Sites

- accepted ADR-053 owner module;
- `crates/talos-cli/src/registry.rs` and related mode builders;
- `crates/talos-runtime/src/lib.rs` or focused internal modules;
- tool contribution modules created by I158;
- equivalence/integration tests;
- `docs/reference/ARCHITECTURE.md`;
- Story/Iteration/Board owner docs.

## Stop And Escalate Conditions

Stop if:

- sharing requires `talos-runtime -> talos-cli` dependency;
- a new crate appears necessary;
- permission behavior changes;
- the Story requires implementing preset/fallback early;
- the new composition needs hidden global state;
- mode equivalence cannot be stated because current behavior is ambiguous.

## Required Reads

- `AGENTS.md`
- program plan
- ADR-024
- ADR-052
- accepted ADR-053
- ARCH-034-R01
- ARCH-031-A
- `docs/reference/ARCHITECTURE.md`
- `docs/reference/RUNTIME-SDK-CONTRACT.md`
- current CLI registry/mode builders
- current runtime builder and tool registration code

## Residual Destination

- public preset and sandbox fallback: ARCH-031-C;
- publication/versioning: ARCH-031-D;
- need for a dedicated composition crate: new evidence-backed ADR.

## 2026-08-14 Readiness Checkpoint

- I159/ARCH-031-A completed through implementation commits `d886917e` and `34c09b14`, exact-head
  CI `31801484313`, independent approval `5293622712` and PR #236 merge `f79c1ead`.
- The published objective, acceptance, exclusions and responsibility map remain unchanged.
- ARCH-031-B is Ready for a dedicated I160 claim. This checkpoint does not activate I160, create an
  implementation branch or authorize shared-composition code changes.

## 2026-08-15 Activation Checkpoint

- PR #238 exact head `edcbe47f81798480447962048fe4f50bb69fdba1` passed CI `31815122170`,
  independent approval `5295372157`, and merge-time CAS, then merged to `main` as
  `71faf8440466668daeef0afd0e779be072978b01`.
- The effective claim authorizes only ARCH-031-B/I160. The implementation worktree
  `/private/tmp/talos-i160-impl` and branch `feat/runtime-I160-shared-composition` start at that
  exact merge commit; no Rust/Cargo change existed at activation.
- I160 is Active. I161-I162 remain blocked, and release/version/tag/publication remain outside this
  Work Slice.

## 2026-08-15 Implementation Baseline

- `talos-tools/src/contributions.rs` already owns typed, source-labelled factories for shell,
  snapshot-aware/ordinary file, workspace, network, Git, image, and symbol contribution groups.
- `talos-cli/src/registry.rs` still repeats profile selection and contribution registration for
  print, TUI, and MCP; `talos-cli/src/mode_interactive.rs` has a fourth interactive selection
  path. The wrappers and scheduler/todo/plugin injections remain product-specific.
- `talos-runtime::RuntimeBuilder` currently accepts caller-provided `AgentTool` values through
  `.tool(...)` and registers them behind its permission adapter; it has no built-in composition
  path. `RuntimeBuilder::new()` must retain this minimal default.
- Shared owner decision: add a focused internal composition module to `talos-runtime`, backed by
  the existing `talos-tools` and `talos-session` contribution factories. CLI will consume the
  module through an explicitly documented internal bridge; no new crate and no CLI/TUI dependency
  in the SDK direction.
- Initial profile boundary: the shared module selects only contribution groups and construction
  inputs. CLI/runtime wrappers, scheduler/todo/plugin/status additions, presentation policy, and
  approval behavior remain in their respective adapters.

## 2026-08-15 Implementation Checkpoint

- Added `talos-runtime::composition` behind the opt-in `shared-composition` feature. It owns
  profile-specific construction and contribution-group selection using existing `talos-tools`
  factories; no new crate was added.
- CLI print/TUI/MCP builders now consume the shared groups. Existing wrapper policy is unchanged:
  `tree` remains unwrapped where required, Git read tools remain unwrapped, MCP keeps ordinary
  file constructors, and `read_image` remains excluded from MCP.
- `RuntimeBuilder::shared_tools()` explicitly consumes the runtime profile. `RuntimeBuilder::new()`
  remains minimal and custom `.tool(...)` additions remain supported. All runtime tools still pass
  through the existing permission adapter.
- Local evidence: runtime shared-composition tests 22/22 passed; CLI registry tests 29/29 passed;
  default runtime and shared feature locked checks passed; governance validators returned 0
  warnings. Exact implementation-head CI and independent review remain pending.
