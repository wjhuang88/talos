# PROMPT-001: Prompt Authority Architecture

| Field | Value |
|---|---|
| Requirement ID | PROMPT-001 |
| Type | Architecture / Requirement Intake |
| Priority | P1 |
| Status | Refinement / Unclaimed |
| Source | [GitHub Issue #285](https://github.com/wjhuang88/talos/issues/285) |
| Selected Iteration | None |
| Depends On | Current prompt assembly, scoped context, memory/evolution authority, SDK prompt customization and provider protocol contracts |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #285 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-18 |
| Handoff / Release Condition | Accept the authority/precedence architecture and decompose independently runnable children before any implementation claim. |

## Identity / Goal / Value

Make model-visible context carry explicit source, scope, authority, precedence, provenance and cache
semantics before rendering, so current user intent and runtime invariants cannot be silently
overridden by advisory memory, learned patterns, broad project context or extension text.

## Scope

- Define one authority and precedence model for runtime/core rules, scoped instructions, current
  user intent, user preferences, advisory context and runtime facts.
- Reconcile Evolution, Memory, nested `AGENTS.md`, identity/capability prompt composition, Todo and
  steering policy, SDK prompt customization and hook prompt contributions against that model.
- Define instruction-aware context budgeting and provenance/diagnostic requirements.
- Establish structural plus representative model-behavior regression scenarios before semantic
  prompt changes.
- Decompose the umbrella into separately claimable decision, harness, authority-correction,
  capability-decomposition and SDK/extension migration children.

## Exclusions

- No prompt, context loader, memory, Evolution, Todo, hook, SDK, provider or runtime implementation
  from this intake.
- No immediate rewrite of every prompt, provider protocol, plugin surface or public API.
- No transfer of I205/GOV-007 workflow-audit authority and no change to current permission policy.

## Dependencies

- ADR-033 remains the current advisory-memory precedence baseline.
- Public SDK changes require a semver decision and migration plan.
- Prompt behavior conclusions require recorded model/provider fixtures rather than string snapshots
  alone.
- Security-sensitive authority or extension changes require independent review.

## Decision Links And Constraints

- A new ADR must define the canonical authority/precedence model and compatibility boundary.
- Existing stable/dynamic prompt partitioning, bounded memory injection, Skill on-demand loading,
  tool schemas and Native/TalosStrict/Compat separation are preservation constraints until evidence
  supports a change.
- Advisory inputs must not acquire runtime/core or current-user authority through imperative wording.

## Uncertainty And Validation Path

Inventory every current prompt contributor and its runtime enforcement boundary, then use a bounded
Spike to capture prompt size/cache baselines and model-behavior fixtures. The exact Rust types,
provider message layout and rollout order remain undecided and cannot become implementation scope
through this intake.

## State / Status Owners

- Requirement scope and readiness: this document.
- External discussion: Issue #285.
- Future architecture decision and children: separately numbered ADR/backlog/iteration owners.
- Board/backlog/Issue mapping: derived views only.

## User-Facing Documentation

Future behavior changes must update SDK prompt customization, `AGENTS.md` precedence, Memory and
Evolution guidance. This intake claims no shipped behavior.

## Required Reads

- [Issue #285](https://github.com/wjhuang88/talos/issues/285)
- `crates/talos-agent/src/prompt/`
- `crates/talos-agent/src/context.rs`
- `crates/talos-agent/src/configuration.rs`
- `crates/talos-evolution/`
- `crates/talos-memory/`
- `crates/talos-plugin/`
- `docs/decisions/033-associative-memory-injection-policy.md`
- `docs/reference/RUNTIME-SDK-CONTRACT.md`
- `docs/sop/REQUIREMENT-INTAKE.md`

## Acceptance For Technical / Governance Work

- [ ] A reproducible inventory classifies every prompt contributor by source, scope, authority,
      precedence, provenance and cache behavior.
- [ ] An accepted ADR defines conflict resolution, public compatibility, migration and rollback.
- [ ] Independently runnable children cover the behavior harness and each affected authority/API
      boundary before implementation selection.
- [ ] Model-behavior fixtures cover nested instructions, current-user overrides, advisory memory and
      Evolution, Skill/tool content as data, reprioritization, Todo restraint and protocol parity.
- [ ] Structural prompt/cache tests, affected SDK docs and residual ownership remain synchronized.

## Residual Destination

Every implementation workstream remains in a separately claimed child Story/iteration. This intake
stays Refinement until at least the architecture decision and first runnable child are ready.
