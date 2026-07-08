# PERM-005: Logical Tool Sandbox Enforcement

| Field | Value |
|---|---|
| Story ID | PERM-005 |
| Priority | P1 |
| Status | Planned — ADR and security review required |
| Source | Maintainer request 2026-07-08 — strengthen repo-sandbox permissions beyond policy-only checks |
| Depends On | `PERM-004`, `TOOL-016`, `TOOL-017`, `VALIDATION-001` |

## Problem

PERM-004 can reduce repeated prompts by treating a detected Git repository as a coarse workspace
sandbox after explicit approval. That is still only a policy boundary unless Talos can understand
what a tool actually touches.

The highest-risk gap is command execution. `bash` and direct `exec` can read, write, delete, or
spawn child processes that touch paths not obvious from the command line. A repo-scoped approval is
not credible unless Talos can either observe touched files/directories or enforce a sandbox boundary.

## Goal

Strengthen the current logical sandbox so tool execution can prove or enforce where it operated.
The first target is repository-local command execution: if a Git workspace is approved as a
repo-sandbox, bash/exec should be able to show the user what paths were touched and reject or
escalate operations that cross the canonical repo root.

This story also prepares a future lightweight real sandbox, but does not select a specific OS-level
technology yet.

## Scope

- Define a tool-access evidence model for path reads, writes, deletes, directory traversal, process
  spawn, network intent, and unknown/unobservable access.
- Add observability for `bash` and direct `exec` commands where the platform can expose touched
  paths without unsafe broadening.
- Treat unobservable access as a security-relevant state: either require stricter approval, deny
  under repo-sandbox mode, or record an explicit diagnostic.
- Enforce canonical repo-root boundaries for repo-sandbox mode, including symlink and `..` escapes.
- Preserve exact-command and command-template approvals for non-Git workspaces and commands that
  cannot be bounded.
- Evaluate a future lightweight sandbox layer as a follow-up design option after logical evidence
  is in place.

## Acceptance

- An ADR or security review records the chosen logical sandbox design and its platform limitations
  before implementation.
- `bash` and direct `exec` result evidence can report touched repo-local paths or mark access as
  unknown/unobservable.
- Repo-sandbox mode rejects or escalates commands that touch paths outside the canonical Git root.
- Non-Git workspaces keep the stricter PERM-002/PERM-003 behavior and do not receive repo-wide
  sandbox trust.
- Tests cover symlink escape, parent traversal, child-process writes, unknown access, denied
  out-of-repo access, and Deny precedence.
- Documentation states clearly that logical sandbox evidence is not the same as a real OS sandbox.

## Non-Goals

- No permission-default relaxation before PERM-004 is accepted.
- No global `bash = Allow` or unrestricted `exec`.
- No push, publish, release, credential, or network gate relaxation.
- No OS-level sandbox dependency without ADR, owner review, and platform fallback design.

## Required Reads

- `docs/backlog/active/PERM-004-workspace-trust-sandbox.md`
- `docs/backlog/active/PERM-003-permission-experience-reference-study.md`
- `docs/backlog/active/TOOL-016-direct-exec-tool.md`
- `docs/backlog/active/TOOL-017-exec-multi-parallel-pipe.md`
- `docs/backlog/active/VALIDATION-001-internal-validation-service.md`
- `crates/talos-tools/src/bash_tool.rs`
- `crates/talos-tools/src/exec_tool.rs`
- `crates/talos-permission/src/lib.rs`
- `crates/talos-sandbox/`
