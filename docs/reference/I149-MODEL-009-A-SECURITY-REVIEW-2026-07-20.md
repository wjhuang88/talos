# I149 / MODEL-009-A Security Review

**Date**: 2026-07-20
**Status**: Accepted
**Decision**: ADR-050

## Boundary Reviewed

The ADR introduces image input capability to Talos. This review covers the new dependency
boundary, file-reading boundary, decoder boundary, and persistence/privacy boundary.

## New Dependency: `image` crate

| Property | Value |
|---|---|
| Crate | `image` (image-rs/image on crates.io) |
| License | MIT OR Apache-2.0 (compatible with Talos Apache-2.0) |
| Native/C bindings | None — pure Rust |
| `unsafe` in public API | No |
| Process spawning | No |
| Security policy | Yes (image-rs/image has a security policy on GitHub) |
| Panic surface | Decoder panics on malformed input (mitigated by `catch_unwind`) |
| Size | Moderate — standard crate, widely used |

### Verdict

The `image` crate is acceptable under AGENTS.md Hard Constraint #1 (no arbitrary C/C++ bindings)
because it is pure Rust with no native dependencies. It is acceptable under Hard Constraint #9
(external dependencies must not crash the process) because every decoder call is wrapped in
`std::panic::catch_unwind` with bounded input.

## File-Reading Boundary

| Threat | Control | Evidence |
|---|---|---|
| Directory traversal | `canonicalize` + regular-file check | Rejects directories |
| FIFO / non-regular file | `std::fs::metadata` file type check | Rejects FIFOs, sockets, devices |
| Symlink retarget / TOCTOU | Canonicalize at grant + re-canonicalize at read; reject if changed | Fail-closed |
| Path traversal (`..`) | `canonicalize` resolves to absolute canonical path | No traversal escape |
| External path without approval | SEC-001/ADR-047 permission pipeline | Deny without explicit approval |
| Headless accidental access | Unresolved Ask fails closed | Structured denial |
| File too large | Byte limit check before full read | 20 MiB single image, 50 MiB total |
| Pixel bomb | Dimension check before full decode | 89M pixel limit |

### Verdict

The file-reading boundary reuses the existing SEC-001/ADR-047 permission pipeline with
additional size and pixel limits. No bypass is introduced. Fail-closed behavior is preserved.

## Decoder Panic Containment

| Scenario | Containment |
|---|---|
| Corrupt PNG header | `catch_unwind` catches decoder panic → error return |
| Truncated JPEG | `catch_unwind` catches decoder panic → error return |
| Pixel bomb (small bytes, huge dimensions) | Dimension check before full decode; if decode needed, byte limit is primary guard |
| Malformed metadata | `catch_unwind` catches decoder panic → error return |
| OOM during decode | Byte + pixel limits bound the input size; `catch_unwind` catches OOM panic |

### Verdict

Every `image` crate call is wrapped in `catch_unwind`. On any panic, the error is caught, an
error message is returned, the composer remains usable, and no process exit occurs. This
satisfies AGENTS.md Hard Constraint #9.

## Persistence / Privacy Boundary

| Surface | Policy |
|---|---|
| TLOG / session log | Path reference only — no image bytes persisted |
| Terminal scrollback | Safe summary `[Image: {filename} ({bytes}, {mime})]` — no binary |
| Clipboard copy | Same safe summary — no binary |
| Export | Same safe summary — no binary |
| Debug output | No image path or binary in Debug impls |
| Logs | No image path or binary in log output |
| Resume after file deletion/move | Safe summary with diagnostic — no panic |

### Verdict

No image binary data appears in any non-provider-request surface. Path references are
canonicalized and re-validated on resume. Privacy is preserved across all display surfaces.

## Residual Risk

- Filesystem state can change between final validation and the OS opening the path. A future
  descriptor-relative/capability-handle implementation could narrow this race. The current
  approach is fail-closed on any detected change.
- The `image` crate may have undiscovered vulnerabilities in format parsing. The `catch_unwind`
  boundary and input size limits mitigate this, but a zero-day in the decoder could still cause
  issues. This is a standard residual risk for any image-processing library.
- The `image` crate's `into_dimensions()` may require a partial or full decode for some formats.
  The byte limit is the primary guard in these cases.

These residuals do not justify blocking the implementation. The fail-closed behavior, bounded
input, and `catch_unwind` boundary provide defense-in-depth sufficient for the MODEL-009 scope.
