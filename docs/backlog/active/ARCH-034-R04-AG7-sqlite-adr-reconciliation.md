# ARCH-034-R04-AG7: Bundled SQLite ADR Reconciliation

| Field | Value |
|---|---|
| Parent | ARCH-034-R04 |
| Finding | I181 AG-7 / ADR-008 consumer-scope drift |
| Status | Ready — documentation/validation slice defined; claim and iteration required |
| Priority | P1 |
| Selected Iteration | None |
| Preserved behavior | SQLite features, schemas, migrations, busy policy, storage paths and all runtime error/fallback behavior |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | None |
| Governance Claim PR | Not applicable |
| Authorization Mode | Independent architecture review required |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-09 |
| Handoff / Release Condition | Establish an effective review-only claim; no SQLite production code is authorized. |

## Confirmed Baseline

ADR-008 describes bundled SQLite as limited to `talos-session` and
`talos-evolution`, while the locked workspace reaches the same
`libsqlite3-sys` through five consumers: those two plus `talos-exploration`,
`talos-memory` and `talos-models`. This is policy/documentation drift; it is not
evidence that any current runtime behavior is unsafe or may be changed here.

## Scope And Acceptance

- Inventory the purpose, ownership, schema/migration surface and existing
  corrupt/busy/locked coverage of all five consumers.
- Amend or supersede ADR-008 to explicitly accept or reject each current bundled
  consumer without silently broadening future dependency authority.
- Add a validator that fails when a sixth production consumer appears without an
  accepted ADR update.
- Synchronize the decision index and architecture/dependency documentation.
- Run dependency inversion evidence with `cargo tree --locked -i libsqlite3-sys`.

## Exclusions And Residuals

No manifest/dependency, schema, migration, timeout, retry, panic handling, storage
path or runtime code change. Any inconsistent SQLite containment discovered by
the inventory receives a separate owner and cannot be implemented under AG-7.

## Minimum Validation

ADR/document-link checks, the new consumer validator, both governance validators,
locked release preflight and independent architecture review of the exact head.
