# PERM-007-F0: Shell Auto Classifier Decision

| Field | Value |
|---|---|
| Story ID | PERM-007-F0 |
| Type | Permission / Security Decision Story |
| Priority | P0 |
| Status | Active / Claimed (proposed; ineffective until claim PR #463 merges) |
| Parent Epic | PERM-007-F / Issue #462 |
| Selected Iteration | I243 |
| Depends On | I241, ADR-012, ADR-040, ADR-069 |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline governance session 2026-09-01 |
| Work Slice | Decision-only ADR-070 classifier context, precedence, exact-request binding, migration/rollback contract, Issue #56/#57 authority reconciliation, and threat matrix. No Rust, Cargo, config schema, runtime behavior, UI, release, or publication authority. |
| Claimed At | 2026-09-01 |
| Source Issue | #462 |
| Governance Claim PR | #463 |
| Authorization Mode | Independent review |
| Authorization Evidence | Maintainer direction in the 2026-09-01 mainline session authorizes a generic model-classifier decision instead of per-command exceptions. Independent exact-head permission/security/API review, CI, validators and merge-time CAS remain mandatory; the proposed claim is ineffective until #463 merges. Shared GitHub identity provides Agent-role separation only, not natural-person identity separation. |
| Implementation PR | None |
| Last Updated | 2026-09-01 |
| Handoff / Release Condition | Accept ADR-070 after independent permission/security/API review; I244 remains separately gated. |

The claim and Active state above are proposals only until governance PR #463 merges to `main`.

## Goal

Replace the incorrect assumption that a shell AST can prove arbitrary commands safe with an
explicit model-classifier contract based on exact action semantics, trusted environment context,
deterministic guardrails, and fail-closed execution binding.

## Scope And Acceptance

Own ADR-070, the classifier precedence/context contract, migration/rollback rules, and adversarial
security matrix. No Rust, Cargo, config schema, or runtime behavior changes. Completion requires an
accepted ADR and exact-head independent permission/security/API approval.

## Required Reads

- `docs/decisions/012-exec-policy-dsl-boundary.md`
- `docs/decisions/040-command-access-evidence-sandbox.md`
- `docs/decisions/069-model-first-permission-triage.md`
- Claude Code official `auto-mode-config` documentation
- `crates/talos-agent/src/auto_resolver.rs`
- `crates/talos-tools/src/bash_tool.rs`
- `crates/talos-permission/src/access_evidence.rs`
