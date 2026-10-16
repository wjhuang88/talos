# Iteration I304: Absent-Journal Pause Without Initialization

> Document status: Review
> Published plan date: 2026-10-15
> Planned objective: avoid initializing SQLite only to pause an absent pending journal.
> MVP deliverable: real Actor/Runtime shutdown preserves receipts without creating an empty journal.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex |
| Work Slice | Absent-journal pause no-op under the existing pending-store guard; storage and real Actor/Runtime shutdown regressions and documentation only. Existing-journal durability, admission authority and shutdown policy remain unchanged. |
| Claimed At | 2026-10-15 |
| Source Issue | None |
| Governance Claim PR | #722 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer authorized continuing the intermittent-timeout investigation and grounded repair using GitHub CI. No distinct maintainer reviewer is available; independent Agent storage/custody/API review, exact-head CI, both validators and merge-time CAS are mandatory. |
| Implementation PR | Not started |
| Last Updated | 2026-10-15 |
| Handoff / Release Condition | No implementation branch or committed implementation before finalized claim reaches main; stop if the optimization requires new custody semantics or an ADR change. |

## Published Baseline

Selected Story: [RUNTIME-011](../backlog/active/RUNTIME-011-absent-journal-pause.md), Ready,
no parent. Preserve its scope, exclusions, acceptance, validation and documentation targets.
Depends on merged #711 and accepted ADR-008/056/058/063. This draft does not activate work;
finalized proposed Active/Claimed records become effective only upon target-branch merge.

### Change-Control Decision

This is a different operation and independently acceptable outcome from I301's recovery-read
optimization. Preserve I301 unchanged in Review, do not widen its published baseline or use
#705's bounded CI exception for production. Source/guard/transaction/error and admission review
must prove the write no-op safe before implementation acceptance. No new API/unsafe/dependency.

### Selection Inventory And Dispositions

Baseline main `9b7ba28119c2edeca86f4af5a65e110fe1f4a4f5` includes #720. All iteration
headers inventoried2026-10-15; no Blocked header found (template excluded).

| Owner | State | Disposition |
|---|---|---|
| I293 / I294 | Active / Claimed | Retain Search router/provider ownership and #716; no takeover. |
| I300 | Active / Claimed | Retain Git permission repair. |
| I301 | Review / Claimed | Retain startup acceptance, #711 merged; new scope does not close it. |
| I302 | Active / Claimed | Retain self-only Windows PSS evidence/#717, including #716's independent diagnostic work. |
| I303 | Active / Claimed | #720 merged, #721 Skill implementation; no takeover of Skill/runtime context APIs. |
| I277 | Review / deferred device | Retain deferred validation. |
| I249 | Planned | Retain dependency-pilot deferral, no dependency changes. |
| I164 | Paused | Retain superseded layout, do not reactivate. |
| DEPENDENCY-003-A | Review / Claimed | Retain separately owned native dependency evidence. |

I162 historical Complete and I290/I291/#590 remain terminal. Open PRs checked: #705 diagnosis,
#716 Search plus separately owned I302 test evidence, #721 Skill. No matching RUNTIME-011/I304
owner or overlapping absent-journal pause production candidate found. Continued maintainer
direction and existing explicit non-overlap authorization apply only to this new bounded slice.
Shared Agent/Runtime files require surgical hunks and latest-main integration before publication;
do not overwrite Skill or Search additions. Board/index unions remain batched governance.

### Risks / Rollback

The write operation's successful return feeds durable reconciliation: returning zero is safe only
as a valid empty snapshot, not as a new custody/admission guarantee. Existing acknowledged work,
ParentDir ambiguity, real errors and WAL transaction behavior must be proven preserved. If review
finds a semantic conflict, stop/narrow rather than change shutdown authority under this claim.
Revert only this slice on regression; keep historical failed CI evidence and original budgets.

## Actual Activation And Execution

2026-10-15: governance-only draft; no implementation branch, production or test edit.
CI3182 evidence in #705 comment6184082259 independently recounted2219pass/1fail/0ignored out of
2220 executions. Six ordinary gates passed, baseline diagnostic failed. Successful direct caller
spans justify bounded optimization, not complete timeout attribution. Independent Agent log/source
review confirms the distinction; shared identity is not a distinct human formal approval.

## Verification / Completion Evidence

Finalized atomic proposal #722 is ineffective until main merge. Both validators, exact-head CI,
independent review and merge-time CAS are required.
No implementation or Completion Commit; not Complete. Future execution appends, not replaces, this plan.

## Variance And Residuals

#705 retains evaluator5s composite failure phase, native child latency, old10s attribution and
existing-journal costs. I301 retains its own owner-first closeout; no release or #590 reopening.

## Target Integration Checkpoint — 2026-10-15

CI3191/run38679683657 completed SUCCESS on original head374bafe4: four required documentation
jobs passed, Windows Rust/Linux feature were skipped by trusted documentation routing. This is
governance evidence only. Before merge, actual main advanced to `bdea322fc30a724c2407daa27972f92ba1b34d4b`
through #721 implementation and #723 owner-first SKILL-006/I303 Complete/Closed. Preserve those
records and production bytes. Resolve the single Board conflict as a union of the I304 proposal
and current I303 completed row; retain auto-merged backlog/index updates. Original selection
inventory above is history; I303 is now terminal, #721 closed, #716 and #705 remain open and
separately owned. No overlapping pause owner appeared. I301 remains Review. Claim #722 is still
ineffective, no production/test implementation exists. Fresh exact-head governance CI/review and
both validators/CAS are required for this integrated candidate, without retrying CI3182.

Local integrated-candidate environment limit: workspace maintenance removed the previous checkout
and compiler. Public Git sources were recovered and the official Rust1.99.0 GNU archive's SHA256
verified before installation. Installed compiler/driver/LLVM bytes match the extracted originals,
with28GB disk available; nevertheless even `rustc -vV` terminates with SIGBUS. The last loader
diagnostic names LLVM initialization; the actual cause remains unresolved. Immediate binding
and an independent physical copy did not recover execution;
ptrace diagnosis is unavailable in this sandbox. Cargo reports1.99.0 normally. Project governance
therefore actually FAILED at locked Cargo metadata's compiler probe; its delivery fixtures passed.
Claim validation passed0warnings and diff/union review passed. No local full-project success is
claimed, no pin/dependency/lock/assertion/CI policy is changed. The unchanged reduced-documentation
CI must run both complete validators successfully on the exact integrated head in its capable
environment before merge; independent review and CAS remain mandatory. This is a retained local
environment failure, never replacement success or a waiver of the remote validation gate.

## Effective Claim And Implementation Checkpoint — 2026-10-15

Claim #722 merged as `23e659e2403739b74bc018ec46305346e8e2c7ad` after exact integrated
head `13656eb21d1c3d93d7752c0beb247d10a4d27898` passed CI3195/run38684420862:
four required documentation jobs passed, two Rust jobs mechanically skipped. The completed
job116255949682 ran both full governance validators successfully. Independent exact-remote
review6184799657 and CAS6184836666 precede merge. Shared maintainer identity is not another
natural person's formal approval. The preceding proposal/blocked-toolchain records remain history.

The rebuilt candidate starts from later main `0ef5a532930642d730bcc6d974f2a79605efaaee`.
Its only extra main change is #725's separately owned PROVIDER-008 intake/Proposed ADR-093.
I303 is Complete; I301 remains Review; I293/I294 Search, I300 Git, I302 PSS/#716 and #705
diagnostics retain independent ownership. No release or #590/I290/I291 reopening is implied.

Production change: acquire the original store guard, reuse the existing error-preserving
`sidecar_exists`, and return zero only on verified absence. Existing/ambiguous paths execute the
unchanged connection/PRAGMA/SQLite immediate transaction/schema/UPDATE/commit/drop path. The
helper comment now says operation snapshot. No new admission fence, cache, connection lifetime,
schema, SQL, dependency, feature, lockfile, native call, unsafe or API is introduced.

Storage regressions cover no directories/files on absence and later durable admission; existing
Accepted/Running/Paused exact records, receipts, payloads and FIFO after reopen; independent-store
pause/admission interleaving and post-ack pause; invalid parent, malformed database and schema;
ParentDir with previously acknowledged custody and empty conservative initialization; poisoned
guard; final/ancestor symlinks, directory links and loops. Permission-denied inspection is also
asserted for pause before restoring the fixture's mode (privileged NotFound remains an explicit
alternative). Windows may normalize ParentDir before access; only the intermediate-directory
creation assertion is Unix-specific. No ignored test or weakened original assertion is added.

The existing real Actor legacy-submit fixture now checks sidecar absence AFTER shutdown/task
join. A real Runtime empty-shutdown fixture verifies complete/idle/durable-zero receipts,
post-close `RuntimeClosing`, the legacy joining wrapper and no `.talos/runtime` storage. Its own
one-second bound is not an edit to an existing deadline. Existing admission/start-fence,
nonempty pending, terminal and truthful reconciliation-failure fixtures remain unchanged.
This is the SDK/Actor storage slice, not a claim of new CLI functionality or full product closure;
post-repair real Windows Auto/binary comparison remains acceptance work below.

### Retained Previous Local Attempt (Not Rebuilt-Head Evidence)

The prior scratch workspace and unpublished candidate were removed before this continuation.
The following are retained execution notes from that attempt, not reproducible raw logs or
validation of this rebuilt candidate. Independent review must examine the new exact candidate.

- A real test-first run of `absent_pause_creates_no_storage_and_later_admission_remains_durable`
  failed before the production change at the no-parent-directory assertion. After the change,
  `cargo test --locked -p talos-session` passed241 tests; the exact permission fixture passed with
  observed PermissionDenied. Agent passed495; Runtime passed66/failed1 at the unchanged Unix
  socket fixture (`runtime_evidence_rejects_directory_symlinks_and_socket_entries`, Operation
  not permitted). These were not an affected-crates full pass.
- Standard `./scripts/release_preflight.sh` passed site/installers, both governance validators,
  text/classifier/fmt, workspace check and workspace Clippy `-D warnings`, then failed workspace
  tests: completed suites1941passed/1failed/0ignored. The failure was the unchanged Plugin
  `wasm::tests::language_provider_admission_accepts_complete_boundary` is_ok assertion, not the
  Runtime socket case (not reached). SDK validation was not reached. The assertion did not expose
  its error, so the root cause is UNKNOWN; a default500ms admission deadline is a source fact,
  not proof that contention caused the failure. No Plugin fix or retry-to-green is authorized by
  this storage slice. Preserve the failed attempt even if the rebuilt run succeeds.
- Prior GNU SIGBUS was bypassed by a separately installed, SHA-verified official GNU1.99 toolchain;
  native procedural macro and real project builds worked. An intermediate musl experiment found
  extracted core.rmeta shorter than its verified archive member; streaming restoration recovered
  its probe. That does not establish the cause of earlier GNU SIGBUS or extraction truncation.
  No project pin/lock/flags or CI policy changed. All that temporary environment is now gone.

### Rebuilt Candidate Validation And Remaining Gates

The current Rust1.99.0 installation reports b940084d7/LLVM23.1.1; unchanged locked dependencies.
Results below will be appended from this checkout before stable publication. Historical successes
above cannot stand in for the new candidate. No assertion/deadline/skip/filter/CI waiver is allowed.

Still required: stable exact-head native Windows/macOS full CI, both validators, independent
storage/custody/API review and merge-time CAS. After implementation merge, compare a separately
bounded #705 diagnostic candidate against preserved CI3182 while retaining Auto15s/cleanup10s/
tool120s/evaluator5s and all historical failures. That later Windows evidence, original composite
failure phase, native-child latency, old10s attribution and existing-journal cost remain open.
Review / Claimed is truthful; no Completion Commit or universal timeout guarantee exists.

Rebuilt local checkpoint: before changing production, the exact absence test executed1case and
FAILED at `!dir.path().join("missing").exists()` (0pass/1fail/0ignored). After the guarded change,
`cargo test --locked -p talos-session` PASSED241 tests (214unit/27integration,0fail/ignored;
doctests0). The exact permission fixture separately PASSED1case with actual PermissionDenied,
so the new pause error assertion ran. Independent Agent static review against base0ef5a532
APPROVE found no blocking storage/custody/API/scope defect; final exact-head validation remains
separate. Standard preflight has passed both governance validators0warnings, text23/classifier14,
site/installers/fmt and is currently running workspace checks. Final outcome is appended below.

Rebuilt standard `./scripts/release_preflight.sh` actually exited101: workspace check PASSED,
workspace Clippy `-- -D warnings` PASSED, then completed test suites2195passed/1failed/0ignored
before fail-fast stopped at Runtime. Its73 Runtime unit cases were72pass/1fail; the sole failure
is the unchanged Unix socket fixture at lib.rs2138: PermissionDenied / Operation not permitted.
The new Runtime empty-shutdown, Actor post-join absence, existing start-commit/grace/fence and
truthful durable-reconciliation failure tests all PASSED. Plugin's73 unit tests all PASSED,
including the old admission assertion; this does not explain or erase its previous failure.
No filter/ignore/waiver was applied, and this is NOT a local full-preflight pass. SDK and later
workspace suites were not reached by that invocation. Capable native CI must run the unchanged
full suite before merge. Separate `cargo clippy --locked -p talos-session --all-targets --
-D warnings` also PASSED. Agent/Runtime doctests and both external SDK modes are being completed
separately below; they do not replace the failed standard invocation.

Independent environment observation: a disposable Python stdlib probe importing no Talos code
also failed with PermissionError errno1 / Operation not permitted at `socket(AF_UNIX, SOCK_STREAM)`,
before bind. This supports the local socket-creation capability limit; it is not a passing Talos
test and does not waive native execution. Separate locked Agent/Runtime doctests PASSED12/0,
0fail/ignored. The external SDK validator confirmed its independent Cargo root and one direct
Talos dependency; its two executable modes are still running at this checkpoint.

Final local fixture outcome: the separate chained validation exited0. Both independently locked
external SDK executable modes (default and `--features coding`) printed
`talos-runtime external fixture passed`; the independent-root/single-direct-dependency boundary
check passed. No workspace or fixture lockfile changed. Final staged diff review covers exactly12
intended files and no generated artifacts/secrets/unrelated owners. This supersedes only the
in-progress checkpoint wording above, never the retained failed full preflight. Final governance,
format and diff checks are rerun on these completed notes before publication. Exact remote CI,
review/CAS and Windows comparison remain acceptance gates; Review / Claimed remains unchanged.
