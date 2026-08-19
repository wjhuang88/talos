# Iteration I212: Catalog-Assisted Custom-Model Context Window

> Document status: Planned / Claimed proposal
> Published plan date: 2026-08-19
> Planned objective: resolve a custom provider model against the packaged Talos model catalog only
> when one conservative local identity match exists, then use its context window as an editable
> catalog-derived default without replacing explicit configuration or asserting endpoint capability.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: a maintainer can register or configure exact, normalized, ambiguous and unknown
> custom-model identities and observe deterministic context-window resolution and provenance through
> the existing CLI model lifecycle without a network request or config-schema migration.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline planning session |
| Work Slice | I212/MODEL-013 only: pure local catalog identity resolution and custom-model context-window projection with explicit precedence, ambiguity rejection and derived provenance. No active probe, network, capability inference, migration, dependency or release work. |
| Claimed At | 2026-08-19 |
| Source Issue | #312 |
| Governance Claim PR | #314 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Draft claim PR #314; finalized exact-head governance/CI checks, independent technical review or documented single-maintainer authorization, and merge-time CAS remain required. |
| Implementation PR | Not started |
| Last Updated | 2026-08-19 |
| Handoff / Release Condition | Claim #314 is ineffective until finalized and merged to `main`; activate I212 separately only after that merge, then branch from the claim merge or later current `main`. |

## Published Baseline

### Selected Story

| Story | Status At Selection | Depends On | Outcome |
|---|---|---|---|
| MODEL-013 / Issue #312 | Ready | MODEL-008 complete custom registration; packaged `models.toml`; existing config precedence | One runnable local-only catalog inference slice with explicit provenance and ambiguity rejection |

### Decision Checkpoint

- `ModelConfig.context_limit = Some(value)` remains an explicit configured override and always wins.
- `context_limit = None` remains schema-compatible. Resolution may report `CatalogInferred` only
  when the packaged catalog yields one accepted identity with known context metadata; otherwise it
  reports `Unknown` and preserves the existing conservative runtime fallback.
- No persisted provenance field is required: configured, catalog-inferred and unknown are derived
  from the existing optional field plus the deterministic resolver. This avoids a public struct
  change, config migration and rollback surface.
- Matching order is exact model ID first, then at most one reviewed leading gateway/provider segment
  separated by `/` or `:`. A normalized candidate is accepted only when it identifies exactly one
  catalog record. `@`, version suffixes, substrings and edit distance are never stripped or guessed;
  exact catalog IDs containing those characters remain valid exact matches.
- Only context-window metadata is projected. Provider, pricing, tools, images, reasoning, output
  limit and other capability fields are not inherited from the matched record.

### Scope

- Add a pure catalog identity resolver in the existing model/config boundary with an explicit
  configured/catalog-inferred/unknown result.
- Apply the resolver to custom-provider context-limit resolution and model-picker presentation while
  preserving the current explicit-value and built-in-provider paths.
- Surface a secondary catalog-derived hint through an existing compatible display surface without
  changing published conversation/session protocols.
- Cover exact, one-prefix normalization, duplicate, conflicting-limit, unknown, missing-context,
  `/`, `:`, `@`, suffix, explicit override and no-network cases.
- Update configuration/model documentation and Issue #312 evidence.

### Non-Goals

- No provider call, active probe, `/model-probe`, database, catalog refresh or duplicated lookup table.
- No fuzzy/semantic matching, suffix removal or inference across multiple plausible records even
  when their current context values happen to agree.
- No capability, pricing, output-limit, role/routing or provider-identity inference.
- No `ModelConfig` field addition, public API break, persistent migration or Cargo dependency change.
- No MODEL-011/#124 or MODEL-012/#146 implementation.

### Acceptance And Planned Validation

- Exact and supported one-prefix identities infer only when one catalog record with a known context
  window remains; duplicate, ambiguous, unknown and missing-context inputs do not infer.
- Explicit configured context always wins before matching and remains unchanged after catalog data
  changes; inference performs no write solely to materialize a derived value.
- Runtime can distinguish `Configured`, `CatalogInferred` and `Unknown`, and picker/user messaging
  labels catalog provenance without treating it as capability proof.
- Built-in provider/model behavior, custom registration, structured model IDs and conservative
  fallback remain compatible.
- Focused `talos-config` and `talos-cli` tests, locked workspace checks/tests, strict Clippy, release
  preflight, both governance validators and `git diff --check` pass.
- Independent Agent technical review and merge-time CAS pass. A natural-person custom-provider
  walkthrough is tracked in Issue #302/I211 while I212 remains Review.

### Documentation Target

- MODEL-013, Issue #312, configuration reference and the user-facing custom-provider/model guidance
  that owns the context-window display and override behavior.

### Risks And Fallback

- Duplicate public model IDs are common across catalog providers and may disagree on limits; any
  multiplicity rejects inference instead of choosing the first row or a majority value.
- Prefix removal can damage legitimate opaque IDs; exact match always runs first and only one leading
  `/` or `:` segment is considered. `@` and suffixes remain opaque.
- Fallback: retain the current unknown/conservative behavior and keep I212 Review/Partial rather
  than persisting or displaying an unproven limit.

## Actual Activation And Execution

No activation has occurred. Claim #314 is a proposed governance record and is ineffective until its
finalized record reaches `main`; no implementation branch or code change is authorized by this plan.

## Verification Evidence

Pending an effective claim and implementation.

## Completion Evidence

Completion Commit: Pending. A status-only commit cannot serve as its own evidence.

## Variance And Residuals

Alias registries, active capability probes, role routing and broader model metadata inference remain
owned by separate Stories.

## Retrospective

Pending execution.
