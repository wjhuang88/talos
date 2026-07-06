# PERM-003: Permission Experience Reference Study And Redesign

| Field | Value |
|---|---|
| ID | PERM-003 |
| Type | Security/UX Design Story |
| Priority | P1 |
| Status | Complete — reference study, taxonomy, UX copy, and first measured trace closed |
| Source | Maintainer request 2026-07-05 — repeated bash/tool approvals make long-running Talos tasks impractical |
| Depends on | PERM-001, PERM-002, TOOL-017 |
| Blocks | Any broad change to bash `always` semantics, unattended execution policy, or default permission presets |

## Problem

Talos currently has the right safety instinct — write-capable tools and shell execution must route
through permissions — but the interaction model is still too noisy for real long-running work.
Repeated approval prompts break unattended execution, interrupt task flow, and make users distrust
the "always approve" choice when the same practical operation asks again.

At the same time, fixing this by making `bash` broadly trusted would violate Talos' hard constraint:
all write-capable tools remain permission-gated. The redesign must reduce approval frequency without
turning one approval into unrestricted local command execution.

## Design Questions

- What should "always approve" mean for:
  - an identical command in the same working directory;
  - a command family such as `cargo test` with varying test filters;
  - write operations in one directory;
  - network operations for one host;
  - multi-step validation plans in a long task?
- Which approvals should persist only for the current session, and which can be saved to config?
- How should Talos display the exact reusable scope before the user approves it?
- How should deny rules override previously allowed scopes?
- How should unattended/long-running tasks declare the operations they expect before execution?
- How should Talos avoid using broad host shell commands when an internal tool or typed adapter can
  cover the task?

## Required Reference Study

Before implementation, record a comparison table covering at least:

- Claude Code permission modes and project/user settings.
- Codex CLI approval behavior and sandbox/escalation model.
- OpenCode permission/tool execution behavior.
- Aider or another established coding agent's command execution safeguards.
- Talos current PERM-001/PERM-002 behavior.

The study must distinguish verified behavior from inference. If a reference project has changed
since the local docs were written, update the evidence before drawing conclusions.

Reference study completed on 2026-07-05:
`docs/reference/PERMISSION-EXPERIENCE-REFERENCE-STUDY-2026-07-05.md`.

## Candidate Direction

This story should produce a permission design that balances stability and control:

- Use exact-operation approval for high-risk shell commands by default.
- Add deliberate, user-visible reusable scopes for common validation commands, not hidden broadening.
- Keep write approvals directory-scoped when the user chooses `always`.
- Prefer internal host-tool adapters and typed tools over shell fallback for common project tasks.
- Let long-running tasks preflight a bounded permission plan so approvals happen up front where
  possible.
- Preserve explicit Deny precedence over all runtime/session allow rules.

## Reference-Study Findings

- Claude Code and Codex both separate approval decisions from sandbox/filesystem containment.
  Talos should preserve that separation: an approval scope is not the same thing as broader write or
  network capability.
- OpenCode's current-session `always` behavior is the closest fit for Talos, but Talos should make
  the reusable scope stricter by default: exact command first, audited command templates only for
  low-risk validation families.
- Aider's broad yes mode is useful for flow but is too coarse for Talos' safety boundary. Talos
  should prefer long-task preflight plans made of normal scoped permissions, not a global yes mode.
- Host-tool adapters must be project-type-gated. Cargo is a Rust adapter, not a generic Talos
  validation model.

## Proposed Talos Taxonomy

| Scope | Purpose |
|---|---|
| `exact_command` | Same normalized command, working directory, and risk class. |
| `command_template` | Audited low-risk validation families after project-type detection. |
| `directory_write` | Write/edit/delete within one approved directory subtree. |
| `remote_network` | One host/service/action family. |
| `long_task_preflight` | Ordered batch of bounded scoped permissions for unattended work. |
| `internal_service` | In-process Talos capability with no host command execution. |
| `host_tool_adapter` | Ecosystem-specific host tool, selected only after project detection. |

## Non-Goals

- No blanket `bash = Allow`.
- No automatic self-approval by the model.
- No weakening of write, network, git push/publish, or destructive command gates.
- No implementation before the reference study and acceptance matrix are reviewed.

## Acceptance Criteria

- [x] Reference-project comparison is recorded with links or local evidence paths.
- [x] Talos permission taxonomy distinguishes exact command, command template, directory write,
      remote/network, and long-task preflight scopes.
- [x] UX copy for approval prompts shows the reusable scope before the user chooses `always`.
- [x] Repeated approval reduction is measured against at least one recorded long-task trace.
- [x] Security review proves deny rules still win and high-risk command families are not broadened
      accidentally.
- [x] TOOL-017 and PERM-002 are updated or superseded consistently after this design lands.

## Closeout

2026-07-05 implementation closeout:

- Approval prompts now show the exact reusable `always` scope as a session allow rule and explicitly
  state that configured deny rules still win.
- Bash low-risk read-only inspection and validation-build commands now use scoped command-template
  resources loaded from `crates/talos-tools/src/bash_permission_policy.toml`. Choosing `always` for
  an eligible template covers different target objects in the same cwd and command family, matching
  the Codex-style prefix behavior without allowing all bash.
- Runtime `always` rules continue to insert before the default `Ask` catch-all, not before
  configured deny policy.
- Deny precedence is covered by `test_configured_deny_precedes_runtime_always_allow`.
- Repeated approval reduction is recorded in
  `docs/reference/PERMISSION-LONG-TASK-TRACE-2026-07-05.md` and covered by
  `test_repeated_always_approval_reduces_same_operation_to_zero_prompts`.
- Different-object low-risk bash approval reduction is covered by
  `test_low_risk_bash_template_reduces_different_object_prompts` and
  `test_bash_read_only_template_shares_across_objects_in_same_cwd`.
- Template safety is covered by tests proving parent/absolute paths, `find -exec`, complex shell
  operators, and mutating commands keep exact resources.
- I098 adds `talos permissions preflight`, a read-only long-task planning surface that uses the
  real tool permission profile to show current decisions and reusable `always` scopes before a run.
  It does not execute tools or install allow rules.
- `TOOL-017` remains allowed only after the PERM-003 taxonomy and deny-precedence tests; its
  implementation must preserve the same scoped facets instead of adding shell-like blanket
  permission.

## Required Reads

- `docs/backlog/active/PERM-001-guardian-exec-policy.md`
- `docs/backlog/active/PERM-002-operation-scoped-permissions.md`
- `docs/backlog/active/TOOL-017-exec-multi-parallel-pipe.md`
- `docs/reference/REFERENCE-PROJECTS.md`
- `docs/decisions/024-embeddable-runtime-api-boundary.md`
- `crates/talos-permission/src/lib.rs`
- `crates/talos-cli/src/approval.rs`
- `crates/talos-tools/src/bash_tool.rs`
- `crates/talos-tools/src/exec_tool.rs`
