# ARCH-031-A: `talos-tools` Lightweight Default And Real Feature Boundaries

| Field | Value |
|---|---|
| Story ID | ARCH-031-A |
| Type | Architecture / Cargo Boundary Story |
| Parent Epic | ARCH-031 |
| Priority | P1 after I158 |
| Status | Refinement — blocked on I158 |
| Depends on | ADR-052; ADR-053 Accepted; ARCH-034-R01/I158 Complete |
| Selected Iteration | I159 (Planned/Blocked) |
| User/maintainer value | Embedders can use a lightweight local read-only tool surface without compiling unrelated heavy capability families |

## Problem

`talos-tools` currently exposes a broad collection of file, search, shell, Git, network/web, image,
and code-intelligence capabilities from one crate. Heavy dependencies remain hard dependencies and
there are no real Cargo feature gates.

Hiding tool registration is insufficient: a lightweight build must not resolve or compile the
dependencies, modules, or re-exports for disabled capability families.

ADR-052 decides the direction but does not implement it.

## Goal

Implement real Cargo feature boundaries with a lightweight local read-only default while preserving
Talos CLI behavior through explicit feature selection.

## Current-State Baseline

Before implementation, the developer must record:

- the current `[dependencies]` and absence/presence of `[features]` in
  `crates/talos-tools/Cargo.toml`;
- all public modules/re-exports in `crates/talos-tools/src/lib.rs`;
- all CLI/runtime/agent dependencies on concrete tool types;
- current default CLI tool inventory;
- current `cargo tree -p talos-tools` output.

Do not trust this Story's candidate names if current code has changed. Record any mismatch before editing.

## Required Feature Model

Use these stable public feature groups unless the baseline proves a concrete incompatibility. Any
rename requires updating this Story before implementation.

```toml
[features]
default = ["file-read", "search"]

file-read = []
file-write = []
search = []
shell = []
git = []
network = []
image = []
code-intelligence = []

coding = [
  "file-read",
  "file-write",
  "search",
  "shell",
  "git",
  "network",
  "image",
  "code-intelligence",
]
```

Rules:

- `coding` is a convenience aggregate, not a permission grant.
- Feature names describe compile-time capability availability only.
- Permission rules still decide runtime use.
- `file-read` and `search` are the only default families.
- A finer split is allowed only when current dependency ownership makes the above impossible; update
  the Story and record why before coding.

## Scope

### Cargo manifest

- Add the feature model.
- Convert family-specific dependencies to `optional = true`.
- Link optional dependencies through `dep:<name>` feature entries where required.
- Keep dependencies needed by default file-read/search behavior non-optional only when justified.
- Preserve target-specific dependency sections.
- Do not introduce new third-party dependencies.

### Modules and exports

- Gate family modules with `#[cfg(feature = "...")]`.
- Gate every public re-export of disabled family types.
- Gate tests and examples consistently.
- Ensure disabled modules are not parsed or compiled.
- Keep shared protocol/utility types available only when they do not pull heavy dependencies.

### Downstream crates

- `talos-cli` must explicitly enable the feature set needed for the current product experience,
  normally `features = ["coding"]`.
- `talos-agent`, `talos-runtime`, examples, and tests must request only the capability groups they use.
- Do not change the product tool inventory in this Story.
- Do not implement `RuntimePreset::coding()` in this Story.

### Documentation

- Update crate-level docs with:
  - default surface;
  - feature table;
  - compile-time capability versus runtime permission distinction;
  - migration note for users that relied on broad implicit defaults.
- Update publication matrix and runtime SDK planned/current descriptions only after implementation.

## Explicit Exclusions

- no new composition crate;
- no tool registration redesign beyond using the I158 result;
- no permission-default changes;
- no sandbox fallback API;
- no runtime preset;
- no tool behavior changes;
- no splitting `talos-tools` into sibling crates;
- no real crate publication;
- no workspace version bump.

## Expected Change Sites

| Path | Expected change | Must preserve |
|---|---|---|
| `crates/talos-tools/Cargo.toml` | features and optional dependencies | package metadata and target behavior |
| `crates/talos-tools/src/lib.rs` | module/re-export cfg gates | documented current public boundaries |
| family modules/tests | cfg-compatible imports/tests | existing behavior when enabled |
| `crates/talos-cli/Cargo.toml` | explicit coding feature | current CLI capability inventory |
| downstream manifests | minimal explicit features | dependency direction |
| crate/reference docs | feature documentation | ADR-052 security wording |

## Invariants

- disabling a feature removes the related dependency from the resolved graph when no other enabled
  family needs it;
- `coding` does not bypass permission or sandbox policy;
- current CLI behavior is unchanged;
- default `talos-tools` users receive file-read + search only;
- no disabled public type remains re-exported;
- no `cfg` combination panics at compile time or creates contradictory APIs.

## Acceptance

### Structural

- [ ] `talos-tools` has a documented `[features]` section.
- [ ] default features are exactly the approved lightweight read-only surface.
- [ ] each heavy family dependency is truly optional where technically attributable.
- [ ] corresponding modules and re-exports are feature-gated.
- [ ] `cargo tree` proves disabled heavy dependencies are absent from a default-only build.
- [ ] `talos-cli` explicitly selects the full product feature set.
- [ ] no product tool disappears from CLI/TUI/MCP when built with the product feature set.

### Build matrix

At minimum:

```bash
cargo check --locked -p talos-tools --no-default-features
cargo check --locked -p talos-tools
cargo check --locked -p talos-tools --no-default-features --features file-read
cargo check --locked -p talos-tools --no-default-features --features search
cargo check --locked -p talos-tools --no-default-features --features file-read,search
cargo check --locked -p talos-tools --no-default-features --features coding
cargo test --locked -p talos-tools --no-default-features --features coding
```

Add family-specific combinations for actual dependency seams discovered during baseline.

### Workspace and runtime

- [ ] full locked validation passes.
- [ ] real `talos` CLI/TUI smoke proves the product tool inventory is unchanged.
- [ ] a default-only external/minimal example compiles without heavy capability families.
- [ ] crate docs and migration note are updated.
- [ ] Story, I159, ARCH-031, matrix, and Board state are synchronized.

## Stop And Escalate Conditions

Stop and record a blocker if:

- a default read/search module depends directly on a heavy family in a way that requires behavior change;
- feature gating requires a public API break not covered by ADR-052;
- a new crate or third-party dependency appears necessary;
- the current tool registration model still has duplicated lists because I158 was not actually complete;
- a product mode loses a tool and the reason is not an explicitly documented prior exclusion.

## Required Reads

- `AGENTS.md`
- `docs/tasks/2026-07-26-v0.6-runtime-productization-program.md`
- `docs/decisions/052-sdk-publication-and-composition-boundary.md`
- accepted `docs/decisions/053-tool-registration-composition.md`
- `docs/backlog/active/ARCH-031-crate-publication-boundary.md`
- `docs/backlog/active/ARCH-034-R01-tool-registration-composition.md`
- `docs/reference/CRATE-PUBLICATION-MATRIX.md`
- `docs/reference/RUNTIME-SDK-CONTRACT.md`
- `crates/talos-tools/Cargo.toml`
- `crates/talos-tools/src/lib.rs`
- downstream `Cargo.toml` files using `talos-tools`

## Residual Destination

- independent sibling tool crates: new proposal/ADR;
- capability changes: separate product Story;
- runtime preset and sandbox fallback: ARCH-031-C;
- version bump and publication: ARCH-031-D.
