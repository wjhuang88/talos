# I044: Session Integrity And Lifecycle Hardening

> Document status: Active
> Published plan date: 2026-06-23
> Planned close date: 2026-07-07 (≈ 2 weeks)
> Planned objective: Fix six transaction-consistency defects in session
>   persistence and implement session deletion. Session writes become O(1),
>   crash-safe, race-free, and reconcilable. Users gain `/delete`.

## Selected Stories

| Story | Parent | Priority | Outcome |
|---|---|---|---|
| SESSION-002-A | SESSION-002 | P0 | O(1) append: last_entry_id in memory, no full-file re-read |
| SESSION-002-B | SESSION-002 | P0 | Concurrent write safety: per-file mutex + parent_id atomicity |
| SESSION-002-C | SESSION-002 | P1 | Crash reconciliation: startup scan repairs stale SQLite from JSONL |
| SESSION-002-D | SESSION-002 | P1 | Session switch ordering: old events write to old session |
| SESSION-002-E | SESSION-002 | P1 | Failure cleanup: orphan JSONL removed on prepare/commit failure |
| SESSION-002-F | SESSION-002 | P1 | Session deletion: /delete command + SessionManager method + SQLite cleanup |

## Execution Order

```
Week 1:
  SESSION-002-A (O(1) append) ─── 1-2 days
         │
  SESSION-002-B (write safety) ─── 1-2 days (builds on A's last_entry_id)
         ∥
  SESSION-002-E (failure cleanup) ─── 1 day (independent)

Week 2:
  SESSION-002-C (reconciliation) ─── 2 days
  SESSION-002-D (switch ordering) ─── 2 days
  SESSION-002-F (deletion) ─── 2 days
  Closure + verification ─── 1 day
```

## Scope

### SESSION-002-A: O(1) Append

**Problem**: `Session::append()` calls `self.read_entries()` to get the last entry's ID for the
`parent_id` chain. This reads the entire JSONL file on every append — O(n) per write, O(n²) per
session.

**Fix**:
- `Session` gains `last_entry_id: std::sync::Mutex<Option<String>>`
- `append()` and `append_event()` read `last_entry_id` from the mutex instead of `read_entries()`
- After writing, update `last_entry_id` to the new entry's ID
- On first use (or after `resume_session`), populate `last_entry_id` by reading only the last line
  of the file (`seek(SeekFrom::End(0))` + backward scan to last newline)

### SESSION-002-B: Concurrent Write Safety

**Problem**: Bridge forwarder and user persister each hold `Session::clone()` and call `append()`
concurrently. Both read the same `parent_id`, producing duplicate chain links. `writeln!` may
interleave across two file handles.

**Fix**:
- `Session` gains `write_lock: Arc<std::sync::Mutex<()>>` cloned across `Session::clone()`
- `append_entry()` acquires `write_lock` before reading `last_entry_id` and writing the line
- Within the lock: read `last_entry_id` → construct entry → write line → update `last_entry_id`
- This makes append atomic per-session across all clones

### SESSION-002-C: Crash Reconciliation

**Problem**: JSONL write and SQLite `update_index` are not atomic. A crash between them leaves
the index stale (wrong message_count, wrong preview). Or: orphan SQLite entry if JSONL is deleted.

**Fix**:
- `SessionManager::reconcile_index()` — called once at startup
- For each `*.jsonl` in all workspace dirs:
  - Scan file for message count and last preview
  - Compare with SQLite index entry
  - If missing or stale: re-index from JSONL
- For each SQLite entry without a matching JSONL file: delete the entry
- JSONL remains source of truth (ADR-002); SQLite is always rebuildable

### SESSION-002-D: Session Switch Ordering

**Problem**: After `transition.commit()`, `session_watch_tx.send(new_session)` updates the shared
session before the bridge forwarder finishes draining old-actor events. Old events land in the new
session's JSONL.

**Fix**:
- `handle_session_new/resume/fork` sends the `old_session` from `CommitResult` to the bridge
  forwarder alongside the new `eq_rx` via a new channel message:
  `SessionSwitch { old_session, new_eq_rx }`
- Bridge forwarder receives this, stores `old_session` as the write target for remaining events
- Bridge uses `old_session` while `current_eq_rx.recv()` returns events
- When `current_eq_rx` returns `None`, bridge switches to the watch-channel session for future
  events from the new `eq_rx`
- This closes the race window

### SESSION-002-E: Failure Cleanup

**Problem**: `/fork` and `/new` create JSONL files before `transition.commit()`. If commit fails,
orphan files remain on disk forever.

**Fix**:
- In each lifecycle handler's error path (prepare failure, commit failure after rollback):
  ```rust
  let _ = std::fs::remove_file(&new_session.file_path);
  ```
- Best-effort: log warning if removal fails, don't block the error response
- Applies to: `handle_session_new`, `handle_session_resume`, `handle_session_fork`

### SESSION-002-F: Session Deletion

**Problem**: No `/delete` command, no `SessionManager::delete_session()`, no cleanup path.

**Fix**:
- `SessionManager::delete_session(id: &Uuid) -> Result<(), SessionError>`:
  - `std::fs::remove_file(jsonl_path)`
  - SQLite: `DELETE FROM sessions WHERE session_id = ?`
  - SQLite: `DELETE FROM forks WHERE parent_id = ? OR child_id = ?`
  - Refuse if session is the active session (caller checks before calling)
- `/delete` BuiltinCommand in conversation engine:
  - `/delete` with no args: list workspace sessions with ordinals (like `/resume`)
  - `/delete <N>`: resolve ordinal, show confirmation in bottom panel
  - Confirmation panel: "Delete session N? This cannot be undone." → [y] confirm / [n] cancel
  - On confirm: send `SessionLifecycleRequest::Delete(session_id)` to mode runner
  - Mode runner: validate not active session, call `delete_session`, send result message
- `SessionLifecycleRequest::Delete(DeleteRequest { session_id: String })` variant

## Non-Goals

- Cross-workspace session management
- Session rename
- Batch cleanup / auto-GC policy
- Soft-delete / trash / undo

## Acceptance

- `Session::append()` does not call `read_entries()` — verified by test
- Concurrent appends from two threads produce sequential parent_id chain — verified by test
- Process restart after simulated crash reconciles SQLite index — verified by test
- After `/fork`, old-actor trailing events appear in OLD session JSONL, not new — verified by test
- Failed `/new` does not leave orphan JSONL file — verified by test
- `/delete <N>` removes session; `/delete` on active session returns error — verified by test
- `/delete` shows confirmation panel — verified by test
- `cargo test --workspace` passes
- `cargo clippy --workspace -- -D warnings` clean

## Verification

- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- Unit tests: O(1) append benchmark, concurrent append chain integrity, reconciliation, orphan cleanup, delete
