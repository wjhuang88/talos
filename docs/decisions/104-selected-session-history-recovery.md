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


### Archived fork compatibility decision still outstanding

ADR-037 section 6 assumes a compacted head is sufficient to make its copied child
independent. Legacy count placeholders do not satisfy that assumption: original context
now comes from sibling archives. Startup, TUI staging and the shared `Session::fork`
API (including the REPL path) therefore refuse these histories pending a replacement
provenance contract. The shared guard runs before in-memory branch mutation. This is a
temporary behavior restriction, not complete fork support or a reinterpretation of
preserved parent IDs.

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
