# Session archive / recovery emergency

Status: Partial — parent writer repair and isolated recovery evidence; not deployed or complete.
Authorization Mode: Emergency override.
Authorizing maintainer: current user, explicit request to handle current-session restore and history-query failure.
Branch: feat/bg-default-continuation (existing worktree; preserve unrelated edits).
Incident: same session loses visible model context after -c. CLI bridge automatically archives >200 records and retains 50 plus a count-only placeholder. Startup reads head only. Archive compaction also preserves original parent IDs after adding a newly generated placeholder ID, incompatible with strict history recovery parent-chain validation.
Scope: stop automatic destructive head trimming; investigate archive reconstruction and recovery without weakening provenance validation. Never overwrite live session data during diagnosis.
Evidence: mode_runners.rs completion forwarder; compaction_engine.rs apply_rules; message_scan.rs begin_message_scan; checkpoint_history.rs parent validation. Local logs identify 4404450a-443c-479d-8153-f530d10faa21 at 08:27 startup with archive artifact dated 08:26. Artifact contents decoded during takeover below; completeness remains unverified.
Containment: remove CLI automatic compact_archived(50) call. Existing archives retained. Requires rebuilt/restarted host to take effect.
Validation: pending focused checks and regression. No claim of workspace validation or independent review.
Rollback: revert only this containment hunk, but doing so re-enables known lossy restoration behavior. Do not delete archives.
Remaining: reproduce broken parent chain in test; safely inspect archive records, reconstruct session-specific validated history into a separate artifact; design archive-aware restore/query with bounded reads and duplicate handling; exact diagnostics; tests/docs/security review; reconcile governance within two business days. No relaxation of parent validation or archive ownership.
## 2026-11-11 takeover evidence (Partial)

Requested outcome: recover history and diagnose history/checkpoint unavailable; preserve the dirty
worktree and all session data; do not default-enable background continuation.
Artifacts: parent writer repair, real persistence regressions, content-free diagnostics, isolated
recovery evidence. Status owner and residual destination: this document. No completion SHA,
commit, push, release, deployment, or live-session rewrite has occurred.

Hard constraints: original IDs/parents and checkpoint authority cannot be fabricated; unrelated
changes and session files must be preserved. Cold append now uses strict streaming record decoding to initialize the tip, retaining one record
plus an I/O page rather than the whole transcript. It has a five-second deadline; the maximum
individual host record remains uncapped for compatibility, so this is not a hard RSS bound.

Confirmed from production code and failing-then-passing regression:

- `Session::build_entry` cached a parent but never advanced it after append. Parent selection
  happened outside the write lock. Three real appends reproduce a stale shared parent; this is
  a writer bug, not evidence of a legal DAG/batch-parent contract. Existing durable commit and
  fork validation require the preceding entry. No parent validation was removed.
- Append now chooses and advances the tip inside the shared write lock, advances only after
  success, handles poison as an error, and durable commit updates the same cache. Fork identity
  gets its own empty cache. Long multiline tail restart and concurrent cloned writers are tested.
- CLI TUI, interactive and inline production entrypoints call `set_persistence`, not
  `set_durable_persistence`. `session/turn.rs` prepares HISTORY only for the latter. Therefore
  parent repair alone does not enable CLI durable history search. This is a source-level finding;
  no runtime trace proves which exact unavailable branch the user's old binary took.
- Diagnostic reasons now distinguish missing execution scope, no live snapshot/history,
  missing permission state/snapshot, missing durable owner, and parent mismatch without recording
  content, paths, IDs, tokens, or credentials. Other failure branches still share the generic error.

Preserved evidence: `/tmp/talos-recovery-evidence-20261111-170639/` (private directory).
36 TLOG/chain/archive files copied after per-file inode/length/mtime stability checks; each has a
SHA-256 manifest. SQLite online backups of the selected pending DB and session index both passed
integrity_check. These are individually consistent backups, not an atomic cross-file snapshot.
All 32 archive files were length-decoded from isolated copies. Exact identical-ID overlap from
session `4404450a-443c-479d-8153-f530d10faa21` selected 25 candidate archives. The other two live
session heads have zero ID overlap. No stable-ID content conflicts were found.

Candidate corpus: 4313 distinct records including 25 count placeholders. Excluding placeholders
leaves 4288 original records with a unique order constrained by observed record adjacency and no
missing original parent IDs. Original parent values and raw record bytes are retained in
`candidate-original-records.jsonl`, alongside source filenames and hashes. This is a recovery
evidence corpus, not an authorized TLOG, checkpoint, semantic summary, or completeness proof.
Seven unmatched archives are not silently attached. Live files were not rewritten.

Validation:

- `cargo test --locked -p talos-session -j 1`: passed (316 unit tests plus integration tests;
  before adding the explicit archive known-gap characterization). A final rerun including it passed (317 unit tests plus integration tests).
- `cargo test --locked -p talos-agent --lib checkpoint_ -j 1`: 86 passed, 0 failed. Includes
  real ordinary append + event + atomic durable commit + event + reopen, then actual registered
  history tool discover/search. Permission/current-source checks remain in force.
- `archive_reopen_preserves_original_records_but_does_not_recover_older_messages` documents the
  known gap with 205 real writes → archive(50) → reopen: only 50 model messages, with original
  retained parents rather than a fabricated link to the placeholder. Passing is not recovery acceptance.
- `./scripts/release_preflight.sh`: governance checks passed; stopped at pre-existing formatting
  differences in background continuation changes. No full workspace green claim.

Remaining acceptance (not delivered): CLI-owned bounded history preparation/source authority;
legacy-parent recovery with explicit provenance rather than relabeling original records; validated
archive ownership beyond overlap candidates; tool-pair boundary and repeated-overlap recovery;
bounded context restoration with genuine source-backed summary and explicit unloaded history;
actual host discover/search and compact Applied; complete diagnostics, full preflight, independent
review and deployment. No default background continuation change. Continue from the isolated
corpus and production CLI wiring; do not rerun blind directory concatenation or rewrite live data.

Final checks: archive characterization passed; project governance and Collaboration Claim validators
both passed with zero warnings. `git diff --check` passed. `EVOLUTION.md` records the mixed-writer
regression lesson. All original uncommitted changes remain; no staging or commit was performed.

## Continued validation: strict cold-tip initialization

The first writer repair used the compatibility whole-file reader. Replaced that with the existing
strict host transcript parser, extended to return the exact last decoded ID. TLOG content that
looks like a record header inside a multiline payload cannot become a parent. Malformed trailing
records fail before append and preserve source bytes; deferred first writes still work when the
file has not been created. The initial stricter version broke two deferred-session tests; the
NotFound exception is now restricted to unpersisted sessions and both tests pass.

`cargo test --locked -p talos-session -j 1`: 318 unit tests and all integration tests passed.
The mixed production writer test now includes a real provider terminal diagnostic in addition to
events, messages, atomic turn commit, restart, and registered history discover/search.

CLI integration must establish a host-owned recovery source for the existing selected Session,
with the same permission/projection and freshness gates. Do not create a second empty embedded
session or pretend a CLI path is an existing durable binding: that would change identity and
still omit the user's history. Existing durable binding code reconstructs paths under `durable/`,
so simply calling `open_or_create` from CLI is not a valid repair. CLI-specific binding/recovery
ownership and archive provenance remain implementation work in this owner.

## CLI history integration implementation (local, not deployed)

Implemented selected-session binding and Actor wiring per proposed ADR-103. The existing
`set_persistence` path now prepares history using the same selected TLOG and caches its recovery
owner/progress; embedded persistence still uses its original owner. No empty replacement session
is created. Identity mismatch and symlink transcript/database bindings fail before indexing.

A real ordinary-persistence Actor test initially still failed discover: CLI appends provider
terminal diagnostics between model responses, invalidating the original seal's file version.
History tool execution now verifies the original captured prefix digest within two seconds
before renewing its seal. It never extends cutoff or query scope. Mutation still fails, and
post-query freshness and permission checks remain mandatory. Original credential lines in
ordinary CLI logs are now redacted by the search projection without rewriting originals.

Validation so far: talos-session 320 unit tests plus integrations passed; checkpoint suite
87 passed. The CLI Actor test passed discover/search/forged-reference refusal/detail/provider
continuation. A cold-run repeat alongside the embedded Actor test is being checked. These are
real production Actor/provider/tool paths with test model responses, not deployed-host evidence.

Remaining: archive provenance and legacy-parent recovery, context-budgeted startup recovery,
CLI TUI/inline startup acceptance, compact Applied under recovered history, full workspace
preflight, independent review, governance closeout, and deployment. Earlier owner text saying
CLI binding is entirely unimplemented is superseded by this section; archive recovery remains
unimplemented and the task remains Partial. Original dirty-tree changes and session data remain.

Cold validation results: both CLI and embedded Actor/provider recovery tests passed without
prewarming history preparation. `cargo check --locked -p talos-cli -j 1` passed. Project governance
validator passed with zero warnings. README now distinguishes supported selected-TLOG text
recovery from unsupported unverified tool results and unrecovered archives. Agent full-library
validation is running separately; no workspace or deployment success is inferred from these checks.

Final local result for this CLI slice: `cargo test --locked -p talos-agent --lib -j 1`
passed 694 tests with zero failures/ignored. Existing full-goal residuals above remain open;
no commit, merge, release or deployment was performed.

## Rust archive evidence reconstruction and CLI inspection

Implemented `Session::inspect_archive_recovery` and the runnable
`talos storage inspect-recovery --transcript /path/to/UUID.tlog` diagnostic command.
The command acquires the session execution lease and reports `EvidenceOnly`, not Applied or
installed. Permanent runtime lock anchors may be created; transcript/archive/index/pending bytes
are not rewritten. It accepts an isolated evidence copy and refuses a concurrently owned session.

The inspector reads regular, non-symlink sources with 256 directory-entry, 64 MiB aggregate
stored/decoded byte and 100000 decoded-entry limits, cancellation/deadline checks, and the existing
2 MiB single-record decoder bound. These are input/work bounds, not a hard RSS limit: records,
exact source bytes and overlap maps consume additional bounded memory. Native zstd decoding is
inside an unwind/error boundary and checked between output chunks. Unsupported compressed builds
fail explicitly. Source versions are checked again after reconstruction, including excluded heads.

Selection starts from the host-selected UUID head and expands only across identical original
record IDs and bytes. Conflicts fail. Record adjacency must establish a unique acyclic order;
missing/forward parents and overlap with another live UUID head fail. Exact legacy count-only
placeholders are omitted, not called semantic summaries. Every original ID and parent remains
unchanged, including the known stale-parent defect. Files without overlap remain excluded.
This is stronger reconstruction evidence but still not authorization to bypass linear index
validation, manufacture recovery receipts, or install a model checkpoint.

Production-path regressions: real writes → compact(50) → more writes → compact(50) → reopen;
a tool call immediately before the retained result boundary; unrelated same-directory session
and archive; original stale parents; conflicting stable-ID content; divergent branch adjacency;
shared live head; malformed archive; missing parent; cancellation/deadline; read budget; symlink
archive; CLI report and active-lease refusal. Sources remain byte-identical after inspection.

Real isolated-copy command (debug build, not deployed host):
`cargo run --locked -p talos-cli -j 1 -- storage inspect-recovery --transcript /tmp/talos-recovery-evidence-20261111-170639/original/4404450a-443c-479d-8153-f530d10faa21.tlog`.
Report `/tmp/talos-archive-inspection-real.json`: 4288 original records, 25 omitted placeholders,
7 excluded archives, 4267 non-adjacent original parents, and 26 selected source hashes. This agrees
with the independent earlier byte parser. Original live files were not opened for mutation.

Remaining integration: the recovered evidence corpus is not yet a model/query source. It needs
host-owned provenance tied to the selected head and archive versions, bounded startup restoration,
permission-projected query results, and live-source revalidation without pretending original
parents form a linear chain. Do not change `Preparation` to blanket-accept earlier-parent DAGs.
Whole-task validation, review, deployment and the other handoff backlog remain open.

Archive inspector validation results: session suite 327 unit tests plus all integrations passed;
CLI inspection/active-lease refusal regression passed; archive tests without default zstd feature
passed. Project governance and Collaboration Claim validators passed with zero warnings;
`git diff --check` passed. All 26 selected source SHA-256 values from the Rust CLI report match the
original evidence manifest. This closes the inspector implementation/verification slice only;
archive-backed model restoration and query installation are still unimplemented.

## Archive-backed startup and query integration (local candidate)

Implemented normal paged-scan detection of the exact legacy placeholder, followed by version-
checked archive reconstruction into original host messages. Startup acquires the execution lease;
transition snapshot recovery uses its existing fence. Tool pairs use one decoder across the
reconstructed stream. Missing archives fail startup instead of silently restoring the recent tail.
Source versions are rechecked before publishing messages; original bytes/IDs/parents are retained.

Added a distinct host-owned archive query source after strict linear preparation fails without
resumable progress. It requires actual overlapping archives and legacy placeholders, uses the
validated unique original-record order, seals the selected head, and checks all supporting file
versions. It retains permission, scope revocation, opaque reference/cursor and response-size gates.
No linear parent check was relaxed. SQLite records owner scope and completed current-head scanning;
archive results come from the authenticated in-memory corpus, not invented offsets. User/assistant
projections are searchable; old tool results without receipts remain unavailable. Matching is a
bounded case-insensitive substring scan, not the ordinary-head FTS ranking path. See ADR-103.

Verified so far: seven CLI history restoration tests passed; archived Actor/provider
search/detail/forged-reference refusal passed; archive pagination, cursor replay/query mismatch,
legacy-parent preservation, append cutoff, scope revocation and changed-archive refusal passed.
The archive Actor regression now also submits a source-backed checkpoint, observes its actual
Context projection in the next provider request, and asserts ModelCheckpointTerminal Applied
with after < before. It passed. This is production Actor/tool flow with a test model, not proof
that the user's running host was upgraded or that a real provider generated an adequate summary.

Budgeted-startup regression and expanded full-library validation are next. Independent security
review, full preflight, running-host/deployment evidence, remaining handoff backlog and final
closure documentation remain open. Prior statements that archive startup/query integration is
entirely unimplemented are superseded by this section; no Complete claim or completion SHA exists.

## Follow-up validation and remaining restoration boundaries

The budgeted-startup test passed: 205 original records survive a real compact(50);
capacity 64 emits SubmissionContextBudget before a provider request, while capacity
128000 dispatches the oldest original fact. This proves bounded dispatch, not an
automatic recovery-summary workflow for histories that exceed the selected model.
Such over-budget continuation remains unresolved; a pause must not be called repair.

All nine CLI history_restore tests passed, including archive restoration, cancellation,
missing-source refusal and staged fork cleanup. Archived forks now explicitly fail before
publication, preserving the source; full child archive provenance support remains open.
Copying only the current head would lose original context and could make the original's
archive evidence ambiguous through shared IDs, so refusal is containment only.

Whole-repository preflight reached workspace tests after governance, checks and Clippy
passed. It exposed a first-append compatibility regression in the existing model-switch
round-trip test: Session::new does not create its file, despite persisted=true. The cold
parent reader now permits a missing source on first append, matching the public constructor
contract; malformed existing sources still fail. Exact-head verification is rerunning.

A further code audit found the optional Actor set_session_paths route archived every
nonempty head with keep_recent=0 before starting turns. Current CLI has no caller, but
external embedding can call this public API. Removed this automatic trimming too; the
API remains a documented compatibility no-op and callers configure persistence through
set_persistence. Existing real Actor/history/provider tests now configure those legacy
paths and assert that original head bytes remain a prefix after successful continuation.
This is an additional prevention fix, not restoration of already lost provenance.

Preflight's dormant background enable entrypoint is now test-only, with its production
activation still absent. Collapsible-if fixes preserve existing behavior. No background
automatic continuation was enabled. README describes local candidate restoration, query,
budget and archived-fork limitations. No commit, release or live-session rewrite occurred.


## Remote rebase requested by user

Pre-rebase complete `./scripts/release_preflight.sh` passed, including locked workspace
tests and external SDK fixtures; log `/tmp/talos-recovery-integrated-preflight.log`.
This evidence belongs to the pre-rebase tree only.

Fetched origin and rebased feat/bg-default-continuation from c065236a onto
55617ad7 (86 upstream commits, no local commits to replay). Preserved all 40 dirty
paths in `/tmp/talos-before-rebase-20261111-181823` with patch, tar and SHA manifest,
and in retained stash b3c0e442e114bce0862378ee1a99a2e5080774da. Restored untracked files
and the original empty index. Resolved two text conflicts: retained both EVOLUTION
entries; combined upstream private-replay path invalidation with disabled automatic
archival in Actor. The earlier compatibility no-op description is superseded: path
tracking and replay invalidation remain, destructive archival does not. No live session
data changed. Post-rebase checks/tests must pass before using the new candidate.

Post-rebase integration corrected private_replays argument forwarding through the local
runtime-context preparation helper, preserving upstream replay isolation. Locked workspace
check and Clippy passed; Agent 705, CLI 427 and TUI 654 unit tests passed, alongside the
recovery regressions. Full preflight failed at Agent doctests because the linked session
rlib lacked ArchiveRecoveryEvidence although current source exports it; that artifact's
.d dependency list omitted archive_recovery.rs. Existing shared-target output is not valid
whole-preflight evidence. Restarted the standard preflight with an isolated build target
`target/session-recovery-verify`, log `/tmp/talos-post-rebase-isolated-preflight.log`.
No existing build cache was deleted and no other task was terminated.


## Oversized archive restoration and source-inventory regression

Source audit corrected the earlier conclusion that over-budget restoration only pauses:
request admission already invokes source-bound checkpoint folding for Critical/Emergency
pressure. Added a real regression: write 205 originals, compact(50), reopen, restore,
submit at 20000-token capacity, produce multiple source-bound summaries, observe automatic
Applied, and dispatch continuation containing Context plus recent originals. Original head
bytes remain a prefix; another restart recovers all 207 original user/assistant messages.
Test passed in `/tmp/talos-archive-fold-test.log`. This validates actual Actor integration
with a test summary provider, not real-provider semantic quality. The 64-token refusal test
continues to cover irreducible capacity. No new summarization algorithm was added.

Security follow-up: prepared archive evidence previously revalidated captured paths only.
A newly created overlapping UUID head could therefore evade the ambiguity recheck. Source
validation now enumerates bounded matching archive/head paths and requires exact inventory
equality before publishing restored messages or query results. Ordinary sidecars are not
classified as transcript sources. Regression creates a real archive, prepares evidence,
adds a sidecar (still valid), then copies an overlapping live head and verifies all-source,
supporting-source and restoration refusal. Independent review remains required.

Final candidate verification after the inventory fix and archived-fold regression:
`CARGO_TARGET_DIR=target/session-recovery-verify ./scripts/release_preflight.sh` exited 0;
log `/tmp/talos-recovery-final-preflight.log`. This includes pinned/locked workspace
check, Clippy, all workspace tests/doctests and both external Runtime SDK fixture runs.
Session unit suite: 329 passed. The archived restart folding and newly-added-source
revocation tests are present and passing in that full-run log. Both governance validators
also passed separately with zero warnings after README/Board synchronization; logs
`/tmp/talos-recovery-governance-final.log` and `/tmp/talos-recovery-claims-final.log`.
`git diff --check` passed. This is local candidate evidence only: no commit, push, independent
review, deployment or Complete claim. Next: independent boundary review and running-host
acceptance, archive-fork provenance residual, then remaining handoff background/UI scope.

## Native candidate inspection on preserved data

Ran `target/session-recovery-verify/debug/talos storage inspect-recovery --transcript`
against the preserved isolated head under `/tmp/talos-recovery-evidence-20261111-170639/original/`.
The actual CLI exited 0 and reported EvidenceOnly: 4288 original records, 25 omitted
count placeholders, 7 excluded archives, 4267 non-adjacent original parents, 26 supporting
sources and installed=false. Report: `/tmp/talos-handoff-native-recovery-report.json`.
Verified SHA-256 of all 36 preserved original files against the pre-existing manifest
after execution: zero mismatches. The command's lock anchor is permitted isolated
coordination metadata; original transcript/archive bytes are unchanged.
This strengthens native CLI evidence, but is not installed-host continuation, a live-provider
semantic acceptance test, or independent provenance review. No live session was rewritten.

## Compressed decoder window boundary

Further local review found that stored/decoded byte limits do not constrain the zstd
frame's native back-reference window. A real valid nine-byte frame declares a 128 MiB
window but decodes to zero bytes; the new regression failed before the fix because
archive inspection accepted it. Set Decoder::window_log_max to MAX_BYTES.ilog2()
(64 MiB), separately from decoded output accounting. API semantics verified against
pinned zstd 0.14.0 stream/mod.rs. Decoder setup/read remains inside the existing panic
boundary and failures return an error without source changes. This is not a claim of
64 MiB total process RSS: input, output, records and native overhead are separate.

After correction, all 9 archive recovery tests passed, including the oversized-window
fixture and unchanged-source assertions. Logs `/tmp/talos-archive-window-before.log`
(expected failing regression) and `/tmp/talos-archive-window-after.log` (pass).
Standard full preflight is being rerun, log `/tmp/talos-archive-window-preflight.log`.

Before this final window change, `cargo xwin check --locked -p talos-session -p talos-agent
-p talos-cli -p talos-tools -p talos-tui --all-features --tests --target
x86_64-pc-windows-msvc` exited 0 using pinned Rust 1.99.0. One existing test-only
History::prepare dead-code warning was emitted; no warning suppression added. Log:
`/tmp/talos-recovery-current-xwin.log`. Same check rerunning after window fix in
`/tmp/talos-recovery-window-xwin.log`. Cross-compilation is not native Windows acceptance
or independent security review. No live-session data, index or archive was modified.

Final window-fix candidate: standard preflight exited 0 in
`/tmp/talos-archive-window-preflight.log`, including all workspace tests/doctests and both
SDK fixtures. Post-fix all-feature MSVC test compilation also exited 0 in
`/tmp/talos-recovery-window-xwin.log` with the same disclosed test-only warning. Current
worktree file hashes are captured in `/tmp/talos-recovery-review-candidate-manifest.json`
for review correlation (not a Completion Commit or independent review). Live-host/native
Windows execution, independent review and archive-fork provenance remain outstanding.

## Binding database open-time symlink regression

A deterministic real SQLite regression replaces the regular binding path with a
symlink after the caller's metadata check. Before correction, `open_bindings`
accepted the link (`/tmp/talos-bindings-nofollow-before.log`, expected failure).
The shared connection opener now adds SQLite `SQLITE_OPEN_NOFOLLOW`, including
bounded scan/fork reopen paths, without adding schema initialization/retries to
those bounded paths. Semantics checked against the bundled SQLite API description
and rusqlite OpenFlags. The regression asserts rejection and unchanged target
bytes/no target WAL or SHM, exercising both schema and connection-only openers.
This covers SQLite's symlink rejection, not arbitrary directory replacement or
native Windows runtime behavior. Initial Session suite passed 331 tests in
`/tmp/talos-bindings-nofollow-after.log`; final shared-opener validation follows.
No live session files changed; independent security review remains outstanding.

Final shared-opener Session suite: 331 passed, zero failures, in
`/tmp/talos-bindings-nofollow-final-session.log`. A fresh full preflight for the
final source is running in `/tmp/talos-bindings-nofollow-final-preflight.log`;
the earlier preflight overlapped the reopen-path extension and cannot certify
the final tree. No completion/commit/deployment claim is made.

Final shared-opener full preflight exited 0 in
`/tmp/talos-bindings-nofollow-final-preflight.log`, including both external SDK fixtures.
The previous overlapping run also passed but is not used as final-source evidence.
Storage-only changes have been copied, without moving/removing any root changes, into
`/tmp/talos-session-recovery-storage-20261111`, branch
`fix/session-recovery-storage-20261111` at base 55617ad7. Its 331 Session tests pass;
its independent full-workspace preflight is running in
`/tmp/talos-storage-isolated-preflight.log`. Candidate owner is
`docs/tasks/session-recovery-storage-candidate.md` in that worktree. No storage-only
candidate is represented as full CLI/Actor recovery or handoff completion.

Compatibility follow-up: SQLite NOFOLLOW rejects ancestor symlinks too. A real
parent-directory-alias regression failed (`/tmp/talos-bindings-parent-alias-before.log`).
The connection helper now canonicalizes only the parent directory, never the final
DB name, before opening with NOFOLLOW. This supports host aliases such as macOS /tmp
while retaining final-database-link refusal; it does not claim directory-handle confinement.
Session suite now passes 332 tests (`/tmp/talos-bindings-parent-alias-after.log`).
The prior 331-test full preflight does not cover this final adjustment.

Storage-only candidate full preflight failed at the existing Actor checkpoint process
restart acceptance test: child turn deadline exceeded after 20 seconds. Log
`/tmp/talos-storage-isolated-preflight.log`. Focused retry is running in
`/tmp/talos-storage-isolated-checkpoint-retry.log`; cause is not yet established.
No failed gate was waived and no candidate commit was made.
Focused checkpoint process-restart retry passed (0.32 seconds). This does not establish
the earlier timeout's cause. Final isolated candidate preflight is running in
`/tmp/talos-storage-isolated-final-preflight.log`; next resume should poll session 23336
before considering another run. Original worktree remains dirty and preserved.

## Isolated integration candidate

Storage prerequisite is now local commit `1f5602eb` on
`fix/session-recovery-storage-20261111`; it is not pushed, merged or a full recovery
completion commit. Integration continues from it in
`/tmp/talos-session-recovery-integration-20261111`, branch
`fix/session-recovery-integration-20261111`. Only recovery/checkpoint code and its
README/owner/ADR are extracted; background, approval and unrelated UI changes remain
in the original preserved worktree. Three actual Actor/provider history regressions
passed in `/tmp/talos-recovery-isolated-actor-tests.log`, including archived search,
forged-reference refusal and Applied. CLI compile passed in
`/tmp/talos-recovery-isolated-integration-check.log`. CLI restoration suite is running
in `/tmp/talos-recovery-isolated-cli-tests.log`. Integration full preflight and independent
review remain pending. The inherited owner history above describes the primary mixed
candidate, not proof that every validation has run on this isolated branch.
Isolated CLI history_restore suite passed all 10 tests, including real archived restart
and source-bound multi-summary folding. Both governance validators pass with zero
warnings; logs `/tmp/talos-recovery-isolated-cli-tests.log`,
`/tmp/talos-recovery-isolated-governance.log`, `/tmp/talos-recovery-isolated-claims.log`.
Full integration preflight remains pending; run after storage validation completes to
avoid unnecessary shared-target build contention.


## Startup fork coverage correction

Source review found that startup `--fork` bypassed the interactive staged-history
archive guard. Real regression `startup_fork_refuses_archive_without_publishing_overlapping_child`
failed before the fix because the startup path published a recent-head-only child;
log `/tmp/talos-startup-fork-before-fixed-test.log`. It now scans with the existing
bounded, version-checked decoder and refuses count-placeholder history before child
creation. It retains the normal tail identity for ordinary forks. Both startup fork
tests pass in `/tmp/talos-startup-fork-after.log`, including ordinary copied bytes,
runtime identity and index, and archive refusal with unchanged source and all 205
originals still recoverable. Dedicated integration repeat is running in
`/tmp/talos-isolated-startup-fork-after.log`. This closes an unsafe containment gap;
full child-owned archive provenance remains unimplemented.

The earlier dedicated build handles 23241 and 3428 and regression handle 53184 are
now absent; their logs end during compilation without success evidence. Storage
preflight resumed with CARGO_BUILD_JOBS=2 as process 30102, log
`/tmp/talos-storage-dedicated-resumed-preflight.log`. Dedicated integration fork tests
run as process 91696. Do not reuse the cross-worktree shared Cargo target for final
validation, and do not count interrupted logs as passing results.


## Native isolated integration acceptance

Ran the actual binary built from the isolated integration candidate:
`target/recovery-integration-candidate/debug/talos storage inspect-recovery --transcript`
against the preserved isolated head. Exit 0; report
`/tmp/talos-isolated-integration-native-report.json`: EvidenceOnly, 4288 originals,
25 omitted count placeholders, 7 excluded archives, 4267 nonadjacent original parents,
installed=false. All 36 original-file SHA256 values still match the preservation
manifest. This validates the isolated CLI inspection path, not deployed continuation,
live-provider summary quality or complete fork support. Live session files are untouched.

The isolated provider deadline retry passed (1 test, 2.69 seconds) in
`/tmp/talos-storage-provider-deadline-retry.log`; it does not prove the cause of the
full-run failure. Integration preflight remains live as process 80429 in
`/tmp/talos-integration-dedicated-resumed-preflight.log`. No final gate waived.


## Unavailable diagnostic coverage

All remaining checkpoint_tools generic execution-unavailable returns now emit fixed
content-free reason literals: execution-state poisoning, issued/current snapshot absence,
permission lease/commit rejection, delivery source/snapshot unavailability, delivery
cancellation/deadline, query-state reset failure, missing execution scope and closed
Actor channel. Model-visible errors and authorization behavior are unchanged. This
covers the generic tool return points, not every internal `.ok()?` storage/preparation
failure; precise underlying preparation errors remain a diagnostic residual.
Existing checkpoint tool safety suite passes 13 tests in
`/tmp/talos-checkpoint-reasons-tests.log`, including post-query mutation/reset/cancel.
The running integration preflight began before this logging-only addition and cannot
certify the final changed tree; finish collecting it, then validate the final candidate.


## Archive preparation diagnostic follow-up

Requested outcome: distinguish archive-history preparation failures without exposing
source data or weakening provenance gates. Scope: `checkpoint_history.rs` and this
owner; original dirty worktree, incident sources and retained backups remain preserved.
The internal archived preparation now returns fixed stage codes for evidence, head
capture/sealing, source freshness and index operations. Cancellation/deadline and
blocking-worker failure are distinguishable. Dependency error strings, paths, session
identities and transcript text are never included in those diagnostic fields.

Two real-I/O regressions distinguish cancellation, expired deadline, corrupt disposable
archive evidence and an unopenable index; they verify unchanged source bytes and that
failed evidence does not initialize an index. All 20 checkpoint_history tests pass in
`/tmp/talos-archive-preparation-diagnostics-tests.log`. Standard pinned/locked preflight
passed (exit 0) in `/tmp/talos-archive-preparation-diagnostics-preflight.log`, including
workspace check, Clippy, tests/doctests and both external SDK fixtures. Tests used the
same serialized, credential-env-cleared configuration as the prior candidate. Both
governance validators pass with zero warnings. No new source edits followed validation.

Remote CI run 43041022143 validates the previous ae27ee2f head, not this follow-up.
Its Linux Desktop job 130592601092 was cancelled at the 30-minute job limit during
native dependency installation: the log records repeated Ubuntu mirror DNS failures.
Desktop regressions did not run; this is not a passing gate. Evidence:
`/tmp/talos-ci-linux-desktop-130592601092.log`. Other jobs were still running at the
last check. A new source head requires fresh exact-head CI and independent review.

Status remains Partial. This closes the archived preparation stage-code gap only;
ordinary linear preparation still has combined failure paths, and evidence rejection
codes do not distinguish every underlying archive parser/provenance cause. Full archived
fork provenance, independent safety/API review, native host acceptance and the remaining
handoff work stay owned here and in the primary handoff. Review authorization has been
requested; no unapproved reviewer was started and no merge/deployment occurred.


## Linear preparation diagnostic follow-up

The remaining linear preparation aborts now emit fixed stage codes for source
capture/ownership, seal scanning/revalidation, index creation/cursors/page advancement,
persisted scans/continuations and pending resolution. The async wait boundary identifies
cancellation, deadline, retained partial work and lock contention. Optional cached
continuation misses still follow the original fresh-source path; these are not reported
as terminal preparation failures. Retry eligibility, source/permission checks, cutoffs
and model-visible errors are unchanged. Dependency errors are deliberately not formatted.

A real-I/O regression captures tracing events for malformed source bytes, an unopenable
index, cancellation and deadline expiry, asserts their reason codes, and rejects source
text, paths and session IDs in all captured fields. It also verifies unchanged transcript
bytes. On baseline 38c3d6f8 plus the new test, it failed because the actual decoder refusal
emitted no reason (`/tmp/talos-linear-preparation-diagnostics-before.log`, exit 101).
The fixed source passed all 21 history preparation tests, log
`/tmp/talos-linear-preparation-diagnostics-tests.log`; source was restored byte-for-byte
after the baseline probe. Final combined pinned/locked preflight passed (exit 0), log
`/tmp/talos-linear-and-fork-final-preflight.log`, including workspace checks, Clippy,
tests/doctests and both SDK fixtures. No product source edits followed this validation.

Remote run 43041022143 completed: Linux main and native Windows workspace checks passed;
overall conclusion is cancelled because Linux Desktop dependency installation timed out
with mirror DNS failures. Run 43046879069 for 38c3d6f8 is now running. Neither run proves
this follow-up, full archived fork support or running-host acceptance.
Status stays Partial; independent review and the remaining handoff deliverables are open.


The first full preflight for linear diagnostics exited 101 in unchanged
`i255_plugin_e2e`: the tool result reported its existing 250ms WASM execution timeout,
so the success assertion failed. Log `/tmp/talos-linear-preparation-diagnostics-preflight.log`.
An isolated rerun passed both plugin tests in 1.54 seconds,
`/tmp/talos-linear-plugin-deadline-retry.log`; the original timeout cause remains
unestablished. No deadline, policy or assertion was relaxed. The final combined
preflight cited above subsequently passed; the earlier failure remains recorded.

## Shared fork boundary correction

Further entrypoint review found that the REPL `/fork` worker calls `Session::fork`
directly, bypassing both startup and TUI staging checks. A real EventLoop input/worker
regression failed before the fix by publishing a partial archived child; log
`/tmp/talos-repl-archive-fork-before.log`. The shared Session API now refuses exact
legacy count placeholders before mutating branch state. This also covers direct API
callers. The existing EntryNotFound behavior remains for unknown fork points.

Core coverage verifies no changed head/branches on refusal and then successfully
appends to the original session, recovering all 206 originals. REPL coverage uses both
implicit and explicit recent fork points and verifies no child session/index relation
and all 205 originals recoverable. All 25 Session fork tests and 10 CLI fork tests pass
in `/tmp/talos-core-archive-fork-after.log` and `/tmp/talos-repl-archive-fork-after.log`.
The separate durable committed-tip API already rejects count placeholders through its
mandatory turn-identity/parent validation before publication; it is not weakened.

This closes another unsafe entrypoint only. Full archived-fork provenance remains
unimplemented; do not turn a shared refusal into a full-fork completion claim. Final
combined preflight passed (exit 0), log `/tmp/talos-linear-and-fork-final-preflight.log`.
The formerly failing plugin fixture passed within that full run, without deadline or
assertion changes. Independent review remains required. Original working tree/session
files and background continuation defaults are unchanged.

Final-source Session/Agent/CLI all-feature MSVC test compilation passed via pinned
`cargo xwin check --locked --all-features --tests --target x86_64-pc-windows-msvc`,
log `/tmp/talos-linear-and-fork-final-xwin.log`. The existing test-only History::prepare
dead-code warning remains; no suppression was added. This is not native Windows
execution or independent review.


## Parallel validation follow-up — 2026-11-12

CI main validation runs on macos-15, not Linux; earlier references to "Linux main"
were inaccurate. Run 43046879069 failed two source-binding assertions in the Agent
suite: cooperative committed-source availability and registered model checkpoint
initial recovered-source binding. Both paths use bounded deadlines. Logs are retained
in `/tmp/talos-ci-130620743862.log`. Deadline pressure is a hypothesis, not an established
cause; no production/test deadline was increased. Current CI 43053922103 remains live.
Its separate remote reconciliation failure is newly opened unrelated Issue #919 absent
from the global matrix (`/tmp/talos-ci-130638511340.log`); this is separate governance
drift under ADR-071, not evidence of a recovery defect or a waived CI gate.

A local default-parallel Agent run found a different regression in the new log-capture
test: no events captured, with 681 other tests passing. Explicit dynamic tracing
callsite interest did not fix it. Consulted tracing-core 0.1.36 `src/callsite.rs`, whose
documentation confirms process-global callsite registration/interest caching. The exact
parallel interaction is not established. Log-capture acceptance now runs the same real
I/O and disclosure assertions in an isolated child test process, consistent with existing
restart probes. All 682 Agent tests pass with default parallel scheduling in
`/tmp/talos-recovery-parallel-agent-isolated.log`; failed attempts remain in
`/tmp/talos-recovery-parallel-agent-repro.log` and
`/tmp/talos-recovery-parallel-agent-after.log`. This is a test-only correction; production
source remains the fully preflight-validated d54a2490 source. Native macOS failures remain
unresolved until exact-head remote evidence and, if reproduced, a causal fix are obtained.


## Discovery budget diagnostics and expiry regression — 2026-11-12

Added fixed, content-free diagnostic reasons for expiry during reference construction,
after scanning and after source metadata cloning in `refresh_cooperative`. The
two-second budget, yields, source validation and publication guard are unchanged.
A paused-clock regression polls the real refresh through multiple records, expires
the budget, verifies discover refuses unpublished/old authority, then verifies a
fresh refresh restores discover. This tests deadline behavior, not the cause of
the remote macOS failure. Parallel Agent suite: 683 passed, exit 0, log
`/tmp/talos-discovery-deadline-agent-tests.log`. Changes also applied as a checked
patch to the preserved primary worktree without replacing unrelated edits.
This follow-up remains local pending validation and review; task remains Partial.


## Contention reproduction and WAL initializer budget — 2026-11-12

The exact 8300-message discovery regression reproduced on Linux with 32 concurrent
instances pinned to one CPU: 1/32 failed at the same compactable-snapshot assertion;
1 and 8 instances passed. A second, temporarily instrumented run failed 2/32, with
refresh wall times 2.032s and 2.071s versus 58ms for one instance. Evidence:
`/tmp/talos-history-contention-20261112/summary.json` and `timed-summary.json`.
This establishes local sensitivity to contention, not macOS causality. Temporary
instrumentation was restored byte-for-byte; uninstrumented checkpoint tests pass.
Tokio 1.53.1 `task/yield_now.rs` documents scheduling non-guarantees; no scheduler
assumption, production budget increase or CI serialization was introduced.

Exact-head remote Windows job 130656602802 failed two tests: Actor checkpoint
acceptance timed out waiting 10s for a fork; concurrent durable opens returned
SQLite `locking protocol`. Log: `/tmp/talos-ci-windows-130656602802.log`.
SQLite documents SQLITE_PROTOCOL as exhausted WAL transaction lock races:
https://www.sqlite.org/rescode.html#protocol . Do not blindly retry it or claim
the two failures share a proven cause.

Inspection of pinned rusqlite 0.40.2 `inner_connection.rs` found the default 5000ms
busy handler. `open_bindings` claimed bounded initialization retries but did not
disable that handler. A real WAL writer-lock regression failed before the fix
(`/tmp/talos-wal-init-budget-before.log`): init remained blocked beyond two seconds.
The initializer now explicitly disables the driver busy handler, retaining the
20 x 25ms retry sleeps and restoring the existing five-second transaction handler
after initialization. Native calls/scheduling remain outside a hard wall-clock
guarantee. No new protocol-error retries and no live database changes.

Session tests passed: 334 unit plus 27 integration tests, exit 0,
`/tmp/talos-wal-init-budget-session-tests.log`; MSVC all-feature test compilation
passed, `/tmp/talos-wal-init-budget-xwin.log`. This fixes a reproduced initialization
budget defect; native Windows protocol/fork failures still require resolution.
Patch was checked and applied to the preserved primary worktree. Status Partial.

Validation follow-up: standard full `./scripts/release_preflight.sh` passed with
Rust 1.99.0 and locked dependencies; log `/tmp/talos-wal-init-final-preflight.log`.
This includes both local follow-ups (discovery diagnostics and WAL init budget),
workspace checks/Clippy/tests/doc-tests and SDK fixtures. Both primary governance
validators also passed. Remote Linux Desktop job 130656602835 failed installing
system dependencies due Ubuntu mirror DNS errors before running Desktop tests;
log `/tmp/talos-ci-linux-desktop-130656602835.log`. Native Windows failures and
independent review remain unresolved; no deployment or completion claim.


## 2026-11-12 Windows review findings: local implementation in progress

Independent reviewer `/root/independent_review` reviewed exact head
`39d2f3279888258c39901f4524c230f3b87ab105` and requested changes for two P1s:
legacy CLI provider diagnostic writes conflict with whole-turn Windows recovery
leases, and archive evidence retains write-excluding handles on unrelated live
heads. Previous green CI excluded the new legacy Actor tests on Windows and does
not disprove these findings. PR #916/#921 governance candidates had no blocking
findings; their claims remain ineffective until target merge.

The isolated candidate now releases Windows prepared-History leases between
operations, retains original source/cutoff/digest, and revalidates under bounded
fresh leases for each use. Same-owner diagnostics are coordinated with the shared
Session write mutex. Archive evidence retains inert identity plus raw-byte digests
and revalidates all inspected files/inventory under bounded fresh leases. ADR-104
and SDK contract document the additive read-lock API and limitations. No native
unsafe changes, source normalization, weaker metadata-only authority or default
background continuation were introduced.

Regression additions: existing real multi-response Actor history tests now compile
on Windows and assert persisted provider diagnostics, final reply and tool exchange
after reopen; a real paused-provider/two-session Actor regression verifies the other
session can commit and reopen while History is retained; storage regressions cover
selected/unrelated appends while evidence stays live, and Windows equal-length
content mutation despite matching metadata. Windows History tests verify idle
writes, restrictive active reads, original cutoff, digest mutation refusal and
cancellation. Linux archive recovery tests and all four focused Actor scenarios
passed. Latest MSVC all-feature test compilation exited 0 in
`/tmp/talos-review-scoped-leases-final-xwin.log` (existing dead-code warnings).
Full standard preflight currently running:
`/tmp/talos-review-scoped-leases-preflight.log`.

These are uncommitted local changes, not part of the previously tested PR head.
Windows native execution and fresh independent review remain required; do not mark
the findings resolved from cross-compilation. Patch backup:
`/tmp/talos-review-scoped-leases-20261112.patch`. Applying this patch to the preserved
primary tree failed its dry-run on checkpoint_history.rs because that tree contains
additional changes; nothing was overwritten or partially applied. Reconcile
selectively after candidate validation. Full archived fork, live host/provider
acceptance and the remaining handoff scope are still incomplete. Status Partial.


### Same-day follow-up review: sibling-write race remains open

The same authorized independent reviewer inspected the uncommitted correction.
P1-1's direct mechanism is corrected at code level: idle History owns no Windows
lease, and synchronous reads hold the selected owner's shared write mutex until
all restrictive handles are dropped. Native execution is still required.

P1-2 remains open: check_supporting_sources retains fresh restrictive handles for
all sibling heads during its bounded full-content check. A sibling that starts
append/commit during that interval can still fail with sharing violation. The
paused-provider Actor test covers idle lifetime only, not this interval. Next:
a deterministic barrier with a sibling lease held must race a real independently
opened writer; writer must wait and succeed or reader relinquish custody. Inspect
both Session append and DurableSession atomic replace paths. A per-owner mutex
alone cannot coordinate sibling/independently reopened owners or other processes.
No metadata-only fallback or digest relaxation is acceptable. Also propagate one
caller deadline through repeated current/supporting/prefix checks; currently an
outer expired deadline refuses delivery but does not stop nested fresh budgets.
No production-complete or fixed-finding claim is made for the uncommitted patch.


Resume checkpoint: goal remains Active. Source manifest for the in-flight standard
preflight is `/tmp/talos-review-scoped-leases-source-manifest-20261112.json`.
Execution handle `21349` was re-polled and is still live (currently workspace tool
tests); do not restart it because the observation window elapsed. Log remains
`/tmp/talos-review-scoped-leases-preflight.log`. No full-preflight result yet.
No commit, push, native Windows execution, merge or deployment was performed for
this local correction. The remaining sibling conflict needs an implementation,
not another status-only review. Evaluate narrow bounded Windows sharing-conflict
waits at append-open and final atomic-rename acquisition boundaries, without
retrying bytes already written; test both real append and durable commit under a
held read lease and a permanent-conflict deadline. This is an unimplemented option,
not an accepted design or completed repair. Existing primary reconcile artifacts
are in `/tmp/talos-scoped-leases-primary-reconcile-20261112/`: three file merges
are clean; checkpoint_history.rs has one unresolved constructor conflict, and none
have been copied over the primary dirty tree.


## 2026-11-12 sibling-writer coordination candidate

Implemented narrow Windows sharing-conflict waiting at built-in JSONL/TLOG append
open (including first TLOG header acquisition) and final atomic rename. Only
ERROR_SHARING_VIOLATION retries, with five seconds per acquisition/publication and
10 ms maximum sleeps. No byte writes, flushes, entire transactions or other errors
are replayed. This addresses the remaining overlap between sibling recovery reads
and business persistence without weakening source validation. ADR-104 records the
bounded-wait tradeoff. Tests synchronize on the actual sharing-error branch, then
release the real read lease; cover independently reopened append owners in both
formats, atomic durable commit, unchanged bytes before publication, reopen,
idempotent replay, permanent conflict bounds and no AccessDenied retries.

Also propagated one deadline through supporting-file, prefix and final freshness
checks within query/availability operations. Checkpoint focused suite: 92 passed,
`/tmp/talos-review-writer-waits-checkpoint.log`. Latest MSVC all-feature test check
exited 0: `/tmp/talos-review-writer-waits-final-xwin.log`. Previous source snapshot
preflight finished exit 0: `/tmp/talos-review-scoped-leases-preflight.log`; because
writer waits/deadline changes were added afterward, a new standard preflight is
running with handle 32909, log `/tmp/talos-review-writer-waits-preflight.log`.
Source manifest `/tmp/talos-review-writer-waits-source-manifest-20261112.json`;
patch `/tmp/talos-review-writer-waits-20261112.patch`.

Authorized independent reviewer examined the uncommitted update and found no new
confirmed blocking defect. The two P1 repair mechanisms are accepted at code level,
but the findings remain open pending native Windows evidence. In particular,
atomic-commit barrier must prove pinned Rust fs::rename returns the expected
sharing-conflict code and succeeds after release. Do not broaden retry codes to
AccessDenied without evidence. This is not final exact-commit review, native
acceptance, merged code or deployment. Primary tree reconciliation remains pending;
its checkpoint_history.rs is older than candidate diagnostics and must not receive
a blind overwrite. All original data and dirty changes remain preserved; default
background continuation is still off. Goal Active, owner Partial.


## 2026-11-12 delayed header acquisition and final validation attempt

Review found an additional preservation edge in the newly retried first-header
open: a stale empty-file observation plus truncate(true) could erase an owner that
initialized during the wait. TLOG header acquisition now uses non-truncating append
and checks the acquired handle length before writing its header. A Windows barrier
regression pauses on the actual sharing conflict, initializes through another
Session, resumes the delayed store append and verifies the complete old-byte prefix
plus both records. Independent read-only review found no new regression in this
correction. Its scope is content preservation; it does not prove parent correctness
or duplicate-header exclusion for arbitrary simultaneous independent writers.

Validation: latest Session suite 336 unit + 27 integration passed
(`/tmp/talos-review-header-wait-session.log`); MSVC all-feature test compilation
passed (`/tmp/talos-review-header-wait-xwin.log`). Writer-waits preflight handle
32909 exited 101 in existing i255_plugin_e2e: actual plugin result was execution
timed out after 250ms. This same symptom occurred earlier, root cause remains
unestablished. Isolated package rerun passed 2 tests in
`/tmp/talos-review-writer-waits-plugin-isolated.log`; its separate package build is
not an exact replacement for workspace feature-unified validation. No timeout,
permission rule or test was weakened to force a pass.

Final-source standard preflight now runs as handle 67345, log
`/tmp/talos-review-header-wait-final-preflight.log`; poll this handle before taking
any new validation action. Source manifest
`/tmp/talos-review-header-wait-source-manifest-20261112.json`; source patch
`/tmp/talos-review-header-wait-20261112.patch`. No final result yet. PR #917 was
requeried: OPEN Draft, head still 39d2f327, reviews empty. All latest corrections
remain local/uncommitted; no push/merge/deployment. Native Windows and final
exact-head review remain mandatory before closing the P1s. Primary dirty tree and
live session data remain preserved. Goal Active, broader handoff still Partial.


## 2026-11-12 final local candidate validation passed

Final-source standard preflight handle 67345 exited 0, including locked workspace
checks/Clippy/tests, doctests and SDK fixtures. Log:
`/tmp/talos-review-header-wait-final-preflight.log`. The earlier plugin 250ms
failure remains recorded; this successful run does not establish its root cause.
The isolated candidate's staged diff has been reviewed; primary dirty tree and
live session files are untouched. Windows native tests and final committed-head
independent review are the next gates. No merge, deployment, full archived-fork
completion or default background continuation change is claimed.


## 2026-11-12 candidate CI conflict and shared archive deadline

Windows lease/writer corrections were committed and pushed as
`ad6519cbfa1862c4573050c732a5a6566238a1a5`. Authorized independent read-only
review bound to that exact head found no new blocking code issue; native Windows
execution is still required to close both P1s, especially the atomic rename barrier.
GitHub reported PR #917 CONFLICTING and no checks for that head; earlier successful
run 43064415263 belongs to 39d2f327 and is not new-head evidence.

Merged origin/main f61a15cd into the isolated candidate as
`f48ef1d17fdbff2b1f02d07b14b583832a48a808`, preserving reviewed commits. The only
conflict was Board content: retained recovery entries and main's updated Azure
status. No primary rebase or source/data overwrite. Both governance validators
passed; independent review confirmed this merge introduces documentation only.

Follow-up implementation passes the caller deadline into archive search/detail,
removing the independent fresh scanning budget noted by review. Existing real
archive evidence regression now verifies expired-budget search and detail refusal;
23 focused checkpoint_history tests passed in
`/tmp/talos-recovery-shared-deadline-tests.log`. Independent review found no new
blocking issue in this uncommitted increment. Full standard preflight is running
as handle 98545, log `/tmp/talos-recovery-main-deadline-preflight.log`; no result
claimed yet. Native Windows, final committed-head review and live host/provider
acceptance remain pending. Full archived fork remains unimplemented; containment
is not completion. Default background continuation stays off. Status Partial.


Follow-up validation: standard preflight handle 98545 exited 0, including locked
workspace checks, Clippy, tests, doctests and external SDK fixture. Log:
`/tmp/talos-recovery-main-deadline-preflight.log`. Source stayed unchanged during
the run. This does not replace Windows native execution or host acceptance.


## 2026-11-12 native failure investigation (not resolved)

Exact-head dec7f5ec macOS job 130918904400 failed in the real Actor regression
registered_model_checkpoint_discovers_admits_and_applies_through_actor: provider
step expected an installed checkpoint but found none. Agent suite: 683 passed,
1 failed. Log `/tmp/talos-ci-macos-dec7f5ec.log`. The old assertion did not report
checkpoint terminal outcome, so timeout/cancellation/preparation/installation
cause is not established. Keep this as a recovery gate failure, not a CI-only flake.

Isolated candidate now has a test-only diagnostic enhancement: missing projection
prints checkpoint operation terminal metadata and provider step, never raw history.
Success assertions unchanged. Correctly selected exact test passed once (13.76s)
and all 684 Agent library tests passed in parallel (19.05s), logs
`/tmp/talos-recovery-dec7-checkpoint-repro.log` and
`/tmp/talos-recovery-dec7-agent-parallel.log`. An initial mistyped exact filter ran
zero tests and was corrected; zero-test execution is not evidence. Linux success
does not explain or resolve the macOS failure. Windows job remains in progress;
do not cancel/restart its run solely for this diagnostic increment.

Archived fork contract review draft is retained at
`/tmp/talos-archived-fork-contract-review-20261112.md`. Independent reviewer found
ordinary manifest+digest insufficient to authorize live-head overlap: a copied
prefix plus a self-consistent forged parent/cutoff manifest passes content checks
without a host fork publication. Full implementation must use host-issued private
custody receipt binding UUIDs/branch/cutoff/initial prefix/bundle digests, staged
intent then durable bytes then transactional publication, independent child bytes
and receipts surviving parent deletion, and exact inherited-prefix restrictions.
Existing durable_fork_lineage lacks content digests and is not sufficient as-is.
Nested forks require bounded materialized proof. Full copied parent sources may
contain post-cutoff records; those cannot enter the child's inherited corpus.
No overlap exception or incomplete fork implementation was introduced.


## 2026-11-12 Windows native evidence and unresolved atomic publication

Run43138294146 finished. Windows job130918904373 has two failures: Agent
actor_interrupt_and_shutdown_release_recovery_source_before_persistence still
asserted the superseded whole-turn write-excluding lease; Session atomic commit
wait regression did not reach ERROR_SHARING_VIOLATION observer (channel disconnected).
The latter hides its commit result, so actual OS error is not established. No
AccessDenied retry or other widening is authorized by this observation.

Native successes explicitly confirmed in `/tmp/talos-ci-windows-dec7f5ec.log`:
real CLI/archived Actor search/detail, paused-provider sibling write, Windows idle
lease/original cutoff verification, equal-metadata mutation refusal, independent
JSONL/TLOG append waits, bounded permanent sharing failure, and delayed header
preservation. The 8300-record Actor Applied test also passed on Windows; macOS
failure remains unexplained. Both P1s are not globally closed, atomic publication
still fails its required native gate.

Candidate corrections preserve cancellation/persistence assertions while changing
the stale Actor expectation to permit idle writes. Atomic waiter now prints its
actual writer Result if no sharing-conflict barrier occurs. Missing checkpoint
assertion prints terminal metadata only. These are test/diagnostic changes, not
an atomic-publication fix. MSVC all-feature tests check passed, log
`/tmp/talos-native-failure-diagnostics-xwin.log`. Primary source is unchanged during
its still-running preflight71089. Do not synchronize test edits until it ends.

## 2026-11-12 native diagnostic results and local publication correction

Exact-head CI43141073000 (32e4d227) completed with failures. Windows atomic commit
returned `IoError` OS5 PermissionDenied; logs `/tmp/talos-ci-windows-32e4d227.log`
and `-clean.log`. It was not an observed sharing conflict. macOS Actor checkpoint
Applied regression passed this run, but prior missing-checkpoint failure remains
unexplained. New macOS failure is the WAL schema initialization budget regression
at durable.rs:4021; log `/tmp/talos-ci-macos-32e4d227.log`. No rerun is treated as
root-cause closure. Unrelated issue reconciliation remains separately owned.

Local candidate now coordinates Windows atomic publication at nontruncating,
no-follow target write acquisition, with bounded sharing-only retry, retained
write handle through rename, and conservative file-write ACL requirement documented
in ADR104/SDK. A new native probe checks direct rename under lease, write acquisition,
exclusion of new recovery reads, publication and readonly refusal. Existing real
durable commit/reopen/idempotency regression remains unchanged. Native execution
of this correction is pending; MSVC all-features tests compilation passed at
`/tmp/talos-atomic-acquisition-xwin.log`.

Independent review also identified an end-of-validation pathname gap. A real Unix
archive regression failed before the fix (one executed failure, log
`/tmp/talos-archive-replacement-red.log`), then all12 archive tests passed after
final inventory/path-version rechecks (`/tmp/talos-archive-replacement-green.log`).
An earlier unqualified --exact invocation selected no tests and is not evidence.
Read-only review found no new blocker in this archive increment, with the explicit
limit that this is not a multi-path atomic snapshot. Primary mixed tree has not yet
received these new source changes. Full preflight, exact-head review/native CI,
archived fork and running-host acceptance remain pending; status remains Partial.

Read-only independent review of the full publication/path-binding increment found
no new blocking code issue. SDK compatibility paragraph relocated to selected-history
section per review. The readonly probe covers the Windows readonly attribute, not
an exhaustive DACL permission matrix. Full preflight is still running; no completion
or native acceptance claim is made.

Full standard locked preflight completed exit0 after the source changes, including
workspace tests, Clippy and independent SDK fixture:
`/tmp/talos-publication-path-binding-preflight.log`. This is local Linux evidence;
new Windows publication probes have only compiled so far. No source was edited
during that preflight. The SDK paragraph was relocated and the owner appended
review evidence while validation ran; governance checks are repeated before staging.

## 2026-11-12 SQLite initialization retry deadline regression

A new real-WAL-lock regression injects600ms scheduler delay after the first actual
busy response. Existing code then consumed20 retry waits; the single executed test
failed at expected1 vs actual20 (`/tmp/talos-binding-budget-red.log`). The local
uncommitted correction uses one500ms retry deadline plus the original retry-count
ceiling, checking before and after sleeps. Ten binding-focused tests passed
(`/tmp/talos-binding-budget-green.log`), including the unchanged native wall-time
regression and successful initialization after release. Read-only independent review
found no blocker, but correctly limits the contract: this is a retry deadline,
not a hard real-time bound on synchronous SQLite/open/scheduler latency. The macOS
historical two-second failure's full cause remains unproven. Full preflight56987
is running at `/tmp/talos-binding-budget-preflight.log`; no source edits during it.
CI43145317842 for previous candidatec1630445 remains live and is not interrupted.

Full locked standard preflight56987 completed exit0, including SDK fixture:
`/tmp/talos-binding-budget-preflight.log`. Afterwards only the explanatory source
comment was corrected to say retry deadline rather than total-call bound. Runtime
logic/tests match the validated source. Native c1630445 CI43145317842 still runs;
this follow-up will not be pushed while doing so. No claim of full macOS root cause.

## 2026-11-12 repeated macOS WAL timing failure diagnostics

Native c1630445 macOS job130941707853 failed again in the unchanged two-second
WAL initialization assertion. Log `/tmp/talos-ci-macos-c1630445.log`;336 session
tests passed,1 failed. Its head predatesa5c30d08's retry-deadline correction.
Windows job130941708290 still runs; do not cancel it by pushing prematurely.

Added test-only failure diagnostics for queue delay, worker initialization elapsed,
actual retry timestamps and success boolean. Original timing/result assertions remain;
no error body/path/history is printed. This measures worker duration (including tiny
observer setup/cleanup), not an isolated SQLite call. Ten binding tests and MSVC
all-feature test compilation passed (`/tmp/talos-binding-timing-diagnostics.log`,
`/tmp/talos-binding-timing-xwin.log`). Independent read-only review found no blocker.
Production logic is unchanged from full-preflight-validateda5c30d08.

## 2026-11-12 archived-fork raw preparation implementation in progress

Local uncommitted archive_recovery increment retains unique-order original record
bytes by moving them from the already-validated raw overlap map. New host-only
original_record_prefix copies through an inclusive entry cutoff under the caller's
original cancellation/deadline/byte bound and source rechecks before/after copying.
It creates no file/receipt and grants no inherited overlap or disclosure permission.
Existing fork guards remain. This is the first preparation primitive for the reviewed
materialized-child contract, not full fork support.

Real205-write/compact50/append regression preserves noncanonical valid metadata bytes
exactly, checks archived cutoff and tip, and rejects absent IDs, cancellation, expiry
and stale source.13 archive tests passed (`/tmp/talos-fork-raw-prefix-tests.log`);
MSVC all-feature tests compilation passed (`/tmp/talos-fork-raw-prefix-xwin.log`).
Windows fixture releases its captured original stamp before compaction. Independent
review requested; increment is uncommitted and not synchronized into primary.
CI43148646800 remains live at ee9bbc06; do not push and interrupt its native evidence.

Read-only review found no blocker in this preparation increment. Memory limitation:
64MiB bounds source bytes and each raw prefix copy, not whole-process RSS. Evidence
now retains both decoded entries and original raw bytes; a prefix request adds a
bounded raw copy. Publisher header/record-boundary/receipt verification remains
unimplemented. Do not lift any current fork guard based on this helper alone.

Full pinned-toolchain locked standard preflight completed with exit 0 for the raw
prefix increment (`/tmp/talos-fork-raw-prefix-preflight.log`, handle 13137).
This includes workspace tests and the external runtime fixture; it validates this
preparation increment only, not the still-unimplemented fork publication lifecycle.
Latest native CI remains on ee9bbc06; no new push while that run is active.

## 2026-11-12 raw-prefix durable finalization increment (uncommitted)

A real legacy record with noncanonical valid metadata reproduced old finalize
rewriting inherited bytes (`/tmp/talos-finalize-raw-prefix-red.log`). The current
increment copies validated old raw bytes and only encodes new turn records/marker.
Reopen and idempotent replay preserve the exact prefix; valid blank separators are
retained, while an incomplete unvalidated tail is refused without rewriting it.
This is not a tail repair mechanism and does not deliver archived fork receipts.

Independent review and the existing 3MiB-record recovery-scan test caught misuse of
the model-facing 2MiB record decoder. The increment now uses the host decoder limit,
as host transcript validation already does. All 340 session unit tests plus session
integration/doc tests pass (`/tmp/talos-finalize-raw-prefix-green.log`).
Windows cross-check and full preflight remain pending. No primary sync or push yet.

Limitations: one additional whole-transcript raw buffer is retained during commit;
no new total-memory bound is claimed. Source-handle version checks are not a path
CAS against independent outside writers; the host execution lease remains required.
The Windows read handles are released before publication acquisition. Compaction
still needs trusted physical-source locator migration and raw retained-record copying;
receipt ownership must bind the immutable logical inherited prefix across that move.
The reviewer explicitly rejected permanent compaction refusal as full delivery.

MSVC all-feature tests compilation completed exit 0
(`/tmp/talos-finalize-raw-prefix-xwin.log`). Independent read-only re-review after
host decoder/separator corrections found no new blocker. First full preflight
39318 ended exit 101 at Clippy: Unix FileStamp is non-Drop Metadata. Explicit
stamp releases are now cfg(windows), preserving Windows lease release. The second
standard preflight is running as handle 91038 with log
`/tmp/talos-finalize-raw-prefix-preflight-r2.log`; no source edits while it runs.

## 2026-11-12 exact-head macOS preflight step passed

GitHub job130952412310 on ee9bbc06 reports Release preflight SUCCESS, completed
2026-11-12T03:48:42Z. Job itself remains live at Optional language Plugin acceptance;
Windows130952412169 remains live at Test workspace. This is step-level native
validation of ee9bbc06, not whole-run success, nor proof of the complete cause of
older timing failures. Preserve final logs when the job reaches terminal state.
The uncommitted raw-finalize increment is not included in this CI head.

Local preflight91038 remains live in workspace tests. Prepared but did not apply
`/tmp/talos-finalize-raw-primary.patch`; primary source backups and 730-source hash
manifest are in `/tmp/talos-finalize-raw-primary-backup-20261112/`. Dry-run passed.

Second full locked standard preflight91038 completed exit0:
`/tmp/talos-finalize-raw-prefix-preflight-r2.log`. SDK contract now documents raw
prefix preservation, strict tail refusal and memory/receipt limits. This validates
the finalize increment only; archived-fork receipt/publication and compaction
locator lifecycle remain outstanding. No Complete status or deployment claim.

## 2026-11-12 compaction retained raw bytes increment (uncommitted)

Real205-write/noncanonical-metadata/compact50/inspect regression failed on the
previous implementation with conflicting stable record ID:
`/tmp/talos-compaction-raw-red.log`. Compaction now uses validated original retained
record bytes and atomic head publication instead of truncating/re-encoding the head.
The shared raw helper also retains finalize's whole-prefix behavior. Repeated
compact50+append recovery succeeds. Custom store dispatch uses additive default
SessionStore::replace_with_retained_suffix; only built-in TLOG promises raw encoding
preservation. A real JSONL-encoded custom store with .tlog extension and keep0 verifies
that extension names cannot bypass the custom writer. Independent review found no
new blocker after this dispatch correction.342 session unit tests and integration/
doc tests passed (`/tmp/talos-compaction-raw-green.log`); MSVC check pending73414.
This remains candidate-only, uncommitted, not primary-synced. Archive syncing,
trusted receipt locator migration, keep0 recovery and fork publication remain open.

Compaction increment MSVC all-feature tests compilation73414 exited0. Standard
full locked preflight84691 exited0 (`/tmp/talos-compaction-raw-preflight.log`).
Staged source and SDK diff reviewed. Scope remains raw retained-record preservation
and atomic head publication; trusted archive/receipt lifecycle is still outstanding.

## Archive no-overwrite publication increment (Partial)

Archive publication now stages and syncs complete bytes before hard-linking a final
name, refusing existing destinations. Unix syncs the directory before head
replacement. Soft constraint: hard-link support in the archive filesystem; an
unsupported filesystem fails before head replacement. Replace this dependency only
with a reviewed no-replace publication primitive and equivalent durability tests.
Windows directory durability remains unproven. Failure may retain temporary aliases
or complete archives; no cleanup of session evidence is attempted. This increment
does not implement receipt/locator intent recovery or complete archived forks.
Real raw and zstd tests cover existing-destination preservation and fresh exact-byte
publication. Focused all-feature tests and MSVC compilation passed; full preflight
is pending. No deployment or completion claim.

Validation update: standard full preflight exited 0, including locked workspace
check, Clippy, tests and external SDK fixtures; log
`/tmp/talos-archive-publication-preflight.log`. Read-only independent re-review found
no new blocker after compressed coverage and limitation documentation. The seven
all-feature compaction tests passed (`/tmp/talos-archive-publication-compressed.log`).
MSVC compilation is not native Windows publication execution evidence; that gate
remains pending. No receipt/locator/fork completion is claimed.

## Zero-retention source locator implementation (uncommitted, Partial)

Real 205 writes → compact0 → reopen → append regression failed because the new
record named the synthetic placeholder as parent (`/tmp/talos-keep-zero-red.log`).
Candidate now privately registers exact marker bytes, archive name/digest and
logical original tail before head replacement. Only actual first-record markers
activate locators; bounded archive traversal validates raw bytes, tail commitments,
acyclic transitions and strict new suffix parents. No keepN tail-to-retained edge
is added. Cold append resolves a marker tail through full validated source evidence.
Original IDs and legacy parent values remain untouched. This is not fork authority.

Read-only review identified a deadline/write-initialization bug in the first locator
reader. Replaced it with READ_ONLY/NOFOLLOW, zero busy timeout, caller-deadline SQLite
progress guard and bounded field decoding; schema non-initialization, expired/stopped
calls and oversized-row regressions pass. Cancellation is checked at row boundaries;
inside one SQL step the deadline guard applies. No immediate-cancellation claim.
Soft compatibility bounds:256 locators per owner,512 bytes per original tail ID;
exceeding limits fails before replacing the original head. No locator is synthesized
for legacy keep0 archives. No UUID reuse/deletion-lifecycle completion claim.

351 session tests plus integration/doc tests passed before adding the process-crash
fixtures (`/tmp/talos-keep-zero-session-final.log`). Seven focused keep0 tests now
pass (`/tmp/talos-keep-zero-crash-tests.log`), including genuine subprocess exit42
immediately after private registration and after head publication. Both restart and
append with the original logical parent; this is process death, not power-loss proof.
Additional tests cover repeated no-append compaction, two independent sessions,
missing receipt/archive, wrong tail, and marker/archive/suffix corruption.
Full standard preflight and final MSVC checks pending; no commit, primary sync,
new candidate push, merge or deployment for this increment yet. Archived fork
receipts/origins, remaining lifecycle gates and original handoff scope stay open.

Validation refinement: preflight60438 ended exit101 at the unchanged talos-config
`store_unset_builtin_provider_clears_credentials` test, log
`/tmp/talos-keep-zero-preflight.log`. It passed in isolation
(`/tmp/talos-keep-zero-config-isolated.log`). Source inspection shows this test
reads the shared provider environment without ENV_MUTEX while other tests mutate
it under ENV_MUTEX; this is a plausible parallel interference path, not a proven
trace of this occurrence. No config source/test changes were mixed into recovery.
Added missing cancellation checks on locator no-file/no-table returns. The real
keep0→keep1→keep0→cold append regression passes
(`/tmp/talos-keep-zero-mixed.log`). Final MSVC tests compilation35688 exited0
(`/tmp/talos-keep-zero-xwin.log`). Fresh standard preflight will validate the final
source; do not count the failed run as a pass.

Validation update: second standard preflight94811 exited0, including external SDK
fixtures (`/tmp/talos-keep-zero-preflight-r2.log`). Remote run43152762505 completed
successfully at older head0b8ccbe4; it is not CI evidence for uncommitted keep0.
Added actual History prepare/query/detail/paging/source-revocation keep0 coverage
using the same legacy-parent fixture as keep10. First test invocation accidentally
used a fresh target and failed compilation with disk quota exceeded
(`/tmp/talos-keep-zero-agent-query-tests.log`); retry uses existing candidate target.
No configuration fix, session cleanup, or background enablement was included.

## Keep0 model-history query correction (uncommitted, Partial)

Actual History prepare/query/detail regression exposed a separate omission: a head
containing only the count marker passes linear parent adjacency, so archive recovery
was never selected. Red evidence `/tmp/talos-keep-zero-agent-query-tests-r2.log`.
SessionEntry now exposes a descriptive marker predicate (no provenance authority);
linear preparation routes markers to the existing validated archive fallback.
Keep0 and keep10 share real legacy-parent query/page/detail/source-revocation tests.
Missing archive cannot become an empty available history.23 checkpoint-history
tests passed (`/tmp/talos-keep-zero-agent-history-final.log`), under CPU100%,
MemoryHigh2GiB/Max3GiB, no swap, jobs1/test threads1; reported memory peak2GiB.

Independent review then confirmed old completed continuation cursors bypassed the
new per-page check. A persisted old-writer index/continuation fixture reproduced
this (`/tmp/talos-keep-zero-cache-red.log`). Standard preflight35693 was deliberately
stopped before completion; wrapper exit0 is NOT a validation pass. Preparation now
rechecks its sealed first page before accepting either fresh or restored completion;
the classification flag is never persisted. Both archive-present recovery and
archive-missing refusal are tested. Full history tests and renewed preflight pending.
No primary source sync, commit, push, deployment or full-fork claim for this increment.

Old-cache regression now passes both archive-present and archive-missing cases.
Read-only review found no remaining seal/continuation blocker. The first expanded
24-test run passed23 but exposed a diagnostic compatibility regression: the new
first-page read emitted linear_marker_page_failed instead of the established
linear_head_page_failed. Restored the original code rather than weakening the
real log/privacy regression; final rerun pending44056.

Validation state correction: preflight64218 deliberately stopped at CLI test linking
after sustained reclaim (>81000 memory.high events) and high host I/O pressure.
Actual termination was SIGTERM15 after6m29s; wrapper/service success is not a pass.
Check and Clippy had passed; final workspace tests/external fixtures did not finish.
No OOM or swap observed. Keep the same CPU/memory limits for future validation;
reduce debug/link memory cost rather than raising caps. Prior focused24 history
tests remain valid, but do not substitute for final workspace validation.

24 final history tests passed (`/tmp/talos-keep-zero-cache-final.log`), including old
completed cursor and fixed diagnostic privacy coverage. Independent read-only
review confirmed the seal/continuation correction. This bounded increment is being
saved as a local candidate commit with final workspace validation explicitly
incomplete. Next fixed-commit standard validation will reduce dev/test debug info
to0 under unchanged CPU100%, MemoryHigh2GiB/Max3GiB/no-swap caps, with a separate
disk-backed target; this preserves test assertions and does not raise host limits.
No merge, deployment or full archived-fork completion is authorized or claimed.

## Physical owned archive publication increment (Partial, uncommitted)

Keep0 implementation saved locally as f59c9134; not pushed or merged. Independent
fixed-commit worktree `/tmp/talos-recovery-keep0-validation-20261112` runs standard
preflight with dev/test debug=0 and unchanged resource caps. First low-debug attempt
failed C compilation due /tmp quota (`/tmp/talos-keep-zero-lowdebug-preflight.log`),
not a Rust assertion. Retry uses disk-backed TMPDIR, handle pending in session; log
`/tmp/talos-keep-zero-lowdebug-preflight-r2.log`. Final whole-workspace evidence remains
incomplete until that actual command completes.

Candidate now records immutable physical archive intent before hard-link publication,
uses reserved s-owned names, refuses absent/changed private custody, excludes foreign
archives and rechecks all observed ownership during evidence validation. This closes
the reviewed publication-before-registration window without granting fork authority.
Actual materialized-child fixture/append/compact/delete preserves parent205 records;
child-process exit42 at prepublication intent and post-hardlink phases also preserves
parent recovery after child deletion.359 session unit tests passed, log
`/tmp/talos-owned-archive-session-tests-r2.log`, memory peak1.2GiB/no swap. First run
passed356/failed2: owned classification included .replacement sidecar, and an old
fixture overwrote history-bindings.sqlite while calling it unrelated. Classification
now requires transcript suffix; that fixture uses unrelated.sqlite, with dedicated
missing/changed-authority tests retaining strict rejection. No existing acceptance
was removed. Production protocol independent review pending; no primary sync.

Still required: host-issued inherited receipts/origins, materialized child restore
and query without marker, unified staged startup/TUI/REPL publication, deletion/UUID
reuse and downgrade-writer gate, full acceptance and original handoff residuals.

Read-only production review found no new cross-owner import hole. It identified an
unintended legacy filename narrowing; restored the original Uuid::parse_str legacy
acceptance and added uppercase/simple UUID-name fixtures (execution pending after
fixed-commit preflight62690). Owned names remain canonical and require custody.
Foreign files are still fully decoded and charged to the global inventory budget;
a corrupted foreign file may make recovery unavailable. This protocol is import
isolation, not complete availability isolation.359-test evidence predates the
legacy compatibility test/edit; do not count that new test as passed yet.


## 2026-11-12 final owned-archive regression and validation environment

Current owned-archive candidate passed all 360 session library tests (zero failed or
ignored), including legacy uppercase/simple UUID archive-name compatibility, real
process-death publication cases and materialized-child compact/delete isolation.
Log: `/tmp/talos-owned-archive-session-final-r2.log`; CPU100%, MemoryHigh2GiB,
MemoryMax3GiB, no swap, jobs1/threads1; peak226.7MiB. Format check passed.
This is local candidate evidence, not full fork delivery or native Windows acceptance.

Previous attempt `/tmp/talos-owned-archive-session-final.log` passed351/failed9:
large-record writes exceeded /tmp quota, including SQLite disk-I/O errors. No tests
were suppressed. Approved disk-backed `/var/tmp/talos-recovery-tests-wjhuang`
resolves both quota and repository-parent discovery contamination.
Fixed f59c9134 preflight r2 terminated101:684 agent tests passed, three empty/missing
AGENTS fixtures inherited primary repository AGENTS.md because TMPDIR was under it.
No unrelated context source or tests changed. Standard fixed-commit preflight r3
now uses this external temp root and unchanged low-debug/resource settings; log
`/tmp/talos-keep-zero-lowdebug-preflight-r3.log`. Its result remains pending.

Independent downgrade review confirms a TALOS header-version bump is insufficient:
old parser ignores versions. A receipt-bound binary envelope could prevent old
compaction before publication, but must survive all rewrites/archives/nested forks.
It would not prohibit all old direct/cached-tip appends. This is reviewed design,
not implementation; private fork receipts, UUID incarnation, publication/discovery,
all CLI integration and acceptance remain outstanding. Background remains default-off.

Independent read-only review of the current owned-archive diff found no new blocker
for saving this physical-ownership increment: intent precedes discovery and binds
actual stored bytes; immutable owner and foreign keep0 rejection remain enforced;
source/ownership revalidation and legacy UUID compatibility are intact. Reviewer
read the360-test log but did not rerun tests. No full-fork or power-loss acceptance
follows from this review.

Added actual-file regression `undecodable_child_cannot_publish_archive_even_after_unchecked_append`:
a direct append can bypass header inspection, but keep0/keep1 compaction and atomic
replacement must reject an invalid-UTF8 head without publishing any archive/binding
or changing its bytes. This is a test of the existing decoder/publication boundary,
not a newly supported fork envelope. Execution waits behind fixed preflight88157;
360-test evidence predates this additional test. ADR records the reviewed gate
constraints and remaining permanent-header/receipt requirement.


2026-11-12 selective primary synchronization: five owned-archive source files now
match the reviewed candidate, including the additional (not yet run) undecodable
child regression. Before copying, all four existing files matched f59c9134 exactly;
backup `/tmp/talos-owned-archive-primary-backup-20261112-143152/` retains their bytes
and hashes of all735 pre-copy Rust sources. All731 other existing Rust files are
unchanged. ADR/SDK physical-ownership and downgrade constraints appended selectively;
no wholesale doc replacement, no session data or index mutation, no commit/deployment.
360 passing session tests predate only the additional test, not production changes.
Primary whole-tree validation is still outstanding; fixed-commit preflight88157 does
not validate this newer primary or owned-archive candidate.


## Fixed keep0 commit standard preflight passed

Fixed worktree at f59c91343857d749f3a3c14bb26de82618d5daac completed the standard
`./scripts/release_preflight.sh` with exit0, including pinned locked workspace
check/Clippy/tests, governance and both independently resolved external SDK fixture
modes. Log `/tmp/talos-keep-zero-lowdebug-preflight-r3.log` ends with `release
preflight: passed`. Completed test summaries total5346 passed/0 failed/0 ignored.
Runtime16m29s, peak2GiB/no swap; unchanged resource limits, dev/test debug0 and
approved external disk TMPDIR. Previous environment failures were not suppressed.

This proves only fixed f59c9134. New owned-archive production changes, its added
undecodable-child regression and the primary queue/history changes are later work,
not covered by that exact-commit result. Candidate session tests now running serially
in `/tmp/talos-owned-archive-session-361.log`; no overlapping heavy jobs. Complete
archived fork, native acceptance and remaining handoff scope still outstanding.


Current owned-archive candidate now passes361 session library tests, including the
new undecodable-child/direct-append publication refusal fixture. Log
`/tmp/talos-owned-archive-session-361.log`, exit0, no ignored tests, peak1GiB/no swap.
Pinned format check and diff whitespace check also passed. Independent production
review precedes only this additional test/doc change. Primary source copies match
these five candidate source files; primary TUI transition suite is running separately
and serially in `/tmp/talos-queue-history-transition-tests.log`. Final candidate
workspace/MSVC checks and complete archived fork remain open.


Owned-archive candidate MSVC cross-check completed exit0 using repository-pinned
`cargo xwin check --locked -p talos-session -p talos-agent --all-features --tests
--target x86_64-pc-windows-msvc`. Log `/tmp/talos-owned-archive-msvc.log`; existing
unused-code warnings remain visible. This is compilation, not native Windows
execution.361 Linux session tests, format/diff checks and independent production
review passed. Saving this bounded increment locally does not complete archived
fork or the overall task; candidate-wide preflight and remote/native evidence remain
required. Primary uncommitted changes and all session data remain preserved.

## Shared startup/TUI fork staging increment (uncommitted, tests pending)

Owned-archive implementation saved locally as6483241c. Clean validation worktree is
now detached at that commit; standard preflight handle99418/log
`/tmp/talos-owned-archive-preflight.log` remains live. No overlapping heavy test run.

Candidate-only CLI change extracts shared begin/copy/finish staging from TUI and
routes startup --fork through it. Startup no longer scans one snapshot then copies
a later unvalidated snapshot, or writes directly over an existing destination.
Source capture/version validation, exact bytes, create-new staging, sync and
hard-link publication are shared. Existing destination is never enrolled in rollback;
after successful install the manager retains sole sidecar/index rollback/error handling.
Blocking path keeps its10s cooperative deadline through finish checks; async path
retains per-page2s/cancellation and adds bounded finish checks. Kernel I/O cannot be
interrupted by those checks.

Independent static review found no new ownership/cleanup blocker and identified
missing finish deadline checks, now corrected. Actual-file regression added for
multiline Unicode/tab byte preservation, destination collision, retained publication
and expired finish cleanup; not run yet. Existing startup index/runtime-identity and
archive-refusal tests must pass too. Two CLI files remain candidate-only/uncommitted;
primary unchanged. This unifies two entry paths but does not enable archived fork:
shared storage receipts/origins, format/incarnation gate, REPL integration and full
publication/restart/native acceptance remain outstanding.


Owned-archive fixed commit6483241c standard `./scripts/release_preflight.sh` completed
exit0, `/tmp/talos-owned-archive-preflight.log` ends `release preflight: passed`.
Workspace check/Clippy/tests, governance gates and both external SDK fixture modes
passed under unchanged resource limits and low-debug/external TMPDIR settings;
runtime13m46s, peak2GiB/no swap. This validates6483241c only; it does not cover the
new shared CLI staging increment or primary UI/edit/background changes.
Shared staging fork-filter CLI tests now run serially as14182 in
`/tmp/talos-shared-fork-tests.log`. No source reset, remote push, merge or deployment.


Shared startup/TUI staging validated: all11 fork-filter tests and all431 CLI binary
unit tests passed, zero failed/ignored. Logs `/tmp/talos-shared-fork-tests.log` and
`/tmp/talos-shared-fork-cli.log`. Pinned format and whitespace checks passed.
Coverage includes source bytes, existing destination protection, expired finish,
startup runtime identity/index, TUI cancellation during metadata/commit and successful
retained publication. Static review found no cleanup blocker; finish deadline checks
were added after its observation. Complete archived fork remains unimplemented.

The two tested CLI files were selectively synced into primary after proving both
matched6483241c base bytes. Backup `/tmp/talos-shared-fork-primary-backup-20261112-150745/`; all other Rust source hashes unchanged.
No commit/push/deployment for this new increment; primary full-tree validation and
final candidate gates remain outstanding. No heavy verification process remains live
from this increment (99418/14182/85891 all terminal exit0).

### 2026-11-12 REPL default fork compatibility tail (implementation pending execution)

The latest selected-fork CLI run completed successfully:433 passed, zero failed or
ignored, `/tmp/talos-selected-fork-cli-final.log`. This predates the changes below.
JSONL direct-store read+append permission migration is now recorded in ADR-104,
RUNTIME-SDK-CONTRACT and store Rustdoc; no ACL is changed automatically.

Added `default_repl_fork_excludes_incomplete_tail_and_can_append_after_reopen`,
using actual incomplete TLOG bytes, exact prefix equality, cold append/reopen parent
link checks and source-byte preservation. REPL now selects the last retained record
boundary inside the same bounded source scan. No whole-file tip pre-read is used.
The last-record offset commits only after each page's source/cancellation checks;
finish reuses the complete/source-current checks. The existing multi-page JSONL/TLOG
compatibility-tail regression now also checks last-record cutoff versus physical EOF.
Startup/TUI preserve their existing physical-EOF copying behavior; their incomplete-tail
continuation limitation remains, and full archived fork receipt/envelope remains absent.

Independent reviewer statically accepted the single-scan implementation, with no new
blocker; no execution was performed by the reviewer. Format and diff checks passed.
Both project governance and claim validators passed with zero warnings before this
append. New Rust regression execution and full candidate gates are pending: another
worktree's `talos-git-path-preflight.service` was explicitly confirmed active/running
(MainPID23206), so no competing heavy compilation was started. Do not reuse the old
433 CLI or6483241c preflight results as verification of this dirty candidate.
Next: after that service terminates, run CLI binary and session tests under the existing
CPU100%/2GiB high/3GiB max/no-swap/jobs1/threads1 limits and approved external TMPDIR;
then MSVC tests and exact candidate standard preflight. Latest candidate changes are
not synced to primary, committed, pushed or deployed. Overall recovery remains Partial.

### Unified fork tail boundary follow-up (supersedes REPL-only scope above)

The implementation now applies the same last-record cutoff to startup, REPL and
async TUI staging. Ordinary scan physical-EOF API semantics stay unchanged. The
REPL-only helper and boolean selection were removed. Valid records, including
non-message diagnostic/activation records, retain their original bytes; source files
are untouched, and invalid interior JSONL lines are not cleaned or re-encoded.
The startup+REPL actual-file regression now checks both entry paths, and a new async
staging test covers incomplete TLOG and skipped JSONL suffixes, install, cold append,
reopen and parent identity. This addresses the previously recorded startup/TUI tail
limitation in code, but execution remains pending. The other worktree preflight was
again confirmed live, with cargo59240 and active rustc child, so it was not restarted
or interrupted. Full archived-fork custody/envelope work remains outstanding.

Independent static review accepted unified last-record cutoff: it updates before
model-message filtering, so valid diagnostic/control SessionEntry tails and metadata
are retained. Unknown non-entry JSONL trailers are excluded, not all arbitrary
metadata formats preserved. Async regression now ends its valid prefix with a filtered
diagnostic record and checks its ID/content, exact prefix, and continuation parent.
No compile result is available yet; live preflight23206 remains the serial resource wait.

Cancellation follow-up: expanded the existing mid-page retry test across twelve
callback stop positions using an explicitly selected entry. On stopped pages it
asserts no offset, selected cutoff, last-record cutoff or message publication, then
retries and checks the exact original byte boundary, one message and tail identity.
This locks the new cutoff into the existing transactional page semantics. Format
and diff checks passed; Rust execution is still pending the confirmed live external
preflight, not claimed passed. No second heavy job was started.

### Unified tail execution results

The external preflight service terminated (inactive/dead, MainPID0). Its workspace
tests failed; its evidence does not validate this recovery candidate. A concurrent
sandbox launch also failed with /tmp disk quota exceeded. No session files or test
evidence were removed. Recovery validation used the previously approved external
TMPDIR and unchanged resource limits, with logs on the primary target filesystem.

First CLI run:434 passed, one failed. The older multi-page raw-copy fixture expected
the invalid JSONL trailer in the child. Updated only child expectations to the exact
valid prefix; source-byte preservation, collision refusal and cleanup checks remain.
Final CLI run:435 passed, zero failed/ignored, exit0. Log:
`/home/wjhuang/WorkSpace/personal/talos/target/recovery-unified-tail-cli-r2.log`.
Session suite:362 library tests plus27 integration tests passed, zero failed/ignored,
exit0; doc tests contain zero cases. Log:
`/home/wjhuang/WorkSpace/personal/talos/target/recovery-unified-tail-session.log`.
This includes callback-position cancellation, multi-page record cutoff, and original
byte preservation. Format and diff checks passed after the final test correction.
MSVC test compilation and full exact-candidate preflight remain outstanding; no
commit/push/deployment or full archived-fork completion is implied.

Windows follow-up: `cargo xwin check --locked -p talos-session -p talos-cli
--all-features --tests --target x86_64-pc-windows-msvc` completed exit0 under the
same resource caps; log `target/recovery-unified-tail-msvc.log` in the primary
workspace. Existing CLI workspace_root dead-code warning remains. This is build
compatibility, not native Windows lifecycle acceptance. The standard candidate
`./scripts/release_preflight.sh` is now running as
`talos-recovery-unified-tail-preflight.service`, tool session88951, logging to primary
`target/recovery-unified-tail-preflight.log`. Poll this exact run before restarting
or launching another heavy check; its result is pending.
Current source files remain fixed during validation. Final independent read-only
review found no new production blocker and confirmed the CLI/session logs; it did
not run compilation. Complete archived fork remains unimplemented.

### Disk-backed candidate and preflight correction

The first unified-tail preflight exited101 on CLI EventLoop.workspace_root dead code,
left unused by the shared fork refactor. Removed that field, its constructor argument
and the single production/test initialization; no warning suppression. Format and
diff checks passed. /tmp quota prevented appending the owner tail, so the complete
owner was preserved in primary target/recovery-owner-repaired.md and restored.
Git worktree move failed across filesystems; copied all candidate files and repaired
worktree registration to /home/wjhuang/WorkSpace/personal/talos/target/session-recovery-integration.
The old /tmp directory remains a backup, no longer the active registered worktree.
No session data or dirty primary files were removed. Standard preflight r2 now runs
on the relocated candidate, tool session78276, unit talos-recovery-unified-tail-preflight-r2,
primary log target/recovery-unified-tail-preflight-r2.log. Result pending; poll this
run rather than restart. Recheck MSVC after constructor change; full archived fork
and remaining handoff requirements remain open.

### Unified-tail final standard validation

Relocated candidate standard preflight r2 completed exit0 (`release preflight: passed`),
14m7s, peak2GiB/no swap under unchanged caps. Log: primary
`target/recovery-unified-tail-preflight-r2.log`. Completed test summaries report5359
passed/0failed/0ignored; external SDK fixture validation passed. This validates the
current shared fork/cutoff/JSONL candidate including constructor cleanup. Independent
review separately accepted the constructor cleanup. Migration compared2126 files,
no missing files; only two intended source cleanup files and this owner differed.
MSVC needs recheck after constructor cleanup. No full archived fork, primary sync,
commit/push, native host or native Windows acceptance is claimed.

Final constructor-cleanup MSVC all-features tests check exited0 (4.93s, peak984MiB),
primary log target/recovery-unified-tail-msvc-final.log; the previous dead-code warning
is gone. This remains cross-compilation only. Saving this reviewed increment locally
does not close archived-fork receipt work or authorize deployment.


## 2026-11-15 selected recovery candidate checkpoint — Partial

Recovery source now includes receipt/incarnation-backed archive inheritance,
identity custody and retirement fences, bounded original-prefix capture, Actor
operation exclusion, and startup/REPL/TUI runtime replacement. Original record
bytes and parents remain unchanged; unavailable or ambiguous provenance is refused.
SDK/ADR describe additive APIs, storage access requirements and migration limits.
This is implementation evidence, not installed-host or release acceptance.

Mixed candidate standard preflight passed5496 tests/doctests,0failed/0ignored and
both external SDK fixture modes: primary-worktree target/recovery-candidate-system-wb-r2.log.
Its940 source-input hashes matched target/recovery-candidate-system-wb-manifest.json.
Earlier run failed an unchanged Provider HTTP deadline fixture; same-binary
focused/full reruns and subsequent full gate passed, but cause remains unresolved.
These results cover the mixed candidate tree, not a separately built selective
commit tree. Independent reviews and isolated CLI/PTY evidence remain recorded
in the preserved primary coordination/recovery owners and original candidate file.

The selected recovery source excludes queue presentation/layout, checkpoint test
diagnostics, and primary-only background-completion interactions. Fixture isolation
was separately saved in local commit736b77f992a5a135b285cca3a72548d853c32f94.
The original mixed owner history remains preserved in the worktree and backup;
only this checkpoint accompanies the selective recovery artifact. No completion
status or Completion Commit is asserted. Native-platform/installed-host acceptance,
real-provider summary quality, exact-head remote checks and delivery remain open.
No session data cleanup, default background activation, push or merge authorized
by this checkpoint.

## 2026-11-15 selective recovery standard gate — Partial

The exported recovery-only index passed standard release_preflight.sh (exec80710):
5490 tests/doctests, zero failed/ignored, both external Runtime SDK modes passed;
exit0, runtime15min19.776s, peak2G and swap0. All2445 exported inputs were unchanged
at completion. Evidence in the primary target directory:
recovery-selective-noincremental.log, recovery-selective-validation-manifest.json,
and recovery-selective-preflight-result.json. This supersedes pending selective-tree
validation above; the mixed-tree gate remains separate historical evidence.

Current index differs from the tested snapshot only in the previously reviewed ADR
keep0/release-enablement wording and this post-run evidence. Source equality is
recorded in recovery-selective-index-comparison.json; current documents receive
separate governance checks. Earlier failures/stopped runs remain retained.
No installed-host/native-platform acceptance, remote exact-head validation or
release/merge authorization follows from this local gate. Overall status remains
Partial; original sessions, dirty changes, backups and migration sources are retained.

## 2026-11-15 recovery ADR renumbering

Remote main now contains ADR104-host-composition-ownership-boundary.md. Recovery
ADR is therefore renamed to [ADR-105](../decisions/105-selected-session-history-recovery.md);
SDK and decision-index references follow the new name. Earlier ADR-104 references
in this historical owner describe the recovery decision at that time, not the host
composition decision. Originals are retained in target/recovery-adr105-migration-20261115
under the primary workspace. This is document-only; the5490-test selective gate
continues to cover the unchanged production source. Status remains Partial.


## 2026-11-15 parent-directory alias recovery gate — Partial

Implementation commits: 2e85fdb0 (recovery directory aliases) and f0eb335b
(portable Agent workspaces and closing the external test writer before recovery).
SQLite recovery opens canonicalize the parent directory only, retaining final-file
NOFOLLOW and each existing read/write/create policy. The index and pending runtime
reader use the same boundary. Database-file aliases remain refused; regressions
check no-create observation and unchanged original bytes. Independent review found
no blocker in the combined eight-file slice.

The exact source export passed standard release_preflight.sh: 5491 tests/doctests,
zero failed/ignored, both external Runtime SDK modes, exit 0. All 2445 input hashes
matched at completion. Evidence in the primary target directory:
recovery-alias-validation-manifest.json, recovery-alias-selective-preflight4.log,
and recovery-alias-selective-preflight-result.json. Run time 9min23.464s, peak 2G,
swap 0B, single-job resource limits and explicit LC_ALL=C/LANG=C. Earlier setup,
formatting and locale failures remain retained; no test assertions were relaxed.
Focused alias-path Session 428 / Agent 730 tests also passed.

Windows MSVC all-features/tests cross-check for Session and Agent passed with
locked dependencies (recovery-alias-xwin-check.log); one Agent dead_code warning
remains. This is compilation evidence only. Native CI at df29bb9c failed macOS
recovery paths and Windows tests; refreshed native exact-head validation is still
required. The Windows concurrent first-open database-lock failure is unresolved;
its open/schema/transaction phase must be established before changing behavior.
No installed-host acceptance, deployment, merge or Complete claim follows. UI,
approval-history and background work are excluded; background remains default-off.
Existing sessions, migration sources, backups and unrelated dirty work are retained.

## 2026-11-16 atomic binding schema publication — Partial

Implementation commit: `6ad2f96fcdb3eb393eb9018dcfe4fbc585a9ad79`.
First-open schema publication now uses one DEFERRED transaction, keeping WAL/FULL
configuration outside. Failed DDL/commit rolls back before retry; a connection still
inside a transaction is not retried. Existing500ms/20retry/25ms limits, five-second
post-initialization busy timeout and six-worker success assertions are unchanged.
No database format or user-data rewrite is introduced; existing bindings and companion
rows survive incremental schema completion, and both cleanup triggers retain their effect.

Deterministic regressions failed on the former implementation: denying the fourth
CREATE left three committed tables; successful schema setup invoked six commits.
The atomic implementation passes those regressions plus rejected-COMMIT/reopen and
partial-schema/data/trigger checks. Focused durable45 and final Session433 tests passed,
zero failed/ignored. Logs: target/binding-schema-{red-tests,green-tests,session-tests}.log.
Exact-commit independent source review found no blocker. Standard clean-checkout
preflight passed:5494 unfiltered tests/doctests, zero failed/ignored, two external SDK
modes; all2445 source files matched before and after the run. Logs/manifest/result:
target/binding-schema-preflight{.log,-source-manifest.json,-result.json}. Runtime
15min8.937s, peak2G, swap0. This gate used pinned1.99.0, locked commands and the
specified TMPDIR. Native validation for this new implementation remains pending.

Previous diagnostic headc8703bdb native run43739517986 passed scoped Windows/macOS
jobs (Session407/429), while global Issue reconciliation failed independently. Its
Windows timings localized329–411ms to initialization-to-BEGIN, not head creation;
they include scheduling and do not prove the earlier intermittent failure's cause.
This production change requires fresh native validation. The earlier macOS
identity-custody failure remains independently unresolved. Installed-user-session,
real-provider summary quality and the full handoff residuals remain open; no merge,
deployment, cleanup or default background activation is claimed.

## 2026-11-16 final logical identity-lease release — Partial

Independent review confirmed that the isolated Linux probe retains no unintended
local lock references. Concurrent-spawn explicit-unlock control then produced
0/20,000 WouldBlock results (target/custody-lock-unlock.log), versus981/20,000
with close-only release. This establishes a bounded lock-lifetime repair target,
not the cause of the original macOS receipt failure.

Source commit f5217299 changes only identity_custody.rs: Arc now owns an internal
lock holder that explicitly unlocks once, when the final logical lease ends.
Failure/panic is contained and File closes as fallback; no retry relaxation,
unsafe code, public API or persistent format changes. Deterministic extra-file-
reference regression fails before the fix (target/custody-release-red.log), then
passes for SH/EX with live clones protected. A second regression preserves
independent shared holders. Old descriptor close cannot release a new lease.
Full Session435passed,0failed/ignored (target/custody-release-green.log).
Independent source/security/API review found no blocker; native validation pending.

Clean detached worktree target/custody-release-validation-20261116 is running
standard release_preflight at f5217299 under resource limits, pinned1.99.0, locked
commands and requested TMPDIR. Service talos-custody-release-preflight-20261116;
exec49682; log target/custody-release-preflight.log. Source manifest and terminal
result are produced by target/custody-release-preflight.py. No push yet.
All other dirty files and migration/session data preserved. Overall remains Partial.

Full clean preflight subsequently completed exit0:5496 unfiltered tests/doctests,
zero failed/ignored, two filtered subprocess passes and two external SDK modes.
All2445 source file hashes matched before/after. Runtime15min47s, peak2G, swap0.
Evidence: target/custody-release-preflight.log, -result.json and -source-manifest.json.
This is source f5217299325b667ca8c8ae3f78b3a4c9e2241d12; native checks remain pending.

## 2026-11-16 authorized SQLite dependency candidate — Partial

User explicitly requested migrating SQLite to latest when feasible. Isolated
source baseline4e8054a29a605e54b2752179331b318be4d6c49c; existing dirty
worktrees and real session/migration originals remain untouched. Selected
observed non-yanked stable rusqlite0.43.0/libsqlite3-sys0.40.0 (SQLite3.54.1)
replaces0.40.2/0.38.2 (SQLite3.53.2), retaining all five consumers' features.
Official sparse-index package checksums verified; API endpoint403 recorded.
MSRV1.89/1.88 is below workspace1.95, pinned validation compiler1.99.0.

Primary motivation is upstream Windows long-DOS-device/UNC fix2429af2e,
plus native boundary fixes in rusqlite release notes. Native old/new path
comparison, API semver assessment, old-data reopen/rollback compatibility,
full locked preflight and all-feature consumer gates remain required.
No schema migration or user-data rewrite is intended; source rollback must
restore manifests and lockfile together. Existing originals are preserved.
This candidate does not imply deployment, acceptance or default continuation.

## 2026-11-16 SQLite migration focused validation — Partial

Five-consumer locked all-target/all-feature compile passed after explicit mutable
progress-guard migration. Session library435tests passed0failed/ignored, including
real cancellation/deadline rollback, panic cleanup, legacy Todo migration and
private receipt retirement. Log target/sqlite-upgrade-followup-checks.log; external
API consumer is still compiling in the same live service/session2692. Independent
RAII source review found no blocker; no final exact-head gate yet. Initial workspace
check found Agent mutable-index bindings, now adjusted; workspace rerun still due.
Both governance validators passed0warnings; scale assessment retained existing policy.
Formatting check found wrapping differences, correction due after active build.

Native old-version path comparison at4e8054a2/run43796341946 completed successfully:
24ordinary+24extended opens, SQLite3.53.2, all successful. Extended spelling survives
SQLite open, but protocol failure was NOT reproduced and causality remains unproven.
Raw artifact14194429524/log target/wal-path-native-4e8054a2.log and SHA-bound
-evidence.json retained. User-authorized latest dependency upgrade proceeds on its
own merits; do not claim this experiment proves a PROTOCOL repair.

External typed-error consumer added in isolated candidate tests/fixtures/sqlite-api-external;
existing Runtime/Browser SDK lockfiles updated only for4SQLite packages after removing
unrelated offline resolver drift. Old/new/old synthetic-data harness is prepared at
target/sqlite-version-migration-20261116/run.py, not yet run. Full/native/new-version
checks, data compatibility, installed-host and all original handoff residuals remain.

## 2026-11-16 SQLite expanded validation and real-copy acceptance — Partial

The corrected candidate passed the full standard locked preflight (5498 passed,
zero failed/ignored), strict workspace all-target/all-feature Clippy, workspace
all-feature tests with real packaged Rust/Python language guests, Session minimal
feature tests, deterministic dependency audit (93-record Bash/PowerShell parity),
and external Browser/SQLite API consumer runs. Five SQLite consumers also passed
Windows MSVC --tests cross-check; native execution remains a separate gate.
Evidence in primary target/: sqlite-upgrade-full-preflight-v2-result.json,
sqlite-upgrade-extended-v6-result.json (overall failure was the subsequent Browser
fixture lock, after passing all-feature/minimal stages),
sqlite-upgrade-external-final-result.json, sqlite-stamp-followup-validation-result.json,
and sqlite-upgrade-dependency-audit-tests-v3-result.json. The Browser lock correction
also removes an obsolete hashbrown -> foldhash feature edge; the earlier description
of only four SQLite package changes was incomplete. Workspace source hashes remained
unchanged during the recorded gates.

Old/new/old synthetic migration passed with committed WAL, BLOB/FTS, fork lineage,
partial schemas, rollback/integrity/checkpoint and old-reader reopening. Evidence:
target/sqlite-version-migration-20261116/result-v2.json. This direct rusqlite harness
is supplementary, not a substitute for production Talos acceptance.

Production CLI on preserved real data COPIES passed two cold-start rounds: exact
archived-text discover/search/detail, fresh compact_context Admitted/Applied receipt
correlation and inclusion of each round's unique derived checkpoint in a subsequent
provider request. Original38 files (36 transcripts/archives +2SQLite backups) retained
matching hashes; other copied transcripts unchanged and selected head prefix retained.
Evidence: target/sqlite-real-copy-acceptance-v5-20261116/{result.json,case/report.json}.
Binary SHA256:6ebe2e17e412085510680cb4cade47b214b6ed5277f63b7c84a49600801d05fb.
Outer bwrap kept originals read-only and network isolated (loopback only); scripted
provider requests were not persisted. Each cold start required41 structured source
folds. This proves query and fresh checkpoint installation on copies, NOT persistent
derived-checkpoint restoration, installed-host acceptance or real-model summary quality.

Earlier failed harness runs are retained: v1 used a model identity conflicting with
the copied durable activation. V2-v4 retained the original identity but returned plain
text to a structured JSON checkpoint request; no history tool was reached. Correcting
the fixture to return source-cited JSON completed v5 in8.99s. No production code was
changed to bypass these failures. The REPL's lack of a visible terminal outcome after
an invalid checkpoint response warrants a focused follow-up; cause/required production
change remains unproven and must not be claimed fixed by the valid-response fixture.

Remaining: final independent current-diff review, selective commits separating test
lint prerequisites, exact-head native Windows/macOS CI and upgraded path-probe evidence,
plus original host/UI/approval handoff residuals. Candidate is not merged, installed or
released; background continuation remains default-off. Overall remains Partial.

## 2026-11-16 REPL pre-start checkpoint rejection follow-up — Partial

Invalid checkpoint response reproduction identified a presentation gap: compatibility
submission preparation can reject before TurnStarted/Completed, emitting
SubmissionRejected(ContextBudgetExceeded). REPL ignored it and remained AgentRunning.
A focused event-forwarder regression failed before the fix (session70564, one-second
forwarding deadline), then all9event_loop tests passed (session84529). Forward bounded
rejection metadata as AgentError to return to input; no provider body or submission ID
is displayed and no transcript rewrite or admission relaxation is introduced.

Real production CLI PTY with the same preserved data copies and invalid plain-text
checkpoint response passed in5.64s. It saw the context-budget error and prompt, exactly
one summary attempt/no normal tool request, unchanged selected head before the next
request, and all38original hashes unchanged. A subsequent request-preview returned to
prompt. Evidence target/repl-invalid-checkpoint-acceptance-20261116/result.json and
case/invalid-checkpoint-report.json; binary SHA256
e49c55dc62e62a74740198b66465f8e75ba26c8a2bd2d1180e88aae07c662f0f.
Follow-up remains uncommitted pending review; no claim of general invalid-response
recovery, installed-host delivery or default background continuation.

PR917 at e9f52a55 has no checks and reports DIRTY. Fetched origin/main b524c7a9;
non-mutating merge-tree identified only docs/decisions/README.md conflict (ADR104
upstream entry and local ADR106 insertion). No numbering collision. Preserve both.
Candidate synchronization and fresh validation/native CI remain required.

## 2026-11-17 plain-text recovery query follow-up — Partial

Base PR917 head4b9760b799913be5eeabcf5a7c559a275db0d3f7 previously passed exact-head
CI43828356349 (8jobs) and independent review. A subsequent combined synthetic-only
real-provider REPL walkthrough reproduced history recovery Unavailable for ORCHID-7319.
Read-only synthetic SQLite index reproduced raw FTS no such column:7319; quoted phrase
matched. Real commit/reopen/discover/search/detail regression failed before correction.

This follow-up treats model queries as text using additive
SessionIndex::recovery_text_candidates: raw input<=1024UTF-8bytes, bounded internal
phrase escaping<=2050. Raw recovery_candidates API keeps its original syntax and1024
limit. Scopes,source seals,permission leases,deadlines,pagination bindings and native
failure boundary are preserved. Search matching remains linear tokenized phrase versus
archive case-insensitive containment; no claim of identical byte matching. Empty browse,
quotes,operators,1024ASCII/quotes/UTF-8 and1025reject are covered. The first agent-only
escape attempt failed the1024regression and was replaced; failures retained.

Schema description increased request budget and exposed a CLI model-switch fixture
assumption:32000capacity/output8192 measured25666tokens. Fixture low baseline is now
40000 with all pressure,output-reserve,history and tool-equality assertions preserved;
production thresholds unchanged. Combined focused storage17/checkpoint99/child1 and
CLI fixture tests passed; exact combined commit0e0ee5d3 technical/API/security approved.
Evidence in primary target/handoff-query-length-red-20261117.log,
 target/handoff-query-length-focused-20261117.log and
 target/handoff-pressure-diagnostic-20261117.log. New standalone full/native validation
and exact-head review remain pending; combined evidence alone does not close them.
No real-provider semantic success,installation or product merge claimed. This is original
history-unavailable scope; background continuation remains default-off.

## 2026-11-17 source-reference protocol and real-provider acceptance — Partial

The model-visible source protocol now provides history_search source_detail for
discover UUIDs, with bounded user/public-assistant projections and explicit unsupported
source errors. Durable search references remain separate. Wrong source/snapshot input
no longer consumes issued state; a cancellation-safe admission reservation prevents
rediscovery from admitting a second operation while Actor admission is pending.
Source,permission,identity and final cancellation checks remain mandatory. SDK docs
state the8192byte/item,2MiB snapshot text payload and unsupported tool/multimodal limits.

Red regression78065 failed2/2 before implementation. Expanded focused96892 passed
109checkpoint+1child and3real host paths. Two test-fixture errors were preserved and
corrected:refutable enum binding and synchronous source-bind aggregate limit; the
latter now uses production cooperative bind_until without changing production limits.
Final focused6passed, then full release_preflight passed5619summed tests/0failed/0ignored,
SDK and SQLite external consumer checks. Exec11272 completed0,12min53.509s,CPU8min15.157s,
peak2GiB/swap0; log target/checkpoint-source-final-r2-20261117.log. Independent
technical/API/security APPROVE for source patch771e48f62e5944059f51d230f085d76d6fa68159.
CLI SHA256 a46172b993223c8442ca8a4c1c50ad51a8700d8cc59b1375b12724eeeaae8874.

Real-provider r3 correctly read source_detail and obtained Admitted but reported
Preparation failure. Its8eligible messages estimate252tokens; even UUID-only submitted
render estimates401tokens versus maximum226 for required10%reclaim. This proves the
short fixture violates a necessary reclaim condition, not a unique diagnosis from
generic Preparation. Preserved r3 artifacts. No production thresholds were relaxed.
R4 adds redundant synthetic progress text and preserves configured model limits;
REPL seeds13turns, then cold inline resumes so actual ModelCheckpointTerminal is
observable. Same goal prompt and binary; only synthetic data sent. Five actual calls
discover→search→detail→source_detail→compact all successful, exact issued source refs
verified; operation0 Admitted→actual inline Applied:59689→58259 estimated request tokens.
Final summary retained goal,constraints,decision,fixture-reported38hashes,pending native
acceptance,unknown approval cause,rejected automatic continuation,and next step.
Exec66671 completed0,48.211s,peak250.9MiB. Evidence:
 target/handoff-real-provider-inline-r4-20261117/{structured-review,semantic-review}.json
and real-provider.pty.log. Provider config removed after run. TLOG does not persist
terminal receipts here; its parser receipt count0 is NOT an Applied/non-Applied verdict.

This certifies one synthetic cold inline real-provider case and tested Actor paths,
not installed-host acceptance,REPL terminal display,or broad summary quality. Original
sessions remain intact; no default background activation,merge,installation or release.
Standalone recovery follow-up still needs its own full/native CI before delivery.

Standalone source protocol patch matches combined commit ceb86be47cfa6e33dd21fae6d86dec173403a70e. Combined source/API/security review and r4 evidence review approved. This standalone branch still awaits full validation and exact-head native CI; no product merge or installation.


## 2026-11-17 Windows binding initialization follow-up — Partial

Authorizing maintainer: current user explicitly assigned the native Windows storage
blocker for TUI evidence PR #965 to this recovery task. This continues the existing
emergency recovery scope; no Search, storage-composition or TUI performance change.
Branch: `fix/session-windows-init-diagnostics-20261117`, based on main `08a302ea`.
Rollback: revert only this follow-up; retain databases, transcripts and all backups.

Native failure: PR #965 head `a14b8727481e20ec3cfb44a0b5ed4fbadbf7db37`,
run `43994859879`, job `133744347337`, actual six-worker create_or_open test.
Five workers failed SQLITE_BUSY(5,5) in initialization after approximately529–530ms;
one reached its subsequent business transaction at528ms and completed664ms.
The48-worker WAL probe and later green CI are not root-cause elimination evidence.

A real two-connection regression independently reproduces SQLITE_BUSY: an early
connection caches empty schema, another publishes full schema and holds ordinary
binding DML, and the early connection then tries IF NOT EXISTS initialization.
Candidate establishes a read snapshot of sqlite_schema in the existing transaction
before preparing DDL. The deterministic regression fails without this read and passes
with it. This proves the local stale-schema contention mechanism, not yet that it
fully explains or eliminates the recorded native Windows failure. No timeout, retry
count, six-worker success assertion or schema atomicity requirement is weakened.

Test-only SQLite statement/profile callbacks emit fixed labels for journal mode,
synchronous, begin, DDL, commit and rollback. No SQL text, bound values, paths or
identifiers are emitted. Statement callbacks do not prove preparation succeeded;
PROFILE means returned, not successful. Per-attempt autocommit state is recorded.

Validation so far: pinned/locked49durable tests and full talos-session468tests passed
on Linux, zero failures/ignored. Logs retained in the primary workspace:
`target/windows-init-stale-schema-red-20261117.log`,
`target/windows-init-stale-schema-fix-20261117.log`,
`target/windows-init-session-gate-20261117.log`. Native Windows verification,
independent review remain pending; full preflight results follow below.
The known approval-test initializer mismatch is now fixed by authorized UI merge
#962 (`08a302ea`) in the target branch; this storage candidate has no CLI fixture
delta. Pre-integration patch and stash remain preserved in the primary workspace.

No installation/release/default background enablement. TUI #965 waits for an actual
validated repair merge SHA. Existing handoff/host/Desktop/low-rate residuals remain
open; this follow-up does not close the complete recovery task.


Final local gate on main08a302ea: standard pinned/locked release_preflight passed
5624tests,0failed/ignored; external Runtime SDK and SQLite API fixtures passed.
Exit0,17min53.301s,peak2GiB/swap0; all2686tracked file hashes unchanged throughout.
Log: target/windows-init-preflight-20261117.log; source manifest: matching
windows-init-preflight-source-manifest-20261117.json in the primary workspace.
Native Windows exact-head CI and independent review remain required before merge.


## 2026-11-17 native recovery latency and visible lifecycle follow-up (Partial)

Authorization Mode: Emergency override.
Authorizing maintainer: current user; original-session recovery incident and full handoff closure.
Branch: fix/session-native-recovery-failure-20261117, based on b2a7d471.
Scope: bounded checkpoint batching and automatic-compaction presentation only.
Original installed candidate failed original-session admission after restoring 2724 messages.
Exact original error category is unknown; delayed-provider regression proves a matching
shared-deadline failure mechanism, not conclusive attribution of the original failure.

Replaced fixed 64-entry batches with geometric serialized-byte admission, preserving
complete tool groups, whole-request checks, 120-second deadline, 64-call bound,
source verification and recent history. The previously failing 2724-message/4-second
provider regression and all 33 checkpoint_request tests pass. Packing-only CLI passed
both delayed current-session-copy cold resumes in 42.64 seconds, with history detail,
fresh Applied, source hashes and archive/head preservation verified. That executable
predates the presentation edits and is not installed.

Automatic preparation now keeps a visible activity indicator after tip expiry;
typed terminal reasons remain in presentation history, without transcript writes.
Focused frame test and bridge test passed (one each); bridge test additionally checks
provider failure clears activity and retains the bounded failure reason, plus stale
generation rejection. Extended bridge test rerun passed 1/0 on 2026-11-17.
Primary evidence: target/recovery-ui-focused-20261117.log and
 target/handoff-delayed-provider-green-20261117/delayed-current-session/case/report.json
in the preserved primary workspace. No original session or installed executable changed.

Validation remaining: complete preflight, combined CLI isolated acceptance, independent
review, exact-head CI, authorized delivery and original-session reacceptance.
Rollback: revert this follow-up only; retain session/archive/pending files and snapshots.
Residual owner: this document; full handoff remains Partial. No release or background
continuation enabled. Existing primary dirty edits and missing old /tmp provenance
remain preserved/tracked, not silently reconciled by overwriting documents.


## 2026-11-17 combined recovery candidate full validation (Partial)

Full pinned/locked release_preflight exited0: workspace check/Clippy,5626 tests
(0failed/0ignored; TUI716), Runtime SDK default and coding configurations, external
SQLite API fixture all passed. Log target/recovery-combined-preflight-20261117.log;
26m15.937s, peak2G/swap0. Combined CLI SHA256
934e742b2534b60ae44f7e6299f26325add888ae28b1f16e92af4c8e5601c40f.
Same binary passed delayed current-session snapshot two cold resumes/history/fresh
Applied (40.99s), real ANSI TUI automatic-compaction activity after tip expiry,
retained provider-failure reason and cleared activity, queue/history spacing,
approval history and edit-line display. No transcript-copy mutation on failed
admission; preserved originals and archive copies checked unchanged.
Evidence: target/handoff-combined-recovery-20261117/result.json and
 target/handoff-combined-recovery-20261117-r2/result.json in primary workspace.
Initial status harness failed on pyte display empty-character IndexError, before
assertions. Original artifacts retained; changed screen extraction to the existing
buffer-cell approach and ran remaining checks, without rerunning passed latency case.

Not installed, merged, released or original-session reaccepted. Independent review,
exact-head CI and authorized delivery remain open. Missing Nov11 evidence provenance
and low-auto-approval causality remain handoff residuals, not erased by these passes.

## 2026-11-18 original-host rejection diagnostic follow-up (Partial)

Authorization Mode: Emergency override. Authorizing maintainer: current user,
continuing the original-session recovery incident after failed PR969 reacceptance.
Branch: fix/session-checkpoint-rejection-20261118 (isolated existing worktree,
based on d97ddcb2). Scope: content-free checkpoint rejection diagnostics, isolated
real-provider reproduction and regression for the observed cause. No unrelated
Search/Git/SQLite dependency work. Preserve all primary dirty edits and original data.

User confirmed host exit; process check found no versioned host and all non-database
snapshot files remain byte-identical. Diagnostic logs admit only audited constant
reasons; unknown preparation errors and provider strings are not echoed. Tests cover
known schema/source/budget/tool-group distinctions and unknown/private-text masking.
Diagnostic build and focused test running under pinned/locked/resource-bounded flow.
No functional recovery fix, merge, installation or new original-host attempt claimed.
Next: one real-provider attempt on isolated snapshot; inspect exact rejection and add
causal regression before changing behavior. Keep privacy/source/budget gates intact.

### 2026-11-18 isolated real-provider findings and bounded correction (Partial)

Diagnostic binary 0838a9be reproduced `checkpoint exceeds rendered token budget`
on the verified original-session snapshot with the actual configured provider.
Evidence: target/recovery-real-provider-diagnostic-20261118 (source snapshot unchanged).
This proves the isolated-run cause; old screenshot lacks exact detail, so attribution
of the earlier original-host failure remains bounded.

Candidate adds a conservative whole-JSON byte target and one correction only for
parsed/source-valid RenderLimit. It shares original 120s deadline, 64-call cap,
256KiB aggregate generated output, whole-request admission and source validation;
no raw truncation or threshold enlargement. Regression checks correction success,
persistent oversize refusal, invalid-source no-retry and shared deadline. Old d97ddcb2
fails this regression; corrected implementation passes. 114 checkpoint-filtered tests
passed; full preflight/independent review/CI/merge/install are not yet complete.

First real corrected run obtained three actual successful history_search tool results,
but the PTY harness mistook a quoted `> ` for the final prompt and stopped early.
Preserved as incomplete evidence, not a passed run. Anchored prompt matching was fixed.
Second run returned provider-category failure around 120s with no history calls;
not accepted. Added content-free fold counts, sizes, remaining-time and differentiated
provider/deadline labels before another diagnostic run. Do not repeat runs merely to
get green or claim robustness from a single successful tool sequence.

All attempts use isolated snapshots/config/workspace; installed executable and original
session remain unchanged. Source changes only in isolated fix/session-checkpoint-rejection-20261118.

### Follow-up diagnostic and correction request size

Detailed isolated run `target/recovery-real-provider-render-diag-20261118` identified
first fold: 2578 sources, estimated 258698 request tokens, ~90.75s provider latency,
18201 output bytes / 4551 estimated tokens. Render cap remained 4096. Original
correction resent full sources (258791 request tokens) with 28553ms left and hit
`checkpoint deadline exceeded`. This is observed evidence, not inferred network failure.

Candidate now shortens only the parsed/source-validated oversized checkpoint with
no raw-source replay, retaining source whitelist, prior semantics instruction and
all shared bounds. First edit missed the prompt wiring; regression caught it
(113passed/1failed), preserved log, corrected before any real run. Updated114 tests
and CLI build passed, including empty raw Sources on correction, reduced prompt size,
source retention, rejection and deadline assertions. Real changed-candidate check pending.
No public API change or provider-specific reasoning override introduced. Existing
stream_decision provider API was inspected but is a bounded decision contract with
unsupported-provider refusal; not repurposed silently for all checkpoint providers.

### Latest real-provider result: incomplete, no delivery

`target/recovery-real-provider-small-correction-20261118`: first fold admitted with
2578 sources /258698 estimated tokens, no successful summary within shared120s;
exact diagnostic `checkpoint deadline exceeded`. Correction was never entered.
Original transcript/archive non-database files still match prelaunch snapshot.
Stop repeating provider runs to seek green. Candidate is NOT a complete recovery fix:
render-budget correction unit coverage passed, but real-provider first-fold latency
remains unresolved and correction's live execution was not observed. No merge,
install or original-session restart authorized by this result. Overall Partial.
Next work: research/test a bounded preparation path that can complete the large
source corpus under the existing deadline, retaining complete source coverage and
privacy/budget rules. Do not increase timeout or silently drop history. Full preflight,
independent review and exact-head CI remain gates after the implementation converges.

### 2026-11-18 latency diagnosis research before further redesign

Primary source consulted: https://developers.openai.com/api/docs/guides/latency-optimization
(retrieved 2026-11-18; local HTML/text under target/recovery-latency-reference-20261118.*).
Guidance distinguishes time to first visible text from generation duration, warns
that output token caps can yield incomplete responses, and limits parallelization
to independent work. Applicable to diagnosis, not evidence that this third-party
OpenAI-compatible endpoint implements all OpenAI options. Do not silently switch
models, use provider-specific compaction, enable background continuation, trim
source coverage or raise the shared deadline on this basis.

Read-only config inspection: selected model already uses reasoning effort low;
provider timeout max_attempts=3. Existing logs lack dispatch/first-text/generation
separation, so total120s alone does not establish large input as root cause.
Added content-free stream-open and stream-drop timing, first nonempty text delay,
byte/event counts and success flag; drop covers timeout cancellation. Added paused
clock test for timing/UTF-8 byte counters. Candidate build and focused tests running.
Next controlled real-provider diagnostic only after this instrumentation passes;
no installation or original-session mutation.

### Stream timing result: generation duration, not connection stall

`target/recovery-real-provider-stream-timing-20261118/timing-evidence.json`:
stream opened ~15.67s after admission; first nonempty text 8ms later. Stream
continued for103730ms,14746bytes/5123text events, then shared deadline cancelled
before successful terminal. Original snapshot unchanged, history not executed.
This discriminates this run from slow dispatch; no claim that all runs share it.
The existing conservative8192-byte target was not obeyed by the provider. Raising
timeouts, adding parallel input folds or only reducing input is not justified by
this evidence. Next examine how to enforce/communicate bounded summary generation
without consuming truncated output or discarding source coverage; account for
reasoning-inclusive limits and provider compatibility. Do not install current candidate.
Validation:114 checkpoint tests +4 summary-stream tests passed, CLI build passed;
full preflight/CI/review remain pending implementation convergence. Overall Partial.

### 2026-11-18 prompt-format experiment scope

Inspected checkpoint prompt and provider messages: controls and source data share a
user message, but no observed evidence currently proves historical-instruction
interference. Do not label that the cause or widen role/API changes without evidence.
Existing prompt did not ask for minified JSON. Add only explicit no-indentation,
no-formatting-newline/no-Markdown output wording, retaining string whitespace,
complete schema, semantics and all validation. This is a latency experiment, not
proof that formatting caused prior timeout, and not a delivered fix. Focused tests
and build precede a measured isolated run. No model/config/timeout changes.

### Minified-format experiment rejected; bounded-provider API inventory

Actual isolated run `target/recovery-real-provider-minified-20261118` still failed:
stream opened after21.35s, first text1ms later,14004bytes/4855events over98.044s,
then deadline before normal terminal. No history calls. Prompt-only minification
has no proven acceptance benefit; removed the experiment from candidate source,
retained executable/log/result evidence. Do not retry the same experiment for green.

Inspected existing `LanguageModel::stream_decision` contract and implementations:
OpenAI Chat, Responses, Anthropic (provider lib.rs) and Gemini implement bounded
output and transport retries; default third-party implementation rejects before
dispatch. It is tool-free, System/User only and strips inherited reasoning settings.
Thus directly replacing checkpoint `stream_with_protocol` would change model
reasoning/compatibility semantics and may turn recovery into unsupported-provider
refusal. A mere output cap also leaves MaxTokens/partial JSON, which current stream
reader correctly rejects. These are design constraints for the next implementation,
not justification to weaken checks or silently use the decision API as a fallback.

Next: design a checkpoint-specific bounded-generation contract with explicit
capability/compatibility behavior, and verify useful complete output under bounds;
retain source-bound semantic coverage tests and old refusal behavior for incomplete
streams. Current candidate remains Partial/unpublished/uninstalled; source snapshots
unchanged. No unrelated tasks introduced. Existing114 checkpoint +4 stream tests and
CLI build passed before the removed prompt-only experiment; no full delivery claim.

### 2026-11-18 request-local source identifier experiment

Implementation evidence: HistorySource::stable_entry_id and bind_content_until use
`generation:{generation}:entry:{ordinal}:<64hex digest>`; repeated generated citations
can be long. Prior stream logs do not quantify citation share, so this is a measured
optimization candidate, not a proven explanation of every timeout.

Private CheckpointWire maps identifiers longer than32 bytes to request-local refN,
skipping collisions with any selected canonical ID. Short IDs stay unchanged.
Only prior checkpoint sources and current source fields are rewritten; source text
and raw-source metadata remain untouched. Decode accepts only this request's issued
IDs, expands to canonical IDs, checks existing allowed coverage, then runs original
canonical render budget. Wire aliases never become stored checkpoint identities.
Public API, schema and on-disk format unchanged. Packing remains conservatively
budgeted with original identifiers; actual wire request is also fully admitted.

Added regressions for collision avoidance, unchanged source body, unknown/raw-only
reference refusal, revoked canonical coverage, prior-checkpoint roundtrip and canonical
budget expansion. Initial112passed/2failed were old wire-format assertions; adjusted
only transport expectation, retained original-output identity check. New targeted
round running. No installation or original-session attempt; overall Partial.

### Wire-alias measured result and canonical citation budget

116 checkpoint tests +4 summary-stream tests and CLI build passed. Isolated actual
provider run `target/recovery-real-provider-wire-20261118`: request estimate dropped
258698 ->193443 tokens without removing source records; first fold completed in
~87.51s total,13613 output bytes/2717 estimated wire tokens. Canonical expansion
still exceeded render budget. Correction used only validated checkpoint (37213
request tokens including reserve) but expired with ~31.85s remaining. Thus wire
aliases improve measured request overhead and one first-fold completion, not full
acceptance or proof of robust latency.

Generation now explains canonical citation expansion (up to128bytes per reference)
and targets <=render_limit/128 total reference occurrences, with minimal sufficient
evidence and duplicate-statement consolidation. It explicitly retains distinct active
constraints, unresolved work, failures and decisions. This is generation guidance;
canonical parse/source/render gates remain authoritative and unchanged. All original
source entries still supplied.116+4 tests/build passed; changed-candidate actual
provider check running. No source merge/install/original-session attempt.

### Real history path passed and byte-rate regression proved red/green

`target/recovery-real-provider-wire-budget-20261118/result.json` actual provider
history path passed: automatic preparation first fold completed normally (~73.35s),
three history_search calls returned success, final anchored REPL prompt observed,
original snapshot hashes unchanged. Binary7de15665450356b8bfdb50ff97e8f3040fd6aa4b1f13873a81083f89f8a49dbe.
Model answer honestly notes history detail lacks independent role/time fields; this
is not broad completeness or original-host acceptance. Fresh manual Applied pending.

Added deterministic paused-clock byte-rate provider regression: fixed100s preparation
plus10ms per generated byte; eight distinct long references across required fields.
Old d97ddcb2 fails shared120s deadline; candidate passes with restored exact canonical
references, retained recent20 messages and admitted final request. Evidence:
 target/recovery-wire-red-test-20261118.json and recovery-wire-regression-20261118.log.
117 checkpoint tests +4 stream tests pass. Old-source red check restored candidate
source byte-for-byte. No installed binary or primary implementation files replaced.

Fresh manual checkpoint check now runs on a new isolated copy of the closed successful
history fixture; requires independently verified typed Applied receipt, not model
wording or Admitted. Full preflight/review/CI/merge/install/original-host acceptance
remain outstanding. Overall Partial.

### Fresh manual Applied harness observation incomplete

First manual check `target/recovery-real-provider-fresh-applied-20261118` completed
second cold-start automatic compaction (~82.28s, normal terminal) then seven actual
history_search successes. The harness's180s whole-dialog observation expired before
any compact_context call; it closed the isolated process. This is incomplete manual
acceptance, not product checkpoint timeout and not Applied. Failed artifacts retained.

A corrected harness uses600s for the entire multi-tool dialog, explicitly separate
from unchanged product120s per automatic preparation. It starts a fresh isolated
copy of the previous successful closed fixture, not the interrupted one. No model
request retry loop is introduced. Unit talos-wire-applied-r2-20261118 and exec handle
85210 track the current run; inspect this exact handle before restarting anything.
Original session and installed program untouched. Overall Partial.

### Fresh manual response requires runtime receipt verification

Corrected600s observer run `target/recovery-real-provider-fresh-applied-r2-20261118`
returned normally after5history calls and1compact_context success. Persisted tool
result proves operation0 Admitted; model reported Applied844089->92987. No independent
typed receipt was retained by that harness (only model prose), so do NOT mark Applied
verified from this artifact. Keep passed=false pending stronger evidence.

Added content-free `checkpoint_terminal` tracing only after ledger.finish_from accepts
the runtime terminal: session/generation/operation ID, origin, typed outcome. No payload,
provider errors, or transcript content logged; no outcome/control-flow change.
117 checkpoint +4 stream tests and CLI build passed. Actual receipt-focused isolated
run tracks exec81361/unit talos-wire-receipt-real-20261118. No original data touched.

### Runtime terminal exposed fresh manual preparation failure

`target/recovery-real-provider-terminal-receipt-20261118` normal final prompt,
6history calls and1compact tool success; actual runtime ledger terminal for
session4404450a... generation0 operation0 originModel is
FailedWithEstimate{before:844383,reason:Preparation}. This contradicts any assumption
that source-query success or Admitted establishes fresh Applied. First automatic
fold succeeded (~100.14s), so keep automatic/history progress separate from manual
failure. Original non-database transcript/archive snapshot hashes still match.

Extracted submitted checkpoint only into chmod600 private artifact; raw submitted
JSON5237bytes/estimated1015tokens,3references. Its translated canonical coverage,
source age and final request remain to be checked; small raw payload does not prove
it valid. Code inspection found submitted source's stored preparation deadline is
min-combined with the fresh operation deadline after multiple model/tool calls.
Possible expired-work-budget versus live-identity-lifetime confusion is a hypothesis,
not proven cause of this runtime result. Do not remove expiration checks speculatively.

Added content-free submitted-preparation diagnostic and booleans for current/submitted
source budget expiration; build/test running as talos-submitted-diag-build-20261118,
exec9535. No new real-provider run started until the instrumentation passes. Next
reproduce via focused paused-time Actor test, then check actual submitted failure
category before changing semantics. Full preflight/review/CI/delivery remain pending.

### 2026-11-18 source discovery age regression red/green

New real Actor archived/CLI source_detail -> compact_context regression advances
Tokio clock121s between reading source and submitting a valid checkpoint. Existing
implementation fails installed-checkpoint/turn success assertions; same fixture
passes after removing only submitted.source's OLD preparation deadline from the
new operation min-deadline. Logs: recovery-source-age-red/green-20261118.log.
Green asserts actual ModelCheckpointTerminal Applied, after<before, preserved
original history/head/restart, not merely model text or Admitted.

Current source bind_continuation_until already establishes this operation's120s
budget. matches_source revalidates every covered identity/generation under it;
current source deadline, authority/permission checks and atomic commit checks remain.
Submitted discovery binding is identity evidence, not a lease on old work budget.
Existing source_binding_deadline_is_not_reset_before_checkpointing continues to
verify same-operation expiration is not reset. No deadline constant increased.

This proves a matching real-path defect deterministically; original provider-run
terminal lacked the specific old-expiration diagnostic, so do not overstate exact
attribution of that earlier run. Broader checkpoint tests/build now running in
unit talos-source-age-build-20261118/exec13164, followed by typed-receipt actual
provider acceptance. Overall Partial; no merge/install/original-session launch.

### 2026-11-18 source-age fix validation and live acceptance limit

118 checkpoint tests +4 stream tests and CLI build passed. Actual provider candidate
bdd0d08132c2787a2d63df3ec1e64ffa7cc9f4c7ec3ee0da8506457b61fb438d run
`target/recovery-real-provider-source-age-20261118` completed automatic summary
normally, then47history_search calls within600s; no compact_context invocation.
Harness observation expired and closed only isolated process. No new manual terminal
exists to verify; fresh Applied remains unverified. Do not call this a product120s
checkpoint timeout, and do not rerun merely seeking green. Original snapshot hashes
unchanged. Source-age regression red/green proves defect independently; live outcome
is limited by model not submitting before observation end.

README English/Chinese now explain request-local source IDs/canonical validation,
one bounded shortening attempt, incomplete-response refusal and fresh preparation
budget after revalidated source/authority. Full pinned/locked release_preflight starts
next sequentially under existing resource limits. No merge/install or original host
restart. Entire handoff stays Partial with original acceptance and all earlier
residuals retained.

### 2026-11-18 converged source candidate full local preflight passed

Pinned Rust1.99.0/locked standard release_preflight completed exit0 in13min44.553s,
peak2GiB/swap0: workspace check/Clippy/tests and external Runtime SDK default/coding
passed. Log test totals including child-process tests5633passed/0failed/0ignored.
Evidence target/recovery-source-age-full-preflight-20261118.log and source manifest
 target/recovery-source-age-preflight-manifest-20261118.json;2686 tracked files
unchanged throughout validation.118 focused checkpoint +4 stream tests also passed.

Candidate includes reversible short wire references with canonical restore/validation,
canonical citation budget guidance, one bounded RenderLimit correction using only
validated summary, content-free stage/terminal diagnostics, and fresh manual operation
budget independent of old discovery-work deadline. Three regressions demonstrate
old failure/new success: generated render overflow, long-ID generation byte cost,
and Actor manual checkpoint after121s source reading. Existing source identity,
privacy, complete tool batches, recent history and120s per-operation budget retained.

Actual automatic/history query acceptance passed; manual fresh Applied remains
UNVERIFIED (latest600s provider run47history calls/no compact submission). No claim
that automatic success or model wording closes manual acceptance. Full independent
read-only review authorization requested for existing reviewer; earlier authorization
was #969-specific. Exact-head CI and delivery not yet performed; no source merge,
install, PATH/config change or original host launch. Overall Partial.

### 2026-11-18 request-directed eligible-source search implementation (local, Partial)

Within the same recovery emergency scope, added source_search for case-sensitive
literal matching of safe checkpoint-source projections, five bounded previews per
page, with snapshot/query-bound cursors. Source_detail can now project a requested
late source from the runtime-owned exact message view before tool delivery. This
does not translate durable-history query tokens into compact authority. Existing
source identity, prefix/generation, permission/cancellation and durable identity
delivery checks remain in force; raw message views are not retained by this step.

Reserve16KiB of the existing2MiB text budget for request-directed responses; the
initial detail cache uses the remainder. Preparation is cooperative and bounded to
2s/16 history calls; unsupported or over-budget projections explicitly fail. Search
covers existing safe bounded text projections, not hidden fields or unlimited text.
No background continuation, storage/schema changes, original-data writes or install.

New late-source regression failed on91ce6f25 (invalid discovery input), then passed
after implementation; it locates/reads raw-260 beyond the initial cache, checks its
canonical mapping and retained text budget. Added pagination, stale/query-changed
cursor, rediscovery, revoked-permission-at-delivery, generation/prefix and refresh
regressions. Real archived CLI Actor fixture now searches eligible sources, reads
one and submits; it retains the121s source-age case and typed Applied/raw/restart
assertions. First broader run114passed/1failed only at old terminal-record count
(8 actual vs7 expected after adding a provider round); expectation updated to8.
Initial test compile typo and failing run logs retained. Agent full tests running
in target/recovery-source-read-agent-tests-20261118.log. Full preflight, native CI,
independent review and real-provider fresh Applied remain required for this local
follow-up. PR974 head91ce6f25 CI cannot validate these uncommitted edits.

### 2026-11-18 source-navigation validation and remaining actual acceptance

Standard pinned/locked release_preflight passed5644/0/0 (including child tests),
agent767/0/0, check/Clippy and external SDK/SQLite fixtures passed. Source manifest
target/recovery-source-navigation-preflight-manifest-20261118.json records2717
unchanged files; log target/recovery-source-navigation-preflight-20261118.log.
CLI SHA256 df6f79bc271b0045e3c725ff008fbab487dd594a04fcf428d42bc81434b3abea.
Old head91ce6f25 CI44246634552 all eight checks passed, not evidence for this follow-up.

Actual provider navigation run returned normally after12 successful history calls
and0compact calls: model declined for missing latest-state citations. Read-only
fixture audit proves cited88%/handoff facts occur at head records118-120 with only
one subsequent fixture user; these facts are retained-tail, not eligible-prefix.
A corrected instruction on the same binary then made27history calls/0compact in
the600s observer window; harness closed the isolated host. Neither run proves fresh
Applied or semantic preservation. Product120s checkpoint budget was not reached by
a submission. Both preserve snapshots, transcript prefixes and archive hashes.

Evidence: target/recovery-real-provider-source-navigation-20261118/result.json,
target/recovery-real-provider-prefix-aware-20261118/result.json, and
target/recovery-source-navigation-retained-tail-audit-20261118.json. Cancelled turn
was not committed to TLOG; audit explicitly rejects the older fixture's last user
as acceptance evidence. Do not mistake harness exit0 or old transcript for success.

Source-navigation implementation is ready for exact-head review/CI, with actual
manual Applied, semantic audit, original-host acceptance and versioned delivery
remaining open. New review scope authorization is pending; no merge/install.
Full original handoff residuals, including low-approval causal diagnosis, missing
Nov11 evidence and intermittent highlighter attribution, remain Partial.
