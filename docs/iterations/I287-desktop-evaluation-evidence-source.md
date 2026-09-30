# I287 — Desktop Evaluation Evidence Source

> Document status: Active / Claimed
> Planned objective: Provide a production-authoritative, session-bound Evaluation evidence source so Desktop distinguishes missing, stale and current evidence and gates Delivery without fabricated PASS results.

## Scope

- Define the authoritative producer, subject identity, workspace revision and provenance for Evaluation evidence.
- Connect the existing Desktop projection to current/stale/missing evidence without a Desktop-owned business store.
- Preserve fail-closed `EvaluationUnavailable` behavior until authoritative evidence exists.
- Add deterministic revision/staleness/Delivery tests and a native H6 acceptance recipe.

## Out Of Scope

- No new durable Mission schema or migration without a separate accepted contract.
- Fixtures, harnesses, or manually entered results cannot be production evidence.
- No new evaluator authority, global event bus, scheduling system, or unrelated Desktop redesign.

## Readiness And Acceptance

- Requires an accepted production Evaluation-source contract and owner before implementation.
- Evidence is bound to a session/task subject and workspace revision with provenance.
- Desktop distinguishes missing, stale and current evidence; Delivery requires current authoritative evidence.
- Restart/resume and changed-workspace cases cannot replay or upgrade stale evidence.
- Locked tests cover source lifecycle, revision mismatch, unavailable evidence and Delivery gating.
- The maintainer can reproduce and record native H6 evidence in #29 using the exact candidate build.

## Governance

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | `codex / single maintainer` |
| Work Slice | Runtime-owned session-bound Evaluation evidence producer, bounded workspace revision, Desktop projection wiring and H6 validation; no durable Mission/Evaluation schema |
| Claimed At | 2026-09-30 |
| Source Issue | #29 |
| Governance Claim PR | Direct commit 7d405136 |
| Authorization Mode | Direct commit |
| Authorization Evidence | Maintainer accepted ADR-084 contract on 2026-09-30 |
| Implementation PR | Not started |
| Last Updated | 2026-09-30 |
| Handoff / Release Condition | Implementation merge plus independent security/API review and native H6 evidence |

ADR-084 is the accepted production contract for this slice. Implementation starts locally from the
current `main` and must remain within the Work Slice above.

Activation provenance correction (2026-09-30): `5bb36663` is the starting baseline, not an
existing claim activation commit. The maintainer accepted ADR-084 and repeatedly authorized
local I287 commits. Local commit `7d405136` recorded both governance and partial implementation;
this did not follow the SOP's separate claim-before-implementation sequence and is not evidence
of remote activation or merge approval. Earlier uncommitted implementation must not be described
as having started from an already-published claim. Reconcile this local checkpoint before any
remote candidate; do not treat local commit authorization as a remote-gate waiver.

## Local Review Checkpoint — 2026-09-30

The initial local implementation passed 50 Runtime tests, 109 Desktop tests, and both governance
validators with zero warnings. Independent Agent-role security/API review returned REQUEST CHANGES;
these passing tests are not candidate approval or H6 acceptance.

Required local corrections before a stable candidate:

- Hash-only capture must not become criterion-level PASS; connect a real semantic validation
  producer before claiming production Evaluation is available.
- Derive revision and evidence from the same bounded capture and recheck the current subject after
  model evaluation, before presentation and Delivery.
- Bound file reads, directory traversal and depth, preserve cancellation responsiveness, and use
  a confined-open boundary that rejects symlink replacement races.
- Prevent external construction of authoritative snapshots; include complete Mission/Goal revisions
  in their integrity identity.
- Define subsequent goal/claim lifecycle and add production-path coverage for changed goals,
  workspace changes during evaluation, unavailable evidence and restart.

Partial corrections now keep hash-only evidence Unavailable and reject it before model assessment,
make snapshot fields private, include Mission/Goal revisions in the digest, and limit per-file reads
before allocation. These do not yet resolve the complete review. I287 remains Active / Claimed;
H6 and the stable candidate commit remain pending. The maintainer has authorized the local candidate
commit after convergence and requested native acceptance only after local tests/docs are complete.

Snapshot follow-up: revision and record identity now derive from the same collected content digest,
with a subsequent stability check; path lengths are framed in that digest and native workspace path
bytes avoid lossy identity collisions. Added assertions cover Mission/Goal revision changes and
rejection of the previous subject after file modification. This does not prove an atomic filesystem
snapshot or resolve confined-open races, semantic evidence, or post-evaluation revalidation.

Traversal follow-up: snapshots now stop at 4,096 inspected directory entries or depth 32, including
empty-directory trees that do not consume the regular-file limit. A regression exercises both
guards. These limits complement the 256-file/16-MiB bounds; safe confined opens and cancellable
blocking isolation remain required before the production path is accepted.

Host follow-up: every explicit Evaluate request now creates its own context from the requested
goal/current workspace, rather than caching the first goal indefinitely. After assessment the host
reads the workspace revision again and calls `observe_current_subject` before projecting a verdict
or invoking Delivery. An unreadable workspace returns Unavailable. Existing Runtime contract tests
exercise workspace-only staleness and irreversible invalidation of the old PASS; a production-path
race regression and asynchronous confined scanning are still required.

## Local Implementation Checkpoint — 2026-09-30

Confined capture now pins a cap-std directory handle and uses one-component no-follow directory
traversal and no-follow/nonblocking file opens, validating the opened handle before reading.
Eight focused Runtime evidence tests passed, including ambient root replacement, directory
symlink and socket rejection. Replacing the ambient root leaves this source pinned to the old
directory; reconciling that identity with the live tool workspace remains a production residual.

The authorized local commit preserves this partial implementation, not a push-ready or accepted
H6 candidate. Remaining I287 work: Runtime-produced semantic observations without fabricated
validation PASS, cancellable blocking isolation, production host race/restart/goal-change tests,
independent security/API approval, and native acceptance. Goal verdict and Mission-level Delivery
must remain distinct; missing Mission evaluation cannot be synthesized. No broad workspace
content upload or automatic validation-command execution is authorized by this checkpoint.

Local verification after confined capture: `./scripts/release_preflight.sh` passed formatting,
workspace locked check and workspace Clippy with `-D warnings`. Its workspace test compilation
was interrupted with SIGINT (exit 130) when free disk fell to approximately 1.6 GiB; the full
preflight and full workspace test suite have not passed for this checkpoint. Prior focused tests
are not a substitute for that remaining gate. No remote candidate was pushed.

Semantic-evidence follow-up: local code now adds a separate bounded artifact-observation request
field, rather than converting file collection into validation PASS. Runtime hooks register matching
permission-allowed successful file calls; Desktop requests fresh confined content observations.
Four registry tests and thirteen evaluator tests passed. Snapshot work now uses a single retained
blocking-worker slot with bounded caller wait. Full production integration/race tests, native-feature
build, incremental security/API review and H6 remain outstanding. The API migration is recorded in
ADR-084. These results do not supersede the remaining full-preflight gate above.

Latest local correction (2026-09-30): the bounded snapshot retains per-file digests and rejects
artifact content or deletion observations that do not belong to that exact snapshot, including an
A-to-B-to-A race. Source and registry verify the pinned workspace directory identity before and
after capture and fail closed if the ambient tool root was replaced. The Desktop production-path
test covers an authorized write, current evaluation, workspace mutation during assessment,
superseding submission cancellation, and an empty post-restart registry. The observation contract
is deliberately limited to current contents at an authorized successful path; hooks do not claim
an atomic execution handle for arbitrary embedding tools.
