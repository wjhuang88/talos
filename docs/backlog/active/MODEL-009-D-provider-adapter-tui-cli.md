# MODEL-009-D: Provider Adapter And TUI/CLI Interaction

| Field | Value |
| --- | --- |
| Story ID | MODEL-009-D |
| Type | Product / API / UX Story |
| Priority | P2 |
| Status | Refinement — selected into I152 (2026-07-20) |
| Source | Maintainer requirement recorded 2026-07-20; child of MODEL-009 |
| Parent Epic | MODEL-009 |
| Depends on | MODEL-009-C (I151) safe local image ingestion |
| Blocks | — |

## Problem

MODEL-009-C can safely stage a validated image attachment, but the OpenAI-compatible and
Anthropic-compatible adapters cannot emit image content blocks, and the TUI/CLI have no
attachment affordance. A validated image has nowhere to go.

## Goal / Value

Wire the validated attachment through the two existing protocol adapters as protocol-native image
request content, provide a complete and cancellable TUI attachment UX, provide an equivalent CLI
argument or a documented safe rejection, and render safe summaries in history/resume/copy/export
per the I149 ADR — all without breaking text-only behavior.

## Scope

- OpenAI-compatible adapter generates protocol-native image request content.
- Anthropic-compatible adapter generates protocol-native image request content.
- Fixtures prove multi-part text/image ordering and exact request shape for both protocols.
- TUI supports:
  - Explicit local path attachment.
  - Attachment list/summary.
  - Remove.
  - Cancel.
  - Pre-send visibility (the user can see what is attached before pressing send).
  - `Unsupported` and `Unknown` capability early rejection (before any file byte is read).
- CLI must have an equivalent explicit argument, or a clear, safe rejection with a documented
  pointer to the TUI path.
- For `Unsupported` and `Unknown`, reject before any file byte is read.
- History, resume, copy, and export render the ADR-selected safe summary.
- No raw binary in terminal/history/copy/export.
- No unconditional full local path exposure.
- Preserve all text-only behavior and provider fixtures.

## Explicit Exclusions

- Remote URL image fetching.
- Audio, video, PDF, screenshot, clipboard, or image generation.
- New `unsafe` blocks or native dependencies beyond what the I149 ADR approved.
- Inferring capability from model names.
- Changing provider protocols, OAuth, arbitrary JSON/headers, or new transport code.
- Editing, deleting, or reordering attachments after they are sent (the message is immutable
  once submitted).

## Design / Security Constraints

- Provider JSON stays inside `talos-provider` adapters (ADR-013).
- Credentials stay masked in all surfaces (ADR-023).
- Path and binary content stay masked in all surfaces (ADR-023 extension + I149 ADR).
- Capability gating must reject `Unsupported`/`Unknown` before any file byte is read.
- The TUI viewport contract (composer minimum one row, narrow-height collapse) must not be
  broken by the attachment affordance.

## Acceptance

- Given an active model whose confirmed capability is `Supported`, when the user attaches a valid
  local image and sends a message, then the selected provider adapter receives ordered text and
  image parts in its protocol-native request format.
- Given a text-only or `Unknown` or `Unsupported` model, when the user attempts to attach an
  image, then Talos rejects the action before file bytes are read and explains why.
- Given multiple ordered text and image parts, when the request is sent through each supported
  protocol, then fixtures prove ordering and request shape, without live credentials or network.
- Given a session containing an image attachment, when resumed, exported, copied, or viewed in
  history, then the ADR-selected safe summary is rendered and binary data is never dumped to
  terminal output by default.
- Given a text-only conversation, when sent, resumed, or exported, then existing wire shape,
  behavior, and fixtures remain unchanged.
- Given the TUI with an attachment, when the user removes or cancels, then the composer returns
  to its pre-attachment state with no partial attachment retained.
- Given the CLI, when the user passes an image argument (if implemented) or attempts to pass one
  (if rejected), then the CLI either sends the image through the same adapter path or prints a
  documented safe rejection pointing to the TUI path.
- Given README EN/zh-CN and site documentation, when users read image attachment instructions,
  then they describe the explicit path argument, the capability gating, and the safe summary
  behavior.

## Required Reads

- `docs/backlog/active/MODEL-009-multimodal-image-input.md`
- `docs/backlog/active/MODEL-009-A-image-input-adr-and-security-spike.md`
- `docs/backlog/active/MODEL-009-B-capability-content-types-persistence.md`
- `docs/backlog/active/MODEL-009-C-safe-local-image-ingestion.md`
- `docs/decisions/013-provider-config-schema-boundary.md`
- `docs/decisions/023-inline-api-key-boundary.md`
- `docs/decisions/050-<slug>.md` (the I149 ADR)
- `crates/talos-provider/src/openai.rs`
- `crates/talos-provider/src/openai_request.rs`
- `crates/talos-provider/src/anthropic_request.rs`
- `crates/talos-tui/src/state.rs`
- `crates/talos-tui/src/app.rs`
- `crates/talos-cli/src/main.rs`

## Minimum Validation

- Two-protocol image request fixtures (OpenAI-compatible and Anthropic-compatible) proving
  ordered text/image request shape.
- Text-only full regression: existing provider fixtures and session tests pass unchanged.
- TUI state/app/Buffer render tests for attach/list/remove/cancel/pre-send-visibility/capability
  rejection.
- CLI parameter or rejection-path tests.
- attach/remove/cancel/error-recovery integration tests.
- history/resume/export/copy safe-summary rendering tests.
- Locked fmt/check/clippy/test and `scripts/validate_project_governance.sh .`; `git diff --check`.
