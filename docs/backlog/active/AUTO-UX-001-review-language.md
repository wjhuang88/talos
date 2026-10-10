# AUTO-UX-001: Conversation-Language Auto Review Explanations

**Status**: Complete / Closed
**Type**: Product Story
**Source Issue**: #590

## Current Product Closure — 2026-10-11

Completion Commit: `a3a4753d6e42123e33ead1d7e6f769a96f85f37e` (implementation PR #682).

All original product acceptance is complete after exact-head
[CI3109](https://github.com/wjhuang88/talos/actions/runs/38032860943), independent Agent/API/security
review and CAS. The [I291 final matrix](../../iterations/I291-auto-review-locale-product-followup.md#final-product-acceptance-and-closure)
records real multilingual surfaces, session behavior, deterministic fallback, authority/API checks
and bounded cost measurements. Earlier scheduling and phase checkpoints below remain historical,
not current pending work. The phase1 Completion Commit is not the final product completion evidence.

## Goal And Scope

Present Auto review effects, uncertainty and human decision points in the user's
conversation language, without limiting support to Chinese and English. Use an
extensible language identifier; do not infer a regional locale from script alone.
Language selection affects presentation only, never permission authority.

## Required Reads

- [ADR-075](../../decisions/075-bounded-model-decision-invocation.md)
- [I281](../../iterations/I281-auto-shell-review-and-windows-timing.md)

## Refinement And Acceptance

- Evaluate local language detection against mixed-language, short, code-only and
  ambiguous input. Unicode script alone cannot distinguish all languages.
- Prefer user-authored language evidence over tool output or reasoning. Define
  session boundaries, language switching and deterministic fallback before coding.
- Preserve non-Chinese/English language support and document actual detector coverage.
- Do not add a dedicated model call or resend full conversation history for detection.
- Pass a bounded language hint to the assessor; keep structured result values,
  commands, paths, redaction and Allow/Ask/Deny semantics unchanged.
- Test multilingual explanations and fallback, and measure latency/token overhead;
  earlier conversational microsecond/token estimates are not benchmark evidence.
- Document behavior in the Auto permission user guide when implemented.

## Scheduling And Residuals

Selected for I290, proposed activation only. PR #630 is a historical draft created before an
effective claim; it must be rebuilt from the claim merge or later and receive fresh checks.
No retrospective authorization or completion is claimed.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 |
| Work Slice | AUTO-UX-001 / #590: session-local conversation locale and Auto review presentation only |
| Claimed At | 2026-10-03 |
| Source Issue | #590 |
| Governance Claim PR | #639 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | User requested completion of #590, authorized publication to wjhuang88/talos and continued work. No independent maintainer is available in this session. Governance exact-head CI, validators and review are required; implementation requires independent API/security review. |
| Implementation PR | #640, #652, #662, #674, #677, #679, #680, #682 (merged; product acceptance complete) |
| Last Updated | 2026-10-11 |
| Handoff / Release Condition | Effective claim merge precedes rebuilt implementation; no release/version/permission-policy change. |


## Phase 1 Evidence

Completion Commit: `9bb565e09f5e0f3de8602183b4a670bf63f0aefc` (PR #640). Exact-head CI run 2961 passed.

## Follow-up Implementation Evidence

PR #652 merged at `9582733cca444c3ccfe70faf9daff869757c7fcf`; exact-head CI run 2988 and
independent API/security review passed. Corrected I291 implementation PR #662 merged at
`1a397cfa1cbbee4cdc435cb450980b0fa6a25011`; product status remains Review / Claimed. See I291
Execution Evidence And Remaining Acceptance for the outstanding matrix and resume sequence.
