# TUI-029: Thinking Content History Archive

| Field | Value |
|---|---|
| Story ID | TUI-029 |
| Priority | P2 |
| Status | Ready for Implementation — ADR-034 v4 accepted |
| Source | [GitHub Issue #26](https://github.com/wjhuang88/talos/issues/26) |
| Depends On | `MODEL-003`, `TUI-020`, ADR-034 |

## Problem

GitHub Issue #26 requests that model thinking/reasoning content be archived into the visible
history/scrollback after the model moves from thinking to answering or tool use.

That is not implemented today. The pre-v4 behavior is the opposite by design:

- `TUI-020` keeps thinking visible only as a live preview and keeps finalized history clean.
- ADR-034 v3 persisted structured reasoning only for provider replay/request-history correctness.
- ADR-034 v4 now permits a bounded visible-history projection of displayable text while retaining
  the opaque provider metadata boundary.

The issue was incorrectly closed on 2026-07-08 with a comment claiming thinking content enters
history. That claim is false for the current codebase.

## Approved Scope

- Archive text already exposed through `AgentEvent::ThinkingDelta` when the provider response moves
  to answer text or tool use.
- Represent archived thinking as a distinct typed reasoning history entry, not Assistant/System
  content and not a string-prefix convention.
- Render one static scrollback block with a `Thinking` label and indented `| ` body lines before the
  associated answer/tool entry.
- Rehydrate the same displayable archive from existing `AssistantReasoning` metadata on resume.
- Keep default `/copy` and `/export` output free of reasoning; add
  `/export <path> --include-thinking` for explicit filtered-text export.
- Preserve provider replay correctness and byte identity for signatures/redacted blocks.

## Non-Goals

- Do not expose hidden chain-of-thought by default.
- Do not render `ReasoningBlock::Redacted` payloads.
- Do not render or export `ReasoningBlock::Thinking.signature`.
- Do not copy reasoning into `Message::Assistant.content` or create duplicate session storage.
- Do not change session storage defaults without migration and rollback planning.
- Do not add a collapsible widget, managed history viewport, or alternate-screen history.
- Do not archive unfinished reasoning after cancellation/provider failure in the first slice.

## Acceptance

- [x] ADR-034 v4 allows a bounded visible-history policy.
- [x] The policy distinguishes displayable reasoning text from hidden/signed/redacted provider
      payloads.
- [x] The implementation contract specifies a static, visually distinct scrollback format.
- [ ] Thinking -> answer archives exactly one reasoning block before the assistant answer.
- [ ] Thinking -> tool use archives exactly one reasoning block before the tool entry, including
      repeated provider rounds inside one Talos turn.
- [ ] The TUI scrollback block is static text, readable after the turn, and visually distinct from
      assistant answers.
- [ ] Resume reconstructs only displayable `Thinking.text` / `Plain.text` and never renders
      signatures or `Redacted.data`.
- [ ] Default `/copy` and `/export` exclude reasoning; explicit `--include-thinking` export contains
      filtered text with a `Thinking` heading.
- [ ] Error/cancellation clears unfinished thinking without creating a misleading archive entry.
- [ ] Runtime evidence proves thinking history appears only when the approved policy permits it.
- [ ] Provider replay tests remain byte-identical and workspace validation passes.

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

## Superseded Decision: Rejected (2026-07-09)

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

## Change-Control Decision: Approved (2026-07-10)

The maintainer explicitly requested implementation after reviewing the rejection. This is new user
evidence and satisfies the rejection's own reversal triggers 2 and 4. Per `CHANGE-CONTROL.md`, the
request is a scope addition with a materially different acceptance target, so it must use a new
implementation iteration rather than reopening or rewriting completed TUI-020/I078 history.

ADR-034 v4 is accepted with this boundary:

1. The archive is a display projection, not provider conversation content.
2. Live display uses the existing `ThinkingDelta` stream. Resume display uses a centralized helper
   that reads only `ReasoningBlock::Thinking.text` and `ReasoningBlock::Plain.text`.
3. Signatures and redacted payloads remain opaque and non-displayable.
4. Existing durable reasoning metadata is reused; no session schema/default migration is needed.
5. Static terminal scrollback remains canonical under ADR-035.
6. Existing export/copy behavior remains safe by default; inclusion requires the explicit
   `--include-thinking` flag.

The 2026-07-09 rejection remains above as historical evidence but no longer governs future
implementation.

## Implementation Slices

| Slice | Crates | Deliverable | Validation |
|---|---|---|---|
| TUI029-A | `talos-conversation` | Typed reasoning role/source, transition finalization, transcript exclusion | focused engine tests |
| TUI029-B | `talos-tui`, `talos-core` | centralized display projection, static scrollback rendering, resume hydration | TUI + security sentinel tests |
| TUI029-C | `talos-conversation`, `talos-cli` | `--include-thinking` export parsing and filtered output | export/copy tests |
| TUI029-D | workspace | real TUI fixture for preview -> archive -> answer/tool and resume; docs sync | workspace/clippy/governance/runtime evidence |

## Public API And Migration

`MessageRole` and `MessageSource` are public conversation-layer enums and are not currently marked
`#[non_exhaustive]`. Adding `Reasoning` is therefore a source-breaking change for downstream
exhaustive matches. The implementation iteration must:

- record the change in release notes for the next pre-1.0 minor release;
- update every in-workspace exhaustive match;
- tell embedders to add `Reasoning` handling (or a wildcard where appropriate);
- avoid adding a new `talos-core::Message` variant, which is unnecessary for this feature.

## Test And Runtime Evidence

- Unit tests must prove no duplicate archive across `TextDelta`, `ToolCallStarted`, `ToolCall`, and
  `TurnEnd` boundaries.
- Security tests must use sentinel signature/redacted values and assert they never appear in
  scrollback, copy, or export output.
- Resume tests must cover both current compact-text sessions and legacy JSONL through the shared
  `SessionStore::read_messages` path.
- Runtime evidence must use the real TUI event path and capture static history ordering for answer
  and tool-use cases; unit tests alone are insufficient.

## Implementation Activation Gate

TUI-029 is ready to select into a new iteration. Activation must inventory all current
Active/Review/Planned/Blocked iterations, name the user-facing TUI deliverable, and preserve the
completed TUI-020 baseline. No production code should be merged until the implementation iteration
records ADR-034 v4 and ADR-035 as required reads.

## Historical Reversal Trigger

This decision can be revisited if:
1. A provider releases a model where reasoning text is explicitly user-facing (not chain-of-thought)
2. Users provide clear feedback that archived reasoning improves their workflow
3. Context window limits increase enough that reasoning archival is not a cost concern
4. A new ADR specifically addresses displayable vs. hidden reasoning with a clear boundary
