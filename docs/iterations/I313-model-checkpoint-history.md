# Iteration I313: Actor Model Checkpoints And Bounded History Recovery

> Document status: Review / Partial (#819 merged; stable stage candidate, full implementation remains unmerged)
> Published plan date: 2026-11-01
> Baseline rule: preserve all T5 and Issue #390 acceptance; preparation is not implementation.
> Planned objective: complete handoff T5 and remaining Issue #390 model-directed checkpoint/recovery acceptance, consuming I312 without creating another compaction or history authority.
> MVP deliverable: a model discovers stable compact_context/history_search tools, voluntarily commits a structured checkpoint at an Actor-safe boundary, then recovers an omitted original current-branch fact and continues correctly after restart/fork.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 handoff closure session |
| Work Slice | Handoff T5: Actor model checkpoints, stable tool discovery, bounded branch-scoped authoritative history recovery and persistence/privacy regressions |
| Claimed At | 2026-11-01 |
| Source Issue | #390 (remaining T5 acceptance) |
| Governance Claim PR | #819 |
| Authorization Mode | Independent review |
| Authorization Evidence | Exact-head independent governance/API/privacy review, CI, validators and CAS required before activation |
| Implementation PR | #837 (draft, Partial stage) |
| Last Updated | 2026-11-02 |
| Handoff / Release Condition | Effective target-branch claim and merged T4 completion evidence before implementation |

The draft does not reserve ownership. Finalized Claimed/Active state becomes
effective only on target-branch merge; no implementation branch before that gate.

## Preconditions and inventory

I312 implementation #818 merged as `65500a86931cfbadf8bef5e97e45321c20b62e7a`;
this governance batch records its owner-first closure. I313 was unused in the
fresh target inventory. No T5 implementation is authorized before this finalized
claim merges. The only open implementation PR at selection is independent #808.
I293/I294 search and I310 Gemini retain independent Active ownership; I277 and
I302 retain Review; I249 remains deferred Planned; I164 remains Paused. Recheck
all target owner declarations and open PRs immediately before activation.

## Ownership

MEM-005 remains policy owner; MEM-003 supplies the unified checkpoint pipeline.
Existing Session persistence/index owns additive recovery receipts/index changes.
MEM-007/#143 retains pre-entry tool filtering; ARCH-006 stable-prefix invariants
remain unchanged. T1 and T6 remain later queue items. No release, paid probe,
second database, vector engine, transcript rewrite or new raw-private-data store.

## Implementation contract

1. Stable registered compact_context and history_search definitions when enabled.
   Advisory pressure metadata is dynamic, content-free and based on existing
   whole-request admission/reserves. No dynamic tool-list mutation at thresholds.
2. compact_context accepts the existing structured bounded checkpoint schema plus
   runtime snapshot identity. Runtime computes eligible prefix/recent window.
   Model cannot name arbitrary deletion ranges or cover its own live tool batch.
3. Reuse the Actor operation owner/ledger. Tool returns Admitted with operation ID;
   complete paired batch reaches the installation gate. Final Applied/Rejected/
   Cancelled/Stale receipt enters next request's dynamic context before dispatch,
   or terminal Session status if no continuation. Never await an Actor that awaits
   the same tool batch, nor interpret Admitted as success.
4. history_search has discovery/search/detail modes and opaque snapshot/cursor/refs.
   Scope binds generation/session/branch/revision/cutoff and the immutable
   covered-prefix digest. Append-only changes after cutoff are compatible and
   retain the complete live batch; generation/branch/permission changes or covered
   prefix mutation invalidate it. Every page/detail and result delivery revalidates
   scope; installation recomputes the current full-request budget. Shared ancestors are eligible; siblings are
   excluded before query LIMIT. Same IDs or copied TLOG are not branch authority.
5. Extend existing SessionIndex with scoped query/result types and authorized
   projection FTS/source mapping. Raw FTS matching/rank/count cannot influence
   model results. Global BM25 statistics across other sessions must not influence
   scoped ranking either: use deterministic authorized-source order or scoring
   computed solely inside the authorized scope. Human search API remains intact. Re-read authoritative entries
   and verify eligibility receipts before returning projected bounded content.
6. Reuse stored text when model recovery projection exactly matches persisted text:
   store only a versioned receipt binding entry/call/tool/projection version/digest.
   If different, persist bounded recovery projection only when existing persistence
   policy permits. Bind receipts to the final post-policy stored representation, never an earlier
   unfiltered projection. A denied/omitted projection cannot receive a recovery
   authorization receipt. Atomic source/receipt commit, unknown-version refusal, no hidden
   inputs/reasoning/raw UI copy. Receipt proves runtime provenance, not fact truth
   or cryptographic protection from an arbitrary host rewriting the source store.
7. Host-owned maintenance updates/rebuilds the existing index within authorized
   Session storage. Read-only model query cannot implicitly authorize writes.
   Missing/corrupt index reports typed unavailable until bounded host repair.
8. Explicit initial bounds to validate: query 1024 UTF-8 bytes; discovery/search
   page <=20 entries; snippet <=512 bytes; detail <=8192 bytes; response <=16384
   bytes; source scan <=8192 entries / 2 MiB; operation deadline 2 seconds for
   query/detail and 10 seconds for host rebuild. Reuse stricter source limits.
   SQLite progress/interrupt and bounded record reads must enforce work, not merely
   timeout a still-running blocking task. Measurements may tighten/change these
   constants with recorded evidence, without dropping supported recovery behavior.
   Entry/byte limits apply per incremental scan page, not to total session length.
   Host rebuild persists a resumable cursor and continues over bounded operations;
   incomplete coverage reports IndexBuilding/Unavailable rather than complete
   search results. Never use read_entries() full materialization underneath a
   nominal page bound. Validate long sessions exceeding each single-page limit.
9. A denied/omitted persistence projection remains denied. Legacy unverified tool
   facts may be explicitly unavailable, but newly supported tool facts must recover
   after restart under permitted policy without reexecuting the original tool.
   Document Runtime and legacy CLI storage support separately: verify each path
   has an atomic record/receipt commit before issuing durable authority; paths
   without it return explicit Unavailable. At least the supported Runtime/Session
   path must recover newly supported tool facts across restart; unavailable-only
   behavior cannot close this requirement.
10. Diagnostics contain pressure/count/size/reclaim/trigger/recovery-call metrics,
    never checkpoint/user content. Detail is explicit and respects the same scope.

The model selects a natural task boundary; a compact_context call itself is
protocol ToolUse, not EndTurn. Installation waits for the complete paired batch.
Preserve distinct natural-boundary and unsafe-boundary regression scenarios.

## Acceptance and validation

The following allocation preserves all 30 numbered requirements from Issue #390;
using the allocation below as the published baseline, not implementation evidence.
Also cover bounded full detail; raw-only FTS hits yielding no model ref/count/rank;
policy-denied persistence; manual/model contention; generation reset/cancel/stale
receipt; index corruption/rebuild; same-ID sibling/clone refusal; source mismatch;
unknown extension version; cancellation that actually stops database/source work.
Explicitly cover discovery → ordinary tool-call append → successful installation,
and covered-prefix mutation → Stale; cross-session corpus changes cannot alter
scoped search results/ranking. Test post-policy omission cannot mint a valid receipt.

Representative deterministic coding/research scenarios must assert omitted-fact
recovery, no repetition of completed/known-failed work, facts distinct from
assumptions, and correct next step after checkpoint A→B and restart/fork. Record
request budgets, checkpoint and recovery counts. Do not claim model quality from
fixed responses. Exercise real Runtime/CLI composition and discovery, not only
internal helper calls.

Use Rust 1.99.0, locked tests/Clippy, standard release_preflight.sh including SDK
modes, both governance validators, independent API/privacy/storage review,
exact-head platform CI and merge CAS. Completion requires merged implementation
SHA evidence in owner then derived views; Issue #390 closes only after all rows
have current evidence. Existing T4 evidence is baseline, not proof of new tools.

## Published Issue #390 acceptance allocation

| Row | Published requirement | Allocation / evidence still needed |
|---|---|---|
| 1 | Context below advisory threshold: no pressure metadata; stable checkpoint tool definition unchanged. | T5 implementation and integration evidence required |
| 2 | Crossing advisory threshold: pressure metadata appears only in dynamic request context. | T5 implementation and integration evidence required |
| 3 | Model elects not to compact at the first eligible boundary and ordinary continuation still fits. | T5 implementation and integration evidence required |
| 4 | Model voluntarily checkpoints at a natural `EndTurn` boundary. | T5 implementation and integration evidence required |
| 5 | Checkpoint preserves last configured recent turns verbatim. | T4 baseline; T5 must rerun through model-directed/recovery composition |
| 6 | Checkpoint attempt during an unsafe active tool-use boundary is deferred/rejected according to policy. | T5 implementation and integration evidence required |
| 7 | Valid structured checkpoint produces deterministic canonical rendering. | T4 baseline; T5 must rerun through model-directed/recovery composition |
| 8 | Same structured input + same rendering version produces byte-identical checkpoint text. | T4 baseline; T5 must rerun through model-directed/recovery composition |
| 9 | Oversized checkpoint is rejected without changing active projection. | T4 baseline; T5 must rerun through model-directed/recovery composition |
| 10 | Insufficient token reclamation is rejected or handled by the documented bounded policy. | T4 baseline; T5 must rerun through model-directed/recovery composition |
| 11 | Checkpoint retains completed/in-progress/pending state without repeating completed work in the continuation fixture. | T4 baseline; T5 must rerun through model-directed/recovery composition |
| 12 | Failed-attempt fixture prevents a known-failed route from being the only surviving context. | T4 baseline; T5 must rerun through model-directed/recovery composition |
| 13 | Assumption remains distinguishable from verified fact after compaction. | T4 baseline; T5 must rerun through model-directed/recovery composition |
| 14 | `history_search` recovers a fact from a covered old user message. | T5 implementation and integration evidence required |
| 15 | `history_search` recovers a prior decision/tool outcome through a stable entry/Turn ref. | T5 implementation and integration evidence required |
| 16 | Search defaults to current Session/current branch and does not leak a sibling fork/session result. | T5 implementation and integration evidence required |
| 17 | Search does not expose reasoning signatures, `raw_content`, credentials, or hidden tool inputs. | T5 implementation and integration evidence required |
| 18 | Repeated checkpoint A -> checkpoint B still recovers original raw history rather than only checkpoint A wording. | T5 implementation and integration evidence required |
| 19 | Resume after checkpoint reconstructs the same logical working state from durable history + valid derived checkpoint or rebuild path. | T4 baseline; T5 must rerun through model-directed/recovery composition |
| 20 | Export/transcript remains raw-history based and does not replace covered messages with checkpoint text. | T4 baseline; T5 must rerun through model-directed/recovery composition |
| 21 | Stable-prefix hash remains unchanged across pressure and checkpoint projection changes. | T5 implementation and integration evidence required |
| 22 | One huge tool result still receives the existing bounded/pre-entry handling before another checkpoint opportunity. | T4 baseline; T5 must rerun through model-directed/recovery composition |
| 23 | Interaction with `MEM-007`/Issue #143 filtered tool output does not create duplicate compression metadata or stores. | T4 baseline; T5 must rerun through model-directed/recovery composition |
| 24 | Critical pressure with no voluntary checkpoint enters the one emergency checkpoint path. | T4 baseline; T5 must rerun through model-directed/recovery composition |
| 25 | Emergency checkpoint provider/tool failure never results in an over-limit normal provider request. | T4 baseline; T5 must rerun through model-directed/recovery composition |
| 26 | Cancellation during forced checkpoint produces one deterministic terminal lifecycle result. | T4 baseline; T5 must rerun through model-directed/recovery composition |
| 27 | Current model changes to a smaller/larger context window and pressure is recalculated through the existing model-limit policy. | T5 implementation and integration evidence required |
| 28 | Output/reasoning reserves materially reduce effective available context as defined by `MEM-005`. | T4 baseline; T5 must rerun through model-directed/recovery composition |
| 29 | Fork/resume/session-index rebuild retains valid recovery behavior from authoritative TLOG/JSONL. | T5 implementation and integration evidence required |
| 30 | Representative long coding/research sessions record tokens before/after, number of checkpoints, recovery searches, repeated-work regressions, and task-completion quality. | T5 implementation and integration evidence required |


## Documentation / migration / rollback

Update MEM005, Session persistence/index reference, ADR096 or an explicit additive
ADR, Runtime SDK migration, CLI/tool docs, README en/zh, iterations index, Board,
handoff and Issue390 status. Do not add required fields to existing public result
structs; additive types/extension encoding keep legacy reads. Public enum changes
follow next-minor migration. Rollback disables new tools/projection cache; raw
Session entries remain authoritative and exportable, index remains rebuildable.

## Concrete internal integration work

Current Agent snapshot transport is Vec<Message>; implement an internal envelope
carrying projected messages and verified recovery provenance. Session commit binds
receipts only after assigning final entry IDs and applying PersistencePolicy.
Index maintenance consumes committed receipts; it must never infer provenance
from tool names, text equality or index rows. Existing SessionStore exposes whole
read and atomic replacement, not bounded page reads or generic receipt support;
those additive capabilities require implementation and format regressions.

## Evidence

Preparation only. No implementation, tests or completion asserted. Governance
review and the exact claim PR will be recorded before activation.

## Preparation review corrections — 2026-11-01

Independent review requested changes for scan-page semantics, append-compatible
snapshot revisions and post-policy atomic receipt binding. The contract above
incorporates those corrections; implementation evidence and activation approval
remain absent. No new user scope decision was required.

Follow-up independent preparation review: APPROVE design direction after these
corrections, without tests or activation approval. A disposable SQLite experiment
(/private/tmp/i313-scoped-fts-ranking-experiment.log) confirmed a scoped WHERE
clause does not isolate BM25 statistics from sibling corpus insertion. This is
design evidence only, not a production regression or implementation deliverable.

## Effective activation and first storage increment — 2026-11-01

Claim #819 merged as `8b11e5f49794ea17f09c96eff8d59a6bf8f9a3d6` after
exact-head governance CI `41280178297` (four success, four correctly skipped
Rust/platform jobs), independent governance/API/privacy/storage review against
head `dafdcf879d0060c74ba40e0579dfed54ee667d8c`, both validators and merge CAS.
The proposed Active/Claimed state above is now effective. Issue #390 remains open.

First local implementation adds `history_page` bounded source reads for native
SessionEntry JSONL and TLOG, fixed captured cutoff, byte/entry limits, cooperative
cancellation and strict malformed-record refusal. Five focused regressions pass,
including multi-page >2 MiB source, >8192 entries, append-after-cutoff exclusion,
corrupt UTF-8 lengths, overflow, truncation and deadline/cancel behavior.
`cargo test --locked -p talos-session --lib history_page` and all-target
warnings-denied Clippy pass with the pinned toolchain and consistent low-debug,
non-incremental profiles. Logs: `/private/tmp/i313-history-page-tests.log` and
`/private/tmp/i313-history-page-clippy.log`.

This is not a model-facing recovery implementation: no scoped index, projection
receipts, Actor tools or end-to-end recovery exist yet. Source offset is not
identity/authorization; callers must bind it to validated snapshot lineage and
receipts. Legacy message-envelope JSONL and over-bound individual records return
unavailable/error rather than fabricate provenance. Synchronous regular-file reads
are cooperatively cancellable between 8 KiB reads, not preemptible kernel calls.
Full preflight, independent implementation review, composed scenarios and remote
CI/merge remain outstanding. T1/T6 and the full goal are unchanged.

### Bounded-source review corrections

Independent review requested changes for FIFO-open blocking and valid JSONL
compatibility. The correction uses nonblocking/no-follow Unix opening plus
opened-handle regular-file validation; Windows restricts local absolute drive
paths and rejects opened reparse/special handles. ADR-096 records the platform
boundary and the two direct already-locked OS-constant dependency edges. An
initial locked test correctly refused the changed manifest until `cargo metadata
--offline` refreshed only those two lock edges; no dependency versions changed.

JSONL now accepts blank/CRLF lines and a complete final record without newline
only at the captured EOF. TLOG accepts empty metadata. A first record that fills
one byte budget is not rejected merely because the file header occupies the first
page: a header-only page advances before reading that record. Eight new storage
regressions and all 223 Session library tests pass; all-target Clippy passes.
Native Windows opening and full workspace/runtime integration remain unverified.
Source paths, offsets and decoded entries still confer no model authority.

### Atomic recovery receipt increment — local, unmerged

The existing turn terminal marker now supports additive versioned recovery digest
receipts without modifying public SessionEntry/outcome fields or creating another
store. The new atomic finalize API accepts explicit runtime-owner projections;
policy omission or a mismatch after redaction cannot mint authority. Idempotent
retries compare receipt sets, source verification checks stored/projection digests,
and transcript exports omit internal markers. New regressions exercise reopen,
corruption, duplicate retries and persistence-policy denial/redaction.

This is storage integration only. Agent snapshot envelopes, live tool projection
provenance, branch-scoped index/query, checkpoint tools and Runtime/CLI behavior
remain unimplemented. Different model/persistence projections are not recoverable
through this increment; this residual must be resolved or explicitly bounded by
actual tool policy evidence before T5 can close. No real-model quality claimed.

Receipt review found two additional correctness gaps: a content-only marker size
check omitted storage encoding overhead, and retry lookup could select outcome and
receipts from different duplicate markers. Corrections check actual complete
JSONL/TLOG encoded sizes and accept only one matching version-valid terminal
entry. Regressions cover near-boundary escaped marker roundtrip in both formats,
over-bound atomic refusal, unknown versions and duplicate marker refusal without
writing. Pairing also refuses cross-user boundaries and duplicate invocation IDs
or results. Current Session library tests pass 227/227; the preceding receipt
stage all-target Clippy passed, with final refreshed review/check still required.

Receipt correction independent static review approved the storage boundary (not
Actor authorization or full T5). Additional adversarial pairing regression proves
cross-user, duplicate-call and repeated-result cases cannot mint receipts.
Current Session tests pass 228/228 and refreshed all-target Clippy passes; both
governance validators pass. No workspace/platform or model-facing acceptance is
claimed by these local storage checks.

### Execution snapshot envelope — local, unmerged

Agent safe-boundary snapshots now carry projected messages and explicit recovery
projections. The successful durable turn path uses only the final matching
snapshot and commits its receipts atomically. Projection eligibility requires the
existing exact-result/live-tool-instance receipt and identical model/persistence
text; restored raw text alone does not mint a receipt. Focused regression covers
unverified input, differing persistence text and replaced tool instances. Agent
library tests pass 588/588 (`/private/tmp/i313-agent-envelope-all.log`).

This stage remains under independent review. Cancelled/error-turn receipt transfer,
stronger per-turn provenance/replay tests, real Actor restart recovery, differing
eligible/persisted projection support, scoped index and model tools remain open.
A successful narrow snapshot test does not close those acceptance requirements.

### Success/error/cancel receipt custody regression

The final Agent envelope is now generated once and supplies both returned messages
and recovery provenance, avoiding duplicate stateful projection calls. Cancellation
preserves provenance from the latest complete boundary before adding the interruption
notice; error finalization uses the final matching envelope. Both paths use atomic
receipt finalization. Real Actor failed/cancelled durable fixtures now reopen and
verify original-entry receipts. The fifty-turn fixture verifies all fifty real tool
execution receipts after reopen. Agent 588/588 and all-target Clippy pass; logs
`/private/tmp/i313-envelope-terminal-tests.log` and
`/private/tmp/i313-envelope-terminal-clippy.log`.

Cross-turn receipt cache isolation/replay regressions are still open. These tests
prove storage custody, not a model-facing history_search result or branch authority.
No full T5 acceptance or merge is claimed; the increment remains local/uncommitted.

### Per-execution receipt isolation

Each `run_prepared_inner` now scopes an unguessable task-local execution UUID.
Execution receipts retain that identity; persistence snapshots require the current
identity plus exact result digest and live tool instance. Historical checkpoint
eligibility is unchanged. A nested distinct-execution regression rejects identical
old call IDs/text until actual execution records a new receipt. Spawned unrelated
tasks do not inherit this authority; existing joined tool futures remain inside
one execution scope. Success/error/cancel Actor custody regressions still pass.
Agent 588/588 and all-target Clippy pass in
`/private/tmp/i313-execution-scope-tests.log` and
`/private/tmp/i313-execution-scope-clippy.log`. Independent review is pending.
Remaining scope includes differing permitted projections, scoped indexing/search,
model checkpoint tools and all composed T5 validation; no completion claimed.

Execution envelope review APPROVE: tracked diff SHA-256
`2506514450669455ce7c1c18bbfe3706a076f9d293547058ddd72763e0729134`, snapshot
file SHA-256 `1c2fc679eed0e0ef16504a5c55693220072ea35fc571c1e0113906d30fb27fa0`.
Reviewer did not run tests and shares Agent/GitHub identity; approval is bounded
to this increment, not I313 closure. Two focused snapshot tests pass, including
scope non-inheritance by spawned tasks and nested scope restoration across yield.
Earlier full Agent suite 588/588 predates this test-only addition. Both governance
validators passed with zero warnings; no new full-goal completion evidence claimed.

### Index input projection — local, unmerged

Added a bounded model-search text projection for host-authorized entries. User
text and assistant text exclude metadata; assistant tool argument blocks are
removed by a linear scanner including truncated blocks. System/tool results
require a matching same-turn committed receipt; unknown roles and unverifiable
results are unavailable. Focused projection regression and Session all-target
Clippy pass (`/private/tmp/i313-index-projection-tests.log`,
`/private/tmp/i313-index-projection-clippy.log`).
This helper assumes host-validated branch/source membership and does not implement
that authorization, FTS indexing, scoped query or model delivery. Independent
review and composed receipt/index tests remain required. The existing general
strip_tool_syntax helper assumes complete blocks and can overrun a truncated
block; this recovery path avoids it, but the broader helper correction remains
an identified follow-up, not a completed fix.

### Shared tool-syntax parser correction

The prior truncated-block follow-up is now implemented: core strip_tool_syntax
uses a linear scan and excludes an unfinished tool block through EOF, avoiding
out-of-range replacement and argument disclosure. Recovery index projection now
reuses that shared parser instead of maintaining a duplicate implementation.
Two focused Core regressions and all 229 Session library tests pass. Logs:
`/private/tmp/i313-tool-syntax-tests.log`,
`/private/tmp/i313-projection-session-tests.log`. Independent privacy review and
Clippy are pending for this local increment; host branch authority, indexing and
model discovery/query still remain unimplemented.

### Serialized-argument fence privacy correction

Independent review found that a triple-backtick sequence inside a JSON argument
could terminate the naive scanner early and disclose the remaining argument text.
The shared parser now locates a complete JSON value with the JSON decoder before
accepting its closing fence; malformed/truncated blocks exclude the rest through
EOF. Real message_parts serialization regressions cover backticks, escaped quotes,
backslashes/newlines and missing closing fences. Independent bounded privacy review
APPROVE: tracked diff `9703258e7135825d7b158d3e1699ef91ddeddab704ef67a1c2976156d610857e`,
projection file `426ad1045442412500eda1db12c1f05abacf361abd6c351324213b9376e1016d`.
Reviewer did not run tests and shares Agent/GitHub identity. Two Core regressions,
230 Session library tests and affected all-target Clippy pass. This resolves the
local parser/privacy finding, not host authorization or full T5 acceptance.

### Recovery projection cache — local, unmerged

Added recovery_sources and recovery_projection_fts in the existing SessionIndex
connection, with scope/entry/order mapping and bounded-page atomic host writes.
Only eligible projected text enters this FTS table; original global human search
is unchanged. Focused regression proves raw-only tool terms have zero matches,
conflicting page rollback preserves prior data, and replayed page writes do not
increase row count. Test log: `/private/tmp/i313-recovery-index-tests.log`.

This is not a query/disclosure authority: host snapshot binding, incremental
completion cursor, source revalidation, SQLite cancellation/deadline enforcement,
read-only scoped queries and model composition remain required. The write API
bounds input bytes/count but does not yet bound SQLite instruction/time work.
No indexing-complete or T5-complete claim is made by this local increment.

### SQLite VM cancellation increment

Enabled the existing rusqlite hooks feature (no version change) and added a
clonable recovery cancellation/deadline budget capped at ten seconds. Projection,
per-entry writes and pre-commit checks share it; SQLite VM progress interrupts
long-running SQL. During the operation busy waiting is disabled, then the prior
busy timeout and progress handler are restored/cleared by a guard. Kernel I/O is
still synchronous; no hard preemption claim. A long recursive SQL regression
observes OperationInterrupted and proves the connection works after guard cleanup.

Initial timeout restoration used an unsupported SQLite u64 decode and failed
compilation; corrected to u32 before rerunning checks. Current full Session tests
and Clippy logs are `/private/tmp/i313-index-session-tests.log` and
`/private/tmp/i313-index-budget-clippy.log`. Independent review remains pending;
scoped query, source/branch authorization and model integration are still absent.

### Recovery SQLite review corrections

Independent review required panic containment at the new native integration
boundary and budgeting schema initialization. Both initialization and page writes
now catch unwinds around actual SQLite/transaction/guard work and return a
content-independent error. Schema creation is atomic and uses the same deadline,
VM callback and temporary no-busy-wait policy. Removed the inaccurate claim of a
typed database-busy error; the existing IndexError retains string-backed details.
Injected panic after a write proves rollback and restored busy_timeout; an expired
schema budget creates no tables. Session 235/235 and all-target Clippy pass.
Follow-up independent review and workspace/native verification remain outstanding.

### Read-only scoped candidates — local, unmerged

The query API now returns at most twenty entry IDs/ordinals from a host-selected
scope, in source order with keyset pagination. Empty query supports discovery;
FTS searches only recovery projections, filters scope before LIMIT, and uses no
global BM25 statistics. It shares cancellation and caps query work at two seconds.
It never creates/repairs schema or returns source text. Missing schema remains an
IndexError at this layer; typed model-facing unavailable mapping is still required.

Seven focused recovery tests pass, including sibling corpus independence, paging,
missing-schema refusal, cancellation/expiry, malformed MATCH with an actual source
row, no mutation and connection reuse. An initial malformed-query fixture had an
empty table and SQLite skipped MATCH evaluation; the populated fixture corrects
that test assumption. All-target Session Clippy with warnings denied passes.
Logs: `/private/tmp/i313-query-boundaries.log`, `/private/tmp/i313-query-clippy.log`.
Independent incremental static review APPROVE against `6f3cc9e3e` with two-file
diff SHA256 `dbfbb9f0951ff379cf622d7263766c743233b8fec27a8d144b66572207873b3f`.
Reviewer did not run tests and shares Agent/GitHub identity. Host snapshot/branch
authority, complete incremental rebuild and source revalidation, model tools and
Actor integration remain required; no full T5 acceptance is claimed.

### Persistent recovery rebuild cursor — local, unmerged

The existing SessionIndex now stores immutable-cutoff build progress in
recovery_builds. Host maintenance begins a fresh scope and atomically advances
byte/record cursors with each bounded projection page using a compare-and-swap
condition. Stale cursors, duplicate indexed source IDs and writes bypassing tracked
progress are rejected; page failure rolls back the cursor too. Reopening the index
resumes the recorded position. Tracked incomplete builds refuse candidate queries.
This is maintenance progress, not source/branch authority: the host still must
validate the source prefix and lineage, and model-facing typed status mapping is
not yet implemented. Untracked low-level cache scopes still require caller-owned
completeness checks; they are not a model-query surface.

Real native SessionEntry JSONL fixtures with 10,000 records each compose bounded
source reads and multiple atomic cache pages, then recover the final entry.
A separate reopen/rollback fixture advances beyond both single-page counters;
this does not certify cross-page tool receipt resolution or an Actor rebuild loop.
Session 239/239 and all-target warnings-denied Clippy pass. Logs:
`/private/tmp/i313-cursor-session-tests.log`, `/private/tmp/i313-cursor-clippy.log`.
Independent static review APPROVE against `6b0bc4ac1`, two-file diff SHA256
`d1233ecc7c2061701b6a56226a1cc39312814e41f14ddb43c4c7b99f108ff40f`;
reviewer did not execute tests and shares Agent/GitHub identity. Fresh fetch kept
origin/main at `8b11e5f49794ea17f09c96eff8d59a6bf8f9a3d6`. No T5 completion,
remote implementation CI, source disclosure authority or model-tool integration
is claimed. Full original acceptance and T1/T6 remain outstanding.

### Original-record locator and bounded reread — local, unmerged

Bounded source pages now retain exact record byte ranges. read_history_record
accepts a single bounded range and rejects empty/oversized, widened multi-record,
misaligned and cancelled reads. JSONL/TLOG fixtures verify exact text after an
append beyond the captured range. The additive recovery_locations table stores
optional ranges with cached candidates; tracked writes constrain them to the
current scan page. Replacement clears old ranges and corrupt decoded ranges fail
closed. Missing ranges cannot support bounded disclosure.

The real 10,000-entry JSONL fixtures now compose query with direct original-record
reread and compare source ID/text. This proves locator plumbing, not immutable
source digest, permission or branch authority. Cross-page terminal-receipt lookup,
host rebuild scheduling, model tools and Actor integration remain required.
The affected structs are new unmerged I313 APIs; added fields are not presented
as source-compatible changes to an already released struct-literal API.

Session 240/240 and all-target warnings-denied Clippy pass before one additional
test-only regression; the final two focused locator tests pass, covering cache
replacement and corruption. Logs: `/private/tmp/i313-locator-session-tests.log`,
`/private/tmp/i313-locator-clippy.log`, `/private/tmp/i313-locator-regressions.log`.
Independent incremental static review approved the production delta against
`99340a0ee` and confirmed the final test-only addition, with two-file diff SHA256
`56533650a5fbc52ac6a6f1ed5a77caec9a675ad1cdf626be9582bfe4a33ad877`.
Reviewer did not independently run tests and shares Agent/GitHub identity.
No full T5 or remote validation/completion claim is made.

### Explicit Read recovery view — local, unmerged

Added the default-refusing AgentTool recovery_projection hook and forwarding
through CLI/Runtime wrappers. Agent calls it only after verifying live execution
provenance when model/private-filtered text differs from persistence text. The
result is private-filtered again, bounded, and required to equal the tool's
permitted persistence view. Panic refuses recovery. Read opts into its already
sanitized numbered text, excluding transient snapshot IDs/anchors. Other tools
retain default refusal for differing views. Session persistence policy still owns
the final receipt decision; this is no new permission to retain model-only text.

A real Read execution fixture composes Agent snapshot, atomic durable commit and
reopen under both permitted and denied persistence. It initially failed because
Session redaction also normalizes trailing LF/CRLF. Binding now accepts only that
line normalization, hashes the final stored text, and still refuses sensitive-line
replacement. The corrected three focused Agent tests pass. Separate retained
recovery text differing from persisted tool text remains unimplemented, as do
model search/checkpoint and full Actor integration.

Independent API/privacy review approved production code and the normalization
correction (six-file diff SHA256
`cafef60c76c0322fcbfcf87dd18d6511543538ec9775e15f2d840bd8b1d55789`).
Reviewer did not independently run tests and shares Agent/GitHub identity.
Expanded all-target Clippy failed on two existing unwrap_err calls in
search_backend/health.rs (lines 212/216), independently confirmed in origin/main;
log `/private/tmp/i313-read-recovery-clippy.log`. This is not a passing all-target
check. Standard-preflight-style production Clippy for all six affected crates
passes; full Agent 590/590 and Session 241/241 library tests pass. Logs:
`/private/tmp/i313-read-recovery-production-clippy.log`,
`/private/tmp/i313-read-recovery-full-tests.log`; focused log
`/private/tmp/i313-read-recovery-tests.log`. Full workspace preflight, native
platform evidence and complete I313 review/CI/merge remain required.

### Shared operation identity and submitted checkpoint preparation — local

CompactOperations now distinguishes runtime-selected Manual/Model origins while
retaining one pending slot and one 64-receipt window. Each origin has its own
monotonic sequence; model requests cannot consume manual IDs or complete/replay
a manual receipt with the same number. Existing manual APIs remain wrappers.
Mixed-origin Actor integration must use replace_generation_with_origin so its
cancellation retains origin; this integration is not yet implemented.

The existing complete-tool-batch checkpoint job can carry a submitted structured
payload. Preparation uses the same runtime-selected eligible prefix, source
binding, recent-window preservation, reclamation check, whole-request estimate
and deferred projection candidate. Submitted payloads do not invoke the summary
provider and may voluntarily compact below pressure. The current manual producer
still supplies no payload: model registry, discovery snapshot authorization,
Actor admission/receipt routing and installation composition remain unimplemented.
The new preparation path alone is not model-compaction delivery.

Independent review initially rejected a privacy check against JSON text: quoted,
backslash and newline tokens remain escaped after rendering. The corrected path
checks decoded JSON string values before render, with regressions for all three
plus escaped Unicode. Submitted preparation and its deferred installation candidate cannot renew a source deadline.
Final Agent 593/593 and all-target warnings-denied Clippy pass; logs:
`/private/tmp/i313-submitted-install-deadline-tests.log`,
`/private/tmp/i313-submitted-install-deadline-clippy.log`. The shared-ledger focused tests
passed 5/5, and checkpoint regression group passed 28/28 before the final deadline
assertion; the final full test run includes it. Final independent static APPROVE
covers five-file diff SHA256
`e5a24e0204e68e1e23e98068f52a4b09855625fd90321244ba1cd5fde255595d`.
Reviewer did not run tests and shares Agent/GitHub identity. Full T5, remote
preflight/platform/CI/merge and later T1/T6 acceptance remain outstanding.

### Frozen submitted coverage and shared pairing gate — local, unmerged

Submitted checkpoint jobs now carry the runtime-held HistorySource and frozen
covered_end with their payload. The complete-batch path compares generation,
entry identities and digests for that prefix before preparation, bounds coverage
to 8192 entries, and retains the earlier current/frozen source deadline through
candidate installation. The shared engine uses the smaller of the frozen cutoff
and current recent-window cutoff, so later appended exchanges cannot silently
expand coverage. This establishes source consistency only: session/branch/
permission authority, opaque discovery tokens and actual model admission remain
unimplemented. The manual producer still sends no submitted job.

Moved complete-tool-exchange validation before automatic/submitted route
selection; the submitted route had skipped the automatic summarizer packing
loop's pairing check. Regression rejects missing results, duplicate call IDs,
wrong result IDs, orphan results and a valid call/result cut by the frozen cutoff.
Other regressions preserve appended history, reject prefix mutation/generation
change and retain private-content/size/deadline rejection without early install.

Production delta passed full Agent 595/595 and all-target warnings-denied Clippy:
`/private/tmp/i313-submitted-snapshot-agent-tests.log`,
`/private/tmp/i313-submitted-snapshot-clippy.log`. The later test-only mid-batch
cutoff addition passes the three submitted tests:
`/private/tmp/i313-submitted-cut-batch-tests.log`. Independent static review
approved the production delta and final test-only addition; two-code-file diff
SHA256 `87f03807cb9e235a77a6d8a3b1300ca2006a6888ce9b0220e30239ad8f07e882`.
Reviewer did not run tests and shares Agent/GitHub identity.
No model-tool/Actor delivery, full T5 completion or remote validation is claimed.

### Actor coordinator model-boundary ingress — local, unmerged

The existing Actor compaction coordinator accepts an internal model submission at
the complete-batch boundary, using the same pending slot and origin-qualified
ledger. Runtime-issued model IDs do not consume manual sequence IDs; model terminal
outcomes use a dedicated oneshot rather than aliasing manual UI events. Busy model
submissions refuse without disturbing queued manual work. Preparation dispatch
failure, turn cancellation and abandoned receipt release the slot and resume the
boundary; terminal send occurs before resume. This is not tool-level Admitted
semantics: the production boundary sender still supplies submitted: None. Model
registration, discovery authorization and pre-result admission remain required.

Review initially found that a receipt closed after admission could still permit
installation. Waiting now observes receipt closure and cancels; completion aborts
the waiter and checks cancellation/closed receipt before candidate installation.
This check establishes an installation linearization boundary, not an acknowledged
receipt-delivery transaction: closure racing after that check does not roll back
an installed checkpoint. Future model dispatch must consume its terminal receipt
before continuation and handle channel closure as unavailable, never Applied.

Six coordinator regressions cover shared manual/model capacity, terminal-before-
resume ordering, manual-ID isolation, cancellation, closed dispatch, pre-admission
abandonment and receipt closure after dispatch or candidate preparation. Full
Agent 599/599 and all-target warnings-denied Clippy pass. Logs:
`/private/tmp/i313-model-boundary-final-tests.log`,
`/private/tmp/i313-model-boundary-final-clippy.log`.
Independent static APPROVE: two-file diff SHA256
`6cc754443609448ed2bb1291cf5a8589fdd7f78ae6dc9ede032a4b2d72c98741`.
Reviewer did not run tests and shares Agent/GitHub identity. Full T5 and later
T1/T6, workspace/native/remote gates and merged owner evidence remain outstanding.

### Pre-batch model admission and positive installation — local, unmerged

The shared Actor coordinator now separates model admission (runtime operation ID
and shared capacity reservation) from complete-batch preparation/installation.
Turn termination cancels a queued model submission without cancelling queued manual
work; explicit cancellation also releases the model slot. This interface is not yet
called by a production model tool: tool registration, scoped discovery authorization,
admission transport and dynamic terminal receipt delivery remain outstanding.

A submitted structured-checkpoint regression exercises admission, actual shared
preparation and Actor installation, asserting no early projection, Applied available
before continuation, unchanged raw history and reusable manual capacity. Another
regression covers cancellation between admission and batch arrival. The initial
positive fixture exceeded its retained-history budget and correctly failed; the
final fixture uses sufficient capacity for voluntary compression.

Final locked Agent tests pass 601/601 and all-target warnings-denied Agent Clippy
passes. Logs: `/private/tmp/i313-preadmission-agent-final.log` and
`/private/tmp/i313-preadmission-clippy.log`. Independent static review approved the
two-code-file diff SHA256
`9127f1667d28630674685ed5ea5af54f115775fcdccb099db761dc1faf67eff1`;
reviewer did not run tests and shares Agent/GitHub identity. A queued receipt closure
is cleaned up at boundary or turn terminal, not by an immediate wakeup; installation
still checks closure. This does not establish model-tool delivery, full workspace
validation, remote CI or T5 completion. T1/T6 and the overall goal remain open.

### Abandoned admission wakeup — local, unmerged

Superseding the queued-receipt cleanup limitation above, receipt closure now wakes
Actor selection before batch arrival or turn termination. The coordinator releases
the abandoned model slot without cancelling the active turn. Running-work scheduling
retains its existing predicate; only the completion wait includes queued model work.
A regression drops the receiver, awaits the real coordinator wakeup with a deadline,
and verifies manual admission succeeds while the turn token remains uncancelled.

Locked Agent tests pass 602/602; all-target warnings-denied Agent Clippy passes.
Logs: `/private/tmp/i313-abandon-agent.log`, `/private/tmp/i313-abandon-clippy.log`.
Independent static review approved the two-file diff SHA256
`40ca650e6ceaf8de0467866f6249959922e550e0c6de15718f6ebc2c624d6712`.
Reviewer did not run tests and shares Agent/GitHub identity. This is a lifecycle
repair, not production model-tool admission transport or T5 delivery. Those entry
points, full workspace/remote validation, T1 and T6 remain outstanding.

### Explicit model-tool Actor path — local, unmerged / Partial

The lower-level Agent now exposes fallible `enable_model_checkpoint_tools(&mut self)`.
The host must opt in before Actor composition; registration is atomic and reports
same-name collisions without replacing existing tools. Definitions remain stable
through pressure changes. CLI and RuntimeBuilder do not yet enable this capability.
The default Actor tool inventory and request budgets remain unchanged.

`history_search({"mode":"discover"})` currently returns an execution-local opaque
snapshot and at most 20 opaque source references, without source content. More than
20 references are explicitly incomplete; pagination, search and detail remain
unimplemented. `compact_context` accepts the structured checkpoint plus snapshot,
translates issued references, checks source/permission freshness and asks the live
Actor to admit the operation before returning Admitted. One execution-scoped receipt
is retained until the complete paired batch reaches the shared preparation/install
path. The actual terminal metadata is appended to the next observed request and
re-estimated/sealed without rerunning observation hooks. This is distinct from the
still-missing durable branch authority, authoritative history queries and terminal
Session receipt when there is no continuation.

Permission Session identity and all revision counters are captured with discovery.
The installation gate now uses additive `PermissionSessionState::with_current_snapshot`
to compare and commit under one state lock. Lock order is runtime admission then
nonblocking permission acquisition; contention refuses the commit. Callbacks must
not await, perform blocking I/O or reenter that permission state. Existing CLI
publication/rebind holds permission only while publishing synchronous routes; the
new nonblocking inner acquisition avoids reverse lock waiting.

The real registered-tool Actor regression observes discovery, a subsequent model
compact call, Admitted in its paired tool result, operation 0's actual Applied
metadata in the following provider request, successful continuation and unchanged
raw prefix. Additional regressions cover cross-execution token refusal, covered
prefix mutation, permission reset, standalone unavailability, registration collision
and concurrent permission reset vs installation fence. These deterministic fixtures
prove execution behavior, not semantic checkpoint quality or restart/fork recovery.

Review found and fixed three defects: a cached observed plan dropped terminal parts;
a substring assertion could falsely match Applied in tool descriptions; and the
initial enable API swallowed registration errors. A second security finding replaced
check-then-install permission snapshots with the atomic fence described above.
The initial always-on registration failed three baseline inventory/budget tests;
explicit host opt-in preserves the default path while full composition is unfinished.

Final locked tests: Agent 606/606 and Permission 169/169. Both crates' all-target
warnings-denied Clippy passes. Logs: `/private/tmp/i313-tools-fenced-tests.log`,
`/private/tmp/i313-tools-fenced-clippy.log`. Independent static security/API APPROVE:
tracked-code diff SHA256 `d0b9f2306dbeb6df2012179b70ea74cc8427764ddc0ab493bd1f8268805fef41`;
new checkpoint_tools.rs SHA256 `777a8ed30e80dc3b6fd010de4bf691a7382187a5a2e7b63b646ee0d8d454113d`.
Reviewer did not run tests and shares Agent/GitHub identity. Full workspace preflight,
SDK/platform/remote CI, final API/docs migration, default host activation and all
remaining 30-row acceptance stay required. T5, T1, T6 and the whole goal remain open.

### Execution-local discovery pagination — local, unmerged

Discovery now returns a single-use opaque next_cursor, at most 20 references per
page in original source order, and the same issued snapshot across pages. Append
and refresh do not expand that snapshot. Rediscovery replaces the cursor;
cross-execution, permission reset, generation change and covered-prefix mutation
refuse continuation. State retains only current/issued snapshots and one cursor,
not a growing page cache. This supersedes the discovery-only pagination gap above;
durable index search/detail and their paging/branch authority remain unimplemented.

Locked Agent tests pass 608/608 (`/private/tmp/i313-pages-agent.log`). Independent
static review approved the pagination delta; reviewer did not run tests. Clippy
initially flagged a test lock lifetime; an explicit lexical scope releases it
before the final await. Final Clippy evidence: `/private/tmp/i313-pages-clippy.log`.
This stage remains local and does not close T5, T1, T6 or the whole handoff goal.

### Bounded recovery response projection — local, unmerged

Authoritative rereads can now use recovery_response_text with an explicit
RecoveryTextKind: Snippet has a fixed 512-byte ceiling and Detail 8192 bytes.
The helper reuses index eligibility/terminal receipt verification, never falls back
to raw unverified tool content, preserves UTF-8 boundaries and reports truncation.
A truncated detail is not full-fact recovery. The existing real 10,000-record
resumable-build test now follows its cache candidate through original-record reread
and snippet/detail projection; Unicode/private/unverified-output regressions cover
the response boundary. This does not establish host branch authority or source
freshness, cross-page terminal lookup, or model search/detail dispatch. Those remain
required within T5, alongside host activation and the remaining published acceptance.

Validation logs: `/private/tmp/i313-response-final-tests.log` and
`/private/tmp/i313-response-final-clippy.log` (locked Session library tests and
warnings-denied all-target Session Clippy). No full workspace or remote CI evidence
is claimed for this local increment; T5/T1/T6 and the overall goal stay Active.

### Read-only recovery index connection — local, unmerged

SessionIndex::open_read_only opens an existing database with SQLite READ_ONLY,
query_only and zero busy timeout inside a panic boundary. It never calls the
write-capable constructor, changes journal mode, creates schema or repairs cache.
SQLite may coordinate WAL readers through shared memory; this is not a filesystem
sandbox or branch authorization. Missing files fail to open; corrupt files may
fail on opening or querying and must be repaired by host maintenance. SQLite
opening and underlying kernel I/O remain synchronous; zero busy timeout does not
make those operations asynchronously cancellable.

Real SQLite regressions cover missing-file/no-schema noncreation, DDL/rebuild write
refusal, incomplete build refusal, a reader observing a completed host WAL build,
scope-separated results, cancellation and unchanged corrupt bytes. Locked Session
tests pass 244/244 and all-target warnings-denied Session Clippy passes; logs:
`/private/tmp/i313-readonly-session.log`, `/private/tmp/i313-readonly-clippy.log`.
Model search/detail dispatch, authoritative source scope, cross-page receipt lookup
and host rebuild scheduling are still unimplemented. This connection prerequisite
does not close T5, T1, T6 or the full goal; final workspace/remote gates remain open.

### Durable recovery source binding and record reread — local, unmerged

DurableSession can capture a storage-issued fixed path/session/branch descriptor
and byte cutoff, then read bounded original pages. DurableRecoveryRecord binds one
captured original record's serialized-entry digest and refuses changed rereads,
including same-length text replacement. Later appended turns remain outside the
captured cutoff. Source and record share an Arc-backed lightweight descriptor;
review rejected the initial full Session clone because its branch history was not
bounded by a page. No full Session or branch entries are retained by this descriptor.

Capture still uses synchronous locking/metadata I/O. The record digest proves
consistency only after capture_record, not between the prior page read and record
capture, nor the entire prefix. These objects do not establish current permission,
branch membership, cloned-file refusal or model disclosure authority. Host query
integration must supply those checks and cross-page terminal receipt lookup.

Locked Session tests pass 245/245 and all-target warnings-denied Clippy passes:
`/private/tmp/i313-source-final-tests.log`, `/private/tmp/i313-source-final-clippy.log`.
The real durable fixture covers cutoff exclusion, identity, cancellation/deadline,
out-of-cutoff locators and same-length replacement. Independent static APPROVE:
code diff SHA256 `7774833978b9eedcb7f863ecd0f03a385821eaca8e670f139a1074c7c2221c13`.
Reviewer did not run tests and shares Agent/GitHub identity. Model search/detail,
full T5 acceptance and final workspace/remote validation remain unfinished; T1/T6
and the overall handoff remain open.

### Composed original-record and terminal reread — local, unmerged

DurableRecoveryRecord::recover_text rereads both the original source and optional
terminal receipt marker from the same captured descriptor/cutoff, checks each
captured digest, then applies eligibility and fixed snippet/detail projection.
The two record ranges must fit the combined 2 MiB source-operation budget before
I/O begins. No marker means unverifiable tool text remains unavailable. A different
capture cannot lend its marker even when it names the same durable path.

The durable restart regression now recovers a newly persisted permitted tool fact
through these real rereads, refuses missing/cross-capture markers and cancellation,
and modifies actual on-disk terminal extension bytes while leaving the tool record
unchanged; recovery then returns Stale. A private oversized-range fixture verifies
combined-budget rejection before I/O. This still does not establish full-prefix or
current branch/permission authority, cross-page marker lookup or model dispatch.

Session 246/246 and all-target warnings-denied Clippy passed before the final
marker-test enhancement (`/private/tmp/i313-composed-session.log`,
`/private/tmp/i313-composed-clippy.log`); the enhanced real marker regression passed
in `/private/tmp/i313-marker-replacement.log`. Independent review approved the
production delta; reviewer did not run tests. All final T5, T1/T6 and whole-goal
implementation, workspace validation and remote closure obligations remain open.

### Cross-page terminal locators — local, unmerged

Host scan pages now atomically persist original terminal entry IDs, turn IDs,
ordinals and byte ranges alongside the recovery cursor in the existing index.
Only version-one markers with matching metadata turn identity and bounded source
ranges qualify. Scope/turn lookup returns an untrusted original-record locator,
not marker text or disclosure authority. Duplicate terminal identity rolls back
both page and cursor; non-append replacement removes that entry's prior locator.
Existing untracked terminal state prevents reusing its scope for a fresh build.

A real >2 MiB durable TLOG fixture places a permitted tool result on an earlier
page than its terminal marker. Host pagination, index reopening read-only, marker
lookup and both original-record/receipt rereads recover the committed tool fact.
The test retains the tool record captured during scanning: it does not yet prove
backfilling that early tool result into searchable FTS. Host scheduling, full
source/branch authority and model search/detail remain required. Maintenance may
look up markers during an incomplete scan; that cannot authorize model disclosure.

Locked Session tests pass 248/248 and all-target warnings-denied Clippy passes:
`/private/tmp/i313-terminal-session.log`, `/private/tmp/i313-terminal-clippy.log`.
Independent static APPROVE, diff SHA256
`e8422ddba79d4af18ae60ea08084781eb76f6babd4bedf213706c1cb42d342d7`;
reviewer did not run tests and shares Agent/GitHub identity. Existing indexes need
host schema initialization before terminal lookup; model queries never migrate.
T5/T1/T6 and the whole goal remain incomplete; final workspace/remote gates remain.

### Host recovery scan and cross-page backfill — local, unmerged

Host `maintain_recovery_source` composes bounded durable page reads, persisted
scan progress and a second phase resolving original tool/terminal records. Scan
transactions retain pending locators without copying private text. Query refuses
scopes with pending resolution even after the byte scan completes. Resolution
atomically inserts eligible FTS text and removes its pending locator; missing
receipts produce no text. Cancellation or read/transaction failure retains work.
Each resolution reads at most two records totaling 2 MiB under the same deadline.
Existing pre-backfill caches must be rebuilt under a fresh scope; schema creation
does not retroactively certify their coverage.

The real >2 MiB TLOG regression now runs the host maintenance entrypoint, reopens
the database, verifies pending-query refusal and cancellation, resolves persisted
work and finds the early tool fact through read-only FTS with an original locator.
Session 248/248 and all-target warnings-denied Session Clippy pass:
`/private/tmp/i313-backfill-session.log`, `/private/tmp/i313-backfill-clippy.log`.
This is explicit host maintenance, not automatic Runtime scheduling or model
search/detail dispatch. Current source/branch/full-prefix authority, product
activation and remaining acceptance still require implementation. T5 stays
Partial; T1/T6 and full workspace/remote closure remain outstanding.

### Dynamic model pressure notice — local, unmerged

With the built-in checkpoint tools explicitly enabled, the real provider request
sealing path appends content-free pressure metadata at Advisory or higher using
the existing full-request estimate. Normal requests have no notice. Definitions
and instruction prefix remain stable. The notice is request-only, is re-estimated
with image/output/reasoning reserves and passes the same final admission check.
Its level describes the pre-notice/pre-final-recovery estimate, not a claim about
the final reduced request. A fitting Advisory request can continue normally.

Focused tests include actual provider dispatch, unchanged tool definitions,
absence from returned persistence messages, unchanged original prefix, no private
content in metadata, threshold removal on increased capacity and rejection when
the notice cannot fit. Six pressure-related tests pass in
`/private/tmp/i313-pressure-focused.log`. Locked Agent tests pass 610/610 and
warnings-denied all-target Agent Clippy passes in
`/private/tmp/i313-pressure-agent.log` and `/private/tmp/i313-pressure-clippy.log`.
Independent static review APPROVE against `c9ddf1360`, code diff SHA256
`031a2f99e4e49fd6a13470c4b9c326fa484f985995751cc33f5e1d1a29b7f17a`;
reviewer did not run tests. The query-normalization experiment was reverted after
it changed existing semantics without advancing source authority. Durable model
search/detail, authorization context and Runtime/CLI activation remain unfinished;
this notice does not close those requirements or T5/T1/T6.

### Incremental raw-prefix consistency evidence — local, unmerged

RecoveryPrefixScan reads raw source bytes in steps of at most 2 MiB, checks
cancellation/deadline every 8 KiB read and retains one rolling digest rather than
a page-sized manifest. Verification must reach the captured cutoff and compare
the entire digest. Unix dev/inode/ctime/mtime/length stamps reject changes during
a scan; other platforms currently return UnsupportedSource. This detects source
consistency only, not permission/branch authority, and must be composed with a
current delivery fence. It does not prevent an arbitrary host from rewriting data.

A real >2 MiB fixture covers partial/cancelled scan refusal, same-length atomic
replacement with preserved mtime between chunks, true suffix append compatibility
and changed-prefix rejection. Session 250/250 and all-target warnings-denied
Session Clippy pass in `/private/tmp/i313-seal-session.log` and
`/private/tmp/i313-seal-clippy.log`.

Important integration gap found by the raw-byte regression: current durable turn
finalization atomically re-encodes TLOG and regenerates its creation header. That
is not byte-prefix append and this seal correctly refuses it. Header preservation
or an explicitly validated format-aware digest contract remains required before
normal durable turn appends can use this seal. The passing append fixture uses an
actual filesystem suffix append, not durable finalization. Windows support,
source-context binding, model search/detail and product activation remain open.
No T5 completion or full workspace/remote certification is claimed.

### Durable turn prefix compatibility fix — local, unmerged

Atomic TLOG replacement now preserves the validated existing creation header,
read with a 128-byte bound before creating a temporary file. Missing/empty files
receive a new header; malformed/unknown-version headers fail without replacing
the source. This fixes the header regeneration gap recorded above for newly
encoded durable turns. It does not promise arbitrary legacy record serialization
is byte-stable across decode/reencode, nor authorize copied files as branches.

The >2 MiB prefix regression now commits a real second durable turn, asserts the
entire original byte prefix remains intact and verifies the existing prefix seal.
The filesystem-only suffix append has been removed from that test. Additional
coverage preserves a fixed historical timestamp and checks malformed/oversized
headers leave both source bytes and directory contents unchanged. Locked Session
tests pass 252/252 and all-target warnings-denied Session Clippy passes:
`/private/tmp/i313-header-session.log`, `/private/tmp/i313-header-clippy.log`.
Model search/detail wiring, branch/permission context and Windows source checking
remain unfinished; final workspace/remote closure and T1/T6 remain open.

Header-reader review required reusing the nonblocking/no-follow regular-file
opener. The fix and actual FIFO/directory/symlink quick-refusal regression pass;
no temporary file is created and the linked target stays unchanged. Windows
local absolute-drive path restrictions of that opener remain applicable; native
platform CI has not yet certified this storage change.

### Configured durable owner provenance in Actor recovery — local, unmerged

DurableSession now carries a private shared owner capability. Handle clones retain
it; independently opened handles and copied TLOG identities do not. Storage-issued
history snapshots and bounded source descriptors retain this capability. Actor
recovered-source binding now checks configured-owner provenance as well as exact
message equality, replacing the previous UUID-only ownership check. Rebinding an
Actor requires a freshly issued snapshot from the new configured owner; ordinary
Runtime construction reads from that same handle and remains compatible.

Regression coverage includes same-ID/path/content independent reopen refusal,
explicit rebind then successful restoration, actual copied TLOG refusal despite
equal entry IDs/bytes, and legitimate handle-clone acceptance. This proves handle
provenance only: source mutation, generation, current permissions and authorized
branch prefix still require their own validation. No model search/detail dispatch
or complete T5 behavior is claimed. Locked Agent 610/610 and Session 253/253
library tests plus warnings-denied all-target Clippy pass. Validation logs are
`/private/tmp/i313-owner-tests.log` and `/private/tmp/i313-owner-clippy.log`;
final workspace/SDK/platform/remote gates and T1/T6 remain outstanding.

### Composed index candidate disclosure backend — local, unmerged

RecoveryPrefixSeal retains the verified file stamp and can read one exact record
through an opened-handle before/after version fence. SessionIndex candidate-text
lookup first matches the candidate against the scoped complete index, then reads
original source and terminal receipt through the seal, enforces their combined
2 MiB bound and a two-second operation budget, and returns the existing bounded
snippet/detail projection. It rechecks the verified file version before return.
Append/replacement after verification requires completing a fresh verification;
no unchecked reread follows a discarded consistency result.

The actual >2 MiB cross-page fixture now queries read-only FTS, seals the full
source, recovers the committed tool fact through this composed API, and refuses
wrong scope, cancellation and source replacement. Seal assertions are Unix-only;
existing portable index/reread assertions remain active. Locked Session 253/253
and all-target warnings-denied Session Clippy pass. Logs:
`/private/tmp/i313-query-composed-tests.log`,
`/private/tmp/i313-query-composed-clippy.log`.
This API is a host backend, not model tool dispatch. Scope ownership/branch,
generation and permission must still be bound and rechecked inside the final
result-delivery fence; Windows seal support and Runtime/CLI integration remain
unfinished. No completion or workspace/remote certification is claimed.

### Explicit distinct recovery projection persistence — local, unmerged

Live execution provenance plus explicit AgentTool recovery_projection opt-in can
now retain a bounded recovery view different from ordinary persisted tool text.
Private-token filtering runs first; storage refuses projections changed by its
redaction policy and emits no receipt when tool persistence is disabled. Equal
views retain v1 digest-only receipts. Different permitted views use v2 with at
most 8192 bytes of text, bound to the final stored entry digest and call identity.
The existing atomic terminal-marker size limit remains in force.

Real execution snapshot tests commit and reopen both views under allow/deny
policies, preserve default tool refusal, verify idempotence and reject changed
stored data, unknown versions, v1 with retained text, missing/oversized retained
text and changed projection digests. Agent 610/610 and Session 253/253 passed before
the final test-only strengthening; the focused snapshot suite and final Clippy
logs are `/private/tmp/i313-projection-focused.log` and
`/private/tmp/i313-projection-clippy.log`; full suite log:
`/private/tmp/i313-projection-tests.log`. Independent static review approves the
production delta; reviewer did not run tests. This closes the prior unsupported
representation path locally, not model search/detail or whole T5 acceptance.
Old readers reject receipt sets containing v2 rather than disclosing unsupported
text; raw transcripts remain readable. T5/T1/T6 and remote closure remain open.

### Actual durable Actor search/detail wiring — local, unmerged

Explicitly enabled checkpoint tools now prepare a durable recovery context in
turn forwarding before provider execution. The host uses the existing root index,
checks linear parent order, builds eligible projections, resolves cross-page
receipts and verifies the full prefix captured before maintenance. Queries reopen
read-only, validate current owner/scope readiness before and after lookup, read
through the sealed source, and share one 2 MiB record-read allowance across results.
A host per-session scope slot atomically removes the prior transient scope's FTS,
locators, terminals, pending rows and cursor before replacement. Retired contexts
return Unavailable, not complete empty results.

history_search search/detail require the issued discovery snapshot, unchanged
execution prefix and a current permission lease. The final permission publication
closure checks cancellation without filesystem I/O. File/index validation occurs
immediately beforehand: it is not an atomic filesystem/permission transaction.
The cancellation token is observed inside actual SQLite progress and source-read
budgets through a nonblocking callback. Search returns explicit complete=false
when capped at twenty results or response bytes; pagination_available=false is
honest current behavior, not acceptance of missing pagination.

A real Actor fixture now drives provider discover -> search -> detail -> ordinary
continuation using durable committed history, asserting the recovered original
fact and unchanged raw prefix (`/private/tmp/i313-wire-actor.log`). Additional
fixtures cover permission reset, cancellation, scope retirement and host cleanup.
Final locked Agent 613/613 and Session 253/253 pass, including the scope-readiness
correction; affected all-target Clippy is recorded alongside them. Final logs:
`/private/tmp/i313-wire-final-tests.log`, `/private/tmp/i313-wire-final-clippy.log`.
Independent static review approves this bounded wiring; reviewer did not run tests.

Remaining within T5: result pagination, >8192-entry discovery, host preparation
resumption across timeout/next turn (current preparation starts fresh with a
10-second budget), tighter final delivery coordination, Windows source checking,
default Runtime/CLI activation, all remaining thirty acceptance rows and full
workspace/SDK/platform/remote gates. Unsupported or unfinished paths return
Unavailable; that does not satisfy their required acceptance. T1/T6 remain open.

### Search result cursor pagination — local, unmerged

Search now returns an opaque single-use next_cursor bound to its query and last
processed source ordinal. Pages retain the existing twenty-result, combined source
read and response-byte limits. A new search replaces the cursor; rediscovery
clears search cursors and detail references, while continuation discovery pages
retain them. Changed query, replayed cursor, stale scope/source and permission
changes refuse reuse. Detail references currently cover the latest search page;
retaining stable references across pages remains an acceptance gap.

A real durable 45-result fixture verifies three ordered pages, no skipped results,
query mismatch/replay rejection and final detail lookup. Actual tool-layer tests
verify rediscovery rejects old cursor and reference even with the new snapshot.
Focused logs: `/private/tmp/i313-pagination-tests.log`; final locked Agent 614/614
and warnings-denied all-target Clippy pass: `/private/tmp/i313-pagination-agent.log`,
`/private/tmp/i313-pagination-clippy.log`. Source-byte exhaustion still returns
Unavailable rather than a continuation page; long discovery, resumed preparation,
platform/default activation and final T5/workspace/remote closure remain open.

### Stable cross-page detail references — local, unmerged

History query state no longer retains a latest-page reference map. A fresh
per-discovery secret derives fixed-format ordinal references, allowing detail
lookup from older pages without memory proportional to searched history. The
reference is checked before scoped ordinal lookup; current index/source/permission
checks remain unchanged. Rediscovery rotates the secret, invalidating every prior
reference without retaining a revocation list. Source order is encoded in the
reference; it is client-uninterpreted, not encrypted or fully opaque.

The construction hashes a fixed domain, fixed-width ordinal and a private 32-byte
suffix from two random v4 UUIDs. Independent review approved this fixed-input use,
not a general MAC or signature. It does not provide constant-time comparison;
format/purpose expansion requires a standard keyed primitive. No new dependency
was added. Regression covers earlier-page detail after three pages, ordinal
modification, another discovery secret, noncanonical forms and rediscovery.
Logs: `/private/tmp/i313-stable-refs-agent.log`,
`/private/tmp/i313-stable-refs-clippy.log`. This supersedes the latest-page-only
reference limitation above; byte-budget continuation, long discovery, resumable
host preparation and all remaining T5/platform/remote gates remain open.

### Source-byte pagination — local, unmerged

The shared candidate reader now distinguishes page allowance exhaustion from
invalid/stale/cancelled evidence through RecoveryReadError. Search returns a
partial page and continuation without advancing past the unread candidate;
errors still refuse disclosure. A candidate that cannot fit even a fresh page
remains unavailable rather than issuing a nonadvancing cursor. The changed
shared-budget API is new and unpublished within this I313 branch; existing
single-candidate API behavior is preserved.

A real durable ten-record, approximately 3 MB fixture requires multiple pages
and verifies all ten results in order without duplication or omission. All four
checkpoint_history tests pass (`/private/tmp/i313-budget-tests.log`). Broader
validation and independent review remain pending; this is not T5 completion.

Budget pagination validation: locked Agent 616/616, Session 253/253 and affected
all-target warnings-denied Clippy pass (`/private/tmp/i313-budget-full-tests.log`,
`/private/tmp/i313-budget-clippy.log`). Both governance validators pass. Independent
static review approves the source-budget increment (reviewer did not run tests).
An additional actual tool/terminal regression confirms allowance exhaustion after
the original read, zero remaining allowance and successful fresh-page retry
(`/private/tmp/i313-budget-terminal.log`).

### Long-history recovery discovery — local, unmerged

Durable recovery discovery now works without a materialized compaction source.
It returns an execution/host-scope-bound snapshot and no compaction sources,
explicitly reporting compact_available=false. Search/detail retain source seal,
owner-scope, cancellation and permission checks. Discovery validates source and
index availability before publication, without I/O inside the permission fence.
This removes the 8192-entry discovery dependency without widening compaction
eligibility or constructing a whole-history reference map.

The real 8300-record durable tool fixture reaches the last original fact and its
detail, and covers scope replacement and permission reset. Initial six tool tests
passed; strengthened validation and independent review remain pending. Runtime
activation, preparation resumption, Windows and full T5 closure remain open.

Final long-discovery Agent 617/617 and affected warnings-denied all-target Clippy
pass (`/private/tmp/i313-long-final-agent.log`, `/private/tmp/i313-long-final-clippy.log`).
Both governance validators pass. Independent static review approves the revised
scope binding and availability checks; reviewer did not run tests. The existing
non-atomic filesystem-validation/permission-publication boundary remains open.

### Runtime activation — in progress, 2026-11-02

RuntimeBuilder now registers the built-in checkpoint/history tools through the
fallible Agent registration API; name conflicts fail construction rather than
silently replacing caller tools or granting builtin trust. A Runtime preview
regression checks default tool disclosure; validation is running at
`/private/tmp/i313-runtime-activation.log`. Preview alone does not certify live
provider recovery or restart behavior. CLI paths and real Runtime recovery
acceptance remain unfinished.

During this increment, 82 tracked files unexpectedly disappeared, including CLI
entrypoints and Runtime tests. Missing paths alone were restored from current
HEAD, retaining existing files; status was clean immediately after restoration.
Cause is unknown. This is environment recovery, not implementation progress.

Runtime activation regression now reopens a committed Session through SessionManager,
builds the actual Runtime facade and drives provider discover/search/detail before
ordinary continuation. The strengthened fixture now passes for explicitly permitted tool facts across
restart; checkpoint-omitted fact and fork scenarios remain to be verified.
Initial full Runtime run passed 76 tests and failed one obsolete default tool-set
assertion; the assertion now includes checkpoint/history while retaining skill_read
opt-in semantics. Added reserved-name collision refusal tests. Final validation
is running at `/private/tmp/i313-runtime-final.log` and
`/private/tmp/i313-runtime-clippy.log`; independent review requested.

Runtime activation final baseline: 78/78 tests, warnings-denied all-target Clippy
and both governance validators pass. The recovery fixture was then strengthened:
the original fact now exists only as an actually executed tool result with an
explicit recovery_projection. The producer Runtime commits and shuts down; a
new Runtime reopens the session without registering that tool and completes
search/detail/continuation. Original tool execution count stays one. Focused
regression passes (`/private/tmp/i313-runtime-tool-recovery.log`). This establishes
permitted tool-fact recovery across reopen, not checkpoint-omission/fork coverage.

### CLI activation — local, unmerged

Print, inline, interactive and TUI construction/rebuild paths now register the
same fallible checkpoint/history contributions before tool presentation. Locked
CLI all-target check passes (`/private/tmp/i313-cli-enable-check.log`). This is
compile evidence only; real CLI discovery/admission still needs walkthrough.
Legacy CLI persistence does not produce the DurableSession recovery receipts;
search/detail remain explicitly unavailable on that storage path. Runtime's
verified receipt path is not claimed as CLI durable recovery evidence.

CLI activation validation: all 384 binary unit tests pass
(`/private/tmp/i313-cli-enable-tests.log`). Independent static review approves
Runtime and all five CLI entrypoints, including TUI rebuilding, permission
pipeline retention and caller conflict refusal; reviewer did not run tests.
Actual CLI walkthrough, checkpoint omission/fork, resumable preparation, Windows,
full workspace/SDK and remote closure gates remain open.

CLI affected all-target warnings-denied Clippy passes
(`/private/tmp/i313-cli-enable-clippy.log`). A current CLI binary build is running
for the required real provider walkthrough (`/private/tmp/i313-cli-enable-build.log`);
this build is necessary because check/unit-test artifacts do not update the CLI
executable. No paid provider calls are planned.

Real inline CLI walkthrough now passes through a localhost SSE provider:
checkpoint/history definitions remain present, discovery returns sources and a
snapshot, legacy-storage search returns explicit execution-unavailable, and the
provider continues normally. Existing raw transcript prefix remains unchanged.
Reproducer: `python3 scripts/test_checkpoint_history_cli.py`; evidence:
`/private/tmp/i313-cli-history-walkthrough.log`. The first run expected the wrong
error wording; the corrected assertion checks the actual legacy-path error.
This certifies discovery/refusal/continuation, not successful CLI durable search
or model checkpoint installation. No paid calls were made.

### Actor-retained preparation progress — local, unmerged

Preparation now separates bounded steps from orchestration. DurableTurnPersistence
owns a shared pending preparation slot across Actor turns; the ten-second loop
retains completed-page cursor/parent and prefix scan state instead of always
starting a new scope. Each step has a two-second work budget; a step error or
cancellation discards pending authority. Parent state advances only after the
SQLite page commits. Queries still require a completed source seal.

Five focused tests pass (`/private/tmp/i313-resume-preparation.log`), including a
real >2 MiB preparation resumed after one committed page and rejection after a
source-version change. This is in-process retained progress only: normal appended
turns still require safe prefix revalidation, process restart recovery is absent,
and a timeout inside a step currently discards progress. Those remain acceptance
gaps, not delivered resumable-host completion. Independent review requested.

Preparation review correction: the operation deadline now starts before mutex
acquisition/initialization; lock waiting observes cancellation and timeout. Each
step's cooperative work deadline is capped by the remaining operation allowance.
Expired initialization is refused. The actual expired-operation path retains a
pending scope and the regression then resumes it successfully. Five focused tests
pass (`/private/tmp/i313-preparation-deadline.log`). Synchronous OS I/O is still
not hard-real-time bounded. The previous review requested changes on these bounds;
re-review is pending. Step-internal timeout classification and normal appended
turn/restart continuation remain unimplemented.

Revised preparation received independent static approval for stable-file,
in-process retained progress only; reviewer did not run tests. Agent 618/618
passed before the deadline correction, and corrected all-target warnings-denied
Clippy passes (`/private/tmp/i313-preparation-final-clippy.log`). Added a contention
regression: expired and cancelled waiters cannot consume pending work while its
mutex is held; releasing it permits the same scope to finish. Current focused
validation: `/private/tmp/i313-preparation-waiters.log`.

Source-step timeout refinement: typed HistoryPageError::Stopped from original-page
reads or prefix scanning now retains pending progress only when cancellation is
not active. Corruption/staleness and database failures still discard it. Seven
focused tests pass (`/private/tmp/i313-preparation-source-timeout.log`), including
actual expired seal scanning, cancelled refusal and successful unchanged-state
retry. This supersedes the all-step-timeouts-discard limitation only for source
reads/seal scanning; SQLite timeout, appended source and process restart remain
open. Independent review requested for this refinement.

Final retained-preparation Agent 620/620 passes
(`/private/tmp/i313-preparation-final-agent.log`); independent static review
approves source/seal timeout retention without certifying SQLite timeout, append
or restart continuation. Reviewer did not run tests. The source/seal refinement
above is the current boundary and supersedes earlier all-step-discard wording.

### Observed SQLite deadline interruption — local, unmerged

RecoveryWorkBudget now records when its own check/progress handler actually
observes deadline expiration, distinct from elapsed time and cancellation. Actor
preparation retains pending state after failed cursor/page/receipt maintenance
only with that observation and no cancellation. Successful completion still
requires the source seal and current owner scope. This does not make generic DB
errors retryable or confer source authority. Regression covers actual expired
SQLite cursor lookup without parent advancement and observed-vs-elapsed/cancel
semantics. Full tests/Clippy running in `/private/tmp/i313-sql-stop-tests.log`
and `/private/tmp/i313-sql-stop-clippy.log`; independent review pending. Normal
append and process restart remain open.

Observed-stop validation: Agent 620/620, Session 254/254 and affected all-target
warnings-denied Clippy pass (`/private/tmp/i313-sql-stop-tests.log`,
`/private/tmp/i313-sql-stop-clippy.log`). Independent static review approves;
reviewer did not run tests and notes the flag records an observed deadline stop,
not necessarily an error's sole cause. Retention never authorizes disclosure.
An additional real recursive SQLite query is interrupted mid-execution by its
50 ms budget after an uncommitted cursor update: transaction drop restores the
zero cursor and a fresh-budget transaction successfully retries. This focused
regression passes (`/private/tmp/i313-sql-interrupt.log`). Append/restart remain
open and this is not full host-recovery completion.

### Append-compatible preparation — local, unmerged

Preparation now completes its immutable-prefix seal before indexing source pages.
If the file version changes after sealing, it incrementally verifies the original
captured cutoff against that seal before continuing the existing scope/cursor.
Only a matching digest permits continuation; newly appended records are outside
that snapshot. Unsealed partial scans still fail on source-version changes.
Seven existing focused tests pass after phase reordering; a real commit-append
regression runs in `/private/tmp/i313-append-regression.log`. Independent review
is pending. This does not implement cross-process preparation recovery or certify
all concurrent mutation boundaries; those remain open.

Append review found a mixed-version indexing risk: restoring original bytes could
allow later digest verification to retain an index populated during a rewrite.
The current correction checks the existing seal after page reads, after index
commit and after receipt resolution, discarding preparation on version change.
Deadline retry additionally requires the seal still current. A deterministic
page-boundary mutation/restore regression checks preparation removal, zero cursor
advancement and query refusal. Validation log:
`/private/tmp/i313-append-mutation-tests.log`; re-review pending. Earlier append
success does not certify this concurrent-mutation boundary by itself.

Final append/concurrency validation: Agent and Session library suite passes
(`/private/tmp/i313-append-final-tests.log`, Session 255/255), affected all-target
warnings-denied Clippy passes (`/private/tmp/i313-append-final-clippy.log`). Revised
independent review approves the version checks; it does not certify every
possible concurrent mutation timing or cross-process recovery.

### Persisted prefix expectation — local, unmerged

Storage now exposes bounded versioned cutoff/digest metadata from a completed
seal. A newly configured source can revalidate that untrusted expectation only
through a complete bounded-page scan; deserialization or scan creation alone
never yields a seal. Real reopen with appended suffix preserves the original
cutoff; incomplete scanning, changed digest, unknown version and oversized cutoff
are refused (`/private/tmp/i313-persisted-expectation.log`). Actor persistence and
restart scheduling are not yet wired; this prerequisite does not complete restart
continuation. Source ownership/branch/permission binding remains the host's duty.

### Persisted preparation wiring — local, unmerged

Existing SessionIndex now stores bounded continuation metadata with exact
owner/scope/cursor and source-cutoff checks. Actor preparation saves completed
sealed stages and reopens by fully verifying persisted prefix expectations before
index continuation. Crash gaps between index and metadata commits refuse cursor
mismatch and rebuild rather than treating stale parent metadata as current.

Review found and prompted corrections for saving an old seal during append
revalidation, invalid persisted expectation initialization permanently blocking
fresh builds, and missing cutoff/build consistency. Corrected focused validation
is running (`/private/tmp/i313-restart-corrected.log`). Additional corruption and
independent-process regressions remain required. Current reopen fixture rebuilds
objects; it is not evidence of separate-process restart. This wiring is not yet
approved or delivered.

Restart correction validation now includes a separate test-binary child process:
a >2 MiB source is sealed and one index page committed in the parent, then a new
process loads the same scope/parent, fully revalidates the prefix and queries the
last original fact. Eleven focused tests plus the child test pass
(`/private/tmp/i313-restart-process.log`). This supersedes the object-reopen-only
evidence limitation above. Additional unknown-version rebuild and cutoff/build
mismatch refusal regressions are running (`/private/tmp/i313-restart-corruption.log`).
Independent re-review requested; full current workspace/platform gates remain open.

Twelve focused restart/corruption tests pass
(`/private/tmp/i313-restart-corruption.log`), including unknown-version fallback
and cutoff mismatch refusal. Independent static re-review approves this limited
increment; reviewer did not run tests. Added an explicit crash-gap regression:
commit one index page without updating saved metadata, then require a new scope
and correct rebuilt query rather than reusing a stale parent cursor. Log:
`/private/tmp/i313-restart-crash-gap.log`. Full affected suites and Clippy are
running in `/private/tmp/i313-restart-final-tests.log` and
`/private/tmp/i313-restart-final-clippy.log`.

Persisted continuation convergence: Agent 625/625 plus separate child test and
Session 256/256 pass; the additional crash-gap regression passes independently.
Warnings-denied affected all-target Clippy and both governance validators pass.
Logs: `/private/tmp/i313-restart-final-tests.log`,
`/private/tmp/i313-restart-final-clippy.log`, `/private/tmp/i313-restart-crash-gap.log`.
This closes this local storage increment's checks, not I313 acceptance or merge.

### Model checkpoint omitted-fact recovery — local, unmerged

The actual Actor/provider checkpoint fixture now stores original durable history,
installs a model-submitted checkpoint and asserts a unique original fact is absent
from the next projected provider request. It then rediscovers, searches and requests
detail before normal continuation. Initial attempt reused the consumed checkpoint
discovery snapshot and correctly failed; the fixture now follows rediscovery.
Current validation: `/private/tmp/i313-checkpoint-omitted-recovery.log`.
This is deterministic integration evidence, not real-model task-quality evaluation;
A→B and fork/restart composition remain separate required scenarios.

Omitted-fact Actor recovery now passes end-to-end
(`/private/tmp/i313-checkpoint-recovery-final.log`): Applied terminal metadata,
unique fact absent from the post-checkpoint request, rediscovery, original-source
search and detail, ordinary success, raw transcript byte-prefix preservation.
Counters assert exactly six provider requests, one compact_context invocation and
four history_search invocations (two discoveries/search/detail). This is one
checkpoint fixture, not A→B or real-model quality evidence.

### A→B recovery and discovery progress — local, unmerged

Expanded the real Actor fixture to two separately Applied model checkpoints,
then rediscovery/search/detail of an original fact omitted by both projections.
The first run exposed a genuine protocol collision: the third identical discover
call triggered doom-loop detection despite two successful checkpoint installs.
The current fix carries typed Applied status from the Actor receipt and resets
only built-in history_search discovery counters after Applied; other calls remain
counted, and failed/cancelled checkpoint receipts do not reset detection.
Validation running: `/private/tmp/i313-double-checkpoint-recovery.log`;
independent review requested. This is not a general doom-loop exemption.

A→B actual Actor recovery passes (`/private/tmp/i313-double-checkpoint-recovery.log`):
both checkpoints report typed Applied, original omitted fact is recovered by
search/detail, original transcript prefix is unchanged, with eight provider
requests/two compact calls/five history calls. Independent static review approves
the narrowly scoped discovery counter reset; reviewer did not run tests.
Added receipt regression for Applied versus Failed/Cancelled/NotEligible/channel
closure and single consumption. Full Agent/Clippy validation running at
`/private/tmp/i313-double-final-tests.log` and `/private/tmp/i313-double-final-clippy.log`.

A→B convergence: Agent 627/627 and affected warnings-denied all-target Clippy
pass; both governance validators pass. Logs:
`/private/tmp/i313-double-final-tests.log`, `/private/tmp/i313-double-final-clippy.log`.
Independent code review approves the typed Applied-only reset; negative receipt
regression now covers Failed/Cancelled/NotEligible/channel closure and one-time
consumption. Windows identity support and full I313 closure remain open.

Windows investigation: Rust 1.99 stable MetadataExt file_index,
volume_serial_number and change_time remain nightly-only in the pinned standard
library docs (https://doc.rust-lang.org/1.99.0/std/os/windows/fs/trait.MetadataExt.html).
Do not replace identity/change checking with size/mtime. A reviewed OS-boundary
implementation is still required; this investigation is not Windows support.

### Windows stamp investigation — unapproved local implementation

Added an isolated OS file-ID/basic-info sampling boundary and ADR-096 safety
analysis. Existing local seal tests pass. Full Windows cross-check is blocked by
missing Windows C headers for bundled SQLite; an isolated locked/offline build of
the actual stamp module passes x86_64-pc-windows-msvc type checking
(`/private/tmp/i313-windows-stamp-isolated.log`). Neither is native execution.

Independent security review rejects using this stamp to authorize reads while
shared writers remain possible: Microsoft File Times documents delayed timestamp
updates until modifying handles close. Windows same_file_version therefore still
returns UnsupportedSource. A stable handle denying write/delete sharing, with
correct lifetime across page scans and release before normal commit, remains
unimplemented. The two ABI calls' buffer/handle invariants were not rejected,
but their evidence semantics are not approved. No Windows completion claim.
Source: https://learn.microsoft.com/en-us/windows/win32/sysinfo/file-times

Windows lease follow-up (local, unmerged): recovery source opens now share only
READ, and sampled FileStamp retains a cloned handle in Arc through its lifetime.
Preparation timeout drops in-memory work on Windows instead of retaining a lease
across turns; persisted preparation metadata remains subject to fresh verification.
Added native Windows regressions for pre-existing writer refusal, retained-clone
write/rename exclusion, compatible readers and final-drop release. These tests
are not yet executed on Windows. same_file_version remains fail-closed pending
security review and native verification. Actor finalization ordering and timeout
release integration coverage remain required; this does not activate Windows.

Lease increment review approves preparation only, not Windows activation. Static
inspection confirms task-local History drops before successful finalization and
abort is joined before cancellation finalization. Reviewer ran no tests. Added
native-only real DurableSession regression for partial Preparation drop followed
by successful commit; this is not yet executed and is not full Actor coverage.
Pre-existing writable memory mappings remain an explicit activation blocker.
Local seal regression and isolated Windows stamp type check pass at
`/private/tmp/i313-lease-local.log` and `/private/tmp/i313-lease-cross.log`.

Current lease follow-up also passes locked talos-agent check and both governance
validators (zero warnings): `/private/tmp/i313-lease-agent-check.log`,
`/private/tmp/i313-lease-governance.log`, `/private/tmp/i313-lease-claims.log`.
Fetch completed; current implementation remains 37 local commits ahead and 21
commits behind origin/main. Integration and final exact-head gates remain open.

### Rebased native-validation preparation — 2026-11-02

Read-lease preparation committed as `341791924`, then all 38 local commits rebased
without conflicts onto `9ace8968603a67ada8d118459c81367611ad3fd2`; new lease head
`886170426`. Earlier local SHA references describe pre-rebase validation history,
not exact-head evidence for the rebased candidate. Standard workspace preflight is
running at `/private/tmp/i313-rebased-preflight.log`.

Added a native Windows CI diagnostic for an existing writable mapping after its
original file handle closes. It attempts READ-only sharing and records whether
mapped writes remain visible through that read handle; unexpected open errors or
inconclusive visibility fail the probe. Read buffering is disabled. Syntax parses
locally in PowerShell; native behavior has not run. This probe is evidence gathering,
not recovery authorization; the production Windows path remains fail-closed.

Rebase navigation map (not transferred exact-head test/review approval).
Earlier local execution entries retain their original source SHA; the following
rebased commits are their counterparts on the refreshed target baseline.

| Historical local commit | Rebased commit |
|---|---|
| `b87ce4e6707a8304561e09b57bd7e331fb9d5836` | `9e3f58b327bd7fafa2f2f1f40c3480ac4db42144` |
| `1dc42efe5f6a0b7941268b74c06123940e3f08b7` | `1eea0563a58dd772c3944926e218f1991476a582` |
| `348163e01e8320eefb8dae4badedff249dc8fcf5` | `cf2ae2949af12ac18baa8c5358666a04daa427c5` |
| `07cc2a6c9178cc75f6734c5d861a81977998979f` | `a949bd5d32adb5cdba641449ea7cfe70859d8935` |
| `f77bb29c5af0d85c3b4a9fd6c728cbbffe04127a` | `9cfb6084afde0a0e097cbffbd0ed746c68129148` |
| `0efa8eff9c833fb800020698eb0d9629d9c65330` | `8a84e9e51f22a9fb9518fbe70c75a48f48da6875` |
| `b6ced20b4687d235d5150d83f634d1a818150b8e` | `ba5ec4e57591c7f2ae0877d15a71c5b6c1decc07` |
| `a9feecd992b68c3b98043ece2f3c06c10fa31886` | `d9c4f4b84768db92fb8c6ce515b435a428d244fe` |
| `4fd715133bd3f26a563e9caab329bdf9b997aefb` | `c9d4f84e0047e23c8680d04ceefbe6c9b6f157de` |
| `63fd4d44f8dfbaac2022ae9360c65b7bc7281646` | `462bec08341522adad9551d0f3e4c49448703382` |
| `2e90833e62a6cf6643790fecc0d2f54336ffa9b6` | `4063c762e0edc0d3a74840b6d5f212fc6cc00abc` |
| `a4eb07c31cac25cb1b9b21e894f2c5bc8b9299d5` | `7aea88450513d255ceec8f576c1cea29a044e0fe` |
| `577d79b6063128613d092a64da45322a1aa2e6b4` | `842aff438ad220d0bd57259bd070e55be71aa922` |
| `4d7553df64cb1aae3438f02f557f8b1805b84ff1` | `b02fe62f944d933d689874c42806cd3793da9806` |
| `07e98dab7609dd198ccbc95fca6c387f1513abed` | `81ffa374fcc475aa77f11ebef9c680072afe98c2` |
| `404a838cd5734e12e3ec78e1b4a7f24316535b75` | `fc87e6c14d3e9bae9eab7021b5f9184ad806be9f` |
| `43e753733475645fe45a65811bd2abfeffe4c0bf` | `f832ab1cbff8f094faab434f8858262a5842dae9` |
| `b099e8253b6d05b0220315a247e2ffe881cc1a92` | `b2c7b22a386015c7cd01fcd11dc48da685673f7b` |
| `e2c557bbb83d2ba2c3ec2de8f62c5ce0bd1c4513` | `9cce32a9c301922876712f976bce64ab697ec124` |
| `3aff034f70bf71e9e42d8181c4f7a7968ee9371f` | `3470681097beaac276bd14a7b4322ce23b2f4c62` |
| `d08b9799c226b3792b7f8ba3bebd4e149759b27c` | `91f64e64d3cc52f39452fb955e6eaed9db676a02` |
| `c9ddf1360fbc710c087dd2eee12e78abd6545fbb` | `135cad7706b066c02b9672ec770ffae8a167f8fa` |
| `11db5036a2462fe7aa219082bc2f7766c259144d` | `00d40f1a214edc75ee940f434551ada2670f2a67` |
| `99d2a40b4a5333fb1ac2c42d39177b28b72d3cf3` | `7ba48dbb5c1b8da95db5c94f4abc23e9c95ed9aa` |
| `3322e2dcb1d3979c81fec16d4ef314fe1b49f4f6` | `e6de30c14c81b391533a1ea8ef176b8064587245` |
| `1a9437d76df13354cc6b773ce042866739aa973a` | `88659ba75dbe3f6782fbf48f3f042d83ab577afa` |
| `b34c4ec9877dfe0e0da8779bae02b6cbb705558d` | `d7c1fad6fb1dfc5da27ab3461e611838774adf94` |
| `cea39e7bc3c7a4fe90124b30c723aeb74122c8aa` | `339e5aba5b543e3f2ad2a8b4380a8f9e49c7727a` |
| `c87d2452fa7ebfa0705352393f4c7b48dea368b4` | `711a3476d4d430d5160f8c165afb5058a7641c60` |
| `3576e53ef6c15d61c6d25039604ebe5d8cc4e3ce` | `299415b25e88f38903942700e2e8a4c6c9f5979e` |
| `e65bf45544c553e2a9147b09165c87989869710f` | `d8a0ae1357d0c045702af761f8804ca7800de3f0` |
| `165e691094cebe5e4766cea041a8a4cf6c6aef82` | `4cc269004ff518fa297da26e4542061f739746aa` |
| `ae475ebe57e3d72c86fc88846ac6f8ef27e13ef9` | `982c5df75fa4b8f163dd5d9f3fcd2c1e9b36e0fc` |
| `1eaf3aa71b9117e41fd3d7fc6616fc25a0257b37` | `afecb0cbae9f0218ea2938598b809c90f327a786` |
| `adcf93a4d3978dc24ec56779695c9c45122c0b19` | `c25e28bf4e4ae8ca6b067a97be7ab0047ea227b4` |
| `188917ed21afca0c0dc88bbfcc3b66a49e81fc10` | `73ec499357047f13a8a3df6aa84787814dee8bff` |
| `b9462d0fd8059f61a45b94f0e45cc1a0ccd9dedc` | `166f16131df29f0fef3044b56ef84840ba852caf` |
| `34179192440768368f7902d53830b21063d75d1c` | `8861704261a58ae5767ec3d71918b556af472dc1` |

### Full candidate review and local gate — 2026-11-02

Independent static review REQUEST CHANGES on `6414552343e704d74a417a5b3a1ebed70b9a82f3`
against `9ace8968603a67ada8d118459c81367611ad3fd2`. Blocking acceptance gaps:
model receipt completion has no separately identified Session terminal when no
continuation consumes it; initial/unsealed large-source scans cannot persist
verified progress across budgets; actual fork/checkpoint/restart recovery coverage
and branch resolution remain incomplete; source/generation validity is not bound
to final permission-commit delivery; Windows remains unavailable; all thirty rows
still require current composition evidence. Incremental approvals do not approve
this complete scope. Next implementation: model-origin terminal status with no
manual-ID aliasing and receiver-closed/cancellation regression.

Standard preflight `/private/tmp/i313-rebased-preflight.log` terminated exit 101:
workspace check/Clippy passed; talos-tools had 1056 passing tests and one EPERM
failure in `prepared_local_push_runs_all_eight_admitted_stages` at process
observation. This is not a successful workspace gate. Authorized host preflight
is running at `/private/tmp/i313-rebased-preflight-host.log`; do not restart it
solely because observation expires. Both post-probe governance validators passed
with zero warnings (`/private/tmp/i313-mapping-governance.log`,
`/private/tmp/i313-mapping-claims.log`). Native mapping execution remains pending.

Model terminal fix (local): Actor ledger completion now emits the additive
ModelCheckpointTerminal event even when the oneshot receiver closed, retaining
model/manual ID separation. Inline/TUI display it without finishing a manual
command or turn. Updated coordinator regressions cover cancellation, closed
receiver and no duplicate terminal; actual A→B Actor fixture asserts exactly two
Applied terminals with model IDs 0 and 1. Validation is in progress. The existing
host preflight started before these Rust edits and cannot certify this new delta;
its result is baseline evidence only, even if later SDK builds see updated sources.

Terminal increment validation: Agent 627/627 and CLI 384/384 pass; affected
core/agent/CLI all-target warnings-denied Clippy passes. Logs:
`/private/tmp/i313-model-terminal-agent.log`, `/private/tmp/i313-model-terminal-cli.log`,
`/private/tmp/i313-model-terminal-clippy.log`. Independent static review approves
this delta only; normal Actor lifetime sends once, not durable delivery after
process/receiver destruction. SDK lock correction `a7a623524` adds only existing
Session libc/windows-sys dependency edges; both locked external fixture modes now
pass (`/private/tmp/i313-sdk-lock-fixed.log`). Host preflight itself exited 1 at
its stale SDK lock; do not relabel it successful. New complete-candidate preflight
and native CI remain required, alongside the other full-review blockers.

### A→B then fresh Actor reopen recovery — local

Extended the actual registered model checkpoint fixture: after both Applied
checkpoints and omitted-fact recovery, shut down/drop the original Actor and
DurableSession, reopen the durable store, and build a fresh Actor/model. The new
Actor starts without a checkpoint projection, replays durable raw messages, binds
fresh history sources, then performs discovery/search/detail and ordinary success.
Assertions require four new provider requests, no reexecuted/replayed checkpoint
terminals, unchanged replay prefix, and append-only original persisted bytes.
Reopen calls use distinct discovery/search/detail IDs and require their paired
results; historical tool IDs cannot supply the previous discovery or reference.
Independent review caught that reverse selection alone could fall back to old
results, so the initial reused-ID version was corrected before convergence.

Focused test passes (`/private/tmp/i313-double-restart.log`). This covers same-
process object/store reopen with raw-history rebuild; it does not certify an
independent process restart, persisted checkpoint projection, fork ancestry or
all Issue #390 semantic rows. Those remaining requirements are not waived.

Revised unique-ID fixture passes; affected all-target warnings-denied Clippy and
both governance validators pass. Logs: `/private/tmp/i313-double-restart-clippy.log`,
`/private/tmp/i313-double-restart-governance.log`, `/private/tmp/i313-double-restart-claims.log`.
Independent static review approves the corrected fixture only, without executing
tests or certifying cross-process/fork/full I313 acceptance.

### History result post-query revalidation — local

HistorySearch now revalidates the issued/current source or recovery-only token/scope
after query construction. It retains the execution-state guard through the
permission snapshot commit, using discovery's state→permission lock order. Source
stamp and SQLite owner-scope checks run before acquiring the permission lock;
cancellation and the shared two-second delivery deadline are checked inside commit.
The added scope check uses only the remaining query/delivery budget.

Execution refresh now tracks valid-source generation: explicit source withdrawal
or a generation change clears issued/recovery/cursor state; a valid same-generation
append preserves recovery discovery. Deterministic post-query hooks test both
compaction-eligible and recovery-only paths against same-size source mutation,
execution withdrawal, changed generation with the same entry IDs, permission
reset and cancellation. Each starts with a successful healthy query; same-generation
append is a positive control. All eight checkpoint_tools tests pass at
`/private/tmp/i313-history-delivery-race.log`; full Agent/Clippy are running.

Independent static review approves the revised increment only. This narrows the
source/index-to-permission delivery window; it does not make external file/index
changes atomic with permission commit or synchronous I/O hard real-time. The
complete delivery coordination requirement remains open, alongside large-source
persistent verification, fork composition, Windows and final candidate gates.

Current delta passes Agent 628/628 and affected all-target warnings-denied Clippy:
`/private/tmp/i313-history-delivery-agent.log`, `/private/tmp/i313-history-delivery-clippy.log`.
These are local incremental checks, not full candidate platform/merge approval.

### A→B then independent-process Actor recovery — local

The prior same-process reopen fixture is now strengthened to a separate test
process. Parent completes both checkpoints, shuts down/drops its Actor and durable
handle, then launches the exact current test with only root path and parent PID.
Child asserts a distinct PID, opens durable state and creates a fresh Actor/model,
performs the four recovery requests with unique reopen call IDs, verifies no
checkpoint replay and unchanged raw prefix, and emits a completion marker. Parent
requires successful exit plus that marker (not merely an empty test selection),
and verifies the child's append preserves the parent's complete persisted prefix.
The subprocess wait has a 30-second timeout and kill-on-drop.

Focused regression passes (`/private/tmp/i313-checkpoint-process-restart.log`).
This supersedes the same-process-only limitation for this raw-rebuild scenario;
it does not establish persisted checkpoint projections, fork/shared-ancestor
recovery, all semantic scenario rows, Windows or final candidate completion.

Affected all-target warnings-denied Clippy passes
(`/private/tmp/i313-checkpoint-process-clippy.log`). Independent static review
approves the subprocess fixture, including distinct PID, exact test selection,
completion marker, dedicated call IDs and bounded child lifetime; reviewer did
not execute tests. This remains incremental evidence, not whole-I313 approval.

### Initial sealing sub-page progress — local, uncommitted

Initial scan now reads at most 64 KiB per step and retains at most one 2 MiB
pending page. Only completed original-sized pages update the rolling digest;
version-one expectation bytes remain unchanged. Previously a deadline during a
2 MiB read discarded the entire page, allowing repeated short budgets to stall.
The new regression interposes expiry between each successful chunk, asserts
retained progress and memory bounds, and compares the final digest against the
original full-page algorithm over a source larger than one page. Four seal tests
pass (`/private/tmp/i313-seal-subpage.log`); host preparation tests and static review
are pending. This fixes in-memory sub-page progress only, not persistent initial
verification across process restart or Windows lease release. Those remain open.

Sub-page follow-up: clarified that a final interruption may retain pending bytes
but never advances committed offset/digest. Added a deterministic stopped-check
regression at the post-hash boundary with a complete pending page; retry performs
no duplicate append and produces exactly the uninterrupted digest. Five seal tests
pass (`/private/tmp/i313-seal-subpage-final.log`); thirteen Agent host-preparation
tests pass (`/private/tmp/i313-subpage-agent.log`). Independent static review
approves the page-size/digest/lifetime increment, not persistent restart support.
The new post-hash test uses the shared stopped-check branch rather than a wall-clock
sleep and does not claim a hard real-time filesystem deadline.

### Persistent initial-scan authority design — activation pending

Independent architecture/security review approves implementation of the ADR-096
amendment using existing host-owned durable bindings for scan provenance, separate
from the untrusted search cache. This is a deliberate trusted-storage extension,
not cryptographic integrity; even provenance-only host tampering is outside its
promise. No extra user authorization or scope reduction is required. Restoration
is not implemented or activated yet. APIs must accept only actual Session scans,
bind configured owner/branch and opened-file version, validate page-aligned bounds,
atomically persist bounded data with cancellation/CAS, and never bypass full scan
completion or index verification. Windows remains fail-closed.

Implemented a private serde Unix source-version value and reused it for existing
same-file comparison without changing the compared fields. Regression covers
roundtrip, read-only access and equal-length file replacement. This is groundwork,
not cross-process initial-scan progress; persistence/restore and all acceptance
regressions remain required.

Persistent-scan implementation draft (uncommitted, not Actor-activated): private
StoredScan binds version/session/branch/cutoff/completed offset/digest/expected
prefix and Unix opened-file version. Session save accepts only an actual scan
issued by its configured handle; bindings transaction checks the configured
external-ID/UUID mapping and source version before/after write. Restore rejects
malformed/out-of-range/non-page-aligned/zero-offset-invalid state and mismatched
source versions; it returns a scan, never a ready seal. A new bindings table holds
only bounded provenance JSON, not raw messages or search content.

Locked Session check passes (`/private/tmp/i313-scan-store-check.log`). Added a
nonzero-page save/reopen/resume regression with cross-owner save refusal and
append invalidation; validation is running. Draft API still requires bounded
cancellation throughout SQLite work, concurrent stale-owner CAS, lifecycle cleanup,
independent process/tamper tests, final API/security review and Actor integration.
Do not activate or claim persistence acceptance from this draft.

Scan-store reopen failure resolved: generic Session construction minted a random
root branch on every reopen. Durable factories now bind their single root branch
to the persistent UUID; generic Session/fork construction is unchanged and owner
Arc provenance remains mandatory. Nonzero-page restore passes with exact source
version. Scan-store methods take existing RecoveryWorkBudget, use cancellation/
progress handlers and zero busy wait, and skip initialization retries. The same
transaction rejects regression to an older offset for the same stamped source.
Tests cover cross-owner save refusal, cancelled save/load, old-page overwrite
refusal, reopen from nonzero offset, completion and append invalidation.

Full Session tests pass 260/260 (`/private/tmp/i313-scan-store-session.log`). This
is still an uncommitted implementation draft: independent review is pending,
Actor wiring, lifecycle cleanup, panic containment audit, cross-process and tamper
matrix validation remain required. No persistent-initial-scan delivery claim.

Scan-store review follow-up (still uncommitted/not Actor-activated): wrapped full
save/restore/clear database operations in panic containment. Added exact-payload
cleanup scoped by configured external-ID/UUID so old progress cannot delete a
newer checkpoint, plus a bindings-delete trigger for lifecycle removal. Regression
injects panic after transaction insert and confirms rollback; binding deletion
removes provenance; stale cleanup preserves newer progress and exact cleanup
removes it. ADR now records canonical durable-root migration/cache invalidation
without transcript rewrite and distinguishes stable branch IDs from handle owner
authority. Full Session validation running at `/private/tmp/i313-scan-cleanup-tests.log`.
Concurrent-owner/source-version races, process restart, Actor integration and
complete review remain required.

Cleanup/panic follow-up Session suite passes 261/261. The first test compile used
unsupported SQLite u64 count decoding; corrected to i64 and reran successfully.
No successful result is claimed for that initial compile. Current log:
`/private/tmp/i313-scan-cleanup-tests.log`.

Scan cleanup receipt follow-up: save returns opaque RecoveryScanReceipt only after
commit, binding owner Arc and exact serialized record. Each save includes a fresh
record UUID, preventing an older receipt from deleting a same-offset resave.
Clear no longer rereads or requires an unchanged source; it checks receipt owner,
configured binding and exact stored payload. New regression covers obsolete
same-offset receipt, foreign independently reopened handle, append-then-cleanup
and idempotent repeat. Full Session tests pass 262/262
(`/private/tmp/i313-scan-receipt-session.log`). Actor integration and concurrent/
independent-process scan-store validation remain open; this remains uncommitted.

### Unix initial-scan preparation wiring — local draft

Restore now returns RecoveryScanResume containing the scan and an exact cleanup
receipt bound to the restoring handle. Unix Preparation owns the durable handle,
latest receipt and last persisted completed-byte count. Without an existing index
continuation, begin restores valid host scan progress; persist saves only when a
complete-page offset advances. Completion/failure/cancellation retirement uses
the latest exact receipt. No partial scan yields History authorization.

New actual host-preparation regression scans 32 chunks of a 3 MiB history, saves
2 MiB progress, drops/reopens the owner, verifies nonzero resumed offset and cleanup
receipt, then completes preparation, queries the last fact and verifies record
retirement. All fourteen host-preparation tests pass
(`/private/tmp/i313-scan-actor.log`). This fixture is same-process reopen; separate
process initial-scan and concurrent-owner races remain required. Existing index
continuation revalidation still uses its original full-prefix scan path and is
not claimed to have complete persistent verification. Windows remains disabled.
Incremental safety review requested before committing this draft.

Source/abort review corrections (local): restore_recovery_scan_for_source checks
handle provenance and uses the exact already-captured source; Preparation no longer
mixes its cutoff with a second capture. New deterministic test captures old source,
appends, saves newer-owner progress and verifies old-cutoff refusal, foreign-handle
source refusal, and explicit fresh-source success. Session regression passes at
`/private/tmp/i313-scan-source-race.log`.

Unix preparation scan state now has a cancellation-aware Drop owner for its receipt;
Actor-style token cancellation followed by task abort/await is tested for saved
record retirement. Cleanup uses a short cooperative SQLite budget and can still
fail on busy/error; durable guaranteed deletion is not claimed. Source/reset and
abort changes require review. An initial compile referenced the old field name;
corrected before rerunning host-preparation tests at
`/private/tmp/i313-scan-drop-tests.log`.

### Initial scan independent-process and owner ordering evidence

The initial-scan host fixture now launches a separate exact test process after
persisting 2 MiB and dropping the original preparation/owner. Child asserts distinct
PID and nonzero restored offset, completes preparation, queries the last fact and
checks record retirement. Parent requires successful exit and a child-only marker;
30-second timeout/kill-on-drop bounds subprocess lifecycle. This passes at
`/private/tmp/i313-initial-scan-process.log`; it supersedes the same-process-only
limit for this initial scan scenario, not later index-prefix verification.

Added two separately reopened owners on distinct threads: newer owner commits a
complete page before releasing the older writer, whose lower-offset save must fail;
a third owner verifies the persisted nonzero offset. This is deterministic
interleaving, not a proof of every simultaneous busy/lock schedule. Full Session
suite passes (`/private/tmp/i313-scan-owners-session.log`). Source-bound restore and
cancelled-task receipt cleanup have independent incremental static approval, with
synchronous Drop I/O and ignored cleanup errors explicitly still limited.

Unix initial-scan stage converged locally: Session 264/264, Agent 630/630 and
Session/Agent all-target warnings-denied Clippy pass. Logs:
`/private/tmp/i313-scan-owners-session.log`, `/private/tmp/i313-scan-final-agent.log`,
`/private/tmp/i313-scan-final-clippy.log`. Independent full-stage static review
approves this bounded Unix initial-scan chain against `37901a322`; reviewer did
not run tests or approve full I313/merge. Windows, index-continuation prefix
revalidation, fork, cleanup error observability and final platform/remote gates
remain open. These residuals are not waived by the stage approval.

### Index-prefix verification restart — local follow-up

Initial-scan stage committed as `20f65827d`. The next uncommitted delta restores
host-issued progress for an existing index continuation only when version, cutoff
and expected digest match exactly. No seal or query authority is issued early.
The independent-process index-restart regression now interrupts verification at
2 MiB and requires the child to resume that nonzero offset before querying the
last fact. It passes at `/private/tmp/i313-prefix-resume.log`; exact expectation
negative controls (initial scan, changed digest/cutoff/version) pass at
`/private/tmp/i313-prefix-expectation.log`. Formatting and diff checks pass.
Static review is requested; same-source provenance with a different expectation
still needs lifecycle/overwrite analysis and regression before this delta commits.
Full I313 platform, fork, cleanup and final delivery gates remain outstanding.

Prefix restart review correction: incompatible scan provenance is now retired by
its exact receipt before a fresh verification starts. Cleanup failure or a changed
record returns unavailable for this attempt, never authorizes unconditional
replacement. The initial-scan-to-index transition now persists actual initial
progress before the independent child resumes verification. Added changed-digest
cache regression: restart at zero, save replacement progress, obsolete receipt
cannot clear it, forged digest cannot finish a seal. All sixteen host preparation
tests pass (`/private/tmp/i313-prefix-host-suite.log`); transition subprocess also
passes (`/private/tmp/i313-prefix-transition.log`). Fresh static review requested.

Revised increment receives independent static approval against `20f65827d`
(code diff SHA256 `e114c44ee60327a7605a5873e2af533bbf698c7e07b2271f654167580adf3667`).
Reviewer did not run tests or approve full I313. The forged-digest host regression
proves no seal is issued, but does not distinguish digest rejection from timeout;
no stronger terminal-cause claim is made. Affected all-target Clippy passes at
`/private/tmp/i313-prefix-clippy.log`. Fetch confirms no missing origin/main
commits at this stage; no rebase is necessary.

### Cleanup failure observability and bounded retries

Scan retirement and cancelled-task Drop now share exact-receipt cleanup with at
most three immediate attempts per invocation and content-free debug/warn events.
Real SQLite BEGIN IMMEDIATE contention regression proves failure retains a usable
receipt, rollback permits retry, and successful deletion is idempotent
(`/private/tmp/i313-cleanup-busy.log`). No durable cleanup guarantee is claimed:
sustained locks can defeat immediate retries, budgets are cooperative rather than
hard I/O deadlines, and retire plus Drop can each invoke a retry group.
Independent static review approves the two-file delta against `1404a247c`
(SHA256 `d616cea45ab3f86041bc484d1d7bd698975403583b50798226c3932b8e887cbf`),
not full I313; reviewer did not run tests. Host preparation suite passed before
the final common-helper deduplication; final affected checks are recorded below.

Final cleanup delta host suite passes 16/16
(`/private/tmp/i313-cleanup-final-host.log`); affected all-target warnings-denied
Clippy passes (`/private/tmp/i313-cleanup-clippy.log`). Next uncovered acceptance
is actual durable fork integration: generic CLI fork does not supply durable
bindings/lineage authority. A host-only bounded prefix-fork design is under
independent architecture review; no fork implementation is claimed yet.

### Durable fork groundwork — uncommitted

Independent architecture review accepts a host-only committed-tip fork direction,
not an implementation: Actor idle/admission gate, bounded verified source copy,
new durable identity and atomic trusted lineage publication, no copied index or
checkpoint authority, and real ancestor/sibling/restart integration evidence are
required. Source inspection corrects a design assumption: TLOG headers contain
format version and creation time, not session UUID.

Added draft RecoveryPrefixSeal staging-copy primitive with 64 KiB reads, source
checks after writes and cooperative cancellation/deadline. Raw-prefix equality,
pre-cancelled empty output and writer-induced source mutation refusal pass at
`/private/tmp/i313-fork-copy.log`. Partial staging output must be discarded on
error. This is not a published child or working fork; API review, publication/
lineage/crash protocol and Runtime integration remain unimplemented.

Durable fork storage draft now streams source verification and page-wise turn
terminal checks, creates a fresh private TLOG, synchronizes it, and publishes the
new external binding plus lineage in one transaction. Existing IDs fail rather
than reopening; parent binding is rechecked. Normal errors remove unpublished
files; crash orphan recovery is not implemented. The whole storage operation is
panic-contained. Narrow actual fork/reopen test verifies original raw copy,
ancestor plus child suffix, parent/sibling exclusion, collision/cancel refusal
and owner-capability separation (`/private/tmp/i313-fork-storage-test.log`).
Initial compilation failed on SQLite u64 parameters; checked i64 conversion fixes
that failure. Full tool pairing, Actor idle gate, orphan recovery, independent
storage review and Runtime/process-restart composition remain outstanding.
No completed fork delivery or full acceptance claim; this draft is uncommitted.

Fork storage review requests changes. Corrected shared file/SQLite deadline to
at most ten seconds, moved copy/fsync before the publication transaction, and
preserve the child file once commit is attempted because failure can have an
uncertain outcome. Added parent-chain and pending tool pairing checks (at most
8192 outstanding calls). Unpaired-call terminal refusal and existing fork/reopen
regressions pass (`/private/tmp/i313-fork-pairing.log`). Duplicate entry/turn/call
identity detection, byte bounds for pending identities, crash reconciliation,
Actor gate and complete independent approval remain open. This remains a draft,
not activated fork delivery; current narrow tests do not certify those gaps.

Fork identity validation now uses disposable TEMP tables on the existing bindings
connection for entry/turn/call uniqueness and pending call completion. FILE temp
storage and a 256 KiB temp cache avoid a transcript-sized Rust identity set; this
is not a measured total process-memory guarantee. No persistent second database
is introduced. This supersedes the draft 8192 pending-call cap. Duplicate entry
and reopened terminal turn fixtures refuse child publication; three storage fork
tests pass (`/private/tmp/i313-fork-identities-final.log`). Revised independent
storage review requested. Runtime idle gate, crash reconciliation, real tool-fact
fork/restart composition and final gates still remain; draft stays uncommitted.

Call-identity review correction: tool-call uniqueness/pairing is scoped to
(turn_id, call_id), preserving legitimate ID reuse across turns. The generic
extractor silently skips malformed blocks and invents missing IDs, so fork
validation now strictly parses each JSON block, requires ID/name/args and a closing
fence, and handles fences embedded inside JSON strings. Positive repeated-ID and
embedded-fence fixtures plus malformed/missing-ID/unclosed refusals pass; all four
fork storage tests pass (`/private/tmp/i313-fork-call-scope.log`). The temporary
SQLite cache setting is not a hard RSS or disk quota. Deadline cleanup evidence,
crash reconciliation and Actor integration still remain before activation.

### Draft Runtime fork path

Added host SessionOp::ForkDurable with direct completion receipt and RuntimeHandle
fork_durable admission. Actor rejects active turns, running compaction, pending
submissions or shutdown; successful storage fork leaves the active owner unchanged.
Receipt closure/admission shutdown requests cooperative cancellation. Runtime check
and actual submit→complete→fork→reopen/history-equality/collision regression pass
(`/private/tmp/i313-fork-runtime-check.log`, `/private/tmp/i313-fork-runtime-test.log`).
This is an uncommitted API draft under independent review. Synchronous storage
currently occupies Actor execution; no hard cancellation guarantee is claimed.
Busy/race/forced-shutdown tests, crash reconciliation, API/ADR docs, full tool-fact
checkpoint/fork/restart recovery and full gates remain before merge/activation.

Runtime gate review correction: dedicated is_idle_for_fork includes queued manual,
queued model, running and retained model receipt/boundary work, rather than just
is_running. Missing in-process receipt now emits an explicit unsupported-operation
error instead of silently ignoring deserialized fork commands. Actual gated-model
runtime test proves busy rejection with no child then successful fork after turn
completion; both Runtime fork tests pass (`/private/tmp/i313-fork-busy.log`).
Queued manual predicate regression is under validation. Direct Session queued
Interrupt/Shutdown cannot interrupt the current synchronous storage call; only
receipt closure/admission closure are polled. This remains an unresolved lifecycle
limitation, not full cancellation acceptance. Public SessionOp migration and crash
reconciliation are also required before this uncommitted API draft can merge.

Fork worker lifecycle draft replaces synchronous Actor I/O: Actor retains a
ForkWork busy owner and polls a blocking worker, suppresses turn/compaction start,
rejects new work (tracked submissions receive rejection), preserves observational
reconciliation, and processes global Interrupt/Shutdown cancellation. Normal
shutdown waits for worker exit; dropping the Actor cancels its token but cannot
forcibly abort a blocking kernel call. Two Runtime fork regressions pass before
the final tracked-receipt handling correction (`/private/tmp/i313-fork-worker.log`);
final Runtime check passes (`/private/tmp/i313-fork-worker-check.log`). Queued manual
idle-predicate test also passed (`/private/tmp/i313-fork-queued.log`). Independent
lifecycle review and deterministic worker-gate cancellation/shutdown tests remain,
as do crash reconciliation, public API migration and full T5 fork acceptance.

Worker review corrections: deadline captured before spawn_blocking, compaction
gets operation-specific Busy/StaleGeneration, and temporary structured rejection
uses new SessionBusy rather than SessionClosed. Runtime/CLI check passes before
test-hook additions (`/private/tmp/i313-fork-busy-protocol.log`). Deterministic
worker gate regression passes: Compact Busy, global Interrupt observed while
worker is held, no child on cancellation, then successful retry and shutdown
(`/private/tmp/i313-fork-worker-gate.log`). ADR-096 records draft public-enum
migration and in-process-only fork receipt protocol. Shutdown-in-flight, receipt
drop, Actor abort, publication race and crash reconciliation still require proof.

Deterministic worker lifecycle suite now passes both tests, including Shutdown
while held and Actor abort. Abort test waits for the blocking worker's final
receipt sender to drop before asserting no child binding, so it observes worker
exit rather than merely token cancellation (`/private/tmp/i313-fork-worker-lifecycle.log`).
These gates precede storage publication and do not prove cancellation during
fsync/commit. Receipt-drop and publication races remain required.

Independent recovery design review accepts persistent intent in existing bindings,
per-fork create_new lock plus std File locking, and idempotent recovery. Intent
contains only UUID-derived paths, must precede child writes, and publication joins
binding/lineage in one transaction. Recovery must acquire the lock then reread
intent/binding; preserve published children, reject symlink/nonregular cleanup,
retain lock files to avoid inode replacement races, and bound records/time/I/O.
No crash recovery implementation is claimed yet. Independent-process crash-point
and filesystem lock semantics tests remain required before fork activation.

Fork intent/recovery implementation draft: create_new private UUID lock held by
worker, synchronized before intent commit; intent precedes child bytes, publication
transaction validates it, successful publication retires intent best-effort. Lock
inodes remain. New bounded recover_fork_intents checks at most 32 fixed UUID paths,
skips held locks, rereads binding after lock, preserves published lineage/cutoff,
and deletes only exact unbound regular files before retiring intent. Same-process
lock exclusion, unrelated-file preservation and idempotency pass
(`/private/tmp/i313-fork-recovery-test.log`); four storage fork tests still pass
(`/private/tmp/i313-fork-recovery-check.log`). Startup integration, cross-process
lock/crash matrix, published preservation tests, review and pagination fairness
remain; this is not complete crash recovery acceptance or a merged delivery.

Recovery review correction adds keyset continuation and reports scanned/retired/
failed/next_cursor. Per-record failures retain the intent and permit other records
to proceed; global budget/connection failure still terminates the operation.
New regression reaches a stale record after 32 missing-lock failures, then
preserves a published child with simulated leftover intent. Both recovery tests
pass (`/private/tmp/i313-fork-recovery-pages.log`). Published checks certify only
binding/lineage/length correspondence and preservation, not content integrity.
The planned symlink negative regression remains unimplemented. Startup caller,
actual independent-process crash/lock matrix and fresh review remain required.

Recovery negative/platform evidence: symlink lock and symlink child fixtures keep
both the target bytes and failed intents. Independent child test process holds
the actual std file lock; parent recovery skips it, then kill/wait releases it and
recovery removes only the recorded partial child. A child-process guard bounds
startup and kills/reaps on failure. All four recovery tests pass
(`/private/tmp/i313-fork-recovery-process.log`). This verifies current macOS/local
filesystem lock behavior and a seeded orphan, not the full real-fork crash-point
matrix or other filesystems/platforms. Startup integration remains outstanding.

Actual fork crash-path regression now exits a dedicated child process without
Rust destructors after intent, copy, sync, precommit and published commit. Parent
recovery retires all five intents, removes unbound files, retains published raw
child bytes and preserves parent bytes (`/private/tmp/i313-fork-real-crashes.log`).
This is process exit, not power-loss proof; mid-copy kill still remains.

Runtime startup now sweeps paged intents under one cooperative two-second budget,
reports content-free unresolved counts/failures, and retains failed records for
retry. Rebuild regression confirms duplicate-fork residual intent retired while
existing child remains; both Runtime fork tests pass
(`/private/tmp/i313-fork-startup-final.log`). Initial compile lacked tracing in
Runtime; added an edge to the already-locked tracing version (no package changes),
then reran locked tests successfully. External SDK fixture lock synchronization
and broader gates remain outstanding; no complete fork/T5 claim.

Fork stage convergence: actual six-point process-exit matrix now includes a
verified nonempty partial 64 KiB copy (`/private/tmp/i313-fork-midcopy.log`).
Affected Session/Agent/Runtime all-target warnings-denied Clippy passes
(`/private/tmp/i313-fork-converge-clippy.log`); full library suites pass Agent
634/634, Runtime 80/80, Session 276/276 (`/private/tmp/i313-fork-converge-tests.log`).
Independent static review approves this bounded stage against `ebe09bd0d`, code/
lock diff SHA256 `5b22f4637e17de5204c6149300b0d54fa4ad12fe962dfe948951530854a50fe8`,
not full I313. Reviewer did not run tests. SDK lock now adds only existing tracing
edge; unrelated offline resolver downgrades were discarded. SDK modes underway.

Residual boundaries: a sweep budget error loses continuation, so arbitrary
backlog eventual cleanup is not promised; retained lock files are intentional to
avoid inode races. Recovery preserves published binding/lineage/length, not
content integrity. Cancellation may race successful publication; kernel I/O is
not forcibly interruptible. Real tool-fact→checkpoint→fork→independent-process
recovery, Windows support and final full-candidate gates remain outstanding.

Both external SDK fixture modes pass (`/private/tmp/i313-fork-sdk.log`); both
governance validators pass with zero warnings (`/private/tmp/i313-fork-governance.log`,
`/private/tmp/i313-fork-claims.log`). Staged code/lock/ADR/owner diff reviewed; SDK
contract records host usage and next-minor enum migration. These are stage-local
checks, not standard final preflight/platform CI/merge evidence.

### Actual Runtime tool fact across fork and independent process

Extended the existing recovery fixture to retain ordinary reopen and add a real
Runtime-produced recoverable tool fact, Runtime child/sibling fork, and separate
parent/sibling suffixes containing the same query keyword. Child recovery runs in
a distinct test process, discovers/searches/details the original tool fact, and
requires exactly one search result. The original tool executes once and is not
registered in the recovery runtime. Parent raw prefix remains unchanged. Parent
and child tests pass (`/private/tmp/i313-fork-tool-process.log`). This is actual
fork/restart/tool-projection evidence, but does not yet combine checkpoint A→B
with fork; that final combined scenario and full T5 gates remain open. Delta is
uncommitted pending test review.

Runtime subprocess regression now requires child-only success marker as well as
exit zero, closing the zero-selected-tests false-positive gap; passes at
`/private/tmp/i313-fork-tool-marker.log`. Existing Actor A→B fixture now invokes
real ForkDurable after both Applied terminals, verifies new identity/raw equality,
and independently reopens both parent and child using explicit existing binding
identity and required success marker. This passes at
`/private/tmp/i313-checkpoint-fork-process.log`.
Independent static review approves the two-test delta (SHA256
`d8d9c2ba8acbe6cdd904e37fda4560b0cf688bd7fa690b667ce891e2a2291022`);
reviewer did not run tests. Runtime covers tool fact/sibling exclusion; Actor covers
A→B/fork/original user fact. Combined tool fact+A→B+fork remains unverified, and
these separate tests must not be represented as that single scenario.

Combined Actor regression now produces the sole omitted fact through an admitted
registered read tool (execution count exactly one), persists that real tool turn,
then appends older-context filler before checkpoint A→B. The existing assertion
requires the fact to leave the projected request; discovery/search/detail recovers
it, actual ForkDurable copies the committed tip, and independent processes reopen
both parent and child with required success markers. Locked targeted test passes
(`/private/tmp/i313-combined-tool-checkpoint-fork.log`). An initial fixture put the
fact in the retained recent turn and correctly failed the omission assertion;
chronological placement was corrected, not the assertion. This combined delta is
uncommitted and awaits independent review; full I313 acceptance/platform/final
preflight/remote gates remain outstanding.

Fork lifecycle regression additionally closes the receipt receiver while the
worker is held before publication, awaits channel exhaustion (all senders retired)
without Shutdown/Interrupt, and confirms no child binding before shutting down
the responsive Actor. Locked targeted test passes
(`/private/tmp/i313-fork-receipt-retirement.log`). This covers receipt cancellation
before publication; cancellation concurrent with committed publication remains a
separate race boundary and is not certified by this test.

Independent static review approves the combined two-file test delta against
`cec780133`, SHA256
`8ac43bb3f7fd8ee82677a12eec751fad45a79a2a64a579186d77be37fb4835b9`.
Reviewer did not run tests and does not approve full I313. On reopen, ordinary
raw history includes earlier recovery outputs: fresh call IDs and results prove
new history_search/detail execution, not absence of the fact from every ordinary
replayed message. Both governance validators are running for this local evidence stage.

### Deterministic fork cancellation publication boundaries

Stage `23e75dd56` committed the combined recovery/receipt-close regressions;
both governance validators finished with zero warnings (combined-governance and
combined-claims logs), superseding the earlier running snapshot.

New test-only thread-local probes cancel immediately before the final publication
check and immediately after successful commit. The regression requires the probe
to fire, verifies rollback/no child in the first case and successful durable child
with identical raw bytes in the latter, and preserves parent bytes in both cases.
This proves boundary behavior, not every scheduling race during commit or between
last check and commit. Production behavior is unchanged. Targeted regression,
full Session library suite (277/277), and all-target warnings-denied Clippy pass:
`/private/tmp/i313-fork-publication-cancel.log`,
`/private/tmp/i313-session-publication-suite.log`,
`/private/tmp/i313-publication-clippy.log`.
Independent static review approves this code delta against `23e75dd56`, diff SHA256
`baeb3e0979d38e4832cebed89e9cda317db82f47a04915bfce94f333a2eb9188`;
reviewer did not run tests. This delta remains local/uncommitted; full I313
acceptance, platform validation, final preflight and remote closure remain open.

### Task-state retention through model checkpoints A and B

The combined Actor fixture now supplies categorized task state through the actual
recoverable tool result. A reads that live request evidence; B must read installed
checkpoint Context and explicitly cannot fall back to raw tool output. Both actual
post-installation requests assert nonempty completed/in-progress/pending work,
known-failed route, verified fact, unverified assumption and next step in distinct
schema fields. Fresh detail after restart/fork must equal original task state.
This is deterministic retention evidence, not real-model task quality or proof
that a model independently decides to avoid repeated work; calls remain scripted.
The source failure/work labels are fixture data, not actual compiler/research runs.
The locked targeted regression passes (`/private/tmp/i313-task-state-checkpoints.log`)
and records both Applied request token estimates. Agent all-target Clippy passed
before the final assertion/category tightening (`/private/tmp/i313-task-state-clippy.log`);
that small final delta has compiled/passed the targeted test but not fresh Clippy.
Independent static review approves tests.rs diff against `23e75dd56`, SHA256
`318ddd961db9d9fa54c49eca7b64ed2091d8a9520047fe330028ebf83f581883`.
Reviewer did not run tests. No full T5/Issue #390 completion is claimed; these local
changes remain uncommitted and final governance/whole-workspace gates remain open.

Retention/publication local convergence: final Agent+Session all-target Clippy
passes (`/private/tmp/i313-retention-converge-clippy.log`); both governance
validators pass with zero warnings (`/private/tmp/i313-retention-governance.log`,
`/private/tmp/i313-retention-claims.log`). These supersede the earlier final-delta
Clippy gap. Latest Applied estimates are 50,250→21,645 and 50,905→22,306;
these are runtime estimates, not measured provider billing tokens.
Windows inspection reconfirms same_file_version deliberately rejects authorization:
retained READ-only sharing does not yet have native evidence excluding an existing
writable mapping. Do not activate from stamp equality alone. Platform implementation
and native validation remain required, independently of these passing local tests.

### Stable Partial candidate preparation

Current implementation head `0514fec35461e2e87aa6d57e080a8b03a7963fbc`
against base `9ace8968603a67ada8d118459c81367611ad3fd2` has independent static
approval for Partial stage publication to obtain native platform evidence only;
full diff SHA256 `41086d19b690fb64cf8b0d60ef88682c56fe965711456c29a52ed0a701985c72`.
Review inherits continuous incremental reviews plus current critical-boundary
inspection, not a fresh line-by-line review of all changes; reviewer ran no tests
and shares Agent/GitHub identity. It does not approve merge/release/full I313.
Standard preflight is running in `/private/tmp/i313-stable-stage-preflight.log`.
README en/zh now expose stable tool registration, distinct admission/final outcome,
permitted recovery projections and unavailable Windows support as unreleased/partial.
New audit residual: Execution::refresh refuses live compaction discovery above
8192 eligible messages. Verify/fix long-session model compaction separately from
already-paged durable history queries; no whole-session scalability claim.

Stable-stage preflight exited 101: talos-tools had 1056 passes and one Git Push
process-observation EPERM failure under the sandbox. This is not a successful
workspace gate. The authorized standard host rerun is live at
`/private/tmp/i313-stable-stage-preflight-host.log`; no pass is claimed.
Four-file documentation review approved after clarifying that legacy CLI durable
search/detail remains unavailable; no expansion of stage publication authority.

Standard host preflight completed exit 0 including both external Runtime SDK modes
(`/private/tmp/i313-stable-stage-preflight-host.log`), implementation `0514fec35`
and documentation head `446aa98d3`. Owner now enters Review/Partial for a stable
stage draft; all listed acceptance residuals remain open. Independent review
permits draft publication at `446aa98d34891a913ccf6d0b5d17c36899c30eb7`
against `9ace8968603a67ada8d118459c81367611ad3fd2`, diff SHA256
`e948f08b010a9eafba6f24ceb4fd6e75656ed042d36d9abdaad9b9953fc62aed`,
not merge or Windows activation. Reviewer did not run the preflight.

Stable stage published as draft PR #837 at `6153c913d`; full T5 acceptance remains
open. CI is required for its exact published head; no remote pass or merge claim.

### Continuation work bounds and first native CI

Local continuation binding now caps the appended canonical suffix at 8192 entries,
not the total request length including its synthetic prompt gap. The existing
2 MiB hashing budget, deadline, source-start checks and atomic replacement remain.
A regression with an 8300-entry synthetic gap binds one real new entry, refuses
authority for the gap, rejects 8193 new entries and preserves the prior mapping.
All 12 source tests pass (`/private/tmp/i313-source-bound-suite.log`). This does
not fix initial long-history binding or live discovery's total-count limitation.
Independent static review approves source.rs delta against `6153c913d`, SHA256
`da806c6010b065cd41f16e751b0b5dc077fc28d1d8a1296ecce1d58c8bf0b92f`;
reviewer did not run tests.

PR #837 CI run `41387946490` uses published head `6153c913d` (not this local
continuation fix). Issue reconciliation fails because the global matrix omits
independent Issue #832; track as governance residual, not a T5 implementation
failure. Windows mapping probe failed and workspace check was skipped; the job
continues later Git guard tests. Job log download currently returns BlobNotFound,
so the probe failure cause is unconfirmed and Windows authorization remains off.

### Cooperative asynchronous source binding beyond total-history limits

Local bind_content_until/bind_until no longer reject a whole history solely for
exceeding 8192 entries or 2 MiB cumulatively. They yield/check deadlines while
building identities and hashing records, restarting the byte budget after a
bounded failed record attempt; a single oversized record still refuses binding.
Review caught initial all-history ID construction/preallocation before yield;
those now grow incrementally after an initial deadline check. Metadata remains
O(history), allocator growth/drop is not hard-real-time, and this is not a
persistently resumable scan. Synchronous commit and discovery limits remain open.
All 14 source regressions pass (`/private/tmp/i313-source-pages-final.log`), including
8300 entries, cumulative bytes, cross-batch duplicate IDs, single-record refusal,
and expiry after crossing an entry batch. The last test proves deadline expiry,
not every cancellation path. Independent static review approves source.rs diff
SHA256 `f1c2b0c6416559243523f4964f7bc5dd59e24ceb3cda05a5a34253dd05e4d304`;
reviewer did not run tests. Local changes remain uncommitted; no full T5 claim.

### Cooperative live discovery and refresh ordering

Production discovery refresh now yields per entry under a cooperative two-second
deadline rather than rejecting all histories above 8192 messages. Test fixtures
use the same async implementation; query-completion test hooks are async to avoid
keeping a separate synchronous authority path. A 8300-message regression verifies
8290 eligible references with bounded 20-reference discovery pages.
Before yielding, refresh withdraws prior current authority. Independent review
caught overlapping refresh resurrection; a unique refresh epoch now allows only
the newest preparation to publish. Old-future/new-withdrawal and old-future/new-
generation regressions pass. Source cloning is O(history), checked again after
clone, and is not a hard-real-time operation. Dropped preparation returns no new
snapshot. Synchronous commit/recovery snapshot limits remain unresolved.
All 11 tool tests pass (`/private/tmp/i313-discovery-epoch.log`); real Actor
checkpoint A/B/fork/restart passed before the epoch tightening
(`/private/tmp/i313-discovery-actor.log`). Static review approves two-file diff
SHA256 `5fa61e9d1822e254c908b4d9df7518088400544bba6c5e49b895e941272ffaef`;
reviewer did not run tests. Subsequent test-only lexical lock scope fixes a Clippy
await_holding_lock report; final Clippy rerun is pending. Delta remains local and
uncommitted, not full T5 or platform evidence.

Final cooperative-refresh convergence passes all 640 Agent library tests,
including the real Actor checkpoint/fork/restart scenario after epoch correction
(`/private/tmp/i313-discovery-agent-suite.log`). Warnings-denied all-target Clippy
also passes (`/private/tmp/i313-discovery-clippy.log`), superseding pending notes.
These remain local stage checks, not exact published-head CI or whole T5 closure.

### Cooperative long-history recovery and ordinary-turn commit (local, unmerged)

Runtime startup now uses additive async set_recovered_history_sources_until under
an explicit two-second deadline. It clears prior authority, validates the
storage-issued snapshot owner, length and exact message contents, and installs
only a complete binding. The 8300-message Actor regression covers foreign owner,
expiry, changed raw data, dropping the future at its first yield, successful retry,
and a real subsequent ordinary turn preserving old and newly persisted IDs.

Ordinary turn workers bind completed source identity cooperatively under a fresh
two-second budget. They preserve unchanged-prefix identities, apply actual receipt
IDs to new entries, reject duplicates and withdraw inherited IDs after divergence.
Actor installation checks generation/revision and never retries a refused worker
scan synchronously. Compatibility/preview paths retain bounded synchronous binding.
Startup snapshot I/O/materialization and Actor history/source clones remain
O(history), not bounded-memory or hard realtime. Interrupt during completed-turn
binding waits for its cooperative budget rather than immediately aborting the scan.

First full Agent run failed 2/642: scheduler completion timeout and checkpoint/fork
requesting an idle Actor before worker binding finished
(`/private/tmp/i313-long-commit-agent.log`). Completed was emitted too early.
The worker now forwards progress but retains Completed in its record; Actor emits
it after history installation, custody completion and finish_active. Corrected
Agent suite passes 642/642 (`/private/tmp/i313-long-commit-agent-fixed.log`).
Agent/Runtime all-target warnings-denied Clippy passes
(`/private/tmp/i313-long-commit-clippy.log`). Fresh Runtime 8300-message
reopen/child/sibling/independent-process regression passes
(`/private/tmp/i313-long-commit-runtime.log`). Both governance validators pass
with zero warnings (`/private/tmp/i313-long-commit-{governance,claims}.log`).

Independent same-session static review approves seven code/script files against
`e911e4902b4de9f5776828b978b055322df23507`, diff SHA256
`adc556fd75054167b6b6a9a03787fe61c27dc69762726dee744893772e47bc43`.
Subsequent changes only align inserted field indentation inside macro bodies.
Reviewer did not run tests or publish remote review; approval is scoped, not full
I313 acceptance. Standard preflight, current published-head CI, Windows support
and full acceptance remain outstanding.

Windows job 125092886203 failed; native evidence is retained at
`/private/tmp/i313-windows-native-837.log`. The mapping probe failed before
observation because PowerShell converted null map name to an empty string.
Its local NullString.Value correction passes syntax/type checks but lacks native
verification. Session recovery fixture retained a flushed BufWriter while asking
for a restrictive read lease; dropping that writer passes the local targeted test
(`/private/tmp/i313-source-writer-close.log`), not native proof. Both fixes remain
separate local test/probe work. Windows Actor history/checkpoint functionality
remains unavailable and must be implemented, not skipped.

### Actual long-history checkpoint acceptance increment

The actual registered checkpoint A/B/fork/separate-process restart scenario now
contains 8360 filler messages plus its genuinely executed original tool fact.
Its first execution failed because SubmittedCheckpoint::matches_source and
checkpoint_request still refused total message counts over 8192
(`/private/tmp/i313-long-checkpoint-actor.log`). These scans now yield and check
the caller deadline; fixed/eligible/byte/pairing/reference preparation also scans
cooperatively. Generation/ID/digest and complete-tool-batch checks remain intact.
The cumulative 2 MiB projected-text guard remains an explicit residual, and
projection/estimation/clone/collection construction still include synchronous work.

Real scenario passes (`/private/tmp/i313-long-checkpoint-fixed.log`): checkpoint
estimates 81250→9281 and 81905→9942, actual source fact retrieval, fork and
independent-process parent/child reopen. After completing cooperative preparation
loops, full Agent tests pass 642/642 (`/private/tmp/i313-long-checkpoint-suite.log`).
This scripted-model fixture proves execution and state retention, not autonomous
model quality. Independent static review approves four-file diff SHA256
`8163374c7f31a98550ad9c5138da03ff1d95bf1ead7c398b849ba5fba4fb8680`
against `73ac2d398b81146fb56232ed5acb23820c9e4406`; reviewer did not run tests.

Old published-head CI 41387946490 is terminal failure; its host preflight job
passed but Windows and Issue reconciliation failed. Windows Desktop specifically
failed unused import recovery::install_budget; the re-export now uses cfg(unix),
matching all its durable.rs consumers. This does not activate Windows recovery.
Native log: `/private/tmp/i313-windows-desktop-failure.log`. Standard host preflight
`/private/tmp/i313-long-stage-preflight.log` terminated 101: 641/642 Agent tests,
with the expanded long-history scenario exceeding its aggregate 10-second wait.
That scenario now has a 30-second total wait for its multiple bounded operations;
the helper's two-second event-stall check and production deadlines are unchanged.
Fresh standard host preflight passed exit 0, including both external Runtime SDK
modes (`/private/tmp/i313-long-stage-preflight-retry.log`, session30556 terminal).
Validated executable source is committed through `889a86707`; this is local
stage validation, not native Windows or complete T5 acceptance. Both standalone governance validators
passed zero warnings (`/private/tmp/i313-long-checkpoint-{governance,claims}.log`).
Native platform gates remain open. Whole queue stays Active.

Independent static review also approves the isolated Unix re-export condition,
diff SHA256 `ad05bb0aff10778177aee791d7568a5d8b53e127e99750f6656a2b2dace53ff2`;
reviewer confirmed all five external consumers are Unix-only, without compiling.
