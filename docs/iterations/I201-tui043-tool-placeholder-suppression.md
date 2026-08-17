# Iteration I201: Tool-Call Placeholder Suppression

> Document status: Planned
> Published plan date: 2026-08-14
> Planned objective: prevent an exact standalone `Calling tools…` compatibility marker from
> becoming visible history only when the same assistant response proceeds to structured tool calls.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: a maintainer can replay Unicode-ellipsis, three-dot, split-chunk and legitimate
> text fixtures and observe only synthetic tool-turn markers suppressed while every structured tool
> row and non-tool phrase remains intact.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #111 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-14 |
| Handoff / Release Condition | After I197 disposition, establish an effective I201 claim on `main` and branch only from that claim merge or later current `main`. |

## Published Baseline

### Selected Story

| Story | Status At Selection | Depends On | Outcome |
|---|---|---|---|
| TUI-043 / Issue #111 | Ready | Existing OpenAI request marker; canonical TUI ordered-content lifecycle | One runnable conditional presentation filter with ordering and negative evidence |

### Scope

- Hold only a normalized exact standalone Unicode-ellipsis or three-dot marker at the TUI-visible
  ordered-content boundary.
- Discard the held marker when the same response starts a real structured tool call; otherwise flush
  it unchanged on normal text, terminal completion or error.
- Preserve chunk joining, legitimate larger sentences, every structured tool/result row, ordering,
  approval, cancellation, failure, retry, timeout and persistence boundaries.
- Update owner/Issue evidence describing the narrowly corrected visible behavior.

### Non-Goals

- No OpenAI request serialization, core Message, SSE architecture, tool execution, provider
  capability negotiation, global phrase filter or broad session/export-format change.
- No I199/I200/I197 layout or anchor correction.

### Acceptance And Planned Validation

- All Issue #111 exact/split/whitespace/multi-tool/non-tool/embedded-text ordering fixtures pass with
  no leaked marker, blank replacement row or lost structured call.
- Focused stream lifecycle tests and relevant `cargo test -p talos-tui --locked` targets pass.
- Existing OpenAI request serialization tests, `cargo test --workspace --locked`, release preflight,
  both governance validators and `git diff --check` pass.
- Independent natural-person exact-head review confirms the filter cannot suppress legitimate
  non-tool text; shared-account identity and role separation are disclosed.

### Documentation Target

- TUI-043, I201 and Issue #111 evidence. No README feature claim is planned because the slice removes
  leaked synthetic presentation text without changing the supported tool protocol.

### Risks And Fallback

- A global or early provider filter could hide legitimate assistant text or alter model context.
- A pending marker not flushed on every terminal path could lose visible content.
- Fallback: preserve current text and leave I201 Review/Partial; never suppress without a confirmed
  same-response structured tool transition.

## Actual Activation And Execution

No activation has occurred. I201 remains Unclaimed and follows the layout/anchor corrections in the
ordered task to minimize overlapping TUI exact-head review churn; it is not technically coupled to
their outcome.

## Verification Evidence

Pending implementation after an effective claim reaches `main`.

## Completion Evidence

Completion Commit: Pending. A status-only commit cannot serve as its own evidence.

## Variance And Residuals

Provider capability negotiation and general synthetic-status filtering require separate owners.

## Retrospective

Pending execution.
