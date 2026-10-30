# MODEL-014 authority/schema proposal for atomic activation review

Not effective until the governance proposal is independently reviewed and merged.
Preserves the original MODEL-014 owner and all eight behavior acceptance rows.

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

## Required tests and walkthrough

Unique exact / single-prefix identities; duplicate exact and normalized matches;
missing fields; context override plus output inference; explicit false surviving
catalog changes; configured true with unknown identity; config round-trip without
derived persistence; false suppressing variants and reasoning request parameters;
image eligibility plus permission/content checks; adapter output caps and refusal
for unsupported protocols. Local fixture walkthrough must exercise picker/source
display and real Talos image/reasoning request construction, no paid probes.

## Activation sequence

Finish I308 exact-head CI and merge first. Restore original MODEL-014 owner from
1bda5181f033b49f3f05f304cfd1bef2489e6a04 and reconcile related MODEL-011/013
references without reopening their completed baselines. Inventory nonterminal
iterations and current target claims; allocate unused iteration/ADR IDs only then.
One atomic governance PR can close merged I308 with existing SHA and activate
MODEL-014 after all predecessor acceptance, Issue and derived views are reconciled.

## Confirmed consumer gap — 2026-10-30

Current `all_models_with_catalog` manufactures a manual ModelMetadata with default
false capabilities, and `ImageInputCapability::from_metadata(Some(_))` maps false
to Unsupported. Thus a custom model without evidence currently loses Unknown.
The new field-resolution API must retain Option<bool>/source independently of the
legacy boolean metadata projection. Route actual CLI print, runtime builder and
attachment eligibility through resolved capability, including unknown diagnostics;
merely changing metadata display does not satisfy MODEL-014.

`Config::validate` only warns about reasoning capability; it does not suppress
request fields. The explicit false implementation must be enforced at effective
runtime materialization and provider construction, including direct factory users.
Add negative request-capture tests rather than treating a warning as enforcement.

Migration must avoid broadening authority through known-name inference: inferred
image support allows only the existing explicit-user attachment flow, retaining
protocol gating, normal file permission, image format/size checks and privacy.

## Live overlap inventory update

PR #811 at b56c97aadb1b191f26b851c398b88cb1d0e9bbd7 proposes Gemini
PROVIDER-008-B / #810 using I310 and ADR-094. Reserve neither ID for MODEL-014;
requery the target and open PRs before allocating I311/ADR-095 or later.
Gemini/auth implementation remains independently owned. The MODEL-014 adapter
matrix must cover current supported protocols and explicitly reject unsupported
protocol variants without pretending that this scope implements future adapters.

## Historical independent review findings and resolution

Reviewer identified four activation blockers: future fresh-probe precedence;
reasoning/replay/variant compatibility; explicit ADR-050 inference amendment;
concrete protocol parameter and output-cap mapping. Static only, no builds.

Revisions proposed for the first three:
- Conceptual precedence: explicit config > valid applicable fresh probe > catalog
  > unknown. No probe source exists today; no branch/schema/runtime claim of
  probe execution. MODEL-011 owns freshness and applicability before introduction.
- Explicit reasoning_supported=false suppresses effort, thinking budget and
  reasoning replay in effective runtime requests; it does not delete stored raw
  history or disable unrelated tool-call pairing. Request-capture tests must prove
  all three without changing persisted configuration.
- Omitted reasoning_supported with unknown identity preserves existing explicit
  reasoning options, including replay, as compatibility behavior; those options
  remain configuration, not inferred endpoint support. A selected valid variant
  continues to override configured effort as current materialization does;
  budget/replay retain their configured values unless explicit capability false.
- New ADR must expressly supersede ADR-050 sections 8/10's prohibition only for
  MODEL-014 unique canonical catalog identity matching; unknown/ambiguous remains
  Unknown. No fuzzy-name inference. Existing permission/protocol/content gates
  and explicit attachment action remain mandatory. This is a reviewed authority
  extension, not a claim that the old ADR already permits it.

Resolved by the source-verified adapter matrix below: exact effort/budget/replay,
output-cap mapping, precedence and unsupported-parameter diagnostics. Implementation
and request-capture validation remain outstanding, as does effective claim merge.
The independent reviewer subsequently accepted the completed design direction.
That review does not establish an effective claim or approve implementation.

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


## Current review and delivery state

Revised design direction received independent static approval on 2026-10-30.
The reviewer required reconciliation of the precedence summary and historical
unresolved-matrix statement; both are reconciled above. No builds were run by
the reviewer. The original MODEL-014 acceptance remains authoritative.
This proposal is not an ADR, activation record, implementation or completion.
Before activation, transpose this decision into an unused ADR, amend ADR-050's
applicable prohibition explicitly, preserve published iteration inventory and
obtain target-branch claim/activation review and merge.
