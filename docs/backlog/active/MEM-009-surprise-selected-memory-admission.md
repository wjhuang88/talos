# MEM-009: Surprise-Selected Memory Admission

**Type**: Technical Story / Policy Benchmark
**Parent Epic**: MEM-001
**Status**: Complete (2026-07-16, I137/I138) — 4-policy benchmark with JSON output; novelty × utility policy implemented with noise detection and sensitive filter
**Priority**: P2
**Source**: Maintainer request 2026-07-16; arXiv:2607.02303
**Iteration**: None

## Identity / Goal / Value

**Recipient**: Talos users relying on cross-session memory and maintainers auditing why something
was remembered.

Replace the current keyword/message-length memory-admission heuristic with a deterministic,
explainable `novelty × committed_utility` policy. Preserve the normalized session transcript as the
only exact episode content (TLOG for new writes, JSONL for legacy reads), keep semantic/procedural
memory evidence-backed, and avoid adding a redundant memory layer.

## Scope

- Build a deterministic offline benchmark for candidate admission policies.
- Define observable novelty and committed-utility signals available at the Runtime boundary.
- Keep evidence confidence separate from admission score.
- Decide benchmarked admission thresholds and reason codes.
- Evaluate whether a content-free sparse TLOG entry-reference index materially improves exact
  recall; do not implement it unless the benchmark justifies it.
- Narrow recency to freshness/same-key version resolution rather than general importance.
- Record public-API and SQLite migration impact before implementation selection.

## Exclusions

- No model training, linear-attention, GDN/DeltaNet, KV-cache, RMSNorm, or simulated `β · ||e||`.
- No second copy of transcript content and no fifth memory layer.
- No vector/graph database dependency.
- No dependency from `talos-memory` to `talos-session`.
- No TLOG, JSONL compatibility, TranscriptEntry, provider, event, permission, credential, or
  approval-state change.
- No default-on direct or associative memory injection.
- No implementation inside the currently complete I019 or I134 baselines.

## Dependencies

- ADR-046: admission replacement and non-redundancy boundary.
- ADR-016: four-layer memory, ADD-only consolidation, provenance, contradictions.
- ADR-033: automatic associative injection remains default-off.
- I128/ADR-042: normalized transcript and durable session lifecycle if sparse references advance.

## Decision Links And Constraints

- ADR-046 supersedes keyword/message-length confidence as the admission rule; it does not silently
  change current runtime behavior before benchmark evidence.
- `MemoryItem.confidence` remains evidence confidence.
- The normalized session transcript remains exact episodic truth; an optional index contains
  references and scores only.
- Current instructions, files, tests, and ADRs outrank all memory output.
- Every model-facing result is bounded, provenance-bearing, filtered, and advisory.

## Uncertainty And Validation Path

The repository does not yet have deterministic definitions for novelty and committed utility.
Candidate signals include existing FTS/entity coverage, contradiction, explicit correction or
preference, observable plan/code/decision changes, successful recovery, validated outcomes, and
later reinforcement. Their weights and thresholds must be selected from fixtures rather than
asserted in documentation.

The sparse reference index is optional. Phase 1 can close with a decision not to build it if direct
semantic/procedural retrieval plus TLOG history already meets exact-recall needs.

## State / Status Owners

- This owner document controls MEM-009 scope and readiness.
- ADR-046 controls accepted architecture and supersession boundaries.
- A future iteration with a new ID controls activation and execution evidence.
- `docs/backlog/PRODUCT-BACKLOG.md` and `docs/BOARD.md` are derived views; the Board should change
  only after iteration activation or material operating-priority selection.
- Any unselected sparse-index or prompt-injection work remains a recorded residual here; it must not
  be folded into another iteration silently.

## User-Facing Documentation

If implementation is selected, update memory configuration/reference documentation to explain:

- what signals may cause a memory to be admitted;
- how to inspect admission reasons and evidence;
- that exact episode content remains in session storage (TLOG for new sessions);
- that memory never grants authority or permission;
- how memory retrieval/injection is disabled and bounded.

## Required Reads

- `docs/decisions/046-surprise-selected-memory-admission.md`
- `docs/decisions/016-layered-memory-architecture.md`
- `docs/decisions/033-associative-memory-injection-policy.md`
- `docs/decisions/042-embedded-durable-runtime-session-boundary.md`
- `docs/iterations/I019-layered-memory-foundation.md`
- `docs/iterations/I050-memory-consolidation-pipeline.md`
- `docs/iterations/I051-bounded-memory-prompt-injection.md`
- `docs/backlog/active/MEM-001-layered-memory-foundation.md`
- `docs/backlog/active/MEM-006-memory-pattern-research-headroom.md`
- `docs/backlog/active/MEM-008-weighted-associative-memory-graph.md`
- `crates/talos-memory/src/consolidation.rs`
- `crates/talos-memory/src/store.rs`
- `crates/talos-memory/src/prompt.rs`
- `crates/talos-session/src/durable.rs`
- [HOLA paper](https://arxiv.org/pdf/2607.02303)

## Acceptance For Behavior

- Given a long routine message and a short validated correction
  When admission policies are compared
  Then the replacement prefers the correction for explainable novelty and utility reasons rather
  than length.

- Given an old high-utility item and recent low-utility noise
  When admission/retention is evaluated
  Then recency alone cannot displace the high-utility item.

- Given a candidate already covered by memory
  When novelty is low
  Then it is rejected or deprioritized without overwriting existing evidence.

- Given credential-shaped, hidden tool/system, raw provider, approval, or UI-state content
  When admission runs
  Then it is rejected before any memory or sparse reference is written.

- Given an ambiguous exact-recall query
  When no candidate clears both minimum score and winner margin
  Then no exact episode is returned.

- Given a deleted session or missing entry
  When a sparse reference is resolved
  Then resolution fails closed and exposes no stale content.

## Acceptance For Technical / Governance Work

- [ ] A fixture corpus covers correction, preference, routine length, duplication, validation,
      recovery, recency conflict, contradiction, and sensitive-content rejection.
- [ ] Current heuristic, recency, novelty-only, and `novelty × committed_utility` are compared on the
      same corpus.
- [ ] Evidence reports precision, important-item recall, duplicate admission, contradiction
      handling, old-important retention, deterministic reason codes, and context cost.
- [ ] The chosen policy beats the current heuristic on the agreed primary metrics before runtime
      behavior changes.
- [ ] Public-API compatibility and SQLite migration plans are recorded before coding.
- [ ] If sparse references advance, tests prove no content duplication, session-delete cleanup,
      path/ID validation, missing-source failure, and transcript filtering.
- [ ] Prompt injection remains unchanged unless separately activated under ADR-033.
- [ ] Locked fmt/check/clippy/test, release preflight, governance validation, and `git diff --check`
      pass for any implementation iteration.
- [ ] Owner, future iteration, backlog, Board, README/config reference, and any source issue are
      synchronized at implementation closeout.

## Minimum Deliverable Slice

One offline benchmark and decision report. It may select a deterministic admission policy without
changing persistence, retrieval, or prompt behavior. Sparse indexing is a separately accepted
follow-on only if the report demonstrates additional value.

## Residual Destination

- Sparse exact-reference indexing: remains in MEM-009 until benchmark-selected or rejected.
- Automatic direct/associative injection: ADR-033 and a separately authorized story.
- Vector retrieval: STORE-001 or a future dependency Spike.
- Model-internal HOLA support: provider/model selection research, not `talos-memory`.
