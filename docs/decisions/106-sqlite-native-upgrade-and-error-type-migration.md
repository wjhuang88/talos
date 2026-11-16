# ADR-106: SQLite Native Upgrade and Error Type Migration

Status: Proposed — isolated implementation and compatibility validation pending.
Owner: [Session archive / recovery emergency](../tasks/session-archive-recovery-emergency.md).

## Context and evidence

The user authorized upgrading SQLite to latest stable on 2026-11-16 while preserving
existing data. The observed non-yanked registry versions are rusqlite 0.43.0 and
libsqlite3-sys 0.40.0, bundling SQLite 3.54.1, replacing 0.40.2 / 0.38.2 / 3.53.2.
Registry package checksums were verified before inspecting the sources. Their MSRVs
(1.89 and 1.88) fit Talos's 1.95 minimum and pinned 1.99.0 development toolchain.

[SQLite upstream fix](https://github.com/sqlite/sqlite/commit/2429af2e0ce2e4c643786f8d2533f79d974e3270)
corrects long DOS-device paths being misclassified as UNC and selecting shared SHM
handles. The old bundled source lacks this fix; 3.54.1 contains it. This supports
investigation of native concurrent-open SQLITE_PROTOCOL failures, not proof that all
observed failures have that cause. The ordinary/canonical native probe remains required.

## Decision

Upgrade all five direct consumers together: Session, Memory, Models, Evolution and
Exploration. Keep current default features and bundled/backup/hooks selections. The
single native sqlite3 links boundary cannot support competing sys versions. Retain
ADR-008's approved bundled native boundary and existing panic/error containment.
Do not strip production path prefixes, relax retry limits, add unsafe code, rewrite
schemas or erase original data as part of this dependency change.

This is a public dependency-type migration, not a semver-neutral internal update.
PendingSubmissionError::Storage, MemoryStoreError::Database,
ExplorationError::Database and CatalogError::Database expose rusqlite::Error.
IndexStoreError/IndexError, TodoError and StoreError/EvolutionError also expose conversions from
that type. A 0.40 Error and a 0.43 Error are distinct downstream Rust types.

New rusqlite requires exclusive connection access when replacing progress handlers.
The private budget guard holds a mutable connection borrow and exposes shared SQL
access through Deref. Statements/transactions finish before it clears the handler
and restores the original busy timeout. No permanent shared callback state is added.

Public SessionIndex receiver migration includes save_recovery_continuation,
recovery_continuation, clear_recovery_continuation, replace_recovery_owner_scope,
recovery_owner_scope_ready, begin_recovery_build, recovery_build_cursor,
recovery_candidates, recovery_terminal_candidate, init_recovery_schema,
init_recovery_schema_with_budget, recovery_candidate_text,
recovery_candidate_text_with_source_budget and record_fork_with_budget (14 methods).
The related record_fork already required a mutable receiver before this upgrade;
it is not an additional API break.
Callers must hold mutable indexes for these operations. Final compile/API review
must verify this inventory against the candidate diff before acceptance.

## Consumer and data migration

Downstream Rust consumers must update their direct rusqlite requirement to 0.43,
resolve one compatible libsqlite3-sys, rebuild, and validate error construction,
matching and question-mark conversions. Retain typed errors; changing them to strings
would introduce an unrelated API break. Compile an external consumer fixture covering
these boundaries before accepting the candidate.

A release containing this breaking dependency-type migration requires a pre-1.0 minor
version transition (at least 0.11.0 from 0.10.x), synchronized across internal package
requirements under the standard release process. It must not ship as a compatible
0.10.x patch. This candidate does not authorize a release or change the workspace
version in advance of the release owner.

Validate copies created by the old bundled SQLite, open/query/write with the new
version, then reopen with the old version and check rows, triggers, integrity and
existing session recovery behavior. Include WAL/checkpoint and partial-schema cases.
Never test migration by modifying the user's original databases or preserved backups.
No persistent schema change is intended; compatibility is unproven until this gate passes.

## Acceptance and reversal

Require locked full preflight, all-target/all-feature checks and Clippy, all-feature
workspace tests, affected minimal/default feature cases, deterministic dependency-audit
checks, independent native/API review, and exact-head Windows/macOS CI. Compare the
same path probe before and after upgrade; do not equate a lone green run with causality.
Record actual results and remaining host/real-provider acceptance in the owner.

If native failures persist, keep their evidence and investigate without retry inflation.
If API or data compatibility fails, repair the candidate or restore manifests and
Cargo.lock together. Do not overwrite user data during source rollback. Preserve old
copies until native and installed-host acceptance are established. Acceptance requires
implementation/review evidence; this proposal alone does not close the incident.
