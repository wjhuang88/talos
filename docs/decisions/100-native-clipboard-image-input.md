# ADR-100: Explicit native clipboard image input

Status: Proposed; effective only after independent implementation review and merge
Date: 2026-11-09
Owner: [CLIPBOARD-IMAGE-001](../tasks/2026-11-09-clipboard-image-input.md)

Use arboard 3.6.1 with wayland-data-control in the CLI host only, following Codex
commit e54a5713caf331f42a6eef0166186df5f7563c29. Linux uses Rust x11rb and
wl-clipboard-rs; Windows uses system Windows APIs; macOS uses system AppKit and
CoreGraphics through objc2. These are OS interfaces, not arbitrary third-party C
libraries. No Talos-authored unsafe or shared runtime native dependency is added.
This records the OS-ABI boundary under ADR-007; it grants no general FFI exception.

The explicit TUI action invokes the current executable with fixed worker arguments.
Native calls and image encoding run in that isolated child with panic containment.
The parent bounds output and duration, kills and reaps failures; no shell, host
clipboard utility, or persistent clipboard polling is used. At most one acquisition
runs per bridge. Native library allocations occur before returned RGBA validation;
process isolation contains crashes but is not a hard memory quota. A bounded native
read API remains preferable if arboard exposes one; do not claim pre-allocation limits.

Clipboard file lists are metadata only until exact-path permission admission. A
refused/invalid offered file is not silently replaced by another clipboard representation.
Raw pixels become a private PNG in the Talos data directory. Pending files are owned
by the composer and deleted when detached or abandoned; accepted submissions retain
stable files for recovery. Successfully submitted images are retained independently
of session deletion in this initial slice. Document manual retention/cleanup; automatic
reference-aware storage reclamation is a residual, not a delivered feature.

The TUI uses the existing user-command transport for /paste-image, intercepted by
the CLI before model submission. Adding UiOutput::ClipboardPastePending(bool) extends an exhaustive public Rust enum.
This requires the next pre-1.0 minor release, not a 0.10 patch. Downstream exhaustive
matches must handle this presentation event (disable send while true, preserve draft,
re-enable on false); non-TUI hosts may ignore it. No durable wire format changes.
The command is explicit, never inferred from arbitrary pasted text or model output.
Other conversation hosts report unsupported rather than silently submitting it.

Ghostty default Ctrl+Shift+V is terminal text paste. Ctrl+V/Alt+V reach the application;
forwarded Ctrl+Shift+V is accepted too. Do not change terminal config automatically.
