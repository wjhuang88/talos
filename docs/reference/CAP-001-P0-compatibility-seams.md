# CAP-001-P0 Compatibility Seam Handoff

This reference records the bounded preparation delivered by I246/#467. It does not accept the
CAP-001 provider architecture or authorize Desktop implementation.

## Stable boundaries

`talos-text` owns renderer-neutral `LanguageId`, `HighlightSpan`, `HighlightResult`,
`SourceLocation`, and `SymbolInfo` values. These types contain no Arborium, Tree-sitter, TUI,
Crossterm, or GPUI values. Arborium remains an explicitly bounded built-in adapter in the existing
TUI and symbol consumers; its spans are converted at that boundary. No second parser registry is
introduced.

| Concern | Current owner | Compatibility rule |
|---|---|---|
| Language aliases | `talos-text` contract, consumer extension maps | Canonical names and existing aliases remain accepted |
| Highlight rendering | TUI Arborium adapter | Convert to neutral spans, then apply TUI colors |
| Symbol traversal | tools Arborium adapter | Neutral locations/results only; preserve error and limit semantics |
| Plugin package | `talos-plugin` manifest | Persisted fields and carrier names are unchanged |
| Plugin runtime | `talos-plugin` registry/WASM adapter | Package selection and execution remain separate responsibilities |

## Guards

- `talos-text` has no UI or parser dependency.
- TUI and tools may consume the neutral contract but do not depend on each other.
- No Desktop, GPUI, browser, bundle installer, provider resolver, permission, session, release, or
  publication behavior is part of this handoff.
- Unsupported or unavailable highlighting remains a plain-text/failure fallback.

Focused evidence on the implementation branch: `talos-text` (3 tests), TUI (570 tests), tools with
`code-intelligence` enabled (51 tests), and `cargo check --workspace --locked` all pass.
