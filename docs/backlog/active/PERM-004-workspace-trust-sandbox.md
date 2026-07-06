# PERM-004: Workspace Trust Sandbox

| Field | Value |
|---|---|
| Story ID | PERM-004 |
| Priority | P1 |
| Status | Planned — ADR required |
| Source | [GitHub Issue #22](https://github.com/wjhuang88/talos/issues/22) |
| Depends On | `PERM-002`, `PERM-003`, `TOOL-017` |

## Problem

Permission prompts remain too noisy for normal development, but broad permanent approvals are unsafe.
The desired model is coarse-grained trust inside a normalized workspace boundary and strict,
non-persistent approval outside that boundary.

## Acceptance

- Produce an ADR before implementation, covering workspace root selection, path canonicalization,
  symlink/`..` escape handling, trust persistence, deny precedence, and out-of-workspace behavior.
- Define directory-scoped write approvals for workspace paths.
- Keep out-of-workspace write/execute/network approvals strict and non-persistent unless explicitly
  configured by a reviewed policy.
- Define how bash/exec command permissions compose with file-resource permissions.
- Include tests for workspace boundary traversal and deny-rule precedence.

## Non-Goals

- No permission-default relaxation before ADR acceptance.
- No silent trust of the entire home directory.
- No bypass of the existing permission pipeline for write-capable tools.

## Required Reads

- `docs/backlog/active/PERM-002-operation-scoped-permissions.md`
- `docs/backlog/active/PERM-003-permission-experience-reference-study.md`
- `crates/talos-permission/src/lib.rs`
- `crates/talos-cli/src/approval.rs`
- `crates/talos-tools/src/bash_tool.rs`

