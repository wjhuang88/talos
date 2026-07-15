# SESSION-005: Durable Bindings Concurrent Open Race

| Field | Value |
|-------|-------|
| Story ID | SESSION-005 |
| Status | Complete (2026-07-15) |
| Depends On | I128 (durable runtime sessions) |
| Relates To | WEB-001 P100 Start Gate |
| Origin | P100 Start Gate finding 2026-07-15 |

## Problem

`open_bindings()` in `crates/talos-session/src/durable.rs` sets `PRAGMA journal_mode=WAL`
inside `execute_batch` on every connection open. When multiple threads race to open a
newly-created bindings database simultaneously (as in the
`external_id_rejects_path_traversal_and_concurrent_opens_share_one_uuid` test under full
`--workspace` parallel load), the WAL mode initialization contends. SQLite returns
`SQLITE_BUSY` / `SQLITE_LOCKED` and the test panics. The `busy_timeout(5s)` is set before
the init batch, so each retry could theoretically block for 5s, making worst-case total
wait ~100s — not the bounded time the initial fix claimed.

The failure is load-dependent: passes in isolation and at the crate level, fails
intermittently under `cargo test --workspace --locked`.

## Scope

- Add a bounded retry loop in `open_bindings()` for `DatabaseBusy` / `DatabaseLocked`
  errors during the initialization `execute_batch`.
- `busy_timeout(5s)` is applied AFTER successful init, not before, so init retries
  fail fast (bounded 20 × 25 ms ≤ 500 ms total).
- No `unwrap()`, `expect()`, or `panic!()` in the retry path; exhaustion returns a
  structured `SessionError::DurableTurn`.
- Deterministic tests: (a) non-BUSY error returns immediately without retrying;
  (b) persistent write-lock causes retry exhaustion returning structured error, no panic.
- No data format, public API, or behavior change.

## Acceptance

- `cargo test --workspace --locked` passes consistently (verified under repeat runs).
- The concurrent-open test passes reliably under full parallel workspace load.
- No new dependencies, no format change.

## Non-Goals

- Redesigning the durable bindings architecture.
- Changing the TLOG or session format.
- Adding a global Mutex or process-level lock.

## Verification Evidence

- **Fix (review v1)**: `crates/talos-session/src/durable.rs:open_bindings()` —
  `busy_timeout` deferred until after init; 20-iteration retry loop (25ms sleep)
  with no internal busy_timeout during init; only `DatabaseBusy`/`DatabaseLocked`
  retried; all other errors propagate immediately; exhaustion falls through to
  `Err(e) => return Err(sql_error(e))` — no panic path exists.
- `open_bindings_non_busy_error_returns_immediately`: corrupt DB file →
  `SQLITE_NOTADB` returns in <400 ms (under the 500 ms retry floor), proving
  non-BUSY errors do not engage the retry loop.
- `open_bindings_busy_exhaustion_returns_structured_error`: persistent write-lock
  via `BEGIN IMMEDIATE` → 20 retries exhausted → `SessionError::DurableTurn` returned,
  not panic.
- `external_id_rejects_path_traversal_and_concurrent_opens_share_one_uuid`: still
  passes under full `--workspace` parallel load.
- `cargo test -p talos-session --lib`: 155 passed, 0 failed; concurrent-open regression repeated five times.
- Independent acceptance reran `./scripts/release_preflight.sh` successfully, including locked workspace check, Clippy, and tests; `scripts/validate_project_governance.sh .` reported 0 warnings; `git diff --check` passed.
