# ADR-040: Command Access Evidence and Logical Sandbox Enforcement

- **Status**: Accepted (maintainer security sign-off recorded 2026-07-12: evidence is diagnostic-only, never auto-Allow; dangerous flags classified Unknown; bash/exec remains per-command Ask/Deny; OS-level sandbox deferred)
- **Date**: 2026-07-12
- **Backlog**: PERM-005, I117/N110-N114

## Context

PERM-004 (ADR-038) established Git repo root as a logical trust boundary for file writes after
explicit user approval. However, bash/exec command execution remains per-command Ask/Deny because
Talos cannot prove which paths a command touches, whether it spawns children, or whether it accesses
the network.

PERM-005 requests the next step: strengthen the logical sandbox so command execution can report or
enforce where it operated. The highest-risk gap is that `bash` and `exec` can read, write, delete,
or spawn child processes that touch paths not obvious from the command line.

The key insight from ADR-012 is that Talos must not attempt to parse arbitrary shell syntax as a
security boundary. Instead, it should classify commands into a bounded surface and fall back to
strict behavior when classification is uncertain.

## Constraint Decomposition

| Constraint | Type | Source | Can Change? |
| --- | --- | --- | --- |
| All write-capable tools gated by permissions | Hard | AGENTS.md #4 | No |
| Permission/sandbox changes require security review | Hard | AGENTS.md #5 | No |
| Deny rules always take precedence | Hard | Permission system invariant | No |
| No bash/exec broadening without touched-path evidence | Hard | PERM-005 dependency | No |
| Out-of-repo access remains strict | Hard | Security boundary | No |
| Network/push/publish/release remain separately gated | Hard | AGENTS.md, PERM-004 | No |
| Arbitrary shell parsing is not a security boundary | Hard | ADR-012 | No |
| Access evidence is observation, not authority | Soft | PERM-005 design | No |
| Portable touched-path observation is incomplete | Assumption | Platform variability | Must be bounded |

## Reasoning

### Declared vs Observed vs Unknown Access

A command's access can be classified into three evidence states:

1. **Declared**: The command's structure provably limits its access. For example, `cat <relative-path>`
   with no shell metacharacters declares a read of one repo-local file. The existing
   `BashCommandClass` classifier already distinguishes read-only inspection from mutating commands.

2. **Observed**: The command's actual file/process/network access was observed at runtime. This
   requires platform support (e.g., `fs_usage` on macOS, `inotify` on Linux, or `strace`/`dtrace`).
   Observation is not universally portable and may not capture all access (e.g., memory-only
   operations, children that exec before observation starts).

3. **Unknown**: The command's access cannot be proven or observed. This includes commands with
   shell metacharacters, variable expansion, subprocess spawning, network tools, or any command
   whose canonical path targets cannot be verified.

### Evidence Is Not Authority

Access evidence describes what a command did or will do. It does not grant permission by itself.
The permission engine uses evidence to narrow or broaden the approval path, but Deny rules,
out-of-repo checks, and high-risk classes remain authoritative. A command claiming it only reads
one file does not prove it touched only that file.

### Safe Fallback: Strict When Unknown

When access is unknown or unobservable, the command escalates to Ask or Deny. It never silently
inherits workspace trust. This is the core security property: **unknown access cannot leak trust**.

### Canonical Root Enforcement

For declared access, paths are canonicalized and compared against the Git repo root. Symlinks and
`..` traversal are resolved before comparison. A path whose canonical form is outside the repo root
is treated as out-of-repo, regardless of how it was specified.

### Why Not OS-Level Sandbox Now

OS-level sandboxing (`bubblewrap`, `sandbox-exec`, `seccomp`) was evaluated in ADR-038 and deferred.
The `talos-sandbox` crate has `BubblewrapSandbox` and `SeatbeltSandbox` implementations, but they
require platform-specific binaries and may not be available in all environments. This ADR covers
**logical** evidence and enforcement only — OS-level enforcement remains a separate follow-up.

## Decision

**Approve** command access evidence and logical sandbox enforcement as follows:

1. **Typed Access Evidence**: Add a serializable `AccessEvidence` type that classifies command
   access into `Read`, `Write`, `Delete`, `Spawn`, `Network`, and `Unknown` categories. Each
   category carries the relevant paths, process names, or domains when known.

2. **Declared Access Classification**: Extend the existing `BashCommandClass` classifier to produce
   `AccessEvidence` for commands with provably bounded structure. Commands with shell
   metacharacters, variable expansion, or unclassifiable structure produce `Unknown` evidence.

3. **Repo-Boundary Enforcement**: When workspace trust is active and a command's declared access is
   fully within the canonical repo root, the command may use a coarser approval path. Any path
   outside the repo root, any `Unknown` access, any `Spawn` or `Network` intent escalates to Ask.

4. **Deny Precedence**: Deny rules always override trust-based allow. No exceptions.

5. **Non-Git Workspaces**: No trust boundary available. All commands remain per-command Ask/Deny.

6. **Trust Status and Revoke**: Expose explicit read-only trust status (`talos diagnostics status`
   already reports this). Add `talos permissions trust revoke` to remove workspace trust. Revocation
   takes effect across new processes because trust is persisted in
   `~/.talos/trusted_workspaces.toml`.

7. **Observation Limitation**: Runtime path observation (fs_usage/inotify) is NOT implemented in
   this slice. The `Observed` evidence state is defined but not produced. Commands that cannot be
   classified by structure alone remain `Unknown`.

**Reject**:
- OS-level sandbox enforcement as a dependency (deferred to separate ADR)
- Trust from observation alone without structural classification
- Broadening bash/exec to repo-wide Allow
- Parsing arbitrary shell syntax as a security boundary
- Auto-approval of Spawn or Network intent under workspace trust

## Reversal Trigger

Revisit when:
1. A proven portable path-observation mechanism is available (e.g., `fanotify`/`fs_usage` wrapper)
2. Security audit identifies classification bypass vectors
3. `talos-sandbox` OS-level enforcement is accepted via separate ADR
4. Users report declared-access classification being too conservative for common workflows

## Related Documents

- `docs/backlog/active/PERM-005-logical-tool-sandbox-enforcement.md`
- `docs/backlog/active/PERM-004-workspace-trust-sandbox.md`
- `docs/decisions/038-workspace-trust-sandbox-boundary.md`
- `docs/decisions/012-exec-policy-dsl-boundary.md`
- `docs/decisions/007-process-hardening-unsafe.md`
- `crates/talos-permission/src/workspace_trust.rs`
- `crates/talos-tools/src/bash_tool.rs`
- `crates/talos-tools/src/exec_tool.rs`
