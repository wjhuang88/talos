# DATA-001: Local Data Lifecycle And Storage Hygiene

| Field | Value |
|-------|-------|
| Story ID | DATA-001 |
| Priority | P1 |
| Status | Planned |
| Depends On | I047 release closeout; MEM-001-A starter; SESSION-002 delete/reconcile support |
| Origin | Post-I047 storage review, 2026-06-26 |

## Problem

Talos now has multiple local persistent data surfaces:

- session JSONL files and the session SQLite FTS index;
- forked session snapshots;
- bounded log files;
- model catalog cache;
- the new semantic/procedural `talos-memory` SQLite store.

Logs and model cache have clear storage bounds or overwrite behavior, and sessions have manual
delete/reconcile paths. However, sessions and memory do not yet have a full lifecycle policy:
there is no storage status command, retention dry-run, SQLite maintenance path, or memory cleanup
gate before automatic memory consolidation is connected to the agent runtime.

Without a first-class data lifecycle, long-running use can create unbounded but reachable local
state. That is not an orphan-file leak, but it is still a product risk before I019 enables
autonomous memory writes.

## Scope

Define and implement local data lifecycle controls for Talos-owned persistent state.

Required capabilities:

- Read-only storage status for local Talos data:
  - session directory total size;
  - session count by workspace;
  - largest sessions;
  - forked-session count and size visibility;
  - session `index.db`, `-wal`, and `-shm` sizes;
  - log directory size and configured rotation bound;
  - model cache size;
  - memory database size when present.
- Session cleanup policy:
  - dry-run and apply modes;
  - protect the active session;
  - support workspace-scoped limits such as max session count and max age;
  - delete JSONL and SQLite index rows together.
- Fork visibility:
  - show fork/source relationship where known;
  - surface fork storage cost in storage status;
  - do not implement copy-on-write in this story.
- SQLite maintenance:
  - checkpoint/truncate WAL where safe;
  - expose manual vacuum/maintenance API for session index and memory DB;
  - avoid automatic heavy maintenance during normal turns.
- Memory lifecycle gate:
  - enable `PRAGMA foreign_keys = ON`;
  - prevent orphan evidence links;
  - add retention policy shape before I019 writes memory automatically;
  - provide dry-run cleanup for memory items without violating ADR-016 ADD-only consolidation
    semantics.

## Non-Goals

- Do not delete user data automatically by default.
- Do not replace JSONL as the durable session source of truth.
- Do not implement copy-on-write fork storage.
- Do not connect autonomous memory consolidation or prompt injection; that remains I019.
- Do not add a new storage backend or vector database.

## Acceptance Criteria

- [ ] A read-only storage status command reports Talos-owned local storage sizes and tolerates
      missing directories.
- [ ] Session cleanup supports dry-run and apply modes.
- [ ] Cleanup refuses to delete the active session.
- [ ] Cleanup removes both session JSONL and associated SQLite index/fork rows.
- [ ] Forked sessions are visible in storage status.
- [ ] Session SQLite maintenance can checkpoint WAL and vacuum through an explicit command/API.
- [ ] `talos-memory` enables SQLite foreign-key enforcement.
- [ ] Evidence insertion for a nonexistent memory ID fails.
- [ ] Memory cleanup policy supports dry-run and is documented as maintenance, not semantic
      overwrite.
- [ ] I019 activation docs explicitly depend on DATA-001 or record a change-control exception.

## Suggested Slices

| Slice | Deliverable | Notes |
|---|---|---|
| DATA-001-A | `storage status` read-only report | Diagnostic foundation; no writes. |
| DATA-001-B | Session cleanup dry-run/apply | Manual only; active session protected. |
| DATA-001-C | Fork storage visibility | Expose cost before optimizing representation. |
| DATA-001-D | SQLite checkpoint/vacuum maintenance | Explicit command/API only. |
| DATA-001-E | Memory lifecycle gate | Foreign keys, orphan prevention, retention dry-run. |

## Pre-Activation Foundation Evidence

2026-06-26 foundation work landed the storage-crate safety primitives needed before the full I048
CLI/user-facing slice activates:

- `talos-session` exposes explicit cleanup candidate/apply APIs with workspace scoping,
  protected session IDs, JSONL deletion, and index-row cleanup.
- `talos-session` exposes explicit session-index checkpoint/truncate and vacuum APIs.
- `talos-memory` enables SQLite foreign-key enforcement at connection open time and rejects
  evidence links for nonexistent memory rows.
- `talos-memory` exposes explicit checkpoint/truncate and vacuum APIs.
- `talos-agent` manual compaction failure now preserves the original message list instead of
  returning an empty continuation payload.

This does not close DATA-001. The user-facing storage status command, active-session protection
at command invocation, fork visibility, and memory retention dry-run remain in the planned I048
acceptance boundary.

## Required Reads

- `docs/iterations/I048-local-data-lifecycle-storage-hygiene.md`
- `docs/iterations/I047-v012-release-readiness-and-runtime-polish.md`
- `docs/backlog/active/MEM-001-layered-memory-foundation.md`
- `docs/backlog/active/MEM-005-context-compaction-policy.md`
- `docs/backlog/active/SESSION-002-session-integrity-lifecycle-hardening.md`
- `docs/decisions/002-local-storage-architecture.md`
- `docs/decisions/008-sqlite-bundled-storage.md`
- `docs/decisions/016-layered-memory-architecture.md`
- `crates/talos-session/src/`
- `crates/talos-memory/src/lib.rs`
- `crates/talos-cli/src/`
