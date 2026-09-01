# ADR-070: Shell Auto Classifier Context And Precedence

**Status:** Accepted
**Date:** 2026-09-01
**Owners:** PERM-007-F0 / I243; implementation PERM-007-F / I244
**Depends on:** ADR-012, ADR-040, ADR-064, ADR-067, ADR-069

## Context

I241 proved a fail-closed model-assessment seam but admitted only a fixed structural `exec`
allowlist. Routine shell calls such as `ls -la` still prompt because `bash` is excluded. Adding
individual commands or flags cannot deliver useful auto mode and turns policy maintenance into an
ever-growing exception table.

Claude Code's documented auto-mode ordering is a relevant product reference: permission `deny` and
explicit `ask` rules run before a second classifier gate; the classifier evaluates action semantics
against trusted-environment context and hard/soft/allow rules; an optional `classifyAllShell` route
sends every shell call through that classifier. Talos must translate that experience without
copying implementation details or weakening its authoritative permission/admission pipeline.

An AST or static `AccessEvidence` is useful structural evidence but cannot prove the behavior of an
arbitrary executable. Treating syntax classification as a security proof would contradict ADR-012
and ADR-040. Conversely, sending only a digest deprives a model of the semantics needed to judge an
unseen command. The decision must therefore define what exact action context the model may see and
which risks remain deterministic.

Official reference: <https://code.claude.com/docs/en/auto-mode-config>.

## Constraint Decomposition

| Constraint | Type | Source | Consequence |
|---|---|---|---|
| Explicit permission Deny always wins | Hard | AGENTS.md; ADR-067 | Classifier cannot run before or override Deny. |
| Explicit Ask represents requested human checkpoint | Hard | Permission contract | Matching Ask bypasses auto classification. |
| Model cannot bypass sandbox, grants, or admission | Hard | ADR-067/069 | Result is at most one bound `AllowOnce`. |
| Secrets must not enter logs or an unrelated provider | Hard | AGENTS.md; ADR-023 | Preflight rejects known secret-bearing shapes; no cross-provider classifier fallback. |
| Reduce routine shell prompts without command exceptions | Soft | Issue #462 | Route shell actions by default while auto mode is enabled. |
| Model classification of arbitrary program semantics is fallible | Assumption | Model boundary | Hard risks require deterministic rules; uncertainty remains human-required. |

## Proposed Decision

### 1. Precedence

1. Normalize the exact tool request once in the Agent-owned permission pipeline.
2. Apply deterministic `deny`, explicit `ask`, existing human grants, sandbox and admission guards.
3. While auto mode is enabled, route remaining foreground shell requests to an isolated classifier.
4. Apply classifier hard-deny, soft-deny, allow-exception, explicit-user-intent, and uncertainty
   rules in that order.
5. Admit only one digest-bound `AllowOnce`, then recheck the existing revision/admission fence.

No classifier rule or user intent may override deterministic permission Deny, explicit Ask,
sandbox denial, stale revision, missing interaction policy, or admission failure.

### 2. Classifier Context

The classifier is a separate tool-free model request. It receives:

- the exact normalized shell command as untrusted data, plus parsed structural evidence;
- tool surface, cwd class, environment-variable names/digest, background/foreground state, and
  policy/session revisions;
- the current user instruction needed to distinguish explicit intent, not unrestricted transcript
  history or prior model reasoning;
- trusted workspace root identity, configured remotes, and user/managed trusted-environment facts;
- closed hard-deny, soft-deny, and allow-exception rules.

The classifier prompt treats command text and user/project content as data, never instructions. It
has no tools, cannot recurse into permission evaluation, and cannot mutate conversation history.
The classifier uses the active provider trust boundary; it must not fall back to a different
provider. Known credential-bearing fields, raw environment values, tool results, unrelated history,
and secrets are excluded or deterministically escalated before the call.

### 3. Result And Binding

The typed result is `AllowOnce` or `HumanRequired`, with a closed reason/severity code and the exact
request digest. The digest binds normalized command bytes, structural evidence, cwd, environment
identity, permission/mode revisions, session, classifier policy version, and bounded user-intent
identity. Any mutation, expiry, cancellation, timeout, provider error, malformed response, missing
context, or stale revision yields `HumanRequired` or Deny according to authoritative policy.

### 4. Configuration Ownership

Classifier trust and rule configuration may come only from user-global, invocation, or managed
configuration outside the repository-controlled workspace. Repository instructions may tighten
behavior but cannot add trusted destinations or allow exceptions. Defaults remain active unless a
future reviewed configuration explicitly replaces them; replacement must be visible through a
read-only effective-config command.

The initial implementation may ship fixed conservative defaults and the context/binding schema.
Persisted `environment`, `allow`, `soft_deny`, `hard_deny`, setup/critique UI, and organization
management are follow-up scope unless I244 explicitly carries their schema, migration, and UI
acceptance.

### 5. Migration And Rollback

- Existing deterministic Deny/Ask and human grants remain authoritative and serialization-compatible.
- Existing I241 structured `exec` path remains available as a conservative fast path.
- `auto.enabled = false`, `/auto off`, or an opened circuit disables classifier calls and restores
  the existing human-required path without changing permission state.
- Unexpected authorization, classifier prompt injection, secret exposure, or cross-surface
  divergence blocks rollout and triggers rollback to human-required behavior.

## Rejected Alternatives

- **Add commands/flags to an allowlist:** does not scale and does not match the requested experience.
- **Treat AST/access evidence as proof:** syntax cannot establish arbitrary executable effects.
- **Send only a digest/effect label:** the model lacks semantics for unseen commands.
- **Send the full transcript/environment:** unnecessary data exposure and unstable authorization
  identity.
- **Let the classifier create persistent grants:** violates the human-owned grant boundary.

## Required Security Matrix

I243 must record at least: destructive Git/filesystem actions, secret/exfiltration commands,
network and protected targets, privilege/system changes, package/install scripts, pipelines and
substitutions, prompt injection inside arguments, environment/PATH mutation, symlink/path escape,
explicit user intent, stale revisions, timeout/cancellation/provider failure, config injection, and
CLI/TUI/Runtime/MCP equivalence.

## Status And Authorization

This ADR is Accepted through I243 decision review. It changes no production behavior and authorizes
no Rust, Cargo, schema, release, or publication work by itself. I244 has a separate effective
implementation claim and is the only owner for the bounded implementation described in its owner
document.

Acceptance evidence: decision content was present in the pre-existing governance merge
`cc4f22e5`; exact-head CI `33495286016` and independent permission/security/API approval
`5492369576` were bound to the I243 claim candidate. I244 claim PR #465 was subsequently merged at
`94ba2dc5`; implementation remains subject to its exact-head review and closeout gates.
