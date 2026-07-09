# TUI-029: Thinking Content History Archive

| Field | Value |
|---|---|
| Story ID | TUI-029 |
| Priority | P2 |
| Status | Rejected — ADR-034 transient boundary preserved |
| Source | [GitHub Issue #26](https://github.com/wjhuang88/talos/issues/26) |
| Depends On | `MODEL-003`, `TUI-020`, ADR-034 |

## Problem

GitHub Issue #26 requests that model thinking/reasoning content be archived into the visible
history/scrollback after the model moves from thinking to answering or tool use.

That is not implemented today. Current behavior is the opposite by design:

- `TUI-020` keeps thinking visible only as a live preview and keeps finalized history clean.
- ADR-034 v3 persists structured reasoning only for provider replay/request-history correctness.
- Hidden or provider-native reasoning blocks are request-history metadata and must not be displayed
  by default.

The issue was incorrectly closed on 2026-07-08 with a comment claiming thinking content enters
history. That claim is false for the current codebase.

## Scope

- Decide whether Talos should expose any reasoning/thinking content in visible history.
- If approved, design a static scrollback format compatible with the inline terminal history model.
- Preserve provider replay correctness for signed/redacted reasoning blocks.
- Define persistence/export controls for any displayable reasoning archive.

## Non-Goals

- Do not expose hidden chain-of-thought by default.
- Do not render `ReasoningBlock::Redacted` payloads.
- Do not treat provider replay metadata as user-visible transcript content without an ADR revision.
- Do not change session storage defaults without migration and rollback planning.

## Acceptance

- [ ] ADR-034 is revised, or a new ADR is accepted, to allow a bounded visible-history policy.
- [ ] The policy distinguishes displayable reasoning text from hidden/signed/redacted provider
      payloads.
- [ ] The TUI scrollback format is static text, readable after the turn, and visually distinct from
      assistant answers.
- [ ] Resume and export behavior are specified and tested.
- [ ] Runtime evidence proves thinking history appears only when the approved policy permits it.

## Evidence: Current Non-Implementation

- `crates/talos-conversation/src/engine.rs` handles `AgentEvent::ThinkingDelta` by updating
  `current_thinking_text` and emitting `UiOutput::ThinkingPreview`.
- `TurnEnd`, `Error`, and cancellation clear `current_thinking_text` and emit
  `ThinkingPreview { text: None }`.
- `AgentEvent::ReasoningComplete` is ignored by the conversation display path.
- `TUI-020` explicitly requires thinking not to appear in finalized history or normal session
  history.

## Required Reads

- `docs/decisions/034-reasoning-thinking-boundary.md`
- `docs/backlog/active/TUI-020-thinking-preview-not-history.md`
- `docs/backlog/active/MODEL-003-reasoning-thinking-support.md`
- `crates/talos-conversation/src/engine.rs`
- `crates/talos-tui/src/app.rs`
- `crates/talos-session/src/jsonl.rs`

## Decision: Rejected (2026-07-09)

**ADR-034 v3 transient boundary is preserved.** Thinking/reasoning content will NOT be archived
into visible history for the current direction. The request in GitHub Issue #26 is formally
rejected with the following rationale:

1. **Context window pressure**: Adding reasoning text to visible history increases token usage
   on resume without clear user benefit. The current transient preview shows thinking during the
   turn; archiving it would bloat session files and context.

2. **Provider reasoning complexity**: ADR-034 persists structured `ReasoningBlock` data for
   provider replay correctness. Some blocks contain signed/redacted content that must not be
   displayed. Exposing any reasoning text risks leaking provider-internal data.

3. **Design consistency**: ADR-035 (TUI history scrollback boundary) establishes that terminal
   scrollback is the canonical renderer for finalized history. Adding thinking content to this
   scrollback would create visual noise and complicate the clean user/assistant/tool message
   structure.

4. **No new evidence**: ADR-034 was accepted 2026-07-03 after architecture review. No new
   technical evidence, user feedback data, or provider behavior changes have emerged that would
   justify revising the decision.

## Reversal Trigger

This decision can be revisited if:
1. A provider releases a model where reasoning text is explicitly user-facing (not chain-of-thought)
2. Users provide clear feedback that archived reasoning improves their workflow
3. Context window limits increase enough that reasoning archival is not a cost concern
4. A new ADR specifically addresses displayable vs. hidden reasoning with a clear boundary
