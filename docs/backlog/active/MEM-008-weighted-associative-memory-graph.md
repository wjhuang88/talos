# MEM-008: Weighted Associative Memory Graph

| Field | Value |
|-------|-------|
| Story ID | MEM-008 |
| Priority | P2 |
| Status | Research |
| Depends On | MEM-001; MEM-006 |
| Relates To | MEM-004; MEM-005; MEM-007; TOOL-014 |
| Blocks | Associative recall; memory activation graph; optional associative memory prompt injection |
| Origin | User request 2026-06-30; design a data structure for impression strength, association strength, and bounded multi-hop recall |

## Outcome

Evaluate and design a **weighted associative memory graph** that can represent:

- how strong or salient a memory element's impression is;
- how strongly two memory elements are associated;
- how a query or seed element can activate related elements through bounded multi-hop association;
- whether a short, budgeted associative memory section should be injected automatically on each
  model turn.

This is an engineering memory structure inspired by associative recall. It must stay explainable,
bounded, testable, and subordinate to current session context, permissions, source files, and ADRs.
It must not be described as a faithful simulation of human memory.

## Problem

Current Talos memory supports semantic/procedural storage, entity linking, evidence-backed
retrieval, recency/evidence scoring, and bounded prompt injection. That is useful for direct recall,
but it does not explicitly model "this memory reminds me of that memory" across multiple hops.

Examples where direct keyword recall may be insufficient:

- "the browser login-state plan" should associate WEB-005, TOOL-014, BrowserSkill, `fetch_url`
  backend convergence, permission gating, and page records.
- "the release hardening work" should associate crates.io publication, README updates, install
  script separation, release tags, and publish guard policy.
- "that recurring sandbox problem" should associate escape-vector review, process hardening,
  permissions, command execution, and prior validation failures.

Talos needs a structured way to represent and score these associations without introducing a graph
database, a vector dependency, or opaque model-only recall.

## Proposed Data Model

Start with SQLite-backed graph tables inside `talos-memory`; do not add a graph database.

```text
MemoryNode
- id
- kind: memory | entity | procedure | project | file | user_pref | concept
- label
- impression_strength: 0.0..1.0
- confidence: 0.0..1.0
- evidence_count
- last_accessed
- status: active | contradicted | suppressed | forgotten

MemoryEdge
- from_id
- to_id
- relation: mentions | supports | contradicts | used_with | same_task | same_entity | follows
- strength: 0.0..1.0
- confidence: 0.0..1.0
- evidence_count
- last_reinforced
- decay_policy

ActivationTrace
- query_id
- seed_nodes
- visited_nodes
- path_score
- hop_count
- reason
```

`MemoryNode` can reference existing memory items and entities rather than duplicating their full
content. The graph is an index and recall aid, not a second source of truth.

## Association Scoring

Research a deterministic scoring function such as:

```text
score(next) =
  current_activation
  * edge.strength
  * edge.confidence
  * node.impression_strength
  * recency_decay
  * relation_weight
```

Hard bounds:

- maximum hops: default 2, never unbounded;
- per-hop fanout: top-k only;
- minimum score threshold;
- allowed relation set;
- token budget for model-facing output;
- output must include path/evidence, not only ranked labels.

## Agent Value Hypothesis

Associative recall may improve:

- cross-session project continuity;
- weakly specified user references such as "that earlier release issue";
- recurring failure pattern recall;
- preference and governance recall when the user references a task class rather than a keyword;
- handoff quality by surfacing adjacent requirements and prior decisions.

The graph must not be used to prove facts. A strong association only means "likely relevant"; source
files, tests, current user instructions, and ADRs remain authoritative.

## Automatic Associative Memory Injection Research Goal

Research whether each model turn should automatically receive a short, bounded associative memory
section.

Candidate behavior:

```text
# Associative Memory
- Related: TOOL-014 -> WEB-005 via conditional backend design (score 0.82, evidence ...)
- Related: MEM-008 -> MEM-006 via multi-source memory query (score 0.61, evidence ...)
```

Questions to answer before implementation:

- Does automatic associative injection improve task continuity compared with explicit
  `recall_memory(mode = associative)` only?
- What token budget is safe: 300, 600, or 1000 tokens?
- Should injection trigger every turn, only on low-confidence references, or only when memory
  retrieval has high score?
- Does it pollute model context with weak associations and increase hallucination risk?
- Does it preserve ARCH-006 prompt-cache boundaries by living only in the dynamic suffix?
- How should it interact with MEM-007 active context compression and MEM-005 compaction?
- How should users disable it or inspect why a memory was injected?

Default stance until proven otherwise: automatic associative injection is **off**. The first
implementation should prefer explicit or runtime-triggered associative recall.

## Implementation Phases

### Phase 0: Research And Metrics

- Define benchmark transcripts for direct recall vs associative recall.
- Decide success metrics: recall relevance, false-positive rate, token cost, and usefulness in
  agent task completion.
- Decide whether automatic associative injection is rejected, opt-in, or default-off experimental.

### Phase 1: Graph Schema And Deterministic Association

- Add graph tables to `talos-memory`.
- Build nodes from existing `MemoryItem` and entity records.
- Build edges from shared evidence, shared entities, same task/session, procedural usage, and
  contradiction/support relationships.
- Implement bounded multi-hop association with deterministic scoring.

### Phase 2: Recall Integration

- Extend memory retrieval to support `mode = direct | associative`.
- Return association paths, scores, and evidence.
- Keep associative recall bounded and advisory.

### Phase 3: Prompt Injection Experiment

- Add an opt-in associative memory prompt section.
- Verify stable prefix is unchanged when enabled/disabled.
- Measure token overhead and false-positive context pollution.

## Non-goals

- No graph database dependency.
- No vector database dependency.
- No claim that Talos reproduces biological human memory.
- No unbounded spreading activation.
- No use of association strength as proof of factual truth.
- No automatic hard deletion or mutation of memories.
- No automatic prompt injection until Phase 0 metrics justify it.

## Acceptance

- [ ] Produce a design note comparing weighted graph recall with current FTS/entity retrieval.
- [ ] Define schema and migration strategy for weighted nodes/edges/traces.
- [ ] Define deterministic bounded association scoring with tests.
- [ ] Decide whether automatic associative memory injection should be rejected, opt-in, or
      implemented as default-off experimental behavior.
- [ ] If implemented, prove associative memory injection does not modify the stable prompt prefix.
- [ ] Demonstrate at least three agent workflows where associative recall improves useful context
      without overwhelming the model.

## Required Reads

- `docs/backlog/active/MEM-001-layered-memory-foundation.md`
- `docs/backlog/active/MEM-006-memory-pattern-research-headroom.md`
- `docs/backlog/active/MEM-007-active-context-compression.md`
- `docs/backlog/active/MEM-005-context-compaction-policy.md`
- `docs/backlog/active/MEM-004-workspace-session-topology.md`
- `docs/backlog/active/TOOL-014-conditional-tool-backends.md`
- `docs/backlog/active/ARCH-006-prompt-cache-stability.md`
- `docs/decisions/016-layered-memory-architecture.md`
- `crates/talos-memory/src/`
