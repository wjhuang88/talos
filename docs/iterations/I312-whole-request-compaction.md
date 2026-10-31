# Iteration I312: Whole-Request Compaction And Manual Controls

> Document status: Active / Partial (claim #817 merged; implementation in progress)
> Published plan date: 2026-10-30
> Planned objective: deliver full handoff T4 through MEM-005 policy and MEM-003 convergence, preserving the later T5 Actor recovery deliverable.
> Baseline rule: preserve all parent acceptance; successful trimming alone cannot close this iteration.
> MVP deliverable: a long-running session compacts complete provider requests safely, exposes numeric outcomes and accepts a typed /compact command while raw history remains recoverable.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 handoff closure session |
| Work Slice | Handoff T4: whole-request pressure/admission, shared structured checkpoint generation, raw-history-preserving projection, manual command, observable outcomes and long-session regression |
| Claimed At | 2026-10-30 |
| Source Issue | #390 (T4 subset; T5 remains outstanding) |
| Governance Claim PR | #817 |
| Authorization Mode | Independent review |
| Authorization Evidence | Independent exact-head governance/API/privacy review, CI, both validators and merge CAS required; no implementation before merge |
| Implementation PR | Not started |
| Last Updated | 2026-10-30 |
| Handoff / Release Condition | MODEL-014 acceptance closeout and effective target-branch claim before implementation |

## Published Baseline

Selected owners: MEM-005 (eleven acceptance rows) and MEM-003 (five rows).
MEM-002 is Complete. I298/MEM-005-A remains completed bounded request recovery,
not authority or evidence for this new semantic compaction scope. Issue #390's
T4/T5 allocation is explicit in the proposed design; T4 alone cannot close #390.

### Scope and acceptance

1. One existing request-budget authority covers initial, continuation and summary
   calls, including stable prompt, schemas, images, input margin, output and
   applicable reasoning reservations. Explicit config/catalog/fallback precedence
   remains shared with MODEL-014; unknown usage is not zero usage.
2. Central pressure policy separates advisory from automatic critical and emergency
   handling. Evaluate before provider dispatch; post-turn maintenance marks need
   without mutating a running tool batch. Fixed-overhead overflow refuses safely.
3. One bounded structured checkpoint pipeline replaces old generic-summary
   semantics. Preserve goal, constraints, decisions, progress, facts/assumptions,
   resources, failed attempts, unresolved work and next step with raw provenance.
4. Oversized text and tool history, including restart, uses bounded sequential
   folding under the same admission/schema/privacy rules. Intermediate failure,
   invalid output, timeout or cancellation never commits a partial projection.
5. Keep raw Actor/durable history authoritative. Checkpoints are derived in-memory
   state with generation, source revision/range and lineage. Resume/fork reconstruct
   safely; model changes invalidate stale projections. No second transcript store.
6. Preserve recent complete exchanges, current instructions, stable prefix and
   tool pairing. Exclude private arguments/reasoning and UI/export-only raw output
   from summaries; user-visible diagnostics contain no checkpoint content.
7. Register /compact through CMD-001 BuiltinCommand and typed Session operations.
   Idle executes; active turns defer to a complete-batch boundary. Bound pending
   work, duplicate receipts and expiry. Shutdown/cancel/generation changes yield
   exactly one terminal outcome and cannot apply stale results.
8. Visible transient started/applied/skipped/failed/cancelled status includes
   whole-request before/after estimates and bounded reason codes, never durable
   fake user messages or display-history rewrites.
9. Reject empty/error-only/partial-error/failed-terminal/EOF/oversized/unknown-schema
   or insufficient-reclamation checkpoints atomically. A failed attempt falls back
   only to a safely admitted projection or explicit capacity refusal.
10. Real Actor and manual-command walkthroughs plus a 50-turn heavy-tool and
    text-heavy restart fixture prove bounded requests and retained task semantics.
    Repeated checkpoints retain recoverable original raw history and lineage.
11. Complete pinned locked preflight, independent API/privacy/security review,
    exact-head CI, merge CAS and owner-first implementation evidence.

### Non-goals and ownership

T5 retains stable model-callable tool discovery, voluntary semantic checkpoint
selection and bounded current-branch SessionIndex/history recovery. MEM-007/#143
retain pre-entry shell/tool output filtering. ARCH-006 stable prefix and SESSION-011
truth/migration boundaries remain intact. No paid probes, new database, dependency,
raw transcript migration, release/tag or claim to complete those separate owners.

### Design and compatibility gate

[Proposed design](../proposals/t4-whole-request-compaction-design.md) defines the
central thresholds, schema bounds, folding limits, privacy input and operation
lifecycle. [ADR-096](../decisions/096-whole-request-checkpoint-boundary.md) proposes the
SessionOp enum compatibility/migration and checkpoint-context boundary; it requires
independent review and target-branch activation before implementation. Proposed constants require
long-session evidence; changing measurements must not shrink semantic acceptance.

### Planned validation

Run focused checkpoint schema/renderer/pressure/privacy tests, request-capture
admission tests for every call path, Actor ordering/cancel/dedup tests and real
/compact walkthrough. Include text-heavy over-budget restart, early constraints
and failed attempts, repeated lineage, hidden output and missing usage fixtures.
Run `./scripts/release_preflight.sh` with Rust 1.99.0 and locked workspace commands,
both governance validators, then independent review and exact-head platform CI.
No preceding-head check certifies this implementation.

### Documentation targets

MEM-005/MEM-003 owners, CLI command reference, context/memory behavior reference,
README en/zh where commands are listed, Runtime SDK enum migration, iteration
index, Board and full handoff. Preserve original plans and historical failures.

### Risks and rollback

Generated summaries may omit or misinterpret facts: retain raw evidence, source
ranges, recent verbatim exchanges and bounded recovery ownership. Rollback removes
derived checkpoint application without deleting raw history. Never compensate for
failed compaction by replaying tools or dispatching an over-budget request.

## Preparation inventory — 2026-10-30

Observed against main `0469f8041ed0d24bb2dffb465e9ff7fa3609a034`:
I311 completion is proposed in this governance batch after #816 merged; activation
remains ineffective until the finalized claim merges.
I293/I294 retain separate search scopes (#808/#814); I277 retains deferred human
Review; I302 retains diagnostic Review; I249 stays deferred Planned; I164 remains
Paused with superseded layout in I165. Merged #811 activates independent Gemini I310/ADR-094 at
`80b9393375812475901b1b9565cbff6fca273537`; preserve that distinct scope. Terminal owners remain terminal. Recheck target owners and overlap
immediately before proposing activation; this inventory grants no ownership.

## Execution and completion evidence

None. Planned/unclaimed preparation only. No production implementation authorized
or claimed. Overall handoff remains Active through T4 → T5 → T1 → T6.

## Preparation closure ledger

Requested outcome: effective T4 activation after I311 acceptance closeout.
Artifacts: I312, proposed ADR-096/design, MEM-005/MEM-003 reconciliation, indexes,
Board and handoff. Preserve original owner acceptance and independent T5 scope.
Required evidence: independent design/API/privacy review, both governance
validators, exact-head CI and CAS. Current evidence: preparation only; none of
these drafts authorizes production code. Residuals remain I312; full queue remains
in the handoff owner. Activation/package synchronization is not yet complete.

## Preparation design review — 2026-10-30

Independent Agent static review accepts the design direction, with no remaining
architectural blocker; this is not immutable-head approval or activation. It ran
no tests and shares the external account identity. Implementation must additionally
prove bounded input scanning/temporary allocation (stream large entries), final-
only whole-request fit/reclamation (intermediate folds validate their own request
and schema), exactly-once terminal outcomes under cancellation/application races,
and successful long-session fixtures within the proposed 64-call/120-second
bounds. Safe refusal alone cannot satisfy semantic long-session acceptance.

## Atomic activation proposal — PR #817

The maintainer's accepted autonomous handoff queue selects T4 after merged
MODEL-014 implementation and acceptance follow-up. This PR closes I311 owner-first
and proposes Claimed/Active I312 atomically; both new claim and activation remain
ineffective until this finalized record merges to main. Earlier preparation
paragraphs are historical, not competing current status. MEM-005/MEM-003 delegate
this T4 execution to I312 without creating duplicate claim records.

I293/I294 search and I310 Gemini continue under their independent owners. This
handoff's accepted serial queue and existing independent lanes are preserved;
I312 changes no search endpoint, Gemini wire/auth or permission grant policy.
Shared Agent request-budget/API files require overlap inspection before each
stable implementation candidate; do not overwrite another lane's changes.

## Effective activation and local implementation — 2026-10-30

PR #817 merged as `02fee3cc9537e6bf688d167036389b4af3011aeb`;
its claim and ADR-096 contract are now effective. Earlier preparation/activation
conditions above are historical. Implementation branch: `feat/i312-whole-request-compaction`.
Local stream validation bounds summary output/time and rejects unsuccessful streams;
this is only initial implementation, not the structured checkpoint or whole-request pipeline.
Locked compaction tests passed 40/40, including rollback of all preprocessing on
invalid streams and sanitized provider errors (`/private/tmp/i312-compaction-rollback.log`).
No implementation PR, completion evidence or full T4 validation exists yet.
Remaining acceptance stays in this owner; T5/T1/T6 and the overall goal remain open.

### Local checkpoint schema and pressure increment

The local implementation adds the shared v1 semantic payload, bounded JSON parsing,
runtime-supplied source coverage validation, canonical rendering and final-prefix
reclamation validation. Whole-request pressure classification uses integer math
against total context capacity with 80/95 percent boundaries, without subtracting
output reserves twice. Locked focused compaction suite: 47 passed, zero failed
(`/private/tmp/i312-checkpoint-policy.log`). This is component evidence only:
request admission/folding, explicit privacy projection, raw Actor projection,
manual operations and long-session walkthroughs are still unimplemented. The new
components are not yet wired into the dispatch path. No T4 completion is claimed.

### Local sealed-request and privacy integration

Supersedes the preceding component-only wiring status: initial/continuation sealed
request plans now attempt checkpointing at whole-request Critical/Emergency pressure.
Summary calls use the existing estimator with reserves/margins; final request fit
and reclamation precede the atomic projection replacement. Removed both legacy
Actor history replacement paths; durable archival remains separate. Added explicit
checkpoint privacy projection excluding arguments, reasoning and unverifiable tool
payloads, with tool-owned model projection and declared-token filtering.

Evidence: existing Agent library suite 527/527 passed after initial integration
(`/private/tmp/i312-request-agent-tests.log`); subsequent focused checkpoint suite
9/9 passed, including request capture, original-history/tail preservation, invalid
output atomicity, no provider call on summary over-capacity and privacy/provenance
(`/private/tmp/i312-request-checkpoint-tests.log`). Subsequent scan/allocation bound changes passed the same nine focused tests
(`/private/tmp/i312-request-bounds-tests.log`). All-target warnings-denied Agent
Clippy passed after replacing test-only unwraps with explicit fixture expectations
(`/private/tmp/i312-request-clippy.log`). No broad workspace or platform completion is claimed.
This first single-call implementation rejects oversized sources; bounded sequential
folding, runtime lineage/revision state, explicit outcomes, manual Actor lifecycle,
SDK/CLI wiring and long-session success fixtures remain outstanding. Existing
bounded request recovery remains fallback after checkpoint failure. No T4 PR yet.

### Bounded sequential folding increment

The local sealed-request path now packs ordered source entries into admitted
summary calls, carrying the preceding validated checkpoint forward. A shared
120-second deadline, 64-call limit and 256 KiB aggregate output bound apply; final
reclamation/admission alone commits the projection. Input projection is capped at
8192 messages / 2 MiB eligible content. The 50-turn text-history fixture now succeeds
across multiple admitted calls, preserves the recent tail, and visits source IDs
in order (`/private/tmp/i312-folding-tests.log`, 10 focused tests passed).
This deterministic provider fixture proves request flow, not actual model semantic
quality. Oversized single-entry UTF-8 segmentation, complete-exchange chunk packing,
rebuildable runtime lineage, manual lifecycle and full long-session acceptance are
still outstanding. Later-fold failure preserves the original projection after one successful fold;
11 focused tests passed (`/private/tmp/i312-folding-atomicity.log`). All-target
warnings-denied Clippy passed (`/private/tmp/i312-folding-clippy.log`).

### Exchange grouping and UTF-8 segmentation increment

Summary batches now keep assistant tool calls with their complete parallel results;
missing, duplicate and orphan result identities reject the candidate. Oversized
User/Context text is split by admitted request capacity at UTF-8 boundaries with
raw entry ID and contiguous byte offsets. Ordinary recent exchanges remain intact.
The segmentation path currently requires privacy projection to preserve the text's
byte offsets; changed text and oversized indivisible tool batches still refuse.
This limitation remains implementation work, not a completed acceptance exception.

Focused checkpoint tests passed 13/13 (`/private/tmp/i312-segmentation.log`),
including 140 KiB mixed CJK/emoji input reconstructed exactly from captured source
segments, per-call request admission and preserved recent tail. Deterministic mock
output does not establish real model retention quality. Runtime lineage/manual
controls, cancellation/outcomes and remaining long-session evidence are still open.

### Fixed-overhead refusal and cancellation boundary

Before any summary invocation, the request path now checks immutable system
instructions, tool schemas and reservations against total capacity using the shared
estimator. The new oversized-schema regression proves zero provider calls and an
unchanged projection. An in-flight provider future cancellation regression proves
candidate disposal before the final swap leaves the complete original plan intact.
Focused checkpoint suite: 15/15 passed (`/private/tmp/i312-fixed-cancel.log`). This
is request-future cancellation evidence only, not Actor operation receipts,
exactly-once terminal events or generation-race coverage; those remain unimplemented.
The stream-validation increment was committed locally as `9ee585e20`; it is not
merged completion evidence. Other T4 code remains local work in progress.

### Manual-operation receipt component

Added a bounded generation-qualified operation ledger: one pending identity,
monotonic accepted IDs, 64 terminal receipts, duplicate observation, Busy/Expired/
StaleGeneration rejection and protection against late terminal results. Three
focused lifecycle tests passed (`/private/tmp/i312-operations.log`); all-target
warnings-denied Clippy passed (`/private/tmp/i312-operations-clippy.log`). This is
component preparation, not Actor wiring: SessionOp, transient outcomes, /compact,
actual cancellation/swap races and integration tests remain outstanding. Independent
static review of the current checkpoint request/privacy path has been requested
from the existing reviewer; no approval is implied while that review is pending.

### Independent static review — REQUEST CHANGES

Reviewer identified blocking gaps: current registry names do not authenticate
recovered tool provenance; request-plan indices are not durable raw references;
segmentation excludes assistant/multimodal/filtered long text; repeated failed
revisions lack suppression/circuit breaking; deadline and synchronous work bounds
are incomplete. No merge approval. Review was static, ran no tests, and shares
external Agent identity. Fixed mock output is not semantic-retention evidence.
Immediate partial correction starts deadline at operation entry and checks it in
packing/segmentation and before swap; synchronous projection/serialization and
cancellation still require further bounding. All five findings remain tracked here
until full implementation and independent re-review establish closure.

### Failed checkpoint request suppression

Sealed request dispatch now uses an Agent-owned bounded attempt ledger. A streaming
serialization fingerprint covers request messages, tool definitions and budget
parameters; failed identical candidates are suppressed within a 64-entry window.
Three consecutive failures impose a 60-second cooldown for changed candidates.
Attempt reservation precedes await so cancellation cannot bypass suppression.
Focused checkpoint tests passed 16/16 (`/private/tmp/i312-retry-fixed.log`), including
one provider call for repeated invalid candidates. The first compile attempt found
ToolDefinition lacks Serialize; fingerprinting now uses borrowed constituent fields.
This partially addresses review finding 4: content-free visible failure events and
Actor generation/revision identity remain outstanding, as do independent re-review
and the other four blocking findings. No full T4 delivery is claimed.

### Attempt concurrency and cancellation correction

Replaced the provisional pre-await failure reservation with an exclusive active
slot plus an await-spanning RAII guard. Active attempts cannot be evicted from the
failure window; cancellation/drop records failure and releases the slot. Success
resets the consecutive counter only for its own active attempt. Two new tests cover
exclusion, cancellation suppression and circuit cooldown/recovery; checkpoint suite
18/18 passed (`/private/tmp/i312-attempt-guard.log`). This is Agent-level generation
work exclusion, not the still-unimplemented Actor manual operation lifecycle.

### Execution-bound checkpoint eligibility

Tool-result eligibility now requires an Agent-local execution receipt in addition
to a paired call: a SHA-256 digest binds the tool instance, name, call ID, exact
model-facing result bytes and error bit. Both live observation paths register only
when hook-observed call identity matches the actual call. The bounded 1024-receipt
window contains hashes, not another transcript. Unknown/recovered results without
receipts are excluded, even if a tool with that name currently exists. The privacy
fixture now proves recovered-looking bytes are excluded before receipt registration.
This is a partial fix for review finding 1; restart eligibility reconstruction,
same-name replacement integration tests and independent re-review remain required.
No claim is made that dropping recovered tool semantics fulfills long-session
acceptance. After fixing a missing local instance binding found by the first compile, full
Agent library tests passed 544/544 (`/private/tmp/i312-provenance-agent-fixed.log`);
all-target warnings-denied Clippy passed
(`/private/tmp/i312-provenance-clippy-fixed.log`).

### Live receipt adversarial regression

A new regression obtains eligibility through `observe_tool_result` rather than
manually granting it. Exact observed model output is eligible; altered result
bytes and replacement with a different same-name tool instance are excluded.
The original instance remains alive in this fixture, so it does not prove allocator
address-reuse resistance. Focused test passed (1/1,
`/private/tmp/i312-live-receipt.log`). Stable registry-instance identity and restart
provenance still require design/implementation before review finding 1 can close.

### Stable process-local tool identity

Replaced raw tool addresses in checkpoint receipts with registry-issued Weak
instance handles. Weak allocation identity prevents address-reuse inheritance while
allowing the old tool object/resources to be destroyed. Added a documented additive
ToolRegistry::instance_handle API; it grants neither execution permission nor durable
provenance. Checkpoint suite passed 19/19 (`/private/tmp/i312-weak-identity.log`).
The live replacement fixture additionally checks that the original Weak no longer
upgrades and differs from the replacement identity. Final focused rerun passed 1/1 (`/private/tmp/i312-weak-release.log`), and
all-target warnings-denied Clippy passed (`/private/tmp/i312-weak-clippy.log`). Independent API/privacy re-review remains required; durable
reconstruction and raw-reference mapping remain unimplemented.

### Raw-prefix binding integration

HistorySource now binds original message digests before prompt construction; the
prepared turn carries this mapping through initial and continuation sealing.
Checkpoint eligibility requires every non-system source in the candidate prefix
to match its original position and digest. Changed/reordered/synthetic entries
cannot inherit the old raw identity. Missing bindings fall through to existing
bounded budget recovery/refusal, without a summary call.

This is partial source correctness, not durable lineage: binding currently uses
local history-position IDs and placeholder generation/revision zero. Actor-assigned
generation/revision and durable entry identities must replace those placeholders;
new within-turn records are not yet added to the mapping. Do not close review
finding 2 on this basis. Component tests passed 2/2; integrated checkpoint suite
19/19 and changed-source zero-provider-call regression 1/1 passed
(`/private/tmp/i312-source-map.log`, `/private/tmp/i312-source-integration.log`,
`/private/tmp/i312-source-mutation.log`). All-target Clippy passed
(`/private/tmp/i312-source-clippy.log`). Full Actor/manual integration remains open.

### Actor generation/revision binding

Both Actor prepare paths now pass the actual session generation and append-only
raw-history length as revision into source binding before prompt/hook construction.
Prepared turns retain this snapshot for continuation. The generic non-Actor entry
still uses its local snapshot identity; it is not durable session provenance.
Original Agent library suite passed 548/548 (`/private/tmp/i312-actor-source.log`).
New preparation regression passed 1/1, verifying generation/revision retention,
stale identity rejection and exclusion of current synthetic input
(`/private/tmp/i312-actor-source-identity.log`). Clippy initially found the old
prepare wrapper unused in production; it is now test-only and all-target warnings-denied Clippy passed
(`/private/tmp/i312-actor-source-clippy-fixed.log`).
This supersedes the prior Actor zero-generation placeholder but does not supply
durable entry IDs, cumulative lineage, new in-turn source mapping or manual controls.

### Bound IDs reach summary payloads

Corrected a remaining disconnect: although sealing validated HistorySource, the
summary builder regenerated raw indices. It now consumes the bound IDs for source
items and segmented references, and validates model references against those exact
IDs. Actor IDs include generation plus local entry position; attempt fingerprints
include generation/revision. The request-capture test now asserts the supplied ID
appears in both summary input and final checkpoint. Durable entry IDs/session-wide
lineage remain absent; do not treat generation-local positions as durable identity.
Full Agent tests passed 549/549 (`/private/tmp/i312-bound-reference-agent.log`),
and all-target warnings-denied Clippy passed (`/private/tmp/i312-bound-reference-clippy.log`).

### Long assistant text coverage

Extended bounded UTF-8 segmentation to ordinary Assistant text with no tool calls;
reasoning envelopes remain excluded and tool batches remain indivisible. A focused
150 KiB CJK assistant fixture passed, reconstructing all captured text segments,
excluding private reasoning and preserving the recent raw tail
(`/private/tmp/i312-assistant-segments.log`, 1/1). This addresses one rejected input
class from review finding 3; multimodal text, privacy-length-changing projection,
oversized tool batches and full semantic retention evidence remain outstanding.

### Privacy-filtered long text segments

Length-changing private-token projection no longer forces refusal for supported
text messages. Segments contain only projected text; metadata explicitly separates
projected byte offsets from the whole raw-entry source range. Unchanged text keeps
exact raw byte segments. The focused regression passed (1/1,
`/private/tmp/i312-filtered-segments.log`): all projected text reconstructs in order,
no private token reaches any captured request, and projected offsets are never
labeled raw offsets. This provides coarse raw-entry provenance after redaction,
not a character-level reversible map. Independent review of the representation and
remaining multimodal/tool-batch coverage are still required.

### Multimodal text folding

Oversized multimodal text now uses the same bounded segmentation path, extracting
only text parts joined by newline. Metadata names this projection format and binds
the whole raw multimodal entry; projected offsets never masquerade as raw message
byte offsets. No image payload is loaded by this helper. Multi-part long-text
capture regression passed 1/1 (`/private/tmp/i312-multimodal-fold.log`), reconstructing
all projected text in order. All-target warnings-denied Clippy passed
(`/private/tmp/i312-multimodal-clippy.log`). This fixture contains text parts only;
image exclusion integration and oversized tool-batch handling remain outstanding,
along with lineage, manual controls and independent re-review.

### Mixed text/image exclusion regression

Extended the multimodal folding fixture with an actual Image variant referring to
a nonexistent path. Folding succeeds; captured summary requests contain neither
that path nor MIME/digest metadata, while all text segments reconstruct completely.
Focused regression passed 1/1 (`/private/tmp/i312-image-exclusion.log`). This proves
summary-input exclusion with the capture provider, not provider-adapter filesystem
instrumentation or real model semantic quality. T4 remains partial; the outstanding
Actor/manual, lineage and tool-batch implementation is unchanged.

### Oversized complete tool-batch folding

Oversized validated tool batches now serialize only privacy-eligible projected
messages into ordered UTF-8 fragments. Each fragment identifies the complete raw
batch source IDs and projection format/offsets; no partial tool exchange is placed
in an ordinary request. Final application still waits for all folds and admission.
Successful large verified-result regression passed 1/1
(`/private/tmp/i312-batch-success.log`): captured fragments reconstruct the complete
projected call/result batch, arguments remain excluded, and recent history is intact.
This uses manually registered execution eligibility and deterministic summaries;
real heavy-tool Actor semantics, cancellation, lineage and independent design/privacy
review of batch fragments remain required. No T4 completion or merge is implied.

### Packing work bounds and cooperative cancellation

Greedy prompt packing now caps each batch at 64 source entries while preserving
complete tool groups; packing and segment-size search yield to the executor before
checking their shared deadline. Source message hashing refuses serialized inputs
above 2 MiB instead of scanning arbitrary single-entry content. These are partial
work bounds, not a proof that all projection/serialization costs are bounded.
Checkpoint suite passed 24/24 (`/private/tmp/i312-packing-bounds.log`); a new
cancellation-at-packing regression passed 1/1, observing zero provider calls and
unchanged projection (`/private/tmp/i312-packing-cancel.log`). Actor-level cancellation
receipts, full-operation bounds and independent re-review remain outstanding.

### Real Actor long-history entry path and second static review

Added an AppServerSession execution fixture starting with 50 recovered text turns.
It performs multiple summary calls, dispatches exactly one ordinary continuation,
then shuts down; every original history entry remains byte-for-byte equal and new
turn messages are appended. Focused test passed 1/1
(`/private/tmp/i312-actor-long-history-final.log`). Earlier fixture compile failures
were corrected (Success is a struct variant; Actor Drop prevents moving history).
This uses deterministic summary output, not real-model semantic quality or durable
storage replay. Manual safe-boundary requests and heavy-tool Actor flow remain open.

Second independent static review remains REQUEST CHANGES: tool replacement identity,
segmentation coverage and failure suppression materially improved; cumulative input
work bounds and full source mapping/lineage still block. Review ran no tests.
Its prefix-shift concern must be tested against actual builder order (current builder
appends system/context after history); new continuation entries and durable identities
remain concrete missing implementation regardless of that ordering detail.

### Cumulative identity-scan bounds

HistorySource binding now shares one 2 MiB serialized-byte budget across the entire
raw snapshot; the request fingerprint writer rejects aggregate serialization above
4 MiB. Both stream directly into hashes and stop without allocating a serialized
transcript. Focused aggregate-bound regressions passed 1/1 each
(`/private/tmp/i312-source-total-bound.log`, `/private/tmp/i312-fingerprint-bound.log`).
These cap scanned serialized bytes, not all serializer traversal cost or cumulative
wall-clock latency. Unified deadline coverage and cooperative source binding remain
review residuals; larger-history success may require incremental binding rather than
claiming bounded refusal satisfies the full long-session deliverable.

### Cooperative production source binding

Both production turn-preparation paths now bind source messages incrementally,
yielding before each bounded message hash and checking a supplied deadline. A
cancelled/expired binding never returns partial authority. Focused deadline/drop
regression passed 1/1 (`/private/tmp/i312-cooperative-source.log`), and the real Actor
50-turn history fixture passed again after wiring (`/private/tmp/i312-cooperative-actor.log`).
Binding currently receives its own 120-second deadline; it is not yet the one
shared end-to-end checkpoint deadline. Single-message hashing remains synchronous
within the cumulative byte cap. These limits are explicit residuals, not review
closure. Full manual/lineage implementation remains outstanding.

### System/workspace context Actor verification

Extended the real 50-turn Actor fixture with workspace context. Successful ordinary
dispatch retains a System message and the exact workspace Context; summary requests
exclude that context, and the original raw history remains unchanged. Focused test
passed 1/1 (`/private/tmp/i312-context-prefix-actor.log`). This verifies current
builder ordering (system/context appended after history), rather than assuming
prefix insertion. The specific static prefix-shift concern is not reproduced on
this path. General hook transformations, new continuation-source mapping, stable
durable IDs and repeated lineage still need implementation/verification.

### Continuation source mapping

Before continuation sealing, canonical messages after persist_start are now bound
to generation-qualified append positions. The synthetic prompt gap remains unbound;
unbound Context entries are preserved verbatim and excluded from checkpoint sources.
Failure/cancellation cannot partially replace the mapping. A new regression verifies
new-input/reply references, prompt-gap exclusion and preservation after invalid
rebinding (`/private/tmp/i312-continuation-map.log`). Full Agent library suite passed
559/559 (`/private/tmp/i312-continuation-agent.log`). This extends local source mapping,
not durable entry identity or checkpoint parent lineage; those and Actor manual
operations remain outstanding. End-to-end continuation-triggered checkpoint fixture
and independent re-review are still required.

### Continuation sealing integration evidence

Added a sealed-request integration fixture whose eligible range crosses an unbound
workspace Context and includes newly bound continuation messages. Checkpointing
succeeds, captures contain the new generation-qualified raw IDs but no synthetic
Context, and ordinary output retains that Context verbatim. Original input remains
unchanged and final admission fits. Focused test passed 1/1
(`/private/tmp/i312-continuation-sealed.log`). This covers real sealing/admission,
not a live tool-driven multi-call Actor turn; that broader scenario remains pending.

### Shared attempt deadline includes fingerprinting

The 120-second checkpoint-attempt deadline is now created before request identity
calculation and passed unchanged into projection/packing/provider folding. The
fingerprint writer checks that deadline on each write, alongside its aggregate byte
cap. It can no longer consume a separate pre-folding time budget. Focused checkpoint
suite passed 28/28 (`/private/tmp/i312-shared-deadline.log`), including expired
fingerprint refusal. Earlier history binding remains a separate preparation step
with its own bound; this does not establish one deadline for the entire Actor
submission or complete the manual cancellation/lifecycle contract.

### Local stage validation — 2026-10-31

The current Agent library suite passed 561/561 with `cargo test --locked -p
talos-agent --lib` (`/private/tmp/i312-local-stage-agent.log`). This supersedes the
earlier suite counts for the current local stage only; it is not workspace
preflight, independent approval, exact-head CI or merge evidence. I312 remains
Active / Partial with the implementation and validation residuals above.

### Source binding and folding share the operation deadline — 2026-10-31

Production initial-history bindings now retain the caller's deadline; checkpoint
fingerprinting, projection and provider folding reuse it rather than starting
another 120-second window. Each continuation starts its next request's budget
before binding and atomically installs that same deadline with its source mapping.
Expired or cancelled rebinding preserves the prior mapping. This supersedes the
separate-binding-budget limitation recorded above, but does not implement the
manual Actor operation lifecycle or its terminal cancellation events.

`cargo test --locked -p talos-agent --lib deadline` passed 8/8
(`/private/tmp/i312-source-deadline-tests.log`), including virtual-clock regressions
for exhausted initial preparation (zero summary calls, no projection mutation)
and continuation expiry without partial rebinding. Full-stage preflight and
independent review remain required.

### Workspace gate correction — 2026-10-31

The standard preflight passed workspace check, warnings-denied Clippy and 563
Agent library tests, then failed `i169_request_plan_source`: its source contract
recognized only the old `prepare_session_turn` spelling. Updated the fixture to
accept the generation/revision-aware preparation entrypoint while retaining the
sealed-plan dispatch and forwarding checks; focused integration tests passed 2/2.
The original failed run is `/private/tmp/i312-local-stage-preflight.log`; repaired
standard preflight is `/private/tmp/i312-local-stage-preflight-repaired.log` and
is not yet certified successful here. Independent reviewer retry failed with
HTTP 429 before returning a review; no new approval exists.

### Projection allocation boundary and remaining gate failures — 2026-10-31

Checkpoint projection now rejects cumulative tool-owned model text/private-token
output above 2 MiB before inserting tokens into the trie or retaining model output
for later copies. This bounds downstream allocation, not allocations or execution
inside synchronous tool callbacks. Expansion regression covers two individually
fitting outputs whose aggregate exceeds the bound, and an oversized private token.
The checkpoint-focused suite passed 30/30 (`/private/tmp/i312-projection-bounds.log`).
An initial fixture compile failure from an unavailable provider-crate reference
was fixed with a local no-dispatch model fixture, without adding a dependency.

The repaired sandbox preflight failed on native Git test process observation
(`ps`, EPERM). Host preflight (`/private/tmp/i312-local-stage-preflight-host.log`)
passed that Git test but failed the search refused-connection budget test: it
reported Request on Lite instead of RequestLimit on InstantAnswer. Reusing that
run's exact `talos_tools-9655835ff4de3fa3` binary in isolation passed the actual
test 1/1; cause under suite concurrency is unresolved. An earlier package-only
reproduction ran zero tests because network was not enabled and is not evidence.
No complete preflight success is claimed.

Independent static review found no new blocker in shared deadline handling, but
did not approve full T4 or run tests. Durable source/branch identity, persistent
in-memory Actor projection and lineage, manual lifecycle and visible outcomes
remain implementation gaps. The latest projection-output bound still needs
review and current workspace validation.

### Validation follow-up and rejected fixture change — 2026-10-31

Current Agent all-target Clippy passed with warnings denied
(`/private/tmp/i312-projection-clippy.log`). Independent static review accepted
the local projection-output bound, retaining the callback/allocation limitations
above; this is not full T4 approval.

A trial to retain a bound, non-listening socket for refused-connection fixtures
failed both actual socket tests with Deadline instead of ConnectionRefused
(`/private/tmp/i312-refusal-reservation.log`). The trial was reverted completely;
production search and the original fixture assertions remain unchanged. Static
review of the trial did not establish runtime behavior. The prior host-suite
search failure remains unresolved; no full-preflight success or root-cause claim.

### Rebuildable request projection reuse — 2026-10-31

Sealed requests now retain one bounded in-memory derived checkpoint projection
with its generation, source revision, covered raw positions, context limit and
private-token fingerprint. Later requests reuse it only when every covered source
ID/digest, generation, capacity and privacy token set still match; changed raw
bytes, generation, capacity or filtering inputs invalidate the cache atomically.
Raw Actor history remains untouched and new tail messages are appended to the
reused request. Regression passed in the full Agent library suite: 565/565
(`/private/tmp/i312-reuse-agent.log`), including zero additional summary calls
and invalidation cases. This is request-level projection reuse only; durable
entry IDs, repeated parent lineage, manual lifecycle and visible terminal
outcomes remain outstanding.

### Generation-qualified content references — 2026-10-31

Production HistorySource bindings now derive each entry identifier from the
source generation, ordinal and full SHA-256 digest of the exact raw message.
The ordinal disambiguates identical adjacent messages; the full digest prevents
same-position replacement from aliasing a prior entry. Continuation entries use
the same identity format. These references are reproducible only for identical generation, ordinal and
message bytes. They are not durable TLOG entry or branch identities, and do not
establish restart/fork recovery. Generation changes invalidate the projection. Source and Agent coverage passed 566/566
(`/private/tmp/i312-stable-agent.log`), and warnings-denied Agent Clippy passed
(`/private/tmp/i312-stable-clippy.log`). This does not yet provide persisted
branch records or Actor restart/fork recovery evidence.

### Production source binding correction — 2026-10-31

Removed the synchronous pre-binding content-ID scan introduced above. Production
preparation now derives IDs from the already bounded cooperative digest scan,
under the original deadline and aggregate 2 MiB/8192-message limits. The manual
protocol/registry draft was withdrawn before publication because it had no
complete execution/terminal path. `/compact` remains unimplemented and hidden.
The earlier claim of durable source identity was too strong: current references
remain generation-qualified in-memory identities, not durable entry/branch IDs.

Current correction validation: source tests 8/8 passed
(`/private/tmp/i312-content-binding.log`); Agent all-target warnings-denied Clippy
passed (`/private/tmp/i312-content-binding-clippy.log`). These checks do not
certify the whole workspace or full T4 acceptance.

### Projection reuse policy and cooperative cancellation — 2026-10-31

Configuration changes to model identity, request-budget policy, tool presentation
and protocol invalidate the derived projection. Independent review identified
that fitting cached requests at Critical pressure skipped automatic checkpointing.
Reuse now accepts only Normal/Advisory requests; Critical/Emergency candidates
leave the canonical plan intact for fresh source folding. The reuse scan releases
the cache lock before work, yields between covered messages/copies, and shares
the source deadline through final estimate/swap. Cancellation before swap keeps
the plan and cached checkpoint unchanged. The combined regression passed 1/1
(`/private/tmp/i312-critical-reuse.log`), including growing the reused request to
96% and observing additional summary dispatch from raw sources. Independent
static review accepted these two corrections but ran no tests and did not approve
full T4. Concurrent invalidation may discard another request's newer cache; this
is a reuse-efficiency residual, not permission or raw-history authority.

Standard preflight `/private/tmp/i312-stage-convergence.log` started before these
review corrections; its result cannot certify the latest local candidate.

### Stage convergence validation — 2026-10-31

Standard host preflight completed successfully (exit 0), including both external
Runtime SDK modes: `/private/tmp/i312-stage-convergence.log`. It began before
the final reuse-policy correction, so this is baseline workspace evidence, not
an exact-candidate certificate. The previously failing search test did not fail
in this run; its historical cause is still unproven. Latest Agent library tests
passed 567/567 (`/private/tmp/i312-final-reuse-agent.log`); latest all-target Agent
Clippy passed (`/private/tmp/i312-critical-reuse-clippy.log`). Both governance
validators passed with zero warnings. Full staged work remains a Partial local
implementation, without implementation PR, exact-head CI or merge evidence.

Current unchanged candidate subsequently passed standard host preflight (exit 0),
including both external Runtime SDK modes:
`/private/tmp/i312-stable-candidate-preflight.log` (session 7757). Independent
read-only staged review found no blocker to a local partial checkpoint commit;
it did not run tests and retains REQUEST CHANGES for full T4 acceptance.
Manual execution, visible outcomes, durable branch identity and repeated
checkpoint lineage remain outstanding; this evidence does not authorize merge.

### Runtime source-lineage increment — 2026-10-31

The local projection now records a monotonic Agent-owned sequence and an optional
parent sequence only when all predecessor covered entries remain covered with
matching generation, IDs and digests and nondecreasing source revision. Cache
invalidation does not recycle sequences; sequence exhaustion preserves both the
previous projection and caller plan. The focused regression passed, followed by
warnings-denied Agent all-target Clippy (`/private/tmp/i312-lineage-clippy.log`).
Independent static review accepted this bounded increment without running tests.
This parent link describes source coverage, not semantic inheritance; only the
current projection and parent number are retained. Durable branch mapping,
ancestry recovery, Actor-owned lifecycle and manual execution remain unimplemented.
The full Agent library suite passed 568/568 (exit 0), recorded in
`/private/tmp/i312-lineage-agent.log`. This increment is uncommitted
and is not certified by the preceding checkpoint's workspace preflight.

### Manual Actor/command increment — 2026-10-31 (uncommitted)

Typed Compact/CancelCompact operations, an Actor-owned single pending slot and
transient numeric outcomes are now wired to TUI/inline `/compact` and the Runtime
facade. Manual preparation shares ordinary prompt/tool-definition/hook estimation,
returns a candidate without installing it, and leaves the Actor to apply under a
commit gate serialized with runtime shutdown/interrupt. Duplicate receipts do not
rerun summaries; source generation/revision are checked before application. Full-width
runtime generation IDs now fit the checkpoint schema's 128-byte reference bound.

Agent 573/573 and Conversation 180/180 passed before the latest targeted-cancel
wiring. The first bridge route/status regression passed; cancellation extension,
current full preflight and both external SDK modes still require verification.
Independent review initially rejected the worker-side cancellation race; the
Actor commit-gate correction and targeted cancellation received independent static
follow-up approval for this partial stage (no test certification or full-T4 approval).
Current execution waits for the entire active turn, so intra-turn safe-boundary
compaction remains missing. Durable branch identity/recovery, complete lineage,
semantic-retention fixtures and final acceptance/review/CI/merge remain open.
No completion or release is claimed by the new command entry.

The optional workspace all-target Clippy pass also found two pre-existing
`unwrap_err` test warnings in `talos-tools/src/search_backend/health.rs` (present
on origin/main). Our new test constructor/declaration-order warnings were corrected;
this is not a claim that workspace all-target Clippy is clean. Standard preflight
and touched-crate all-target checks remain the candidate validation gates.

Current manual-stage candidate passed standard host preflight (exit 0), including
both external SDK dependency modes: `/private/tmp/i312-manual-stage-preflight.log`.
Touched Agent/CLI/Conversation/Runtime/TUI all-target warnings-denied Clippy passed
(`/private/tmp/i312-manual-touched-clippy.log`). Independent read-only follow-up
accepted the targeted cancellation/receipt and TUI activity corrections, without
running tests or approving complete T4. The command now has real Actor execution
and current-candidate regression evidence; the outstanding intra-turn boundary,
durable identity/recovery and semantic acceptance items above remain open.

### Intra-turn boundary implementation — local uncommitted increment

Manual requests now start after a complete tool batch through the existing
Actor/Agent boundary handshake. The Actor retains continuation permission while
Agent prepares a candidate from the current messages, tools, source mapping and
private-token set. A child cancellation token does not cancel the parent turn;
application retains the Actor/runtime commit gate and does not release the active
turn's lifecycle slot. Raw history remains unchanged by projection application.

A real tool fixture proves Applied precedes ordinary continuation, the next
provider request contains the checkpoint, and original raw history is retained.
The observation is reused for the unchanged request, avoiding duplicate hooks;
new steering creates a different request. Observation denial/incompletion aborts
that continuation rather than retrying a potentially denying/effectful hook.
The outer 120-second deadline covers binding, observation and folding. Focused
paused-time timeout and targeted-cancellation tests passed; complete latest-suite
and independent follow-up results must be checked before stage certification.
These fixtures do not prove semantic summary accuracy, durable branch identity,
restart/fork recovery, ephemeral-image parity or all steering-custody races.
Full T4 remains Active/Partial and the complete downstream queue is unchanged.

Boundary-stage validation: Agent library 579/579 passed, warnings-denied Agent
all-target Clippy passed, both governance validators passed with zero warnings,
and standard host preflight exited 0 including both external SDK modes
(`/private/tmp/i312-boundary-stage-preflight.log`, session 29277). Independent
static review accepted the observation-reuse/failure/deadline corrections without
running tests or approving full T4. Latest fetched main is `3c748a11576af6ca16301edecb35d8a234eef923`,
adding only the independent I293 migration proposal; this run precedes base update.
