# Iteration I264: Prompt Authority Architecture Baseline

| Field | Value |
|---|---|
| Status | Planned / Unclaimed |
| Source | PROMPT-001 / Issue #285 |
| Deliverable | Source-backed inventory and accepted authority/precedence decision boundary for prompt inputs; no semantic prompt rewrite. |
| Depends On | ADR-033, current prompt/context/evolution/memory/plugin owners |
| Excludes | No Rust prompt behavior changes, provider changes, SDK breaking change, or model-harness implementation. |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Source Issue | #285 |
| Implementation PR | Not started |

## Acceptance

- Inventory every current prompt contributor, scope, authority, provenance and cache behavior.
- Record conflicts and choose an ADR/migration boundary without changing runtime behavior.
- Decompose independently runnable authority, decomposition and behavior-harness children.
- Preserve current prompt assembly and SDK compatibility until later implementation claims.

Completion Commit: pending
