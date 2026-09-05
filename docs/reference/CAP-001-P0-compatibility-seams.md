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

## Package/runtime migration matrix

These are future migration destinations, not an accepted schema. The CAP-001 decision and Bundle
child must choose versioning, adapters and rollback before changing any public or persisted name.

| Current compatibility surface | Future semantic destination | I246 treatment / preservation |
|---|---|---|
| `PluginManifest`, `PluginMetadata`, `talos-plugin.toml`, `[plugin]` | Bundle identity/manifest | Preserve public names and existing TOML round trips; no global rename |
| `plugin.name`, `version`, `description`, `talos_protocol` | Bundle metadata and compatibility requirements | Keep parsing/validation; do not infer activation or permission from metadata |
| `plugin.carrier`, `plugin.artifact` | Nested executable Plugin descriptor | Preserve WASM-only validation and confined artifact loading |
| `skills`, `tools`, `hooks` | Typed Bundle contributions | Preserve current fields, handler paths and event validation |
| Tool `handler` and hook `handler` | Executable contribution references | Preserve loader and hook invocation/security behavior |
| Current plugin identity in diagnostics/provenance | Distinct Bundle origin and executable identity | Do not change current diagnostic/provenance identity here |

`crates/talos-plugin/src/manifest.rs` already parses and validates without instantiating an
executable artifact. That is the retained internal separation; a module move is unnecessary.
Manifest validation is not installation, activation, or permission authorization. Existing
runtime/WASM tests remain the behavioral evidence, not this table.

## Desktop fixture and adapter handoff

A renderer can test without loading grammars by constructing this neutral result:

```rust
use talos_text::{HighlightResult, HighlightSpan};
let source = "fn main() {}";
let result = HighlightResult::Spans(vec![HighlightSpan {
    start: 0, end: 2, capture: "keyword".into(),
}]);
// Render source[0..2] with a renderer-local keyword style, then the remaining plain text.
// The fallback fixture is HighlightResult::PlainText with the same source.
```

Offsets are UTF-8 byte ranges, not terminal columns. Renderers must validate ranges before
slicing and own colors, width, wrapping, selection, scrolling and accessibility. The TUI's
`segments_from_result` tests exercise this distinction. A Desktop adapter consumes the result;
it must not import TUI layout types. Conversation projection remains in `talos-conversation`;
this slice does not claim to extract all streaming Markdown semantics from TUI.

I246 owns `crates/talos-text/**`, its consumer migrations, and the corresponding root Cargo
changes. Future Desktop work owns its host/renderer/fixture files. Root Cargo, shared contracts
and validation scripts require changed-file overlap checks against current claims before either
lane changes them. This handoff does not activate a Desktop claim.

## Remaining closure evidence

- Current default-release binary-size measurement and its exact build identity are still pending.
  TOOL-008's historical 64 MB baseline is not evidence of this candidate's size. No size reduction
  or dynamic language loading is claimed by this preparation.
- Remote exact-head CI/review, implementation merge and owner-first completion remain pending.
