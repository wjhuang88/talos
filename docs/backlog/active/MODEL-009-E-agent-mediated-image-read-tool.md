# MODEL-009-E: Agent-Mediated Image Read Tool

| Field | Value |
| --- | --- |
| Story ID | MODEL-009-E |
| Type | Product / Tool / Security Story |
| Priority | P2 |
| Status | Active — P3 implementation in progress (2026-07-23); commits `6d4677e`–`bacf292` pushed; all automated gates green |
| Source | Maintainer requirement, 2026-07-21 |
| Parent Epic | MODEL-009 |
| Depends on | MODEL-009-C safe image ingestion; MODEL-009-D attachment capability gate and provider pipeline; SEC-001; ADR-050; ADR-051 |
| Blocks | — |

## Problem

Writing an image path in normal conversation text must not itself read a local file. Users may want a vision-capable model to inspect a named workspace image without first using a composer attachment command. The existing text `read` tool cannot safely return binary content or provider-specific image JSON.

## Goal / Value

Provide a separate, agent-invoked `read_image` tool. A model may request an explicit local image path; Talos then applies the same capability, permission, validation, and provenance controls as user attachments and supplies a Talos-owned image artifact to the following provider request. This never treats a textual path as authorization to read a file.

## Scope

- Add a distinct `read_image` tool. Preserve the text-only contract and behavior of `read`.
- Expose the tool only when the active model has confirmed `ImageInputCapability::Supported`. `Unknown` and `Unsupported` models neither receive the tool schema nor read image bytes.
- Tool input is one explicit local path; the normal permission pipeline grants or denies the exact canonical path.
- Reuse MODEL-009-C validation from a shared `talos-tools` boundary: regular-file checks, canonicalization, SEC-001 authorization, symlink/TOCTOU revalidation at read, format/MIME/magic-byte limits, byte/pixel/count bounds, and panic-contained decoding.
- Represent success as a Talos-owned internal image artifact/content part. Never put base64 or binary content in a text tool result, terminal transcript, export, or debug output.
- After a successful tool read, only the next provider request receives the image through ADR-051's non-persistent overlay and the existing OpenAI-compatible / Anthropic-compatible mappings; history retains only a safe tool/result summary.
- TUI and CLI render only basename, media type, byte count, result status, and tool provenance.

## Explicit Exclusions

- Automatically reading every path in a user message.
- Changing `read` to return binary or multimodal payloads.
- Remote URLs, user data URLs, clipboard/screenshot capture, OCR, audio, video, PDF, image generation, protocol expansion, OAuth, or custom request JSON/headers.
- Persisting raw image bytes or full paths in display/export surfaces.

## Design And Security Constraints

- This is a new tool capability, not a composer shorthand. It must use `ToolRegistry`, tool presentation policy, and the normal permission pipeline.
- The `talos-tools` result stays provider-neutral. Provider wire mapping remains in `talos-provider`; lifecycle ownership remains in agent/conversation/session layers.
- A tool call cannot bypass MODEL-009-D capability gating, SEC-001, or safe ingestion.
- All native or panic-prone integration calls follow AGENTS.md hard constraint #9.
- If the required continuation cannot be represented safely in both provider protocols, stop and amend ADR-050 rather than serializing binary as text.

## Acceptance

- Given a Supported active model and a user request naming a local image, when the model selects `read_image`, then Talos requests permission for the exact canonical path and, after approval, sends the image only in the next protocol-native provider request.
- Given an Unknown or Unsupported active model, when tools are presented, then `read_image` is absent and no image file is read.
- Given denial, headless Ask, external path without approval, invalid image, symlink retarget, oversized/pixel-bomb image, decoder error, or provider failure, when the tool is invoked, then no binary/path leak or partial image artifact reaches the provider and the turn degrades safely.
- Given a successful tool read, when history, resume, copy, export, or TUI display is rendered, then only the documented safe summary appears and tool provenance remains visible.
- Given ordinary text `read` requests and text-only model turns, when this story is enabled, then existing behavior and request shapes remain unchanged.

## Required Reads

- `docs/backlog/active/MODEL-009-C-safe-local-image-ingestion.md`
- `docs/backlog/active/MODEL-009-D-provider-adapter-tui-cli.md`
- `docs/backlog/active/SEC-001-external-path-authorization.md`
- `docs/decisions/047-external-path-tool-authorization.md`
- `docs/decisions/050-multimodal-image-input-architecture.md`
- `docs/decisions/051-one-shot-multimodal-tool-continuation.md`
- `docs/decisions/006-event-architecture-boundary.md`
- `crates/talos-tools/src/`, `crates/talos-agent/src/`, and `crates/talos-provider/src/`

## Minimum Validation

- Registry/presentation tests proving `read_image` is visible only to Supported models.
- Permission integration tests for workspace, external-path approval, denial, and headless Ask.
- Adversarial validation tests reused from MODEL-009-C, including symlink retarget and pixel bomb.
- Agent/session integration tests proving the artifact appears in exactly the following provider request, not in a text tool result or unrelated turn.
- OpenAI-compatible and Anthropic-compatible fixtures, text-only regressions, history/resume/export safe-summary tests, and TUI provenance rendering tests.
- Locked fmt/check/clippy/test, governance validation, and `git diff --check`.
