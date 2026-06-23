# SESSION-002: Session Integrity And Lifecycle Hardening

| Field | Value |
|---|---|
| Story ID | SESSION-002 |
| Type | Epic |
| Priority | P1 |
| Status | Refinement |
| Depends On | SESSION-001-A/B/C complete (I040/I041/I042) |
| Origin | Architecture analysis 2026-06-23 — transaction consistency audit |

## Outcome

Session persistence is free of data races, orphan files, stale indexes, and unbounded read
amplification. Users can delete sessions interactively. All write paths are crash-safe and
reconcilable.

## Problem

Six concrete consistency defects were identified in the session persistence layer after the I041/I042
session lifecycle work:

1. **O(n²) append**: every `Session::append()` reads the entire JSONL file to determine
   `parent_id`. A 1000-message session reads 1000 lines per append.
2. **Concurrent append race**: bridge forwarder and user persister both hold `Session::clone()`
   and call `append()` independently. Both read the same `parent_id`, producing duplicate chain
   links.
3. **JSONL ↔ SQLite non-atomic**: message append and `update_index` are separate operations
   without a transaction boundary. A crash between them leaves the index stale.
4. **Session switch race**: after `transition.commit()`, `session_watch_tx.send(new_session)`
   can update the shared session before the bridge forwarder finishes draining the old actor's
   event queue. Old-actor events get written to the new session's JSONL.
5. **No file-level write serialization**: multiple clones of the same `Session` write to the
   same JSONL path via `OpenOptions::append(true)`. `writeln!` may decompose into multiple
   `write()` calls, risking line interleaving.
6. **No failure cleanup**: `/fork` and `/new` handlers create JSONL files before
   `transition.commit()`. If commit fails, the orphan file is never cleaned up.

Additionally, session deletion is completely unimplemented — there is no `/delete` command, no
`SessionManager::delete_session()` method, and no SQLite cleanup path.

## Child Stories

| Child | Outcome | Priority |
|---|---|---|
| SESSION-002-A | O(1) append: eliminate full-file re-read; maintain last_entry_id in memory | P0 |
| SESSION-002-B | Concurrent write safety: serialize appends per file; fix parent_id race | P0 |
| SESSION-002-C | Crash reconciliation: startup scan repairs stale SQLite index from JSONL source of truth | P1 |
| SESSION-002-D | Session switch ordering: old-actor events write to old session, not new | P1 |
| SESSION-002-E | Failure cleanup: orphan JSONL files removed on prepare/commit failure | P1 |
| SESSION-002-F | Session deletion: `/delete` command + SessionManager::delete_session + SQLite cleanup | P1 |

## Scope

### SESSION-002-A: O(1) Append

- `Session` gains `last_entry_id: Mutex<Option<String>>` field
- `append()` and `append_event()` use the in-memory `last_entry_id` instead of `read_entries().last()`
- On first append (or after `read_messages`), populate `last_entry_id` from the file tail
- Remove the `read_entries()` call from the append hot path entirely

### SESSION-002-B: Concurrent Write Safety

- Add `file_lock: Arc<Mutex<()>>` to `Session`, keyed by `file_path`
- `append_entry()` acquires the lock before writing, ensuring serialized appends across clones
- `parent_id` is read from the locked `last_entry_id` and updated atomically within the lock
- This eliminates both the chain-link race and the line-interleaving risk

### SESSION-002-C: Crash Reconciliation

- New method `SessionManager::reconcile_index()` — called at startup
- Scans all workspace directories for `*.jsonl` files
- For each file: if SQLite index entry is missing or stale (wrong message_count, wrong preview),
  re-index from the JSONL content
- Orphan SQLite entries (index exists, file does not) are deleted
- JSONL remains the source of truth (ADR-002); SQLite is always rebuildable

### SESSION-002-D: Session Switch Ordering

- `SessionTransition::commit()` returns `CommitResult { old_session, new_handle }`
- Lifecycle handler sends `old_session` to bridge forwarder alongside the new `eq_rx`
- Bridge forwarder uses `old_session` while draining old actor's remaining events
- Only after `eq_rx` returns `None` (old actor fully drained), switch to the watch-channel session
- This closes the race window where old events land in the new session's JSONL

### SESSION-002-E: Failure Cleanup

- In `handle_session_new`, `handle_session_resume`, `handle_session_fork`:
  - If `transition.prepare()` fails: remove the newly created session's JSONL file
  - If `transition.commit()` fails (after rollback): remove the JSONL file and any SQLite index entry
- Cleanup is best-effort (log warning on failure, don't block the error path)

### SESSION-002-F: Session Deletion

- `SessionManager::delete_session(id: &Uuid)`:
  - Remove JSONL file from disk
  - Delete SQLite index entry
  - Delete fork table entries referencing this session
  - Refuse if the session is the active session (return error)
- `/delete` BuiltinCommand:
  - Accept ordinal (like `/resume`) or session ID
  - Show confirmation prompt in bottom panel (Approval panel kind)
  - On confirm: call `delete_session`, show result message
  - On deny: cancel, no action
- Protect active session: `/delete` on the current session returns an error

## Non-Goals

- Cross-workspace session management
- Session rename
- Batch session cleanup / GC policy (deferred)
- Session export to non-JSONL formats

## Acceptance Criteria

- [ ] `Session::append()` does not call `read_entries()` — O(1) per append
- [ ] Concurrent appends from two tasks produce correct sequential chain links
- [ ] Process crash between JSONL write and SQLite update is reconciled on next startup
- [ ] After `/new`, `/resume`, or `/fork`, old-actor events persist to the OLD session's JSONL
- [ ] Failed `/fork` or `/new` does not leave orphan JSONL files
- [ ] `/delete <N>` removes session from disk and SQLite; refuses active session
- [ ] `/delete` requires confirmation in the bottom panel
- [ ] `cargo test --workspace` passes with new tests for each fix
- [ ] `cargo clippy --workspace -- -D warnings` clean

## Required Reads

- `crates/talos-session/src/jsonl.rs` — append/read logic
- `crates/talos-session/src/manager.rs` — SessionManager
- `crates/talos-session/src/sqlite.rs` — SQLite index
- `crates/talos-cli/src/mode_runners.rs` — lifecycle handlers + bridge forwarder
- `crates/talos-cli/src/session_transition.rs` — SessionTransition
- `docs/decisions/002-local-storage-architecture.md` — JSONL as source of truth
