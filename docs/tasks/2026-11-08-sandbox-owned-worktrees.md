# WORKTREE-SANDBOX-001: Independent OS-sandbox worktree admission

Status: Planned / Unclaimed
Owner: @wjhuang88
Source: explicit user scope decision on 2026-11-08; split from [WORKTREE-001](2026-11-08-session-owned-worktrees.md).

## Runnable deliverable

A worktree created by an authorized sandbox-owned Git operation becomes available
inside that same logical Session's sandbox and permission pipeline. The host must
not substitute unsandboxed execution, scan unrelated roots or infer custody from stdout.
Creation receipts, sandbox mount/capability publication and current permission revisions
must describe the same operation. Resume revalidates identities; replacement revokes
membership and invalidates old automatic/manual authorizations.

## Acceptance

Test real supported OS sandbox creation, immediate native read/write/edit/search,
Session resume, failed/forged creation, sibling isolation, symlink/reparse replacement,
stale approvals and explicit Ask/Deny. Run locked workspace validation, cargo-xwin and
native platform sandbox gates plus independent security review. Update README sandbox
behavior and ADR-099. Establish an effective collaboration claim before implementation.

## Current boundary

No implementation is claimed. WORKTREE-001 keeps sandbox dispatch authoritative;
sandboxed shell output cannot mint host creation membership. This task must not delay
the separately authorized CLI/TUI host repair.
