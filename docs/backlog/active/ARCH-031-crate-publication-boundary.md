# ARCH-031: Crate Publication Boundary And Distribution Architecture

**Status**: In Progress (publication classification enforced; SDK/composition boundary decided by ADR-052)
**Priority**: P2
**Created**: 2026-06-28
**Source**: User request to make Talos-owned capabilities independently publishable as crates,
not only available through `talos-runtime`
**Depends on**: `RUNTIME-001`; ADR-024; ADR-052; `TOOL-012`; `TOOL-013`; `DIST-001`; `REL-002`

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned — child Stories require separate non-overlapping claims |
| Claimed At | Not applicable |
| Source Issue | None |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | None — Epic parents are not implementation units |
| Last Updated | 2026-08-14 |
| Handoff / Release Condition | Close each selected child through its own owner, iteration, effective claim, implementation PR, validation evidence, independent exact-head review and merge-time CAS. |

## Problem

Talos now has a pre-1.0 embeddable SDK facade in `talos-runtime`, but many self-written
capabilities are valuable outside the full runtime package: configuration, permissions, sandboxing,
provider adapters, tools, skill loading, session storage, memory, plugin foundations, MCP/RPC, and
conversation state.

These crates are currently structured as workspace libraries, not as publish-ready crates. They
mostly lack publish-compatible internal dependency specs, crate-specific public API commitments,
feature flags around optional weight, and a publication order. That makes future external reuse
unclear and lets product-layer coupling hide inside internal dependencies.

## Scope

- Define which workspace crates are reusable library surfaces, SDK facade surfaces, implementation
  surfaces, and product assembly surfaces.
- Make publishability a crate-boundary quality gate even before actual crates.io publication.
- Prepare manifests and documentation so selected crates can pass `cargo publish --dry-run`.
- Keep `talos-runtime` as the primary SDK facade while allowing lower-level crates to be consumed
  directly when their APIs are documented.
- Use the ripgrep-style pattern: binary/product crates aggregate independently reusable library
  crates.
- Add a Cargo-native binary installation path for the product CLI without making `talos-cli` a
  supported library dependency.

## Non-Goals

- Do not publish crates.io packages under this story without an explicit release/ADR gate.
- Do not split every crate immediately.
- Do not make `talos-cli` or `talos-tui` required dependencies for embedders.
- Do not promise a stable `talos-cli` library API as part of `cargo install` support.
- Do not convert release archives, installers, or optional runtime assets into crates.io packages.
- Do not make independent per-crate versioning a pre-1.0 requirement.

## Candidate Slices

1. **Publication matrix**
   - Inventory every workspace crate.
   - Record dependency order, intended audience, public API status, default features, optional
     weight, and dry-run readiness.

2. **Manifest readiness**
   - Add publish-compatible internal dependency specs: `version` plus `path`.
   - Add package metadata, readme/docs pointers, categories/keywords where useful, and crate-level
     docs.
   - Decide which crates need `publish = false` until their surface is intentionally product-only
     or unstable.

3. **First-wave dry-run**
   - Run `cargo publish --dry-run` in dependency order for the lowest-risk library crates.
   - Record failures in the matrix instead of widening scope opportunistically.

4. **Capability feature gates**
   - Identify heavy/default-weight features in `talos-tools`, `talos-tui`, storage, provider, and
     parser-related crates.
   - Add or plan feature flags before any broad public publication.

5. **Docs and release gate**
   - Update README and architecture docs to distinguish binary install, runtime SDK, and
     standalone crate consumption.
   - Draft a release/ADR gate for the first real crates.io publish.

6. **Cargo install package path**
   - Treat `talos-cli` as a binary package candidate, not a reusable library crate.
   - Plan the supported install command as `cargo install talos-cli --bin talos` because the
     top-level `talos` package name is already taken on crates.io.
    - Before removing `publish = false`, verify package metadata/readme, included binary target,
      install from local path, publish dry-run, and README install instructions.

## Child Stories / Execution Slices

The v0.6 runtime productization program converts the Candidate Slices above into bounded,
sequenced child stories. Each child story owns its own acceptance; ARCH-031 remains the parent
and stays `In Progress`. See
[program](../../tasks/2026-07-26-v0.6-runtime-productization-program.md) for the activation order
and gates.

| Child Story | Owns Candidate Slice | Selected Iteration | Initial State | Activation Gate |
|---|---|---|---|---|
| [ARCH-031-A](ARCH-031-A-talos-tools-feature-boundary.md) | 4 (Capability feature gates — `talos-tools`) | I159 | Complete — PR #236 merge `f79c1ead` | Closed |
| [ARCH-031-B](ARCH-031-B-shared-cli-runtime-composition.md) | (new) Shared CLI/runtime internal composition | I160 | Review / implementation merged | Claim PR #238 merged as `71faf844`; implementation PR #240 merged as `97556149`; Completion Commit pending |
| [ARCH-031-C](ARCH-031-C-sandbox-fallback-and-coding-preset.md) | (new) `SandboxFallbackPolicy` + `RuntimePreset::coding()` (ADR-052) | I161 | Refinement — blocked on ARCH-031-B + security review | I160 Complete; independent security review scheduled |
| [ARCH-031-D](ARCH-031-D-v0.6-sdk-publication-readiness.md) | 1/3/5 (matrix, dry-run, docs/release gate) at v0.6 alignment | I162 | Refinement — blocked on ARCH-031-C | I161 Complete; workspace green |

Candidate Slices 2 (Manifest readiness) and 6 (Cargo install path) remain open acceptance items
under ARCH-031 directly and are not yet satisfied (see Acceptance Criteria). I160 is the only
active child; later child gates remain sequential and ADR-053-gated.

## Acceptance Criteria

- [x] `docs/proposals/talos-crate-distribution-architecture.md` is accepted, superseded, or
      converted into an ADR before implementation begins. (Superseded for decision authority by
      ADR-052 on 2026-07-24; retained as background.)
- [x] A publication matrix covers **all current workspace crates** (all 21 members, including
      `talos-models`) and classifies each crate's intended support level, with current workspace
      version, latest published registry version, support classification, and readiness state kept
      as distinct columns/sections. Satisfied by the 2026-07-26 reconciliation in commit `3d0f32a`;
      the matrix separates current workspace version (0.5.0), latest registry version, support
      classification, and readiness state, and moves 0.2.0 evidence under Historical Evidence.
- [ ] Publishable crates have complete Cargo package metadata: `description`, license, `repository`,
      crate-level `//!` docs / readme target, and a stated support boundary. `keywords`/`categories`
      are explicitly NON-mandatory and are not part of this acceptance item. (Several published
      crates still lack crate-level `//!` docs per the A1 audit; this item cannot close until each
      target crate meets the real minimum above.)
- [x] The first selected wave passed `cargo publish --dry-run` in dependency order at the then-0.2.0
      baseline, and real publishes succeeded (see Historical Evidence). Subsequent waves are recorded
      with owning follow-up items.
- [x] `talos-runtime` remains the documented SDK facade; `talos-agent` documents direct-use
      caveats at the crate level. Satisfied by `crates/talos-agent/src/lib.rs` in commit `3d0f32a`
      (implementation-only, not a supported SDK; embedders use `talos-runtime::RuntimeBuilder`;
      pre-1.0 implementation API may change more frequently). The facade contract itself lives in
      `RUNTIME-SDK-CONTRACT.md`.
- [ ] Heavy optional capabilities have REAL feature gates (optional dependencies + gated
      modules/re-exports) or a recorded split trigger. (I159 implements this boundary for
      `talos-tools`: default `file-read + search`, optional heavy families, and explicit CLI
      `coding`. I162 must still audit the other publication targets before this parent-wide item can
      close.)
- [ ] README, README.zh-CN, and architecture docs explain crate distribution when the first
      implementation slice lands.
- [ ] The publish plan defines and validates the Cargo install path for the CLI binary.

## Validation

- `cargo metadata --no-deps --format-version 1`
- `cargo publish --dry-run -p <crate>` for selected first-wave crates
- `cargo test -p <crate>` for each selected crate
- `cargo check --workspace`
- `scripts/validate_project_governance.sh .`

## Execution Notes

### I159 Closeout Evidence (2026-08-14)

- Completion Commit: `d886917e45d5ca0f110e111b966cd379485e3580` and
  `34c09b142766c70ac62ef24424ed035f2fa921a5` (child ARCH-031-A/I159 implementation
  evidence; the parent Epic remains `In Progress` and is not Complete).
- I159 implementation PR #236 merged at `f79c1ead1cd3a547797dea3666295f510d88a13d`.

2026-06-29:

- Accepted `docs/proposals/talos-crate-distribution-architecture.md` as the implementation
  baseline for publication-readiness work; real publish/name reservation remains blocked pending
  explicit maintainer approval.
- Added `docs/reference/CRATE-PUBLICATION-MATRIX.md`.
- Added workspace repository/homepage metadata and publish-compatible `version = "0.2.0"` plus
  `path` specs for Talos crate-to-crate dependencies.
- Checked crate name availability with `cargo search <name> --limit 3`: no exact matches found for
  current workspace crate names; `talos-core` returned only the near-match `talos-core-rs`.
- Checked top-level `talos`: it is already taken by an unrelated crate, so Cargo package
  publication should use the current `talos-*` names.
- `cargo publish --dry-run --allow-dirty -p talos-core` passed.
- `cargo publish --dry-run --allow-dirty -p talos-skill` passed.
- `talos-config`, `talos-permission`, and `talos-session` dry-runs are correctly blocked until
  `talos-core` exists in the crates.io index.
- After maintainer approval, real `cargo publish -p talos-core` was attempted from clean commit
  `30c9abc`, but crates.io rejected the upload because the publisher account does not have a
  verified email address. No crate was published and no name was reserved.
- After email verification, real publishes succeeded from clean commit `c8884f6`:
  `talos-core 0.2.0`, `talos-skill 0.2.0`, `talos-config 0.2.0`,
  `talos-permission 0.2.0`, and `talos-session 0.2.0`.
- `cargo search talos-core --limit 5` confirmed `talos-core = "0.2.0"` is visible in the
  crates.io index before publishing the core-dependent crates.
- Second-wave dry-runs succeeded for `talos-plugin`, `talos-provider`, `talos-conversation`,
  `talos-memory`, and `talos-exploration`.
- Real publishes succeeded for `talos-plugin 0.2.0` and `talos-memory 0.2.0`.
- Real `cargo publish -p talos-exploration` initially passed packaging and verification but
  crates.io rejected upload with a new-crate rate limit. Retry after 2026-06-29 07:28:33 GMT was
  successful, publishing `talos-exploration 0.2.0`.
- Added crate-level support boundary docs for `talos-provider`, `talos-conversation`, and
  `talos-rpc` in commit `92a0c99`.
- `cargo test -p talos-provider -p talos-conversation -p talos-rpc` passed.
- `cargo publish --dry-run -p talos-provider`, `cargo publish --dry-run -p talos-conversation`,
  and `cargo publish --dry-run -p talos-rpc` passed from clean commit `92a0c99`.
- Real publishes succeeded for `talos-provider 0.2.0`, `talos-conversation 0.2.0`, and
  `talos-rpc 0.2.0`. Each package is visible via `cargo search`.
- Classified remaining crates:
  `talos-sandbox`, `talos-tools`, `talos-agent`, `talos-runtime`, and `talos-mcp` are
  gate-before-publish candidates; `talos-cli`, `talos-tui`, and `talos-evolution` are product-only.
- Added `publish = false` to `talos-cli`, `talos-tui`, and `talos-evolution` so product-only crates
  cannot be accidentally published through `cargo publish --workspace`.
- Created the two-month crate distribution hardening plan and programmer handoff to delegate
  published-crate docs, product-only guards, high-risk gates, runtime dependency closure,
  user-facing distribution docs, and feature tracks for WEBFETCH bounded document capture,
  MODEL-004 runtime catalog integration, CONF-001 CLI config editing, and AGENT-002-B shared skill
  discovery without authorizing additional real publishes.
- Reconciled the two-month plan against I045 on 2026-06-29: MODEL-004 M1/M2 and the CONF-001
  `--config-*` flag surface are baseline work already completed; remaining delegated work is
  MODEL-004 TUI/exit metadata, CONF-001 subcommand/validation hardening, WEBFETCH document capture,
  shared skill discovery, and A1-A8 distribution gates.

2026-06-30:

- Added Cargo-native binary install to the publish plan. `talos-cli` is now classified as a
  binary package candidate for `cargo install talos-cli --bin talos`, while its library API remains
  unsupported. Removing `publish = false` requires a dedicated install-package release gate, not a
  reusable-library publication gate.

2026-07-02:

- T133 produced `docs/reference/PUBLISH-GATE-PACKET-2026-07-02.md` for `talos-cli` and
  `talos-runtime`.
- `cargo publish --dry-run -p talos-cli` is intentionally blocked by `publish = false`.
- `cargo publish --dry-run -p talos-runtime` remains blocked by unpublished `talos-agent`.
- `talos-dashboard` was added to the publication matrix and publish guard as a product-only
  `publish = false` crate.
- No crate was published, no `publish = false` guard was removed, and no release tag was created.

2026-07-24:

- ADR-052 (SDK Publication And Runtime Composition Boundary) was accepted, deciding the SDK and
  composition questions this story had left open:
  - Publication proceeds via **route A** in dependency order
    `talos-sandbox` → `talos-tools` → `talos-agent` → `talos-runtime`; the ADR is architecture and
    sequencing only and is **not** authorization to publish.
  - `talos-agent` is publishable as an **implementation dependency only**, not a supported SDK
    entrypoint; embedders continue to use `talos-runtime`. The crate-level implementation-only
    caveat landed in `crates/talos-agent/src/lib.rs` (commit `3d0f32a`, 2026-07-26), satisfying the
    corresponding Acceptance item.
  - Sandbox fallback becomes an explicit caller choice: `SandboxFallbackPolicy`
    (`Deny`/`Ask`/`AllowUnsandboxed`, default `Deny`); `talos-sandbox` stays policy-neutral.
  - `talos-tools` gets **lightweight read-only defaults**; write/shell/git/network/web/image and
    heavy code-intelligence families are opt-in or enabled via an explicit preset (feeds Candidate
    Slice 4 and the `talos-tools` gate).
  - `talos-runtime` gains an explicit overridable `RuntimePreset::coding()` that never bypasses the
    permission pipeline.
  - CLI and SDK keep **separate public entrypoints sharing one internal composition**; a new
    `talos-runtime-core`-style crate is **not** authorized until a later demonstrated need.
  - A general-purpose third-party UI SDK is **deferred**; `talos-conversation` remains experimental
    and product-oriented (docs must not market it as a supported UI framework).
- No crate was published, no `publish = false` guard was removed, and no release tag was created.
  Implementation is activated through ARCH-031 slices under normal iteration governance; ADR-052
  Phases 1–5 map onto the Candidate Slices and remaining Acceptance items.

2026-07-26:

- Commit `3d0f32a` completed the ADR-052 documentation boundary repair: ADR-052 clarifying
  amendments, ARCH-031 acceptance/Required Reads/Open Questions cleanup, restructured
  `CRATE-PUBLICATION-MATRIX.md`, `RUNTIME-SDK-CONTRACT.md` planned-additions + session/provenance
  fixes, crate-level docs for `talos-agent`/`talos-conversation`/`talos-tools`, proposal superseded
  marking, `ARCHITECTURE.md` crate inventory, and `AGENTS.md` membership wording.
- Commit `fed1496` added the quarantined `talos-models` `publish = false` guard.
- No planned API was implemented: `SandboxFallbackPolicy`, `RuntimePreset`, `talos-tools` feature
  gates, and the shared CLI/SDK composition layer remain future implementation work.
- No crate was published; no tag or GitHub Release was created; ARCH-031 is NOT closed.
- ARCH-031 remains `In Progress`; the still-open Acceptance items are real feature-gate
  implementation, complete crate metadata, distribution docs, and the Cargo install path.

2026-08-14 I159 implementation checkpoint:

- Draft PR #236 implements the `talos-tools` portion of Candidate Slice 4: default
  `file-read + search`, optional write/document/shell/Git/network/image/code-intelligence families,
  and an explicit `coding` aggregate selected by `talos-cli`.
- Local feature, product-parity and workspace commands passed before implementation commit
  `d886917e`, but exact-head CI `31794297165` correctly rejected this changed active Epic because it
  lacked the explicit Unclaimed claim metadata now recorded above. The earlier local validator ran
  without a PR-base binding and did not inspect the complete branch diff. I159 remains Active until
  the corrected head passes exact-head CI and independent review; the parent-wide feature audit and
  real publication remain open under I162/I203.
- On the corrected working tree, `COLLABORATION_VALIDATION_BASE=origin/main` makes the collaboration
  validator cover the complete PR diff; it reports 0 warnings, and the base-bound full
  `release_preflight.sh` completes successfully. The correction is recorded by commits `34c09b14`
  and `57bc1585`; this remains local follow-up evidence until GitHub validates the resulting exact
  head.
- PR #236 then passed exact-head CI `31801484313` 5/5, independent approval `5293622712` and
  merge-time CAS, merging as `f79c1ead1cd3a547797dea3666295f510d88a13d`. ARCH-031-A/I159 is
  Complete; ARCH-031-B/I160 is Ready/Planned/Unclaimed, while ARCH-031 remains In Progress.

## Required Reads

- `docs/tasks/2026-07-26-v0.6-runtime-productization-program.md`
- `docs/decisions/052-sdk-publication-and-composition-boundary.md`
- `docs/decisions/053-tool-registration-composition.md` (Proposed — gates ARCH-034-R01/I158, not ARCH-031 directly)
- `docs/backlog/active/ARCH-031-A-talos-tools-feature-boundary.md`
- `docs/backlog/active/ARCH-031-B-shared-cli-runtime-composition.md`
- `docs/backlog/active/ARCH-031-C-sandbox-fallback-and-coding-preset.md`
- `docs/backlog/active/ARCH-031-D-v0.6-sdk-publication-readiness.md`
- `docs/tasks/2026-06-29-crate-distribution-hardening-two-month-plan.md`
- `docs/tasks/2026-06-29-programmer-handoff-crate-distribution-hardening.md`
- `docs/iterations/I045-product-readiness-model-lifecycle-observability.md`
- `docs/backlog/active/WEBFETCH-001-web-and-document-fetch-tools.md`
- `docs/backlog/active/MODEL-004-catalog-runtime-integration.md`
- `docs/backlog/active/CONF-001-config-editing.md`
- `docs/backlog/active/AGENT-002-dotagents-protocol-support.md`
- `docs/proposals/talos-crate-distribution-architecture.md`
- `docs/reference/ARCHITECTURE.md`
- `docs/backlog/active/RUNTIME-001-embeddable-agent-runtime-api.md`
- `docs/backlog/active/TOOL-012-tool-family-progressive-loading.md`
- `docs/backlog/active/TOOL-013-multi-resource-tool-permissions.md`
- `docs/backlog/active/DIST-001-optional-runtime-asset-distribution.md`
- `docs/backlog/active/REL-002-v1-self-bootstrap-release-gate.md`
- `docs/decisions/024-embeddable-runtime-api-boundary.md`
- `docs/decisions/025-ripgrep-library-search-engine.md`
- `docs/decisions/052-sdk-publication-and-composition-boundary.md`
- `Cargo.toml`
- `crates/*/Cargo.toml`

## Open Questions

1. ~~Should the first real publish happen before or after the 1.0 self-bootstrap gate?~~
   **Resolved by historical fact**: the first real publishes already occurred at the 0.2.0 baseline
   on 2026-06-29 (see Historical Evidence in `CRATE-PUBLICATION-MATRIX.md`). Future publishes at the
   current 0.5.0 workspace line still require the maintainer gate.
2. Which crate names should be reserved on crates.io before APIs are fully stable? (Partially
   answered: the 0.2.0 first/second/integration waves already reserved core library names; remaining
   gate crates — sandbox/tools/agent/runtime/mcp — are not yet reserved.)
3. ~~Should `talos-tui` be a reusable UI library package or product-only implementation detail?~~
   **Resolved by ADR-052**: no general-purpose UI SDK is committed now; `talos-tui` stays
   product-only and `talos-conversation` remains experimental/product-oriented. Revisit only when a
   real second external frontend needs a stable contract.
4. Should post-1.0 crates move to independent versions, or stay lockstep for user simplicity?
5. Should the CLI Cargo package remain `talos-cli`, or should a later release choose another
   available package name for product branding while still shipping the `talos` binary?
