# Iteration I201: Tool-Call Placeholder Suppression

> Document status: Planned / Unclaimed
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
| Work Slice | I201/TUI-043 only: suppress an exact standalone tool-call compatibility marker at the TUI ordered-content boundary while preserving legitimate text, tool rows, ordering and persistence. No provider protocol, core Message, execution, permission, persistence, broad renderer or release changes. |
| Claimed At | Not applicable |
| Source Issue | #111 |
| Governance Claim PR | Pending |
| Authorization Mode | Not applicable |
| Authorization Evidence | Proposed governance-only claim from `main@8069ea6a`; it becomes effective only after a claim PR reaches `main`. No implementation branch or product behavior authority exists in this slice. |
| Implementation PR | Not started |
| Last Updated | 2026-08-18 |
| Handoff / Release Condition | Claim becomes effective only after its governance PR merges to `main`; only then create the I201 implementation branch from that merge or later current `main`. Per-child CI, Agent technical review and CAS remain merge gates; eligible natural-person review moves to VALIDATION-002/I211/Issue #302 while I201 stays Review. |

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

I201 claim preparation is proposed from `main@8069ea6a` after I197 implementation merge
`d98f37e7`. The owner remains Unclaimed until its governance PR reaches `main`; no implementation
branch or product behavior authority exists in this slice.

## Verification Evidence

Pending implementation after an effective claim reaches `main`.

## Completion Evidence

Completion Commit: Pending. A status-only commit cannot serve as its own evidence.

## Variance And Residuals

Provider capability negotiation and general synthetic-status filtering require separate owners.

## Change Control - 2026-08-18 Deferred Human Validation Timing

The maintainer selected the long-task Deferred Human Validation Mode. I201's published objective,
negative fixtures and no-legitimate-text-loss acceptance remain unchanged. Exact-head CI,
independent Agent technical review, locked checks, governance validation and CAS remain local merge
gates. The natural-person exact-head compatibility review may move to VALIDATION-002/I211/Issue
#302 after implementation merge; I201 remains Review until that row passes, while I198 may proceed
under its separate claim.

## Retrospective

Pending execution.
