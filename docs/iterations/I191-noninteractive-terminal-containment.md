# Iteration I191: Non-Interactive Terminal Containment

> Document status: Active
> Published plan date: 2026-08-12
> Planned objective: contain command-tool children outside Talos's interactive terminal boundary.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: password-prompting and interactive commands cannot consume or corrupt TUI input.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Codex / GPT-5.6 emergency implementation session 2026-08-12 |
| Work Slice | TOOL-026 only: default-null stdin and Unix controlling-terminal detachment for foreground bash/exec children, tests and ADR-007 update. |
| Claimed At | Not applicable |
| Source Issue | Maintainer incident report, 2026-08-12 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Emergency override |
| Authorization Evidence | @wjhuang88 instructed immediate handling after reporting live TUI corruption from interactive scripts and password prompts. |
| Implementation PR | Not started |
| Last Updated | 2026-08-12 |
| Handoff / Release Condition | Reconcile an effective claim and derived state within two business days; exact-head independent security review remains mandatory before merge. |

## Emergency Override Record

- Authorizing maintainer: `@wjhuang88`.
- Reason: live terminal-input corruption can misroute password input and terminal control replies.
- Exact scope: TOOL-026 only; I188/I189 remain Planned and unactivated.
- Branch: `fix/i191-i192-emergency-terminal-session` from main `6c7e11cc44fdd8c7b48a2d2bf6d5438db036f432`.
- Validation: focused regressions and locked workspace preflight are required; independent
  natural-person exact-head security review is deferred until an implementation head exists.
- Rollback/containment: revert the isolated process-boundary implementation; no persistent data
  migration is involved.

## Published Baseline

The frozen scope, exclusions and acceptance are owned by `TOOL-026`.

## Non-Terminal Inventory

- I188 and I189 remain Planned/Claimed and are not activated.
- I159-I162 remain Blocked; I164 remains Paused.
- SESSION-008-B remains Ready/Unclaimed and RUNTIME-005 remains blocked on it.
- Issues #45, #49 and #59 remain open; archival PRs #120/#121 and unrelated PR #198 are untouched.

## Completion Evidence

- No Completion Commit yet; retain Active until implementation and review evidence exist.
