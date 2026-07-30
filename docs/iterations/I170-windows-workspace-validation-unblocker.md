# Iteration I170: Windows Workspace Validation Unblocker

> Document status: Active
> Published plan date: 2026-07-31
> Planned objective: close the Windows `talos-tools` failures that block I169/PR #68 G13 by
> delivering the already-planned TOOL-023-A/C shell work and stable cross-platform tool output.
> Baseline rule: preserve this target; materially different work uses a new iteration ID.
> MVP deliverable: a rebuilt Windows Talos invokes the `powershell` tool successfully, enforces a
> single-shot timeout, emits platform-stable file/search paths, and passes locked workspace tests.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
| --- | --- | --- | --- | --- |
| `TOOL-023-A` | TOOL-023 | Ready | None | Shell timeout is an absolute deadline and preserves bounded output. |
| `TOOL-023-C` | TOOL-023 | Ready | TOOL-023-A | Windows uses PowerShell with env-scrub-only child hardening. |
| I170 portability slice | None | New corrective scope | None | File/search display paths and long-list metadata are stable on Windows. |

### Non-Terminal Inventory And Disposition

- I169 remains outside this independent change; after I170 merges, its branch must rebase onto the
  updated `main` before claiming the locked workspace gate.
- I168 remains Active in its separate provider-terminal worktree and is not modified by I170.
- I164 remains Paused; I158-I162 remain Blocked under their existing owners.
- No Review or Planned iteration is selected or superseded by this corrective slice.
- The user explicitly corrected the prior decision to stop at the Windows baseline. TOOL-023-A/C
  were already Ready, but their deferred task note is overridden only for this bounded unblocker.

### Branch And Worktree

- Branch: `codex/i170-windows-shell`
- Worktree: `C:/Users/12261/Documents/talos-worktrees/i170-windows-shell`
- Merge target: `main`
- Pull request: pending independent draft PR

### Scope

- Implement TOOL-023-A's single-shot timeout without changing the configured/default duration.
- Implement TOOL-023-C using Windows PowerShell, a `powershell` tool identity, child-only dangerous
  environment removal, and no Windows rlimit/Job Object claim.
- Preserve Unix `sh -c`, the Unix `bash` identity, permission routing, and ADR-007 hardening.
- Normalize workspace-relative file/search output separators to `/`.
- Define a deterministic 10-character Windows long-list permission/type projection without
  claiming Unix ownership or executable bits.
- Add platform-focused tests, user docs, ADR, security review evidence, and full locked validation.

### Non-Goals

- No TOOL-023-B timeout default/configuration change.
- No shell-selection setting, POSIX-to-PowerShell translation, cmd.exe fallback, Job Objects,
  sandbox redesign, new dependency, permission bypass, or broad test skip.
- No change to I169 steering semantics.

### Acceptance

- Given Windows, invoking `powershell` with platform-native commands returns output, stderr, exit
  code, working-directory behavior, and timeout evidence.
- Given dangerous inherited environment names, the Windows child cannot observe them.
- Given Unix, shell selection/name and `sh -c` hardening remain unchanged by cfg construction.
- Given recursive ls/glob/grep output on Windows, workspace-relative paths use `/` consistently.
- Given long ls output on Windows, the first field has exactly one type character plus nine
  conservative permission characters; it does not invent Unix uid/gid/nlink semantics.
- `cargo test --workspace --locked` and the repository's locked check/Clippy/format gates pass.

### Planned Validation

- `cargo test -p talos-tools --locked`
- `cargo fmt --all -- --check`
- `cargo check --workspace --locked`
- `cargo clippy --workspace --locked -- -D warnings`
- `cargo test --workspace --locked`
- governance validation and scale assessment
- rebuilt Windows CLI mock smoke plus a direct PowerShell tool walkthrough
- CI status after push

## Closure Ledger

- Requested outcome: resolve, rather than merely record, all failures blocking PR #68 review.
- Artifacts: TOOL-023-A/C, I170, ADR-057, BashTool, file/search projections, tests, READMEs, Board,
  backlog, I169, manifest, and PR/Issue evidence.
- Existing assets preserved: I169 implementation commits, ADR-007/012 constraints, TOOL-023-B,
  I168's separate worktree, and Unix shell behavior.
- Validation required: focused platform tests, full locked workspace gates, real Windows smoke,
  governance, and remote CI state.
- Residual destination: this owner document; no failed required gate may be hidden as unrelated.

## Execution Record

| Date | Type | Record |
| --- | --- | --- |
| 2026-07-31 | User correction | The prior stop-at-baseline decision was rejected; the requested outcome is a reviewable PR with the failures actually resolved. |
| 2026-07-31 | Reproduction | `cargo test -p talos-tools --locked -- --test-threads=1` produced 27 failures: 19 shell-spawn/semantics, four long/recursive ls, and four glob/grep path assertions. |

## Verification Evidence

- Pending implementation.

## Completion Evidence

- Completion Commit: pending. Do not mark Complete until the implementation commit already exists
  and every required local validation plus the recorded Windows walkthrough passes.
