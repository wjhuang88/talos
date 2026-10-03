# Iteration I290: Auto Review Conversation Locale

> Document status: Active
> Published plan date: 2026-10-03
> MVP deliverable: Auto review effect summaries and human decision prompts follow locally detected conversation language, with deterministic configured UI-locale fallback.

## Scope And Non-Goals

Own AUTO-UX-001 / #590 only. Preserve Allow/Ask/Deny, redaction, tool input, deadlines,
request identity and execution authority. No dedicated model call or transcript resend,
no GPUI version change, no search, Desktop workflow, release or unrelated translation work.
Dependency: existing ADR-075 / I281 decision and review boundaries.

## Recovery And Selection Inventory

Snapshot: main `1b3e515a` on 2026-10-03. Open PRs inspected: #629 (SEARCH-001-A) and
#630 (this historical unclaimed draft). No other locale claimant found in the target owner.
This is recovery-time evidence, not a retrospective preflight for the September draft.

| Owner | State | Disposition |
|---|---|---|
| I164 | Paused | Retain superseded layout target; no reactivation |
| I249 | Planned / Unclaimed | Dependency pilot deferred; no unrelated upgrade |
| I277 | Review, remaining acceptance Deferred | Retain device/human checks in #29 |
| I288 | Active / Claimed, #627 already merged | Keep architecture-only search scope and #629 separate; stale proposed wording is unrelated drift |
| I289 | Complete / Closed | Preserve #628/#638 completion evidence |
| I290 | Active / Claimed (#639 pending merge) | No implementation authority until governance merge |

Current owner headers across all iteration documents were inspected; no additional
Active/Review/Planned/Blocked iteration was found. Separate DEPENDENCY-003-A Story
ownership remains unchanged. Desktop I282-I287 owners are terminal on this target.

## Design And Localization Responsibilities

- Use a Rust-native local language detector with documented language coverage and
  confidence/ambiguity handling. Script alone cannot choose language or region.
- Keep a bounded recent-user-turn evidence window, updated once per user submission;
  ignore model/tool/system messages, code blocks and ambiguous short/code-only input.
- Represent language as a validated BCP-47-style tag; infer no region. Preserve an
  explicitly configured region only for the same language. Uncertain/unsupported
  evidence falls back to the configured UI locale deterministically.
- Config owns validated UI-locale configuration and default; Agent owns incremental
  evidence and presentation hint; assessor owns translated effect_summary and
  decision_points; host surfaces own fixed approval labels and failure text.
- Document actual fixed-copy translation coverage and English fallback for missing
  translations. Follow-up translations do not weaken review gates.
- Keep locale outside authorization digests; do not add a required public struct field
  without compatibility design/migration. Preserve third-party assessor defaults.

## Acceptance

- Chinese, English and at least one other language reach the real assessor prompt and
  human fallback surface; no raw transcript is added solely for locale detection.
- Mixed, short, unsupported, code-only and failed detection have deterministic fixtures.
- Session isolation, resume/history seeding, language switching and once-per-turn
  observation are exercised; no duplicate evidence from multiple approvals.
- Config and public API compatibility are tested; redaction and permission regressions
  retain their existing meaning and pass.
- Measure warmed detection time and bounded prompt byte/token estimate growth; no
  unmeasured latency claim. User guide documents behavior and translation ownership.

## Validation And Documentation Targets

Repository-pinned Rust, --locked checks, focused config/agent/host tests, full preflight,
both governance validators, fresh exact-head CI, independent API/security review and
merge-time CAS. README.md and README.zh-CN.md document configuration, confidence/fallback
and surface translation limits. Delivery requires an existing implementation merge
SHA; the claim/status commit is not completion evidence.

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
| Implementation PR | #630 (historical draft; rebuild after effective claim) |
| Last Updated | 2026-10-03 |
| Handoff / Release Condition | Effective claim merge precedes rebuilt implementation; no release/version/permission-policy change. |
