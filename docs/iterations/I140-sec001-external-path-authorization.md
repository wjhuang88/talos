# Iteration I140: SEC-001 External-Path Authorization

> Document status: Complete — security review and full locked replay accepted 2026-07-17
> Activation date: 2026-07-17
> Objective: let a user approve an exact external file operation without weakening workspace,
> Deny, symlink, traversal, headless, or non-file permission boundaries.

## Selection And Inventory

This corrective iteration was created after the maintainer reported that
`path escapes workspace root` could not be resolved by approval. I018 remains deferred. I135-I139
were under corrective review and are repaired in the same review session without changing their
published objectives. MODEL-007 and TUI-031 remain unselected intake stories. No other Active or
Review iteration was bypassed.

| Story | Prior state | Outcome |
| --- | --- | --- |
| SEC-001 | Confirmed security/product gap | Structured external-path capability and security review |

## Scope

- Preserve workspace-contained file behavior.
- Ask before an external read/write/edit/delete/list operation unless an exact reusable rule exists.
- Convert resolved permission decisions into an exact normalized execution authorization.
- Wire Runtime, CLI print/inline/interactive/RPC, and TUI permission composition roots.
- Revalidate at execution and fail closed on changed symlink targets or malformed paths.
- Add explicit approval, denial, missing-handler, exact-scope, cross-operation, and platform-safe
  regression coverage.

## Non-Goals

- No bash/exec, network, sandbox, workspace-trust, credential, persistence-format, or event-order
  expansion.
- No blanket external-filesystem flag.
- No automatic approval and no persistence of `ApproveOnce`.
- No new dependency or breaking public API.

## Acceptance

- [x] A workspace-external read produces `Ask`, and `ApproveOnce` executes that exact request.
- [x] `AlwaysApprove` reuses the existing bounded runtime rule without a second prompt.
- [x] Deny and a missing headless approval handler fail closed without returning file content.
- [x] A grant cannot be reused for another path or another tool operation.
- [x] A symlink target change between authorization and execution fails closed.
- [x] Workspace-contained behavior and no-path read tools remain unchanged.
- [x] Write/edit/delete/list use the same typed authorization boundary.
- [x] No session/TLOG, approval event, dependency, sandbox, or credential behavior changes.

## Evidence

- Focused tests:
  - `talos-permission`: external Ask, exact reusable Allow, Deny precedence, internal/no-path compatibility.
  - `talos-tools`: all five file operations, wrong path/operation rejection, symlink retarget rejection.
  - `talos-runtime`: approve-once, always, explicit deny, and missing-handler behavior.
- Cross-platform path construction uses `PathBuf`, `Path::components`, and `canonicalize`; the
  existing external-path fixture is platform-neutral and runs in workspace CI.
- Full locked validation and governance results are appended at closeout.

## Closeout Evidence

- `cargo fmt --all -- --check`: clean.
- `cargo check --workspace --locked`: clean.
- `cargo clippy --workspace --locked -- -D warnings`: clean.
- `cargo test --workspace --locked`: all pass, including Runtime, permission, file-tool, and
  platform-native path regressions.
- `./scripts/release_preflight.sh`: passed.
- `scripts/validate_project_governance.sh .`: 0 warnings.
- `git diff --check`: clean.
- Security review: Accepted; Deny/headless/direct-call/path-reuse/symlink-retarget cases fail
  closed.

## Files And Boundaries

- Neutral contract: `crates/talos-core/src/tool.rs`
- Decision/capability producer: `crates/talos-permission/src/lib.rs`
- Consumers: `crates/talos-tools/src/file_tools/`
- Composition roots: `crates/talos-runtime/src/lib.rs`, `crates/talos-cli/src/registry.rs`
- Decision: ADR-047
- Security review: `docs/reference/I140-SEC001-SECURITY-REVIEW-2026-07-17.md`

## Rollback

Remove the authorization overrides and composition-root capability construction; raw file tools
will again reject every external path. Do not weaken canonicalization or make raw execution
implicitly external-capable as a fallback.
