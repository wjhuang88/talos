# Direct Exec Tool Permission Policy

**Status**: Accepted for I077/T114; amended by I099 for bounded multi-step and pipe execution
**Date**: 2026-07-02
**Scope**: TOOL-016 direct `exec` tool permission and validation policy
**Related**: TOOL-016, PERM-001, ADR-012, ADR-026, Issue #16

## Decision

T115 may implement a direct `exec` tool only as a structured single-process executor:

- input is `command`, `args`, optional `cwd`, optional `env`, and optional `timeout_secs`;
- execution uses `tokio::process::Command` directly, never `sh -c`;
- `args` are passed as argv elements without shell parsing, glob expansion, pipelines,
  redirection, command substitution, or background jobs;
- output is bounded and reports exit code, stdout, stderr, and duration;
- timeout is clamped and must terminate the child process.

I099 extends the same boundary to bounded structured composition:

- `steps` may run sequentially or with `mode: "parallel"`;
- `pipes` may connect one step's bounded stdout into the next step's stdin;
- multiple pipe chains may run sequentially or with `mode: "parallel"`;
- every process remains a direct `tokio::process::Command` invocation;
- pipe chains are bounded in-memory transfers, not unbounded shell pipes.

## Permission Policy

- Default decision remains `Ask` because `exec` is `ToolNature::Execute`.
- The tool must override `nature()` to `Execute`; it must not rely on name inference.
- The invocation permission profile must include:
  - an `Execute` facet with `ToolResourceKind::Command` and resource `command`;
  - a `Read` facet with `ToolResourceKind::Path` for `cwd` when `cwd` is supplied.
- Multi-step and pipe invocations must expose the same facets for every step that can spawn.
- A disclosed or approved `exec` call does not approve `bash`, `sh`, plugin execution, or future
  shell-like DSL rules.
- Complex shell behavior is out of scope. If the user needs shell syntax, the existing `bash` tool
  remains the explicit shell surface and still asks by default.

## Environment Policy

- `env` is an explicit map of process environment additions/overrides.
- Sensitive environment variable names are denied before spawn. This includes names containing
  `KEY`, `TOKEN`, `SECRET`, `PASSWORD`, `CREDENTIAL`, `COOKIE`, and `AUTH`.
- Permission and display surfaces may show environment variable names, but must not echo values.
- The implementation must keep the existing process-hardening denylist behavior for inherited
  dangerous variables where supported.
- No `env_remove`, environment file loading, or config/env interpolation is authorized in T115.

## Path And Working Directory Policy

- `cwd`, when present, must be resolved by the runtime against the workspace path policy used by
  existing tools.
- Missing or invalid `cwd` must fail before spawning.
- `cwd` approval does not grant file writes; any side effects remain covered by the `Execute`
  approval for the process.

## Validation Required For T115

- success, non-zero exit, spawn failure, timeout kill, stdout/stderr bounding;
- argument safety proving shell metacharacters are passed literally;
- permission profile exposes command and cwd facets;
- sensitive env names are denied and env values are not echoed;
- no shell process is used for normal execution.

## Validation Required For I099 Structured Composition

- sequential steps still stop after the first failure;
- parallel steps report each step independently;
- pipe chains pass stdout to stdin without invoking a shell;
- parallel pipe chains report each chain independently;
- permission profiles expose command and cwd facets for every pipe step;
- mixed `steps` and `pipes` are rejected in this slice.

## Residuals

- Command allowlists beyond standard permission rules remain deferred to PERM-001/ADR-012 follow-up
  work.
- Per-argument permission matching is not implemented in T115.
- Cross-platform command lookup differences are accepted for the first slice and should be covered
  by portable fixture commands where possible.
- Unbounded streaming pipes remain out of scope. The I099 pipe path is intentionally bounded.
