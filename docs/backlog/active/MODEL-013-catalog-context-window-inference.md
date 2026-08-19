# MODEL-013: Catalog-Assisted Custom-Model Context Window

| Field | Value |
|---|---|
| Story ID | MODEL-013 |
| Type | Model / Configuration Story |
| Priority | P2 |
| Status | In Progress / Claimed; I212 Active |
| Source | [GitHub Issue #312](https://github.com/wjhuang88/talos/issues/312) |
| Selected Iteration | I212 — Active / Claimed |
| Depends On | MODEL-008 custom-model registration; canonical model catalog; MODEL-011 evidence precedence boundary |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline planning session |
| Work Slice | I212/MODEL-013 only: pure local catalog identity resolver and context-window projection for custom models with explicit-value precedence, ambiguity rejection and derived provenance. Excludes active probes, network calls, capability/pricing/role inference, schema migration, dependency changes and release/publication. |
| Claimed At | 2026-08-19 |
| Source Issue | #312 |
| Governance Claim PR | #314 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | PR #314 exact head `ec5c6920` passed CI `32223903534`, both governance validators and merge-time CAS; independent Agent review was attempted but disconnected without a conclusion, so the planning-only claim used the SOP single-maintainer path with disclosure `5338629524` and merged as `a62f448b`. |
| Implementation PR | #318 |
| Last Updated | 2026-08-19 |
| Handoff / Release Condition | Claim #314 is effective through merge `a62f448b`; implementation must branch from that merge or later current `main` and remain within the recorded Work Slice. |

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

## 2026-08-19 Claim Activation Checkpoint

Claim PR #314 final head `ec5c6920` passed exact-head CI `32223903534`, both governance validators
and merge-time CAS, then merged to `main` as `a62f448b`. The independent Agent reviewer attempt
disconnected without producing a conclusion; the planning-only claim therefore used the SOP
single-maintainer path with shared-identity/unavailable-review disclosure `5338629524`. I212 is now
Active/Claimed; its implementation must begin from `a62f448b` or later current `main` and remains
limited to local context-window inference without probes, capability inference or migration.

## 2026-08-19 Implementation Checkpoint

Implementation commit `3cb1a801` reuses the packaged catalog without a duplicated table or network
call, exposes configured/catalog-inferred/unknown context provenance, applies inference to custom
runtime limits and `/model` ready/recent rows, and labels inferred custom values `(catalog)`.
Exact identity wins; ambiguity, missing context, unknown IDs, opaque suffixes and multiple prefixes
remain unknown. Built-in provider matching stays provider-qualified, and no output limit,
capability, pricing, provider identity or routing metadata is inherited.

The isolated-HOME `talos-config` suite passed 224 unit tests plus one doctest; 28 focused CLI model
lifecycle tests, strict Clippy, formatting and `git diff --check` passed. Full
`./scripts/release_preflight.sh` passed outside the outer execution sandbox with macOS seatbelt
tests. Exact-head CI, independent Agent review, CAS and the Issue #302 natural-person walkthrough
remain open, so MODEL-013 stays In Progress and has no Completion Commit. Issue #316 tracks the
separate process-HOME test isolation defect observed during non-isolated runs.
