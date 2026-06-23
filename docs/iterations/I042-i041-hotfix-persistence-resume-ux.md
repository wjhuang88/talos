# I042: I041 Hotfix — Persistence Continuity, /resume UX, Execute Semantics

> Document status: Complete
> Published plan date: 2026-06-22
> Closed date: 2026-06-23
> Planned objective: Fix architecture review findings from I041 and correct
>   the /resume UX requirement. P1 persistence continuity (session switch
>   must update bridge forwarder + user persister), /resume list selection,
>   P2 Execute resource semantics, P3 cleanup.

## Selected Stories

| Story | Priority | Outcome |
|---|---|---|
| P1-1 Persistence Continuity | P1 | After /new, /resume, /fork, the bridge forwarder and user persister write to the NEW session, not the old one |
| /resume UX Fix | P1 | /resume lists workspace-scoped sessions with ordinal numbers; user selects by number, not raw session UUID |
| P2-2 Execute Semantics | P2 | Execute resource extraction takes the first token of the command as a path-like string, documented |
| P2-3 active_session() removal | P2 | SessionTransition no longer exposes active_session(); fork reads source from SessionManager |
| P3 cleanup | P3 | model_context_limit from config; dead_code removal; module docs updated |

## Scope

### P1-1: Persistence Continuity

**Root cause**: `bridge_forwarder` (line ~396) and `user_persister` (line ~437) in `run_tui_mode` capture `session.clone()` and `handle.eq_rx` at startup. When `SessionTransition::commit()` swaps the active session, these tasks keep writing to the OLD session's file and listening to the OLD actor's event stream.

**Fix design**: Introduce a `tokio::sync::watch` channel for sharing the active `Session` and `sq_tx` across tasks. Bridge forwarder receives new `eq_rx` via a dedicated `mpsc::unbounded_channel`.

1. `SessionTransition::commit()` returns `CommitResult { old_session: Session, new_handle: SessionHandle }` instead of just `Session`.
2. In `run_tui_mode`, after creating the initial session/handle:
   - Create `watch::channel(session.clone())` → `(session_watch_tx, session_watch_rx)`
   - Create `watch::channel(handle.sq_tx.clone())` → `(sq_tx_watch_tx, sq_tx_watch_rx)`
   - Create `(bridge_rx_update_tx, bridge_rx_update_rx)` mpsc for eq_rx handoff
3. Bridge forwarder task restructured:
   ```rust
   let mut current_eq_rx = handle.eq_rx;
   loop {
       // Process events from current actor until it shuts down
       while let Some(event) = current_eq_rx.recv().await {
           let session = session_watch_rx.borrow().clone();
           // persist to session, forward to bridge_tx
       }
       // Actor exhausted — wait for new eq_rx from lifecycle handler
       match bridge_rx_update_rx.recv().await {
           Some(new_rx) => current_eq_rx = new_rx,
           None => break,
       }
   }
   ```
4. User persister task restructured:
   ```rust
   while let Some(msg) = user_msg_rx.recv().await {
       let session = session_watch_rx.borrow().clone();
       let sq_tx = sq_tx_watch_rx.borrow().clone();
       // persist + submit
   }
   ```
5. Lifecycle handler, after `transition.commit(actor)`:
   - `session_watch_tx.send(new_session.clone())` — update shared session
   - `sq_tx_watch_tx.send(new_handle.sq_tx.clone())` — update shared sq_tx
   - `bridge_rx_update_tx.send(new_handle.eq_rx)` — hand off new event receiver

### /resume UX Fix

**Current**: `/resume` with no args lists sessions and tells user to type `/resume <session-id>`. `/resume <id>` resumes by UUID.

**Fix**: `/resume` with no args lists workspace-scoped sessions with ordinal numbers (1, 2, 3...). `/resume <N>` resumes the Nth session in that list.

1. `handle_session_resume` when `session_id` is None: list candidates sorted most-recent-first, assign ordinals starting at 1. Store the candidate list in a `watch` or return it via a channel so the next `/resume <N>` can resolve it.
2. For MVP simplicity: `/resume <N>` re-runs the listing query, sorts identically, picks index N-1. No caching needed (listing is cheap).
3. Update the list format:
   ```
   [System] Resumable sessions for this workspace:
   [System]   1. 2026-06-22 19:20 — 15 messages — "Fix persistence layer..."
   [System]   2. 2026-06-22 15:02 — 8 messages — "Upgrade dependencies..."
   [System]   3. 2026-06-21 18:00 — 42 messages — "I040 session foundation..."
   [System] Type /resume <number> to select.
   ```
4. Update the engine's `/resume` handling to pass the argument through (already works — just needs ordinal parsing in the handler instead of UUID).

### P2-2: Execute Resource Semantics

**Current**: `ResourceExtractor::extract()` for `Execute` returns the full command string (`cargo build --release`). Glob matching `scripts/**` against this string is semantically wrong.

**Fix**: For `Execute`, extract only the first whitespace-delimited token if it looks like a path (contains `/` or `.`), otherwise return the full command string. Document this in the ResourceExtractor doc comment.

Actually, simpler and more correct: return the first token (the program/script name). For `bash scripts/deploy.sh --arg`, the resource is `scripts/deploy.sh`. For `cargo build`, the resource is `cargo`.

```rust
ToolNature::Execute => {
    input.get("command").and_then(Value::as_str).and_then(|cmd| {
        cmd.split_whitespace().next().map(String::from)
    })
}
```

Update the doc comment and tests.

### P2-3: Remove active_session()

Delete `SessionTransition::active_session()`. Fork handler reads the source session from `SessionManager` instead.

### P3-1: model_context_limit

In `handle_session_new` and `handle_session_resume`, replace hardcoded `128_000` with the value from config. Add `model_context_limit` to the lifecycle handler's captured config or pass it through.

### P3-2: dead_code cleanup

Remove `#[allow(dead_code)]` from `PreparedSession` and `has_prepared()` in session_transition.rs. Either use them or delete them.

### P3-3: Module docs

Update the top-level `//!` doc in `crates/talos-permission/src/lib.rs` to describe nature-based matching alongside legacy tool_name matching.

## Non-Goals

- Interactive TUI picker for /resume (ordinal selection is sufficient for MVP)
- ArcSwap dependency (use tokio::sync::watch instead)
- Changing the SessionHandle struct or SessionOp enum
- Changing the conversation engine's command registry pattern

## Acceptance

- After `/new`, new messages persist to the NEW session's JSONL file, not the old one.
- After `/resume`, new messages persist to the RESUMED session's file.
- After `/fork`, new messages persist to the CHILD session's file; source file unchanged.
- `/resume` with no args shows numbered list of workspace-scoped sessions.
- `/resume <N>` resumes the Nth session.
- Execute resource extraction returns first command token.
- `cargo test --workspace` passes.
- `cargo clippy --workspace -- -D warnings` clean.
- Module docs updated.

## Verification

- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- Unit test: persistence continuity after simulated session switch
- Unit test: /resume ordinal selection
- Unit test: Execute resource extraction returns first token
