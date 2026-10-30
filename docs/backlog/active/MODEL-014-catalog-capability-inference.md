# MODEL-014: Catalog-Assisted Custom-Model Capability And Limit Inference

| Field | Value |
|---|---|
| Story ID | MODEL-014 |
| Status | Complete |
| Collaboration Claim | Delegated to I311 / merged #813 |

**Type**: Product / Configuration Story
**Parent Epic**: None
**Source**: Maintainer request in the current session; related to MODEL-011 / Issue #124 and MODEL-013 / Issue #312.
**Selected Iteration**: I311 / merged #813; implementation #815

## Execution ownership

Execution claim is delegated to [I311](../../iterations/I311-model014-capability-inference.md)
in merged #813; target-branch claim is effective at `0aff4e4fbf2a653521f0da9907900341772efc24`. The original Unclaimed intake baseline is preserved at `d46b044d3`.
This Story does not create a second effective claim for the same Work Slice.

## Identity / Goal / Value

Custom-provider users should receive catalog-inferred image-input capability, reasoning capability and output limit, alongside the existing context-window inference, without manually duplicating known model metadata. The maintainer explicitly requests this expansion; it is new scope, not a correction to the completed context-only I212 acceptance baseline.

Catalog matching is an inference about the named model, not proof that a custom endpoint supports its capabilities. MODEL-011 / #124 remains the separate active-probe workflow. The earlier variant-selection request must be evaluated alongside reasoning support; copying presets must not silently establish endpoint support.

## Scope

- Reuse the canonical catalog and the existing exact-ID / single-prefix, unique-candidate matching boundary.
- Infer image-input support, reasoning support and output limit for configured custom models when their corresponding explicit override is absent.
- Preserve explicit user configuration, including negative capability overrides; do not silently persist derived defaults.
- Expose the matched catalog identity and inferred source, distinct from explicit configuration and future probe evidence.
- Apply accepted inferred values consistently to model display, image attachment eligibility, reasoning invocation and output-limit resolution, subject to adapter protocol support.
- Define how reasoning variants are derived and selected without confusing presets with endpoint evidence.

## Exclusions

No active or background probe, network request for inference, fuzzy matching, second catalog, pricing inference, role/routing inference, release work or implicit implementation authorization. Do not reopen or replace the published I212 baseline.

## Dependencies

MODEL-013 identity resolver; MODEL-007 variants; MODEL-009 image-input boundary; MODEL-011 evidence-precedence coordination; current provider adapters and atomic configuration mutation boundary.

## Decision Links And Constraints

- This request supersedes the context-only exclusion for this new Story, not retroactively for MODEL-013/I212.
- Image inference changes functionality gating. Review the governing image-input ADR and document the accepted authority before enabling it; preserve file permissions and adapter validation.
- Define precedence between explicit configuration, fresh probe evidence, catalog inference and unknown fallback. Probe schema or execution is not part of this Story.
- A gateway can filter capabilities despite sharing a canonical model ID. Source presentation must not describe inferred metadata as endpoint-verified support.
- Any public API or config-schema compatibility change requires the repository's decision and migration process.

## Historical Intake Uncertainty And Validation Path

The following records pre-activation uncertainty, resolved by ADR-095 and effective claim #813; it does not override the current merged implementation checkpoint.

Before activation, decide the representation of inferred/unknown/explicit capabilities, whether inferred image support directly enables attachments, reasoning enablement versus effort presets, negative override semantics, and protocol-specific output caps. The requested end-to-end inference is accepted as scope; these implementation and safety decisions remain unresolved.

## Acceptance For Behavior

- Given a uniquely matched custom model without overrides, when metadata is resolved, context, image-input capability, reasoning capability and output limit are derived from the same accepted catalog identity and marked inferred.
- Given explicit overrides, including disabled image/reasoning support, when catalog data changes, those overrides remain authoritative.
- Given an unknown or ambiguous identity or a missing field, when metadata is resolved, no capability or numeric limit is fabricated.
- Given inferred image support and an adapter that accepts images, when the user attaches an image, attachment eligibility uses the accepted inference policy while normal permissions and content checks remain in force.
- Given inferred reasoning support, when reasoning or a variant is selected, display and invocation agree and incompatible adapters do not receive unsupported parameters.
- Given an inferred output limit, when request limits are resolved, explicit limits take precedence and adapter-specific constraints are respected.
- Resolution sends no provider requests and does not materialize inference into user-authored config.
- Catalog inference is never labeled as successful endpoint probing and does not implement or complete #124.

## State / Status Owners

This file owns scope and readiness. PRODUCT-BACKLOG.md is the compact index; BOARD.md is a derived operating view. Implementation #815 and acceptance follow-up #816 are merged; see I311 for final acceptance evidence.

## User-Facing Documentation

Update the custom-provider/model setup documentation and model-selection documentation to describe inferred fields, provenance, override behavior, unsupported-adapter behavior and the distinction from active probes.

## Required Reads

- docs/backlog/active/MODEL-013-catalog-context-window-inference.md
- docs/iterations/I212-model013-catalog-context-window-inference.md
- docs/backlog/active/MODEL-011-custom-model-capability-probe.md
- docs/backlog/active/MODEL-007-hierarchical-model-variant-selection.md
- docs/backlog/active/MODEL-009-multimodal-image-input.md
- docs/sop/CHANGE-CONTROL.md
- docs/sop/AGENT-COLLABORATION.md

## Minimum Validation

Focused config/resolver, explicit-override, ambiguity, image-eligibility, reasoning-variant and adapter request tests; locked workspace validation and both governance validators. Add an end-to-end custom-provider walkthrough covering source display and actual attachment/reasoning behavior before completion.

## Residual Destination

Active endpoint verification remains MODEL-011 / #124. Pricing and role/routing inference require separate scope decisions.

## Historical restoration and activation preparation — 2026-10-30

The original published intake above is restored from
`1bda5181f033b49f3f05f304cfd1bef2489e6a04` without reducing its scope.
Status remains Refinement / Unclaimed; no implementation is activated.
The proposed authority, schema compatibility and adapter matrix are recorded in
[the activation design](../../proposals/model014-capability-inference-authority.md).
Independent static design review accepted the revised direction, not an effective
claim or implementation. A finalized ADR and atomic claim/activation remain
required after I308 closure, with fresh iteration/PR overlap inventory.
PR #811 separately proposes I310/ADR-094 for Gemini; those IDs and its protocol/auth
scope are not claimed here. No paid probes or endpoint-support claims are added.


## Historical atomic activation proposal — 2026-10-30

I311 selects the complete original T2+T3 scope. ADR-095 defines the reviewed
authority, negative override/schema compatibility and protocol mapping. These
records are proposed in #813 and ineffective until that finalized claim PR merges.
The single effective execution claim will live in I311; this Story's prior
Unclaimed intake record is historical only after that merge. No code is included.

Implementation #815 merged as `0469f8041ed0d24bb2dffb465e9ff7fa3609a034`;
see I311 for exact-head CI and review evidence. Acceptance closeout remains pending.

## Completion — 2026-10-30

Completion Commit: `0469f8041ed0d24bb2dffb465e9ff7fa3609a034`, `b04421fb964737911551c7e0704d6c81e80aa406`.
All eight acceptance rows are reconciled in I311. Full locked preflight, exact-head
CI `41032515451`, independent review and CAS passed; #816 merged as
`ca4a79961366eba7bc722b90bd449404cc7b07d5`. Earlier pending-closeout entries
are historical. MODEL-011/#124 and handoff T4/T5/T1/T6 remain separate.
