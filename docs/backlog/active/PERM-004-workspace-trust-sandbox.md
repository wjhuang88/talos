# PERM-004: Workspace Trust Sandbox

| Field | Value |
|---|---|
| Story ID | PERM-004 |
| Priority | P1 |
| Status | Complete — ADR-038 file-write trust delivered; ADR-040 command access evidence is diagnostic-only and never changes permission decisions (I117, 2026-07-12) |
| Source | [GitHub Issue #22](https://github.com/wjhuang88/talos/issues/22) |
| Depends On | `PERM-002`, `PERM-003`, `TOOL-017`, `VALIDATION-001` |

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
