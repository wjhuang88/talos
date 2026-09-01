# Iteration I244: Shell Auto Classifier Implementation

> Document status: Blocked / Unclaimed
> Planned objective: implement the accepted ADR-070 classifier contract so routine shell commands
> are model-triaged without per-command auto-approval exceptions.
> MVP deliverable: a runnable TUI/CLI flow in which `bash` command `ls -la` reaches the isolated
> classifier and can receive one `AllowOnce`, while unsafe or uncertain actions remain blocked or
> human-required.

## Selected Story

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| PERM-007-F | PERM-007 / Issue #462 | Refinement / Unclaimed | I243 / ADR-070 | Claude-like shell classifier experience with exact binding and fail-closed evidence |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned; blocked pending ADR-070 |
| Claimed At | Not applicable |
| Source Issue | #462 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | None |
| Last Updated | 2026-09-01 |
| Handoff / Release Condition | ADR-070 accepted and a separate implementation claim effective on main. |

## Planned Scope

- Route shell permission requests through the classifier after deterministic deny/explicit-ask
  evaluation while auto mode is active.
- Build an isolated, tool-free classifier request from the exact normalized action, current bounded
  user intent, trusted workspace/remotes/environment context, policy/session revisions, cwd, and
  environment identity.
- Apply accepted hard-deny, soft-deny, allow-exception, explicit-intent, and unknown-result
  precedence.
- Use parser/AST and existing `AccessEvidence` as advisory structural evidence, not per-command
  authorization or a safety proof.
- Do not add or modify authoritative public typed-effect/resource APIs owned by PERM-006-D / Issue
  #56; do not claim closure of PERM-006-E / Issue #57.
- Admit at most `AllowOnce`; recheck the authoritative permission/admission fence before execution.
- Preserve CLI, TUI, embedded Runtime, and MCP permission semantics and provide concise fallback
  reasons when human action is required.

## Exclusions

No blanket shell approval, permanent grants, classifier tool calls, repository-controlled trust
configuration, policy/sandbox override, unrestricted secrets/environment disclosure, Desktop,
release, or publication.

## Acceptance And Validation

- `ls -la` reaches model classification without adding an `ls -la` special case.
- Previously unseen commands can be classified from semantics and context rather than a command
  allowlist; unknown semantics remain human-required.
- Destructive, exfiltrating, privileged, external-target, secret-bearing, and protected-environment
  fixtures are not auto-approved.
- Exact action/cwd/environment/revision binding rejects mutation between assessment and admission.
- Model timeout/error/malformed response/cancellation and lost context fail closed.
- Focused adversarial tests, locked workspace checks, cross-surface tests, governance validators,
  real TUI acceptance, and independent permission/security/API review pass at exact head.

## User-Facing Documentation

- Update README/config reference for auto classifier behavior, fallback and rollback.
- Update `/auto` help and permission UI copy so automatic allow, human-required and hard block are
  distinguishable without exposing model reasoning or secrets.

## Next Step

Wait for I243 / ADR-070 acceptance, then establish a separate effective implementation claim from
the current target branch.
