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
| `talos-cli` | No exact match | P3 | Binary/product package, not a library dependency target. |

## Publication Matrix

| Order | Crate | Layer | Support level | Publish readiness | First action |
|---:|---|---|---|---|---|
| 1 | `talos-core` | Foundation protocol | Publish-now candidate | `cargo publish --dry-run --allow-dirty -p talos-core` passed | Real publish/name reservation requires maintainer approval. |
| 2 | `talos-config` | Policy/config | Publish-after-core | Manifest-ready; dry-run blocked until `talos-core` exists in crates.io index | Dry-run after core is published/reserved. |
| 3 | `talos-permission` | Policy/safety | Publish-after-core-docs | Manifest-ready; dry-run blocked until `talos-core` exists in crates.io index | Dry-run after core is published/reserved. |
| 4 | `talos-skill` | Capability/parser | Publish-now candidate | `cargo publish --dry-run --allow-dirty -p talos-skill` passed | Real publish/name reservation requires maintainer approval. |
| 5 | `talos-session` | Storage/session | Publish-after-core-docs | Manifest-ready; dry-run blocked until `talos-core` exists in crates.io index | Dry-run after core is published/reserved. |
| 6 | `talos-provider` | Provider/network | Publish-after-docs | Manifest-ready; network/provider API docs needed | Document support boundary. |
| 7 | `talos-sandbox` | Platform safety | Publish-after-ADR-review | Manifest-ready; platform behavior sensitive | Safety review before dry-run. |
| 8 | `talos-plugin` | Extension foundation | Publish-after-boundary-docs | Manifest-ready; depends on core + permission | Document extension support boundary. |
| 9 | `talos-tools` | Built-in tools | Publish-after-feature-gates | Manifest-ready; heavy defaults | Design feature gates. |
| 10 | `talos-memory` | Memory storage | Publish-after-docs | Manifest-ready; SQLite bundled behavior needs docs | Document storage contract. |
| 11 | `talos-exploration` | Exploration storage | Publish-after-docs | Manifest-ready; SQLite/FTS behavior needs docs | Document storage contract. |
| 12 | `talos-conversation` | UI/runtime state | Publish-after-docs | Manifest-ready; alternate UI contract needed | Document state API. |
| 13 | `talos-agent` | Runtime implementation | Advanced/transitional | Manifest-ready; not primary SDK | Publish after lower deps. |
| 14 | `talos-runtime` | SDK facade | Primary SDK | Manifest-ready; depends on lower deps | Publish after implementation deps. |
| 15 | `talos-mcp` | Protocol transport | Publish-after-ADR | Manifest-ready; protocol boundary sensitive | ADR/support boundary. |
| 16 | `talos-rpc` | JSON-RPC transport | Publish-after-ADR | Manifest-ready; remote/control semantics sensitive | ADR/support boundary. |
| 17 | `talos-evolution` | Product learning | Product-specific | Manifest-ready but not first-wave | Defer. |
| 18 | `talos-tui` | Product UI | Product assembly | Manifest-ready but heavy/UI-specific | Defer. |
| 19 | `talos-cli` | Binary product | Product assembly | Manifest-ready as package; not first-wave library | Defer until binary publish decision. |

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

Remaining manifest work before broad publish:

- Decide whether each crate should use a shared README, crate-specific README, or docs.rs-only docs.
- Add crate-specific `keywords` and `categories` once first-wave crate docs are final.
- Add `publish = false` to crates intentionally kept product-only if the team does not want their
  names reserved.
- Add feature gates for heavy tool/UI/provider capabilities before broad public consumption.

## Name Reservation Plan

Recommended reservation sequence if the maintainer explicitly authorizes real publish:

1. Reserve P0 names first with real minimal, usable crates: `talos-core`, `talos-runtime`.
2. Reserve P1 names in dependency order: `talos-skill`, `talos-config`, `talos-permission`,
   `talos-session`, `talos-agent`.
3. Do not plan around the `talos` package name; it is already taken by an unrelated crate.
4. Defer P2/P3 names until docs, feature gates, and API support boundaries are clearer.

Do not publish empty placeholder crates. Each reservation package should compile, include a clear
description, and state its pre-1.0 support boundary.
