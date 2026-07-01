# Autonomous Validation Loop

**Status**: Proposed for I076/T107
**Created**: 2026-07-01
**Related**: `REL-002`, `T107`, `T108`, `TOOL-016`

## Problem

REL-002 requires Talos to plan, implement, verify, and record its own development work before a
future `v1.0.0` claim. Current validation is still operator-driven: agents decide which commands to
run, execute them through the host shell, summarize evidence, and synchronize owner docs manually.

An autonomous validation loop is needed, but it must not become a hidden pass mechanism or a
permission bypass. Validation has to be explicit, observable, bounded, and reproducible.

## Proposed Approach

Split validation into three phases with different authority levels:

| Phase | Surface | Authority | Purpose |
|---|---|---|---|
| 1 | Validation plan/report | Read-only | Show required checks, missing prerequisites, and evidence fields without running commands. |
| 2 | Explicit validation execution | User-initiated CLI or permission-gated tool | Run allowlisted checks and record command-level evidence. |
| 3 | Self-bootstrap rehearsal loop | Talos primary runtime plus evidence record | Use the plan/execution surfaces during real Talos-on-Talos sessions. |

T108 should implement only Phase 1 unless a separate decision approves Phase 2. The first safe
surface should be a read-only command or tool that reports a validation matrix for the current
workspace and selected profile. It may read governance docs, Cargo metadata, scripts, and Git state;
it must not spawn validation commands, install dependencies, push, publish, tag, or edit files.

Recommended T108 shape:

```text
talos validate plan --profile i076
talos validate plan --profile workspace
talos validate plan --profile governance --json
```

If implemented as an agent-visible tool instead of a CLI command, the same Phase 1 boundary applies:
the tool may return planned commands and missing prerequisites, but it may not execute those
commands.

## Security Boundary

- Validation planning is read-only and cannot change repository state.
- Process execution is not part of Phase 1.
- Future Phase 2 execution must be either directly user-initiated in the CLI or routed through the
  existing permission pipeline when invoked by an agent.
- No validation surface may silently downgrade a required check. A skipped check is a recorded
  failure or explicit `not-run` state, not a pass.
- No validation surface may infer success from missing output.
- Network actions, dependency installation, publishing, release tagging, issue mutation, and git
  push are out of scope for validation execution.
- Command profiles must be allowlisted and versioned in source or governance docs. Free-form shell
  input belongs to `TOOL-016` and its separate permission policy.
- Validation evidence must include command text, working directory, timestamp, exit status, and a
  bounded stdout/stderr summary or output digest for every executed command.

## No-Hidden-Pass Rules

- Report `planned`, `passed`, `failed`, `not-run`, and `blocked` as distinct states.
- A profile is green only when every required check has explicit passing evidence from the current
  run or from a named accepted evidence record.
- Cached evidence must name the source file or commit that produced it.
- Missing scripts, missing Cargo workspace metadata, dirty working trees, and unavailable commands
  are visible findings.
- Human or external-agent assistance in a self-bootstrap rehearsal must be labeled in the evidence
  record.

## Initial Profiles

| Profile | Required checks | Phase 1 output |
|---|---|---|
| `governance` | `scripts/validate_project_governance.sh .` | Script presence, expected command, evidence fields. |
| `i076` | Targeted checks for provider, TUI, tools, CLI plus governance | Commands selected from I076 planned validation and current execution status. |
| `workspace` | `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo test --workspace`, governance | Full release-style validation plan without publish/tag actions. |

## Alternatives Considered

- Extend the existing governance status command to run all checks automatically. Rejected for T108:
  the current governance status path already spawns validation, and expanding that pattern would
  blur read-only status with command execution.
- Implement a general `exec`-style validation tool first. Rejected for T108: free-form process
  execution belongs to `TOOL-016` and needs a separate allowlist/default-permission decision.
- Rely on GitHub Actions only. Rejected: external CI is useful evidence, but REL-002 requires Talos
  to participate in its own local development loop.

## Open Questions

- Should validation profiles live in Rust code, governance docs, or a checked-in config file?
- Should Phase 2 evidence be stored as session history, a task evidence file, or both?
- Should `talos governance status` stop executing validation and delegate to a future
  `talos validate` command for clearer boundaries?
- How should targeted profiles discover touched crates without making Git state a hidden source of
  truth?

## Dependencies

- `REL-002` for self-bootstrap release-gate semantics.
- `docs/reference/SELF-BOOTSTRAP-EVIDENCE-TEMPLATE.md` for rehearsal evidence fields.
- `TOOL-016` before any free-form or agent-initiated process execution surface.
- Existing permission pipeline before agent-triggered Phase 2 execution.
