# CAP-001-P0 Compatibility Seam Handoff

This reference records the bounded preparation delivered by I246/#467. It does not accept the
CAP-001 provider architecture or authorize Desktop implementation.

## Continuity with CAP-001 / #466

Issue #467 is the preparatory prerequisite for the parent CAP-001 architecture in #466; it is
not a replacement, reduction, or completion of that Epic. The seam and compatibility evidence
produced here are inputs to separately governed CAP-001-A/B/C, Bundle, Language, Distribution,
and Browser children. Those children retain ownership of Capability/Provider contracts,
registry/resolution, installation/activation, dynamic providers, and final public terminology.

## Stable boundaries

`talos-text` owns renderer-neutral `LanguageId`, `HighlightSpan`, `HighlightResult`,
`SourceLocation`, and `SymbolInfo` values. These types contain no Arborium, Tree-sitter, TUI,
Crossterm, or GPUI values. Arborium is owned by the built-in adapter in `talos-text`; TUI and
symbol consumers call that seam without direct Arborium dependencies. No second consumer parser
registry is introduced. Static language bundling is temporary compatibility behavior permitted
by #467, not the final dynamic-provider or binary-size outcome required by #466.

| Concern | Current owner | Compatibility rule |
|---|---|---|
| Language aliases | `talos-text` contract and extension map | Canonical names and existing aliases remain accepted |
| Highlight rendering | `talos-text` built-in highlighter / TUI renderer | Return neutral spans, then apply TUI colors |
| Symbol traversal | `talos-text` source-only symbol operations | Tools retain file access and traversal; neutral locations/results cross the seam |
| Plugin package | `talos-plugin` manifest | Persisted fields and carrier names are unchanged |
| Plugin runtime | `talos-plugin` registry/WASM adapter | Package selection and execution remain separate responsibilities |

## Guards

- `talos-text` has no UI dependency; its optional `code-intelligence` feature owns Arborium.
- Highlighting uses a 500ms post-operation soft budget, not a hard interruption guarantee.
  Symbol parsing has progress-callback and AST depth/work guards; neither is a process-isolation
  boundary. Future dynamic-provider isolation remains separate #466 work.
- TUI and tools may consume the neutral contract but do not depend on each other.
- No Desktop, GPUI, browser, bundle installer, provider resolver, permission, session, release, or
  publication behavior is part of this handoff.
- Unsupported or unavailable highlighting remains a plain-text/failure fallback.

Validation checkpoint: `./scripts/release_preflight.sh` passed on local implementation head
`fbad20ee`, including workspace checks, Clippy, tests/doctests and both governance validators.
This is local evidence, not remote exact-head CI or a completion claim.
