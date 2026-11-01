# Iteration I313: Actor Model Checkpoints And Bounded History Recovery

> Document status: Active (proposed in #819; ineffective until target-branch merge)
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
| Implementation PR | Not started |
| Last Updated | 2026-11-01 |
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
