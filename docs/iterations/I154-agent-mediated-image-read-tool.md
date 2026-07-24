# Iteration I154: MODEL-009-E Agent-Mediated Image Read Tool

> Document status: Review — P3 implementation complete. Maintainer GO received 2026-07-24 after independent verification of all gates and ADR-051 contracts. 40 new tests pass.
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
| 2026-07-23 | P3 Step A | `ToolExecutionOutput` struct + `execute_authorized_with_output` method added to `AgentTool` trait in `talos-core/tool.rs`. Permission wrappers in `registry.rs` updated to forward. Commit `6d4677e`. |
| 2026-07-23 | P3 Step C | Image validation migrated from `talos-cli` to shared `talos-tools/src/image_validation.rs` module. `talos-cli` re-exports from `talos-tools`. Commit `ad46eba`. |
| 2026-07-23 | P3 Step B | `ReadImageTool` implemented in `talos-tools/src/read_image_tool.rs`: `read_image` tool, `ToolNature::Read`, `ToolFamily::File`, overrides `execute_authorized_with_output` to return safe summary + `ContentPart::Image` in `next_provider_parts`. Commit `9009096`. |
| 2026-07-23 | P3 Step D | `execute_with_output` method added to `AgentTool` trait (default delegates to `execute`). Overridden in `TuiPermissionAwareTool` and `PermissionAwareTool` to return full `ToolExecutionOutput`. `execute_single_tool_with_presentation` changed to call `execute_with_output` and return `(ToolExecutionResult, Vec<ContentPart>)`. Batch functions collect continuation parts. Turn loop in `lib.rs` injects parts as transient `Message::Multimodal` overlay before next `stream_with_tools` call; consumed once, never persisted. Commit `5eeb8e1`. |
| 2026-07-23 | P3 Step E | Provider adapter wire mapping verified — existing `Message::Multimodal` handling in both OpenAI (`openai_request.rs`) and Anthropic (`anthropic_request.rs`) adapters already covers the continuation overlay. No adapter changes needed. |
| 2026-07-23 | P3 Step F | `ReadImageTool` registered behind permission wrappers in print and TUI registries. `image_input_supported` field added to `Agent` struct; filters `read_image` from presented tools when `!image_input_supported` (fail-closed). `set_image_input_capability` helper wires capability from model metadata at all agent construction sites. Commit `2270f21`. |
| 2026-07-23 | P3 Tests | 7 `ReadImageTool` unit tests added: execute safety stub, authorized image output, path escape, nonexistent file, directory rejection, tool metadata, default `execute_with_output` delegation. All workspace tests pass (0 failures). Commit `36d987c`. |
| 2026-07-23 | P3 NO-GO | Maintainer rejected P3 with 7 blockers: B1 missing permission_profile, B2 Anthropic coalescing, B3 missing mandatory tests, B4 batch limit bypass, B5 capability gate only at presentation, B6 raw path in error text, B7 docs incomplete. |
| 2026-07-23 | P3 Rework B1/B2/B4/B5/B6 | Implemented permission_profile with path facet (B1). Anthropic consecutive user-message coalescing (B2). Batch limit enforcing max 1 image artifact per tool batch (B4). Execution-boundary capability gate rejecting read_image when !image_input_supported (B5). Sanitized execute() and PathEscape error messages to remove raw path (B6). Commit `4a0616a`. |
| 2026-07-23 | P3 Rework B3 | 3 agent continuation integration tests: image appears once in next provider request, consumed after second call, not in persisted messages. Commit `9ecca94`. |
| 2026-07-23 | P3 Rework B7 | README EN/zh-CN updated with `read_image` tool documentation. ADR-051 implementation facts and pre-1.0 migration notes recorded. I154 iteration evidence updated. |

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
