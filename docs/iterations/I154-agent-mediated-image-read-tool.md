# Iteration I154: MODEL-009-E Agent-Mediated Image Read Tool

> Document status: Planned — do not activate before MODEL-009-C/D remediation is accepted.
> Published plan date: 2026-07-21
> Planned objective: allow a Supported model to explicitly invoke a safe `read_image` tool for a local path, then receive the artifact in the following provider request.
> Baseline rule: preserve this target; changed targets use a new iteration ID.
> MVP deliverable: a runnable, permission-gated `read_image` tool with two-protocol mocked proof that binary image data never enters a text tool result.

## Published Baseline

### Selected Story

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| MODEL-009-E | MODEL-009 | Planned | MODEL-009-C/D accepted remediation, SEC-001, ADR-050 | Agent-mediated image inspection without automatic path reads. |

### Scope

- Add a separate `read_image` tool; preserve text-only `read` behavior.
- Present it only to `ImageInputCapability::Supported` models.
- Use exact-path authorization and MODEL-009-C ingestion/revalidation before every image read.
- Carry a provider-neutral artifact through the agent/session continuation; adapters alone render provider wire content for the next request.
- Render safe metadata and tool provenance only.

### Non-Goals

- Automatic reads triggered by a path in a user message.
- Binary/base64 text tool results, remote URLs, OCR/media expansion, protocol expansion, or changes to generic `read` semantics.

### Acceptance

- Given a Supported model, when it calls `read_image` for an approved valid image, then the next provider request contains the corresponding protocol-native image part exactly once.
- Given Unknown or Unsupported, when tools are presented, then `read_image` is absent and no file bytes are read.
- Given any permission, validation, revalidation, decoding, or provider failure, when invoked, then no binary/path disclosure or partial artifact is persisted or sent.
- Given a normal text `read`, when I154 is enabled, then its output and provider behavior are unchanged.

### Planned Validation

- Registry/presentation, permission, adversarial-validation, agent/session continuation, OpenAI/Anthropic fixture, TUI history/provenance, and copy/export tests.
- Locked fmt/check/clippy/test, governance validation, and `git diff --check`.

### Documentation To Update

- README EN/zh-CN, site capabilities/command documentation, MODEL-009 parent/child state, Board, and ADR-050 implementation facts if continuation details need clarification.

### Risks And Rollback

- Risk: provider tool-result semantics cannot safely transport an image artifact across both protocols.
- Rollback: do not expose `read_image`; retain explicit composer attachment and record the protocol gap in ADR-050.

## Change-Control Decision

| Date | Classification | Decision | Impact |
|---|---|---|---|
| 2026-07-21 | Scope addition | Accepted into the program as a new iteration after I153; I152's published attachment UX baseline is unchanged. | Adds an estimated two-week iteration. Activation is blocked until I151/I152 security and end-to-end blockers close. |

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-21 | Planning | Created from maintainer request. No implementation is authorized by this plan alone; I154 remains Planned. |

## Variance And Residuals

- I151/I152 own prerequisite remediation: pixel/decoder enforcement, SEC-001 authorization, capability gating, TOCTOU revalidation, attachment UX, and true end-to-end provider proof.
