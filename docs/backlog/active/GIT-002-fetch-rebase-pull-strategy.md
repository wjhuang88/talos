# GIT-002: Native Fetch, Rebase and explicit Pull strategies

**Status**: Refinement
**Type**: Product/API/State Story
**Parent Epic**: GIT-001

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | None |
| Governance Claim PR | Pending |
| Authorization Mode | Not applicable |
| Authorization Evidence | User requested implementation; target-branch claim and protected-scope review still required |
| Implementation PR | Not started |
| Last Updated | 2026-11-27 |
| Handoff / Release Condition | Effective claim before implementation; independent permission/security review before merge |

## Goal and scope

Provide independent native Git Fetch and Rebase tools and explicit Pull integration strategy to CLI/runtime/MCP callers. Use configured native Git behavior through prepared whole-command authorization, retained arguments/environment, configuration revalidation, supervised execution and bounded redacted diagnostics. Do not label Fetch read-only: it can update refs, FETCH_HEAD, objects and execute configured helpers.

Proposed interfaces (implementation must publish final schemas):

- `git_fetch`: optional `remote`, `refspecs`, `prune`, `tags`. Omitted values preserve Git configuration. Validate option boundaries and distinguish an omitted boolean from explicit false. Do not update the checked-out branch through an unreviewed refspec.
- `git_rebase`: `action` (`start`, `continue`, `abort`, `skip`), optional `upstream`, `onto`, `branch` for start only. Reject incompatible combinations before spawning. No interactive editor or arbitrary argv interface. `skip` requires explicit invocation; never automatically skip, abort, resolve conflicts or force-push.
- `git_pull`: optional `strategy` (`merge`, `rebase`, `ff_only`). Omission preserves current configured behavior and old JSON requests. Explicit merge must override inherited rebase and ff-only settings; explicit rebase and ff-only must have tested precedence over conflicting config. Do not change persistent/global configuration.

## Acceptance

1. Fetch transfers new objects/remote refs in a real temporary repository while preserving checked-out branch, worktree and index. Cover custom remote, configured default, prune and refspec validation.
2. Rebase rewrites a diverged fixture onto upstream. A conflict returns the native diagnostic and leaves recoverable state. Continue after manual fixture resolution, abort restoring the original branch, and explicit skip each have real-repository tests. Missing/invalid state fails without automatic recovery.
3. Pull covers diverged merge, diverged rebase and ff-only rejection, configured defaults, conflicting settings, and invalid strategy values.
4. Each new operation has its own identity-bound command evidence and permission capability. No reuse of Pull authority for Fetch/Rebase. Denial, stale proof, config/argv changes and cancellation are tested. Native/MCP/Auto-review routing are audited together.
5. Errors retain actionable bounded diagnostics without leaking fixture credentials. Do not silently swallow process spawn/timeout failures.
6. Existing callers and schemas remain compatible; any breaking Rust public struct/API change requires an ADR and migration plan.

## Required Reads

- `docs/sop/REQUIREMENT-INTAKE.md`
- `docs/sop/AGENT-COLLABORATION.md`
- `docs/backlog/active/GIT-001-embedded-git-tools.md`
- `crates/talos-tools/src/git_write.rs`
- `crates/talos-tools/src/git_write/prepared_add/pull_command.rs`
- `crates/talos-tools/src/git_write/prepared_add/command_evidence.rs`
- `crates/talos-permission/src/native_pull_command.rs`
- `crates/talos-agent/src/auto_resolver/prepared_command.rs`

## Validation and documentation

Use pinned Rust, locked focused tests, permission/security review, and the full release preflight. Windows changes require the documented cross-check and separate native acceptance. Update tool catalog/schema snapshots, CLI/runtime/MCP registration tests, user-facing Git tool documentation and GIT-001 inventory. Record residuals here; no completion claim without implementation commits.

## Execution notes

Intake only. No tool implementation delivered. Existing Fetch name in a permission test is not a registered Fetch tool. Existing GIT-001 lists Rebase as future scope. Pull currently accepts only an optional remote. New capabilities must not simply reuse the `git:pull-command` authority.