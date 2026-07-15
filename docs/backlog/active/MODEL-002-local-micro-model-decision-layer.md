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

## Reference Implementation: oh-my-pi Session Title Generator

oh-my-pi (`can1357/oh-my-pi`, TypeScript) ships a non-authoritative micro-model helper
for one specific task — session title generation — that already implements most of the
architecture direction above. Treat it as an empirical data point when scoping this story,
not as a port target: Talos is pure Rust and MODEL-002 is governed by Hard Boundaries
below.

### Dual-path dispatch (online default, local opt-in)

The user picks one of three modes via `providers.tinyModel` (`models.ts:1-4`):

| Key | Path | Default? |
| --- | --- | --- |
| `"online"` | `completeSimple()` against a configured `smol` role model via `pi-ai` | Yes — the session-title default |
| `"lfm2-700m"` etc. | IPC to a hidden `__omp_worker_tiny_inference` subprocess running a local model | No — explicit user opt-in |
| Anything else | Returns `null`, no fallback | Rejected after issue #3187 (silent billing leak) |

The critical rule is **never silently fall back** from local to online. omp issue #3187:
a user's local worker crashed, the code fell back through `priority.json` and silently
billed the next provider holding an API key (OpenRouter in the reporter's case). After
that, the local path returns `null` on any error and the session stays untitled —
matching MODEL-002 Hard Boundary "low confidence → fall back to existing path".

Entry: `generateSessionTitle()` — `packages/coding-agent/src/utils/title-generator.ts:72`.
Online branch: `generateTitleOnline()` — same file line 142. Local branch:
`tinyTitleClient.generate()` — `packages/coding-agent/src/tiny/title-client.ts:201`.

### Deterministic pre-filter (intent / signal check)

Before invoking any model, the first user message runs through
`isLowSignalTitleInput()` (`packages/coding-agent/src/tiny/text.ts:127`):
lowercase token match against a 78-entry filler set (`hi`, `hey`, `thanks`, `ok`, ...) plus
digit-only tokens. If every word is filler, the generator returns `null` synchronously and
the session stays unnamed until a substantive turn arrives. This mirrors the
"deterministic rules first" flow in the Architecture Direction diagram.

### Prompt and output contract (what to model after)

`prompts/system/title-system.md` is 16 lines total:

- 3-7 word target, sentence case, first word + names capitalized only;
- `<title>...</title>` wrapping required, `<title/>` for "no task";
- two worked examples in-prompt.

The `<title>` wrapper — instead of a forced `tool_choice` JSON call — is chosen because
some providers ignore or reject forced tool calls and then echo the prompt's
`{"title": "..."}` example verbatim as the session title. Markers work uniformly. See
`title-generator.ts:159-164` comment.

`maxTokens=1024` is set deliberately — title is a 3-7 word task but some backends ignore
`disableReasoning` and emit thinking tokens before the marker; raising the ceiling keeps
the `<title>` reachable when reasoning is not actually suppressed (issue #4355).

### Output parsing (anti-noise)

`extractGeneratedTitle()` (`title-generator.ts:246`) strips three real-world failure modes
before accepting the result:

1. **Thinking-block leakage** — `<think>` / `<thinking>` / `<reasoning>` tags and
   `` ```thinking `` fences are scanned; only `<title>` markers in *visible* text count
   (sentinel-based visibility test at line 278).
2. **JSON-shaped echo** — `unwrapJsonTitle()` (line 326) recovers the inner `title` field
   if the model returns the structured shape it was trained on, including partial-JSON
   salvage via `"title": "..."` regex.
3. **Casing reconciliation** — `reconcileTitleCasing()` (`text.ts:186`) walks each title
   token against the source message and applies a 5-rule decision tree: restore
   user-cased identifiers (`TinyVMM`), restore ALL-CAPS acronyms the model
   sentence-cased (`CNPG` → `Cnpg`), lowercase camelCase artifacts the model introduced
   (`dAemon` → `daemon`), but never re-shout emphatic source text. Emphatic input is
   detected via `isShoutySource()` (≥2 consecutive multi-letter ALL-CAPS tokens).

### Worker lifecycle (Rust port reference)

The local path runs in a refcount-shared subprocess:

- `spawnTinyTitleWorker()` (`title-client.ts:169`) spawns the worker lazily, wraps it in
  a `RefCountedWorkerHandle`.
- `#failedModels: Set<TinyLocalModelKey>` (line 182) blacklists a key permanently after
  the first failure — no retry storms.
- `createUnavailableWorker()` is returned instead of throwing when spawn fails
  (`title-client.ts:159`); `generate()` then returns `null` rather than degrading startup.
- The whole process is a hidden subcommand dispatched by the omp CLI entrypoint
  (`cli.ts` `__omp_worker_tiny_inference` selector). This matches the "must not make
  startup fail when the local model is missing, slow, unsupported, or corrupted" Hard
  Boundary.

### Portable findings for MODEL-002

When evaluating the three options in §Evaluation Plan, omp evidence suggests:

- **Small models need non-trivial prompt engineering** (markers, examples, post-parse
  cleanup). A pure-Rust inference engine is necessary but not sufficient — the binding
  layer that protects the rest of the system from malformed output is half the work.
- **The dual-path "online default, local opt-in" pattern** is compatible with MODEL-002's
  "deterministic rules first, local model only for bounded hints". omp's online branch
  is itself a "provider-backed hint" when the local path is unavailable, with no silent
  switching.
- **Failure isolation is the load-bearing property**, not raw inference speed. The
  `#failedModels` blacklist + unavailable-worker fallback is what keeps the rest of omp
  alive when the tiny worker misbehaves. Talos should design the equivalent gate before
  benchmarking models.

### Source anchors

- `packages/coding-agent/src/utils/title-generator.ts` — dual-path dispatcher,
  extractors, casing reconciliation, OSC 0 terminal title writer.
- `packages/coding-agent/src/tiny/text.ts` — `FILLER_TITLE_TOKENS` set,
  `isLowSignalTitleInput`, `reconcileTitleCasing`, `isShoutySource`.
- `packages/coding-agent/src/tiny/title-client.ts` — `TinyTitleClient`, refcount
  worker handle, `#failedModels` blacklist, unavailable-worker fallback.
- `packages/coding-agent/src/tiny/models.ts` — `ONLINE_TINY_TITLE_MODEL_KEY`,
  `DEFAULT_TINY_TITLE_LOCAL_MODEL_KEY = "lfm2-700m"`, local model spec table.
- `packages/coding-agent/src/prompts/system/title-system.md`,
  `title-marker-instruction.md` — the 16-line system prompt and the marker injection.

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
