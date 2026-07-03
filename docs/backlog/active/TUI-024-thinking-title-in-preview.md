# TUI-024: Thinking Title In Preview Area

Type: Product Story
Parent Epic: UX-001 follow-up; candidate for I086 "thinking preview policy" scope
Status: Planned — candidate for I086

## Identity / Goal / Value

Today the preview area renders `thinking: {entire accumulated delta text}`
(`preview_text_for_state`, `crates/talos-tui/src/app.rs:1052-1056`), so long reasoning streams
scroll unreadably through a one-line preview. OpenCode instead shows a section TITLE.

Verified mechanism (sst/opencode @ a4fed69, 2026-07-03): the title is **parsed from the
reasoning text itself**, not from a dedicated provider field. `packages/tui/src/context/thinking.ts`
`reasoningSummary()` matches a leading standalone bold block —
`^\*\*([^*\n]+)\*\*(?:\r?\n\r?\n|$)` on trimmed text — and falls back to a generic
"Thinking"/"Thought" label when absent. OpenAI Responses reasoning summaries conventionally
begin with such `**Title**` lines (OpenCode requests `reasoning: { summary }`); Anthropic and
`reasoning_content` gateways get the same parse applied with fallback.

Goal: when the accumulated thinking text carries a parseable section title, the preview shows
`thinking: {title}` instead of the raw stream; otherwise behavior is unchanged.

## Honest expectations per provider (from ADR-034 ground truth)

- OpenAI official Chat Completions: never streams reasoning — no title, no change.
- OpenAI-compatible gateways (`reasoning_content`: DeepSeek/GLM/Qwen): raw CoT is usually
  unstructured — fallback path is the common case.
- Anthropic thinking: sometimes structured with bold paragraph headings — titles appear when
  present.
- A future OpenAI Responses adapter (separate ADR per ADR-034) is where titles become reliable.

## Scope

- Title extraction from the accumulated transient thinking text (display-side only), matching
  the OpenCode regex semantics; as text accumulates, the most recent section title wins.
- Preview shows `thinking: {title}` when a title exists; existing full-text behavior (or the
  animated `thinking` label alone) when none does — exact fallback decided in implementation.
- Keep the animated gradient on the `thinking` label
  (`crates/talos-tui/src/scrollback.rs::preview_line_spans`).

## Exclusions / Boundary (ADR-034)

- Display-transient only: titles are derived from `ThinkingDelta` accumulation and are never
  persisted, exported, or written to scrollback (ADR-034 Decisions #8/#10).
- No collapsible/scrollable thinking panel (ADR-034 rejected for that slice; richer UX remains
  its own design decision).
- No request-side changes (no new provider fields).

## Dependencies

- None hard. Aligns with I086 planned scope "thinking preview policy".

## Required Reads

- `docs/decisions/034-reasoning-thinking-boundary.md`
- `crates/talos-tui/src/app.rs` (`preview_text_for_state`)
- `crates/talos-tui/src/scrollback.rs` (`PreviewComponent`, `preview_line_spans`)
- `crates/talos-conversation/src/engine.rs` (ThinkingDelta → ThinkingPreview)
- OpenCode reference: `packages/tui/src/context/thinking.ts` (sst/opencode)

## Acceptance for behavior

- Given thinking text beginning with a standalone `**Section Title**` block
  When the preview renders during processing
  Then it shows `thinking: Section Title` (not the raw stream), updating if a later section
  title appears.
- Given thinking text without any leading bold block
  When the preview renders
  Then behavior is unchanged from today.
- Given `**Important:** inline bold that is not a standalone title line`
  When parsed
  Then no title is extracted (parity with the OpenCode test suite).
- Given `/export` after a turn with titles shown
  Then no thinking text or title appears in the export (ADR-034 boundary regression test).
