# Iteration I285: Desktop Integrated Candidate And Acceptance

> Document status: Complete / Closed (four-week integrated development candidate)
> Plan date: 2026-09-22
> Target window: 2026-10-13 to 2026-10-19
> Planned objective: A reproducibly built Desktop candidate passes an integrated real-task walkthrough with documented limits and a clean handoff.
> Baseline rule: once committed, preserve scope and append execution facts; changed targets use a new ID.
> MVP deliverable: A reproducibly built Desktop candidate passes an integrated real-task walkthrough with documented limits and a clean handoff.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 |
| Work Slice | DESKTOP-001-D7 / I285: integrate prior Desktop stages, fix cross-stage acceptance defects including task-page overflow, run reproducible candidate checks and guide/record H1-H6; no new product feature or release |
| Claimed At | 2026-09-24 |
| Source Issue | #29 |
| Governance Claim PR | #608 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | PR #608 merged as `e1a89a1b25092ad37860314f02d1d212055b9fcc`; I282-I284 are merged with technical gates; #29 is the existing cycle acceptance tracker |
| Implementation PR | #609 merged as `422729caecdc8e76c7de8217654921723d5a3fdf` from exact head `8cc7563906e061070b36a50a08367ca63c40d53a`; CI `36535972702` passed; independent Agent-role review `5885650377` approved exact head |
| Last Updated | 2026-09-30 |
| Handoff / Release Condition | H6 native evidence and owner-first closeout are required; no release |

## Published Baseline

### Selected Story And Dependencies

[DESKTOP-001-D7](../backlog/active/DESKTOP-001-D7-desktop-integrated-acceptance.md), parent DESKTOP-001;
Planned / Unclaimed, not yet Ready or activated.
Prerequisites: I282-I284 implementation stages merged with technical gates; acceptance ledger in #29.
Follow [the four-week task](../tasks/2026-09-22-desktop-four-week-delivery.md) for
serial scheduling, inventory, resource limits, authorization and deferred acceptance. Dates do not
activate implementation. Resolve readiness, then use one atomic claim/activation transition.

### Scope

- Converge cross-stage defects locally: focus/IME, scrolling, tool/result identity, permission placement, cancellation, recovery and stale evaluation.
- Reserve three days for fixes and two for acceptance/documentation; do not add new product features.
- Run a reproducible native candidate build and consolidated smoke/acceptance flow against the exact integrated head.
- Guide human checks one at a time, reusing prior valid evidence when code has not affected it; record exact head/device and remaining unsupported platform cases.
- Update launch/configuration/tool/approval/resume/evidence instructions and capability limits; synchronize owner-first completion and clean only verified merged branches/task-owned artifacts.

### Non-Goals

New features, retroactive claims of Windows/Linux native or physical latency acceptance, new installers/auto-update, release/version/tag/crates publication.

### Acceptance

- Given the candidate, the user can create, execute, approve/deny, cancel, restart/resume and inspect results with no mock data presented as live.
- All required H1-H6 acceptance rows have passing evidence or the cycle remains Partial/Review; unavailable pre-existing I277 rows remain explicitly separate residuals.
- The integrated current head passes applicable CI and independent technical/security/API review; documentation reflects actual capabilities and deferred limitations.
- A clean checkout can reproduce the documented build/launch; no signing, distribution, release or publication is implied.
- Completion records cite existing implementation commits and preserve earlier baselines; no open protected blocker is hidden in cleanup notes.

### Planned Validation

Full integrated deterministic E2E and locked release_preflight, Desktop feature builds in applicable CI, native H1-H6 ledger, bilingual docs/link checks, independent Agent-role integration review.

Use the pinned toolchain and --locked. Before a stable code candidate, run focused checks,
./scripts/release_preflight.sh, both governance validators, diff/secret review and applicable
independent Agent-role review. Exact-head remote CI validates the stable stage. Planning-only
changes do not require Rust builds. Required human rows remain Review until accepted.

### Documentation To Update

- Desktop crate README with delivered launch/behavior instructions (create in I282).
- README.md and README.zh-CN.md; site English/Chinese capability instructions where affected.
- Child owner, this iteration, four-week task, parent summary, indexes and #29 acceptance evidence.

### Risks And Rollback

Device availability can delay manual closure. Record Partial with exact outstanding rows rather than marking Complete. Corrections that expand scope require explicit replanning.
Preserve working mock/preceding stage. Disable incomplete live features rather than weakening
safety. Roll back only the failing slice; no destructive cleanup of user sessions or workspaces.

## Actual Activation And Execution

2026-09-24: I282-I284 implementation stages are merged to main. Governance PR #608 merged as
`e1a89a1b25092ad37860314f02d1d212055b9fcc`, activating I285. No implementation code was included
in the governance slice.

## Verification Evidence

No I285 implementation checks are claimed by this governance-only activation. The I284 Desktop
suite passed 102/102 on 2026-09-24. Planning and inherited human acceptance evidence are recorded
centrally in the four-week task and Issue #29.

## Completion Evidence

### Native acceptance checkpoint — 2026-09-25

Local candidate based on `e1a89a1b25092ad37860314f02d1d212055b9fcc`, with uncommitted
Desktop changes; macOS arm64, native window and trackpad. Latest approval candidate executable
SHA-256 before the subsequent artifact-layout fix:
`ded35ebe8189f31c714699a9583e5b75e55cc1ad1f10b2ea61a680e26b667d9e`.
Earlier screenshots exercised earlier local builds; they are not exact-head release evidence.

- H1 partial: real provider returned OK, SECOND_OK and follow-up answers. Provider-error UI remains untested.
- H2 observed: model-reviewed pwd, denied bash, approved-once write (13 bytes, I285_WRITE_OK),
  a fresh approval for another write, session approval reuse and a fresh approval for a different path.
  User confirmed the second-round write received its own Allow once; do not attribute it to the first round.
- H3 partial: streaming cancel and approval-wait cancel stopped the UI. Agent read-only `ls -ld`
  confirmed i285-scope-check-20260925.txt absent; user confirmed Cancelled. Running-tool cancellation remains untested natively.
- H4 partial: restart restored durable history; a subsequent model request recalled END_SCROLL.
  Switching between two saved sessions preserved distinct histories and kept B_SESSION_0925/ACK
  out of the first session. Restart after a successful write/no replay remains to verify.
- H5 observed: short/long output uses one page scroll, resize, Chinese multiline IME,
  locale switch preserving input, directory-picker cancel and Tab/Shift-Tab passed user checks.
- H6 partial: successful file changes appeared and unavailable evaluation preserved Finished
  and prior output. Production current/stale evaluation evidence remains unavailable.

Local corrections: evaluation state no longer overwrites conversation status; live navigation
exposes saved sessions; approval displays exact JSON arguments under the existing bounded request
and request-ID lifecycle. Focused host tests passed 46/46, Clippy and builds passed.
Residuals: artifact metadata horizontally overflowed (now locally changed to stacked path/identity,
native retest pending); saved-task labels are opaque; cancellation followed by a new instruction
once produced a model conflict response (cause unproven). Permission presentation still needs
independent security review before a stable candidate. No whole-row or cycle completion is claimed.

### Native acceptance continuation — 2026-09-26

The earlier generic session-reuse report was insufficient: direct file inspection found
I285_SECOND_WRITE rather than the expected I285_SESSION_WRITE_OK. That earlier report is not
proof of a successful repeated write. Controlled follow-up used the same `edit` tool and path:
after session approval, direct inspection confirmed I285_SESSION_WRITE_OK (21 bytes), then
I285_EDIT_REUSE_OK (18 bytes); the maintainer confirmed no second approval. After restart,
the same edit/path requested fresh approval; cancellation left I285_EDIT_REUSE_OK and mtime
1790352037 unchanged. A different-path request also required approval and was cancelled;
the agent directly confirmed that scope-check file absent.

H4 post-write recovery: inode 384731270, size 17, mtime/ctime 1790337801 and SHA-256
311a64bcf449e380acdb413887ad0fcf0b51937ae24e0fffd8b3dd8aa76ba505 were unchanged across
the earlier restart/resume, with content I285_SECOND_WRITE. No repeated write was observed.
The temporarily empty recent-task list recovered after selecting the original workspace.

H3 running-tool cancellation: maintainer explicitly confirmed bash had begun executing
`sleep 30` before Cancel, and the UI immediately entered Cancelled. This establishes native
UI behavior, not a direct descendant-process inspection; automated cleanup tests remain required.
Remaining native checks: provider-error presentation and the stacked artifact-row layout.
Current/stale production evaluation remains unavailable. Test file remains in the selected
workspace for bounded acceptance; remove only after verification and explicit cleanup scope.

Cancellation-boundary correction: cancelled durable history now ends with a Runtime-authored
assistant interruption marker, preserving completed effects while distinguishing the next user
submission from continuation of the cancelled request. Focused `talos-agent` Session tests passed
35/35, including cancelled durable replay. On 2026-09-28 the maintainer retested an idle Cancel
after a successful turn: status remained Finished. A streaming 5000-line task then transitioned
from Cancelling to Cancelled and stopped output. Full Desktop tests passed 103/103. The earlier
stuck-Cancelling observation preceded the local correction and is not counted as a passing result.
The Cancelled-to-new-request sentinel probe returned CANCEL_BOUNDARY_OK after the correction.

### Product-flow and visual-design variance — 2026-09-29

Code/design comparison confirmed that the live host is a real Runtime/provider/tool/permission
integration, but not the designed Mission-first product workflow. The current live task page accepts
a goal/prompt and sends it into the conversation without structured plan proposal, user review, or a
confirmed Mission baseline. It primarily presents conversation output, controls, and a per-session
Todo-backed work projection; the Mission overview/progress and task-local navigation prototypes
remain fixture-backed. The user also confirmed that high fidelity to the archived design images is
required, not merely reuse of their palette.

Real execution plumbing therefore does not establish product-flow or visual conformance; the user's
impression that this was a mock was reasonable. This is product work outside I285's published
scope. Do not silently expand I285 or claim the live host conforms. The follow-up is registered as
[DESKTOP-001-MISSION-UX](../backlog/active/DESKTOP-001-mission-first-desktop-workflow.md), Refinement /
Unclaimed, using existing Issue #29 rather than creating a per-task Issue. MISSION-UX is not selected or
activated; first dispose I285's outstanding acceptance/status truthfully, then map the shared
Runtime/Work contracts and confirm how to implement the Mission baseline lifecycle without a
Desktop-owned store. Its visual acceptance explicitly compares the four archived reference
surfaces at high fidelity.

### Consolidated human acceptance state — 2026-09-29

This table supersedes the initial H1-H6 summary above where later checkpoints provide newer
evidence. Historical checkpoints remain intact. Evidence below is native/user-observed unless
explicitly identified as automated; observations made on an unidentified local binary are not
exact-candidate release evidence.

| Row | Current result | Remaining action / limit |
|---|---|---|
| H1 — real provider and error presentation | Real configured-provider success responses were observed. | Provider-error/timeout presentation is not verified. No safe, isolated error-injection control is currently available; do not alter real credentials/configuration or simulate failure by disrupting networking. Keep open until a safe test path exists. |
| H2 — Auto, approvals and file effects | Auto/model decision, human deny, allow-once, session grant, a fresh request for another path, and resulting file effects were observed. Session grants were confirmed not to survive host restart. | Preserve as integration evidence; retest only if the relevant permission path changes before final candidate. Independent security/API review remains a separate gate. |
| H3 — cancellation | Streaming cancellation, approval-wait cancellation, and cancellation after a running shell command had begun were observed. Idle Cancel remained Finished; streaming transitioned Cancelling -> Cancelled and stopped output. The cancelled-to-new-request sentinel returned `CANCEL_BOUNDARY_OK`. | Native UI behavior is observed. Descendant/process-tree cleanup is established by automated tests, not direct native process inspection; do not claim the screenshot alone proves cleanup. |
| H4 — resume and isolation | Restart/resume restored history; two saved sessions remained isolated; post-write file fingerprint/timestamps did not change on resume, showing no repeated write. | Existing evidence is sufficient for these scenarios; exact final-candidate recheck is required if session/recovery code changes. |
| H5 — input, layout and navigation | Chinese multiline IME, locale switching, directory-picker cancel, focus traversal, page scrolling and resize were observed. On 2026-09-29, user screenshots after creating `i285-layout-check.txt` showed path, operation, turn, call and session metadata in the live artifact-evidence region without horizontal clipping. A follow-up narrow-window screenshot showed the complete tool result after scrolling, including the completion marker, byte count, preview and Chinese result, with no overlap or truncation. | The 2026-09-29 binary SHA is unknown. The screenshots are not a test of a product `Changes` page (none exists in live UI) and do not establish the full Mission-first product flow. |
| H6 — changes, evaluation and delivery | Successful file-change evidence was shown. Unavailable evaluation preserved Finished/output and did not claim Delivery. | Production current/stale Evaluation remains unavailable because no authoritative live subject/evidence producer is wired. This is a shared contract/source gap, not a remaining manual interaction; do not use fixtures to claim PASS. |

Current actionable native check: none for the H5 layout/scroll scenario; the narrow-window
artifact evidence is complete at the observed viewport. Provider-error presentation is deferred
pending a safe injection path. Product Mission flow and high-fidelity reference surfaces remain
outside I285 and are tracked by MISSION-UX.

Completion Commit: `422729caecdc8e76c7de8217654921723d5a3fdf` (implementation merge; I285 remains Review / Partial because H1/H6 are unresolved).
Only already-existing implementation/evidence commits may close this iteration.

### Stable Candidate Merge Checkpoint — 2026-09-29

PR #609 was merged after merge-time CAS with exact head `8cc7563906e061070b36a50a08367ca63c40d53a`,
base `e1a89a1b25092ad37860314f02d1d212055b9fcc`, successful exact-head CI run `36535972702`,
and independent Agent-role review `5885650377`. The implementation is now in `main` as
`422729caecdc8e76c7de8217654921723d5a3fdf`. This merge closes the implementation candidate only;
it does not close H1 provider-error presentation or H6 authoritative current/stale Evaluation
evidence, and it does not complete the four-week task.

### Local Candidate Checkpoint — 2026-09-29

The local I285 candidate includes the bounded integrated corrections already present in the
working tree: terminal-cancel state handling, distinct Evaluation-unavailable presentation,
real permission arguments, durable cancelled-turn closure, stacked artifact evidence metadata,
and content-driven page scrolling/recent-task navigation. Focused Desktop tests passed `103/103`,
and the `talos-agent` cancelled-turn recovery test passed. One full Desktop run first exposed the
known process-group cleanup timing failure in
`runtime_host::cancellation_tests::running_shell_interrupt_waits_for_descendant_cleanup`; the
same test passed when rerun alone and the subsequent full run passed `103/103`. This remains a
timing-sensitive residual to observe on the stable candidate, not a claim of permanent repair.

H5 is complete for the observed macOS narrow-window layout/scroll scenario. H1 provider-error
presentation remains unverified because there is no safe isolated error-injection path, and H6
production current/stale Evaluation remains unavailable because no authoritative live evidence
producer is wired. Neither gap can be closed by a fixture or ordinary manual click-through.

Completion Commit: `422729caecdc8e76c7de8217654921723d5a3fdf` (implementation merge; residual acceptance remains open).

### Residual Audit Checkpoint — 2026-09-29

The provider audit confirmed that `MockProvider::with_error` and localhost HTTP servers used by
provider tests provide deterministic error/timeout coverage for automated tests only. The Desktop
production configuration has no safe, isolated failure-injection control for a configured provider.
We therefore do not treat fixture/mock output, credential changes, or network disruption as H1
native evidence. H1 remains open pending an explicitly authorized injection contract.

The same audit found no new production session-bound criteria/evidence producer for H6.
`RuntimeEvaluationService` and the Desktop host continue to fail closed with `EvaluationUnavailable`
when authoritative subject, revision, or evidence is absent. No fixture result is promoted to
production authority, and H6 remains an API/source residual rather than a remaining click-through
step. PR #604 is an unrelated review thread and is intentionally excluded from I285 scope.

### Full local preflight — 2026-09-29

`CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
./scripts/release_preflight.sh` exited 0 after the workspace checks and tests and both external
Runtime SDK fixture variants; each fixture printed `talos-runtime external fixture passed`, and
the script ended with `release preflight: passed`. The profile flags reduce regenerable debug
build output; they do not change the preflight gates. This validates the current uncommitted local
candidate, not an exact-head remote CI run. H1/H6 and independent integration/security/API review
remain outstanding.

## Variance And Residuals

The 2026-09-24 below-the-fold observation was corrected: the live task page uses content-driven
viewport overflow scrolling, and output grows with content under the same page scroll rather than
using a collapsed nested scroller. Later native checks confirmed page scrolling, resize and long
output behavior. See the consolidated H1-H6 table above for current evidence and remaining actions.

I277's separately deferred VoiceOver, reduced-motion, Windows/Linux native interaction and physical
display measurements remain outside this I285 manual batch and are not counted as passed.

### Current Acceptance Update — 2026-09-30

I286 closed H1 on 2026-09-30: the configured-provider path and permission gate were observed, and
the opt-in debug error/timeout path exercised native error presentation, cancellation and recovery.
The four-week owner records the exact evidence and limits; synthetic injection is not described as
a real remote-provider timeout.

I287 has since implemented the production session/revision-bound Evaluation evidence source and
Desktop wiring on `main` at `b6e63d85`; exact-head CI `36690452723` passed all six jobs. This
supersedes the 2026-09-29 H6 statement that the authoritative source was missing. H6 is still open:
the native flow using built-in `edit`, a valid current Evaluation, a later workspace mutation,
stale/unavailable Evaluation and disabled Delivery has not been confirmed by the maintainer. The
edit symptom raised from the screenshot also lacks captured tool arguments/error text; the anchored
EOF regression tests do not establish that the screenshot symptom is fixed.

I285 remains Review / Partial until that native H6 evidence and owner-first closeout are recorded.
No fixture, unit test or exact-head CI result substitutes for this human observation.

### Post-I287 Reconciliation — 2026-10-01

I287 is now merged in `main` as `f0e16e857ea76caa7c386fac5006b9f53a86d67a` after exact-head
CI `36792450056` and independent security/API approval `5921753941`. The authoritative
session/revision-bound source is therefore implemented and no longer a missing-source residual.
Native H6 evidence recorded current content, restart unavailability and changed-file stale state
with Delivery blocked. Mission-level Delivery success is not a requirement of H6. Review / Partial
is retained for I287's observed assistant-prose snapshot/anchor disclosure investigation under
ADR-045 and final owner reconciliation, not additional H6 click-through testing. I277's deferred
device/accessibility rows remain independent and non-blocking for this cycle.

### Final Integrated Owner Closeout — 2026-10-03

Completion Commit: `422729caecdc8e76c7de8217654921723d5a3fdf`,
`c97f35b9`, `f0e16e857ea76caa7c386fac5006b9f53a86d67a`,
`8f795d653aeebb8a2f78f30a70115bcc7358adfd` (already-merged
implementation evidence for integrated candidate, H1, H6 and ADR-045 privacy
correction). The four-week ledger records native H1-H6 observations; the
latest H6 confirms current, restart-unavailable and external-change-stale
Evaluation with fail-closed Delivery. PR #634 passed all six exact-head CI
jobs in run `37043830735` and independent Agent-role security/API review in
comment `5958202669`; merge-time CAS passed. The full local preflight and
affected tests passed before that merge. I277's pre-existing platform,
accessibility and physical-display rows remain separately deferred in #29.
The accepted Mission-first workflow/design gap is the unclaimed successor
DESKTOP-001-MISSION-UX, not a hidden part of this development candidate.
