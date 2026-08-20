# Iteration I201: Tool-Call Placeholder Suppression

> Document status: Review / Claimed
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
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline planning session |
| Work Slice | I201/TUI-043 only: suppress an exact standalone tool-call compatibility marker at the TUI ordered-content boundary while preserving legitimate text, tool rows, ordering and persistence. No provider protocol, core Message, execution, permission, persistence, broad renderer or release changes. |
| Claimed At | 2026-08-19 |
| Source Issue | #111 |
| Governance Claim PR | #306 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | PR #306 merged to `main` as `78cb1ddd` from exact base `8069ea6a` after exact-head CI `32209314843`, independent Agent governance review `5336890794`, both validators and merge-time CAS. This effective claim authorizes only the bounded I201/TUI-043 implementation. |
| Implementation PR | #309 |
| Last Updated | 2026-08-19 |
| Handoff / Release Condition | Claim #306 is effective at merge `78cb1ddd`; create the I201 implementation branch only from that merge or later current `main`. Per-child CI, Agent technical review and CAS remain merge gates; eligible natural-person review moves to VALIDATION-002/I211/Issue #302 while I201 stays Review. |

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

I201 claim PR #306 merged to `main` as `78cb1ddd` from exact base `8069ea6a` after I197
implementation merge `d98f37e7`. The effective claim authorizes only the published bounded
TUI-visible placeholder suppression slice; implementation has not started.

## Verification Evidence

Implementation commits `68f4fb7b` and `d1fef291` add a TUI-local ordered-content gate and 14
focused state/event tests. `cargo test -p talos-tui --locked` passed 549 unit tests, 2 integration tests and 2 doctests;
focused tests, strict package Clippy, formatting, both governance validators, `git diff --check` and
the complete release preflight also passed. PR #309 still requires exact-head CI, independent Agent
technical review and merge-time CAS. Natural-person suppression-safety review remains deferred to
Issue #302 / I211.

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

## 2026-08-19 Claim Activation Checkpoint

PR #306 final head `153e470f` merged to `main` as `78cb1ddd` after exact-head CI `32209314843`
passed every routed job, both governance validators reported 0 warnings, independent Agent review
`5336890794` approved the exact head with shared-account identity limits disclosed, and merge-time
CAS passed. No Rust/Cargo or implementation change was included. I201 is now `Active / Claimed`;
its implementation branch may start only from `78cb1ddd` or later current `main` and remains limited
to the published Work Slice.

## 2026-08-19 Implementation Review Checkpoint

Implementation commits `68f4fb7b` and `d1fef291` are published through PR #309 from branch base
`25fe1f0c`. The change is limited to `crates/talos-tui/src/app.rs` and `app/output.rs`: it holds only
a possible standalone compatibility-marker line, suppresses it after a confirmed structured
`ToolCall`, and flushes it unchanged on ordinary text, terminal completion, error, unconfirmed start,
direct result or direct approval events. Independent technical review found and the second commit
closed the result/approval false-confirmation paths before merge.
Provider protocol, core Message, tool execution, permission, persistence, Cargo and release surfaces
are unchanged. I201 stays `Review` with `Completion Commit: Pending`; exact-head PR gates and Issue
#302 / I211 human validation remain open.

## 2026-08-19 Implementation Merge Checkpoint

PR #309 final head `d8d414ce3f2d65c6859fa4f30566efb3ac94196c` passed exact-head CI run `32220300200`
(5/5), independent Agent technical review `5338185591`, both governance validators and merge-time
CAS, then merged to `main` as `7f5a6df2122d9b5ed70e55e59281e3e4e127f18c`. This is implementation
evidence, not completion evidence: the natural-person suppression-safety row remains open in Issue
#302/I211, so I201 remains `Review / Claimed` with `Completion Commit: Pending`.

## 2026-08-20 I211 Human Validation Failure Disposition

Natural-person checkpoint `5341637918` passed the non-tool, legitimate-text, ordinary tool ordering,
deny, failure, cancel and resume observations, but failed the permission-mediated path. Approved
write/read sequences retained `Calling tools…`, then displayed an unnamed approved row, then the
named structured tool row.

I201 and TUI-043 remain Review with `Completion Commit: Pending`. Corrective Story TUI-058 / Issue
#329 separately owns approval-boundary marker correlation and named outcome rows. It is
Ready/Unclaimed and authorizes no product or permission-policy implementation.
