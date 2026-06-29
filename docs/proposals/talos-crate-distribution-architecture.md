# Talos Crate Distribution Architecture

## Status

Accepted as the implementation baseline on 2026-06-29 for publication-readiness work. A separate
release/ADR gate is still required before any real crates.io publish or placeholder name
reservation.

Created 2026-06-28 from the requirement that Talos-owned capabilities should be distributable as
crates, not only as the `talos` binary or the `talos-runtime` SDK facade.

## Problem

`talos-runtime` gives other Rust projects a safe embeddable agent runtime, but Talos has more
self-written capabilities than the runtime facade itself:

- protocol and trait foundations;
- configuration schema and secret-display boundaries;
- provider implementations;
- permission and sandbox policy;
- built-in tools;
- skill loading;
- session, memory, and exploration stores;
- plugin, MCP, RPC, conversation, and TUI support layers.

If these capabilities remain only internal workspace crates, external users must either depend on
the whole runtime facade or copy implementation details. That weakens architecture boundaries and
makes it harder to detect product-layer coupling. We need every reusable capability crate to be
structured as if it could be published to crates.io, even when we delay the actual publish until the
API is ready.

## Ripgrep Reference

The relevant `rg` pattern is not "publish one binary crate and make everyone depend on it." The
published `ripgrep 15.1.0` package is a CLI-oriented package with an `rg` binary target, while
reusable capabilities live in separately published library crates such as `grep-searcher`,
`grep-regex`, `grep-matcher`, and `ignore`.

Talos should follow the same architectural lesson:

1. Product binaries aggregate capability crates.
2. Reusable libraries are independently documented, versioned, tested, and publishable.
3. Heavy or optional capabilities are feature-gated or split before they become default dependency
   weight.
4. The top-level binary is allowed to be opinionated, but lower-level crates must not inherit CLI
   assumptions.

This proposal does not require Talos to copy ripgrep's exact crate count or versioning policy. It
uses the pattern as a boundary discipline: binary/application code depends on reusable libraries,
not the reverse.

## Proposed Architecture

### Distribution Layers

| Layer | Crates | Publication Intent |
|---|---|---|
| Foundation protocol | `talos-core` | Publish first; minimal dependency root and canonical protocol/trait types. |
| Policy and platform | `talos-config`, `talos-permission`, `talos-sandbox` | Publish as reusable safety/config crates once public APIs and platform behavior are documented. |
| Runtime composition | `talos-runtime` | Public SDK facade; depends on implementation crates without exposing their internals as stable API. |
| Runtime implementation | `talos-agent` | Publishable as an implementation crate, but not the primary SDK entrypoint. Public docs must mark lower-level surfaces as advanced/transitional. |
| Capability crates | `talos-provider`, `talos-tools`, `talos-skill`, `talos-session`, `talos-memory`, `talos-exploration` | Publishable libraries with narrow feature flags and no CLI/TUI dependency. |
| Extension and transport | `talos-plugin`, `talos-mcp`, `talos-rpc`, `talos-conversation` | Publish after protocol boundaries stabilize; useful for external integrations and alternate UIs. |
| Product assembly | `talos-cli`, `talos-tui` | May be published for binary/UI reuse, but they are product surfaces, not required dependencies for embedders. |

### Publishable Crate Contract

A Talos crate is publishable only when it satisfies all of these:

- It has complete package metadata: description, license, repository, readme or docs target,
  categories/keywords where useful, and a clear crate-level doc comment.
- Workspace-internal dependencies use both `path` and `version`, so Cargo can rewrite them during
  publish.
- Public exports are intentionally curated. Implementation modules stay private or explicitly
  marked experimental.
- Feature flags separate optional weight, network access, native/process integration, parser
  bundles, storage backends, and product adapters.
- The crate can be tested independently with `cargo test -p <crate>`.
- External dependency failures degrade through typed errors; native, process, or panic-prone
  boundaries follow the AGENTS.md safety rule.
- No crate below the product layer depends on `talos-cli` or on hidden CLI/TUI behavior.
- README or docs explain the happy path, safety defaults, and which APIs are semver-supported.

### Versioning Policy

Use lockstep workspace versions through the pre-1.0 period. This keeps internal compatibility
simple while APIs are still moving. After 1.0, consider independent crate versions only for crates
that have real external consumers and change at materially different speeds.

The immediate technical change is not independent versioning. It is adding publish-compatible
manifest shape and API documentation so each crate can pass `cargo publish --dry-run` in dependency
order.

### Tool-Crate Direction

`talos-tools` is the most likely crate to need a ripgrep-like split later. It currently aggregates
file, search, code-intelligence, Git, shell, HTTP, save, and web-search tools. The near-term path
should be:

1. Keep `talos-tools` as the stable user-facing built-in tool crate.
2. Add feature flags around heavy families: code intelligence, Git, network/web, and shell.
3. Move reusable engines into internal modules with narrow public APIs.
4. Split into sibling crates only when a family has independent consumers or heavy default weight:
   for example `talos-tool-search`, `talos-tool-git`, or `talos-tool-code`.

This matches ripgrep's lesson without creating premature crate sprawl.

## Implementation Phases

1. **Publication inventory**
   - Add a crate publication matrix covering dependency order, package metadata, public API
     readiness, optional features, and dry-run status.
   - Classify crates as `publish-now`, `publish-after-docs`, `publish-after-ADR`, or
     `product-only`.

2. **Manifest normalization**
   - Add workspace-level package metadata where possible.
   - Add `version.workspace = true` plus `path` dependencies for all Talos crate-to-crate
     dependencies.
   - Add crate readmes or docs links where needed.

3. **Foundation dry-runs**
   - Start with `talos-core`, then low-risk libraries such as `talos-config`,
     `talos-permission`, `talos-skill`, and `talos-session`.
   - Run `cargo publish --dry-run -p <crate>` in dependency order.

4. **Runtime and capability hardening**
   - Validate `talos-runtime` as the public SDK facade.
   - Document `talos-agent` as implementation surface.
   - Gate optional tool/provider/storage capabilities through features.

5. **Product package decision**
   - Decide whether crates.io should publish only libraries plus a `talos-cli` binary package, or
     also publish `talos-tui` as a reusable UI library.
   - Keep release archives and install scripts as separate distribution channels from library
     crates.

## Acceptance Criteria

- A publication matrix exists and covers every workspace crate.
- All publishable crates have complete metadata and publish-compatible internal dependency specs.
- `talos-runtime` remains the documented SDK entrypoint; lower-level crates document their intended
  support level.
- `cargo publish --dry-run` passes for the selected first wave in dependency order.
- Heavy/default-weight capabilities have feature-gating or a recorded split plan.
- README and architecture docs explain crate distribution alongside binary installation.
- A release/ADR gate exists before the first real crates.io publish.

## Alternatives Considered

- **Publish only `talos-runtime`.** Too narrow. It hides useful standalone capabilities and does not
  pressure lower-level architecture boundaries.
- **Publish every crate immediately.** Too risky. Current public APIs were not all designed as
  external semver surfaces, and manifests are not yet publish-ready.
- **Create many tiny tool crates now.** Premature. Start with feature gates and split only when
  consumer demand or dependency weight proves the need.
- **Use a monolithic `talos` library crate.** Rejected because it would recreate product coupling
  and make optional capabilities hard to reason about.

## Open Questions

- Which crate names on crates.io are available and should be reserved early?
- Should `talos-tui` be treated as reusable UI infrastructure or product-only UI?
- Which crates are allowed to publish before the 1.0 self-bootstrap gate?
- Should public package docs use one shared root README or one crate-specific README per crate?
- Which optional features should be default-on for developer convenience versus default-off for
  lean embedders?

## Dependencies

- `RUNTIME-001` and ADR-024 for the SDK facade boundary.
- ADR-025 and `TOOL-011` for ripgrep-library search implementation direction.
- `TOOL-012` and `TOOL-013` for tool-family and permission-boundary cleanup.
- `DIST-001` for large optional asset distribution, which remains separate from crates.io library
  publishing.
- `REL-002` for the 1.0 self-bootstrap release gate.
