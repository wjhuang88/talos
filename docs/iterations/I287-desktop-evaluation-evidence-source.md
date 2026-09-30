# I287 — Desktop Evaluation Evidence Source

> Document status: Active / Claimed
> Planned objective: Provide a production-authoritative, session-bound Evaluation evidence source so Desktop distinguishes missing, stale and current evidence and gates Delivery without fabricated PASS results.

## Scope

- Define the authoritative producer, subject identity, workspace revision and provenance for Evaluation evidence.
- Connect the existing Desktop projection to current/stale/missing evidence without a Desktop-owned business store.
- Preserve fail-closed `EvaluationUnavailable` behavior until authoritative evidence exists.
- Add deterministic revision/staleness/Delivery tests and a native H6 acceptance recipe.

## Out Of Scope

- No new durable Mission schema or migration without a separate accepted contract.
- Fixtures, harnesses, or manually entered results cannot be production evidence.
- No new evaluator authority, global event bus, scheduling system, or unrelated Desktop redesign.

## Readiness And Acceptance

- Requires an accepted production Evaluation-source contract and owner before implementation.
- Evidence is bound to a session/task subject and workspace revision with provenance.
- Desktop distinguishes missing, stale and current evidence; Delivery requires current authoritative evidence.
- Restart/resume and changed-workspace cases cannot replay or upgrade stale evidence.
- Locked tests cover source lifecycle, revision mismatch, unavailable evidence and Delivery gating.
- The maintainer can reproduce and record native H6 evidence in #29 using the exact candidate build.

## Governance

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | `codex / single maintainer` |
| Work Slice | Runtime-owned session-bound Evaluation evidence producer, bounded workspace revision, Desktop projection wiring and H6 validation; no durable Mission/Evaluation schema |
| Claimed At | 2026-09-30 |
| Source Issue | #29 |
| Governance Claim PR | Direct commit 7d405136 |
| Authorization Mode | Direct commit |
| Authorization Evidence | Maintainer accepted ADR-084 contract on 2026-09-30 |
| Implementation PR | Direct-to-main implementation commits `80b2b806`, `c266aaf3`, `d24147d9`, `b122067d`, `b94407c5`, `cff9fbef`, `b6e63d85` |
| Last Updated | 2026-09-30 |
| Handoff / Release Condition | Implementation merge plus independent security/API review and native H6 evidence |

ADR-084 is the accepted production contract for this slice. Implementation starts locally from the
current `main` and must remain within the Work Slice above.

Activation provenance correction (2026-09-30): `5bb36663` is the starting baseline, not an
existing claim activation commit. The maintainer accepted ADR-084 and repeatedly authorized
local I287 commits. Local commit `7d405136` recorded both governance and partial implementation;
this did not follow the SOP's separate claim-before-implementation sequence and is not evidence
of remote activation or merge approval. Earlier uncommitted implementation must not be described
as having started from an already-published claim. Reconcile this local checkpoint before any
remote candidate; do not treat local commit authorization as a remote-gate waiver.

## Local Review Checkpoint — 2026-09-30

The initial local implementation passed 50 Runtime tests, 109 Desktop tests, and both governance
validators with zero warnings. Independent Agent-role security/API review returned REQUEST CHANGES;
these passing tests are not candidate approval or H6 acceptance.

Required local corrections before a stable candidate:

- Hash-only capture must not become criterion-level PASS; connect a real semantic validation
  producer before claiming production Evaluation is available.
- Derive revision and evidence from the same bounded capture and recheck the current subject after
  model evaluation, before presentation and Delivery.
- Bound file reads, directory traversal and depth, preserve cancellation responsiveness, and use
  a confined-open boundary that rejects symlink replacement races.
- Prevent external construction of authoritative snapshots; include complete Mission/Goal revisions
  in their integrity identity.
- Define subsequent goal/claim lifecycle and add production-path coverage for changed goals,
  workspace changes during evaluation, unavailable evidence and restart.

Partial corrections now keep hash-only evidence Unavailable and reject it before model assessment,
make snapshot fields private, include Mission/Goal revisions in the digest, and limit per-file reads
before allocation. These do not yet resolve the complete review. I287 remains Active / Claimed;
H6 and the stable candidate commit remain pending. The maintainer has authorized the local candidate
commit after convergence and requested native acceptance only after local tests/docs are complete.

Snapshot follow-up: revision and record identity now derive from the same collected content digest,
with a subsequent stability check; path lengths are framed in that digest and native workspace path
bytes avoid lossy identity collisions. Added assertions cover Mission/Goal revision changes and
rejection of the previous subject after file modification. This does not prove an atomic filesystem
snapshot or resolve confined-open races, semantic evidence, or post-evaluation revalidation.

Traversal follow-up: snapshots now stop at 4,096 inspected directory entries or depth 32, including
empty-directory trees that do not consume the regular-file limit. A regression exercises both
guards. These limits complement the 256-file/16-MiB bounds; safe confined opens and cancellable
blocking isolation remain required before the production path is accepted.

Host follow-up: every explicit Evaluate request now creates its own context from the requested
goal/current workspace, rather than caching the first goal indefinitely. After assessment the host
reads the workspace revision again and calls `observe_current_subject` before projecting a verdict
or invoking Delivery. An unreadable workspace returns Unavailable. Existing Runtime contract tests
exercise workspace-only staleness and irreversible invalidation of the old PASS; a production-path
race regression and asynchronous confined scanning are still required.

## Local Implementation Checkpoint — 2026-09-30 (historical; superseded below)

Confined capture now pins a cap-std directory handle and uses one-component no-follow directory
traversal and no-follow/nonblocking file opens, validating the opened handle before reading.
Eight focused Runtime evidence tests passed, including ambient root replacement, directory
symlink and socket rejection. Replacing the ambient root leaves this source pinned to the old
directory; reconciling that identity with the live tool workspace remains a production residual.

The authorized local commit preserves this partial implementation, not a push-ready or accepted
H6 candidate. Remaining I287 work: Runtime-produced semantic observations without fabricated
validation PASS, cancellable blocking isolation, production host race/restart/goal-change tests,
independent security/API approval, and native acceptance. Goal verdict and Mission-level Delivery
must remain distinct; missing Mission evaluation cannot be synthesized. No broad workspace
content upload or automatic validation-command execution is authorized by this checkpoint.

Local verification after confined capture: `./scripts/release_preflight.sh` passed formatting,
workspace locked check and workspace Clippy with `-D warnings`. Its workspace test compilation
was interrupted with SIGINT (exit 130) when free disk fell to approximately 1.6 GiB; the full
preflight and full workspace test suite have not passed for this checkpoint. Prior focused tests
are not a substitute for that remaining gate. No remote candidate was pushed.

Semantic-evidence follow-up: local code now adds a separate bounded artifact-observation request
field, rather than converting file collection into validation PASS. Runtime hooks register matching
permission-allowed successful file calls; Desktop requests fresh confined content observations.
Four registry tests and thirteen evaluator tests passed. Snapshot work now uses a single retained
blocking-worker slot with bounded caller wait. Full production integration/race tests, native-feature
build, incremental security/API review and H6 remain outstanding. The API migration is recorded in
ADR-084. These results do not supersede the remaining full-preflight gate above.

Latest local correction (2026-09-30): the bounded snapshot retains per-file digests and rejects
artifact content or deletion observations that do not belong to that exact snapshot, including an
A-to-B-to-A race. Source and registry verify the pinned workspace directory identity before and
after capture and fail closed if the ambient tool root was replaced. The Desktop production-path
test covers an authorized write, current evaluation, workspace mutation during assessment,
superseding submission cancellation, and an empty post-restart registry. The observation contract
is deliberately limited to current contents at an authorized successful path; hooks do not claim
an atomic execution handle for arbitrary embedding tools.

Edit-boundary follow-up (2026-09-30): commit `b6e63d85` adds a regression test for existing
single-line EOF-preservation behavior and tool descriptions; it does not change the edit algorithm. All ten
focused anchored-edit tests pass, including single-line/multiline EOF handling, mixed terminators,
stale revisions and concurrent edits. This verifies the tool implementation contract, not the
model's tool selection or argument formation in the native Desktop flow. The maintainer flagged an
edit-tool defect from the H6 screenshot; its exact tool error/arguments are not captured in this
checkpoint, so the screenshot symptom is not yet proven resolved. Native H6 must exercise the
built-in `edit` path and retain the actual tool result; delete/write substitution is not acceptable.

Remote candidate evidence (2026-09-30): `b6e63d85` is present on `origin/main` and its exact-head
GitHub Actions run `36690452723` has six completed successful jobs, including the Windows Rust
workspace and Linux Desktop feature checks. This establishes implementation and CI evidence only;
it does not close H6 or replace the required native edit/evaluation observation.

## Current Implementation And Acceptance State — 2026-09-30

The I287 implementation is on `main` through `b6e63d85` (base `6f651a28`). Exact-head CI
`36690452723` completed all six jobs successfully. Focused Runtime/evaluator/tool tests and the
Desktop mock feature build passed during local convergence; the latest anchored-edit suite passed
10/10. Independent Agent-role security/API review approved the exact implementation head. This
replaces the earlier checkpoint's statements that the production path, security/API review and
exact-head remote validation were still outstanding; those older entries remain as execution
history, not current status.

H6 remains open for native acceptance. The maintainer has not yet confirmed an exact live sequence
that exercises the built-in `edit` tool, obtains a valid Evaluation report, then changes the file
and observes stale/unavailable evidence with Delivery disabled. The screenshot-raised edit symptom
is not yet tied to captured tool arguments or an error result, so it must not be represented as
resolved by the EOF unit test. Next step: run the exact `b6e63d85` Desktop candidate and capture
the native edit result plus current-to-stale Evaluation transition. I287 and the four-week task
remain Active / Review, not Complete, until that evidence and owner-first closeout are recorded.

### Native edit failure and local correction — 2026-09-30

The maintainer's 17:21 screenshot shows three native edit calls rejected with
`invalid type: integer 1, expected a string`. The model's narration attributes it to start/end,
but the generic error does not identify the actual field; that attribution is not established.
The earlier claim that a missing-final-newline defect had been fixed was incorrect.

The local correction documents full line:hh anchors in the generated schema and identifies
invalid field types by field/index without echoing transient values. Integer-to-anchor synthesis
was considered and removed before validation: no compatibility expansion is included. Eleven
focused anchored-edit tests pass, including malformed request/no mutation, private-value-free
errors, corrected retry and existing stale-revision checks. Native model retry and H6 acceptance
remain unverified; this is a local candidate, not a new exact-head CI-approved implementation.

Native edit retest (2026-09-30, 17:37 screenshot): the rebuilt local candidate's read returned
H6_INITIAL and its native edit result showed the expected diff to H6_CHANGED for h6-check.txt.
The displayed sequence contains no delete/write/shell substitution. This demonstrates one
successful native read/edit sequence, not universal model argument correctness or a post-error
automatic retry. Current Evaluation and subsequent invalidation/Delivery checks remain open.
The assistant prose also echoed transient snapshot/anchor information despite sanitized tool
results; record this as a projection/privacy residual requiring investigation under ADR-045.

Native Evaluation observation (2026-09-30, 17:42 screenshot): after the successful edit, the
live host emitted Evaluation started and then Verdict(Inconclusive), with Delivery Blocked
for GoalNotPassed. This proves a parsed evaluator result reached the native UI and that the
non-passing result did not permit Delivery. It does not prove the goal passed or explain why
the evaluator found evidence insufficient; the visible result contains no criterion rationale.

Corrected continuation recipe: submit a read/edit request changing H6_CHANGED to
H6_CHANGED_AGAIN, then inspect the current evaluation before pressing Evaluate again.
Submit itself invalidates the previous evaluation; a successful artifact-change event also
clears it. This sequence verifies submission/tool-change invalidation, not detection of an
external filesystem mutation. A later Evaluate creates a fresh context and may return a new
verdict; it is incorrect to require that fresh verdict to be Stale. Mutation during assessment
and post-restart unavailability remain distinct checks. H6 remains incomplete.

Compatibility verification: all 110 talos-tools library tests with file-write enabled and
locked/offline Clippy passed. The added matrix exercises every anchored operation, optional
delete end=null, unused-field tolerance and subsequent legacy string replacement. The prior
EOF, stale-snapshot and concurrency checks also pass. These are local candidate results.

Native invalidation observation (2026-09-30, 17:43 and 17:44 screenshots): the next turn
displayed `Evaluation unavailable: a new task request invalidated the previous evaluation`
before read/edit successfully changed H6_CHANGED to H6_CHANGED_AGAIN. This confirms native
submission-triggered invalidation and a second successful edit. It does not independently prove
external-mutation detection, artifact-triggered invalidation, or a fresh evaluation of the new
contents. The next guided step is one Evaluate request without another file change or restart.

The 17:45 native reevaluation failed with malformed report / invalid UUID character '<'.
This is not a passing evaluation. Inspection found literal UUID placeholders in the provider
prompt template. The local fix replaces that template with serialized real claim/subject/criterion
identities and a generated report ID, defaulting verdicts to inconclusive. No response rewriting
or relaxation of evidence/identity validation is introduced. All 14 evaluator tests passed,
including direct template parsing with multiple criteria and a non-default workspace revision.
Fresh native evaluation remains required after rebuilding; restart clears ephemeral evidence.

Native retest (2026-09-30, 17:49 and 17:53 screenshots): in the restarted process,
turn_83864_1_2 successfully edited h6-check.txt from H6_CHANGED_AGAIN to H6_FINAL,
with a Runtime-observed artifact entry. Evaluate then returned Verdict(Inconclusive)
and Delivery Blocked / GoalNotPassed, without the previous malformed UUID error.
This verifies the corrected request produced a parseable native result on this attempt;
it does not establish Goal PASS or universal provider correctness. The exact criterion
and reason for inconclusive are not visible in these screenshots. Code inspection confirms
Evaluate uses goal_input, not the latest conversation instruction; inspect that input
before attributing this verdict to a specific missing piece of evidence. H6 remains open.
The provider-boundary regression now checks that the actual isolated outgoing messages
contain a directly parseable report with matching claim/subject/criterion identities;
all 15 evaluator tests pass with --locked --offline. This synthetic regression is not
a substitute for the native evidence above.

The 17:57 screenshot confirms the Goal input contained the latest read/edit procedure,
not an older goal. The earlier stale-goal hypothesis is excluded. A subsequent content-only
criterion was submitted as turn 3; read returned H6_FINAL. Its 18:02 Evaluate attempt failed
with `missing field id`. The raw response is not captured, so a top-level versus nested
missing id cannot be established. Inspection found the prompt specified top-level fields
but omitted the non-empty finding contract. The local request now includes the complete
JSON Schema generated from EvaluationReport, including nested required fields and evidence
references. No missing values are filled in and strict report validation remains unchanged.
The outgoing-request regression verifies the finding requirements are actually sent; all
15 evaluator tests pass. Native acceptance remains failed/pending until a rebuilt candidate
is tested; prompt completeness alone cannot guarantee model compliance.

The desktop-ui debug binary rebuilt successfully with --locked --offline after that change.
The additional `cargo clippy -p talos-agent --lib --tests --locked --offline -- -D warnings`
check failed on 43 unwrap_used diagnostics in unchanged test code (including background_jobs,
auto_resolver and context); this broader check must not be reported as passing. It does not
invalidate the successful evaluator tests but remains a distinct validation limitation.

### Evaluator contract audit — 2026-09-30

Maintainer requested a systematic check after repeated malformed native responses. Static
inspection and independent Agent-role review identified the following open corrections;
they are not established causes of the screenshot's missing-id error:

- ProviderEvaluatorAssessor has no accumulated output-size bound; custom assessor output
  also reaches synchronous parsing without a size guard.
- Its internal timer starts after stream dispatch. IndependentEvaluator's outer timeout
  protects Desktop, but direct assessor callers lack that dispatch deadline.
- Stream closure and any TurnEnd are treated as success; unlike bounded_model, incomplete
  stop reasons and missing explicit completion are not rejected. ToolCallStarted and
  ToolResult are ignored rather than rejected, and provider errors are forwarded verbatim.
- Required PASS evidence is checked, but optional/non-PASS results and finding references
  are not all checked against supplied evidence identities.
- Core report validation permits a result to reference a finding belonging to a different
  criterion and permits Blocking findings alongside PASS. These contradictions require
  regression tests and rejection at the authority boundary.
- The prompt asks the provider to preserve report ID, but that template identity is not
  checked; claim and subject binding are separately enforced and remain intact.
- Desktop discards per-criterion findings when projecting a report, leaving the maintainer
  unable to see why a result is inconclusive. Diagnostic improvements must retain the
  transient-data/privacy boundary, not dump raw responses into UI or logs.

The existing bounded_model path already bounds output, covers dispatch in its deadline,
rejects incomplete streams/tool events, and sanitizes provider errors. Audit that reusable
boundary before introducing another evaluator-specific implementation. No claim that Auto
has been exhaustively audited is made. H6 remains pending; repeated manual retries are not
a substitute for resolving these confirmed contract gaps. Production-library Clippy passes;
the broader test-target limitation above remains distinct.

Local audit corrections: core report construction now rejects cross-criterion finding links
and PASS with applicable Blocking findings (including unlinked/global findings). Six core
evaluation tests pass. The evaluator checks all result/finding evidence references against
supplied records and requires valid supporting evidence for optional as well as required PASS.
Provider responses must explicitly end with EndTurn; truncated/closed streams and tool events
fail closed. Dispatch and reception share a deadline; dispatch panics and provider errors
produce fixed failure messages. Text accumulation and custom-assessor raw responses are capped
at 256 KiB before parsing. Eighteen evaluator tests pass with --locked --offline, including
dispatch panic/timeout and incomplete/oversized stream regressions. These local corrections
remain uncommitted and require review; reasoning-payload bounds, report-ID binding, diagnostic
projection and native acceptance are still open. No full closure is claimed.

Further local corrections: ProviderEvaluatorAssessor now rejects missing/changed template
report IDs without changing the public assessor signature. All assessors still pass through
shared claim/subject/evidence validation; a custom assessor has no provider-template ID contract.
Thinking deltas and completed reasoning (including signatures/redacted payloads) count toward
the output limit using text + max(thinking, completed reasoning), without retaining their text.
Cancellation is selected first and checked again before returning a report; custom-assessor
panics are converted to failure. JSON parse diagnostics contain category/line/column only, not
response values. Global Blocking findings cannot coexist with aggregate PASS even for optional-only
claims. All 22 evaluator tests and core/agent production-library Clippy pass with --locked --offline.
Remaining: diagnostic projection, additional edge-case coverage and final independent review,
followed by rebuilt native H6 acceptance. No commit/push or native success is claimed here.

Diagnostic projection follow-up: Desktop now shows criterion verdicts in claim order and bounded
model-authored findings (16 entries, 512 non-control characters each), with truncation/omission
notices and an explicit no-findings message. This does not serialize raw reports or explicitly
project identity fields; it is not a guarantee that arbitrary model prose cannot mention an ID.
Independent Agent-role read-only review found a result-order numbering bug; it was corrected to
look up results by criterion identity. Desktop's 111 tests passed before that correction; the
focused diagnostic regression additionally covers reordered results and bounded Unicode/control
text. Core evaluation has 7 passing tests and evaluator has 22, including optional-only global
Blocking and hidden reasoning/combined-output limits. H6 still requires real-provider acceptance;
none of this evidence closes the four-week task or the native malformed-report failures.

Full preflight attempt: site/installers, governance, text boundary, workspace check and workspace
Clippy passed. Workspace test compilation failed with OS error 28 / No space left on device;
the remaining tests and external SDK fixture were not completed. This is a failed preflight,
not a gate waiver. The repository's rebuildable target/debug/incremental cache was removed
(approximately 5.9 GB apparent size), restoring 4.8 GiB available space while retaining the
native acceptance binary. Runtime's separate locked/offline library regression passed 59 tests.
The bilingual Desktop README now distinguishes Evaluate from Send and explains bounded findings
and fail-closed malformed reports. Native H6 and full preflight remain required.

Second standard preflight attempt with CARGO_INCREMENTAL=0 completed workspace check, Clippy,
all workspace tests and doctests successfully. The final external Runtime SDK fixture compilation
failed with No space left on device while building its independently locked dependencies.
Thus the full preflight still failed; fixture acceptance is outstanding. After termination,
`cargo clean -p talos-tui` removed 495.3 MiB of rebuildable package artifacts; the Desktop
acceptance binary was preserved. Do not rerun a large build without adequate disk capacity.

Native candidate identity (uncommitted local build, 2026-09-30):
`target/debug/talos-desktop-mock` SHA-256
`95bd276baaadfec47eb1fa3057bc9c32997cd9084cbb7d77965578b5b7641897`.
This identifies the binary for subsequent H6 evidence, not an implementation commit or CI head.
The local changed-file inventory is evaluator.rs (agent), evaluation.rs (core), Desktop
runtime_host.rs/README.md, tools file_tools/write_edit_tools.rs and tests.rs, and the I287/I285
owners, Board and four-week task derived records. No Dashboard implementation or owner changes
are present. A further package-scoped `cargo clean -p talos-cli` removed 974.2 MiB of rebuildable
CLI artifacts; Desktop remained intact, with 1.2 GiB free afterwards. CLI debug artifacts require
rebuilding. SDK fixture validation and native H6 remain outstanding before final closure.

SDK validation recovery: after removing only rebuildable target/debug/deps and target/debug/build
artifacts (retaining the native acceptance binary), available disk returned to 18 GiB.
`CARGO_INCREMENTAL=0 python3 scripts/validate_runtime_sdk_fixture.py` then exited 0: both default
and coding modes compiled and printed `talos-runtime external fixture passed`. This completes the
previously failed final preflight component on unchanged code/lockfiles; the prior full-script
attempts themselves remain failed runs, not retroactively successful. Workspace tests/check/Clippy
and this separately completed fixture constitute the recorded local checks. Independent Agent-role
whole-candidate review approved the uncommitted code, explicitly not an exact-head or closure approval.
Native H6 is still pending; no user observation has been supplied for the rebuilt candidate yet.

### Native H6 continuation and deadline correction — 2026-09-30

The 21:45 native screenshot records successful edit from H6_FINAL to H6_RECHECK and a
subsequent read confirming H6_RECHECK. The 22:13 screenshot records Evaluation Verdict(Pass),
Criterion 1 Pass and a content-based finding. Delivery remains Blocked / MissingMissionEvaluation,
as required for a Goal-only result. Read-only disk inspection independently found exactly ten
bytes H6_RECHECK with no newline. The assistant did not mutate the acceptance file.

The 22:31 screenshot records a second evaluation ending in evaluator deadline exceeded with
Delivery Blocked / MissingGoalEvaluation. This is not stale-revision evidence. Inspection found
Desktop supplied an eight-second total deadline despite the shared evaluator supporting up to
30 seconds. Local correction uses that existing 30-second bound and adds virtual-time shared
evaluation-service coverage for a valid response after nine seconds and failure beyond the deadline.
Production host stale, supersession and restart scenarios remain on real time. Model reasoning configuration is unchanged;
the screenshot does not establish whether network, reasoning or generation consumed the time.

Scope: runtime_host deadline and its regression tests, Desktop README, this owner checkpoint;
no new authority, public API or claim. Validation/build and native stale/restart acceptance remain
pending for this incremental change. The observed snapshot/anchor prose leakage remains open.

Incremental validation: all 112 Desktop tests passed with --locked, including the new 9s/31s
virtual-time regression and existing production host scenarios. The initial attempt to advance
virtual time inside the full host test failed its shutdown assertion; separating the evaluator
clock test from real Runtime shutdown fixed the test design without relaxing the assertion.
Both governance validators passed with zero warnings. This is local evidence, not native H6
closure or exact-head CI. Desktop locked build and Clippy with -D warnings subsequently passed.
Independent Agent-role incremental review approved the final local change, not an exact-head
candidate. Rebuilt target/debug/talos-desktop-mock SHA-256:
`ba6c0537c9131672b8298665339ee7b6b54463cfe70d0664988de45ee4d9c4ac`.
Native acceptance must restart this binary and produce fresh session-bound write evidence;
the earlier screenshots belong to the preceding binary. No remote candidate was pushed.

### Complete-content evidence semantics — 2026-09-30

The 23:03 screenshot reports a permission deadline failure and later stat output; it does not
by itself establish which earlier edit succeeded. Direct read-only file inspection confirmed
exactly H6_RECHECK_30S (14 bytes, no newline). The 23:10 native Evaluate returned Inconclusive,
not timeout: its finding acknowledged that content but questioned full-content/single-line/newline
coverage and missing execution evidence. This is not H6 success.

Inspection confirmed Runtime reads complete bounded UTF-8 artifacts, rejects oversized files,
and preserves whitespace. The provider prompt did not explain those semantics explicitly.
The local correction adds that contract to the system prompt, separates artifact observations
from validation-run evidence, and tells the model to evaluate only the stated criterion without
inventing execution requirements for content-only checks. It still forbids inferring test results,
performance or other-file changes, following embedded instructions, or inventing evidence.
No model verdict is rewritten. Coverage includes producer whitespace/empty/Unicode fidelity and
provider-boundary JSON round trips. Tests and build are pending. Independent Agent-role incremental
security/API review approved the local change; a real model is not guaranteed to return PASS and
native H6 remains open. Changes stay within I287.

Local verification completed: 22 evaluator tests and 4 Runtime artifact tests passed with
--locked; Desktop build and production Clippy (-D warnings) passed. The provider test confirms
the system evidence contract and exact Unicode/CRLF/multiline JSON content reach the request.
The Runtime test covers empty, Unicode, LF, CRLF, multiline and trailing-space content.
Rebuilt acceptance binary SHA-256:
`1f4170453a58a6e6c5381fedf6eb87219f1a932c3a6f242ecbd8eb771e3b1aa2`.
Native acceptance requires restart and fresh write evidence. No commit or push was made.
