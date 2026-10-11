# ADR-088: Trusted Todo Result Presentation

## Status

Accepted by maintainer on 2026-10-11. Acceptance does not expand the effective
Collaboration Claim or replace independent implementation/API/privacy review.

## Context

Todo results contain arbitrary multiline titles, descriptions and tags. Text prefixes,
headers and UUIDs cannot distinguish producer-owned status from user content. Decorating
only the first status does not satisfy I299's complete-list presentation requirement.

## Decision

The registered native Todo producer captures typed status and item-row spans while
constructing each successful result, before appending user text. Never recover provenance
by parsing result text, re-query mutable storage while rendering, or reuse tool-disclosure
continuations as formatting metadata.

Carry bounded optional presentation metadata with exact result and invocation identity
through known wrappers, Agent events, conversation projection and TUI. Eligibility comes
from the registered native producer, not a tool name, MCP/plugin payload or caller text.
Known wrappers preserve eligibility; errors discard metadata. Rendering has no execution,
storage or permission authority.

Use checked UTF-8 byte offsets, ordered nonoverlapping spans, explicit count/size caps and
exact status-text checks. Bind metadata to final visible text by a strong digest and length
or equivalently verified identity. Hooks, redaction or projections that change text without
explicit remapping invalidate metadata wholly. Invalid metadata renders literally; it must
not carry hidden raw content into the UI or guess missing rows.

Map validated original spans before wrapping and head/tail truncation. Omitted spans cannot
leave orphan decoration. Status replacement and item-owned row highlight are distinct;
multiline user text does not establish status authority.

Model input, durable raw transcript and exports remain byte-for-byte unchanged. Restored
legacy results without metadata render literally. Persisting display metadata requires a
separate decision; no transcript migration or new dependency is authorized here.

## Compatibility And Migration

Inventory public result, projection, event and UI carriers before implementation. Optional
field additions still break external Rust struct literals; added exhaustive enum variants
require migration. Document each actual change and supported constructors/defaults, with
absent metadata retaining literal behavior. Source-breaking surfaces ship only in the next
pre-1.0 minor release, not a patch. This ADR does not authorize a release.

Known affected candidates are `talos_core::tool::ToolResult` and
`ToolResultProjection`, `talos_core::message::AgentEvent` and
`talos_conversation::ToolResultDisplay`; the implementation must confirm its final API
inventory. External callers using struct literals must adopt the documented constructor or
supply the new optional field where applicable. Old recorded results remain readable and
unadorned without a storage migration.

## Verification And Rollback

Require all-item tests with multiline titles/descriptions/tags, forged headers/UUIDs,
normal/head/tail rendering, concurrent invocation correlation, errors, unrelated tools,
hook mutation, private-token redaction and raw export equivalence. Independent API/privacy
review and native TUI acceptance remain required. Discard metadata or revert this bounded
projection if trust, privacy, text preservation or correlation fails.

## Related

- [I299](../iterations/I299-preserved-presentation-recovery.md)
- [ADR-080](080-correlated-tool-activity-presentation.md)
- [ADR-045](045-transient-model-private-tool-projection.md)
