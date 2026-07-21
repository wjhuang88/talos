# 050: Multimodal Image Input Architecture And Security Boundary

> Status: Accepted (amended 2026-07-21 for P1-A/P1-B rework)
> Date: 2026-07-20
> Iteration: I149 (MODEL-009-A)
> Gate: This ADR is the hard gate for I150-I152. If any decision point is unresolved, I150-I152 are Blocked.

## Context

MODEL-009 needs a safe image input capability, but the architecture/security decisions were not
previously made. Implementing image sending before deciding content-part schema, persistence, path
authorization, format limits, decoder panic containment, wire mapping, and capability provenance
would lock in unsafe defaults.

## Constraint Decomposition

| Constraint | Type | Source | Decision impact |
| --- | --- | --- | --- |
| No provider wire format in core/session types | Hard | ADR-013 | Content parts must be Talos-owned, not provider JSON. |
| Credentials masked in all display surfaces | Hard | ADR-023 | Attachment diagnostics, persistence, export must not expose keys. |
| External path authorization | Hard | SEC-001/ADR-047 | Image path reads must reuse the existing permission pipeline. |
| Native/panic boundary must degrade gracefully | Hard | AGENTS.md #9 | Decoder boundaries need `catch_unwind` + size limits. |
| Public APIs are semver-bound | Hard | AGENTS.md #6 | New content-part types require migration notes. |
| No speculative features | Hard | AGENTS.md #7 | Only implement what I150-I152 scope defines. |
| `unsafe` requires ADR | Hard | AGENTS.md #2 | No new `unsafe` in image input code. |
| Bounded viewport and memory | Hard | AGENTS.md Simplicity | Projection must have explicit entry and byte bounds. |
| Final history remains scrollback-only | Hard | ADR-035 | Image binary never enters scrollback. |

## Reasoning

The simplest approach satisfying all Hard constraints is:
1. Define a Talos-owned typed content-part enum in `talos-core` that can carry ordered text and
   local-image parts without encoding provider JSON.
2. Keep all provider-specific wire format (data URL, base64, uploaded media) inside
   `talos-provider` adapters.
3. Reuse SEC-001/ADR-047 for local path authorization — no new bypass.
4. Use the `image` crate (MIT/Apache-2.0, pure Rust) for format detection and pixel-bound
   validation. No native/C dependencies. `catch_unwind` at the decode boundary.
5. Reference images by path in the TLOG (not embedded bytes). The path is canonicalized and
   validated at attach time; on resume, the path is re-validated. If the file is missing or
   changed, the attachment is shown as a safe summary with a diagnostic, not a panic.
6. Default custom/discovered models to `Unknown` capability — fail-closed for attachment UI.

## Decision

### 1. Ordered text/image content-part schema

A new `talos_core::message::ContentPart` enum:

```rust
pub enum ContentPart {
    Text(String),
    Image { path: PathBuf, mime: String, byte_count: u64 },
}
```

Messages carry `Vec<ContentPart>` in order. The existing `Message::User { content: String }` type
is preserved for text-only backward compatibility; a new `Message::Multimodal { parts: Vec<ContentPart> }`
variant is added additively. Provider adapters convert `ContentPart::Image` to protocol-native
wire format (data URL for OpenAI, base64 source for Anthropic).

### 2. Pre-1.0 semver migration strategy

- `Message::Multimodal` is a new public enum variant — exhaustive matches must add an arm or
  wildcard fallback. Release must be a minor bump.
- `ContentPart` is a new public type — no migration needed for consumers that don't use it.
- The existing `Message::User { content: String }` path is unchanged.
- Migration note: "If you match `Message` exhaustively, add a `Message::Multimodal { .. }` arm or
  wildcard fallback."

### 3. Image storage, session resume, export, copy, deletion/move behavior

- **Storage policy**: Path reference, not embedded bytes. The TLOG stores the canonical path,
  MIME type, and byte count. No image bytes are persisted in the session log.
- **Session resume**: The path is re-validated on resume. If the file is missing, moved, or
  changed (different size/MIME), the attachment renders as a safe summary with a diagnostic
  ("Image attachment unavailable: file not found / moved / changed"). No panic.
- **Export**: The exported transcript shows `[Image: {filename} ({byte_count} bytes, {mime})]` —
  no binary data in the export file.
- **Copy**: Clipboard copy shows the same safe summary. No binary in clipboard.
- **Deletion/move**: After the original file is deleted or moved, the session history shows the
  safe summary. No error is raised — the user is informed.

### 4. Local path authorization, external path, and symlink strategy

- Reuse SEC-001/ADR-047: the permission pipeline must approve the external path before any image
  byte is read. No bypass because the model is vision-capable.
- **P1-A (2026-07-21 amendment)**: a shared `image_authorization` module in `talos-cli`
  evaluates every `/attach` and `--attach` path against `PermissionEngine` with a synthetic
  `attach_image` tool name and `ToolNature::Read`. Workspace-internal paths auto-allow; external
  paths produce `Ask`, which the TUI resolves through `UiOutput::ToolApprovalRequest` and print
  mode treats as fail-closed. User-approved external paths can be added as runtime allow rules
  scoped to the exact path string.
- **Canonicalization**: `std::fs::canonicalize` at attach time. The canonical path is stored.
- **Regular file**: reject directories, FIFOs, character/block devices, sockets.
- **Symlink**: canonicalize the symlink target. On execution (actual file read), re-canonicalize
  and compare — if the target changed between grant and read, reject (fail-closed).
- **P1-B (2026-07-21 amendment)**: the TOCTOU path guard alone is insufficient against
  same-path file replacement (atomic swap at the same canonical path). `ContentPart::Image`
  now carries a `ContentDigest` (SHA-256, `[u8; 32]`) computed at grant time. The provider
  adapter recomputes the digest at read time and omits the part on mismatch. The all-zero
  `ContentDigest::default()` sentinel means "verification intentionally skipped" and is only
  used by test fixtures.
- **External path**: requires explicit interactive approval (same as SEC-001). Headless mode
  fails closed.

### 5. Supported formats, MIME/magic-byte verification, limits

- **Supported formats**: PNG, JPEG, GIF, WebP. These are the most common image formats supported
  by OpenAI and Anthropic vision models.
- **MIME verification**: extension is not trusted. Magic bytes are checked against known
  signatures:
  - PNG: `\x89PNG\r\n\x1a\n`
  - JPEG: `\xff\xd8\xff`
  - GIF: `GIF87a` or `GIF89a`
  - WebP: `RIFF....WEBP` (offset 0 + offset 8)
- **Single image byte limit**: 20 MiB (20_971_520 bytes). This is the OpenAI limit; Anthropic
  allows up to 5 MiB per image but we enforce the larger limit at the Talos layer and let the
  provider reject if needed.
- **Total byte limit**: 50 MiB across all attachments in one message.
- **Pixel limit**: 89,478,485 pixels (the OpenAI limit). Images exceeding this are rejected
  before the full decode completes.
- **Count limit**: 4 images per message (OpenAI limit). Anthropic allows up to 100, but we
  enforce the smaller limit.

### 6. Decoder dependency, license, security review, panic containment

- **Decoder**: the `image` crate (MIT OR Apache-2.0, pure Rust, no native deps). It provides
  format detection, dimension reading, and pixel counting without full decode for most formats.
- **License**: compatible with Talos's Apache-2.0 license.
- **Security review**: the `image` crate is widely used, has a security policy, and is not a
  native/C dependency. It does not invoke system processes. It is behind the ADR-008 no-C-bindings
  exception boundary (pure Rust, no `unsafe` in the public API).
- **Panic containment**: every `image` crate call is wrapped in `std::panic::catch_unwind`. If
  the decoder panics on a malicious input (pixel bomb, corrupt header, etc.), the panic is caught,
  an error is returned, and the composer remains usable. No process exit.
- **Size limits**: before any decode, the file size is checked against the byte limit. For pixel
  limit, `image::ImageReader::with_guessed_format` + `into_dimensions()` is used to read the
  dimensions without full decode. If `into_dimensions()` would require a full decode (which can
  happen for some formats), the byte limit is the primary guard.
- **No new `unsafe`**: the `image` crate is pure Rust. No `unsafe` blocks are added to Talos
  source code.

### 7. OpenAI-compatible and Anthropic-compatible wire mapping

- **OpenAI-compatible**: `ContentPart::Image` is mapped to the OpenAI `image_url` content type
  with a data URL: `data:{mime};base64,{base64_encoded_bytes}`. The adapter reads the file bytes,
  base64-encodes them, and constructs the request JSON inside `talos-provider/src/openai_request.rs`.
- **Anthropic-compatible**: `ContentPart::Image` is mapped to the Anthropic `image` content block
  with a base64 source: `{ "type": "image", "source": { "type": "base64", "media_type": "{mime}",
  "data": "{base64_encoded_bytes}" } }`. The adapter constructs this inside
  `talos-provider/src/anthropic_request.rs`.
- **Provider JSON stays in adapters**: the `ContentPart::Image` type in `talos-core` carries only
  the path, MIME, and byte count. The actual file reading and base64 encoding happen in the
  provider adapters at request time.
- **No remote URL images**: both protocols support remote URLs, but Talos explicitly rejects
  remote URL images. Only local file paths are accepted.

### 8. Capability provenance for built-in, imported, and custom-provider models

- **Built-in catalog**: `image_input` field in `ModelMetadata` (already exists in `models.toml`).
  If `image_input = true` → `Supported`. If `image_input = false` → `Unsupported`.
- **Imported (opencode/models.dev)**: same `image_input` field from the catalog. Same mapping.
- **Custom/discovered models**: no `image_input` metadata is available from the `/models`
  endpoint (OpenAI and Anthropic `/models` responses do not include vision capability). Custom
  and discovered models default to `Unknown`.
- **No name-based inference**: the capability is never inferred from a model name (e.g., "gpt-4o"
  does not imply image support). Only explicit catalog metadata or explicit user configuration
  can set `Supported`.

### 9. `Supported`, `Unsupported`, and `Unknown` distinction

```rust
pub enum ImageInputCapability {
    Supported,
    Unsupported,
    Unknown,
}
```

- `Supported`: the model's catalog metadata confirms `image_input = true`. The attachment UI
  allows attaching images.
- `Unsupported`: the model's catalog metadata confirms `image_input = false`. The attachment UI
  rejects image attachment with a clear message ("This model does not support image input").
- `Unknown`: the model's capability is not confirmed (custom/discovered). The attachment UI
  rejects image attachment with a clear message ("Image input capability is unknown for this
  model. Configure the model's `image_input` metadata to enable image attachment.").

Both `Unknown` and `Unsupported` fail-closed for the user — no image is read or sent. The
distinction is diagnostic: `Unsupported` means "this model definitely does not support images",
while `Unknown` means "we don't know whether this model supports images". The user can resolve
`Unknown` by adding `image_input = true` to the model's config.

### 10. Custom/discovered models default `Unknown`

- Custom providers registered via MODEL-008-A/I147 and models discovered via
  MODEL-008-B/I148 default to `Unknown` for `image_input`.
- The user can override `Unknown` to `Supported` by adding `image_input = true` to the model's
  config in `[providers.<name>.models.<model>]`.
- No probing: Talos never sends an arbitrary image request to test whether the model supports
  images. The capability must be explicitly configured.
- No name-based inference: model names like "gpt-4o" or "claude-sonnet" do not imply image
  support.

## Rejected Alternatives

- **Embed image bytes in TLOG**: rejected on storage, privacy, and portability grounds. The TLOG
  would balloon with image bytes; export/copy would dump binary; moving/deleting the original file
  would leave stale bytes.
- **Remote URL images**: rejected for security and privacy. Fetching remote URLs would require
  network permission and could leak the user's IP to arbitrary image hosts.
- **Infer capability from model name**: rejected as fragile and unreliable. Model names change;
  capability metadata is authoritative.
- **Probe provider with image request**: rejected as wasteful, potentially harmful (unwanted
  image sent to provider), and unreliable (provider may accept the request but not support
  vision).
- **OS keychain for image path authorization**: rejected as out of scope. The existing
  SEC-001/ADR-047 path authorization is sufficient.
- **New native/C decoder (libpng, libjpeg)**: rejected per AGENTS.md Hard Constraint #1. The pure
  Rust `image` crate is sufficient and avoids the native dependency boundary.

## Consequences

- `talos_core::message` gains `ContentPart` enum and `Message::Multimodal` variant (additive,
  pre-1.0 semver break for exhaustive matches).
- `talos_core::model` gains `ImageInputCapability` enum and a method to resolve it from
  `ModelMetadata`.
- `talos-provider/src/openai_request.rs` and `anthropic_request.rs` gain image content mapping.
- `talos-tui` gains attachment UX (attach, list, remove, cancel, capability gating).
- `talos-cli` gains image path validation (canonicalization, MIME, magic-byte, size, pixel,
  count checks).
- The `image` crate is added as a workspace dependency (MIT/Apache-2.0, pure Rust).
- No new `unsafe` blocks.
- No new native/C dependencies.
- No remote URL image fetching.
- No audio, video, PDF, screenshot, clipboard, or image generation.

## Reversal Trigger

Revisit if:
- A provider requires image content format that cannot be expressed as a data URL or base64
  source (e.g., uploaded media with a separate upload API).
- The `image` crate is found to have unacceptable security or performance characteristics.
- A product requirement emerges for remote URL images, audio, video, or other multimodal types.
- The path-reference storage policy proves insufficient for session resume use cases (e.g., users
  need to resume sessions on a different machine where the image file doesn't exist).

## Related

- ADR-013: Provider Config Schema Boundary
- ADR-023: Inline API Key Storage and Display Boundary
- ADR-035: TUI Conversation History Scrollback Boundary
- ADR-047: External-Path Tool Authorization
- SEC-001: External-Path File Authorization Gap
- MODEL-009: Multimodal Image Input (parent story)
- MODEL-009-A: Image Input ADR And Security Spike (this iteration)
