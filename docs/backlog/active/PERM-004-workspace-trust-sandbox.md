# PERM-004: Workspace Trust Sandbox

| Field | Value |
|---|---|
| Story ID | PERM-004 |
| Priority | P1 |
| Status | Cancelled / Superseded — original umbrella replaced by the delivered slices and explicit dispositions below; not a full sandbox completion claim |
| Source | [GitHub Issue #22](https://github.com/wjhuang88/talos/issues/22) |
| Depends On | `PERM-002`, `PERM-003`, `TOOL-017`, `VALIDATION-001` |


**Status**: Cancelled / Superseded (2026-11-07)

## Current disposition — 2026-11-07

The maintainer requested reconciliation and closure of Issue #22 after the old Partial summary
was found to omit subsequent delivery. This governance-only closure retires the original umbrella;
it does not declare every original proposal implemented or relax any permission/security boundary.
The original problem, acceptance and historical implementation report remain below as the published
baseline. Their future-tense/open-Issue statements are historical, superseded by this section.

| Original requirement / question | Current implementation or decision | Disposition |
|---|---|---|
| Workspace boundary, persistence, Deny precedence, non-Git behavior | ADR-038; `WorkspaceTrustStore`, `is_git_workspace`, `is_within_repo`; repo-contained Write trust in the permission engine; I112/T121 implementation `050c132ddf1c32c66826f22df82b15e203a44100` | Bounded logical file-write trust delivered; no blanket command or network trust. |
| Relative traversal / symlink handling | Permission engine resolves relative paths and canonicalizes existing paths; file authorization canonicalizes nearest existing ancestors and revalidates paths under ADR-047 | The original claim that all relative paths are directly allowed is obsolete. This is not a proof of universal TOCTOU exclusion. |
| External paths must never reuse approval | ADR-047 / [SEC-001](SEC-001-external-path-authorization.md), implementation `fc424edbe752bbe46b77668cd1e39fd1fae157aa`; `exact_session_grant_is_reused_for_external_path` regression | Superseded by explicit exact-path, operation/lifetime-bound approval, with Deny precedence; not arbitrary external or cross-session permission. |
| Trust management and reduced prompts | `talos permissions trust` status/revoke, CLI `--trust`, persistent trust store; I117 / ADR-040 | Delivered CLI workflow replaces the proposed `/trust` spelling; no claim that a `/trust` slash command shipped. |
| Bash/exec, Git and network workflow | [I308](../../iterations/I308-builtin-auto-review.md), PR #809 implementation `5dea4925a143151071ea231779c100a75734e6fe`; PERM-007-G/H shared Auto review and real operation fixtures | Reviewed one-use admission is delivered; it is not repo-wide permanent command trust. Strict denial, uncertainty, freshness, cancellation and higher-risk gates are intended behavior, not automatically unfinished features. |
| Approval/grant/execution composition | PERM-006-A/B/C, including agent-owned pipeline implementation `49d1546c3748930177655dbedc7f3665780d92ab` | Delivered slices; [PERM-006-D](PERM-006-D-typed-effects-and-resources.md) / #56 and [PERM-006-E](PERM-006-E-cross-surface-conformance.md) / #57 retain their own unchanged unfinished scope. |
| Actual arbitrary-command OS confinement | ADR-038/040 and [PERM-005](PERM-005-logical-tool-sandbox-enforcement.md) distinguish diagnostic evidence from enforcement | Not delivered as a universal guarantee. Deferred design; this umbrella is retired rather than declared fully implemented. See retained design options below. |

## Retained design options (not implemented, not activated)

This owner remains the durable destination for the following original ideas. They are explicitly
deferred, not silently transferred into completed PERM-005 or into PERM-006-D/E:

- Repository discovery from subdirectories/submodules: the trust helper still checks `.git` at the
  supplied root; it does not implement ADR-038's proposed `gix::Repository::discover()` selection.
  A future change must reconcile that ADR and positively establish the trust root before broadening it.
- A dedicated directory/file-extension policy DSL and generic one-click command sandbox trust:
  existing rule/grant mechanisms are not evidence that these proposed UX/DSL forms shipped.
- Uniform OS enforcement of arbitrary commands and children, including platform availability and
  fallback guarantees: diagnostic unknown-access evidence and Auto assessment do not provide this.
- The original proposal's system-path defaults and all-operation audit guarantee are not certified
  by this reconciliation; any stricter new defaults or universal audit contract need explicit
  requirements, independent security review and their own runnable acceptance.

Reactivation trigger: a maintainer selects one of these concrete behaviors. Create a separately
claimed executable Story/iteration linked here, with ADR/security review and platform tests.
No new product requirement, implementation branch or relaxed policy is authorized by this closure.

## Closure ledger and validation

- Outcome: close obsolete #22 by supersession, not by asserting complete original implementation.
- Artifacts: this owner, Product Backlog, Board and current Issue reconciliation matrix; historical
  snapshots and original ADR text are preserved.
- Evidence: source/test inspection against `main@4685c2979`; implementation SHAs above are already
  merged. I308 owner records exact-head CI `41007199388` and independent review of PR #809.
- Validation: both repository governance validators, skill governance validator, diff check and
  independent exact-head governance/security-boundary review are required before merge. No Rust
  code changes or new runtime test results are claimed by this documentation-only reconciliation.
- Remaining delivery: merge this governance record, then close #22 as superseded and synchronize
  its remote disposition. Existing #56/#57 scope and unactivated design options remain separate.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 |
| Work Slice | Governance-only reconciliation and supersession of PERM-004 / #22; no runtime or policy changes |
| Claimed At | 2026-11-07 |
| Source Issue | #22 |
| Governance Claim PR | #867 |
| Authorization Mode | Independent review |
| Authorization Evidence | Maintainer requested closure reconciliation; exact-head independent review and CI required before merge |
| Implementation PR | None |
| Last Updated | 2026-11-07 |
| Handoff / Release Condition | Governance-only terminal record effective on target-branch merge; close #22 after merge; retained options require fresh selection |

## Problem

Permission prompts remain too noisy for normal development, but broad permanent approvals are unsafe.
The desired model is coarse-grained trust inside a normalized workspace boundary and strict,
non-persistent approval outside that boundary.

`VALIDATION-001` already owns project-type detection and can distinguish whether a workspace is a
Git repository. PERM-004 should use that signal when selecting the workspace trust mode:

- **Non-Git workspace:** keep the current stricter permission mode. Approvals remain
  operation-scoped or directory-scoped, and Talos must not infer a repository-sized trust boundary.
- **Git workspace:** after explicit user approval, Talos may treat the Git repository root as the
  workspace sandbox boundary. Repo-internal file operations may use a coarse repo-scoped sandbox
  approval instead of repeated per-file prompts, similar to Codex-style workspace trust.
- **Always strict outside the repo:** paths outside the canonical Git root, symlink escapes,
  parent-directory traversal, credentials, network, publish/release/push, destructive cleanup, and
  configured Deny rules remain separately gated. Repo-sandbox trust is not a global `bash = Allow`
  and does not bypass the permission pipeline.

The repo-sandbox mode is a policy design target, not an implementation authorization. It requires
the ADR, tests, and `PERM-005` sandbox-observability follow-up before broad bash/exec relaxation.

## Implementation Status (2026-07-11 Reconciliation)

ADR-038 is accepted and I112/T121 delivered the first bounded slice:

- `WorkspaceTrustStore` persists explicit trust decisions by canonical workspace path.
- CLI `--trust` grants trust only when a Git workspace is detected.
- `PermissionEngine` applies trust only to repo-contained `Write` facets.
- Explicit Deny rules remain authoritative; non-Git and out-of-repo operations keep strict policy.
- Boundary, traversal, persistence, and Deny-precedence tests cover the delivered slice.

This does not complete the broader sandbox objective. Git detection currently uses the bounded
workspace `.git` check rather than claiming arbitrary repository discovery, and bash/exec remains
per-command because Talos cannot yet prove touched paths, child-process access, or unknown access.
Those security requirements belong to PERM-005. No future agent may interpret this first slice as
repo-wide `bash`/`exec` permission.

GitHub Issue #22 remains open by design. It owns the broader workspace-trust and sandbox-policy
residuals; PERM-006 owns pipeline convergence and must preserve this Story's strict external-path,
Deny-precedence, and no-automatic-broadening constraints.

## Acceptance

- Produce an ADR before implementation, covering workspace root selection, path canonicalization,
  symlink/`..` escape handling, trust persistence, deny precedence, and out-of-workspace behavior.
- Use `VALIDATION-001` project-type detection before selecting a trust mode: non-Git workspaces use
  the stricter existing permission behavior; Git workspaces may opt into canonical repo-root
  sandbox trust after explicit user approval.
- Define directory-scoped write approvals for workspace paths.
- Define repo-scoped sandbox approvals for Git workspaces without weakening high-risk gates for
  network, credentials, push/publish/release, destructive cleanup, or out-of-repo access.
- Keep out-of-workspace write/execute/network approvals strict and non-persistent unless explicitly
  configured by a reviewed policy.
- Define how bash/exec command permissions compose with file-resource permissions.
- Defer broad bash/exec repo-sandbox execution until `PERM-005` can observe or enforce touched
  files/directories for command execution.
- Include tests for workspace boundary traversal and deny-rule precedence.

## Non-Goals

- No permission-default relaxation before ADR acceptance.
- No silent trust of the entire home directory.
- No bypass of the existing permission pipeline for write-capable tools.
- No repo-sandbox trust for a directory that has not been positively detected as a Git repository.
- No claim that logical permission checks are a process sandbox; real enforcement is tracked by
  `PERM-005`.

## Required Reads

- `docs/backlog/active/PERM-002-operation-scoped-permissions.md`
- `docs/backlog/active/PERM-003-permission-experience-reference-study.md`
- `docs/backlog/active/PERM-005-logical-tool-sandbox-enforcement.md`
- `docs/backlog/active/VALIDATION-001-internal-validation-service.md`
- `crates/talos-permission/src/lib.rs`
- `crates/talos-cli/src/approval.rs`
- `crates/talos-tools/src/bash_tool.rs`
