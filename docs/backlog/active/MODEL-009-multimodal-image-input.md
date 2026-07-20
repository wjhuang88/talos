# MODEL-009: Multimodal Image Input

| Field | Value |
| --- | --- |
| Story ID | MODEL-009 |
| Type | Product / API / State Story (Epic) |
| Priority | P2 |
| Status | Refinement — split into child Stories MODEL-009-A/B/C/D on 2026-07-20 |
| Source | Maintainer requirement recorded 2026-07-20 |
| Depends on | Catalog capability metadata; provider protocol boundary; session persistence boundary |
| Blocks | — |
| Child Stories | [MODEL-009-A](MODEL-009-A-image-input-adr-and-security-spike.md) (I149 ADR + security spike) · [MODEL-009-B](MODEL-009-B-capability-content-types-persistence.md) (I150 capability + content + persistence) · [MODEL-009-C](MODEL-009-C-safe-local-image-ingestion.md) (I151 safe ingestion) · [MODEL-009-D](MODEL-009-D-provider-adapter-tui-cli.md) (I152 adapter + TUI/CLI) |

## Problem

Talos knows whether a catalog model advertises `image_input`, but this is metadata only. User and
conversation messages are text-only strings, the TUI has no attachment affordance, and the
OpenAI-compatible and Anthropic-compatible request adapters cannot emit image content blocks. A
model marked as vision-capable is therefore indistinguishable from a text-only model in practice.

## Goal / Value

Allow a user to attach a local image to a message when, and only when, the selected model and
provider protocol support image input. The model receives the image through its protocol-native
request shape, while the user can understand the attachment state and the session retains a safe,
portable record of what was sent.

## Scope

- Introduce a Talos-owned, typed multimodal message-content representation that can carry ordered
  text and local-image parts without encoding provider JSON in core/session types.
- Provide a TUI attachment flow for an explicit local file path, including attach, list/preview
  metadata, remove, and cancel before message submission. Non-interactive CLI behavior must have
  an equivalent explicit argument or documented rejection.
- Gate attachment affordances and submission by the active model's `image_input` capability.
  Unknown capability for manually configured/discovered custom models must fail closed for the
  attachment UI, with a clear configuration/discovery diagnostic rather than silently sending an
  unsupported request.
- Validate canonical local paths, regular-file type, supported MIME/image format, file and pixel
  limits, and aggregate request limits before reading bytes. The exact supported formats and bounds
  require ADR-backed selection.
- Send image parts through protocol-specific adapters for the existing `openai-chat` and
  `anthropic-messages` protocols. Preserve text-only requests and all existing provider behavior.
- Persist enough non-secret attachment metadata and replay-safe content representation for session
  resume. Define whether image bytes are retained, copied into Talos storage, or referenced by
  path; the choice must explicitly address deletion, relocation, privacy, export, and portability.
- Render an attachment summary in live and historical TUI surfaces without rendering raw binary
  data or leaking local paths where the existing privacy/display boundary forbids it.
- Update README, configuration reference, public documentation site, and any model-capability UI
  descriptions to distinguish "catalog advertises image input" from "image attachment is usable".

## Explicit Exclusions

- Audio, video, PDF/document understanding, screen capture, clipboard image extraction, image
  generation, OCR as a separate service, and remote image URL fetching.
- Inferring image capability from a model name, probing a provider with arbitrary image requests,
  or treating a successful `/models` response as authoritative multimodal capability evidence.
- Dynamic protocols, arbitrary request JSON, additional provider adapters, OAuth changes, or a
  provider credential-store redesign.
- Automatic attachment of workspace files or images; every attachment is an explicit user action.

## Decision Links And Constraints

- ADR-013 governs provider configuration/protocol boundaries; image request mapping must remain
  adapter-owned and cannot leak provider wire formats into `talos-core`.
- ADR-023 governs credential and display secrecy. Attachment diagnostics, persistence, export, and
  debug output must not expose credentials; file paths and image-derived metadata need an explicit
  privacy policy before implementation.
- The current public message and UI output types are semver-bound. Any breaking replacement of
  text-only content requires an ADR, migration plan, and an appropriate pre-1.0 minor release.
- Local file reading is a security-sensitive boundary. The selected design must reuse or extend
  normal workspace/path authorization rather than bypassing it because a model is vision-capable.
- Image decoding/parsing dependencies may panic or invoke native code. Per project hard constraint
  #9, each integration boundary needs bounded input, `catch_unwind` where applicable, and a safe
  error path. New dependencies require an ADR-backed review.

## Uncertainty And Validation Path

This item is not Ready. Before selecting implementation, create an ADR that decides:

1. Content-part schema and public API migration strategy.
2. Attachment storage/replay/export model and privacy/deletion behavior.
3. Authorization semantics for local image reads, including outside-workspace paths and symlinks.
4. Supported formats, byte/pixel/count limits, MIME verification, and safe decoder strategy.
5. OpenAI-compatible and Anthropic-compatible wire mappings, including data URL versus uploaded
   media handling and provider-specific limits.
6. Capability provenance for built-in, imported, and custom-provider models; unknown must remain
   distinguishable from false.

## Acceptance

- Given an active model whose confirmed capability is `image_input = true`, when the user attaches
  a valid local image and sends a message, then the selected provider adapter receives ordered text
  and image parts in its protocol-native request format.
- Given a text-only or capability-unknown model, when the user attempts to attach an image, then
  Talos rejects the action before file bytes are read and explains why.
- Given an invalid path, non-regular file, unsupported/invalid image, oversized image, excessive
  pixel count, aggregate-limit breach, permission denial, decoder failure, or provider rejection,
  when the attachment is validated or sent, then no panic occurs, the composer remains usable, and
  no partial message/session mutation is committed.
- Given multiple ordered text and image parts, when the request is sent through each supported
  protocol, then fixtures prove ordering and request shape, without live credentials or network.
- Given a session containing an image attachment, when it is resumed, exported, copied, or viewed
  in history, then its documented storage/privacy policy is followed and binary data is never
  dumped to terminal output by default.
- Given a text-only conversation, when it is sent, resumed, or exported, then existing wire shape,
  behavior, and fixtures remain unchanged.

## Required Reads

- `docs/decisions/013-provider-config-schema-boundary.md`
- `docs/decisions/023-inline-api-key-boundary.md`
- `docs/backlog/active/MODEL-008-interactive-custom-provider-registration.md`
- `crates/talos-core/src/model.rs`
- `crates/talos-core/src/message.rs`
- `crates/talos-conversation/src/types.rs`
- `crates/talos-provider/src/openai.rs`
- `crates/talos-provider/src/anthropic_request.rs`
- `crates/talos-tui/src/state.rs`
- `crates/talos-tui/src/app.rs`

## Minimum Validation

- Unit/property tests for typed content serialization, UTF-8/path validation, format/size/pixel
  bounds, and invalid/corrupt fixtures.
- Mock request fixtures for OpenAI-compatible and Anthropic-compatible image requests, including
  text-only regression coverage.
- TUI/CLI tests for capability gating, attach/remove/cancel, error recovery, and no accidental
  bytes read for unsupported/unknown models.
- Session resume/export/copy/history tests for the selected attachment persistence policy.
- Dependency panic-containment and permission-boundary tests for file/decode integrations.
- `cargo fmt --all -- --check`, `cargo check --workspace --locked`, `cargo clippy --workspace
  --locked -- -D warnings`, `cargo test --workspace --locked`, and governance validation.
