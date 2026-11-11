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

Implemented selected-session binding and Actor wiring per proposed ADR-104. The existing
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
bounded case-insensitive substring scan, not the ordinary-head FTS ranking path. See ADR-104.

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
