# AUTO-UX-001: Conversation-Language Auto Review Explanations

**Status**: Refinement / Unclaimed
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

Intake only, not an I281 implementation requirement. No implementation claim or
activation. Detection method, supported-language coverage and nonlocalized UI
fallback remain refinement decisions owned here and in Issue #590.
