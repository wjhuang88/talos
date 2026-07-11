# Iteration I117: Command Sandbox Evidence

> Document status: Planned
> Published plan date: 2026-07-12
> Planned objective: Close the PERM-005 command-execution evidence gap while preserving strict
> behavior for unknown, out-of-repo, network, credential, and destructive access.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: a trusted Git workspace can report/revoke trust and execute only a provably
> bounded command path; unknown or escaping bash/exec access escalates or denies.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| N110 | PERM-005 | Planned, ADR required | ADR-038/PERM-004 first slice | Accepted command-sandbox ADR/security review |
| N111 | PERM-005 | Refinement | N110 | Typed access evidence with unknown state |
| N112 | PERM-005 | Refinement | N111 | Repo-boundary enforcement or strict fallback |
| N113 | PERM-004 residual | Refinement | Existing trust store | Explicit trust status/revoke UX |
| N114 | Month-2 closeout | Planned | N110-N113 | Security evidence and limitations published |

### Scope

- Model declared/observed/unknown path and process/network access without treating evidence as
  permission authority.
- Enforce canonical repo boundaries for any trust-assisted execution.
- Preserve Deny precedence and non-Git strict behavior.

### Non-Goals

- Global bash/exec Allow, network/push/publish/release trust, hidden auto-approval, or an OS sandbox
  dependency without a separate accepted ADR.

### Acceptance

- Given trusted workspace mode, when access is unknown or outside the canonical repo, then the
  command is never silently auto-approved.
- Given an explicit Deny, when trust or evidence would otherwise allow the operation, then Deny wins.
- Given persisted trust, when the operator revokes it, then subsequent writes/commands return to
  strict behavior across a new process.

### Planned Validation

- `./scripts/release_preflight.sh`
- `cargo test --locked -p talos-permission -p talos-tools -p talos-cli`
- symlink, traversal, child-process, unknown-access, Deny-precedence, and revoke tests
- sandbox security review against AGENTS hard constraints

### Documentation To Update

- PERM-004, PERM-005, ADR index, permission README/help, Board/backlog

### Risks And Rollback

- Risk: portable touched-path observation is incomplete or misleading.
- Rollback: ship status/revoke and evidence diagnostics only; keep bash/exec per-command Ask/Deny.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-12 | Planning | Published as Month 2 shell; activation waits for I116 Complete and PERM-005 readiness review. |
