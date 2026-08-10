# ARCH-034-R04-AG7: Bundled SQLite ADR Reconciliation

| Field | Value |
|---|---|
| Parent | ARCH-034-R04 |
| Finding | I181 AG-7 / ADR-008 consumer-scope drift |
| Status | Review — implementation submitted in PR #184 from effective claim merge `7e61454061a9c9df0f7619935fa78397bfbd6f97` |
| Priority | P1 |
| Selected Iteration | I183 (Review) |
| Preserved behavior | SQLite features, schemas, migrations, busy policy, storage paths and all runtime error/fallback behavior |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 architecture session 2026-08-09 |
| Work Slice | Implement only I183/ARCH-034-R04-AG7: inventory the five current direct workspace consumers of bundled SQLite (four runtime consumers plus quarantined non-runtime `talos-models`), reconcile ADR-008 and architecture/decision references to an exact accepted allowlist with that distinction, and add a cross-platform validator over parsed locked Cargo metadata that counts a workspace package only when it has a direct edge to a non-workspace package that transitively reaches `libsqlite3-sys`, rejects an unexpected or missing accepted consumer, rejects multiple resolved `rusqlite`/`libsqlite3-sys` versions, and rejects any workspace package that depends on `talos-models`; wire it into standard governance validation and synchronize owner/iteration/Board evidence; do not change Cargo manifests, Cargo.lock, dependencies, Rust source, schemas, migrations, database behavior, the `talos-models` quarantine or any other R04 child. |
| Claimed At | 2026-08-09 |
| Source Issue | None |
| Governance Claim PR | #183 |
| Authorization Mode | Independent review |
| Authorization Evidence | Review `5231879125` approved exact head `360576c9c32f5335c36185368051152432ad6e5a`; re-review `5231992214` correctly rejected amended head `0284f0f334a3f9dd85e251edb9d04e19e05936af`; final independent re-review `5232111621` approved exact head `17ca8c9f97b35ff9973639c028fd6b69121846e3` with no remaining findings and disclosed that a distinct natural-person reviewer used the shared `@wjhuang88` account. Exact-head CI `31318602990` passed all four jobs; merge-time CAS passed against `main@20a09a473c10eb077759275eaa395f769cdd1854`; PR #183 merged at `7e61454061a9c9df0f7619935fa78397bfbd6f97`. |
| Implementation PR | #184 |
| Last Updated | 2026-08-10 |
| Handoff / Release Condition | Implement only from claim merge `7e61454061a9c9df0f7619935fa78397bfbd6f97` or later `main`; no SQLite production code is authorized by this claim. |

## Confirmed Baseline

ADR-008 describes bundled SQLite as limited to `talos-session` and
`talos-evolution`, while the locked workspace reaches the same
`libsqlite3-sys` through five direct workspace consumers: four runtime crates
(`talos-session`, `talos-evolution`, `talos-exploration`, and `talos-memory`) plus
quarantined non-runtime `talos-models`. Because Hard Constraint #1 permits native dependencies only
through an ADR-recorded exception, the three consumers omitted by ADR-008 are a live policy
non-conformance, not merely stale prose. That fact is not evidence that current runtime behavior is
unsafe and does not authorize changing runtime behavior here; each consumer must instead be
explicitly accepted or rejected by the reconciled decision.

## Scope And Acceptance

- Inventory the purpose, ownership, schema/migration surface and existing
  corrupt/busy/locked coverage of all five consumers.
- Amend or supersede ADR-008 to explicitly accept or reject each current bundled
  consumer without silently broadening future dependency authority.
- Add a validator that uses the resolved dependency graph from parsed, locked Cargo metadata to
  identify each workspace package with a direct edge to a non-workspace package that transitively
  reaches `libsqlite3-sys`, regardless of whether the boundary edge arrives through `rusqlite`,
  `sqlx`, `libsql`, or another native dependency. Reachability only through other workspace packages
  is layering, not an additional consumer.
- Include resolved normal, build, development and target-specific dependency edges, and fail on
  either an unexpected boundary consumer or removal of an accepted consumer, multiple resolved
  `rusqlite` or `libsqlite3-sys` versions, or any workspace package with a direct or transitive
  dependency on the quarantined `talos-models` crate.
- Synchronize the decision index and preserve the already-correct 4+1 classification in
  `ARCHITECTURE.md`, adding only the accepted allowlist/validator pointer there.
- Run dependency inversion evidence with `cargo tree --locked -i libsqlite3-sys`.

## Exclusions And Residuals

No manifest/dependency, schema, migration, timeout, retry, panic handling, storage
path or runtime code change. Any inconsistent SQLite containment discovered by
the inventory is owned by unclaimed AG-11 and cannot be implemented under AG-7.

## Minimum Validation

ADR/document-link checks, positive plus alternate-edge/missing-consumer/duplicate-version/quarantine
negative fixtures for the new metadata-graph validator on Unix and Windows, both governance
validators, locked release preflight and independent architecture review of the exact head.
