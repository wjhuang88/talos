# MODEL-013: Catalog-Assisted Custom-Model Context Window

| Field | Value |
|---|---|
| Story ID | MODEL-013 |
| Type | Model / Configuration Story |
| Priority | P2 |
| Status | Ready / Unclaimed; I212 claim preparation |
| Source | [GitHub Issue #312](https://github.com/wjhuang88/talos/issues/312) |
| Selected Iteration | I212 — Planned / Unclaimed |
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
| Governance Claim PR | Pending |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-19 |
| Handoff / Release Condition | Finalize the I212 claim, obtain exact-head governance/CI authorization and merge it to `main` before activation or implementation. This proposed claim remains ineffective and grants no Rust/Cargo authority. |

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

## Ready Decision - 2026-08-19

- The packaged `models.toml` exposes provider plus opaque model ID and context metadata; it has no
  separate alias table. Duplicate model IDs are common and some duplicates disagree on limits, so
  any multiplicity is ambiguous even when current values happen to agree.
- Match the raw ID exactly first. If absent, consider only one leading gateway/provider segment
  separated by `/` or `:` and accept it only when the remaining ID selects exactly one catalog row.
  Do not strip `@`, version suffixes, substrings or edit distance; exact IDs containing those
  characters remain opaque and matchable.
- Existing `ModelConfig.context_limit: Option<u32>` avoids a schema migration: `Some` is explicit
  configured authority; `None` plus one accepted match is catalog-inferred; `None` without one is
  unknown. Additive resolver provenance may expose those states without changing the public struct.
- Inference is derived at resolution/display time and does not persist merely to materialize a
  default. A later catalog update therefore cannot replace an explicit `Some` value.
- MODEL-011/#124 remains the independent active-probe path. MODEL-013 projects context metadata only
  and never imports endpoint capability, pricing, output limit, provider identity or routing facts.

## Exclusions

- No active context-window probe or network request.
- No generic fuzzy model-name matching or duplicated catalog data.
- No inference of tools, images, reasoning or other endpoint capabilities.
- No model-role/routing decision owned by MODEL-012/#146.
- No implementation iteration, branch or authorization from this intake record.

## Acceptance For I212

- [x] Exact, supported one-prefix normalization, ambiguity and unknown decisions are deterministic.
- [x] Explicit values always win and catalog updates cannot silently replace them.
- [x] Provenance is derived without a config/public-struct migration; rollback is removal of the
      resolver/display projection with existing config remaining readable.
- [x] Catalog entries without context metadata produce no fabricated value.
- [x] MODEL-011 capability probing remains an independent evidence path.
- [ ] I212 has an effective Collaboration Claim on `main` before implementation.

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
- `docs/iterations/I212-model013-catalog-context-window-inference.md`

## Change Control - 2026-08-19 Priority Advance

The maintainer explicitly requested Issue #312 implementation before the remaining I198/#155 child
of the active mainline long task. The requirement remains an independent MODEL-013/I212 slice; this
priority change does not merge it into I201, I198 or I211 and does not authorize implementation
before the separate I212 claim reaches `main`.
