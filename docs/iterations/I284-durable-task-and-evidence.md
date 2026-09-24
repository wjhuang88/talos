# Iteration I284: Desktop Durable Tasks And Evidence

> Document status: Review — Claimed
> Plan date: 2026-09-22
> Target window: 2026-10-06 to 2026-10-12
> Planned objective: A Desktop user can reopen a saved conversation and inspect real work status, changes and revision-bound evaluation evidence.
> Baseline rule: once committed, preserve scope and append execution facts; changed targets use a new ID.
> MVP deliverable: A Desktop user can reopen a saved conversation and inspect real work status, changes and revision-bound evaluation evidence.

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
| Authorization Evidence | I283/D5 implementation and governance closeout merged as `81a5d27c`; I284 is the next serial child |
| Implementation PR | #607 merged at exact head `1fc73c2e`; merge commit `2845ebdf` |
| Last Updated | 2026-09-24 |
| Handoff / Release Condition | Technical implementation merged; retain Review until H4/H6 and remaining production evaluation-source acceptance are resolved; no release |

## Published Baseline

### Selected Story And Dependencies

[DESKTOP-001-D6](../backlog/active/DESKTOP-001-D6-durable-task-and-evidence.md), parent DESKTOP-001;
Planned / Unclaimed, not yet Ready or activated.
Prerequisites: I283 implementation merged with technical gates; ADR-042/061; durable-session and WORK-001 projection compatibility map.
Follow [the four-week task](../tasks/2026-09-22-desktop-four-week-delivery.md) for
serial scheduling, inventory, resource limits, authorization and deferred acceptance. Dates do not
activate implementation. Resolve readiness, then use one atomic claim/activation transition.

### Scope

- Use existing durable Session identity/storage for recent task navigation and transcript resume; no Desktop business database or automatic tool replay.
- Show authoritative Work/Goal projection when available. Missing projection remains unavailable, never replaced by fixture work.
- Present read-only actual file/artifact change evidence with source/task identity; do not infer a complete task diff from arbitrary workspace dirt.
- Integrate existing completion-claim/evaluator/Mission gate interfaces where supported, with explicit evaluation action rather than autonomous resubmission. No executor self-certification.
- Show evaluation missing/fail/inconclusive/stale/current states and Delivery eligibility from the shared gate. Persist only through supported contracts; never resurrect an old PASS after restart.

### Non-Goals

New durable Mission/Evaluation schema, automatic evaluation on every turn, multi-window/reconnect, full artifact editor/merge UI, #308 presets or complete autonomous Mission planning.

### Acceptance

- Given two sessions, reopening one restores only its own durable transcript and workspace association; switching tasks cannot mix approvals or tool results.
- Given an interrupted session, restart/resume does not repeat writes or invent missing output; storage errors become a safe visible failure.
- Given execution artifacts or changes, show their actual provenance, or explicitly mark unavailable attribution; inspection itself performs no mutation.
- Given missing/stale evaluation or a changed Goal revision, the UI cannot display current PASS or eligible Delivery; final model text does not certify success.
- A live Desktop-to-shared evaluation fixture covers pass/fail/staleness; absent durable evaluation storage displays unavailable after restart rather than claiming persistence.

### Planned Validation

Restart/resume/session-isolation fixtures, storage failure and no-replay tests, shared work/evaluation projection and revision-staleness matrix, read-only evidence boundary review; native H4 and integrated H6.

Use the pinned toolchain and --locked. Before a stable code candidate, run focused checks,
./scripts/release_preflight.sh, both governance validators, diff/secret review and applicable
independent Agent-role review. Exact-head remote CI validates the stable stage. Planning-only
changes do not require Rust builds. Required human rows remain Review until accepted.

### Documentation To Update

- Desktop crate README with delivered launch/behavior instructions (create in I282).
- README.md and README.zh-CN.md; site English/Chinese capability instructions where affected.
- Child owner, this iteration, four-week task, parent summary, indexes and #29 acceptance evidence.

### Risks And Rollback

Shared P4 is not proof of durable evaluation storage. Resolve facade access before implementation; any required new public boundary/schema needs decision review. An unavailable-state fallback is honest but does not close unmet baseline acceptance.
Preserve working mock/preceding stage. Disable incomplete live features rather than weakening
safety. Roll back only the failing slice; no destructive cleanup of user sessions or workspaces.

## Actual Activation And Execution

2026-09-23: activation PR #606 merged to `main` as `4e150b3e` after I283 technical and governance
closeout. The first local implementation slice binds configured Desktop RuntimeHost instances to
workspace-scoped durable Sessions through the shared `talos-session` facade. Successful turns are
persisted and readable after shutdown; no automatic tool replay or Desktop-owned database was added.
Local implementation commits `08df087e`, `beed4624`, `ad4a8665`, `1f466182`, and `9f549cd6`
now provide stable workspace+goal identities, durable host persistence, and a read-only tool
provenance projection. The old unparameterized configured-host entry was removed after the
focused check exposed it as unused.

## Verification Evidence

Implementation checks: `cargo test -p talos-session --locked` (189 unit/integration tests plus
6+1+15+1+2+2+2 integration tests) passed; the full Desktop mock suite passed 80/80 with
`--locked`, and `cargo check -p talos-desktop --locked` passed. The durable host integration test
verifies a successful turn is persisted and reopened through the workspace-scoped binding; task
discovery, source attribution and explicit unavailable evaluation display are covered. A full
artifact diff viewer and persisted evaluation staleness remain unfinished in this iteration.

## Completion Evidence

Completion Commit: `2845ebdf57ff26c62539a8516499ed7d29caa228` (PR #607 implementation merge).
This is implementation evidence only; the iteration remains Review while required human and
production-source acceptance rows remain open.

## Merge Checkpoint — 2026-09-24

PR #607 exact head `1fc73c2ea03356d4711625764e060f3eb3b8c32d` merged into `main` as
`2845ebdf57ff26c62539a8516499ed7d29caa228`. Exact-head CI run `35959834215` completed
successfully with all six jobs green: Linux Desktop explicit feature, Windows Rust workspace,
Format/Check/Clippy/Test, Windows installer fixture, change classification and remote owner
reconciliation. Incremental independent review approved the same exact head against base
`4e150b3e1bc6910b3a02d75b249573ecff8191e7`, confirming the only final correction was the
independent runtime SDK fixture lockfile and that no production/API/permission scope changed.

The merge closes the technical implementation gate, not the full acceptance gate. H4 (restart,
resume, session isolation and no repeated writes) and H6 (integrated real task, change inspection,
evaluation and Delivery eligibility) remain deferred human rows in Issue #29. Production still has
no authoritative session-bound acceptance-criteria/evidence producer, so normal Evaluate remains
fail-closed as unavailable; the test-only harness does not count as production evidence. I285
remains the next planned stabilization/acceptance iteration.

## Variance And Residuals

The current UI has workspace-scoped recent-session discovery and explicit resume, reads the shared
Work graph for the bound session, and lists bounded file differences observed around successful
built-in file-tool calls. This evidence is limited to the open host, does not retain file contents
or show a content diff, and does not cover shell/custom tools, symlinks, outside paths or files
larger than 1 MiB. Earlier-run artifact evidence remains unavailable after restart. No durable
Mission/Evaluation projection is exposed to Desktop, so Evaluation and Delivery remain unavailable.
H4/H6 remain human acceptance rows in #29. Carry eligible human rows without transferring
protected security gates.

## Local Review Correction Checkpoint — 2026-09-23

PR #607's prior exact head `8e8ebf6dde23af4286badd1b66d1dfcf41d62c24` received two blocking
findings. Local corrections replace lossy workspace/goal sanitization with domain-separated
SHA-256 identities, preserve legacy task bindings while redacting their old embedded IDs from the
Recent Tasks UI, and make Resume open an existing identity only. The existing-only mode survives
host restart attempts; a missing binding reports an error and does not create a replacement.
Interrupted pending-write restart coverage now proves transcript restoration without tool replay.
Tool request paths are presented as request metadata; actual artifact-change attribution remains
explicitly unavailable. Recent-task SQLite reads now run on GPUI's background executor.

Local verification on the unpushed candidate passed `cargo check -p talos-desktop --locked`,
`cargo clippy -p talos-desktop --features desktop-ui --all-targets --locked -- -D warnings`,
and `cargo test -p talos-desktop --features desktop-ui --locked` (83/83). Formatting, diff checks,
and both governance validators passed; each validator reported 0 warnings. The PR's old exact head
`8e8ebf6` failed the same Desktop Clippy gate on a nested condition in the previous Resume path;
the local candidate replaces that path, and the exact CI command now passes locally. The full
`./scripts/release_preflight.sh` passed on this local candidate, including workspace
check, Clippy, tests, doc-tests and both Runtime SDK fixtures. Stable candidate commit `889c1cf2`
was pushed to the existing PR #607. Exact-head CI run `35856041872` passed the main
Format/Check/Clippy/Test job, Linux Desktop explicit-feature job, change classifier, remote Issue
reconciliation and Windows installer fixture. The Windows Rust workspace job completed its tests
and smoke steps but remains `in_progress` during `Swatinem/rust-cache` post-job cleanup; it has no
failure conclusion yet and is not counted as a completed green job. Independent review for the
new exact head is still pending. Do not reuse CI/review for old head `8e8ebf6` as evidence for the
corrected candidate.

At this local review checkpoint, the live UI still lacked the shared Work projection and
execution-bound artifact event. The later Work and artifact evidence checkpoint below records the
subsequent implementation. H4/H6 still require natural-person acceptance through #29.

## Task Isolation Correction — 2026-09-23

Local commit `61fe13ae` makes New Task use a fresh opaque identity scoped to the selected
workspace, keeps legacy bindings explicitly resumable, and waits for idle-host shutdown before
replacing the host. Active turns and pending approvals block switching; generation checks fence
stale observer updates. New regression coverage verifies the stop boundary, close priority and
stale generation rejection. Desktop UI tests passed 87/87, and Desktop all-target Clippy with
`-D warnings` passed.

PR #607 remains at remote head `6714fd79` / base `4e150b3e`; its CI and review do not validate
`61fe13ae`, which is local-only. Full release preflight passed workspace checks/tests/doctests and
the first Runtime SDK fixture, then failed the second fixture build because the disk ran out of
space. The failed preflight is not acceptance evidence; `cargo clean` removed 19.1 GiB of this
repository's generated `target/` artifacts. A fresh full preflight remains required before the
next stable remote candidate.

At this historical checkpoint, shared Work projection and execution-bound artifact evidence were
still unavailable. The local implementation below records their subsequent state. The durable
Mission/Evaluation source and required H4/H6 human acceptance remain unresolved.

## Local Work And Artifact Evidence Checkpoint — 2026-09-23

The uncommitted local implementation reads the shared Work graph in read-only mode, scoped to the
bound session ID. A Runtime hook snapshots only workspace-contained regular files no larger than
1 MiB before and after successful built-in `write`, `edit` and `delete` calls. It emits
session/turn/call/path evidence only when the bounded digest differs; it rejects symlinks and
outside paths, checks the same tool/path identity at both boundaries, caps pending snapshots at
128 and clears unfinished entries at turn completion. The UI labels these as differences observed
around successful file tools and shows their session, turn and call identity without exposing file
contents.

Local verification: `cargo test -p talos-desktop --features desktop-ui --locked` passed 93/93;
`cargo clippy -p talos-desktop --features desktop-ui --all-targets --locked -- -D warnings`,
`cargo fmt --all -- --check` and `git diff --check` passed. Focused tests cover creation, no-op,
failed calls, path mismatch, outside paths, size limit, symlink rejection and turn cleanup. The
RuntimeHost integration test starts a DurableSession, resolves real approval, executes the shared
`write` tool and verifies the resulting file and session/turn/call/path evidence event. The shared
session projection tests passed with `cargo test -p talos-session --locked`.

PR #607 remains at remote head `6714fd79aab36b67e80a02228922204deb74f0f3` against base
`4e150b3e1bc6910b3a02d75b249573ecff8191e7`; its CI/review evidence does not cover this local
work. The previous full preflight failed during its second SDK fixture build from disk exhaustion;
a fresh full preflight is required before publishing the stable candidate. Durable artifact history,
content-diff display, shared durable Mission/Evaluation storage, and H4/H6 human acceptance remain
open. I284 is not Complete and I285 remains Planned.

## Evidence Boundary And Evaluation Source Audit - 2026-09-24

This checkpoint supersedes the preceding unfinished-work description, not the Published Baseline.
New durable Mission/Evaluation storage is explicitly excluded; neither a new persistence schema
nor a full content-diff editor is a prerequisite to finishing this iteration. The required missing
behavior is the explicit shared evaluation action and revision-bound state/Delivery projection.

Local artifact capture now waits for an Allow result after permission resolution. Pre-permission
handling only retains call metadata; permission-event arguments remain redacted. Capture uses a
held directory capability, nonblocking/no-follow opens, descriptor type/size checks and bounded
digest reads. One retained permit limits outstanding reads per host. A 500ms observer timeout does
not pretend to cancel an OS read; bounded executor teardown reports incomplete cleanup as Error
while that read remains alive. Per-call normalization performs no filesystem access. Independent
Agent-role review found no remaining blocker in this artifact slice; this is not broader I284
approval. Synchronous host-startup filesystem work remains a disclosed limitation.

Current local validation: Desktop explicit-feature suite 98/98; all-target Desktop Clippy with
`-D warnings`; formatting and diff checks passed. Regression cases include unresolved/denied
permission, a FIFO in a watchdog-protected child process, root replacement, missing-root lexical
normalization, stalled-read cleanup receipts and actual approved shared-tool execution. Full
candidate preflight and exact-head remote validation remain outstanding.

Source audit: `talos-runtime` reexports `IndependentEvaluator` and its input types, but exposes no
production completion-claim source or evaluation command. `talos-session` has no Mission/claim
source; its current Todo graph does not provide acceptance criteria, current workspace revisions
or independent Mission verdicts. Repository constructions of `CompletionClaim` and
`MissionEvaluation` currently occur in core/agent tests. Thus the missing integration is not merely
reading an existing database. Do not manufacture Mission/Goal identities, convert final model text
or Todo completion into PASS, or count a fixture-only source as a delivered live action.

ADR-083, defining that handoff while keeping storage and mode choices open, was accepted by the
maintainer on 2026-09-24. Acceptance authorizes implementation of the shared boundary, not a new
durable schema or autonomous evaluation. The next slice must add a real session-bound producer and
explicit Runtime action; Desktop must not manufacture identity or revision values. Preserve H4/H6
in #29.

The first shared slice now adds `talos_runtime::RuntimeEvaluationService`. It reuses the existing
independent evaluator and accepts only caller-supplied revision-bound claims and provenance-bound
evidence; it does not create identities, persist results, mutate Work, or infer Delivery. The
Desktop/session producer and explicit UI action are still required before I284 can close.
`cargo check -p talos-runtime --locked` passed after adding the direct `tokio-util` dependency.

`RuntimeEvaluationContext` now creates ephemeral Runtime-owned Mission/Goal identities and accepts
the caller's verified `WorkspaceRevision`; it snapshots immutable criteria into a claim and does
not guess the workspace revision. A focused runtime test passed. This is the producer foundation,
not proof that Desktop has a live revision source or an evaluation button yet.

2026-09-24 follow-up: `RuntimeEvaluationService::delivery_gate` now delegates to the deterministic
shared `MissionGate`; its focused regression confirms a missing Mission evaluation remains blocked.
Runtime evaluation-context/gate tests passed 2/2. After removing an unconnected whole-workspace
scanner that would read arbitrary files without providing the evaluator their contents, the clean
Desktop rebuild passed `cargo test -p talos-desktop --features desktop-ui --locked` (99/99) and
all-target Clippy with `-D warnings`. Formatting and diff checks passed. The explicit Desktop
evaluation action remains unimplemented: there is no trusted production producer for acceptance
criteria, validation evidence contents/provenance, or workspace revision, and a digest alone is not
evaluable evidence. No PASS or Delivery result may be fabricated from it. This remains an I284
implementation gap under accepted ADR-083, not a scope reduction.

The Desktop now exposes an explicit `Evaluate` action and Runtime command. Until an authoritative
claim/criteria/evidence producer is connected, it emits a visible `EvaluationUnavailable` result;
it does not call the provider. The regression
`explicit_evaluation_without_authoritative_source_reports_unavailable` passed. This closes the
fail-closed interaction path, not the live evaluation acceptance row.

`RuntimeEvaluationService::new` now accepts an injected independent assessor, while the provider
constructor remains the production adapter. This enables a Desktop-to-shared deterministic fixture
to exercise accepted PASS, FAIL and stale-revision outcomes without treating fixture data as live
authority. The fixture still needs to be wired through a real session-bound context before the
acceptance row is complete.

2026-09-24 Runtime contract checkpoint: `cargo test -p talos-runtime -p talos-desktop --features
talos-desktop/desktop-ui --locked` passed (Runtime 50/50, Desktop 100/100). The Runtime test
`runtime_evaluation_contract_passes_and_stale_goal_blocks_delivery` uses an injected deterministic
assessor and explicit fixture claim/evidence to verify that a valid current PASS can reach
`MissionGate::Eligible`, while changing the required Goal revision blocks as `StaleGoalEvaluation`.
This is Runtime service/Gate contract coverage only.

The initial clean build after the authorized `cargo clean` took 2m11s for the affected Runtime and
Desktop suites. The generated artifacts are retained for subsequent local convergence. Current
disk availability after the build is approximately 11 GiB; avoid another workspace-wide rebuild.

2026-09-24 Desktop host fixture follow-up: the test-only `EvaluationHarness` sends
`RuntimeCommand::Evaluate` through `run_host`, calls the shared `RuntimeEvaluationService`, applies
`MissionGate`, and returns `EvaluationResult` over the same output channel consumed by the UI.
`evaluate_command_projects_shared_pass_fail_and_stale_delivery` passed all three cases: current
PASS is Delivery eligible, required FAIL is blocked, and an otherwise-PASS report for an older Goal
revision is blocked as stale. The normal production host constructor supplies no harness and
continues to return `EvaluationUnavailable`; fixture values cannot become production authority.
Desktop tests passed 101/101, Runtime tests passed 50/50, all-target Desktop and Runtime Clippy
passed with `-D warnings`, both governance validators reported 0 warnings, and format/diff checks
passed.

Remaining: resolve a real producer boundary under ADR-083 without manufacturing Mission/Goal
authority or using digest-only artifacts as evaluable evidence. The host fixture is now connected,
but no production session-bound acceptance/evidence source exists. Keep H4/H6 and production live
evaluation open.

The Desktop presentation now keeps evaluation as a structured current state (Unavailable, Running,
or Result with Delivery eligibility) instead of relying only on transcript text. This prevents an
older displayed PASS from being the current UI state after a later stale/unavailable result. The
production state remains Unavailable until a real session-bound producer is available.

Independent review on 2026-09-24 confirms production `RuntimeHost` still has no session-bound
criteria, Goal authority, or workspace-revision lifecycle; the test harness must not become
production authority. Future producers must invalidate results after writes or subject changes.
The current `talos-session` Todo projection confirms this: it exposes only session-owned
WorkUnit identity/revision, title/description/status/priority/tags, and no Mission/Goal or
acceptance-criteria authority. Therefore converting Todo completion into an evaluation claim would
violate ADR-083.

2026-09-24 cancellation/UX correction: evaluation futures now run alongside the host command loop,
so Interrupt and Shutdown do not wait for the evaluator deadline. A cancellation drops the pending
evaluation and emits no late verdict; the production no-authority path returns one explicit
`EvaluationUnavailable` per request, while a test-only harness emits `EvaluationStarted` and a
structured result. Desktop tests passed 102/102 after correcting the no-authority double-request
regression. This does not change the production-source gap: current session-bound criteria,
workspace revision and evidence are still unavailable.
