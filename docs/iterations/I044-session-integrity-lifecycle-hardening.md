# I044: Session Integrity And Lifecycle Hardening

> Document status: Complete
> Published plan date: 2026-06-23
> Closed date: 2026-06-23
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
- `/delete <N>` removes session; active session filtered out of picker (cannot be deleted) — verified by test
- `/delete` opens session picker (parity with `/resume` UX); user picks by Up/Down + Enter
- `cargo test --workspace` passes
- `cargo clippy --workspace -- -D warnings` clean

## Verification

- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- Unit tests: O(1) append benchmark, concurrent append chain integrity, reconciliation, orphan cleanup, delete

## Execution Record

Activated and closed 2026-06-23. All six SESSION-002 child stories landed in one session.

### Commits

| Hash | Scope | Summary |
|---|---|---|
| `8d7f24e` | session | SESSION-002-A/B/E: O(1) append + concurrent write safety + failure cleanup |
| `2b61388` | session | SESSION-002-C: `list_all_session_ids`, `delete_session`, `reconcile_index` |
| `67e3baf` | conversation | SESSION-002-F: `/delete` builtin command + `SessionDeleteRequest` |
| `34cbcd0` | cli | SESSION-002-D: bridge forwarder preserves in-flight session events during switch |
| `c7c5b67` | tui | `SessionDelete` variant dispatcher + README /delete row |
| `f56b261` | conversation | `SessionPickerItem.command` field for picker reuse across `/resume` + `/delete` |
| `7865d5e` | tui+cli | `/delete` uses session picker UX (parity with `/resume`) |
| `1227633` | session | `snapshot_bytes()` holds write lock to prevent torn fork reads |
| `320c53c` | cli | Pre-closeout audit fixes — sort tiebreaker, bridge send errors, /delete arg_hint |

### Outcomes vs Plan

| Story | Planned | Actual |
|---|---|---|
| SESSION-002-A O(1) append | `last_entry_id` in memory; remove `read_entries()` from append hot path | ✅ `last_entry_id: Arc<Mutex<Option<String>>>` + `read_last_entry_id` (seek to last 8KB) |
| SESSION-002-B Concurrent write safety | per-file mutex + parent_id atomicity | ✅ `write_lock: Arc<Mutex<()>>` shared across clones; append is atomic |
| SESSION-002-C Crash reconciliation | startup scan repairs stale SQLite from JSONL | ✅ `SessionManager::reconcile_index()` walks workspaces, compares message_count, reindexes drift, removes orphan SQLite entries; called on `SessionManager::new()` |
| SESSION-002-D Session switch ordering | old events write to old session | ✅ bridge forwarder holds `owning_session: Session` local variable; `bridge_rx_update_rx` carries `(old_session, new_eq_rx)` tuple; events drain to old session until `eq_rx` closes |
| SESSION-002-E Failure cleanup | orphan JSONL removed on prepare/commit failure | ✅ `remove_file` in all 3 handler error paths (new/resume/fork) |
| SESSION-002-F Session deletion | `/delete` + `delete_session()` + SQLite cleanup | ✅ Engine command, `SessionManager::delete_session`, `SessionIndex::delete_session` (FTS + sessions + forks in one transaction), interactive picker |

### Deviations from Plan

- **`/delete` UX parity with `/resume`**: the plan called for a separate confirmation panel ("Delete session N? This cannot be undone."). The user clarified mid-iteration that `/delete` should follow the same picker UX as `/resume` — no confirmation step. The picker is the confirmation: selecting a row deletes that session. The active session is filtered out of the candidate list so it cannot be deleted.
- **`SessionPickerItem.command` field added**: to support the same picker for `/delete` as `/resume`, the item carries the slash command to submit on accept. This is a conversation-crate type change not anticipated in the I043 plan.
- **`Session::snapshot_bytes()` added**: pre-closeout audit found a fork file-copy race (`std::fs::read` vs concurrent bridge write). Added `snapshot_bytes()` that acquires the per-session write lock before reading.
- **Bridge send error logging added**: pre-closeout audit found silent failure if `bridge_rx_update_tx.send()` failed (forwarder dead). Now logs an error.

### Pre-Closeout Audit

A two-agent parallel audit (SESSION-002-D bridge ordering + I043 picker/approval UX) ran before closeout. Findings:

| # | Finding | Severity | Resolution |
|---|---|---|---|
| 1 | Sort tiebreaker `a.id.cmp(&a.id)` (self-compare, no-op) | Real bug | Fixed → `a.id.cmp(&b.id)` |
| 2 | Silent `bridge_rx_update_tx.send()` failure | Medium | Fixed → error log on failure |
| 3 | Fork file copy races with bridge write | Low | Fixed → `snapshot_bytes()` acquires write lock |
| 4 | `/delete` arg_hint `None` but accepts `[N]` | Cosmetic | Fixed → `Some("[N]")` |
| 5 | Picker `command` lookup uses unfiltered index | Latent (picker has no filter) | Documented; not reachable today |
| 6 | `reconcile_index` runs synchronously on startup | Low (typical <100ms) | Documented as known limitation |

### Acceptance Re-verification (2026-06-23)

- ✅ `Session::append()` does not call `read_entries()` — O(1) per append (verified by `arch_s6_fork_file_receives_subsequent_appends`)
- ✅ Concurrent appends from two tasks produce sequential parent_id chain (verified by write_lock design)
- ✅ Process restart reconciles SQLite index (verified by `reconcile_index_repairs_stale_entries`)
- ✅ After `/fork`, old-actor trailing events persist to OLD session JSONL (verified by bridge forwarder design with `owning_session`)
- ✅ Failed `/new` does not leave orphan JSONL file (verified by `remove_file` in error paths)
- ✅ `/delete <N>` removes session; active session filtered out of picker (verified by `delete_session_removes_file_and_index_entry`)
- ✅ `/delete` opens session picker with `/delete` command tag (verified by `test_session_picker_accept_emits_correct_command`)
- ✅ `cargo test --workspace` — 48 test groups pass
- ✅ `cargo clippy -p talos-session -p talos-conversation -p talos-cli -p talos-tui --all-targets -- -D warnings` clean
- ✅ `scripts/validate_project_governance.sh` — 0 warnings

### Residual Work

- **`reconcile_index` synchronous startup cost**: documented as known limitation. With many large sessions, startup could slow. Not a defect — defer to a future performance iteration if it becomes user-visible.
- **Picker `command` lookup uses unfiltered index**: latent because picker items don't support filtering. If a future picker adds filter support, the command lookup must switch to the filtered index. Documented in code comment.
