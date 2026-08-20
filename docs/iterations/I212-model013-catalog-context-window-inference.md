# Iteration I212: Catalog-Assisted Custom-Model Context Window

> Document status: Review / Claimed - implementation merged, human validation deferred
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
| Authorization Evidence | PR #314 exact head `ec5c6920` passed CI `32223903534`, both governance validators and merge-time CAS; independent Agent review disconnected without a conclusion, so the planning-only claim used the SOP single-maintainer path with disclosure `5338629524` and merged as `a62f448b`. |
| Implementation PR | #318 (merged as `5a1709cb`) |
| Last Updated | 2026-08-20 |
| Handoff / Release Condition | Implementation head `a2466c55` passed CI `32319297491`, independent Agent review `5349952979` and CAS, then merged as `5a1709cb`. Keep I212 Review until the Issue #302/I211 natural-person walkthrough passes; no further implementation authority transfers. |

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

Claim PR #314 final head `ec5c6920` passed exact-head CI `32223903534`, both governance validators
and merge-time CAS, then merged to `main` as `a62f448b`. The independent Agent reviewer attempt
disconnected without producing a conclusion; the planning-only claim used the SOP single-maintainer
path with disclosure `5338629524`. I212 is Active/Claimed and may create its implementation branch
from `a62f448b` or later current `main` within the Published Baseline only.

Implementation commit `3cb1a801` adds conservative packaged-catalog context resolution and the
`Configured` / `CatalogInferred` / `Unknown` provenance result. Custom providers infer only from
one exact opaque ID or, when no exact row exists, one `/` or `:` prefix removal that yields exactly
one row with context metadata. Built-in providers remain provider-qualified; custom providers never
inherit catalog output limits, capabilities, pricing or routing. `/model` ready and recent rows
label only inferred custom context as `(catalog)`. Cargo manifests, default features, persistence
and network behavior are unchanged.

## Verification Evidence

- `cargo test -p talos-config --locked`: 224 unit tests and 1 doctest passed with an isolated
  writable HOME; a prior non-isolated run exposed one existing HOME-mutating test race, while every
  I212 test passed and the isolated full rerun was green.
- `cargo test -p talos-cli --locked model_lifecycle::tests::`: 28 focused lifecycle tests passed.
- `cargo clippy -p talos-config -p talos-cli --all-targets --locked -- -D warnings` passed.
- `cargo fmt --all -- --check` and `git diff --check` passed.
- Diff review confirmed no Cargo manifest, `Cargo.lock`, default-feature, dependency, persistence or
  network change. `HOME=/private/tmp/talos-i212-test-home RUSTUP_HOME=/Users/GHuang/.rustup
  CARGO_HOME=/Users/GHuang/.cargo ./scripts/release_preflight.sh` passed outside the outer execution
  sandbox, including macOS seatbelt tests. Exact-head CI, independent Agent technical review,
  merge-time CAS and the Issue #302 natural-person custom-provider walkthrough remain pending.

## Completion Evidence

Completion Commit: Pending. A status-only commit cannot serve as its own evidence.

## Variance And Residuals

Alias registries, active capability probes, role routing and broader model metadata inference remain
owned by separate Stories. Issue #316 owns the non-isolated `talos-config`/CLI test race where
parallel tests mutate process `HOME`; an isolated-HOME full run passed, and this test-environment
defect is not claimed as an I212 product defect.

## Retrospective

Pending execution.

## 2026-08-20 Review And Main Synchronization Checkpoint

Independent Agent review `5349780559` bound to superseded head `7f6838a0` verified the I212 code
semantics and requested changes only for failed exact-head remote owner reconciliation (#316/#317)
and the stale unchecked claim-acceptance row in MODEL-013. PR #319 created the separate Unclaimed
owners and merged as `8d0d3166`; I212 was rebased onto that current `main`, and the owner now marks
the effective claim fact truthfully. This invalidates the old exact-head CI and review disposition.
I212 stays Review/Active with Completion Commit pending until the new head passes CI, independent
review and merge-time CAS; its natural-person custom-provider row remains deferred to #302/I211.

## 2026-08-20 Implementation Merge Disposition

PR #318 exact head `a2466c55641cc893ae5cf9248519af8b1ca4f093` passed exact-head CI
`32319297491` (5/5), independent Agent technical review `5349952979`, both governance validators
and merge-time CAS, then merged to `main` as `5a1709cbcdb4ec1960fae637bfe48cd93e817d87`.
Implementation and machine/technical merge gates are terminal. The source and iteration remain
Review / Claimed with `Completion Commit: Pending` because the natural-person custom-provider
walkthrough remains open in Issue #302/I211. This state-only synchronization cannot serve as
completion evidence.
