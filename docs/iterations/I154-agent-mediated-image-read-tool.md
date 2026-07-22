# Iteration I154: MODEL-009-E Agent-Mediated Image Read Tool

> Document status: Ready — P2 activation evidence and ADR-051 were accepted on 2026-07-22. Implementation remains unstarted until dispatched as P3.
> Published plan date: 2026-07-21
> Planned objective: allow a Supported model to explicitly invoke a safe `read_image` tool for a local path, then receive the artifact in the following provider request.
> Baseline rule: preserve this target; changed targets use a new iteration ID.
> MVP deliverable: a runnable, permission-gated `read_image` tool with two-protocol mocked proof that binary image data never enters a text tool result.

## Published Baseline

### Selected Story

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| MODEL-009-E | MODEL-009 | Ready | MODEL-009-C/D accepted code remediation, SEC-001, ADR-050, ADR-051 | Agent-mediated image inspection without automatic path reads. |

### Scope

- Add a separate `read_image` tool; preserve text-only `read` behavior.
- Present it only to `ImageInputCapability::Supported` models.
- Use exact-path authorization and MODEL-009-C ingestion/revalidation before every image read.
- Carry a provider-neutral artifact through the agent/session continuation; adapters alone render provider wire content for the next request, using ADR-051's one-shot non-persistent overlay.
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

- Registry/presentation, permission, adversarial-validation, agent/session continuation, OpenAI/Anthropic fixture (including Anthropic user-block coalescing), TUI history/provenance, and copy/export tests.
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
| 2026-07-21 | Planning | Created from maintainer request. No implementation was authorized by this plan alone; I154 remained Planned. |
| 2026-07-22 | P2 activation | Code prerequisites were inventoried against current `main`. Existing tool authorization can bind `read_image` to its exact canonical path; existing provider adapters already perform final canonical-path/digest revalidation. ADR-051 closes the missing continuation contract without changing code. I152/I153 remain Review for the unavailable maintainer-owned live Anthropic check; that external check is not an I154 code prerequisite and cannot be claimed complete. I154 is Ready for a separately dispatched P3 implementation phase. |

## P3 Implementation Contract

The dispatched implementation must be one coherent, reviewable change set. It must not invent a
new permission flow or a process-global artifact cache.

1. Add the additive `ToolExecutionOutput` / `AgentTool::execute_authorized_with_output` API from
   ADR-051. Keep `ToolResult` unchanged. Update both existing permission wrappers so their
   approval/Allow/Ask/Deny behavior and their new forwarding path are identical.
2. Move the reusable image validation/content-digest creation from `talos-cli` to `talos-tools`;
   retain the current CLI attachment behavior by calling the shared implementation. Do not make
   `talos-provider` depend on `talos-tools`; provider-side byte-read revalidation remains there.
3. Implement `ReadImageTool` in `talos-tools`: one `path` input; `ToolNature::Read`; `ToolFamily::File`;
   a path permission facet; exact authorization and runtime capability rechecks before ingestion;
   safe success/error summaries only.
4. Register and present the tool only for `ImageInputCapability::Supported`, including startup and
   model-switch/rebuild paths. Unknown/Unsupported must not expose the schema or read a byte.
5. In `talos-agent`, collect the one successful continuation artifact and build an ephemeral overlay
   solely for the next provider invocation. Do not mutate persisted messages or emit the artifact
   to UI, hooks, session actor, compaction, or exports. Drop it when the provider call begins.
6. Make provider request construction obey ADR-051 ordering: OpenAI tool result then user
   multimodal message; Anthropic coalesces the immediate tool-result/image sequence into one user
   content array. Preserve all normal text-only and composer-attachment request shapes.
7. Add public-item documentation and pre-1.0 migration notes wherever an exported API changes;
   update README EN/zh-CN, site documentation, MODEL-009 state, and this execution record only
   after implementation evidence exists.

### Mandatory P3 Tests

- `read_image` schema visible only for `Supported`; startup/model-switch capability changes are
  reflected; normal text `read` remains byte-for-byte/request-shape unchanged.
- Permission tests cover workspace Allow, external Ask/approve, Deny, headless Ask rejection, an
  `attach_image` authorization mismatch, and a different-path mismatch.
- Shared ingestion retains current invalid magic, directory/FIFO, size/count/pixel, decoder panic,
  symlink retarget, and same-path digest-replacement failures for both `/attach` and `read_image`.
- Agent integration proves the safe textual result is persisted/displayed while the image appears
  exactly once in the immediately following provider request; provider failure, cancellation, and
  a later text-only round do not resend it.
- OpenAI and Anthropic request fixtures prove wire shape and the Anthropic coalescing rule; no
  fixture assertion may inspect or log a full local path/data URL outside the controlled request
  body assertion.
- TUI history, export, copy, resume, and tool provenance assertions show only basename, MIME,
  byte count, result state, and tool identity.

## Variance And Residuals

- I152/I153 retain their independently owned live Anthropic-compatible provider walkthrough gate.
  It is explicitly not substituted by fixture tests and is not closed by this activation record.
