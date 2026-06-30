# Talos Crate Publication Matrix

Created: 2026-06-29

This matrix tracks crates.io publish readiness for Talos workspace crates. It is a readiness and
release-gate artifact, not authorization to publish. Real `cargo publish` remains blocked until the
maintainer explicitly approves publishing or name-reservation packages.

## Policy

- Use lockstep workspace version `0.2.0` during pre-1.0.
- Internal dependencies must include both `version = "0.2.0"` and `path = "../..."`.
- `talos-runtime` remains the primary SDK facade.
- Product assembly crates are not required dependencies for embedders.
- `cargo install` support is a binary distribution surface, not a library API promise. Because the
  top-level `talos` package name is unavailable, the planned Cargo install shape is
  `cargo install talos-cli --bin talos` unless a later release decision chooses another package
  name.
- Heavy/default-weight capability crates need feature-gate review before broad publication.
- Name reservation means real crates.io publication. Do not reserve by publishing placeholder
  packages unless the maintainer explicitly approves the package list and version.

## Name Availability Snapshot

Checked with `cargo search <name> --limit 3` on 2026-06-29.

| Crate name | Exact search result | Reservation priority | Notes |
|---|---:|---|---|
| `talos` | Taken | N/A | Existing `talos = "0.1.0"` is unrelated; top-level Cargo package name is unavailable. |
| `talos-core` | No exact match | P0 | Search returned `talos-core-rs`, not exact name. |
| `talos-runtime` | No exact match | P0 | Primary SDK facade name. |
| `talos-agent` | No exact match | P1 | Implementation surface; useful to reserve with SDK wave. |
| `talos-config` | No exact match | P1 | Low-risk standalone config crate. |
| `talos-permission` | No exact match | P1 | Safety policy crate; public API must be documented carefully. |
| `talos-skill` | No exact match | P1 | Low-risk parser/loader crate. |
| `talos-session` | No exact match | P1 | Storage-facing crate; SQLite behavior must be documented. |
| `talos-provider` | No exact match | P2 | Network/provider APIs need support boundary docs. |
| `talos-sandbox` | No exact match | P2 | Platform-sensitive; requires safety review before publish. |
| `talos-plugin` | No exact match | P2 | Protocol/extension boundary needs stability notes. |
| `talos-tools` | No exact match | P2 | Heavy default dependencies; feature-gate review first. |
| `talos-memory` | No exact match | P2 | SQLite-backed; publish after storage API docs. |
| `talos-exploration` | No exact match | P2 | SQLite/FTS-backed; publish after API docs. |
| `talos-conversation` | No exact match | P2 | Useful for alternate UIs after state API docs. |
| `talos-mcp` | No exact match | P3 | Protocol dependency; publish after MCP boundary review. |
| `talos-rpc` | No exact match | P3 | Transport surface; publish after remote/control boundary review. |
| `talos-evolution` | No exact match | P3 | Product-specific until external use case is proven. |
| `talos-tui` | No exact match | P3 | Product/UI surface; likely not first-wave library crate. |
| `talos-cli` | No exact match | P2 | Binary/product package candidate for `cargo install talos-cli --bin talos`; library API remains unsupported. |

## Publication Matrix

| Order | Crate | Layer | Support level | Publish readiness | First action |
|---:|---|---|---|---|---|
| 1 | `talos-core` | Foundation protocol | Published first wave | `talos-core 0.2.0` published | Continue API docs before 1.0 stability claims. |
| 2 | `talos-config` | Policy/config | Published first wave | `talos-config 0.2.0` published | Add crate-specific docs/keywords later. |
| 3 | `talos-permission` | Policy/safety | Published first wave | `talos-permission 0.2.0` published | Add safety support boundary docs. |
| 4 | `talos-skill` | Capability/parser | Published first wave | `talos-skill 0.2.0` published | Add crate-specific docs/keywords later. |
| 5 | `talos-session` | Storage/session | Published first wave | `talos-session 0.2.0` published | Add SQLite storage contract docs. |
| 6 | `talos-provider` | Provider/network | Published integration wave | `talos-provider 0.2.0` published | Continue provider support boundary docs before 1.0 stability claims. |
| 7 | `talos-sandbox` | Platform safety | Gate-before-publish | Manifest-ready; platform behavior sensitive | Complete sandbox safety gate before dry-run or real publish. |
| 8 | `talos-plugin` | Extension foundation | Published second wave | `talos-plugin 0.2.0` published | Continue extension boundary docs before 1.0 stability claims. |
| 9 | `talos-tools` | Built-in tools | Gate-before-publish | Manifest-ready; heavy defaults and tool execution surface | Complete feature/permission gate before dry-run or real publish. |
| 10 | `talos-memory` | Memory storage | Published second wave | `talos-memory 0.2.0` published | Add fuller SQLite storage contract docs before 1.0 stability claims. |
| 11 | `talos-exploration` | Exploration storage | Published second wave | `talos-exploration 0.2.0` published | Add fuller SQLite/FTS storage contract docs before 1.0 stability claims. |
| 12 | `talos-conversation` | UI/runtime state | Published integration wave | `talos-conversation 0.2.0` published | Continue alternate UI/state API docs before 1.0 stability claims. |
| 13 | `talos-agent` | Runtime implementation | Gate-before-publish | Manifest-ready; not primary SDK | Publish only after sandbox/tools dependency gates or decoupling. |
| 14 | `talos-runtime` | SDK facade | Gate-before-publish | Manifest-ready; depends on lower deps | Publish after dependency closure is safe or runtime is decoupled. |
| 15 | `talos-mcp` | Protocol transport | Gate-before-publish | Manifest-ready; protocol boundary sensitive | ADR/support boundary before dry-run or real publish. |
| 16 | `talos-rpc` | JSON-RPC transport | Published integration wave | `talos-rpc 0.2.0` published | Keep support boundary to local stdio; remote semantics still need ADR. |
| 17 | `talos-evolution` | Product learning | Product-only | `publish = false` | Do not publish until external reusable API is proven. |
| 18 | `talos-tui` | Product UI | Product-only | `publish = false` | Do not publish unless Talos intentionally offers a reusable TUI library. |
| 19 | `talos-cli` | Binary product | Cargo-install candidate | `publish = false` today; requires binary package gate | Design and validate `cargo install talos-cli --bin talos`; do not expose or promise a stable library API. |

## Current Manifest Readiness

- Workspace package metadata now includes `repository` and `homepage`.
- Workspace crates inherit `repository.workspace = true` and `homepage.workspace = true`.
- Talos crate-to-crate dependencies now include `version = "0.2.0"` plus `path`.

## Dry-Run Evidence

2026-06-29:

- `cargo package --allow-dirty --list -p talos-core` succeeded.
- `cargo package --allow-dirty --list -p talos-skill` succeeded.
- `cargo publish --dry-run --allow-dirty -p talos-core` succeeded.
- `cargo publish --dry-run --allow-dirty -p talos-skill` succeeded.
- `cargo publish --dry-run --allow-dirty -p talos-config` failed because `talos-core` is not yet
  in the crates.io index.
- `cargo publish --dry-run --allow-dirty -p talos-permission` failed because `talos-core` is not
  yet in the crates.io index.
- `cargo publish --dry-run --allow-dirty -p talos-session` failed because `talos-core` is not yet
  in the crates.io index.
- Real `cargo publish -p talos-core` was attempted from clean commit `30c9abc` after maintainer
  approval. crates.io rejected the upload because the publisher account does not have a verified
  email address. No crate was published and no name was reserved.
- After email verification, real publishes from clean commit `c8884f6` succeeded:
  `talos-core 0.2.0`, `talos-skill 0.2.0`, `talos-config 0.2.0`,
  `talos-permission 0.2.0`, and `talos-session 0.2.0`.
- `cargo search talos-core --limit 5` confirmed `talos-core = "0.2.0"` was visible in the
  crates.io index before publishing `talos-config`, `talos-permission`, and `talos-session`.
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

Remaining manifest work before broad publish:

- Decide whether each crate should use a shared README, crate-specific README, or docs.rs-only docs.
- Add crate-specific `keywords` and `categories` once first-wave crate docs are final.
- Add feature gates for heavy tool/UI/provider capabilities before broad public consumption.
- Keep `publish = false` on product-only crates unless a future story changes their distribution
  model. `talos-cli` is now a binary package candidate, so removing its guard requires a dedicated
  install-package gate rather than a reusable-library gate.

## Remaining Publication Gates

These crates are intentionally not published yet:

| Crate | Current decision | Required gate before publish |
|---|---|---|
| `talos-sandbox` | Candidate, high risk | Security review against escape vectors, platform behavior docs, ADR-007/ADR-008/ADR-020 dependency boundary check, targeted sandbox tests. |
| `talos-tools` | Candidate, high risk | Feature-gate plan for heavy/default tools, permission profile audit, network/write/execute tool boundary docs, dry-run after `talos-sandbox` decision. |
| `talos-agent` | Candidate, transitional | Decide whether public consumers should depend on `talos-agent` directly or only through `talos-runtime`; publish only after sandbox/tools/memory/session dependency support is clear. |
| `talos-runtime` | Candidate, SDK facade | Resolve dependency closure: either publish required implementation crates or decouple runtime from unpublished implementation surfaces; document pre-1.0 SDK support contract. |
| `talos-mcp` | Candidate, protocol sensitive | MCP support boundary ADR or equivalent doc, server opt-in/conflict policy, transport/auth non-goals, dry-run after `talos-tools` decision. |
| `talos-evolution` | Product-only now | Prove an external reusable API; remove `publish = false` only through a new story/decision. |
| `talos-tui` | Product-only now | Decide to offer reusable TUI library; otherwise keep product UI out of crates.io. |
| `talos-cli` | Binary package candidate | Add package metadata/readme for crates.io, ensure the `talos` bin target is included, verify `cargo install --path crates/talos-cli --bin talos` and `cargo publish --dry-run -p talos-cli`, document that only the binary is supported, then remove `publish = false` through an explicit release gate. |

## Name Reservation Plan

Recommended reservation sequence if the maintainer explicitly authorizes real publish:

1. Completed first-wave reservation with real usable crates: `talos-core`, `talos-skill`,
   `talos-config`, `talos-permission`, and `talos-session`.
2. Completed second-wave reservation for `talos-plugin`, `talos-memory`, and
   `talos-exploration`.
3. Completed integration-wave reservation for `talos-provider`, `talos-conversation`, and
   `talos-rpc` after adding crate-level support boundary docs.
4. Keep `talos-runtime` reserved for the SDK facade, but publish it only after its implementation
   dependencies are intentionally published or decoupled.
5. Reserve `talos-cli` for the CLI binary package only after the install-package gate passes;
   install docs should use `cargo install talos-cli --bin talos` unless a later decision chooses a
   different package name.
6. Keep `talos-tui` and `talos-evolution` product-only with `publish = false`.
7. Do not plan around the `talos` package name; it is already taken by an unrelated crate.
8. Defer remaining high-risk names until docs, feature gates, and API support boundaries are
   complete.

Do not publish empty placeholder crates. Each reservation package should compile, include a clear
description, and state its pre-1.0 support boundary.

## A1: Published-Crate Docs Audit (2026-06-29)

All 11 published crates have `description` and workspace-inherited `repository`/`homepage`. None
have `keywords`, `categories`, `readme`, or `documentation` fields. Crate-level `//!` docs exist
for `talos-permission` (comprehensive), `talos-provider`/`talos-conversation`/`talos-rpc` (support
boundary), `talos-core`/`talos-plugin` (minimal). Missing for `talos-config`, `talos-skill`,
`talos-session`, `talos-memory`, `talos-exploration`.

## A2: Product-Only Guard (2026-06-29)

`scripts/check_publish_guard.sh` verifies `publish = false` on product-only crates
(`talos-cli`, `talos-tui`, `talos-evolution`) and its absence on gate crates
(`talos-sandbox`, `talos-tools`, `talos-agent`, `talos-runtime`, `talos-mcp`). All checks pass.

## A3-A6: Gate Analysis (2026-06-29)

### A3: `talos-sandbox` — DO NOT PUBLISH

Dependencies: `talos-core` (published), `libc 1.0.0-alpha.3` (pre-release, ADR-007), `tokio`.
Escape-vector checklist (7 items) must be verified with targeted tests before publish. The
`libc` pre-release version is a stability risk.

### A4: `talos-tools` — DO NOT PUBLISH (heaviest crate)

Dependencies include `gix` (~5MB), `arborium` 25+ langs (~30MB), `reqwest`, `scraper`. Feature
gates needed: `default = ["file", "search", "git", "code-intelligence", "network"]` with optional
 shedding. Permission profiles verified TOOL-013 compliant. Publish after sandbox gate + feature
gates implemented.

### A5: `talos-agent`/`talos-runtime` — Publish Closure Path

Decision: publish implementation crates in dependency order (sandbox → tools → agent → runtime).
`talos-agent` remains implementation surface (embedders use `talos-runtime`). `talos-runtime` SDK
support contract: `RuntimeBuilder`/`RuntimeHandle` are the stable pre-1.0 surface.

### A6: `talos-mcp` — DO NOT PUBLISH until rmcp evaluated

Transport: local stdio only (no TCP/HTTP). Support boundary: server opt-in, tool conflict policy
(built-in takes precedence), bounded timeout. `rmcp 1.7.0` stability must be evaluated. No
`~/.agents/mcp.json` import (gated under AGENT-002-C).
