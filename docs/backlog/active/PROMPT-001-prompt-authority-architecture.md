# PROMPT-001: Prompt Authority Architecture

| Field | Value |
|---|---|
| Requirement ID | PROMPT-001 |
| Type | Architecture / Requirement Intake |
| Priority | P1 |
| Status | Complete / Closed |
| Source | [GitHub Issue #285](https://github.com/wjhuang88/talos/issues/285) |
| Selected Iteration | None |
| Depends On | Current prompt assembly, scoped context, memory/evolution authority, SDK prompt customization and provider protocol contracts |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex |
| Work Slice | Prompt authority, behavior harness, bounded model decisions and protocol automation |
| Claimed At | 2026-09-15 |
| Source Issue | #285 |
| Governance Claim PR | #286 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Governance registration PR #286; child implementation and review evidence recorded in owner documents. |
| Implementation PR | #550, #552, #559, #562 and direct child commits |
| Last Updated | 2026-09-15 |
| Handoff / Release Condition | Complete; child implementations and evidence are merged and reconciled. |

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

### Approved Scope Addition (2026-09-13)

The #285 long task also includes the following independently testable capability slices. This is
an append-only planning record; it does not activate implementation or alter the published baseline:

- **Automatic tool-protocol selection:** prefer Native when provider capability is confirmed,
  otherwise select a validated compatibility path; retain manual override only as a diagnostic
  escape hatch and preserve existing `cargo run`/`build` defaults.
- **Model-assisted protocol recovery:** use a bounded, sanitized diagnostic model call to classify
  protocol failures and propose correction, fallback, stop, or human review. Never replay an
  executed or execution-unknown write, bypass permission Deny, or recurse without a retry bound.
- **Shared bounded model invocation:** extract one typed primitive for isolated-context,
  dedicated-toolset, single-request decisions (initial consumers: auto permission assessment and
  protocol recovery), with explicit deadline, cancellation, budget, provenance, secret-safe
  observability, and no implicit session/memory/authority inheritance.

Acceptance for this addition requires an accepted ADR/migration contract, focused tests for
capability selection and failure classification, and user-visible diagnostics. Permission and
security surfaces require independent security review. Implementation must be split into runnable
child iterations; I270's harness-only scope remains unchanged.

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

- [x] A reproducible inventory classifies every prompt contributor by source, scope, authority,
      precedence, provenance and cache behavior.
- [x] An accepted ADR defines conflict resolution, public compatibility, migration and rollback.
- [x] Independently runnable children cover the behavior harness and each affected authority/API
      boundary before implementation selection.
- [x] Model-behavior fixtures cover nested instructions, current-user overrides, advisory memory and
      Evolution, Skill/tool content as data, reprioritization, Todo restraint and protocol parity.
- [x] Structural prompt/cache tests, affected SDK docs and residual ownership remain synchronized.

Completion Commit: `e4173caf`, `edc0b09a`, `b400443f`

## 2026-09-15 Final Closeout Evidence

- #45 / SESSION-008 is complete and closed at implementation commit `0f021a6f`.
- Prompt authority, scoped AGENTS precedence, advisory Memory/Todo boundaries, SDK contribution
  boundaries, deterministic behavior harness, bounded model decisions, protocol recovery, replay
  protection and automatic Native/Compat selection are present in the merged child commits listed
  above.
- `release_preflight.sh` passed on current `main`, including locked workspace tests, doctests,
  provider/agent protocol suites, governance validation and collaboration validation (0 warnings).
- Independent Agent-role audits confirmed I267/I270/I272/I273 evidence on current `main`; shared
  account constraints mean Agent-role separation, not natural-person identity separation.
- This closeout uses existing implementation commits as evidence; this status-only commit is not
  used as its own Completion Commit.

## Residual Destination

Every implementation workstream remains in a separately claimed child Story/iteration. This intake
stays Refinement until at least the architecture decision and first runnable child are ready.

## 2026-09-15 Umbrella Acceptance Audit

This is an evidence index, not a completion claim. Each item remains subject to the cited owner
and exact implementation evidence.

| Acceptance area | Current evidence | Result |
|---|---|---|
| Prompt contributor inventory | I264 baseline and I266/I269/I272 child records | Partial: umbrella reconciliation remains |
| Authority and precedence ADR | ADR-074; I265 decision | Present; migration/umbrella closeout remains |
| Independently runnable children | I264/I265/I266/I267/I268/I269/I270/I271/I272/I273 | Present; several owners remain Review |
| Nested instructions and truncation | I267 commits and focused tests | Review |
| Current-user precedence over advisory context | I266/I269/I270 prompt fixtures | Review |
| Memory and Todo advisory boundary | I272 evidence and session tests | Review |
| Skill/tool content treated as data | I270 protocol/capability fixtures | Review |
| Reprioritization and interruption behavior | session boundary tests; I272 | Review |
| Todo restraint/usefulness | I270 Todo fixtures | Review |
| Native/TalosStrict/Compat parity | I273 provider and request-plan tests | Review |
| Stable-prefix/cache behavior | prompt cache tests and I270 harness | Review |
| SDK/extension documentation | I271 and Runtime SDK contract | Review |
| Model-behavior harness | I270 deterministic harness | Review |
| Residual ownership and derived views | child owner documents and indexes | Pending final owner-first synchronization |
| Umbrella completion evidence | PROMPT-001 owner itself | Not met; no umbrella completion commit |

The audit confirms that #45 is independently complete, while #285 remains open until the Review
children and this umbrella record are reconciled and closed with existing implementation commits.
