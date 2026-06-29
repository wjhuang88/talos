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
| 1 | `talos-core` | Foundation protocol | Published first wave | `talos-core 0.2.0` published | Continue API docs before 1.0 stability claims. |
| 2 | `talos-config` | Policy/config | Published first wave | `talos-config 0.2.0` published | Add crate-specific docs/keywords later. |
| 3 | `talos-permission` | Policy/safety | Published first wave | `talos-permission 0.2.0` published | Add safety support boundary docs. |
| 4 | `talos-skill` | Capability/parser | Published first wave | `talos-skill 0.2.0` published | Add crate-specific docs/keywords later. |
| 5 | `talos-session` | Storage/session | Published first wave | `talos-session 0.2.0` published | Add SQLite storage contract docs. |
| 6 | `talos-provider` | Provider/network | Publish-after-docs | `cargo publish --dry-run -p talos-provider` passed | Document network/provider support boundary before real publish. |
| 7 | `talos-sandbox` | Platform safety | Publish-after-ADR-review | Manifest-ready; platform behavior sensitive | Safety review before dry-run. |
| 8 | `talos-plugin` | Extension foundation | Published second wave | `talos-plugin 0.2.0` published | Continue extension boundary docs before 1.0 stability claims. |
| 9 | `talos-tools` | Built-in tools | Publish-after-feature-gates | Manifest-ready; heavy defaults | Design feature gates. |
| 10 | `talos-memory` | Memory storage | Published second wave | `talos-memory 0.2.0` published | Add fuller SQLite storage contract docs before 1.0 stability claims. |
| 11 | `talos-exploration` | Exploration storage | Published second wave | `talos-exploration 0.2.0` published | Add fuller SQLite/FTS storage contract docs before 1.0 stability claims. |
| 12 | `talos-conversation` | UI/runtime state | Publish-after-docs | `cargo publish --dry-run -p talos-conversation` passed | Document alternate UI/state API before real publish. |
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

Remaining manifest work before broad publish:

- Decide whether each crate should use a shared README, crate-specific README, or docs.rs-only docs.
- Add crate-specific `keywords` and `categories` once first-wave crate docs are final.
- Add `publish = false` to crates intentionally kept product-only if the team does not want their
  names reserved.
- Add feature gates for heavy tool/UI/provider capabilities before broad public consumption.

## Name Reservation Plan

Recommended reservation sequence if the maintainer explicitly authorizes real publish:

1. Completed first-wave reservation with real usable crates: `talos-core`, `talos-skill`,
   `talos-config`, `talos-permission`, and `talos-session`.
2. Completed second-wave reservation for `talos-plugin`, `talos-memory`, and
   `talos-exploration`.
3. Document support boundaries before publishing `talos-provider`, `talos-conversation`,
   `talos-rpc`, or other protocol/runtime surfaces.
4. Keep `talos-runtime` reserved for the SDK facade, but publish it only after its implementation
   dependencies are intentionally published or decoupled.
5. Do not plan around the `talos` package name; it is already taken by an unrelated crate.
6. Defer remaining P1/P2/P3 names until docs, feature gates, and API support boundaries are
   clearer.

Do not publish empty placeholder crates. Each reservation package should compile, include a clear
description, and state its pre-1.0 support boundary.
