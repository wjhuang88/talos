# PROVIDER-004: Text Tool-Call ID Collision Hangs the Turn

| Field | Value |
|---|---|
| Story ID | PROVIDER-004 |
| Priority | P1 |
| Status | Ready (2026-07-24) |
| Type | Technical Story (bug fix) |
| Source | User transcript (tlog) 2026-07-24, provider `baosight` / model `glm-5.2-global` |
| Depends On | none |

## Problem

On providers without native function calling, tool calls arrive as fenced
```json-tool markdown blocks parsed from assistant text
(`talos-provider::anthropic_stream::parse_json_tool_call`,
`talos-provider::openai_sse` text path). The model frequently reuses a
placeholder id such as `"id":"call_0"` across many separate tool calls.

The text parser trusts a model-supplied id verbatim: it only synthesizes a
UUID when the id is missing/empty
(`parse_json_tool_call`, anthropic_stream.rs ~line 377:
`.filter(|s| !s.is_empty()).unwrap_or_else(|| Uuid::new_v4()...)`). A non-empty
`call_0` is therefore kept as-is. The native OpenAI-SSE path is asymmetric — it
synthesizes an index-stable id via `finalized_tool_call_id(id, i)`
(openai_sse.rs ~line 296) — but the text path has no such fallback
(openai_sse.rs ~line 313 sends the parsed call's id unchanged).

Because Talos emits one text tool call per provider response, the in-turn
duplicate-id guard in `run_inner` (lib.rs ~line 692, which only checks for
duplicates **within a single provider response**) never fires. The turn advances
normally, round after round, each appending
`Assistant{ tool_calls:[id=call_0] }` + `Tool{ tool_use_id=call_0 }` to history.

On the **next** provider request, the OpenAI request builder pairs tool results
to calls using a `pending_tool_call_ids` set (openai_request.rs ~line 197):
the first `call_0` result removes the id from the set; every subsequent `call_0`
result finds the id already gone and is **silently dropped as an orphan**. The
outgoing `messages` array then has broken tool_call/tool_result pairing. The
provider receives a malformed conversation and returns nothing usable, so the
model is never re-invoked and the **turn hangs silently**. The transcript ends
exactly after a successful `__OK__:call_0__` with no following record.

This is distinct from RUNTIME-002 (dispatch-timeout stuck processing, resolved)
and from TOOL-023 (tool-execution timeout). The root cause here is tool_call_id
correlation, not a timeout.

## Evidence

- tlog: 17 tool calls carry `id":"call_0"`, only 8 carry unique ids; the hang
  point is the last of two consecutive `call_0` results.
- Code path confirmed by direct read of `anthropic_stream.rs`, `openai_sse.rs`,
  `openai_request.rs`, and `lib.rs` run_inner.

## Goal / Value

A model that reuses tool-call ids can no longer wedge the turn. Every executed
tool call has a unique id so tool_call/tool_result pairing on the next provider
request is always well-formed.

## Scope

- Primary fix (chosen): make the text tool-call path assign a **unique** id
  regardless of the model-supplied value — either ignore the model id and always
  synthesize a UUID, or make uniqueness per-turn/per-history-guaranteed. Apply in
  both text parsers (`anthropic_stream::parse_json_tool_call` and the
  `openai_sse` text path) so behavior is symmetric with the native path's
  index-stable synthesis.
- The assistant message persisted for that turn must carry the SAME synthesized
  id that the tool result is keyed by, so intra-turn and cross-turn pairing both
  hold.

## Exclusions

- No change to native (structured `tool_calls`) provider handling beyond keeping
  it consistent.
- Not fixing the model's behavior; Talos must be robust to duplicate/placeholder
  ids regardless of what the model emits.
- Broadening the in-turn duplicate guard to cross-turn is NOT the chosen primary
  fix (it would only convert a hang into an error, not keep the turn working).

## Decision Links And Constraints

- Interacts with the tool-call protocol prompt (`ToolProtocol::Compat` /
  `TalosStrict`) — confirm the injected format does not require the model to
  control ids.
- Must not break the native OpenAI-SSE `finalized_tool_call_id` contract or the
  Anthropic native tool_use path.

## Uncertainty And Validation Path

Reproduce deterministically with a mock provider that emits multiple text
```json-tool blocks all using `"id":"call_0"` across consecutive turns; assert
the second turn's request has unique, correctly paired tool_call_ids and no
dropped orphan tool results, and the turn continues instead of hanging.

Optional defensive follow-up (record as residual if not done here): in
`openai_request.rs`, surface a dropped-orphan tool result as a warning/error
rather than silently discarding, so future correlation bugs are observable.

## State/Status Owners

This story file; `docs/BOARD.md` mirror; `docs/backlog/PRODUCT-BACKLOG.md` row.

## User-Facing Documentation

None (internal correctness fix). If tool-call reliability is documented anywhere
user-facing, note that duplicate model ids are tolerated.

## Required Reads

- `crates/talos-provider/src/anthropic_stream.rs` (`parse_json_tool_call`, `parse_text_tool_calls`)
- `crates/talos-provider/src/openai_sse.rs` (text path ~line 313; `finalized_tool_call_id` ~line 414)
- `crates/talos-provider/src/openai_request.rs` (~line 197 pending_tool_call_ids pairing / orphan drop)
- `crates/talos-agent/src/lib.rs` (run_inner in-turn duplicate guard ~line 692)

## Acceptance for behavior

- Given a text-tool-call provider whose model emits three consecutive tool calls
  each with `"id":"call_0"` over three turns
  When each tool executes successfully
  Then each executed call has a unique id, every tool result is paired (no orphan
  drop), and the model is re-invoked each round — the turn completes instead of
  hanging.

- Given a native structured-tool_calls provider
  When it emits tool calls
  Then behavior is unchanged (no regression to native id handling).

## Acceptance for technical work

- [ ] Repro test (mock provider, duplicate `call_0` across turns) fails/hangs
      against current code and passes after the fix.
- [ ] Text path assigns unique ids in both `anthropic_stream` and `openai_sse`;
      assistant message and tool result share the synthesized id.
- [ ] `cargo test --workspace --locked` + `cargo clippy --workspace --locked -- -D warnings` clean.
- [ ] Orphan-drop observability recorded as done or as an explicit residual.
- [ ] Board / backlog synchronized.
