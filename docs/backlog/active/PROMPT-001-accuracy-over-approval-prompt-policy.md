# PROMPT-001: Accuracy Over Approval Prompt Policy

**Status**: Complete (2026-06-28)
**Priority**: P2
**Created**: 2026-06-28
**Related**: ADR-015, ARCH-020

## Problem

Talos's default identity prompt already emphasized tool selection and coding-agent behavior, but
did not explicitly guard against model sycophancy, fabricated certainty, unsupported citations, or
agreement after user pushback without new evidence.

The user supplied a prompt pattern that prioritizes accuracy over approval. Talos should absorb the
behavioral principle without forcing high-noise claim tags into every ordinary coding response.

## Scope

- Add default identity-prompt guidance for:
  - accuracy over approval;
  - direct counterarguments when evidence shows risk;
  - explicit "I don't know" behavior;
  - no fabricated facts, citations, APIs, release status, or named-entity claims;
  - visible uncertainty for guesses, inferences, symbolic frames, and post-hoc reasoning;
  - open revision when a position was held for consistency instead of evidence.
- Keep the prompt asset embedded at compile time per ADR-015.
- Preserve prompt section order and cache marker behavior.
- Update README user-facing default-prompt description.
- Update `AGENTS.md` so the same behavior applies to Agents working in this repository.

## Non-Goals

- Do not require every ordinary coding response to tag every sentence.
- Do not add a new runtime prompt-pack system.
- Do not change provider request schemas, tool schemas, permissions, or context injection order.

## Acceptance Criteria

- [x] Default identity prompt contains the accuracy-over-approval behavior.
- [x] Prompt asset tests prove the guidance remains embedded.
- [x] README and README.zh-CN mention the default prompt behavior.
- [x] AGENTS.md includes the same accuracy-over-approval governance for repository Agents.
- [x] Prompt cache section ordering remains unchanged.

## Validation

- `cargo fmt --all -- --check`
- `cargo test -p talos-agent prompt::tests::test_identity_prompt_contains_accuracy_discipline`
- `cargo test -p talos-agent prompt::tests`
- `cargo check --workspace`
- `sh scripts/validate_project_governance.sh .`
- `git diff --check`
