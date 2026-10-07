# Iteration I291: Auto Review Locale Product Acceptance Follow-up

> Document status: Review
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
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 |
| Work Slice | AUTO-UX-001 / #590: product acceptance follow-up for session locale inference |
| Claimed At | 2026-10-03 |
| Source Issue | #590 |
| Governance Claim PR | #650 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | User explicitly requested execution of the product follow-up and authorized GitHub publication. Independent API/security review and exact-head CI remain required. |
| Implementation PR | #652 (merged; product acceptance remains open) |
| Last Updated | 2026-10-04 |
| Handoff / Release Condition | Effective claim merge precedes implementation; no release or permission-policy change. |

## Execution Evidence And Remaining Acceptance (2026-10-04)

PR #652 merged at `9582733cca444c3ccfe70faf9daff869757c7fcf`. Exact-head CI run
[2988](https://github.com/wjhuang88/talos/actions/runs/37180797013) passed all jobs at
`cce6ca4af4a6ede0e4cb51fc1b9b0562c38c34d5`. Independent Agent API/security review approved
that head after byte-bounded observation, public context-field compatibility and localized
missing-decision-point fixes. The review covered those risk questions, not the whole product matrix.

The merged code observes up to eight recent user messages, truncates each to 4096 characters,
and caps their combined UTF-8 bytes at 16 KiB before copying. It binds observations to the
permission session lease and carries locale through additive assessor hooks. This is merged
implementation evidence, not a product Completion Commit.

| Acceptance area | Current evidence / remaining work |
|---|---|
| Permission and API boundary | Exact-head CI and independent review passed; locale excluded from digest |
| Local detection | Chinese/Japanese and short/code fallback unit fixtures exist; script heuristics still cannot establish language confidence |
| Session history | Recent bounded history is observed; incremental/deduplicated tracking, resume and session rotation tests remain required |
| Fallback/config | Environment LC_ALL/LANG fallback exists; validated UI config integration and malformed/mixed/unsupported matrix remain required |
| Real assessor and surface | Locale-aware hooks and zh/ja fixed copy exist; multilingual captured-provider prompt and failure-path tests remain required |
| Cost and user documentation | Byte bounds exist; warmed latency, prompt overhead measurement and README/user-guide behavior documentation remain required |

Resume in this owner and existing Work Slice: implement and locally test detector/config/fallback
and session tracking together, then capture multilingual assessor/surface evidence and measure cost.
Submit one converged follow-up candidate with fresh exact-head CI and independent review.
Keep AUTO-UX-001, I290, I291 Review / Claimed and #590 open until all published acceptance rows pass.

## Stage A+B+C Candidate Evidence (2026-10-04)

The first acceptance candidate changes locale aggregation from last-message-wins to a bounded
majority selection over the observed session window. Detection now records an internal confidence
level: high/medium script-majority, medium English heuristic, and low for short, code-like,
ambiguous or unsupported input. Low-confidence observations retain the configured fallback.
Supported-script fixtures cover `zh`, `en`, `ja`, `ko`, `ru`, `ar` and `hi`; mixed/unsupported
inputs fall back deterministically. Session-scoped observation tests prove that a foreign session
cannot mutate the resolver and that a resumed bounded history selects the majority locale.

Local evidence for this candidate:

- `cargo fmt --all -- --check`
- `CARGO_BUILD_JOBS=2 cargo test -p talos-agent --lib` — 433 passed
- targeted locale and session-observation tests — passed

This is an in-progress acceptance candidate. UI locale wiring, real assessor prompt capture,
provider failure-path evidence, performance measurement and user documentation remain open.


## Merge Review Correction (2026-10-06)

PR #658 original head `0dcd8b8b` passed CI run 3004 on retry, but merge review found
that the approval entrypoint could overwrite the history-majority result with a single intent,
and valid detections equal to the configured locale were excluded from voting. The candidate
now preserves the observed-history result through repeated approvals, includes all non-low
confidence votes, and retains single-intent compatibility only before history observation.
Regression fixtures cover repeated snapshots, empty-snapshot no-op behavior and foreign-session isolation.
These snapshot fixtures do not establish incremental detection or durable resume acceptance.

The earlier Stage A+B+C label describes intended scope, not completed acceptance. UI locale
configuration integration, incremental history detection, durable resume/rotation integration,
real assessor prompt/failure evidence, measurements and user documentation remain open.
Fresh exact-head CI and independent review are required for the corrected candidate.

## Corrected Candidate Merge Evidence (2026-10-06)

PR #662 merged at `1a397cfa1cbbee4cdc435cb450980b0fa6a25011` from exact head
`2ab3e3b741733548c67b5eb664c1a4b4a2468426`. Exact-head CI run 3011 passed all jobs,
and independent API/security review approved the head. The merged slice preserves session
history through repeated approvals, includes valid fallback-locale votes, keeps empty snapshots
as no-ops, and leaves locale outside permission authority and request identity.

This merge closes the corrected implementation slice, not the product story. AUTO-UX-001,
I290, I291 and #590 remain Review / Claimed while UI locale wiring, real provider prompt and
failure-path evidence, performance/prompt-overhead measurements, and user documentation remain.


## Next Acceptance Slice: Configured Locale Wiring (2026-10-07)

The next candidate wires an optional validated `auto.locale` configuration into the CLI and TUI
resolver construction. Invalid values keep the existing environment/default fallback; the value is
presentation-only and does not enter request identity, permission authority, or assessor decision
semantics. This slice adds configuration and validation coverage. Provider prompt/failure capture,
incremental durable resume/rotation evidence, performance measurement, and user-facing behavior
documentation remain open.


## Configured Locale Fallback Correction (2026-10-07)

Independent review of PR #674 found that an explicit `auto.locale` value could be lost after
low-confidence history observations because the observer re-read `LC_ALL` / `LANG`. The
follow-up correction stores the validated configured locale as the session fallback and preserves
it through short, code-only, unsupported, and empty-history paths. Fresh exact-head CI and review
are required for the corrected candidate.


## Configured Locale Wiring Merge Evidence (2026-10-07)

PR #674 merged at `9cd764874074d77892ff92b22123b6dfe67f1440` from exact head
`8b89b16a7c4348f59a56f6ec49ffff5208d16493`. Exact-head CI run 3042 passed all jobs,
and independent API/security review approved the exact head. The merged slice wires optional
validated `auto.locale` through CLI and TUI resolver construction and preserves it as the
session fallback through low-confidence, short, code-only, unsupported, and empty-history
observations without changing permission authority, request identity, digest, or assessor
decision semantics.

This merge closes the configured-locale wiring slice only. Real-provider prompt/failure
capture, durable incremental resume/rotation evidence, performance/prompt-overhead measurement,
and final user-facing behavior documentation remain open; I291 and #590 stay Review / Claimed and open.
