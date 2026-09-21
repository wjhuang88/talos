# ADR-081: Model Assessment For All Shell Approval Requests

Status: Accepted
Date: 2026-09-21
Owner: I281
Supersedes: ADR-069/070 assessment eligibility for the I281 implementation; other security boundaries remain effective.

## Context

The maintainer requests model-first assessment for bash/PowerShell approval requests, including
compound scripts, with concrete decision points when human judgment is needed. Current
`eligible_bash` rejects mutating/network categories and several context shapes before assessment.
ADR-070 also defines an explicit Ask rule as an intentional human checkpoint. Increasing
assessment coverage and granting execution authority are distinct changes.

## Decision

1. Authoritative Deny rejects without model override. Existing valid grants/Allow do not require
   redundant review. Auto off preserves direct human approval.
2. Auto on sends shell requests reaching approval to the isolated model assessor, including
   complex scripts and explicit Ask rules. Syntax complexity is evidence, not a blanket exclusion.
3. Preserve explicitly configured Ask as a required human checkpoint: model assessment supplies
   impact, risks and decision points but cannot discharge that checkpoint. Default/unmatched Ask
   can receive one model AllowOnce when reviewed effects fit the accepted automatic authority.
   This preserves the meaning of existing user/managed permission rules without migration.
4. Remove command-by-command eligibility as the primary assessment boundary. Separate assessment
   coverage from automatic execution eligibility: available evidence and deterministic security
   constraints control whether model approval may be admitted. No implicit permission to perform
   destructive, credential, privilege or external-data operations merely because they were reviewed.
5. Review actual command/script bytes, arguments, cwd, relevant intent and resolved dependencies.
   Missing/dynamic/remote content and environment-dependent effects must be explicit uncertainties.
   Sensitive content is protected; unavailable safe context causes a visible assessment failure,
   never fabricated model approval. Do not truncate evidence and label it complete.
6. Bind approval to execution, not only a pathname or digest. If mutable scripts/dependencies cannot
   be held or revalidated with a sound execution boundary, automatic execution is not allowed.
   A human click cannot override technical admission or sandbox failures.
7. Use ADR-075 bounded, tool-free model invocation with current provider trust, cancellation and
   independent token/byte limits. Scripts are untrusted data. No recursive tools or hidden history.
8. Return actionable human decision points and effect summaries, not raw reasoning. Distinguish
   assessor failure from a model HumanRequired decision. Diagnostics remain secret-safe.

## Compatibility And Validation

- No silent reinterpretation of configured Ask. Existing Deny, sandbox, grants, admission CAS and
  exactly-once execution remain authoritative; no persistent model-created grants.
- If public assessment/event schemas require changes, document additive paths and external
  exhaustive-match impact before implementation; do not silently break SDK consumers.
- Security matrix must include both shells, scripts/compound commands, writes, network/secret
  effects, prompt injection, unresolved code, content mutation, stale revision, cancellation,
  invalid response, Auto off, configured Ask and valid grants.
- Windows lifecycle test repair is an independent I281 deliverable, not authority to alter
  production timeout or process-hardening semantics.

## Acceptance Gate

Maintainer explicitly accepted ADR-081 in the development conversation on 2026-09-21.
This accepts the distinction between configured human Ask and default Ask, with model assessment
for both and no model override of configured human checkpoints. Independent Agent
permission/security/API review remains required for code. An effective I281 claim is still
required before implementation; no release is authorized.
