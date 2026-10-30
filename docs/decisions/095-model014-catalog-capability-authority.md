# ADR-095: Catalog-Inferred Custom-Model Capability Authority

## Status

Proposed for MODEL-014 / I311. No implementation authority until the atomic
claim/activation and this ADR merge. I310/ADR-094 belong to #811 Gemini.

## Context

MODEL-013 inferred only context. MODEL-014 requires one canonical identity to
resolve context, image, reasoning, output and variants with per-field provenance.
Existing manual boolean metadata loses Unknown, and configured context suppresses
independent output inference. Inference is named-model evidence, not endpoint proof.

## Decision

## One identity, independent fields

Refactor the existing exact-ID / one-prefix unique-candidate resolver to return a
canonical catalog record. Derive context, output, image and reasoning from that
same record. Exact ambiguity stops resolution; do not fall through to a prefix
match. No fuzzy match, endpoint request, pricing or role inference.
Each field follows the four-layer conceptual precedence below; because probe
evidence is not implemented, current execution resolves explicit configuration >
selected catalog field > unknown.
A configured context must not suppress independent output inference.
An unknown numeric field remains None in metadata; existing runtime conservative
context fallback remains a distinct operational fallback, never catalog evidence.


## Explicit negative and compatibility

Add optional ModelConfig.reasoning_supported: Option<bool>, serialized only when
present. false overrides catalog capability and suppresses reasoning presets and
request parameters, including a configured effort. true establishes a user
capability assertion but cannot make an unsupported protocol accept parameters.
Existing reasoning options remain parameter settings, not capability evidence.
Absent capability override preserves catalog-derived or unknown behavior.
Image uses existing Option<bool>; explicit false remains authoritative.
This adds a field to a public Rust struct: document source migration for explicit
struct literals (add None or use ..Default::default()) in the ADR; existing TOML
continues to deserialize. Generate/update schema through the normal repository
path. Derived fields must never be written during load/save or model switching.


## Provenance and variants

Expose per-field configured/catalog-inferred/unknown plus matched catalog
provider and ID in an additive resolution API. Keep existing metadata consumers
working via projection; do not pretend a whole Manual row describes all fields.
Display inference as named-model catalog evidence, never endpoint verification.
Copy supported reasoning presets from the matched catalog only when resolved
reasoning is true; preserve default selection and explicit effort semantics.
Adapter capability remains a second independent gate. Do not invent effort,
budget or output parameter mappings for unsupported adapters.


## Concrete adapter decision matrix (source verified)

| Protocol | Reasoning activation parameters | Output field | Incompatible setting |
|---|---|---|---|
| openai-chat | reasoning_effort low/medium/high; existing default medium when options exist | max_completion_tokens | budget_tokens omitted with bounded diagnostic |
| openai-responses | reasoning.effort low/medium/high plus existing summary:auto | max_output_tokens | budget_tokens omitted with bounded diagnostic |
| anthropic-messages | thinking.enabled with budget_tokens, existing default 80% of max_tokens and strictly below max_tokens; existing trailing tool-use replay guard retained | max_tokens | effort preset has no numeric-budget mapping; omit effort with bounded diagnostic |
| unsupported protocol | no inferred parameter mapping | none | existing provider construction error, never fallback to a different endpoint/protocol |

Explicit output_limit wins over inferred output_limit; it is a configured request
ceiling, not independent verification of a gateway hard maximum. Inferred catalog
output is the request ceiling only when explicit output is absent. Preserve the
existing conservative fallback only when both are absent. Validate zero/range
according to existing schema and enforce provider-native constraints; don't invent
an endpoint limit or silently replace an explicit value with catalog data.
Factory effective_output_limit currently ignores inferred output and only uses
configured output/fallback. Update it to consume resolved output consistently with
request admission. Chat currently emits max_completion_tokens only when reasoning
options exist: independent output emission is required even when reasoning is off.
Add request-capture coverage for reasoning=false plus an inferred/explicit cap.

Replay must be gated separately: removing ReasoningOptions alone is insufficient
because OpenAI Chat request serialization reads Assistant reasoning from history.
The false override must prevent replay on the wire without deleting authoritative
raw history or breaking tool pairing. Preserve existing provider/model provenance
matching and Responses encrypted reasoning confidentiality.

Keep the provider factory's caller-materializes-variants contract. Add explicit
negative-capability enforcement to factory construction as defense for direct
callers, and resolve output through the shared API; no automatic variant selection
inside the factory. Existing lower-level adapter builders remain raw APIs with
explicit caller-supplied options, not config resolution entrypoints.



### Precedence and explicit reasoning compatibility

Conceptually explicit config > valid applicable fresh probe > catalog > unknown.
Probe does not exist in this implementation; MODEL-011 owns its future freshness,
applicability, schema and execution. No fake probe provenance is produced.

| Configuration | Capability and invocation |
|---|---|
| reasoning_supported=false | Suppress effort, budget and replay on wire; retain raw history and tool pairing. |
| reasoning_supported=true | User assertion enables supported protocol options; does not verify endpoint. |
| absent, unique catalog identity true | Use catalog capability with inferred source and supported configured/variant parameters. |
| absent, unique catalog identity false plus explicit reasoning options | Catalog negative disables effort/budget/replay on wire; retain persisted options/history and emit bounded diagnostic. User may explicitly set reasoning_supported=true to override the catalog assertion. |
| absent, unknown/ambiguous identity | Capability unknown; retain existing explicit reasoning options for compatibility, without labeling them inferred support. |
| valid selected variant plus explicit effort | Selected variant overrides effort as current behavior; budget/replay remain configured unless explicit capability false. |

### Image authority amendment

This explicitly supersedes ADR-050 sections 8/10's no-name-inference prohibition
only for MODEL-014's unique exact-ID or single-prefix canonical catalog match.
Inferred true may enable an explicit user attachment; unknown/ambiguous stays
Unknown, explicit false stays Unsupported. File permissions, image validation,
size/privacy boundaries and protocol gating still apply. No fuzzy model-name,
endpoint, pricing, role or routing inference and no paid/background probes.

### Public API migration

Adding optional reasoning_supported preserves existing TOML but breaks exhaustive
Rust ModelConfig literals; use reasoning_supported: None or ..Default::default().
Record this in unreleased SDK migration notes. Any release carrying this source
break requires the next pre-1.0 minor version; no release/tag is in this scope.
Catalog-negative reasoning now suppresses legacy configured options when no
explicit capability override exists; document the diagnostic and opt-in override
as a behavior migration. Unknown identity retains legacy explicit options. Add
request-capture regression for catalog false plus effort/budget/replay and the
explicit true override, with no raw-history deletion.
Existing ContextLimit APIs retain behavior through the new resolution projection.
New result types expose per-field Option values and sources without replacing
existing public model metadata fields. Derived defaults never persist on save.

## Validation and consequences

Test exact/prefix/ambiguous/unknown/missing fields, explicit negatives and partial
numeric overrides, round-trip non-persistence, variant switching, protocol request
capture including reasoning-off output caps and replay omission, actual safe image
attachment and a local custom-provider walkthrough with source display. Run pinned
locked workspace/preflight and governance checks plus independent API/security
review. Additional diagnostics must not disclose secrets or opaque reasoning.
Unknown source remains distinguishable even where legacy metadata projects false.
