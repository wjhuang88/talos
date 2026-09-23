# DESKTOP-001-D6: Desktop Durable Tasks And Evidence

> Document status: Active — Claimed

| Field | Value |
|---|---|
| Story ID | DESKTOP-001-D6 |
| Parent Epic | DESKTOP-001 |
| Type | Desktop integration / behavior Story |
| Priority | P1 |
| Status | Active / Claimed — implementation under local correction |
| Selected Iteration | I284 |
| Source | #29; four-week Desktop task |
| Depends On | I283 implementation merged with technical gates; ADR-042/061; durable-session and WORK-001 projection compatibility map |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 |
| Work Slice | DESKTOP-001-D6 / I284: Desktop Durable Tasks And Evidence |
| Claimed At | 2026-09-23 |
| Source Issue | #29 |
| Governance Claim PR | #606 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | I283/D5 closeout merged as `81a5d27c`; I284 is the next serial child |
| Implementation PR | #607 open; current remote head `8e8ebf6`; local substantive corrections are not yet pushed |
| Last Updated | 2026-09-23 |
| Handoff / Release Condition | Finish local convergence, refresh exact-head CI and independent review, then merge-time CAS; no release |

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
Implementation is in progress on PR #607; completion and human acceptance evidence are pending.

## Local Review Correction Checkpoint — 2026-09-23

The prior exact head `8e8ebf6dde23af4286badd1b66d1dfcf41d62c24` received two blocking findings:
sanitized task IDs could collide/leak prompt text, and requested tool paths were labeled as actual
evidence before execution. The local candidate now uses domain-separated SHA-256 task/workspace
identities, preserves legacy bindings in Recent Tasks without displaying their embedded path/goal,
and reports a tool's requested path separately from actual change attribution. Resume opens only an
existing binding, stays existing-only on retry, restores transcript without submitting a turn, and
fails visibly when the binding is missing. Workspace task-list storage reads run on GPUI's
background executor.

Local validation on the unpushed candidate: `cargo check -p talos-desktop --locked`, the exact CI
Clippy command (`cargo clippy -p talos-desktop --features desktop-ui --all-targets --locked -- -D
warnings`), and `cargo test -p talos-desktop --features desktop-ui --locked` (83/83) passed.
Formatting, diff checks, and both governance validators passed with 0 warnings. PR #607's old head
`8e8ebf6` failed that Clippy gate on the prior Resume path; the local candidate replaces that path.
These local results do not validate the old remote head. Full `./scripts/release_preflight.sh`
passed locally. Candidate `889c1cf22257254fe51316432f44e433ba0d4803` is PR #607's exact head
against base `4e150b3e1bc6910b3a02d75b249573ecff8191e7`. CI `35856041872` passed the main
Format/Check/Clippy/Test job, Linux Desktop explicit-feature job, classifier, remote Issue
reconciliation and Windows installer fixture. The Windows Rust workspace job has passed its test
and smoke steps but has not reached a terminal job state while cache cleanup runs; do not count it
as green until GitHub reports a conclusion. Independent review of this exact head is pending.

I284 remains incomplete: no shared durable Work/Evaluation projection is available to the Desktop
host, and Runtime events do not yet carry actual changed-artifact evidence sufficient for a real
file diff. The live UI must report attribution as unavailable rather than infer it from workspace
dirt or request arguments. H4/H6 also remain unaccepted in #29.
