# ARCH-031-A: `talos-tools` Lightweight Default And Real Feature Boundaries

| Field | Value |
|---|---|
| Story ID | ARCH-031-A |
| Type | Architecture / Cargo Boundary Story |
| Parent Epic | ARCH-031 |
| Priority | P1 after I158 |
| Status | Ready — TUI-037 disposition complete; feature ownership decisions resolved through the 2026-08-14 dependency correction |
| Depends on | ADR-052; ADR-053 Accepted; ARCH-034-R01/I158 Complete; TUI-037 disposition |
| Selected Iteration | I159 (Planned / proposed claim) |
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

## Feature Ownership Matrix

Based on `crates/talos-tools/src/lib.rs` and `crates/talos-tools/Cargo.toml` (verified 2026-07-26).
The planning-time ownership alternatives were resolved on 2026-08-14 after the TUI-037 dependency
closed and the current module/contribution graph was re-read. This section documents ownership; it
does NOT implement any feature.

| Module / Public Export | Proposed Feature | Optional Dependencies | Default | Notes |
|---|---|---|---|---|
| `file_tools` (`read`, `ls`) | `file-read` | `walkdir` (`ls` recursive traversal) | on | Local read-only file access. |
| `file_tools` (`write`, `edit`, `delete`) | `file-write` | (none beyond core) | off | mutating; permission-gated |
| `search_tools` (`glob`, `grep`) | `search` | `grep-searcher`, `grep-regex`, `grep-matcher`, `ignore`, `walkdir`, `regex`, `glob` | on | ripgrep library crates (ADR-025) |
| `tree` | `file-read` | (no additional dependency) | on | Uses its own `std::fs::read_dir` traversal and the shared read-only file boundary. |
| `diff_stat` (`DiffTool`, `StatTool`) | `git` | `similar` | off | Keep this non-mutating comparison/stat family out of the lightweight default and in the current coding aggregate; a future standalone `diff` feature requires separate change control. |
| `bash_tool` | `shell` | `libc` (unix), `talos-sandbox` | off | process/sandbox; security-sensitive |
| `exec_tool` | `shell` | `talos-sandbox` | off | shares the `shell` feature with `bash_tool` |
| `git` | `git` | `gix` | off | heavy native-ish dep; write tools route through permission |
| `fetch_url` | `network` | `reqwest`, `scraper` | off | network + HTML parse |
| `http_request` | `network` | `reqwest`, `scraper` | off | advanced HTTP and HTML text/link extraction; shares `network` with `fetch_url` |
| `save_url` | `network` (+ `file-write`) | `reqwest` | off | dual network+write; both features required |
| `web_search` | `network` | `rust-websearch` | off | Network capability; no second web-search feature in this slice. |
| `search_engine` | `search` | `grep-searcher`, `grep-regex`, `ignore`, `regex`, `glob`, `walkdir` | on | Local ripgrep/legacy search implementations; unrelated to `web_search`. |
| `browser_page` | `network` | `reqwest` (`Url`) | off | Its connector contract represents network-origin page data even when fixtures are local. |
| `document_extract` | `document` (+ `file-read`) | `scraper` | off | Local bounded text/HTML/JSON/CSV/MD/XML extraction. The whole tool is separately gated so the HTML parser stack is absent from the lightweight default. |
| `symbol` | `code-intelligence` | `arborium` (25 langs) | off | heaviest dep; tree-sitter via arborium (ADR-020) |
| `read_image_tool` | `image` | `image`, `sha2` | off | image decode; capability-gated (ADR-050/051); shares `image` with `image_validation` |
| `image_validation` | `image` | `image`, `sha2` | off | shared ingestion/validation; same gate as `read_image_tool` |

Feature combination rule: a tool requiring multiple features (for example `save_url` requiring
`network` + `file-write`) is enabled only when all required features are enabled.

The approved candidate default remains `file-read` + `search`.

`coding` is the only currently proposed aggregate convenience feature in the Required Feature Model
below. `network` is a normal capability feature, not an alias.

`web-search`, `full`, or any additional aggregate/group feature beyond the explicitly approved
`document` capability is not approved by this Story. Such names may appear only as unresolved
alternatives in the ownership matrix above and must be resolved before the Story becomes Ready.

Resolved ownership decisions: `tree` follows `file-read`; `search_engine` follows `search`;
`diff_stat` follows `git`; `web_search` and `browser_page` follow `network`; `document_extract`
uses a default-off `document` feature that also requires `file-read`. Current code confirms that
`document_extract`, `fetch_url`, and `http_request` all compile `scraper`, while `browser_page`
compiles `reqwest::Url`; the feature model must attribute those existing dependencies explicitly.
No new third-party dependency, permission change, or tool behavior change is authorized. The
implementation must still prove every cfg combination and product-parity claim before completion.

## Required Feature Model

Use these stable public feature groups unless the baseline proves a concrete incompatibility. Any
rename requires updating this Story before implementation.

```toml
[features]
default = ["file-read", "search"]

file-read = []
file-write = []
search = []
document = ["file-read", "dep:scraper"]
shell = []
git = []
network = []
image = []
code-intelligence = []

coding = [
  "file-read",
  "file-write",
  "search",
  "document",
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
- `document` is default-off, owns the whole `document_extract` module/export/contribution, requires
  `file-read`, and enables the existing `scraper` dependency. `network` independently enables the
  same dependency for its existing HTML extraction paths.
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

## 2026-08-14 Readiness Checkpoint

- TUI-037/I202 is Complete at implementation commit
  `6d3f85ea9f7e76f617ec9716f17ecdd0f9dd0772` and merge `e0cc782a475c2e5baceb31f2a125f1e268af7ecf`.
- I158 and ADR-053 remain Complete/Accepted target-branch truth.
- The ownership alternatives above are resolved against the current module and contribution graph.
- ARCH-031-A is Ready for I159 claim review. No implementation is active, and no Rust, Cargo,
  version, publish guard or release behavior changes in this readiness update.

## 2026-08-14 Dependency-Fact Change Control

Independent review of PR #235 at `4cd5d6868b42f7efafccf117c78e30173addef01` found that the first
readiness checkpoint incorrectly described `document_extract` as having no additional dependency.
The current module unconditionally calls `scraper::Html` and `scraper::Selector`, and
`crates/talos-tools/Cargo.toml` declares `scraper = "0.27"`. Keeping `document_extract` inside the
default `file-read` family would therefore pull the HTML parser stack into the default closure and
contradict this Story's lightweight-default objective.

The corrected decision adds one stable, default-off `document` capability feature. It gates the
whole `document_extract` module, public exports, tests and contribution, requires `file-read`, and
owns the existing `scraper` dependency for that path. The product `coding` aggregate includes
`document`, preserving the current CLI inventory. This is planning change control only: it does not
gate an HTML subpath, change extraction behavior, add a dependency, or modify Cargo/Rust code.

The same source re-read corrected two non-blocking attribution errors: `tree` uses
`std::fs::read_dir` and follows `file-read`, while `browser_page` directly imports `reqwest::Url`.
`search_engine` is the local search implementation and follows `search`, not `network`. With these
facts and the explicit `document` boundary recorded, all feature-ownership decisions required for
Ready are resolved; implementation evidence remains pending under I159.
