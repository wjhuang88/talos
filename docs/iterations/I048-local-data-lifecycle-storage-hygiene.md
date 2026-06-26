# I048: Local Data Lifecycle And Storage Hygiene

**Status**: Planned
**Created**: 2026-06-26
**Depends On**: I047 release publication; DATA-001; SESSION-002; MEM-001-A

## Objective

Give every Talos-owned local persistent data surface a visible lifecycle before I019 connects
automatic semantic/procedural memory writes to the runtime.

## Non-Terminal Inventory

| Iteration | Current State | Disposition |
|---|---|---|
| I011 | Paused | Not reopened. |
| I018 | Planned | OBS-001 slice was fulfilled inside I047; published baseline remains preserved. |
| I019 | Planned | Full activation should wait for DATA-001 unless a change-control exception is recorded. |
| I020 | Planned | Remains blocked/deferred until I019 or explicit research-priority replan. |
| I028 | Planned | Deferred; scheduling is unrelated to storage lifecycle. |
| I047 | Review | `v0.1.2` tag has been pushed; release workflow evidence remains to be recorded before Complete. |

## Selected Story

- [ ] `DATA-001`: Local Data Lifecycle And Storage Hygiene

## Slices

| Slice | Deliverable | Verification |
|---|---|---|
| I048-S1 | Storage inventory/status command reports sessions, index/WAL, logs, cache, and memory DB sizes without writing files. | Unit tests for missing/partial/full local data roots. |
| I048-S2 | Session cleanup dry-run/apply removes stale sessions while protecting the active session and keeping the index synchronized. | Temp-dir tests for dry-run, apply, active protection, and index row deletion. |
| I048-S3 | Fork storage visibility shows fork/source relationships and storage cost. | Session manager/index tests with fork metadata. |
| I048-S4 | SQLite maintenance API performs explicit checkpoint/vacuum for session index and memory DB. | Tests prove commands run and errors are propagated without data loss. |
| I048-S5 | Memory lifecycle gate enables foreign keys, prevents orphan evidence, and exposes retention dry-run policy shape. | `talos-memory` tests for foreign-key enforcement and retention candidate reporting. |

## Pre-Activation Foundation

2026-06-26: Before activating this planned iteration, a bounded library-level foundation landed
for the highest-risk storage correctness concerns:

- session cleanup candidate/apply APIs now remove JSONL and index rows together while honoring
  protected session IDs;
- session index and memory DB explicit checkpoint/truncate and vacuum APIs exist;
- memory DB connections enable foreign keys, and orphan evidence links are rejected;
- manual compaction failure preserves the original message list.

This is intentionally recorded as foundation evidence, not I048 completion. I048 remains Planned
until the storage status command, CLI cleanup workflow, active-session command protection, fork
visibility, and memory retention dry-run are implemented and validated.

## Scope

- Add read-only storage visibility before adding destructive cleanup behavior.
- Keep cleanup manual and explicit; no automatic deletion by default.
- Keep data ownership in storage crates:
  - `talos-session` owns sessions, forks, and session index maintenance.
  - `talos-memory` owns memory DB lifecycle and evidence integrity.
  - `talos-cli` owns command dispatch and presentation only.
- Keep raw session JSONL as the durable source of truth.
- Treat memory cleanup as maintenance/retention, not semantic overwrite.

## Non-Goals

- No copy-on-write session forks.
- No autonomous memory consolidation or prompt injection.
- No new storage backend or vector/graph dependency.
- No background daemon or scheduled cleanup.

## Acceptance

- Given a fresh machine with no `~/.talos` directory
  When the storage status command runs
  Then it exits successfully and reports missing/zero-size surfaces.

- Given multiple sessions in one workspace
  When cleanup runs in dry-run mode
  Then it reports deletion candidates without removing JSONL or index rows.

- Given cleanup apply mode
  When a stale non-active session is selected
  Then its JSONL file, FTS rows, session metadata, and fork rows are removed.

- Given the active session
  When cleanup or delete policy evaluates it
  Then it is protected.

- Given a memory evidence link for a missing memory item
  When insertion is attempted
  Then the database rejects it.

- Given I019 activation planning
  When memory automatic writes are selected
  Then DATA-001 completion or an explicit change-control exception is required.

## Validation Plan

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- `scripts/validate_project_governance.sh .`

## Residual Destination

- Copy-on-write fork storage, automatic cleanup scheduling, and richer memory archive/export remain
  future stories unless I048 evidence shows they are required for correctness.
