# 028: Plugin Tool Provenance Extension

## Status

Accepted

## Context

ADR-009 introduced `ToolProvenance::{Native, McpRemote}` so TUI/RPC/evolution consumers can
distinguish native and MCP tools. Plugin packages will introduce tools through a plugin runtime
adapter. Without a plugin provenance variant, plugin tools would appear native or MCP-like, hiding
their source and weakening auditability.

## Constraint Decomposition

| Constraint | Type | Source | Can Change? |
| --- | --- | --- | --- |
| Tool provenance must remain typed and auditable. | Hard | ADR-009 / safety model | No |
| Plugin tools must not masquerade as native tools. | Hard | Permission/provenance boundary | No |
| Public APIs are semver-bound. | Hard | AGENTS.md Hard Constraint #6 | No |
| `ToolProvenance` is already `#[non_exhaustive]`. | Known | ADR-009 implementation | No |

## Reasoning

Adding a plugin variant is the intended extension path for ADR-009. The variant should be small:
stable display and filtering need package name, version, and carrier. Runtime-specific details such
as artifact path, hash, permissions, and host-call grants belong to plugin diagnostics, not every
tool-call event.

`carrier` should be a string rather than a closed enum so future carriers can be introduced without
forcing downstream exhaustive updates. Accepted carrier values are still governed by ADR-027.

## Decision

Add a future-compatible plugin provenance shape:

```rust
ToolProvenance::Plugin {
    name: String,
    version: String,
    carrier: String,
}
```

Rules:

- Native built-in tools keep `Native`.
- MCP tools keep `McpRemote`.
- Tools supplied by a plugin package use `Plugin`, even if the plugin internally wraps an MCP
  declaration.
- TUI/RPC/evolution consumers must render unknown future provenance conservatively.
- Plugin provenance is descriptive only; it does not grant permissions.

## Rejected Alternatives

- **Reuse `McpRemote` for plugin MCP declarations.** Rejected because package provenance would be
  lost.
- **Use a string-only provenance field.** Rejected because it regresses ADR-009 type safety.
- **Include artifact paths or hashes in every tool event.** Too noisy for the hot path; expose those
  through plugin diagnostics instead.

## Reversal Trigger

Revisit if plugin package identity needs cryptographic verification in every event rather than in
separate diagnostics.

## Related

- [ADR-009](009-tool-provenance.md)
- [ADR-027](027-plugin-runtime-boundary.md)
- [PLUGIN-001](../backlog/active/PLUGIN-001-wasm-runtime-plugins.md)
