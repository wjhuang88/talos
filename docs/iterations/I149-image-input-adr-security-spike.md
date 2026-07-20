# Iteration I149: MODEL-009-A Image Input ADR And Security Spike

> Document status: Complete (ADR Accepted)
> Published plan date: 2026-07-20
> Planned objective: produce an Accepted ADR (ADR-050) deciding all 10 safety-critical points
> for MODEL-009 before any production image sending lands.
> Baseline rule: this iteration is research, ADR, and documentation only — no production image sending.
> MVP deliverable: an Accepted ADR that gates I150-I152.

## Published Baseline

- Selected Ready story: MODEL-009-A.
- ADR-050 must decide all 10 points:
  1. Ordered text/image content-part schema
  2. Pre-1.0 semver migration strategy
  3. Image storage, session resume, export, copy, deletion/move behavior
  4. Local path authorization, external path, symlink strategy
  5. Supported formats, MIME/magic-byte verification, byte/pixel/count limits
  6. Decoder dependency, license, security review, panic containment
  7. OpenAI-compatible and Anthropic-compatible wire mapping
  8. Capability provenance for built-in, imported, custom-provider models
  9. `Supported`/`Unsupported`/`Unknown` distinction
  10. Custom/discovered models default `Unknown`
- No production image sending code in this iteration.

## Exit Gate

- ADR Accepted on all 10 points ✅
- Security review completed ✅
- If ADR was not Acceptable: mark I149 Blocked, stop, do not enter I150.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-20 | Planning | Baseline published. |
| 2026-07-20 | Research | Read existing provider adapters, config types, SEC-001/ADR-047, message types. |
| 2026-07-20 | ADR | Created `docs/decisions/050-multimodal-image-input-architecture.md` with all 10 decision points resolved. Status: Accepted. |
| 2026-07-20 | Security review | Created `docs/reference/I149-MODEL-009-A-SECURITY-REVIEW-2026-07-20.md` covering new dependency, file-reading, decoder panic containment, and persistence/privacy boundaries. |

## Validation

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | ✅ clean (no code changes in this iteration) |
| `cargo check --workspace --locked` | ✅ exit 0 |
| `cargo clippy --workspace --locked -- -D warnings` | ✅ exit 0 |
| `cargo test --workspace --locked` | ✅ exit 0 |
| `scripts/validate_project_governance.sh .` | ✅ 0 warnings |
| `git diff --check` | ✅ clean |

## Decision

I149 is **Complete**. ADR-050 is Accepted on all 10 points. Security review is complete. No
production image sending code was implemented. I150 may proceed.
