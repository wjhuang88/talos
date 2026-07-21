# I149 / MODEL-009-A Security Review

**Date**: 2026-07-20 (originally drafted), 2026-07-21 (R3–R6 rework applied)
**Status**: Accepted (with documented residuals)
**Decision**: ADR-050
**Rework**: R3–R6 from the Owner acceptance feedback (NO-GO 2026-07-21)

## Scope Of This Review

This review covers the new dependency boundary, file-reading boundary,
decoder boundary, capability boundary, and persistence/privacy boundary
introduced by MODEL-009 (multimodal image input). The original 2026-07-20
draft overstated several controls; this revision reflects what the code
actually enforces as of the R3–R6 rework and explicitly lists residual
gaps that the Owner acceptance found.

## New Dependency: `image` crate

| Property | Value |
|---|---|
| Crate | `image` (image-rs/image on crates.io), pinned to `0.25` via Cargo.lock |
| License | MIT OR Apache-2.0 (compatible with Talos Apache-2.0) |
| Native/C bindings | None — pure Rust |
| `unsafe` in public API | No |
| Process spawning | No |
| Security policy | Yes (image-rs/image has a security policy on GitHub) |
| Panic surface | Decoder panics on malformed input (mitigated by `catch_unwind`) |
| Size | Moderate — pulls in `png`, `jpeg`, `gif`, `webp` decoders |
| Features enabled | `default-features = false`, explicit `["png", "jpeg", "gif", "webp"]` |

### Verdict

The `image` crate is acceptable under AGENTS.md Hard Constraint #1 (no arbitrary
C/C++ bindings) because it is pure Rust with no native dependencies. It is
acceptable under Hard Constraint #9 (external dependencies must not crash the
process) because every decoder call is wrapped in `std::panic::catch_unwind`
with bounded input.

## Capability Boundary (R3 — fail-closed before file read)

The TUI bridge consults `ImageInputCapability` for the active model before
any filesystem access in the `/attach` flow. Models whose catalog metadata
does not confirm `image_input = true` are rejected with an error message and
no file probe occurs. The capability is resolved from the merged catalog
(builtins + Config user models) by `resolve_model_info`, propagated to the
engine via `model_info_watch`, and stored on the engine as
`image_input_capability`. The default is `Unknown` and `Unknown` fails
closed.

| State | Behaviour |
|---|---|
| `Supported` (catalog metadata `image_input = true`) | `/attach` proceeds to validation |
| `Unsupported` (catalog metadata `image_input = false`) | `/attach` rejected before fs access |
| `Unknown` (no metadata available, e.g. custom/discovered model) | `/attach` rejected before fs access |

### Verdict

The fail-closed default is enforced. Three regression tests in
`talos-cli/src/tests.rs` cover the `Unknown`, `Unsupported`, and
`Supported` branches. There is no path from the TUI to file I/O that
bypasses the capability check.

## File-Reading Boundary

| Threat | Control | Status |
|---|---|---|
| Directory traversal | `canonicalize` + regular-file check | Implemented |
| FIFO / non-regular file | `std::fs::metadata` file type check | Implemented |
| Symlink retarget / TOCTOU at provider read | `image_io::read_image_with_toctou_guard`: re-canonicalize, byte-compare to stored canonical, omit on mismatch | Implemented (R5) |
| Path traversal (`..`) | `canonicalize` resolves to absolute canonical path | Implemented |
| External path without SEC-001 approval | **Not implemented.** `validate_image_path` canonicalizes but does not invoke the SEC-001/ADR-047 permission pipeline. The capability check is the only gating layer. | **Residual** |
| Headless accidental access | Unresolved Ask fails closed (preserved by existing permission layer, not by this feature) | Preserved |
| File too large | Byte limit check before full read — 20 MiB single image, 50 MiB total | Implemented |
| Pixel bomb (small bytes, huge dimensions) | `image::ImageReader::into_dimensions()` reads format headers; pixel cap enforced at 89 478 485 pixels | Implemented (R4) |
| Decoder panic on malformed input | `std::panic::catch_unwind` wraps `into_dimensions()`; panics surface as `DecoderPanic` | Implemented (R4) |
| Truncated header passes magic bytes but fails decode | Decoder returns `DecoderError`, not panic | Implemented (R4) |

### Verdict

The file-reading boundary enforces type, size, pixel, decoder-panic, and
TOCTOU controls. The **SEC-001/ADR-047 reuse is not implemented**:
`validate_image_path` canonicalizes paths but does not consult the
`talos-permission` workspace policy. This means a user can `/attach` a
path outside the workspace without an interactive permission prompt.
Closing this gap is the most significant remaining residual and is
tracked separately under R7+ (TUI UX) and the backlog.

## Decoder Panic Containment

| Scenario | Containment |
|---|---|
| Corrupt PNG header | `catch_unwind` catches decoder panic → `DecoderPanic` error return |
| Truncated JPEG | `catch_unwind` catches decoder panic → `DecoderPanic` error return |
| Pixel bomb (small bytes, huge dimensions) | `into_dimensions()` reads headers; pixel cap at 89 Mpx; byte limit primary |
| Malformed metadata | `catch_unwind` catches decoder panic → `DecoderPanic` error return |
| OOM during decode | Byte + pixel limits bound the input size; `catch_unwind` catches OOM panic |
| Header stub (magic bytes only, no IDAT) | Decoder returns `DecoderError` (not panic) → surfaced to user |

### Verdict

Every `image` crate call is wrapped in `catch_unwind`. On any panic, the
error is caught, surfaced as `DecoderPanic` / `DecoderError`, the composer
remains usable, and no process exit occurs. This satisfies AGENTS.md Hard
Constraint #9. Two regression tests in `image_validation.rs` cover the
stub-PNG and truncated-PNG scenarios.

## Persistence / Privacy Boundary

| Surface | Policy |
|---|---|
| TLOG / session log | Stores `Message::Multimodal` with `ContentPart::Image { path, .. }` — path reference only, no image bytes |
| Terminal scrollback (live turn) | Safe summary `[Image: {filename} ({bytes}, {mime})]` via `handle_user_message` — no binary |
| Terminal scrollback (history hydration) | **Residual:** `scrollback.rs::history_message_parts` returns `None` for `Message::Multimodal`, dropping the message from rendered history (R10) |
| Clipboard copy | Safe summary via `engine.handle_user_message` (text content is the summary) |
| Export | Safe summary via the same ChatMessage text content |
| Debug output | No image path or binary in `Debug` impls |
| Logs | Provider adapter logs canonical path on read failure (no binary) |
| Resume after file deletion/move | Provider adapter omits the part via `ImageRead::Omit`; the model sees a `[image omitted: ...]` text marker |
| Resume re-validation of stored path | **Residual:** the session log stores the canonical path from grant time; resume does not re-run `validate_image_path` (R10) |

### Verdict

No image binary data appears in any non-provider-request surface. Path
references are canonicalized and re-validated at provider read time
(R5 TOCTOU guard). Two residuals remain for R10: scrollback drops
multimodal history, and resume does not re-validate paths through the
permission pipeline.

## Residual Risk (Updated)

1. **SEC-001/ADR-047 not reused.** `validate_image_path` canonicalizes
   paths but does not consult the workspace permission policy. A user can
   `/attach` a path outside the workspace without a permission prompt.
   The capability check is the only gate. **Owner-acknowledged residual.**
2. **Filesystem state can change between final validation and the OS
   opening the path.** A future descriptor-relative/capability-handle
   implementation could narrow this race. The current approach is
   fail-closed on any detected change at provider read time.
3. **The `image` crate may have undiscovered vulnerabilities in format
   parsing.** The `catch_unwind` boundary and input size limits mitigate
   this, but a zero-day in the decoder could still cause issues. This is
   a standard residual risk for any image-processing library.
4. **Scrollback and resume gaps (R10).** Multimodal history is currently
   dropped by `scrollback.rs`; resume does not re-run path validation.
   Safe summaries are produced for live turns, but historical context is
   lost on resume.
5. **`/attach` count limit is 4 by default; aggregate byte limit is 50 MiB.**
   These are configurable in source (`MAX_IMAGE_COUNT`, `MAX_TOTAL_IMAGE_BYTES`)
   but not exposed as user-tunable settings.

These residuals do not justify blocking the R3–R5 implementation. The
fail-closed behavior, bounded input, `catch_unwind` boundary, TOCTOU
guard, and capability check provide defense-in-depth sufficient for the
MODEL-009 scope. The SEC-001 reuse gap is the highest-priority residual
and is tracked as a separate backlog item.

## Evidence Cross-Reference

| Control | File | Tests |
|---|---|---|
| Capability check (R3) | `talos-cli/src/tui_bridge.rs`, `talos-conversation/src/engine.rs`, `talos-cli/src/mode_runners.rs` | `bridge_rejects_attach_when_capability_unknown`, `bridge_rejects_attach_when_capability_unsupported`, `bridge_allows_attach_when_capability_supported` |
| Pixel limit + decoder panic (R4) | `talos-cli/src/image_validation.rs` | `validate_png_header_stub_returns_decoder_error_not_panic`, `validate_truncated_png_returns_decoder_error`, `validate_real_png_under_pixel_limit_passes`, `max_pixels_is_89_million` |
| TOCTOU guard (R5) | `talos-provider/src/image_io.rs`, `talos-provider/src/openai_request.rs`, `talos-provider/src/anthropic_request.rs` | `matching_canonical_path_returns_bytes`, `nonexistent_path_is_omitted`, `symlink_swap_is_detected_and_omitted` |
| Residual: scrollback drops multimodal | `talos-tui/src/scrollback.rs:1251` | (none — gap to be closed in R10) |
| Residual: SEC-001 not reused | `talos-cli/src/image_validation.rs::validate_image_path` | (none — gap to be closed) |
