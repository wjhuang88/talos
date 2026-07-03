# MODEL-002: Local Micro-Model Decision Layer

## Outcome

Talos has a researched, ADR-ready decision on whether to embed a small local model for low-risk
micro decisions such as intent classification, routing hints, tool-candidate narrowing, title
generation, and compaction pre-classification.

## Status

Research. Candidate input to I036.

## Priority

P3.

## Origin

User idea on 2026-06-19: evaluate whether Talos can embed a local small model to quickly handle
micro decisions such as routing, intent recognition, simple tool-choice judgments, and title
summaries.

## Problem

Talos currently relies on deterministic logic and the active remote/provider model for most
session decisions. Some decisions are small, repeated, and latency-sensitive:

- identifying whether a user turn is code work, planning, search, Git, configuration, or chat;
- choosing whether a request needs skills, MCP tools, local tools, web/search, or the main model;
- narrowing the candidate tool set before the main model sees the turn;
- creating short session titles and history labels;
- pre-classifying context chunks before compaction.

A local micro-model might reduce main-model token use and improve responsiveness, but it also adds
binary/model size, dependency risk, runtime complexity, and possible opaque decision behavior.

## Scope

Evaluate a local micro-model as a helper layer, not as an authority layer.

Candidate use cases:

- intent classification;
- routing hints;
- non-sensitive tool-candidate narrowing;
- session title and short summary generation;
- context compaction pre-classification and importance scoring;
- low-risk diagnostics such as "likely needs network" or "likely code-editing request".

The first acceptable design must keep deterministic rules first, then use the local model only for
bounded hints with confidence scores and explicit fallback.

## Architecture Direction

The preferred shape is:

```text
User input / session state
        |
        v
Deterministic rules
        |
        v
Local micro-model helper
        |
        v
Confidence gate
        |
        +--> high confidence: attach hint, label, or summary
        |
        +--> low confidence: ignore and fall back to existing path
```

Micro-model output must be structured and bounded, for example:

```json
{
  "intent": "code_edit",
  "confidence": 0.86,
  "needs_network": false,
  "risk": "write_file",
  "suggested_tools": ["grep", "read", "edit"]
}
```

## Hard Boundaries

- The local micro-model must not approve permissions.
- It must not bypass the permission pipeline.
- It must not be the only evaluator for write-capable tools, shell commands, network access, or
  security-sensitive decisions.
- It must not expose or fabricate hidden reasoning.
- It must not make startup, TUI, or normal provider calls fail when the local model is missing,
  slow, unsupported, or corrupted.
- It must not add native runtime dependencies without ADR review under AGENTS.md hard constraint
  #1 and ADR-010.

## Research Questions

- Do real Talos prompts benefit more from a local model than deterministic rules plus a small
  hand-labeled classifier?
- Which engine shape is acceptable: pure Rust inference, ONNX Runtime, llama.cpp-style runtime,
  candle, burn, or another option?
- What model size, quantization, memory mapping, and load strategy meet startup and TUI latency
  expectations?
- Can inference be run with strict timeout and cancellation semantics?
- Can outputs be validated against a schema and discarded on invalid/low-confidence results?
- How large is the release artifact impact for macOS, Linux, and Windows?
- Can users disable the feature fully and can CI/builds run without downloading model weights?
- Should model weights be bundled, manually installed, or downloaded through the shared optional
  asset distribution flow?
- Does it measurably reduce main-model token use or latency for routing/title/compaction tasks?
- How should failures and decisions be surfaced in observability without cluttering history?

## Evaluation Plan

1. Build a small offline evaluation set from real Talos interactions:
   - user intent;
   - expected route;
   - risk level;
   - candidate tool set;
   - title/summary target where applicable.
2. Compare three approaches:
   - deterministic rules only;
   - lightweight structured classifier without generative local model;
   - embedded local micro-model.
3. Measure:
   - accuracy;
   - false-positive rate for sensitive paths;
   - latency;
   - memory footprint;
   - binary/model asset size;
   - implementation and dependency risk.
4. Promote only the smallest use cases with clear benefit.

## Acceptance Criteria

- [ ] A report compares deterministic rules, lightweight classifier, and local micro-model options
      against the same labeled evaluation set.
- [ ] The report identifies which use cases are allowed in v1 and which are explicitly forbidden.
- [ ] Dependency options are assessed for Rust-first fit, native-code risk, model-weight
      distribution, optional runtime asset installation, offline build behavior, and release size.
- [ ] The proposed runtime contract uses structured output, confidence thresholds, timeout,
      cancellation, and fallback.
- [ ] Permission and write-tool decisions remain outside the micro-model authority boundary.
- [ ] A follow-up ADR is drafted before adding any inference runtime or model asset to the
      workspace.

## Non-Goals

- Do not implement local model inference in this item.
- Do not bundle model weights before an accepted ADR.
- Do not use the local model as an auto-approval engine.
- Do not replace the active provider model.
- Do not route security-sensitive work based only on probabilistic output.

## Relationship To Other Work

- `MODEL-001` supplies model metadata and capability vocabulary; MODEL-002 decides whether Talos
  should embed a local helper model at all.
- `DIST-001` owns the optional asset distribution strategy if local model weights are installed
  after Talos itself is installed. I091 A8 records the shared policy in
  `docs/proposals/optional-runtime-asset-distribution.md`: model weights are data-only optional
  assets, must be checksum/signature verified, may be disabled/offline/pre-seeded, and never become
  permission authority.
- `MEM-005` may consume micro-model classification only as a compaction hint.
- `TOOL-002` and the permission pipeline remain authoritative for tool execution.
- `SKILL-001` and `MCP-001` may use routing hints only after startup discovery and provenance are
  stable.

## Residual Work Destination

If the research is positive, create a dependency ADR and a narrow implementation story for one or
two low-risk use cases, likely session title generation and intent/routing hints. If the research
is negative, keep deterministic routing and record the micro-model idea as watch-only.
