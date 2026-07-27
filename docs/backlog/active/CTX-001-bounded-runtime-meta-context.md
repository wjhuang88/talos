# CTX-001: Bounded Runtime Meta Context

| Field | Value |
| --- | --- |
| Story ID | CTX-001 |
| Type | Product / context-management story |
| Priority | P1 |
| Status | Refinement — deferred behind I156 / TUI-035 real-terminal acceptance |
| Source | Maintainer request 2026-07-27 |
| Parent Epic | None |
| Depends On | MEM-005, MEM-007, ARCH-006, model context-limit metadata |
| Blocks | None |

## Identity / Goal / Value

Give the model a small, current, trustworthy Meta snapshot about the active
session—especially remaining usable context window—without repeatedly
appending status text to the conversation. The model should be able to make
better decisions about summarising, tool-output restraint, and turn planning
without the Meta channel itself causing context growth or cache churn.

## Scope

- Define one canonical runtime Meta snapshot, constructed from authoritative
  runtime/model state for an outbound provider request.
- Include only a documented, bounded allowlist of low-cardinality fields:
  active provider/model identity, known context-window limit, conservative
  projected request-context estimate, reserved output budget, and calculated
  remaining usable input context. Optional fields are omitted when unknown;
  they are never invented as zero or exact values.
- Mark derived token values as estimates unless the active provider supplies an
  exact compatible measurement. The remaining-context calculation must reserve
  the selected output/reasoning budget before reporting capacity.
- Render the snapshot as one replaceable dynamic request overlay. It must not
  append a new `Message::System` or `Message::Context` record on each turn,
  mutate durable session history, affect transcript/export output, or create a
  second source of truth for usage.
- Establish deterministic update rules: a new request receives the current
  snapshot; byte-identical semantic state produces byte-identical Meta text;
  only documented field/bucket changes alter it.
- Cap serialized Meta content with a named, tested byte/token budget. On
  overflow, omit lower-priority optional fields rather than truncating a number
  into an ambiguous value or adding another message.
- Keep the existing cache-stable system-prompt prefix byte-identical across
  Meta updates. Runtime Meta belongs only in the dynamic suffix/request layer.
- Surface the same remaining-context semantics in user-visible status only when
  they are already known, without coupling TUI presentation to provider request
  construction.

## Exclusions

- No automatic compaction policy change; MEM-005 remains the owner of when and
  how compaction runs.
- No raw prompt, message body, tool result, path, credential, token, or hidden
  tool content in the Meta snapshot.
- No new provider wire protocol, public crate API, session schema, telemetry,
  or persistence behavior without a separate compatibility/ADR decision.
- No speculative per-tool or per-file diagnostics in model context.
- No I156/TUI-035 implementation work or iteration-scope expansion.

## Decision Links And Constraints

- ARCH-006 requires the system-prompt stable prefix to remain byte-stable
  through normal session operation; Meta updates must not invalidate it.
- MEM-005 owns context-pressure and compaction policy. CTX-001 supplies a
  bounded observation, not a second compaction trigger.
- MEM-007 owns deterministic active-context compression. CTX-001 must describe
  the post-compression request estimate rather than duplicate or bypass it.
- Model context limits come from existing model metadata/config precedence;
  unavailable limits stay unavailable.
- Any required public message/protocol change is a semver/ADR gate, not an
  implicit implementation detail.

## Uncertainty And Validation Path

Before this story becomes Ready, establish which existing token values are
request-local, cumulative, or provider-reported after the fact, and choose the
conservative estimator and output-reserve policy. The design must distinguish
the provider request's projected input from cumulative session usage. Record
the named Meta budget, update-bucket granularity, unknown-value rendering, and
provider-specific exactness rules. If reliable projection cannot be obtained
without a provider protocol change, expose only known limit/usage facts and
keep remaining capacity explicitly unavailable.

## State / Status Owners

- Prompt assembly and cache boundary: `talos-agent`.
- Model limit/usage and request-state authority: current conversation/model
  owners.
- TUI display-only projection: `talos-tui`.
- Story state: this document; a future iteration owns selection and completion.

## User-Facing Documentation

- Update the model/context status documentation to distinguish context-window
  limit, projected request usage, remaining usable context, and unavailable
  values.
- Document that Meta is transient, bounded, non-persistent, and not a copy of
  session history.

## Required Reads

- `docs/backlog/active/MEM-005-context-compaction-policy.md`
- `docs/backlog/active/MEM-007-active-context-compression.md`
- `docs/backlog/active/ARCH-006-prompt-cache-stability.md`
- `docs/backlog/active/MODEL-004-catalog-runtime-integration.md`
- `crates/talos-agent/src/prompt/builder.rs`
- `crates/talos-agent/src/lib.rs`
- `crates/talos-conversation/src/engine.rs`
- `crates/talos-conversation/src/types.rs`
- `crates/talos-core/src/model.rs`

## Acceptance

- Given an active model with a known context limit and sufficient authoritative
  request inputs, when Talos builds a provider request, then exactly one bounded
  Meta snapshot reports the known limit, a clearly labelled projected estimate,
  reserved output budget, and remaining usable context.
- Given unknown context limit, request estimate, or reserve, when Meta is
  rendered, then the unavailable field is omitted or explicitly marked
  unavailable; Talos does not fabricate a remaining-token number.
- Given unchanged semantic Meta state over repeated turns, when provider
  requests are captured, then Meta bytes are identical and the stable prompt
  prefix remains byte-identical.
- Given changed model, limit, projected request bucket, or reserve, when the
  next request is built, then the one dynamic Meta snapshot updates without
  appending an additional history message.
- Given a long session of at least 100 turns, when request/session histories
  are inspected, then runtime Meta contributes at most one bounded overlay per
  request and zero persisted per-turn Meta records; its stored history cost is
  constant rather than linear in turn count.
- Given Meta serialization reaches its named budget, when optional fields would
  exceed it, then lower-priority fields are omitted deterministically and all
  retained numeric fields remain syntactically complete and truthful.
- Given compressed or large tool-result context, when the request estimate is
  calculated, then Meta reflects the actual projected provider request layer
  and does not expose raw/hidden tool content.
- Unit/request-capture tests cover known, unknown, change, budget, 100-turn,
  cache-prefix, compression, and no-persistence cases; `cargo test --workspace
  --locked` passes.

## Residuals

- The exact estimator, output/reasoning reserve, update-bucket granularity,
  and named budget are refinement decisions; no implementation is authorized
  until they are recorded.
- Selection requires a new iteration after I156/TUI-035 reaches its documented
  completion gate.
