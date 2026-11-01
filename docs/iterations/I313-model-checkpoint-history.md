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
