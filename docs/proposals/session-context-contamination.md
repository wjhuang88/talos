# Session Context Contamination (P0)

**Status**: Proposal — P0 (blocking daily use)  
**Created**: 2026-06-02  
**Reporter**: User

## Problem

实际使用中,session/context 出现"乱串":

- 对话内容是乱的
- 让 agent 找上下文时,会出现**其他 session 的内容**(或至少是没见过的内容)
- 严重影响日常使用效率

## Symptoms

1. **Context pollution**: 当前 session 的对话中混入了不属于这个 session 的消息
2. **Cross-session leakage**: Agent 引用了其他 session 的内容
3. **Unfamiliar content**: 用户看到从未在当前 session 中发送/接收过的消息

## Possible Root Causes

### Hypothesis 1: Session Resume Loads Wrong Messages

- `talos -c` / `-r` / `/resume` 加载了错误的 session
- SQLite index 指向错误的 session ID (related to #ARCH-S5, #ARCH-S6)
- JSONL 文件路径计算错误,多个 session 写到同一个文件

**Investigation**:
- Check `SessionManager::resume()` logic
- Verify SQLite `index.db` metadata matches JSONL file paths
- Add logging to show which session file is being loaded

### Hypothesis 2: Multiple Talos Processes Write to Same Session

- 开了多个终端跑 `talos`,都 append 到同一个 session 文件
- Session ID 生成有冲突(不是真正 UUID)
- Working directory 检测错误,不同 cwd 的 talos 共享了 session

**Investigation**:
- Check `Session::new()` UUID generation
- Verify session file path includes working directory hash
- Add file locking or process-level session isolation

### Hypothesis 3: Fork Doesn't Properly Isolate

- `/fork` 后,新 turn 还是写到源 session (known bug: #ARCH-S6)
- Fork 的 `Session` clone 还带着源 session 的 identity/path
- SQLite index 没更新,后续 resume 加载源 session

**Investigation**:
- Check `event_loop.rs:326` fork logic
- Verify forked `Session` has new ID and path
- Add regression test: fork → append → verify fork file has new message

### Hypothesis 4: Context Compaction Pulls from Wrong Session

- Context compaction 时,从错误的 session 加载历史
- Compaction cache 没按 session ID 隔离
- SQLite FTS5 search 返回了其他 session 的内容

**Investigation**:
- Check `talos-agent` context compaction logic
- Verify compaction only queries current session's messages
- Add session ID filter to all SQLite queries

### Hypothesis 5: TUI and Interactive Mode Session Isolation

- TUI 模式创建的 session 在 interactive 模式看不到(或反过来)
- 两种模式用了不同的 session storage path
- Session list 没合并两种模式的 sessions

**Investigation**:
- Check `run_tui_mode` vs `run_interactive_mode` session initialization
- Verify both modes use the same `SessionManager` config
- Check if session list command filters by mode

### Hypothesis 6: SQLite Index Staleness (#ARCH-S5)

- 普通 turn 写 JSONL 后不刷新 SQLite index
- `talos -r` / `--search` 漏掉刚创建的 session
- 用户 resume 了一个"看起来对"但其实是旧的 session

**Investigation**:
- Check `SessionIndex::index_session()` call sites
- Verify index refresh after every `SessionEntry::append()`
- Add regression test: append → list → verify new session appears

## Investigation Plan

### Step 1: Reproduce with Logging

1. Enable verbose logging: `RUST_LOG=talos_session=debug,talos_agent=debug`
2. Run `talos` in TUI mode
3. Create a new session, send a few messages
4. Exit, then `talos -c` to resume
5. Check if the resumed content matches what was sent
6. Look for:
   - Which session file is being loaded
   - Which session ID is in the SQLite index
   - Whether the JSONL file path matches the index metadata

### Step 2: Check for Cross-Process Contamination

1. Open two terminals
2. In terminal A: `cd /project/a && talos`
3. In terminal B: `cd /project/b && talos`
4. Send different messages in each
5. Exit both
6. Check `~/.talos/sessions/` for:
   - Are there two separate session files?
   - Do the file paths include working directory hash?
   - Does SQLite index have two entries with correct paths?

### Step 3: Test Fork Isolation

1. Start `talos` in interactive mode
2. Send 3 messages
3. `/fork`
4. Send 2 more messages
5. Exit
6. Check:
   - Is there a fork JSONL file with the 2 new messages?
   - Does SQLite index point to the fork file?
   - Does `talos -c` resume the fork (not the source)?

### Step 4: Test Context Compaction

1. Start `talos`
2. Send 50+ messages (trigger compaction)
3. Check if compaction only uses current session's messages
4. Look for:
   - Compaction query includes session ID filter
   - Compacted context doesn't include messages from other sessions

### Step 5: Audit SQLite Queries

1. Search `talos-session/src/lib.rs` for all `SELECT` queries
2. Verify every query includes `WHERE session_id = ?`
3. Check if any query could return cross-session data
4. Add session ID filter to any query that's missing it

## Proposed Fix Strategy

### Phase 1: Add Session Management to TUI Mode (2-3 days)

**Root cause**: TUI mode has no session management, so each turn is independent and the LLM hallucinates conversation history.

**Fix**: Port session management from `run_interactive_mode` to `run_tui_mode`.

1. **Add SessionManager to TUI mode**:
   - Call `SessionManager::new()` in `run_tui_mode`
   - Support `-c` (continue), `-r` (resume), `--session <id>` flags
   - Create new session on startup if no resume flag

2. **Pass session to agent task**:
   - Store `Session` in the dispatcher task (line 529)
   - Load conversation history before calling `agent.run_streaming`
   - Append user/assistant messages to session after each turn

3. **Update Agent API**:
   - Add `Agent::run_streaming_with_history(user_message, history, event_tx)`
   - OR modify `run_inner` to accept optional `Vec<Message>` for history
   - Prepend history messages before the current user message

4. **Write session entries**:
   - After each turn, append user message + assistant response to JSONL
   - Use `session.add_user_message()` and `session.add_assistant_message()`
   - Update SQLite index (fixes #ARCH-S5)

**Verification**:
- Start TUI, send 3 messages, exit
- Restart with `talos -c`, verify all 3 messages are in context
- Ask "what did I ask you before?" - agent should correctly reference previous messages

### Phase 2: Fix Fork Identity (#ARCH-S6) (1 day)

- Update `event_loop.rs:326` fork logic
- Ensure forked `Session` has new ID, path, and metadata
- Update SQLite index after fork
- Add regression test: fork → append → verify isolation

### Phase 3: Add Session Isolation Tests (1 day)

- Multi-process test: two talos instances, verify separate sessions
- Fork test: fork → append → verify fork file
- Resume test: create → exit → resume → verify correct content
- History test: long session → verify all messages are in context

### Phase 4: Add Diagnostic Logging (0.5 day)

- Log session file path on load
- Log session ID on every turn
- Log message count in history
- Add `--debug-session` flag to dump session state

## Success Criteria

1. **TUI mode has conversation continuity**: Agent remembers all previous messages in the session
2. **No LLM hallucinations**: Agent never fabricates false memories about previous conversations
3. **Correct resume**: `talos -c` / `-r` always loads the expected session with full history
4. **Fork isolation**: `/fork` creates a fully independent session
5. **Multi-process safety**: Multiple talos instances don't interfere
6. **Index freshness**: `talos -r` / `--search` always shows recent sessions

## Related Work

- #ARCH-S5: Keep the SQLite session index current on normal turns (P2)
- #ARCH-S6: Repair interactive fork session identity and continuation (P2)
- #I010-S3: Slash commands (`/new`, `/resume`, `/fork`, `/status`)
- #I010-S7: Three execution modes (TUI/Headless/SDK via `AppServerSession`)

## Concrete Reproduction Case (2026-06-02)

### What happened

User started a new Talos session (TUI mode, default):

```
> 你好
你好！有什么我可以帮助你完成的编程任务吗？

作为 Talos，我可以协助你进行 Rust 代码开发、阅读文件、执行代码等操作。如果你有具体的需求，请告诉我。

> 你能记住上下文吗
是的，我能记住上下文。

在当前对话会话中，我可以：

1. **记住对话历史** - 我能看到之前所有的交流内容
2. **记住加载的文档** - 比如 AGENTS.md 文件的内容一直在我的上下文中
3. **记住之前的请求和响应** - 你之前问我 "1+1=?"，我能看到这个问题
```

### The bug

**User never asked "1+1=?" in this session.** The first message was "你好".

The "1+1=?" reference came from a **previous session** where we tested the Bailian integration with:
```bash
cargo run -p talos-cli -- -p "用中文回答:1+1=?"
```

Talos loaded content from that previous session into the current session's context.

### Root Cause Analysis (2026-06-02 investigation)

**Primary finding: TUI mode has NO session management.**

Code audit of `run_tui_mode` (crates/talos-cli/src/main.rs:492-550):
- No `SessionManager::new()` call
- No `create_session()` or `resume_session()`
- Each user message spawns a fresh `Agent` instance (line 533)
- `agent.run_streaming(input, event_tx)` only receives the current message

Code audit of `Agent::run_inner` (crates/talos-agent/src/lib.rs:346-375):
```rust
let full_message = if system_prompt.is_empty() {
    user_message
} else {
    format!("{system_prompt}\n\n{user_message}")
};

let mut messages = vec![Message::User {
    content: full_message,
}];
```

The agent creates a NEW messages vector with ONLY:
- System prompt (AGENTS.md)
- Current user message

**No conversation history is loaded.** Each turn is completely independent.

**Why the LLM mentioned "1+1=?":**

The LLM doesn't actually have access to previous sessions. When asked "你能记住上下文吗?" (can you remember context?), it:
1. Has no conversation history to reference
2. Wants to give a convincing "yes" answer
3. **Hallucinates** an example: "you asked me 1+1=?"
4. This example happens to match a previous session (coincidence or pattern matching)

This is a classic LLM hallucination - fabricating plausible but false information to fill knowledge gaps.

**Session file exists but is unused:**

File `~/.talos/sessions/talos/d5734d76-ba07-4181-9352-89da7afd5e01.jsonl` has 4 lines:
- Line 1: AGENTS.md (system prompt)
- Line 2: Assistant response to "你好"
- Line 3: AGENTS.md again
- Line 4: Assistant response mentioning "1+1=?"

This file was likely created by a previous interactive mode session, not TUI mode. TUI mode never reads from it.

### Impact

1. **No conversation continuity in TUI mode** - each turn is independent
2. **LLM hallucinations** - fabricates false memories to appear coherent
3. **User confusion** - sees references to conversations that never happened
4. **Broken trust** - users can't rely on the agent's "memory"

### Likely root cause

This points to **Hypothesis 1 (Session Resume Loads Wrong Messages)** or **Hypothesis 6 (SQLite Index Staleness)**:

- The session being loaded includes messages from a previous session
- OR the SQLite index points to the wrong session file
- OR context compaction is pulling from multiple sessions

### Immediate investigation needed

1. Check which session file was loaded when user started this TUI session
2. Verify the session ID in SQLite index matches the JSONL file path
3. Check if the JSONL file contains messages from multiple sessions
4. Audit the context building logic to see if it queries across session boundaries

## Next Steps

1. **Immediate**: ~~User provides a concrete reproduction case~~ ✅ Done (see above)
2. **Investigation**: ~~Run the 5-step investigation plan above~~ ✅ Done - root cause identified
3. **Triage**: ~~Determine which hypothesis is the root cause~~ ✅ Done - TUI mode has no session management
4. **Fix**: Implement Phase 1 (add session management to TUI mode)
5. **Verify**: Add regression tests and E2E validation

### Estimated effort

- Phase 1 (TUI session management): 2-3 days
- Phase 2 (fork fix): 1 day
- Phase 3 (tests): 1 day
- Phase 4 (logging): 0.5 day
- **Total: 4.5-5.5 days**

### Priority

**P0** - This blocks daily use. The LLM hallucination issue makes TUI mode unreliable for any multi-turn conversation.

### Recommendation

Start with Phase 1 immediately. This is the core fix that addresses the user's reported issue. Phases 2-4 can follow in subsequent iterations if needed.

## Notes

- This is P0 because it blocks daily use
- The problem might be a combination of multiple hypotheses
- Fix should be surgical: don't refactor the entire session system, just isolate the contamination path
- Consider adding a `--strict-session-isolation` flag that errors if any cross-session data is detected
