# DESKTOP-001-D6: Desktop Durable Tasks And Evidence

> Document status: Planned

| Field | Value |
|---|---|
| Story ID | DESKTOP-001-D6 |
| Parent Epic | DESKTOP-001 |
| Type | Desktop integration / behavior Story |
| Priority | P1 |
| Status | Planned / Unclaimed |
| Selected Iteration | I284 |
| Source | #29; four-week Desktop task |
| Depends On | I283 implementation merged with technical gates; ADR-042/061; durable-session and WORK-001 projection compatibility map |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | DESKTOP-001-D6 / I284: Desktop Durable Tasks And Evidence; planned scope only |
| Claimed At | Not applicable |
| Source Issue | #29 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Maintainer requested four-week Desktop planning on 2026-09-22; no implementation activation yet |
| Implementation PR | Not started |
| Last Updated | 2026-09-22 |
| Handoff / Release Condition | Resolve readiness and establish effective serial child claim before implementation; no release |

## Outcome

A Desktop user can reopen a saved conversation and inspect real work status, changes and revision-bound evaluation evidence.

## Scope

- Use existing durable Session identity/storage for recent task navigation and transcript resume; no Desktop business database or automatic tool replay.
- Show authoritative Work/Goal projection when available. Missing projection remains unavailable, never replaced by fixture work.
- Present read-only actual file/artifact change evidence with source/task identity; do not infer a complete task diff from arbitrary workspace dirt.
- Integrate existing completion-claim/evaluator/Mission gate interfaces where supported, with explicit evaluation action rather than autonomous resubmission. No executor self-certification.
- Show evaluation missing/fail/inconclusive/stale/current states and Delivery eligibility from the shared gate. Persist only through supported contracts; never resurrect an old PASS after restart.

## Acceptance

- Given two sessions, reopening one restores only its own durable transcript and workspace association; switching tasks cannot mix approvals or tool results.
- Given an interrupted session, restart/resume does not repeat writes or invent missing output; storage errors become a safe visible failure.
- Given execution artifacts or changes, show their actual provenance, or explicitly mark unavailable attribution; inspection itself performs no mutation.
- Given missing/stale evaluation or a changed Goal revision, the UI cannot display current PASS or eligible Delivery; final model text does not certify success.
- A live Desktop-to-shared evaluation fixture covers pass/fail/staleness; absent durable evaluation storage displays unavailable after restart rather than claiming persistence.

## Validation And Risks

Restart/resume/session-isolation fixtures, storage failure and no-replay tests, shared work/evaluation projection and revision-staleness matrix, read-only evidence boundary review; native H4 and integrated H6.

Shared P4 is not proof of durable evaluation storage. Resolve facade access before implementation; any required new public boundary/schema needs decision review. An unavailable-state fallback is honest but does not close unmet baseline acceptance.

## Required Reads

- [Four-week task](../../tasks/2026-09-22-desktop-four-week-delivery.md)
- [Desktop parent](DESKTOP-001-desktop-product-direction.md)
- [Renderer boundary](../../decisions/059-desktop-renderer-host-motion-boundary.md)
- [Visual baseline](../../design/talos-desktop/DESIGN.md)
- [Localization](../../design/talos-desktop/I18N.md)
- [Shared work foundation](WORK-001-goal-oriented-work-evaluation-foundation.md)
- [Runtime SDK migration](../../reference/I280-RUNTIME-FACADE-MIGRATION.md)

## Documentation And Residuals

Update the Desktop crate README (created in I282), bilingual user-facing instructions as behavior
lands, and this owner before derived views. Preserve #29 and the four-week task acceptance ledger.
New durable Mission/Evaluation schema, automatic evaluation on every turn, multi-window/reconnect, full artifact editor/merge UI, #308 presets or complete autonomous Mission planning.

## Completion Evidence

Completion Commit: pending.
No implementation or acceptance evidence exists for this planned child.
