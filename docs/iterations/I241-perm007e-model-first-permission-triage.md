# Iteration I241: Model-First Permission Triage

> Document status: Refinement / Unclaimed
> Planned objective: establish and implement a new, independently reviewed model-first permission
> triage contract that reduces routine approval prompts without blanket shell auto-approval.
> MVP deliverable: a runnable normalized-request matrix proves bounded low-risk shell/read/validation
> requests can receive one-time model assistance while every excluded or uncertain request remains
> human-required or denied.

## Selected Story

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| PERM-007-E | PERM-007 | Refinement / Unclaimed | PERM-006-C; PERM-007-D; ADR-064 | Model-first triage with constrained shell/exec coverage and fail-closed evidence |

## Governance Gate

This iteration is not active and grants no implementation authority. Before activation, complete
requirement intake, accept ADR-069 (a superseding follow-up boundary), define the normalized shell
schema and threat matrix, then establish an effective Collaboration Claim.

Required decision read: `docs/decisions/069-model-first-permission-triage.md`.

## Scope And Exclusions

Use the PERM-007-E owner as the normative scope. Exclude blanket shell approval, destructive/network
operations, secrets, script interpreters, pipes/redirection/substitution, background execution,
sandbox expansion, Desktop, release and publication.

## Acceptance And Validation

- deterministic classifier runs before model assessment;
- model input is redacted and structurally normalized;
- valid low-risk results admit only one `AllowOnce`;
- stale, ambiguous, malformed, failed or timed-out results fail closed;
- CLI/TUI/Runtime/MCP semantics remain equivalent where applicable;
- focused adversarial tests, workspace locked checks, governance validators and independent security
  review pass at exact head.

## Next Step

Resolve ADR-069 and its security contract first. Do not create an implementation branch or modify production
permission code until the claim is effective on `main`.
