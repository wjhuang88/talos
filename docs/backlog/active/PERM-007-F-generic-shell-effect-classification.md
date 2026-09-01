# PERM-007-F: Generic Shell Effect Classification

| Field | Value |
|---|---|
| Story ID | PERM-007-F |
| Type | Permission / Security / Runtime Story |
| Priority | P0 |
| Status | Review / Claimed (local stable candidate converged; remote evidence pending) |
| Parent Epic | PERM-007 (closed; this is a separately governed follow-up) |
| Source | GitHub Issue #462 |
| Selected Iteration | I244 |
| Depends On | PERM-007-F0 / I243, ADR-070; reconcile PERM-006-D / Issue #56 and PERM-006-E / Issue #57 authority |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline implementation session 2026-09-01 |
| Work Slice | Generic model-first shell effect classification after deterministic deny/ask, isolated tool-free context, exact action/cwd/environment/revision binding, fail-closed fallback, and CLI/TUI/Runtime/MCP equivalence. No per-command exception table; no PERM-006-D/E authority, Dashboard, Desktop, release, or publication. |
| Claimed At | 2026-09-01 |
| Source Issue | #462 |
| Governance Claim PR | #465 |
| Authorization Mode | Independent review |
| Authorization Evidence | ADR-070 Accepted through I243 closeout `be4fbcfc`; claim PR #465 merged at `94ba2dc5`, with exact-head CI `33513662235`, independent review approval and merge-time CAS. Maintainer direction requests Claude-like generic model classification rather than command-by-command exceptions. Independent permission/security/API review, exact-head CI and governance validators remain mandatory. Shared GitHub identity provides Agent-role separation only, not natural-person identity separation. |
| Implementation PR | Not started |
| Last Updated | 2026-09-02 |
| Handoff / Release Condition | Claim PR #465 is effective on main; implementation starts from `94ba2dc5` or later and closeout requires exact-head implementation CI and independent permission/security/API review. |

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

## 2026-09-02 Implementation Checkpoint

The local stable candidate now distinguishes explicit/configured Ask from default Ask using the
same authoritative evaluation report, sends a bounded tool-free classifier request for eligible
foreground shell actions, and rechecks exact context plus the existing revision/admission fence
before execution. Known write/mutation and package/network classes, secrets, composition, non-read
effects, timeout, malformed output and stale context remain human-required or denied. Configured
remote trust is deliberately reported unavailable/empty in this fixed conservative policy version;
network effects cannot be auto-approved.

Open architecture Issues #466/#467 have no effective owner/iteration/claim and are not imported into
this Story. Twenty-two classifier tests, 12 permission-pipeline tests, locked Clippy, full workspace
tests, release preflight and both governance validators pass locally. Real TUI acceptance,
exact-head CI and independent permission/security/API review remain required. Status stays
Review/Claimed and `Implementation PR` remains Not started until the complete local candidate is
pushed once.
