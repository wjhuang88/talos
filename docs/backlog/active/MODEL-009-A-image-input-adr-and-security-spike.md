# MODEL-009-A: Image Input ADR And Security Spike

| Field | Value |
| --- | --- |
| Story ID | MODEL-009-A |
| Type | Spike / Decision Story |
| Priority | P2 |
| Status | Refinement — selected into I149 (2026-07-20) |
| Source | Maintainer requirement recorded 2026-07-20; child of MODEL-009 |
| Parent Epic | MODEL-009 |
| Depends on | MODEL-008-B (I148) custom provider + model discovery; ADR-013 provider config; ADR-023 credential boundary; SEC-001/ADR-047 external-path authorization |
| Blocks | MODEL-009-B (I150), MODEL-009-C (I151), MODEL-009-D (I152) — all Blocked until this ADR is Accepted |

## Problem

MODEL-009 needs a safe image input capability, but the architecture/security decisions are not yet
made. Implementing image sending before deciding content-part schema, persistence, path
authorization, format limits, decoder panic containment, wire mapping, and capability provenance
would lock in unsafe defaults.

## Goal / Value

Produce an Accepted ADR (reserved ADR-050) and a testable prototype that decide every
safety-critical question for image input before any production image sending lands. I149 is
research, ADR, prototype, and documentation only — **no production image sending** is allowed in
this iteration.

## Scope

The ADR must explicitly decide all ten points:

1. **Ordered text/image content-part schema** — how Talos represents ordered text and image parts
   without encoding provider JSON in core/session types.
2. **Pre-1.0 semver migration strategy** — public API impact, additive vs breaking changes,
   migration notes, and the future minor release.
3. **Attachment storage / session resume / export / copy / deletion / move behavior** — whether
   image bytes are retained, copied into Talos storage, or referenced by path; privacy and
   portability on deletion/relocation/export.
4. **Local path authorization, external path, and symlink policy** — reuse SEC-001/ADR-047 or
   define a stricter boundary; symlink revalidation; no bypass because the model is
   vision-capable.
5. **Supported formats, MIME/magic-byte verification, single-image byte limit, total byte limit,
   pixel limit, count limit** — explicit numbers and verification strategy.
6. **Decoder dependency, license, security review, and panic containment** — which decoder,
   `catch_unwind` boundaries, size limits, error propagation; per AGENTS.md Hard Constraint #9.
7. **OpenAI-compatible and Anthropic-compatible wire mapping** — data URL vs uploaded media,
   provider-specific limits, and how the adapter emits protocol-native image content.
8. **Capability provenance for built-in, imported, and custom-provider models** — where
   `image_input` comes from for each source.
9. **`Supported`, `Unsupported`, and `Unknown` distinction** — exact semantics; both
   `Unknown` and `Unsupported` must fail-closed for the user, but the distinction must be usable
   for diagnostics.
10. **Custom/discovered models default `Unknown`** — no default image support for models without
    confirmed capability.

The ADR must also record:

- The threat model for new decoder dependencies (panic, OOM, pixel bomb, adversarial fixtures).
- The threat model for file reading (symlink, TOCTOU, non-regular files, path traversal).
- The privacy threat model for attachment persistence (binary content in history, export, copy,
  debug, logs).
- The exact pre-1.0 semver break analysis for every public type whose shape changes.

## Explicit Exclusions

- Inferring image capability from a model name.
- Sending arbitrary image requests to probe provider capability.
- Introducing remote URL image fetching.
- Introducing audio, video, PDF, screenshot, clipboard image extraction, or image generation.
- New `unsafe` blocks or native dependencies not covered by the ADR + security review.
- Any production image sending code path in this iteration.

## Design / Security Constraints

- ADR-013 keeps provider wire format inside `talos-provider` adapters; the ADR must not leak
  provider JSON into `talos-core` or `talos-session`.
- ADR-023 governs credential display; attachment diagnostics, persistence, export, and debug
  must not expose credentials; file paths and image-derived metadata need an explicit privacy
  policy.
- SEC-001/ADR-047 governs external-path authorization; image path authorization must reuse or
  extend that boundary.
- AGENTS.md Hard Constraint #9: every native/panic boundary must be wrapped with bounded input,
  `catch_unwind` where applicable, and a safe error path.

## Acceptance

- Given the ADR, when a reader asks any of the ten decision points, then the ADR provides an
  executable, testable answer.
- Given a new decoder dependency, when the security review is consulted, then it records the
  license, panic surface, size limits, and error propagation boundary.
- Given a file-read boundary, when the security review is consulted, then it records the symlink,
  TOCTOU, traversal, and non-regular-file threat model.
- Given the ADR, when a downstream iteration (I150) starts, then it can implement the chosen
  schema/policy without making additional architecture decisions.

## Hard Gate

If the ADR cannot be Accepted on all ten decision points:

- Mark I149 **Blocked**.
- Record evidence, alternatives, and the recovery condition in the ADR and in this Story.
- Do not enter I150. I150-I152 remain Blocked.
- Append a hard-stop checkpoint to the long-task record
  (`docs/tasks/2026-07-20-provider-multimodal-four-month-program.md`).

## Required Reads

- `docs/backlog/active/MODEL-009-multimodal-image-input.md`
- `docs/decisions/013-provider-config-schema-boundary.md`
- `docs/decisions/023-inline-api-key-boundary.md`
- `docs/decisions/047-external-path-tool-authorization.md`
- `docs/backlog/active/SEC-001-external-path-authorization.md`
- `docs/reference/I140-SEC001-SECURITY-REVIEW-2026-07-17.md`
- `crates/talos-core/src/model.rs`
- `crates/talos-core/src/message.rs`
- `crates/talos-conversation/src/types.rs`
- `crates/talos-provider/src/openai.rs`
- `crates/talos-provider/src/anthropic_request.rs`
- `crates/talos-tui/src/state.rs`

## Minimum Validation

- ADR document at `docs/decisions/050-<slug>.md` with all ten decision points resolved.
- Security review document at `docs/reference/I149-MODEL-009-A-SECURITY-REVIEW-<date>.md`.
- A testable prototype (in a `tests/` or `examples/` directory) that demonstrates the chosen
  content-part schema and decoder boundary without exposing production image sending.
- Locked fmt/check/clippy/test (prototype code must compile and pass); governance; `git diff --check`.
- If the ADR is not Acceptable, the hard-gate checkpoint is the only required output.
