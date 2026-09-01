# PERM-007-F: Generic Shell Effect Classification

| Field | Value |
|---|---|
| Story ID | PERM-007-F |
| Type | Permission / Security / Runtime Story |
| Priority | P0 |
| Status | Blocked / Unclaimed |
| Parent Epic | PERM-007 (closed; this is a separately governed follow-up) |
| Source | GitHub Issue #462 |
| Selected Iteration | I244 |
| Depends On | PERM-007-F0 / I243, ADR-070; reconcile PERM-006-D / Issue #56 and PERM-006-E / Issue #57 authority |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned; no implementation authority |
| Claimed At | Not applicable |
| Source Issue | #462 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | None |
| Last Updated | 2026-09-01 |
| Handoff / Release Condition | Accept ADR-070 through I243, then establish a separate I244 implementation claim. |

## Goal And Value

Provide Claude-like model-first triage for routine shell commands without maintaining a per-command
allowlist. Deterministic policy runs first; an isolated model classifier then judges exact command
semantics, current bounded user intent, and trusted environment context. Uncertain or dangerous
actions remain human-required or denied.

## Scope

- Route shell calls through the classifier after deterministic deny and explicit-ask rules.
- Provide exact normalized action semantics, bounded user intent, workspace/remotes and trusted
  environment context to a tool-free classifier after authoritative secret/exfiltration guards.
- Use parser/AST and access evidence as advisory context, never as proof that arbitrary commands are
  safe.
- Bind model assessment and execution to the same action, arguments, cwd, environment identity,
  policy revision and session.
- Preserve deterministic deny precedence, existing permission/admission pipeline, fail-closed
  behavior, and cross-surface semantics.

## Exclusions

No blanket shell approval, model override of policy/sandbox/admission, unrestricted environment or
network access, permanent grants, implicit shell semantics, Desktop, release or publication work.

## Acceptance

- Given a routine command such as `ls -la`, when deterministic policy permits assessment and the
  model returns a valid high-confidence result bound to the exact request, then one
  `AllowOnce` is admitted without a human prompt.
- Given destructive, exfiltrating, privileged, protected-target, secret-bearing, or ambiguous
  intent, then the request is never auto-approved and follows the human-required or deny path.
- Given any parser/model timeout, malformed result, stale digest, cancellation, or execution-input
  change, then authorization fails closed.
- Given equivalent CLI, TUI, Runtime and MCP requests, then effect classification and authorization
  semantics remain equivalent.

## Required Validation

- Structural/adversarial tests for quoting, escaping, separators, substitutions and malformed input.
- Effect-classification matrix tests covering read/write/destructive/network/privilege/unknown.
- Exact action/environment/cwd/revision digest binding tests.
- Locked workspace checks, governance validators, and independent permission/security/API review at
  exact head.

## Required Reads

- `docs/decisions/069-model-first-permission-triage.md`
- `docs/decisions/070-shell-auto-classifier-context.md` (proposed by I243)
- `docs/decisions/012-exec-policy-dsl-boundary.md`
- `docs/decisions/040-command-access-evidence-sandbox.md`
- `docs/backlog/active/PERM-007-E-model-first-permission-triage.md`
- `crates/talos-tools/src/bash_tool.rs`
- `crates/talos-tools/src/exec_tool.rs`
- `crates/talos-agent/src/auto_resolver.rs`
- `docs/backlog/active/PERM-006-D-typed-effects-and-resources.md`
- GitHub Issues #56 and #57
- `docs/reference/I243-SHELL-AUTO-CLASSIFIER-THREAT-MATRIX.md`

## Residual Destination

Unknown semantics remain human-required and are tracked as follow-up stories; the classifier must
not silently widen permissions.
