# ARCH-034-R04-AG11: SQLite Containment Evidence

| Field | Value |
|---|---|
| Parent | ARCH-034-R04 |
| Finding | I183 five-consumer failure-containment inventory |
| Status | Refinement — unclaimed residual |
| Priority | P1 |
| Selected Iteration | None |
| Preserved behavior | SQLite schemas, migrations, retry/busy policy, storage paths, runtime fallback behavior and ADR-008 allowlist |

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
| Authorization Mode | Independent architecture review required before protected SQLite containment work |
| Authorization Evidence | I183 inventory only; no implementation authorization |
| Implementation PR | Not started |
| Last Updated | 2026-08-09 |
| Handoff / Release Condition | Refine call-family risks and split any behavior-changing remedy before selection; establish a separate effective claim. |

## Confirmed Residual

The I183 inventory confirms that focused SQLite failure evidence is uneven across the five accepted
ADR-008 consumers. Session has selected corrupt and busy/locked fixtures; memory and models have
focused corrupt-open coverage; evolution and exploration have migration/error-propagation evidence
but no focused corrupt or busy/locked fixtures. None of those facts proves a uniform panic boundary
or operation deadline for all SQLite calls.

## Refinement Scope

- Map corrupt, busy/locked, migration, panic, deadline, retry and fallback behavior per call family.
- Separate evidence-only test gaps from behavior-changing containment proposals.
- Decide whether dependency-only panic adapters or operation bounds are necessary and safe; do not
  infer one global policy from selected session fixtures.
- Preserve ADR-008's exact consumer allowlist and `talos-models` quarantine.
- Require a new iteration, effective claim and applicable architecture/security review before any
  Rust, schema, migration, retry, timeout or fallback change.

## Acceptance Before Ready

- Every five-consumer call family has an explicit caller/authority/failure/fallback matrix.
- Proposed tests distinguish current behavior evidence from behavior changes.
- Each behavior change is isolated into a bounded, reviewable child with rollback and cross-platform
  validation; no catch-all panic swallowing or silent data loss is proposed.

## Evidence

- [SQLite Consumer Inventory](../../reference/SQLITE-CONSUMER-INVENTORY.md)
- ARCH-034-R04 I181 boundary matrix and its accepted "other containment needs evidence" disposition
- I183/AG-7 changes policy truth and validation only; it does not remediate this residual
