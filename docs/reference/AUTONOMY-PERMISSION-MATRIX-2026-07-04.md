# Autonomy Permission Matrix

**Status**: I092/A11 policy baseline.
**Date**: 2026-07-04
**Scope**: scheduled, batch, Guardian, and exec-style autonomy gates.
**Related**: I092, PERM-001, SCHED-001, TOOL-010, TOOL-016, ADR-011, ADR-012, ADR-026.

## Decision Boundary

This matrix does not enable new autonomy behavior. It defines the non-bypass permission contract that
must be satisfied before any scheduled direct tool execution, batch write/edit, Guardian approval, or
exec DSL work can ship.

The current safe rule remains:

- read-only operations may be `Allow` by default when resource policy allows them;
- write, execute, and network operations default to `Ask`;
- any explicit `Deny` on any facet denies the whole operation;
- approval of one operation does not approve follow-on operations, transitive actions, or scheduled
  future executions.

## Matrix

| Capability path | Current allowed behavior | Default decision | Required facets | Non-bypass rule | Activation gate |
|---|---|---|---|---|---|
| Scheduled message injection | `delay`/`schedule` may inject a future user-like message into the same session. The LLM mediates any later tool call. | Tool declaration may be read/internal; follow-on tools use their own decisions. | Internal/session scheduling facet; no write/execute facet unless the scheduled tool itself persists state. | A scheduled message is not a permission grant. Every later tool call is evaluated normally at execution time. | Existing SCHED-001 v1 shape only. |
| Scheduled direct tool execution | Not allowed. | Deny until a future ADR. | Would need the target tool's full permission profile captured and re-evaluated at fire time. | Cannot reuse approval from schedule creation. Cannot run after policy changes without re-checking. | New ADR plus tests for policy drift, cancellation, shutdown, and stale approvals. |
| Persistent scheduled tasks | Not allowed. | Ask or Deny depending on future design; no default Allow. | Write facet for scheduler state plus target-operation facets at fire time. | Persistence cannot create hidden write/execute authority across restarts. | Scheduler storage ADR and recovery tests. |
| Batch read | Not implemented; allowed design target. | Allow per readable path when each path is allowed. | Read facet per path. | A denied path returns a per-item denial and must not abort into an unstructured fallback that hides the denial. | TOOL-010 schema and per-item result tests. |
| Batch write/edit | Not implemented. | Ask per write/edit path; Deny if any path rule denies. | Write facet per path. | One approved path does not approve sibling paths. Partial success must be explicit. | TOOL-010 implementation tests for allow/ask/deny mix. |
| Batch mixed read/write | Not implemented. | Conservative aggregate: Deny if any denied facet; else Ask if any ask facet; else Allow. | Full set of read/write facets. | The model-facing summary must not collapse denied items into success. | Multi-resource permission tests and result schema. |
| Guardian advice | Guardian remains disabled. | No execution authority. | None; advisory only. | Advice cannot mutate `PermissionDecision`. | ADR-011 follow-up. |
| Guardian auto-approval | Not allowed. | Deny for write/execute/network in first slice. | Would need the target operation's normal facets plus an explicit Guardian provenance facet. | Guardian cannot approve write-capable tools in the first slice and cannot bypass `PermissionEngine`. | New iteration with deny-first tests. |
| Direct `exec` tool | TOOL-016 policy only. Structured argv execution, no shell. | Ask. | Execute facet for command; Read facet for `cwd` when supplied. | Approval of `exec` does not approve `bash`, shell syntax, plugin execution, or future DSL rules. | Already governed by `EXEC-TOOL-PERMISSION-POLICY-2026-07-02.md`. |
| Exec DSL | Not implemented. | Ask for simple typed rules; complex shell-like syntax falls back to Ask or Deny. | Typed facets emitted by DSL compiler. | DSL must compile to permission rules, not parse or execute shell text. | ADR-012 follow-up with compiler tests. |
| Plugin-originated tool autonomy | Not allowed beyond explicit local read-only plugin tool slices. | Same as native tool profile, plus plugin provenance must be visible. | Tool's declared facets plus plugin provenance. | Plugin install/load does not approve tool execution. | PLUGIN-001 runtime gate and DIST-001 policy gate. |

## Required Regression Coverage Before Runtime Expansion

Any future implementation that touches this matrix must include tests for:

- default `Ask` for write, execute, and network facets;
- explicit `Deny` winning over `Ask` and `Allow`;
- multi-facet aggregation where any denied facet denies the operation;
- per-resource batch evaluation with mixed allow/ask/deny outcomes;
- scheduled fire-time re-evaluation rather than schedule-time approval reuse;
- stale approval invalidation after config or workspace policy changes;
- no hidden execution from Guardian, plugin, or scheduler provenance;
- no secret values in permission logs or decision summaries.

## Current Validation Evidence

The current codebase already has the base permission primitives needed for this policy:

- `PermissionEngine::evaluate_profile` conservatively aggregates multiple facets.
- `ToolPermissionFacet` supports nature/resource/resource-kind metadata.
- Defaults keep Write, Execute, and Network at `Ask`.
- TOOL-016 direct `exec` exposes command and cwd facets and avoids shell parsing.

I092/A11 validation re-runs:

- `cargo test -p talos-permission`
- `cargo test -p talos-tools exec_tool`
- `scripts/validate_project_governance.sh .`
- `git diff --check`

## Residuals

- This matrix is policy and regression target, not implementation.
- No scheduled direct tool execution, persistent scheduler, batch write/edit, Guardian
  auto-approval, exec DSL, or plugin-originated autonomy is enabled by this document.
- Future runtime work must update this matrix and owner backlog docs before coding.
