# ADR-099: Session-owned worktree admission

Status: Accepted (effective with governance merge #889)
Owner: [WORKTREE-001](../tasks/2026-11-08-session-owned-worktrees.md)

## Context

A session can create a sibling Git worktree but its file tools and automatic review
remain bound to the startup workspace. This blocks development. Membership must be
synchronized without interpreting arbitrary command output as authority. ADR-064's
single managed root is extended only for verified session-owned additional roots;
ADR-089's exact Git effect and denial boundaries remain applicable.

## Decision

User-confirmed default: ordinary native workspace file operations are allowed.
External file operations require authorization through the existing pipeline: retain
applicable automatic review and fall back to human approval when it cannot allow.
Do not expand automatic eligibility for external paths. This supersedes
the default ordinary local-write Ask/trust requirement only for this file scope.
Explicit restrictions, protected paths, Execute/Network and Git administrative
effects are unchanged. Tool provenance must not be inferred from a name.

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

## Policy precedence and ordinary-file boundary

Explicit Deny and protected-path restrictions win. Explicit user Ask remains an
intentional human checkpoint; replacing the default local-file Ask must not override
it. After explicit policy, the new workspace default applies only to registry-proven
built-in ordinary read/write/edit operations on regular file paths. Read-only listing
and search retain their existing rules. Recursive deletion, Git administration,
credential/secret stores, sensitive configuration and symlink escapes do not acquire
new default authority. Plugin/MCP tools cannot gain this privilege by copying names.

External requests retain existing auto-eligibility rules, so a workspace-only
auto policy remains ineligible outside it and reaches the human fallback. This is
not a new blanket permission for model-approved external writes.

## Implementation design checkpoint

This claim activates investigation and implementation of approval/execution repair.
Before automatic membership expansion, independently review the concrete custody
and persistence implementation plan. Destination custody must use exclusive directory
creation and a held directory identity before native Git execution, with verified
Git backlinks and unchanged identities on completion. Failure never registers a root.
A native trusted producer must record completion before mutable display/hooks; generic
post-command scans cannot mint receipts. Native Windows must use real directory
identity, not timestamp-only identity. Persisted evidence must survive process restart
without allowing recreation at the old path to inherit authority.

The absolute-path write repair must resolve paths into an appropriate directory
capability, preserving create-new semantics and parent identity checks. Do not merely
skip the atomic capability for absolute inputs. Regression tests must install the
same capability configuration as the CLI and exercise actual approval delivery.
