# WORKTREE-001: Session-owned worktree workspace admission

> Document status: Active / Claimed (proposed; effective only after claim merge)
> Deliverable: a worktree verifiably created by the current Session becomes an additional workspace root for that Session without a second workspace-admission prompt.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / gpt-6 |
| Work Slice | Verified Session-owned worktree creation receipts, additional workspace capabilities, resume revalidation, tool/permission integration and adversarial tests |
| Claimed At | 2026-11-08 |
| Source Issue | None |
| Governance Claim PR | Pending |
| Authorization Mode | Independent review |
| Authorization Evidence | User requests automatic workspace admission for worktrees created by the Session; independent security design review identifies receipt and capability prerequisites |
| Implementation PR | Not started |
| Last Updated | 2026-11-08 |
| Handoff / Release Condition | Reviewed ADR and effective implementation claim; full preflight, cargo-xwin and native Windows identity checks; independent security review |

## Problem and evidence

The user's terminal shows successful creation of a sibling worktree followed by
`write` failing with `path escapes workspace root`, even with an always-approved
write rule. Current PermissionEngine, ManagedWorkspaceLease and file tools bind a
single initial root. No Session-owned worktree registration connects creation to
these boundaries. A broad tool Allow is not workspace admission.

Required Reads: ADR-064, ADR-089, docs/sop/AGENT-COLLABORATION.md,
docs/sop/LOCAL-DEV.md, docs/sop/TESTING.md; talos-permission lib.rs/session.rs;
talos-agent auto_resolver.rs/tool_execution.rs; talos-tools file_tools.rs,
file_tools/atomic_create.rs and bash_tool.rs; CLI/runtime Session composition and
resume. Affected user documentation: README workspace/permission behavior.

## Contract

- After authorized creation actually succeeds, admit only the canonical worktree
  root to the originating Session. The creating operation remains permission-gated.
- Registration is workspace membership, not blanket Allow: explicit Deny, sensitive
  paths, ordinary Ask, capability restrictions and sandbox boundaries remain active.
- A native execution receipt binds logical Session, invocation, source repository,
  target directory identity and verified bidirectional Git worktree metadata.
  Model output, transcript text, tool stdout and hook-rewritten results are not receipts.
- Shell-created worktrees are included in the requested behavior. Define and test
  deterministic supported invocation syntax; do not infer authority from substring
  matching or a before/after worktree-list scan. Ambiguous shell effects need a
  structured creation path or separate exact-root admission, not silent broad trust.
- Multiple roots must flow consistently through permission evaluation, automatic
  assessment leases, exact path authorizations, file-tool capabilities and sandbox
  construction. Adding or revoking a root advances workspace revision and invalidates
  prior assessments. Do not grant the shared parent directory.
- Persist creation evidence scoped to the logical Session. On `-c`, revalidate
  repository and directory identities before rebinding the new permission Session.
  Unrelated Sessions and pre-existing worktrees do not inherit membership.
- Revalidate before sensitive use. Removal, relocation, identity replacement,
  symlink/reparse redirection or invalid Git linkage revokes membership. A replacement
  directory at the same spelling must never inherit authorization.

## Acceptance and validation

Real CLI regression: authorized shell worktree creation followed immediately by
native read/write/edit/search succeeds within the registered root under the existing
policy, without changing workspace or granting a second directory approval.
Then resume the same logical Session and repeat after identity validation.

Negative tests: failed/partial creation; pre-existing target; concurrent external
creation; forged stdout or hook result; another repository or Session; explicit Deny;
credential/VCS-internal protection; symlink/reparse escape; directory deletion and
recreation; invalid metadata; stale workspace revision; ambiguous shell invocation.
Root capabilities and path identity require native Windows execution as well as
`cargo xwin ... --locked --target x86_64-pc-windows-msvc --tests` checks.
Run pinned `./scripts/release_preflight.sh`, exact-head CI and independent security
review before merge. No unsafe or new native dependency without an approved ADR.

## Design review and outstanding work

Independent Agent security design review found that changing only PermissionEngine
or scanning Git worktrees is insufficient. There are three enforcement layers and
shell success alone does not prove creation ownership. The reviewed recommendation
is trusted creation receipts plus per-root capabilities and resume revalidation.
This is design advice, not approval of implementation or a completed repair.

Before activation, settle the bounded shell syntax, atomic creation/identity custody
on Unix and Windows, persistence schema and public API compatibility in an ADR.
Prefer an existing audited host process boundary; document any host-git dependency,
unavailable-host behavior and replacement trigger. Do not implement broad directory
allowlisting as an interim fix. This owner is the residual destination.

## Scope inventory

I310 Active; I318/I302/I277 Review; I249 Planned/deferred; I164 Paused retained.
I162 Complete with historical Review outcome is not reopened. No Blocked owner
selected. Open #838/#849/#861 retain their existing scopes. BG-CONTINUE-001 owns
background completion continuation separately; its files and worktree are untouched.
No permission expansion or production code has been delivered by this intake record.

## Execution plan and business failure regression

User explicitly requested implementation and reported that external-workspace writes
have no human approval opportunity. Treat this as a workflow-blocking defect.
First reproduce the real Agent → auto resolver → human resolver → file tool chain.
An external path without an explicit Deny must reach manual approval when automatic
review cannot authorize it. Approval must issue exact path authorization and execute
successfully; denial must leave the target absent. The write tool currently rejects
absolute paths again when its optional atomic-create capability is installed, even
after successful exact-path authorization. Cover this defect with production-style
capability wiring, not only bare WriteTool tests. Do not equate this repair with the
separate automatic worktree admission requirement.

Implementation checkpoints: (1) approved external write and manual fallback;
(2) trusted worktree creation custody and multi-root permission/lease integration;
(3) durable logical-Session binding and resume/revocation; (4) adversarial/native
validation, exact-head security review and merged evidence. Owner remains Active
until all accepted behavior is implemented; partial milestones do not close it.

Decision: [ADR-099](../decisions/099-session-owned-worktree-admission.md).
