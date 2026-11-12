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
