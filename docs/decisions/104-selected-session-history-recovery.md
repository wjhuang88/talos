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
