# MODEL-014: Catalog-Assisted Custom-Model Capability And Limit Inference

**Status**: Refinement
**Type**: Product / Configuration Story
**Parent Epic**: None
**Source**: Maintainer request in the current session; related to MODEL-011 / Issue #124 and MODEL-013 / Issue #312.
**Selected Iteration**: None

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | None; related discussion #124 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-10-09 |
| Handoff / Release Condition | Resolve capability authority and adapter semantics, then establish a target-branch claim before implementation. |

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

## Uncertainty And Validation Path

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

This file owns scope and readiness. PRODUCT-BACKLOG.md is the compact index; BOARD.md is a derived operating view. No implementation is active or complete.

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