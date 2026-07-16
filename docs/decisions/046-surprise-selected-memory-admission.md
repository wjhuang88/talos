# ADR-046: Surprise-Selected Memory Admission

## Status

Accepted for architecture (2026-07-16). Implementation is not activated; MEM-009 remains in
Refinement until its benchmark and iteration gates are satisfied.

## Context

ADR-016 separates working, episodic, semantic, and procedural memory. Its implementation keeps
the normalized session transcript (TLOG for new sessions; JSONL legacy read compatibility) as
episodic truth, consolidates selected episodes into ADD-only
semantic/procedural memory, and retrieves with FTS5, recency, evidence, and entity signals.

The weakest part is admission. `RuleBasedExtractor::compute_confidence()` currently treats an
explicit marker such as `remember` as high confidence, a long user message as medium confidence,
and most other eligible messages as a default-confidence candidate. This mixes three different
questions:

1. Is the statement well supported?
2. Is it new relative to what Talos already knows?
3. Did it actually affect behavior enough to deserve scarce durable memory?

HOLA (Hippocampal Linear Attention) separates a lossy compressive state from a bounded exact cache
and admits exact KV pairs by the model-internal committed residual magnitude `β · ||e||`: retain
what the compressed state represented poorly and what the model actually wrote strongly. Talos
does not own provider layers, delta-rule state, `β`, `e`, or KV tensors, so it cannot implement or
claim HOLA. The transferable design principle is narrower: scarce exact or consolidated memory
should be selected by novelty multiplied by demonstrated utility, not by message length or recency.

Source: [A Hippocampus for Linear Attention: An Exact Memory for What the Recurrent State
Forgets](https://arxiv.org/pdf/2607.02303), arXiv:2607.02303v1, 2026-07-02.

## Constraint Decomposition

| Constraint | Type | Source | Can Change? |
| --- | --- | --- | --- |
| Normalized session transcript remains the sole source of exact episodic content; TLOG is the new-write format and JSONL is legacy read compatibility. | Hard | ADR-016, ADR-037, ADR-039, ADR-042 | No |
| Semantic/procedural memory remains ADD-only, evidence-linked, contradiction-aware, and advisory. | Hard | ADR-016 | No |
| Memory cannot grant permissions or outrank current instructions, files, tests, or ADRs. | Hard | AGENTS.md, ADR-016 | No |
| Hidden tool/system output, credentials, and raw provider responses cannot enter memory. | Hard | ADR-023, ADR-042 | No |
| Prompt injection remains bounded and default-off. | Hard | ADR-033 | No |
| Keyword/message-length confidence is a suitable admission policy. | Soft, disproven by inspection | Current `RuleBasedExtractor` | Yes; replace |
| Recency should materially influence general memory ranking or retention. | Soft | ADR-016 refinement / MEM-006 hypothesis | Yes; narrow |
| A Talos proxy can reproduce HOLA's `β · ||e||`. | Invalid assumption | Provider boundary | No; reject |
| A sparse reference index can improve exact recall without copying episode content. | Assumption | This decision | Benchmark before implementation |

## Decision

### 1. Replace the current admission heuristic

Candidate admission will be based on two separately explainable signals:

```text
admission_score = novelty × committed_utility
```

- `novelty` estimates how poorly existing semantic/procedural memory covers the candidate. It may
  use existing FTS/entity/contradiction signals, but no vector dependency is introduced.
- `committed_utility` estimates whether the information changed or repeatedly guided observable
  behavior: a user correction or durable preference, a plan/code/decision change, a successful
  recovery, a validated result, or later reinforcement.

Both values are bounded to `[0, 1]`. An admission decision must retain a small reason-code set and
the component scores. The exact formula, thresholds, and weights are benchmark outputs, not facts
chosen in this ADR.

`MemoryItem.confidence` remains epistemic/evidence confidence. It must not be reused as admission
score. Evidence remains a separate trust and retrieval signal; novelty does not make a claim true.

The current keyword/message-length `compute_confidence()` policy is superseded as an admission
rule. Explicit user phrases may contribute to committed utility, but cannot alone establish truth
or admission.

### 2. Narrow recency to version/freshness resolution

Recency remains useful when selecting among time-varying facts or same-key versions. It must not
decide whether an episode deserves admission, and it must not evict an old, high-utility memory
merely because it is old.

General retrieval continues to require query relevance, evidence, entities, contradictions, and
bounded results. Any reduction of the current recency weight requires the MEM-009 benchmark; this
ADR does not silently change runtime ranking.

The MEM-006 `RecencyBoostRanker` hypothesis is therefore narrowed: it may be evaluated for explicit
freshness/version queries, but it is rejected as a general admission or retention policy.

### 3. Reuse the session transcript for exact content

Talos will not create a second episodic content store. New session content remains in TLOG and
legacy JSONL remains readable through the normalized transcript boundary. If benchmark evidence
justifies exact cross-session recall, the maximum additional structure is a sparse reference index
containing:

```text
session_id, entry_id, turn_id, content_hash,
admission_score, novelty, committed_utility, reason_codes, created_at
```

The index resolves content through the normalized session transcript API. It stores no copied
message body, tool payload, provider response, credential, approval state, or UI state. Missing or
deleted source entries fail closed and are eligible for orphan cleanup. Session deletion must
remove or invalidate its references.

This index is not a fifth memory layer. It is an optional sparse access path over ADR-016 episodic
memory. It is not required for the first admission-policy benchmark.

### 4. Keep exact and generalized recall distinct

If sparse exact recall is implemented, exact transcript entries and consolidated semantic or
procedural memories remain distinct result categories. They are not averaged into one opaque
score. Exact recall uses a high threshold, small top-k, and a top-1/top-2 margin; ambiguous exact
matches return no exact episode rather than several weak candidates.

Every result remains advisory and includes stable provenance. Automatic prompt injection remains
default-off under ADR-033 and requires a separately activated experiment.

## Replacement Map

| Existing direction | Decision |
| --- | --- |
| Keyword/message-length candidate confidence as admission | Replace with benchmarked `novelty × committed_utility` |
| `MemoryItem.confidence` | Keep as epistemic/evidence confidence |
| Normalized session transcript | Keep as sole exact episodic content; TLOG new-write, JSONL legacy read |
| ADD-only semantic/procedural memory | Keep |
| FTS5, entity, evidence, contradiction retrieval | Keep |
| Recency in same-key/freshness resolution | Keep |
| Recency as general admission/retention importance | Reject |
| Weighted associative graph | Orthogonal; not required by MEM-009 |
| Automatic memory injection | Keep default-off; no change |
| Vector DB, model-internal KV cache, RMSNorm, simulated `β · ||e||` | Reject |

## Dependency And API Boundary

- Primary implementation belongs in `talos-memory`.
- `talos-memory` must not depend on `talos-session`; an optional sparse-reference resolver is owned
  by a CLI/Runtime coordinator or a small trait in an acyclic common boundary.
- Existing SQLite remains the only structured dependency. No new crate dependency is authorized.
- Changes to public `MemoryCandidate`, `EpisodeExtractor`, or retrieval result shapes require a
  compatibility review and migration note before implementation. Prefer an additive admission
  stage over changing the extractor contract merely to carry scoring state.
- No TLOG, JSONL compatibility, transcript, permission, provider, or event format change is authorized.

## Validation Gate

Before implementation can be selected into an iteration, MEM-009 must define a deterministic
fixture corpus containing at least:

- explicit preference and user correction;
- long but routine text;
- repeated/covered knowledge;
- validated plan or code change;
- failure followed by successful recovery;
- old high-utility fact versus recent low-utility noise;
- contradiction and changed fact;
- hidden tool/system output and credential-shaped content.

The comparison must include current heuristic, recency, novelty-only, and
`novelty × committed_utility`. It must report precision, important-item recall, duplicate admission,
contradiction handling, old-important retention, exact-reference hit/miss if evaluated, context
cost, and deterministic score explanations. No runtime default changes before this evidence.

## Consequences

- The memory pipeline becomes more selective without adding a new content store or dependency.
- Admission, evidence confidence, retrieval relevance, and recency have distinct meanings.
- Some high-value events may require session-window analysis instead of per-message classification.
- The first slice is an offline policy benchmark; sparse exact indexing and prompt integration are
  separately gated outcomes, not bundled requirements.
- HOLA remains cited as inspiration only. Talos metrics cannot be presented as the paper's neural
  mechanism or inherit its reported model results.

## Reversal Trigger

Revisit if representative Talos fixtures show that the replacement policy is no more precise than
the current heuristic, cannot produce stable reason codes, or requires provider-internal signals.
Retain the current behavior until the replacement wins its benchmark; do not ship a speculative
formula because the architectural direction is accepted.

## Related

- ADR-016 Layered Agent Memory Architecture
- ADR-033 Associative Memory Injection Policy
- ADR-039 Runtime Event Semantic Single-Flow Boundary
- ADR-042 Embedded Durable Runtime Session Boundary
- MEM-009 Surprise-Selected Memory Admission
- MEM-006 Memory System Pattern Research
- MEM-008 Weighted Associative Memory Graph
