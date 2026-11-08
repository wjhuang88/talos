# ADR-099: Session-owned worktree admission

Status: Proposed
Owner: [WORKTREE-001](../tasks/2026-11-08-session-owned-worktrees.md)

## Context

A session can create a sibling Git worktree but its file tools and automatic review
remain bound to the startup workspace. This blocks development. Membership must be
synchronized without interpreting arbitrary command output as authority. ADR-064's
single managed root is extended only for verified session-owned additional roots;
ADR-089's exact Git effect and denial boundaries remain applicable.

## Decision

Workspace-boundary Ask is recoverable through the existing human approval surface.
Only explicit Deny remains terminal. Exact path approval must not be discarded by
a subsequent single-root atomic-write check. Use the existing capability for paths
it can address, and the existing exact-authorized execution path for others.

Automatic membership requires a host-owned creation receipt: logical Session,
invocation, canonical original repository, exclusive destination creation custody,
and verified target/Git management identities. Each admitted root has its own held
directory capability. No common parent grant; no global or repository-wide list.
Persist only receipt evidence in host-owned session storage, not workspace files.
Resume validates identity and Git linkage before rebinding the current permission
Session; directory/metadata replacement or deletion revokes membership. Bump workspace
revision and invalidate pending approvals on membership changes. Explicit Deny,
secret/VCS protections and sandbox policy continue to apply.

Support bounded literal shell `git worktree add` by interpreting a proven invocation
before execution, retaining the approved effects and capturing native completion
before hooks. The implementation must own exclusive destination creation; a scan of
newly appearing worktrees or a successful exit alone is not creation proof. Dynamic
shell expressions and unprovable compound commands do not silently acquire roots:
report that membership was not registered and retain the exact human approval route.
A structured creation entry point may share this path; it cannot bypass approval.

Public APIs must be additive. A persisted record requires versioned serde/schemars
schema, logical-session binding, bounded size and fail-closed decoding. Do not infer
registration from old transcripts. Existing manually created worktrees retain manual
approval unless explicitly admitted. No new unsafe implementation is authorized.

## Dependency tradeoff

Soft: creation uses the project's existing reviewed host-Git process boundary;
Git's worktree metadata is validated locally. Missing Git produces a clear creation
error and no membership. Replace host Git only when a Rust-native implementation
supports equivalent worktree creation, metadata and effect validation. Do not add a
shell runtime or unreviewed process launcher.

## Validation

Owner acceptance includes real manual approval fallback and real shell creation,
failed and concurrent creation, forged output/hooks, explicit Deny, sensitive paths,
cross-session isolation, deletion/replacement and resume. Full pinned locked preflight,
cargo-xwin MSVC test compilation and native Windows identity tests are required.
Independent security review must verify creation custody and execution-time fencing.
This ADR defines requirements; it does not claim implemented capabilities.
