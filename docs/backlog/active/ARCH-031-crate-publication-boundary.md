# ARCH-031: Crate Publication Boundary And Distribution Architecture

**Status**: Planned
**Priority**: P2
**Created**: 2026-06-28
**Source**: User request to make Talos-owned capabilities independently publishable as crates,
not only available through `talos-runtime`
**Depends on**: `RUNTIME-001`; ADR-024; `TOOL-012`; `TOOL-013`; `DIST-001`; `REL-002`

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

## Non-Goals

- Do not publish crates.io packages under this story without an explicit release/ADR gate.
- Do not split every crate immediately.
- Do not make `talos-cli` or `talos-tui` required dependencies for embedders.
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

## Acceptance Criteria

- [ ] `docs/proposals/talos-crate-distribution-architecture.md` is accepted, superseded, or
      converted into an ADR before implementation begins.
- [ ] A publication matrix covers all workspace crates and classifies each crate's intended
      support level.
- [ ] Publishable crates have complete Cargo package metadata and publish-compatible internal
      dependency specs.
- [ ] The first selected wave passes `cargo publish --dry-run` in dependency order, or failures are
      recorded with owning follow-up items.
- [ ] `talos-runtime` remains the documented SDK facade; implementation crates document direct-use
      caveats.
- [ ] Heavy optional capabilities have feature gates or recorded split triggers.
- [ ] README, README.zh-CN, and architecture docs explain crate distribution when the first
      implementation slice lands.

## Validation

- `cargo metadata --no-deps --format-version 1`
- `cargo publish --dry-run -p <crate>` for selected first-wave crates
- `cargo test -p <crate>` for each selected crate
- `cargo check --workspace`
- `scripts/validate_project_governance.sh .`

## Required Reads

- `docs/proposals/talos-crate-distribution-architecture.md`
- `docs/reference/ARCHITECTURE.md`
- `docs/backlog/active/RUNTIME-001-embeddable-agent-runtime-api.md`
- `docs/backlog/active/TOOL-012-tool-family-progressive-loading.md`
- `docs/backlog/active/TOOL-013-multi-resource-tool-permissions.md`
- `docs/backlog/active/DIST-001-optional-runtime-asset-distribution.md`
- `docs/backlog/active/REL-002-v1-self-bootstrap-release-gate.md`
- `docs/decisions/024-embeddable-runtime-api-boundary.md`
- `docs/decisions/025-ripgrep-library-search-engine.md`
- `Cargo.toml`
- `crates/*/Cargo.toml`

## Open Questions

1. Should the first real publish happen before or after the 1.0 self-bootstrap gate?
2. Which crate names should be reserved on crates.io before APIs are fully stable?
3. Should `talos-tui` be a reusable UI library package or product-only implementation detail?
4. Should post-1.0 crates move to independent versions, or stay lockstep for user simplicity?
