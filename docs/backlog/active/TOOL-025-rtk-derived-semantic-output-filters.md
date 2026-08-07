# TOOL-025: RTK-Derived Semantic Shell Output Filters

| Field | Value |
|---|---|
| Story ID | TOOL-025 |
| Source Issue | #143 |
| Status | Intake |
| Priority | P1 |
| Type | Tool Runtime / Integration |

## Disposition

Register the RTK-derived shell-output filtering request for source, architecture, and provenance
refinement. The owner must preserve Talos's existing permission, process, timeout, cancellation,
exit-status, and result authorities. No source extraction, dependency, or filter implementation is
authorized by this intake record.

## Required follow-up

- Select only bounded parser/filter logic with pinned upstream provenance and Apache-2.0 notices.
- Define raw-output fallback, never-worse bounds, diagnostic retention, and cross-platform behavior.
- Keep semantic filtering after authorization and outside native structured tools.
- Add fixture provenance review, savings evidence, and a runnable iteration before implementation.

## Dependencies

Coordinate with #23/#36 error semantics, #52–#57 permission convergence, #59 background jobs,
TOOL-005, and ADR-057. Keep this scope separate from R02 CLI/TUI decomposition.
