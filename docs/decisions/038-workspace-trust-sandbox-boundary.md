# ADR-038: Workspace Trust Sandbox Boundary

## Context

Talos operates with explicit per-operation permission prompts. Long-running tasks trigger repeated
approval prompts for operations within the same Git repository, making autonomous workflows
impractical. PERM-004 requests a workspace trust mechanism where a detected Git repository root
serves as a coarse sandbox boundary after explicit user approval.

The current permission system (`PermissionEngine` in `talos-permission`) evaluates each tool call
against allow/deny rules. There is no concept of "workspace trust" or repo-scoped approval. This
ADR records the design boundary for adding that concept.

## Constraint Decomposition

| Constraint | Type | Source | Can Change? |
| --- | --- | --- | --- |
| All write-capable tools gated by permissions | Hard | AGENTS.md #4 | No |
| Deny rules always take precedence | Hard | Permission system invariant | No |
| No bash/exec broadening without touched-path evidence | Hard | PERM-005 dependency | No |
| Out-of-repo access remains strict | Hard | Security boundary | No |
| Git repo root as trust boundary is a Soft constraint | Soft | PERM-004 | Yes |
| Non-Git workspaces keep stricter mode | Soft | PERM-004 design | Yes |

## Reasoning

### Why workspace trust is needed

Current behavior: every `bash` command, file write, and Git write within a repo triggers an
approval prompt. A 50-step autonomous task can produce 100+ prompts. The `always` scope helps but
requires per-command approval and doesn't generalize to "this repo is safe to operate within."

### Design boundary

**Trust scope**: When a Git repository is detected at the workspace root, and the user explicitly
approves workspace trust, operations **within the repo root** get a coarser permission path:
- File read/write/edit within repo root: reduced prompt frequency (trust-based allow)
- Git operations within repo root: trust-based allow
- Bash commands targeting files within repo root: still require per-command approval until PERM-005
  provides touched-path evidence

**What stays strict**:
- Out-of-repo paths (home directory, /etc, /tmp outside workspace)
- Credential access (api keys, .env files)
- Network operations (push, publish, release)
- Destructive cleanup (rm -rf, force delete)
- Deny rules (always override trust)
- Non-Git workspaces (no repo root = no trust boundary)

### Why not OS-level sandbox now

OS-level sandboxing (bubblewrap, sandbox-exec) was evaluated in I004 and deferred. PERM-005 will
evaluate lightweight sandbox enforcement for bash/exec as a follow-up. This ADR covers the
**logical** trust boundary only — OS-level enforcement is a separate concern.

### Trust persistence

Workspace trust should persist across sessions for the same workspace path. Storage options:
- SQLite index (existing `index.db`) — trust table keyed by workspace hash
- Config file (`~/.talos/trusted_workspaces.toml`) — simpler, human-readable

Selected: config file, because trust decisions are user-facing and should be inspectable/editable.

## Decision

**Approve** workspace trust sandbox as a logical permission boundary:

1. **Detection**: `gix::Repository::discover()` at workspace root determines if a Git repo exists.
2. **Trust grant**: Explicit user approval via TUI/CLI prompt. Stored in
   `~/.talos/trusted_workspaces.toml` keyed by canonical workspace path.
3. **Permission effect**: When trust is active for a workspace:
   - File operations (read/write/edit/delete) within repo root: `Allow` unless Deny rule matches
   - Git operations within repo root: `Allow` unless Deny rule matches
   - Bash/exec: unchanged (still per-command) until PERM-005
4. **Deny precedence**: Deny rules always override trust. No exceptions.
5. **Non-Git workspaces**: No trust boundary available. Current strict mode continues.
6. **Out-of-repo access**: Any path outside the repo root keeps current strict permission mode.
7. **Symlink/`..` escape**: Canonicalized path comparison prevents escaping the trust boundary.

**Reject**:
- OS-level sandbox enforcement (deferred to PERM-005)
- Auto-trust without explicit user approval
- Trust applying to network/push/publish operations
- Trust for bash/exec without touched-path evidence

## Reversal Trigger

Revisit when:
1. PERM-005 provides touched-path evidence enabling bash/exec sandbox enforcement
2. Security audit identifies trust boundary escape vectors
3. Users report trust decisions being too coarse (need per-directory granularity)
4. Non-Git workspace users need a trust equivalent (e.g., Cargo workspace without Git)

## Related Documents

- `docs/backlog/active/PERM-004-workspace-trust-sandbox.md`
- `docs/backlog/active/PERM-005-logical-tool-sandbox-enforcement.md`
- `docs/decisions/007-process-hardening-unsafe.md`
- `docs/backlog/active/PERM-002-operation-scoped-permissions.md`
