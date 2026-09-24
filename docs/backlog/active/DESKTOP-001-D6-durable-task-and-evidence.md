# DESKTOP-001-D6: Desktop Durable Tasks And Evidence

> Document status: Review — Claimed

| Field | Value |
|---|---|
| Story ID | DESKTOP-001-D6 |
| Parent Epic | DESKTOP-001 |
| Type | Desktop integration / behavior Story |
| Priority | P1 |
| Status | Review / Claimed — implementation merged; human acceptance residuals remain |
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
| Implementation PR | #607 merged at exact head `1fc73c2e`; merge commit `2845ebdf` |
| Last Updated | 2026-09-24 |
| Handoff / Release Condition | Technical implementation merged; close only after H4/H6 and authoritative evaluation-source acceptance; no release |

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

Completion Commit: `2845ebdf57ff26c62539a8516499ed7d29caa228` (PR #607 implementation merge).
Implementation is merged, but the story remains Review because H4/H6 and the production
evaluation-source acceptance are pending.

## Merge Checkpoint — 2026-09-24

PR #607 exact head `1fc73c2ea03356d4711625764e060f3eb3b8c32d` merged as `2845ebdf`. Exact-head
CI run `35959834215` was fully successful across all six jobs. Independent incremental review
approved the exact head and confirmed the final diff was limited to the runtime SDK fixture lock
file. No production, permission, security or public API scope was added by the correction.

Carry H4 and H6 to Issue #29. The production path remains fail-closed when no authoritative
criteria/evidence/revision producer exists; the deterministic EvaluationHarness is test-only and
cannot close the live acceptance row. I285 owns final integrated stabilization and acceptance.

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

At this review checkpoint, the shared Work projection and execution-bound artifact event had not
yet been added. The later local checkpoint below supersedes this implementation-state note; H4/H6
remain unaccepted in #29.

## Local Task Isolation Correction — 2026-09-23

Commit `61fe13ae` gives every explicit New Task a fresh workspace-scoped UUID identity and
serializes New Task/Resume behind the old idle host's `Stopped` event. Switching is rejected while
a turn or approval is active; host-generation checks prevent stale observers from mutating the
replacement task. Focused coverage now exercises stop-before-switch, close superseding a pending
switch, active/approval rejection, and stale observer generations. The locked Desktop UI suite
passed 87/87 and the exact Desktop Clippy command passed with `-D warnings`.

This correction is local and not included in PR #607 yet. Remote head remains `6714fd79` against
base `4e150b3e`; the CI and review evidence on that head does not cover `61fe13ae`. Full
`./scripts/release_preflight.sh` reached the independent Runtime SDK fixture after workspace
validation, tests and doctests, but the second external fixture build failed with `No space left on
device`. The failed run is not a green preflight; its task-owned `target/` output was removed with
`cargo clean`, freeing 19.1 GiB. Re-run the complete preflight on a disk-safe candidate before
publishing it.

At this task-isolation checkpoint, the shared Work projection and execution-bound artifact event
were not yet implemented. The later local checkpoint below records the subsequent changes. H4/H6
human acceptance remains open in #29; do not mark I284 Complete until its technical scope and
required gates are genuinely resolved.

## Local Work And Artifact Evidence Checkpoint — 2026-09-23

The uncommitted local slice reads the shared Work graph read-only, scoped to the bound session ID.
A Runtime hook snapshots only workspace-contained regular files no larger than 1 MiB before and
after successful built-in `write`, `edit` and `delete` calls. It emits session/turn/call/path
evidence only when the bounded content digest differs. It rejects symlinks and outside paths,
checks the same tool/path identity at both boundaries, caps pending snapshots at 128 and removes
unfinished entries at turn completion. The UI describes these as differences observed around
successful file tools; it shows source identity without exposing file contents.

Local verification: `cargo test -p talos-desktop --features desktop-ui --locked` passed 93/93;
`cargo clippy -p talos-desktop --features desktop-ui --all-targets --locked -- -D warnings`,
`cargo fmt --all -- --check` and `git diff --check` passed. Focused regressions cover creation,
no-op, failed calls, path mismatch, outside paths, size limit, symlink rejection and turn cleanup.
A RuntimeHost integration test starts a DurableSession, obtains real approval, executes the shared
`write` tool and asserts the actual file plus its session/turn/call/path event. The shared session
Work projection has a separate passing `cargo test -p talos-session --locked` result.

This evidence list is in-memory for the open host, does not cover shell/custom tools and does not
show a content diff. Earlier-run file evidence remains unavailable after restart. The accepted
scope still has no shared durable Mission/Evaluation source; Evaluation and Delivery therefore
remain unavailable. I284 is incomplete; H4/H6 are open in #29, and I285 remains Planned.

## Current Source Audit - 2026-09-24

The [I284 source audit](../../iterations/I284-durable-task-and-evidence.md#evidence-boundary-and-evaluation-source-audit---2026-09-24)
records artifact-boundary corrections, independent slice review and 98 passing Desktop tests.
The remaining evaluation gap is a production claim/current-subject/evidence handoff, not a request
for new durable Mission/Evaluation storage (excluded by the baseline). Existing shared evaluator
types alone do not supply that data. ADR-083 was accepted on 2026-09-24 and the shared
`RuntimeEvaluationService` is now available, but a real producer and explicit action are still
required before the story can close. Do not substitute fabricated live evidence or remove the
acceptance target.
