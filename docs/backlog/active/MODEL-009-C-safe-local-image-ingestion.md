# MODEL-009-C: Safe Local Image Ingestion

| Field | Value |
| --- | --- |
| Story ID | MODEL-009-C |
| Type | Product / Security Story |
| Priority | P2 |
| Status | Refinement — selected into I151 (2026-07-20) |
| Source | Maintainer requirement recorded 2026-07-20; child of MODEL-009 |
| Parent Epic | MODEL-009 |
| Depends on | MODEL-009-B (I150) typed content + capability semantics + persistence boundary |
| Blocks | MODEL-009-D (I152) |

## Problem

MODEL-009-B provides types and persistence, but no code path yet reads a local image file safely.
A user who attaches an image must be protected against: directory paths, FIFOs, corrupt images,
fake MIME types, oversized files, pixel bombs, aggregate-limit breaches, authorization denials,
external paths, symlinks, and decoder panics — all before any byte reaches a provider.

## Goal / Value

Provide an explicit local image path input that, before reading any file bytes, completes every
rejection that can be completed early, and on any failure leaves the composer usable, leaves the
session unchanged, leaves no partial attachment, and leaks no sensitive path or binary content.

## Scope

- Explicit local image path input only. No auto-scan of workspace files; no automatic attachment.
- Reuse the existing path authorization and SEC-001/ADR-047 external-path security boundary. No
  bypass because the model is vision-capable.
- Per the I149 ADR, implement:
  - Path canonicalization.
  - Regular-file validation (reject directories, FIFOs, character/block devices, sockets).
  - Symlink policy (revalidate on execution; reject if target changed between grant and read).
  - MIME and magic-byte verification (extension alone is not sufficient).
  - Format restriction (only ADR-approved formats).
  - Single-image byte limit.
  - Total byte limit (across all attachments in the current message).
  - Pixel limit (decoded dimensions).
  - Attachment count limit.
- Complete every rejection that can be completed before the full file is read.
- At every native or potentially-panicking dependency boundary: bounded input, error propagation,
  and `catch_unwind` where applicable.
- On failure:
  - The composer remains usable.
  - No partial session is written.
  - No partial attachment is retained.
  - No sensitive path or binary content is leaked (per ADR-023 privacy extension).

## Explicit Exclusions

- Remote URL image fetching.
- Auto-scan or auto-attach of workspace files.
- Audio, video, PDF, screenshot, clipboard, or image generation.
- New `unsafe` blocks or native dependencies beyond what the I149 ADR approved.
- Bypassing SEC-001/ADR-047 path authorization.
- Trusting remote capability metadata.

## Design / Security Constraints

- Per AGENTS.md Hard Constraint #9: every native/panic boundary must have bounded input, error
  propagation, and applicable `catch_unwind`.
- Per AGENTS.md Hard Constraint #4: every file read goes through the permission pipeline.
- Per ADR-023: credentials, full local paths, and binary content must not leak into logs, Debug
  output, panel labels, or UI surfaces.
- Per ADR-013: provider wire format stays inside `talos-provider` adapters.

## Acceptance

- Given a regular-file image path within the authorized boundary, when the user attaches it, then
  the file is read, validated, and the attachment is staged for sending.
- Given a directory path, when the user attempts to attach it, then the attachment is rejected
  before any byte is read and the composer remains usable.
- Given a FIFO or non-regular file, when the user attempts to attach it, then the attachment is
  rejected before any byte is read.
- Given a corrupt or invalid image, when decoding is attempted, then the failure is contained, no
  panic occurs, and the composer remains usable.
- Given a file with a fake MIME type (extension does not match magic bytes), when validation
  runs, then the attachment is rejected.
- Given a file exceeding the single-image byte limit, when validation runs, then the attachment is
  rejected before the full file is read.
- Given a pixel-bomb image (small bytes, huge dimensions), when decoding is attempted, then the
  pixel limit catches it before OOM and no panic occurs.
- Given multiple attachments whose total exceeds the aggregate byte limit, when validation runs,
  then the new attachment is rejected.
- Given an attachment count that exceeds the count limit, when validation runs, then the new
  attachment is rejected.
- Given an external path without explicit authorization, when the user attempts to attach it,
  then the permission pipeline denies it before any byte is read.
- Given a symlink whose target changed between authorization and execution, when validation runs,
  then the attachment is rejected (fail-closed).
- Given a decoder panic, when `catch_unwind` catches it, then the failure is contained, no
  process exit occurs, and the composer remains usable.

## Required Reads

- `docs/backlog/active/MODEL-009-multimodal-image-input.md`
- `docs/backlog/active/MODEL-009-A-image-input-adr-and-security-spike.md`
- `docs/backlog/active/MODEL-009-B-capability-content-types-persistence.md`
- `docs/backlog/active/SEC-001-external-path-authorization.md`
- `docs/decisions/047-external-path-tool-authorization.md`
- `docs/decisions/023-inline-api-key-boundary.md`
- `docs/decisions/050-<slug>.md` (the I149 ADR)
- `crates/talos-tools/src/file_tools/`
- `crates/talos-permission/src/lib.rs`

## Minimum Validation

- Adversarial fixtures for every case listed in Acceptance: directory, FIFO/non-regular, corrupt
  image, fake MIME, oversize, pixel bomb, aggregate-limit breach, auth denial, external path,
  symlink retarget, decoder panic, decoder error.
- Panic-containment tests (where applicable, force the decoder into a panic path and assert
  `catch_unwind` catches it).
- Permission-pipeline tests proving no bypass.
- No-leak tests asserting no path or binary content appears in logs, Debug output, panel labels,
  or UI surfaces.
- Locked fmt/check/clippy/test and `scripts/validate_project_governance.sh .`; `git diff --check`.
