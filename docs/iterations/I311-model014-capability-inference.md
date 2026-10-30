# Iteration I311: Catalog-Assisted Custom-Model Capability And Variant Inference

> Document status: Active (proposed in #813; effective only on merge)
> Published plan date: 2026-10-30
> Planned objective: deliver full MODEL-014 and handoff T2+T3 using shared canonical identity, explicit overrides, provenance, protocol-safe requests and a real local walkthrough.
> Baseline rule: preserve this scope; a narrower compatibility patch cannot close it.
> MVP deliverable: a configured custom gateway model uniquely matching catalog metadata exposes inferred context/image/reasoning/output and eligible variants, with actual permission-gated image and reasoning requests and truthful source display.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 handoff closure session |
| Work Slice | MODEL-014 / handoff T2+T3: capability/limit/variant resolution, explicit-negative schema, config/runtime/provider/UI consistency, regression and local walkthrough; excludes new protocol/auth implementations and probing. |
| Claimed At | 2026-10-30 |
| Source Issue | None; maintainer full handoff request |
| Governance Claim PR | #813 |
| Authorization Mode | Independent review |
| Authorization Evidence | Finalized exact-head governance/API/security review and CI required before merge; no implementation before effective claim. |
| Implementation PR | Not started |
| Last Updated | 2026-10-30 |
| Handoff / Release Condition | Full MODEL-014 acceptance, merged implementation and owner evidence; no release/tag. |

## Published Baseline

Selected Story: MODEL-014. Preserve its original eight acceptance rows in full.
Dependencies: completed I308/T8, MODEL-013/I212 identity matching, MODEL-007 variant
selection, MODEL-009 image boundaries and ADR-095. MODEL-011 active probing stays
separate and unimplemented. No implicit endpoint-support inference.

### Scope and acceptance

1. Unique exact-ID/single-prefix catalog identity resolves context, image,
   reasoning and output independently with per-field inferred provenance.
2. Explicit overrides including false remain authoritative across catalog changes;
   context-only overrides do not disable output inference.
3. Unknown, ambiguous or missing fields remain unknown; no fabricated limits,
   capabilities, presets or endpoint evidence.
4. Inferred image true gates real user-requested attachment only with supported
   protocol, file permission and normal bounded safe-ingestion checks.
5. Reasoning capability, variant picker and actual requests agree. Explicit false
   suppresses effort/budget/replay; raw history and tool pairing remain intact.
6. Explicit output overrides inferred ceiling; consistent admission/factory mapping
   applies native protocol caps without changing endpoint/protocol authority.
7. Resolution performs no network probes and never persists derived fields during
   config save, switch, resume or runtime materialization.
8. Inference is labeled configured/inferred/unknown, never probe success; local
   custom-provider walkthrough demonstrates source display and real requests.

### Non-goals

No Gemini/Azure/Bedrock/auth implementation, fuzzy matching, new catalog, pricing,
roles/routing inference, paid probes, release/tag, arbitrary request JSON, removal
of raw history or reopening completed I212/variant/image baselines.

### Validation and documentation

Focused config resolver, explicit-negative, ambiguity, image-eligibility,
variant/materialization and provider request-capture tests (including catalog
reasoning=false plus legacy options and explicit-true override); live local fixture
walkthrough through Talos. Pinned Rust 1.99.0, locked workspace tests/check/Clippy,
standard release_preflight; both governance validators and independent security/API
review. Exact-head CI and merge-time CAS before completion.
Update README en/zh, custom provider/model and selection docs, SDK migration notes,
ADR-050 amendment, MODEL-014 owner, indexes, Board and full handoff checkpoint.

### Inventory and non-overlap

Inventory re-read on main `85aa2d0f4` on 2026-10-30: I293/I294 retain independent search work; I277 retains
Review/deferred device-human acceptance; I302 retains Review diagnostics; I249
remains Planned with original dependency pilot objective; I164 remains Paused and
superseded layout belongs to I165. #809 implementation and #812 owner closure are merged; I308 is Complete/Closed. #811 proposes I310/ADR-094 Gemini and is a distinct scope.
No ownership takeover or parallel implementation inferred from this inventory.

### Risks and rollback

Named-model catalog evidence may differ from a gateway. Visible provenance,
explicit false overrides and protocol-specific validation remain mandatory.
Public Rust literal compatibility requires ADR migration; existing TOML loads.
Rollback restores explicit-only custom capabilities without deleting raw history
or mutating user config. Do not downgrade/retry through another wire protocol.

## Execution and completion evidence

Proposed Active/Claimed state in #813 is ineffective before merge; no implementation
or test evidence yet. Final claim/activation
must reach main before implementation starts. Completion requires an existing
merged implementation SHA and all acceptance above; full handoff remains Active.
