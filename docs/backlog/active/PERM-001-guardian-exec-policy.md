# PERM-001: Guardian and Exec Approval Policy

## Outcome

AI-mediated approval and exec policy rules can be implemented without weakening Talos'
permission boundary.

## Status

In Progress — I077/T114 selected only the direct `exec` policy slice. Guardian approval and exec
DSL implementation remain deferred.

I092 activation note (2026-07-04): PERM-001 is selected only for the autonomy permission matrix.
Guardian auto-approval and exec DSL implementation remain disabled. A11 must prove deny/ask/allow
behavior for scheduled, batch, and exec-style paths before any runtime expansion.

I092 A11 result (2026-07-04): `docs/reference/AUTONOMY-PERMISSION-MATRIX-2026-07-04.md` records
the non-bypass matrix. Guardian remains disabled; Guardian auto-approval is denied for
write/execute/network in the first slice; exec DSL remains unimplemented and must compile to typed
permission rules before any runtime use.

## Priority

P2.

## Required Reads

- `docs/decisions/011-guardian-approval-boundary.md`
- `docs/decisions/012-exec-policy-dsl-boundary.md`
- `docs/roadmap/REQUIREMENT-CONVERGENCE.md`

## Acceptance Criteria

- [ ] Runtime Guardian implementation remains disabled by default and cannot bypass
      `PermissionEngine`.
- [ ] Runtime Guardian implementation cannot auto-approve write-capable tools in the first slice.
- [x] Autonomy permission matrix records that Guardian remains disabled by default and cannot
      bypass `PermissionEngine`.
- [x] Autonomy permission matrix records that Guardian cannot auto-approve write-capable tools in
      the first slice.
- [ ] Exec DSL compiles into typed permission rules and is not a shell parser.
- [x] Complex shell features fail back to Ask unless a future ADR changes the boundary.
- [ ] Decision logs avoid secrets and full sensitive arguments.

## Residual Work Destination

Implementation requires a dedicated iteration activation after current portability/provider/TUI work
is resolved or reprioritized.

## I077/T114 Direct Exec Policy Slice

`docs/reference/EXEC-TOOL-PERMISSION-POLICY-2026-07-02.md` defines the policy for TOOL-016's
direct `exec` tool. This does not activate Guardian approval or the exec DSL. It only clears a
structured argv-based tool implementation that defaults to `Ask`, denies sensitive env names before
spawn, avoids shell parsing, and exposes command/cwd facets through the existing permission profile.
