# MODEL-009-B: Capability Model, Content Types, And Persistence Foundation

| Field | Value |
| --- | --- |
| Story ID | MODEL-009-B |
| Type | Product / API / State Story |
| Priority | P2 |
| Status | Refinement — selected into I150 (2026-07-20) |
| Source | Maintainer requirement recorded 2026-07-20; child of MODEL-009 |
| Parent Epic | MODEL-009 |
| Depends on | MODEL-009-A (I149) ADR Accepted |
| Blocks | MODEL-009-C (I151), MODEL-009-D (I152) |

## Problem

MODEL-009-A produces the ADR, but the typed content parts, capability semantics, and persistence
boundary do not yet exist. Without them, I151 cannot safely read image bytes and I152 cannot
safely emit provider requests.

## Goal / Value

Establish the Talos-owned typed ordered content-part representation, the
`Supported` / `Unsupported` / `Unknown` capability semantics, and the ADR-selected attachment
metadata/storage policy — all without leaking provider wire format into core/session types and
without breaking existing text-only behavior.

## Scope

- Create Talos-owned typed ordered content parts in `talos-core` (or the ADR-selected crate).
  Provider JSON shapes remain confined to `talos-provider` adapters.
- Establish the capability semantics:
  - `Supported` — confirmed `image_input = true` for built-in catalog models.
  - `Unsupported` — confirmed `image_input = false` for built-in catalog models.
  - `Unknown` — custom/discovered models with no confirmed capability.
- `Unknown` and `Unsupported` both fail-closed for the user; the distinction must be usable for
  diagnostics (e.g., "configure capability metadata" vs "this model does not support images").
- Built-in catalog's confirmed `image_input` becomes `Supported`.
- Custom/discovered models default to `Unknown`.
- All existing text-only requests, history, resume, export, and copy maintain their current wire
  shape and behavior.
- Implement the ADR-selected attachment metadata/storage policy (path reference, byte copy, or
  metadata-only — whichever the ADR chose).
- Default rendering policy: image binary must not appear in terminal output, history text, copy,
  or export.
- Complete a public API semver impact inventory, migration notes, and a future-minor-release note.
- All new public API items must have `///` doc comments. No `#[allow(missing_docs)]` on public
  APIs.

## Explicit Exclusions

- Reading image bytes from disk (MODEL-009-C owns ingestion).
- Emitting protocol-native image requests (MODEL-009-D owns adapters).
- TUI attachment UX (MODEL-009-D owns TUI).
- Trusting remote price/capability metadata as authoritative.
- Inferring capability from model names.
- New `unsafe` blocks or native dependencies beyond what the I149 ADR approved.

## Design / Security Constraints

- Provider wire format must stay inside `talos-provider` adapters (ADR-013).
- The typed content parts must be serializable and round-trippable; serde + schemars per AGENTS.md
  Rust-Specific Rules.
- Persistence boundary must follow the I149 ADR's storage policy exactly.
- No binary content in terminal/history/copy/export by default.
- Pre-1.0 semver break analysis must be explicit; if a break is unavoidable, an ADR + migration
  plan must exist before the change lands.

## Acceptance

- Given a text-only message, when serialized and deserialized, then its wire shape and behavior
  are unchanged from pre-I150.
- Given a typed content part with ordered text and image parts, when serialized and deserialized,
  then ordering and content are preserved exactly.
- Given a built-in model with `image_input = true`, when its capability is queried, then it
  returns `Supported`.
- Given a built-in model with `image_input = false`, when its capability is queried, then it
  returns `Unsupported`.
- Given a custom/discovered model with no confirmed capability, when its capability is queried,
  then it returns `Unknown` and the attachment UI fails closed.
- Given a session containing an image attachment (per the ADR's storage policy), when resumed,
  exported, copied, or viewed in history, then the documented storage/privacy policy is followed
  and binary data is never dumped to terminal output by default.
- Given a downstream consumer that matches the public content-part enum exhaustively, when the
  public API is upgraded, then migration documentation identifies the new variants and release
  notes require handling or wildcard fallback.

## Required Reads

- `docs/backlog/active/MODEL-009-multimodal-image-input.md`
- `docs/backlog/active/MODEL-009-A-image-input-adr-and-security-spike.md`
- `docs/decisions/013-provider-config-schema-boundary.md`
- `docs/decisions/023-inline-api-key-boundary.md`
- `docs/decisions/050-<slug>.md` (the I149 ADR)
- `crates/talos-core/src/model.rs`
- `crates/talos-core/src/message.rs`
- `crates/talos-conversation/src/types.rs`
- `crates/talos-session/src/types.rs`

## Minimum Validation

- Typed content serde round-trip tests (text-only, text+image, image-only, empty, oversized).
- Text-only regression: existing session resume/export/copy/history fixtures pass unchanged.
- Capability provenance tests (built-in Supported, built-in Unsupported, custom/discovered
  Unknown, Unknown fail-closed for attachment).
- Session resume/export/copy/history tests for the ADR-selected attachment policy.
- Public API semver impact inventory + migration notes recorded in the iteration owner doc.
- Locked fmt/check/clippy/test and `scripts/validate_project_governance.sh .`; `git diff --check`.
