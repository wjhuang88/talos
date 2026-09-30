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

## Draft Candidate And Remaining Gates

A draft candidate stores one locale per resolver and samples at most 4096 characters of
current user intent during approval. It adds a presentation hint to the contextual assessor
payload and Chinese/Japanese wrapper templates. Locale is excluded from the authorization
digest. No extra model call or conversation-history resend is introduced.

This draft does not satisfy acceptance and must not be merged as complete:

- The detector uses script heuristics and a small English vocabulary. Cyrillic, Arabic,
  Devanagari and Han scripts cannot reliably identify language or region. Fixed regional
  mappings need replacement; mixed-language/code-only filtering is not implemented.
- It observes approval-time current intent, not incremental user turns across the session.
  Unsupported/uncertain input retains the previous hint rather than the configured UI locale.
- `LC_ALL`/`LANG` fallback is a prototype; there is no configured UI-locale integration.
- Adding a required field to the public assessment-context struct needs API compatibility
  handling. Non-shell assessors do not receive the locale hint.
- Failure text, incomplete-explanation text and approval UI labels remain English.
- Locked compilation, formatting, security regressions, latency/token measurements,
  user-guide documentation, and effective claim/iteration activation remain pending.

The previous local candidate incorrectly marked this owner Claimed without an effective
target-branch claim. The header is restored to Refinement / Unclaimed; a draft PR does not
establish ownership or completion evidence.

## Scheduling And Residuals

Intake only, not an I281 implementation requirement. No implementation claim or
activation. Detection method, supported-language coverage and nonlocalized UI
fallback remain refinement decisions owned here and in Issue #590.
