# AUTO-UX-001: Conversation-Language Auto Review Explanations

**Status**: Review / Claimed (follow-up I291 required)
**Type**: Product Story
**Source Issue**: #590

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
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 |
| Work Slice | AUTO-UX-001 / #590: session-local conversation locale and Auto review presentation only |
| Claimed At | 2026-10-03 |
| Source Issue | #590 |
| Governance Claim PR | #639 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | User requested completion of #590, authorized publication to wjhuang88/talos and continued work. No independent maintainer is available in this session. Governance exact-head CI, validators and review are required; implementation requires independent API/security review. |
| Implementation PR | #640 and #652 (merged; I291 acceptance remains open) |
| Last Updated | 2026-10-04 |
| Handoff / Release Condition | Effective claim merge precedes rebuilt implementation; no release/version/permission-policy change. |


## Phase 1 Evidence

Completion Commit: `9bb565e09f5e0f3de8602183b4a670bf63f0aefc` (PR #640). Exact-head CI run 2961 passed.

## Follow-up Implementation Evidence

PR #652 merged at `9582733cca444c3ccfe70faf9daff869757c7fcf`; exact-head CI run 2988 and
independent API/security review passed. Product status remains Review / Claimed. See I291
Execution Evidence And Remaining Acceptance for the outstanding matrix and resume sequence.
