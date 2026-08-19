# MODEL-013: Catalog-Assisted Custom-Model Context Window

| Field | Value |
|---|---|
| Story ID | MODEL-013 |
| Type | Model / Configuration Story |
| Priority | P2 |
| Status | Refinement / Unclaimed |
| Source | [GitHub Issue #312](https://github.com/wjhuang88/talos/issues/312) |
| Selected Iteration | None |
| Depends On | MODEL-008 custom-model registration; canonical model catalog; MODEL-011 evidence precedence boundary |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #312 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-19 |
| Handoff / Release Condition | Decide conservative identity matching, precedence and provenance representation, then select one runnable iteration through a separate effective claim. This intake grants no implementation authority. |

## Identity / Goal / Value

Reduce avoidable custom-model setup by reusing a uniquely matched canonical catalog context window
as an editable default without turning model-name matching into authoritative capability evidence.

## Proposed Scope

- Reuse the existing local canonical model catalog; do not add a second lookup table.
- Prefer exact canonical identity, then only explicitly reviewed provider-aware aliases or
  deterministic gateway normalization that yields one unique candidate.
- Apply precedence: explicit user configuration, then catalog inference, then existing
  unknown/conservative fallback.
- Preserve enough provenance to distinguish a user-authored value from catalog-derived metadata.
- Leave unknown or ambiguous models registerable without a fabricated context size or network call.

## Required Decisions Before Ready

- Inventory the canonical model identity/alias fields and current custom-model persistence path.
- Define supported normalization rules for `/`, `:`, `@`, provider prefixes and version suffixes;
  generic substring, edit-distance or semantic matching is prohibited.
- Decide whether existing config metadata can represent configured/catalog-inferred/unknown
  provenance without a migration; otherwise create a separate migration/ADR owner.
- Define when inference occurs and prove a later catalog update cannot overwrite an explicit value.
- Keep MODEL-011/#124 active probing and capability evidence independent from passive metadata
  convenience.

## Exclusions

- No active context-window probe or network request.
- No generic fuzzy model-name matching or duplicated catalog data.
- No inference of tools, images, reasoning or other endpoint capabilities.
- No model-role/routing decision owned by MODEL-012/#146.
- No implementation iteration, branch or authorization from this intake record.

## Acceptance For Refinement

- [ ] Exact, alias, supported-normalization, ambiguity and unknown cases are deterministic.
- [ ] Explicit values always win and catalog updates cannot silently replace them.
- [ ] Provenance representation and any migration/rollback consequence are explicit.
- [ ] Catalog entries without context metadata produce no fabricated value.
- [ ] MODEL-011 capability probing remains an independent evidence path.
- [ ] One runnable iteration and effective Collaboration Claim exist before implementation.

## State / Status Owners

- Story status and refinement decisions: this file.
- Remote requirement state: GitHub Issue #312.
- Compact planning view: `docs/backlog/PRODUCT-BACKLOG.md`.
- Derived operating view: `docs/BOARD.md`.

## User-Facing Documentation

A future implementation must document the visible inferred-source hint and override behavior. This
intake changes no runtime or user-visible behavior.

## Required Reads

- `docs/backlog/active/MODEL-008-interactive-custom-provider-registration.md`
- `docs/backlog/active/MODEL-011-custom-model-capability-probe.md`
- `docs/backlog/active/MODEL-012-utility-model-role-and-bounded-routing.md`
- `docs/decisions/022-agent-config-compatibility-boundary.md`
