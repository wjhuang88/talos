# T4 Whole-Request Compaction Implementation Design (Proposed)

Scope: full handoff T4 / MEM-005 whole-request trigger, observable results and manual control.
T5 model-callable compaction/history recovery remains a separate dependent delivery.
Do not implement from this draft until the new target-branch claim is effective.

## Current evidence

- Session::start_submission has two should_compact(history) paths (legacy and runtime-managed).
  Both replace self.history with compactor output before request preparation.
- ProviderRequestPlan owns messages/tool_definitions and full estimated_tokens. recover_request_budget
  only removes complete old exchanges from a request projection and preserves latest exchange.
- SessionOp has no compact operation. Compactor::manual_compact exists but checks history-only
  threshold and is not proof of user command wiring.
- Existing I298 completion is bounded request recovery, not authority for T4.

## Proposed implementation boundary

1. Keep raw actor/durable history authoritative. Introduce explicit request-context projection state
   with a source revision/cursor; summaries never replace raw records. Invalidate stale projections
   on reset/fork/recovery using the rebuildable-state lifecycle below.
2. Use the same estimator as the sealed outbound request, including stable prompt, tool definitions,
   output reserve, applicable reasoning reserve and image budget. Do not invoke hooks twice to
   estimate and then dispatch a different request. Evaluate both initial and continuation paths.
3. Compaction decision takes pressure from the full plan and targets available history budget after
   fixed request overhead. If fixed overhead alone cannot fit, return an explicit budget refusal.
   Preserve complete tool exchanges, reasoning replay integrity and latest live tool batch.
4. Automatic compaction triggers before provider dispatch; post-turn only marks future need.
   A continuation is a safe boundary only after the entire executed tool batch has paired results.
   Summarizer failure uses bounded deterministic projection or refuses safely if still too large.
5. Add typed actor compact command with request/generation identity. Idle runs immediately; active
   turn gets explicit queued/deferred acknowledgement and runs at the safe boundary, never mutates
   in-flight execution state. Bound queue and make cancellation/duplicate handling explicit.
6. Wire /compact through the existing BuiltinCommand registration and typed session adapter, not a
   UI-local history rewrite. Emit started/applied/skipped/failed/cancelled metadata with before/after
   whole-request estimates; never include summary or hidden tool text in display events.
7. Model-controlled compaction T5 must later reuse this actor boundary; no tool executes a direct
   history rewrite. Bounded authoritative history recovery requires its own privacy/access design.

## Required regression matrix

- Small history + large tool schemas triggers; fixed overhead impossible => clear refusal.
- Initial and continuation requests use identical admission math; output/reasoning reservations.
- Below threshold/manual no-op, deterministic success, summary failure, timeout and cancellation.
- Complete tool pairing and hidden output stay intact; raw history unchanged and recoverable.
- Active-turn queue order, duplicate compact requests, shutdown and generation changes.
- Real /compact invocation shows transient status without persisting summary into scrollback.
- Restart/fork and durable raw-history recovery; request projection revision mismatch.

## Governance before activation

Read MEM-002/MEM-003/MEM-007 and ADR-016 plus request-budget ADRs; inventory every live iteration;
select a new ID and record public enum migration for SessionOp changes; keep parent's published
acceptance unchanged. Establish claim + activation on main with independent API/privacy review.
This draft is preparation only, not implementation or acceptance evidence.

## Semantic checkpoint contract shared with T5

Issue #390 refines MEM-005. Automatic and manual T4 summarization must produce
the same bounded structured checkpoint later accepted by T5: schema version,
summary, current goal, user constraints, completed/in-progress/pending work,
decisions, verified facts, assumptions, resources, unresolved failures/open loops
and next step. Runtime computes covered range and validates source references;
the model cannot choose arbitrary deletion boundaries. Canonical rendering is
derived context, never an original user transcript entry. Preserve lineage across
repeated checkpoints so original raw records remain recoverable.

Reject empty, malformed, oversized, partial-error, failed-terminal and insufficient
reclamation outputs without committing projection changes. Cancellation must win
before atomic application and produce exactly one terminal operation outcome.
Bound provider time, buffered output and attempts; reuse circuit-breaker handling.
Never expose reasoning, private arguments or UI-only raw output to the summarizer.

T4 delivers independently runnable automatic/manual compaction, shared checkpoint
validation/rendering, and raw-history-preserving projection. T5 subsequently adds
stable tool discovery, model-directed safe-boundary requests and bounded current-
branch recovery using existing SessionIndex with authoritative source reads.
T4 does not close #390 or claim T5 recovery coverage. Both stages retain the full
Issue acceptance matrix; no second transcript/index store or compression pipeline.

## Existing-owner reconciliation

MEM-002 is Complete, with higher-layer residuals assigned to MEM-003. MEM-003's
published claim that the actor lacks provider access is historical: current
Session::start_submission calls Compactor::compact in both paths. Preserve its
five acceptance rows and add the missing 50-turn heavy-tool integration evidence
while converging to the shared checkpoint path, rather than wiring a second LLM.
ADR-016 keeps raw TLOG/legacy JSONL authoritative. MEM-007/#143 pre-entry output
filtering, ARCH-006 stable prefix, and SESSION-011 representation work retain their
own boundaries. New SessionOp variants require an explicit public API migration
decision before implementation.

## Delivery gates

The next iteration must inventory current owners and remote overlaps, reconcile
MEM-005/MEM-003, publish the complete acceptance/validation and documentation
targets, and establish an effective claim on main. MODEL-014 must receive its
acceptance closeout; this proposal grants no production implementation authority.
Focused real Actor/CLI tests must prove safe manual behavior and a 50-turn heavy-
tool fixture, including repeated checkpoint provenance, failure/cancel and resume.
Run pinned locked workspace preflight, independent API/privacy/security review,
both governance validators, exact-head CI and merge CAS. Record only existing
merged implementation SHAs as completion evidence.

## Proposed acceptance-to-test map

| Requirement | Required executable evidence |
|---|---|
| Whole request pressure | Tiny history plus large tool schema/system prefix; both initial and continuation plans; compare dispatch capture with admission estimate |
| Reservations | Explicit output and protocol reasoning budget reduce capacity; avoid double counting reasoning already inside output cap; unknown usage uses conservative estimate |
| Semantic retention | Structured checkpoint retains goal, constraints, decisions, progress, failures and next step across 50 heavy-tool turns; original covered facts remain in raw history |
| Atomic replacement | Intermediate folding failure/cancel, empty/error-only/partial-error/EOF-without-terminal/oversized/invalid-schema/insufficient-reclaim summaries leave projection unchanged |
| Safe ordering | Manual idle operation applies once; active-turn request defers to completed batch; cancellation/shutdown/generation change cannot apply stale results |
| Observability | Started/applied/skipped/failed/cancelled outcomes include only content-free estimates and reason; command returns a visible transient result |
| Manual reachability | Registered BuiltinCommand /compact reaches typed Session operation through supported CLI/TUI adapters; no display-history rewrite |
| Persistence | Repeated checkpoints preserve raw range lineage; export/replay/fork/resume read original history; absent/corrupt derived state rebuilds safely |
| Privacy/pairing/cache | UI-only raw output/private reasoning excluded; latest complete exchanges paired; stable prefix/tool definitions byte-identical |
| Irreducible request | Fixed prefix/schema/reserves alone overflow: bounded refusal and zero ordinary provider requests; no unbounded retry |

This map does not replace MEM-005's eleven published acceptance rows or #390's
thirty-test matrix. T5 adds model-callable discovery, stable tool composition,
current-branch bounded recovery and recovery privacy/concurrency tests. Parent
#390 remains open until all selected T4/T5 requirements are proven.

## Proposed policy and lifecycle

These are proposed initial bounds; independent review and effective activation
are required. Measure them against long-session fixtures before acceptance.

- Assess pressure before admission refusal using the existing full request
  estimate, including input margin and output/reasoning reservations, divided by
  the resolved total context limit (never the limit with reserves subtracted
  again). Below 80%
  is Normal; 80% to below 95% is Advisory; 95% through capacity is Critical;
  above capacity is Emergency. Advisory permits ordinary fitting continuation.
  Critical triggers automatic checkpointing at a complete-exchange boundary before
  dispatch. Emergency requires a fitting checkpoint or bounded refusal. Manual
  below Advisory returns NotEligible without a provider call. Post-turn only marks
  next-boundary pressure. T5 adds voluntary model discretion within this policy.
- Normal, continuation and summarizer calls all pass the existing request admission
  authority. Bounded sequential checkpoint folding, not recursive compaction.
  Admit exact rendered input/output reservations before invoking it; admit the
  resulting ordinary request again before dispatch. Hooks are not run twice to
  construct different requests. Model/config/history revisions invalidate stale
  candidates. If fixed instructions/schema/reserves cannot fit, refuse safely.
- Oversized eligible history, including text-heavy restart, uses sequential
  checkpoint folding. Partition privacy-filtered raw history in original order
  at complete-exchange boundaries, packing each request with a provisional
  checkpoint and a bounded next raw chunk under the same admission authority.
  Use the same structured schema and renderer for every intermediate checkpoint.
  No normal turn or transcript mutation occurs between chunks. Carry cumulative
  raw-range lineage and source offsets; commit only the final validated result.
  Tool-result deterministic layers may reduce eligible tool payloads, but cannot
  substitute arbitrary text deletion for semantic retention. Each chunk prompt
  preserves prior goal/constraints/decisions/failures/pending work unless later
  source evidence explicitly supersedes them; source references retain provenance.
- Folding has at most 64 provider calls, no retries or recursive compaction, a
  total 120-second deadline and aggregate generated output limit of 256 KiB.
  Each request and provisional checkpoint also obeys per-request/schema bounds.
  Stop on cancellation, timeout, budget exhaustion or any invalid intermediate
  output and preserve the prior active projection. A single oversized text entry
  is split at UTF-8 boundaries into ordered segments with the same raw entry ID
  and byte offsets. Segmentation is summarizer-only: recent verbatim tail and
  tool-call/result pairing in ordinary requests never become partial exchanges.
  If mandatory prompt/checkpoint/source segment cannot fit, return a typed
  irreducible-content refusal; do not silently drop semantic fields. Prove a
  text-heavy restart exceeding one request with early constraints, failed routes
  and pending work surviving the final checkpoint. Provider summaries remain
  fallible derived interpretations, not authoritative truth.
- Derived checkpoint state is initially in-memory and rebuildable, with schema and
  renderer versions, sequence/parent sequence, generation/source revision and
  cumulative covered raw range. No new durable store. Resume/fork reconstruct from
  raw history under current limits; model replacement invalidates derived state.
  Reconstruction need not produce byte-identical summaries, but must preserve
  raw recoverability, paired recent history and hard admission. T5 uses raw refs.
- At most one pending compact operation. Key: (generation, monotonic operation ID).
  A pending duplicate observes the same operation; another ID receives Busy.
  State: Queued → Running → Applied/Skipped/Failed/Cancelled, exactly one terminal.
  Active-turn requests defer until the entire tool batch is paired. Shutdown or
  generation replacement cancels pending work. Atomic projection swap is the
  linearization point: cancellation observed first prevents application; after
  Applied it cannot undo a checkpoint. Retain 64 terminal receipts per generation;
  retained duplicates return the receipt, older sequence IDs return Expired
  without re-execution. Generation replacement invalidates receipts and requests.
- Checkpoint v1 requires summary, goal, next step and all named collections (empty
  collections allowed). Items contain text and source references. Each text field
  is at most 4096 UTF-8 bytes, each collection 32 items, each item 8 refs, total
  refs 256, each ref 128 bytes; payload at most 32 KiB. Rendered token bound is
  min(4096, 10% of context). Reject unknown versions, invalid/uncovered refs and
  empty mandatory text. Fixed field order and preserved list order give canonical
  rendering. Require at least 10% eligible-prefix token reclamation and a fitting
  final request; otherwise return InsufficientReclaim without mutation. Contexts
  too small for mandatory fields return InsufficientCapacity.

## Explicit checkpoint privacy projection

`Agent::persistence_projection_with_tokens` in `talos-agent/src/lib.rs` retains
reasoning envelopes and uses raw-string fallback for unknown tool results. It is
an observer/persistence sanitizer, not a complete summarizer eligibility check.
`PrivateTokens` only filters declared values; it is not generic secret detection.

Add a checkpoint projection at the existing Agent boundary. Consume canonical
model-facing messages; reuse tool-owned projections and declared-token filtering
where applicable; remove all reasoning envelopes and tool argument bodies from
summarizer text (retain call names/IDs). Never load `raw_content` or UI/export
projections. Unknown or unverifiable tool provenance on recovery excludes its
payload rather than falling through to raw text. UI-hidden but model-visible
results may be summarized; UI/export-only raw output may not. Summary text never
enters scrollback, only content-free operation outcomes do. Ordinary user-authored
text remains user content. Test live and resumed projection paths separately.
References prove source provenance, not truth of model-generated interpretations.
T5 must reuse this eligibility boundary for authoritative history recovery.

## Acceptance allocation

MEM-005's eleven published rows all belong to T4 (limits, reservations, precedence,
pre/post-turn policy, manual command, visible outcomes, raw history, hidden output,
failure fallback and tests). Post-turn maintenance marks future pressure only.
MEM-003's five rows belong to T4 convergence; its old wiring description is retained
as historical evidence, not a request for a second provider integration.

Issue #390 mandatory tests 1–3 and 21 are T4 pressure/cache foundations with T5
stable-tool/model-discretion integration; 4–6 are T5 voluntary safe-boundary tests
plus T4 manual ordering; 7–13, 18–20, 22–28 and 30 are T4 schema/lifecycle/emergency
and long-session tests, repeated under T5's model-callable path. Tests 14–17 and 29
are T5 authoritative recovery/isolation/index-rebuild tests. Existing MEM-007/#143
filter behavior is preserved and integration-tested (22–23), not claimed complete.
Both stages must reconcile their shared rows before #390 can close.
