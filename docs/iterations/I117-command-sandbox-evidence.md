# Iteration I117: Command Sandbox Evidence

> Document status: Partial (2026-07-12) — evidence API and classifier delivered as diagnostic-only; not yet wired into bash/exec execution pipeline; formal security sign-off remains
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
| 2026-07-12 | I117 activated | I116 Complete; I117 activated. ADR-040 accepted as the PERM-005 design gate. Execution from LT020-LT024. |
| 2026-07-12 | LT020 complete | ADR-040 (Command Access Evidence and Logical Sandbox Enforcement) written. Defines declared/observed/unknown access, canonical root enforcement, Deny precedence, and safe-fallback-to-strict behavior. NOTE: formal production security sign-off remains a separate gate before any deployment broadening bash/exec trust. The implementation deliberately follows the conservative fallback: no bash/exec broadening, diagnostics/revoke only, evidence-based read-only enforcement with path traversal protection. |
| 2026-07-12 | LT021 complete | `AccessEvidence` type with `AccessKind` (Read/Write/Delete/Spawn/Network/Unknown) and `EvidenceState` (Declared/Observed/Unknown) implemented in `crates/talos-permission/src/access_evidence.rs`. Serializable; 19 unit tests including serialization, classification, and repo-local checks. |
| 2026-07-12 | LT022 complete | `PermissionEngine::evaluate_command_with_evidence()` enforces: Deny always wins; Unknown/Spawn/Network/Delete/Write never inherits trust; only Declared Read with repo-local paths may proceed under trust; out-of-repo escalates to Ask. 8 security tests cover traversal, pipe, symlink-equivalent, child-process, unknown-access, deny-precedence, and non-Git strictness. |
| 2026-07-12 | LT023 complete | `talos permissions trust status` shows workspace trust state (Git detection, trust active, trust effect, ADR references). `talos permissions trust revoke` removes trust with cross-process persistence test. CLI smoke verified. |
| 2026-07-12 | LT024 closeout | 92 permission tests pass (including 27 access-evidence/security tests). Release preflight, governance validation, and diff check all clean. ADR-040 documents the OS-sandbox limitation residual. bash/exec remains per-command Ask/Deny unless structural evidence proves repo-local read; unknown/out-of-repo never inherits trust. |
