# Iteration I291: Auto Review Locale Product Acceptance Follow-up

> Document status: Planned
> Published plan date: 2026-10-03
> Parent: AUTO-UX-001 / #590 / I290
> MVP deliverable: session-level locale inference and acceptance evidence that satisfies the original product story without changing permission authority.

## Why This Follow-up Exists

I290 phase 1 is merged in PR #640 (`9bb565e0`) and its exact-head CI passed. Review found that
product acceptance still requires multi-turn session aggregation, broader extensible language
coverage, and a complete fallback/compatibility matrix. This iteration records the remaining
work; it does not invalidate the phase 1 implementation or rewrite its history.

## Scope And Non-Goals

- Observe bounded recent user-authored turns incrementally, including seeded history once per session;
  ignore model, tool, system, code-only and ambiguous short content.
- Detect language-only validated BCP-47 tags with confidence; never infer a region from script.
- Keep locale presentation-only: no changes to Allow/Ask/Deny, redaction, request identity, digest,
  eligibility, grant source, permission mode, deadlines or execution authority.
- No transcript resend, dedicated detection model call, unbounded storage, GPUI Git dependency change,
  search behavior, release/version change or unrelated translation work.

## Design Ownership

- Agent/session layer owns a bounded, deduplicated evidence tracker with session isolation.
- Detector owns documented language coverage and deterministic mixed/short/unsupported fallback.
- Config owns validated UI locale and the final fallback locale.
- Auto assessor receives only a bounded locale hint marked as presentation-only.
- Existing public APIs remain source-compatible through additive defaults or migration.

## Acceptance Matrix

- Chinese, English and at least one third language reach the real assessor prompt and human fallback.
- Multi-turn switching, history seeding, resume, duplicate approval events and session isolation pass.
- Mixed language, short text, code-only text, malformed locale, unsupported language and assessor
  failure all fall back deterministically to the configured UI locale.
- Locale changes do not change request digests or permission decisions.
- Captured prompt contains the locale hint without raw transcript content; byte/token growth is bounded.
- Warmed detection latency and full `--locked` CI evidence are recorded.
- Independent API/security review and governance validators pass before completion.

## Completion Gate

Mark AUTO-UX-001, I290 and I291 Complete only after an implementation merge commit exists,
exact-head CI passes, the acceptance matrix is evidenced, and the owner documents the final
Completion Commit. Until then the parent remains Review / Claimed.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 |
| Work Slice | AUTO-UX-001 / #590: product acceptance follow-up for session locale inference |
| Claimed At | 2026-10-03 |
| Source Issue | #590 |
| Governance Claim PR | Pending |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | User explicitly requested execution of the product follow-up and authorized GitHub publication. Independent API/security review and exact-head CI remain required. |
| Implementation PR | Pending |
| Last Updated | 2026-10-03 |
| Handoff / Release Condition | Effective claim merge precedes implementation; no release or permission-policy change. |
