# ADR-104: Selected CLI Session History Recovery

Status: Proposed — emergency implementation under local validation; independent review pending.
Owner: [Session archive / recovery emergency](../tasks/session-archive-recovery-emergency.md).

## Context

CLI `set_persistence` selects an ordinary Session, while history preparation previously required
an embedded DurableSession. Opening an embedded session creates another UUID and transcript;
that cannot recover the selected CLI history. CLI also appends provider diagnostics between
model responses, invalidating a file-version-only history seal even when the indexed prefix
is unchanged. Legacy ordinary transcripts do not enforce embedded persistence redaction.

## Decision

Add the host-only `DurableSession::bind_existing_history(Session)` constructor for an existing
UUID-named TLOG. It retains the Session ID, path, write lock and selected branch. It writes an
identity binding in `history-bindings.sqlite` beside the selected transcript, separate from
embedded logical-ID bindings. It never copies, truncates, normalizes or relabels the transcript.
Reopening requires a host-selected Session again, not an external ID converted into a path.
The Actor caches this owner and its bounded preparation progress for the selected persistence
configuration. Binding work runs on a blocking worker. The host retains its execution lease.

The existing history pipeline still validates parent chains, seals the captured byte prefix,
indexes permitted projections, and validates permission generation and source freshness before
and after disclosure. A binding does not certify a corrupt legacy chain. Native SQLite failures
remain inside the existing unwind/error boundary. Identity mismatch and symlink transcript or
binding-database paths are refused. Concurrent hostile filesystem replacement is not authorized
by this API; source verification remains mandatory.

Before history tool execution, a changed file version may renew its seal only by verifying the
original prefix against its original digest. Verification has a two-second deadline, checks
cancellation between bounded reads, and never extends the indexed cutoff, changes index scope,
or grants fresh references. Prefix mutation fails closed. Post-query source checks remain in
place so a mutation or append during disclosure cannot borrow an earlier validation.

User/assistant search projections apply the existing credential-line redaction even for legacy
ordinary writes. Assistant tool syntax, hidden metadata and reasoning remain excluded; tool
results still require their existing receipt. This is not permission to infer receipts for old
tool output. The original file is preserved.

## Constraints and consequences

- Hard: preserve session identity and original bytes; no implicit embedded replacement session.
- Hard: binding, digest and parent checks do not replace current model-disclosure permission.
- Hard: no default background continuation change or live-session archive rewrite.
- Soft: prefix revalidation can repeat after every provider diagnostic append; work is bounded
  by deadline and page reads, but large histories may still return unavailable. Persistently
  resumable query-time renewal remains a performance follow-up, not permission to skip hashing.
- Assumption: the host execution lease excludes cooperating writers; old binaries and hostile
  external filesystem changes remain outside cooperative ownership and must fail source checks.
- API addition is additive; no existing constructor, transcript format or call signature changes.
- Legacy JSONL history bindings, broken-parent archives and authenticated archive reconstruction
  are not made valid by this change. Their recovery remains in the emergency owner.

## Validation

Real Actor/provider tests cover both persistence configurations through discover, search,
forged-reference refusal, detail, and provider continuation. The CLI variant retains credential
text on disk to prove projection redaction rather than merely testing an already-redacted file.
Storage tests cover identity, unchanged bytes, reopening authority, and symlink refusal. Prefix
renewal tests cover append without cutoff extension and rejection of changed original bytes.
Full workspace preflight, independent review and running-host acceptance are still required.

## Legacy archive recovery extension (local candidate)

The original linear-index path remains strict. A failed, non-resumable preparation may instead
construct a separate archive-evidence query source within the same ten-second preparation budget.
This requires at least one exact legacy placeholder and an overlapping archived segment. The
inspector requires identical original record bytes for shared IDs, a unique adjacency order,
no conflicting or duplicate IDs, no missing/forward parent, and no candidate-ID overlap with
another live UUID head. Original non-adjacent parents are preserved as legacy evidence; they are
never relabeled into a valid linear chain. This supersedes the earlier statement that all archive
recovery is unimplemented; cases failing these requirements still refuse recovery.

The host-selected current Session establishes the root. Exact overlap/adjacency and supporting
file versions establish the candidate extension; directory order, timestamps and shared
`chain.tlog` do not. This is a separate provenance contract requiring independent review, not
removal of the original linear-source check. The query corpus remains in host memory, bounded by
the inspector's input/record limits. SQLite stores its owner-scope revocation/completion ledger;
it does not supply archive result content or forged source offsets. The current head is sealed,
and every supporting file version is checked before and after disclosure. Query references and
cursors use the existing per-discovery capability secret, permission generation and scope gates.

Archive queries currently disclose projected user/assistant text only. They use case-insensitive
substring matching, chronological pages of at most 20 hits, 512-byte snippets, 8192-byte details,
and the existing serialized response ceiling. No legacy tool receipt is inferred. Supporting-file
mutation, revoked scope, cancellation and invalid/replayed cursors refuse disclosure. Head-only
append may renew the original prefix seal; it never expands that execution's archive corpus.

CLI startup and fenced transitions detect the exact legacy count placeholder during their normal
paged scan and reconstruct original messages through the same host decoder, including tool pairs
cut by compaction. Startup acquires an execution lease for archive reconstruction; transitions
retain their existing fence. All source versions are checked again before publishing the decoded
history. Missing/corrupt/ambiguous archives cause an error, not silent recent-tail restoration.
No transcript is rewritten. Retained Actor history is distinct from provider request size: existing
request-budget admission/checkpoint planning remains mandatory. A count marker is never submitted
as a semantic summary. A real model checkpoint still requires source references and an Actor
Applied outcome before the projection is installed.

### Follow-up compatibility and containment boundaries

Automatic archival is removed from both CLI completion and the optional Actor startup
path. `AppServerSession::set_session_paths` retains path tracking and invalidates private
replay when the path changes; it no longer triggers archival. Turn persistence still
requires `set_persistence` or `set_durable_persistence`.
This preserves the API signature while intentionally disabling destructive automatic
head trimming. Embedders must not rely on automatic disk compaction from this hook.

Archived forks are refused before child publication until child-owned archive provenance
is implemented. A current-head-only fork is insufficient and would also introduce shared
IDs into another live head. Restored history uses the existing source-bound automatic
checkpoint folding path under pressure. Each summary request is admitted independently;
validated installation precedes continuation. Insufficient fixed/recent-context capacity
or rejected summaries still pause before dispatch. A real archive/restart/Actor regression
proves multiple summary calls, Applied, continuation with Context and retained recent
originals, and restart preservation; its test model does not prove real-provider summary
quality. Supporting-source checks also re-enumerate the bounded archive/head inventory;
new or removed candidate paths revoke previously prepared evidence.


### Archived fork compatibility and receipt migration

ADR-037 section 6 assumes a compacted head is sufficient to make its copied child
independent. Legacy count placeholders do not satisfy that assumption: original context
now comes from sibling archives. Direct `Session::fork` therefore refuses these
histories before in-memory branch mutation. Startup, TUI and REPL instead use the
receipt-backed publisher described below, preserving original parent IDs and
establishing durable child provenance. The direct-API restriction alone is only
containment; full fork support depends on publication and recovery evidence.

The current SQLite fork index records only parent/child IDs, the fork entry ID and a
timestamp; it contains no content digest or immutable source cutoff. It cannot authorize
an exception to other-live-head overlap rejection. Moving children into arbitrary nested
directories would also break SessionManager's existing workspace discovery contract.
A full implementation must retain workspace discovery, independently owned historical
bytes after parent deletion, original identities, bounded validation, and unpublished
child cleanup across every startup/interactive/restart path. Independent review must
validate the source-ownership evidence before any overlap exception is enabled.


### Windows live-history lease correction (independent review follow-up)

The independent review of `39d2f327` found that a whole-turn Windows read lease
conflicts with CLI provider diagnostics and that retaining all archive inventory
leases blocks unrelated live sessions. This supersedes the whole-turn retention
assumption of ADR-096 for prepared History, not its content-stability requirement.

Prepared Windows History retains only the storage-issued source and original
prefix expectation between operations. Each synchronous query/freshness operation
recomputes the captured prefix under a fresh restrictive handle. It keeps the
original cutoff and scope, never includes later diagnostic bytes, and drops the
lease before returning to provider/tool awaits. A bounded try-lock loop on the
selected Session's shared write mutex serializes these reads with its diagnostics;
the read lease is dropped before that mutex. Cancellation or contention exhaustion
returns unavailable. `DurableRecoverySource::try_lock_for_recovery_read` is an
additive host API: it grants no disclosure authority and does not coordinate
independently reopened owners or external writers. Existing no-follow/local-volume,
permission, digest and generation checks remain mandatory.

Archive inventory evidence stores inert Windows identity plus the exact stored-byte
SHA-256 of every inspected source, including excluded heads. On reuse it reacquires
restrictive handles, verifies identity and full content within the existing total
byte limit/deadline, and rechecks the inventory while those handles remain held.
Handles live only through validation, not through the model turn. A changed other
head revokes old evidence; it is not silently accepted as still disjoint. Metadata
equality without a fresh byte digest is explicitly insufficient, including equal-
length mutations. Concurrent external access may still make a bounded operation
unavailable; this is not a claim of cross-process writer scheduling.

Soft tradeoff: Windows queries repeat content verification instead of retaining a
turn-long lease. This can consume the existing two-second query budget on large
histories; no deadline increase or permission relaxation is introduced. Native
Windows multi-response Actor, concurrent-session, mutation and cancellation tests
and fresh independent review are required before delivery. Local cross-compilation
alone does not satisfy that gate.


Short leases alone do not protect sibling writers: a write can start during
archive validation. Built-in JSONL/TLOG persistence therefore waits on Windows
`ERROR_SHARING_VIOLATION` at append-handle acquisition and final atomic rename,
with a five-second total wait per acquisition/publication operation and at most
10 ms between attempts. Byte writes, flushes, complete transactions and other I/O
errors are never replayed. The original parent cache is published only after a
successful append. This covers independently reopened owners without requiring an
in-process registry, while retaining restrictive recovery reads. A persistently
held conflicting handle still returns the original error after the bound; this is
not an unlimited availability guarantee. Soft tradeoff: persistence can wait up
to five seconds for an OS sharing conflict, instead of immediately failing.

The native regressions synchronize on an actual sharing-conflict result before
releasing the recovery handle. They cover both append encodings and durable atomic
commit, checking unchanged bytes before publication, one resulting record/turn,
reopen and idempotent retry. Permanent conflicts and non-sharing errors have
separate bounds. Native Windows execution remains required; compiled tests alone
do not certify the kernel's rename return behavior.

Source revalidation now carries one deadline through archive supporting-source
checks, prefix reads, query reads and final availability checks within each query
or availability operation. Expiry cannot silently start another two-second phase.

### Atomic publication acquisition and final path binding

Native Windows run 43141073000 at 32e4d227 returned OS code 5 from the
held-lease durable publication regression. Sharing-only rename retry therefore
cannot coordinate this operation. Built-in atomic TLOG/JSONL publication now
acquires an existing target's write handle without creating, truncating or writing
it, under the same bounded sharing-conflict retry. It rejects final reparse points,
nonregular objects and nonlocal storage. Default delete sharing lets publication
retain this handle through rename and excludes newly acquired recovery leases.
If a target appears after an initial NotFound, a failed access-denied rename may
retry only after acquiring that target, within the original retry deadline.
General AccessDenied errors are not made transient; transactions and byte writes
are never replayed. Host single-writer ownership remains required.

Soft compatibility restriction: on Windows, replacing an existing transcript now
requires write access to that file even when directory delete/replace permissions
alone would have permitted rename. Readonly/write-denied targets fail conservatively
with their original bytes and unpublished temporary retained. This restriction avoids
new unsafe Windows publication code and broad permission-error retries. A future
reviewed native publication primitive may replace it if directory-only replacement
must be supported. Native held-lease acquisition, publication, readonly refusal and
durable reopen/idempotency regressions are required; MSVC compilation alone is not
native acceptance.

Archive validation also rechecks inventory and each selected path's captured file
version after all source reads. A Unix real-compaction regression first demonstrated
that identical-byte atomic replacement in that window incorrectly retained evidence;
it now revokes it. Supporting-source checks still exclude the selected head's version,
which remains protected by its original prefix seal. This is not a multi-file atomic
snapshot or a guarantee against changes after return: ownership and disclosure
before/after checks remain mandatory. All checks share the caller's deadline.

## Zero-retention source bridge and old-index migration (local candidate)

For new UUID TLOG keep0 compactions, a private host locator may replace the overlap
requirement for the exact first marker: it binds marker raw digest, synced archive
name/digest and the original logical tail before head replacement. Recovery still
validates the full bounded source chain, original tail, acyclic marker transitions
and strict appended suffix. It does not authorize cross-session inheritance. Legacy
keep0 without this evidence remains unavailable. Physical publication intents and
full archived-fork lineage are separate mechanisms implemented locally below;
locator validation alone does not provide them.

History preparation must classify markers even when a persisted old index cursor
already reports completion. The sealed first page is rechecked in each new
preparation; the classification flag is not persisted. A marker never constitutes
an empty complete linear history. Actual query and old-index regression fixtures
cover archive presence and absence. Source locator limits and cancellation behavior
are specified in the runtime SDK contract. UUID reuse/deletion lifecycle is
implemented below; native Windows crash-durability acceptance remains pending. This additive classification API and
locator implementation do not change the ADR's Proposed status.

## Physical archive ownership before publication (unreleased candidate)

New built-in UUID TLOG compactions use `s-owned-<UUID>.tlog[.zst]`. Before the archive
name becomes visible, the host commits an immutable archive-name/owner/digest intent
to private history bindings. A reserved owned name cannot fall back to legacy when
its DB/table/row is absent or invalid. Names remain reserved after head deletion;
intents are not removed by ordinary binding cleanup. This is physical ownership,
not a fork receipt or authority to import records without overlap/marker proof.

Recovery validates all owned files in the bounded inventory, excludes foreign-owned
archives from overlap closure, and rechecks both selected and excluded ownership
before/after evidence validation. Legacy `s-<UUID>` remains on its existing proof
path. The private DB has the same host custody trust boundary as durable bindings.
SQLite FULL commit precedes publication; no new power-loss/Windows directory-sync
guarantee is claimed. Unsupported writers must not be allowed to compact future
materialized children into unowned legacy archives: a format/version gate is still
required for archived-fork publication. The local envelope, UUID incarnation and
inherited receipt/origin implementation below supplies these mechanisms; supported
old-writer compatibility and native acceptance remain separate validation gates.

### Archived-child downgrade constraint (historical design, implemented locally below)

Code inspection and independent review establish that the current legacy whole-file
TLOG decoder ignores the version in a TALOS header. Raising that version alone is
therefore not a supported-writer gate. The proposed receipt-bound child format must
fail that decoder before compaction can publish any legacy archive. A fixed binary
non-UTF8 envelope is a candidate: the old decoder rejects before reading records,
while the retained original record bytes remain unchanged. This historical proposal
is implemented by the local receipt-bound envelope described below; it is not a
claim of release, old-writer compatibility or native acceptance.

Every new-writer atomic finalize, retained-suffix rewrite, archive and nested fork
must preserve the envelope; stripping it on read and rewriting a v1 head reopens the
same contamination hole. An envelope is format identification only: acceptance and
origin exceptions still require the private inheritance receipt and incarnation.
Direct store append and older cached-tip writers may append without examining the
header, so the gate cannot be described as rejecting every old write. Its required
property is that those appended bytes cannot pass old compaction and become an
unowned legacy archive. Discovery refusal must remain explicit, never an empty
successful restoration. Validate exact supported old versions and the complete
publication lifecycle before release enablement of this format.

### Legacy JSONL append boundary and migration (unreleased candidate)

Exact raw-prefix forks preserve a valid JSONL final record without a newline.
Appending serialized JSON directly would join two records and lose them on reopen.
The built-in JsonlSessionStore therefore inspects the final byte through the same
read+append handle and inserts a newline only when a nonempty file lacks one.
It never rewrites the inherited prefix. Read/seek failure propagates before writing;
a separate read handle or unconditional separator is not a safe substitute.

Compatibility change: direct JsonlSessionStore append callers now require read as
well as append permission on the file. Normal Session callers already read the tip.
Before upgrading, direct-store hosts must grant their writer read access or retain
a custom SessionStore with an explicitly supported append-only contract. Do not
silently broaden ACLs. This is an unreleased pre-1.0 candidate behavior change;
release/migration review remains required. Existing write leases still own concurrency.
If separator insertion succeeds but the subsequent body write fails, the original
record remains intact with a trailing newline; atomic append is not newly promised.

### Archived fork identity retirement boundary (implemented locally; delivery pending)

Independent review rejected first-record ID/hash as an incarnation proof: after
deleting A, a new head with the same UUID and original first record but a new suffix
passes that check and incorrectly receives the old origin identity. Binding more
prefix bytes cannot distinguish an external full copy followed by append either.

The publication implementation must permanently reserve UUID-to-incarnation mappings
in host-private custody. Identity states are Active/Retired; a reserved or retired
UUID cannot be reassigned. Managed deletion records retirement before removing any
head; a crash leaving the head behind must not permit reopen/append/fork/compaction.
Retirement does not revoke a child receipt containing a validated inherited prefix.
Intent-owned failed child publication also retains its UUID reservation.
SessionManager currently routes delete_session and rollback_session_artifacts through
remove_owned_session_artifacts; retirement must cover that shared boundary without
preventing retryable cleanup. Discovery and independently constructed Session writers
need the same check; indexing alone is insufficient.

The host-private database is the existing custody boundary. Arbitrary same-UID
external deletion/replacement or copying of both private custody and transcript bytes
is outside it; neither a digest nor an envelope authenticates such external activity.
A first-record commitment may detect accidental replacement, but must not be described
as proving incarnation. Tests must cover managed retirement before deletion, process
death leaving a head, attempted UUID reuse, and child recovery after parent retirement.
This corrects the original design. The local candidate now implements the identity
ledger and retirement checks in identity_custody.rs; receipt-backed publication
uses that boundary. Implementation evidence and remaining gates are recorded in
the recovery owner; this decision is not a delivery claim.

### Identity operation lease candidate

Built-in UUID transcript operations now create/open permanent sibling
.identity-locks/<canonical UUID> files, shared across TLOG/JSONL. This deliberately
requires a writable parent even for the first read: inability to create/acquire
the identity anchor fails unavailable, never bypasses retirement. Native std File
try_lock/try_lock_shared are nonblocking; busy operations must retry, never assume
retirement committed. Reserve/retire take exclusive ownership; ordinary read/write
takes shared ownership before state lookup, retained through operation completion.
Cleanup ordering is runtime lease then identity lease. Nested TLOG header/rewrite
reads clone the existing Arc-backed guard rather than reacquiring. No lock file is
unlinked by cleanup. This candidate still needs cross-process/native Windows and
full delivery-fence validation; long-lived History must not hold these operation
locks indefinitely. Receipt publication separately uses this identity boundary.

### Local Actor publication integration (2026-11-13)

Add host-only SessionOp::ForkHistory with fresh child UUID, final branch UUID,
optional original entry cutoff and an in-process receipt channel. This is an
additive enum variant requiring exhaustive match consumers to add handling; wire
requests without a receipt channel do not publish. The Actor admits only idle
requests, reuses fork-worker exclusion/cancellation, and retains its runtime lease
inside the blocking publisher. Success returns the original tail ID; the host
still owns runtime/index/lineage metadata and activation. Existing ForkDurable
semantics remain distinct. CLI/TUI callers must migrate to this operation for an
already-running source instead of reacquiring its execution lock.

### Receipt-bound branch reopening and API migration (2026-11-13)

A published child's immutable receipt is authoritative for its branch UUID.
SessionManager reopening restores that UUID before loading entries, including
materialized and repeatedly compacted heads; it does not invent a new branch.
The in-memory branch root representation remains the existing Session constructor
representation, not a recovered original-entry root. Receipt-bearing sessions must
not use Session::fork to replace this branch only in memory. That call now returns
an explicit error; SDK callers migrate to Session::publish_history_fork with a fresh
child, and running hosts use SessionOp::ForkHistory. Legacy non-receipt fork behavior
is retained. Branch restoration alone does not establish runtime-identity or
lineage metadata reconciliation; the separate startup replay below provides those
operations with its documented bounds and remaining acceptance requirements.

### Publication-time runtime identity snapshot (local candidate)

New host captures write version2 inheritance receipts containing the exact committed
source provider/model/variant or explicit absence. Version1 remains readable for
history, but cannot authorize runtime identity reconstruction. Older version1-only
readers refuse version2 rather than silently infer a runtime identity. This extends
the receipt payload, not the physical envelope encoding. No mutable parent settings
or current host defaults substitute for a missing snapshot after a crash.

Child runtime recovery requires its runtime lease and validated history evidence.
It initializes only absent generation-zero identity, rejects a conflicting initial
identity, and preserves later activation state. Recovery can write sidecar schema
or identity before a final validation error; errors retain all artifacts and retry
idempotently. Snapshot and restoration share the caller deadline for SQLite VM
work and use nonblocking locks; filesystem calls remain cooperative. Startup now
preserves interrupted pending heads, reconciles the ordinary index, replays
published version2 runtime/lineage metadata, then maintains orphan sidecars.
Published replay uses bounded index opening and idempotent lineage registration;
ordinary transcript indexing remains outside that recovery deadline. Missing-head
pending artifacts are retained, version1 cannot restore runtime identity, and busy
or invalid candidates may defer work. Full host/native acceptance remains open.

### Host runtime ownership during restoration

REPL and initial TUI startup acquire execution ownership before identity/history
preparation and transfer it to the Actor. Owned history restoration validates the
exact transcript and avoids duplicate lease acquisition on keep0 archive fallback.
TUI resume/new and both ordinary and receipt-backed fork preparation retain ownership until
Actor adoption. Shared CLI fork metadata registration holds child ownership until
return, after which runtime construction reacquires before further preparation.
Same-session activation replacement reacquires after old Actor retirement and
validates durable generation and the complete activation before preparing the
replacement. Competition refuses preparation; no continuous old-to-new transfer
is claimed. Ordinary TUI fork acquires child ownership before installation and
transfers it to the Actor after construction. Before transfer, explicit failures
release ownership and perform one rollback; cancellation uses the staging guard.
After transfer, cancellation and commit/publication failure retain a discoverable
child for recovery. Source-side ordinary fork fencing uses the Actor capture described below; full
native terminal acceptance remains open. These cooperative leases do not fence older or nonparticipating writers.

### Ordinary active-host fork capture and CLI integration

The additive OrdinaryForkSnapshot/capture_ordinary_fork storage API captures raw
TLOG/JSONL prefix, original tail and optional committed runtime identity under an
exact source lease. It bounds raw bytes to64MiB, shares the caller deadline and
cancellation through scan completion, and rejects runtime-state drift, unfinished
activation, archive placeholders and receipt envelopes. It preserves incomplete-tail
compatibility and never converts JSONL into TLOG. This is preparation only: it does
not publish a child or authorize archive inheritance. No existing public signature
is removed; current host staging remains available during migration.

Actor-idle capture under existing ForkWork exclusion supplies the fixed snapshot
and identity used for child publication. A runtime lease alone does not exclude
writes by its own Actor; the operation gate is required in addition to the lease.

Actor integration now exposes host-only CaptureOrdinaryFork with an in-process
receipt carrying raw prefix, original tail ID and optional provider/model/variant.
It reuses the same ForkWork exclusion as receipt/durable publication, so turns,
compaction and other fork work cannot overlap capture; interrupt, receiver loss,
admission closure and shutdown cancel the worker while retaining its source lease
until completion. This adds a SessionOp variant: downstream exhaustive matches must
handle CaptureOrdinaryFork (or their existing unsupported-operation fallback).
The wire-deserialized operation has no channel and cannot authorize capture.
TUI and REPL obtain this Actor capture before staging an ordinary child; later
parent reads do not replace the captured bytes or identity. Publication and runtime
installation retain child ownership and preserve a committed child on post-transfer
failure. Local production PTY regressions cover ordinary and archived fork, model
switch and cold child resume; native-platform and delivery gates remain separate.
