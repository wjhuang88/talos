# PERM-001: Guardian and Exec Approval Policy

## Outcome

AI-mediated approval and exec policy rules can be implemented without weakening Talos'
permission boundary.

## Status

In Progress — I077/T114 selected only the direct `exec` policy slice. Guardian approval and exec
DSL implementation remain deferred.

## Priority

P2.

## Required Reads

- `docs/decisions/011-guardian-approval-boundary.md`
- `docs/decisions/012-exec-policy-dsl-boundary.md`
- `docs/roadmap/REQUIREMENT-CONVERGENCE.md`

## Acceptance Criteria

- [ ] Guardian remains disabled by default and cannot bypass `PermissionEngine`.
- [ ] Guardian cannot auto-approve write-capable tools in the first slice.
- [ ] Exec DSL compiles into typed permission rules and is not a shell parser.
- [ ] Complex shell features fail back to Ask unless a future ADR changes the boundary.
- [ ] Decision logs avoid secrets and full sensitive arguments.

## Residual Work Destination

Implementation requires a dedicated iteration activation after current portability/provider/TUI work
is resolved or reprioritized.

## I077/T114 Direct Exec Policy Slice

`docs/reference/EXEC-TOOL-PERMISSION-POLICY-2026-07-02.md` defines the policy for TOOL-016's
direct `exec` tool. This does not activate Guardian approval or the exec DSL. It only clears a
structured argv-based tool implementation that defaults to `Ask`, denies sensitive env names before
spawn, avoids shell parsing, and exposes command/cwd facets through the existing permission profile.
