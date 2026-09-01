# Iteration I244: Shell Auto Classifier Implementation

> Document status: Active / Claimed (proposed; ineffective until claim PR #465 merges)
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
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline implementation session 2026-09-01 |
| Work Slice | Generic model-first shell effect classification after deterministic deny/ask, isolated tool-free context, exact action/cwd/environment/revision binding, fail-closed fallback, and CLI/TUI/Runtime/MCP equivalence. No per-command exception table; no PERM-006-D/E authority, Dashboard, Desktop, release, or publication. |
| Claimed At | 2026-09-01 |
| Source Issue | #462 |
| Governance Claim PR | #465 |
| Authorization Mode | Independent review |
| Authorization Evidence | ADR-070 Accepted through I243 closeout `be4fbcfc`; maintainer direction requests Claude-like generic model classification rather than command-by-command exceptions. Independent permission/security/API review, exact-head CI, governance validators and merge-time CAS remain mandatory; claim is ineffective until #465 merges. Shared GitHub identity provides Agent-role separation only, not natural-person identity separation. |
| Implementation PR | None |
| Last Updated | 2026-09-01 |
| Handoff / Release Condition | ADR-070 accepted and a separate implementation claim effective on main. |

The proposed Active/Claimed state becomes effective only when finalized claim PR #465 merges to
`main`; implementation must start from that merge or a later target-branch commit.

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

## Claim Preparation

This governance branch prepares an atomic I244 claim after ADR-070 acceptance. The claim remains
pending until the Draft PR receives its number and the finalized exact-head record is reviewed and
merged; no implementation authority exists on this branch.
