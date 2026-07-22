# Iteration I152: MODEL-009-D Provider Adapter And TUI/CLI Interaction

> Document status: Review
> Published plan date: 2026-07-20
> Activated: 2026-07-20

## Published Baseline

- Selected Ready story: MODEL-009-D, under ADR-050.
- OpenAI-compatible adapter emits protocol-native image request content (data URL).
- Anthropic-compatible adapter emits protocol-native image request content (base64 source).
- Fixture tests prove multi-part text/image ordering and request shape.
- `catch_unwind` at file read boundary in both adapters (AGENTS.md Hard Constraint #9).
- TUI attachment UX: not yet wired (implementation plumbing for future session).
- CLI: safe rejection with documented pointer to TUI path (not yet implemented).

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-20 | Implementation | OpenAI adapter: `OpenAIMessage.content` changed from `Option<String>` to `Option<Value>`; Multimodal arm constructs array of `image_url` parts with data URLs. Anthropic adapter: Multimodal arm constructs `image` content blocks with base64 source. Added `base64` dependency to `talos-provider`. |
| 2026-07-20 | Safety | Added `catch_unwind` wrapping `std::fs::read` in both adapters — handles I/O errors and panics gracefully (empty bytes + tracing::warn). |
| 2026-07-20 | Tests | 3 wire mapping fixture tests: OpenAI image_url data URL shape, OpenAI text-only array, Anthropic image base64 source shape. |
| 2026-07-21 | Security rework | Added capability fail-closed gating, SEC-001/ADR-047 attachment authorization, real decode and pixel limit, content-digest verification at provider read, `/attachments`/`/detach`, print-mode `--attach`, and safe multimodal scrollback summaries. |
| 2026-07-22 | Owner acceptance | P1-A/P1-B code paths accepted after canonical-path authorization and bounded byte-snapshot remediation. Commit `17e3fef` adds regressions for approved-symlink drift and actual-snapshot oversize rejection. Real-terminal evidence remains required before Complete. |
| 2026-07-22 | Terminal-found repair | Maintainer attached an authorized external PNG and verified `/attachments`, but the status bar omitted its pending attachment count. The engine already emitted `StatusSnapshot.attachment_count`; `build_status_text` had not rendered it. The status metrics now show `N image(s)` only for nonzero pending attachments, with wide and narrow rendering regressions. Status remains **Review** pending the rest of the real-terminal packet. |
| 2026-07-22 | Startup capability repair | Maintainer restarted on the same supported `zai-coding-plan/glm-5v-turbo` model and `/attach` failed closed as `Unknown`, while switching away and back succeeded. The bridge only applied `ModelInfo` after a watch change, not its initial value. It now applies the initial snapshot before accepting input; the supported attachment regression deliberately starts the engine at its default `Unknown` and proves the bridge promotes it from the initial watch value. Status remains **Review** pending the rest of the real-terminal packet. |
| 2026-07-22 | Filename rendering repair | Real image submission succeeded, but the Markdown-rendered system attachment notice swallowed underscores in the basename. The safe history summary was already correct. At maintainer direction, the system-only basename display is wrapped in triple backticks, without escapes or zero-width characters; image path, digest, and provider payload behavior are unchanged. |
| 2026-07-22 | Terminal acceptance | Maintainer rebuilt and verified that the fenced attachment filename renders as expected. Implementation and regression evidence: commit `65eb108`. I152 remains **Review** because this accepts only the filename-display repair, not the remaining terminal packet. |
| 2026-07-22 | Terminal acceptance | Maintainer verified the Unsupported/Unknown image-capability gate rejects `/attach` before authorization, filesystem access, or pending-attachment mutation. This accepts the fail-closed gate portion of the terminal packet; I152 remains **Review** pending detach, text-only, and configured-provider checks. |
| 2026-07-22 | Terminal acceptance | Maintainer verified `/detach 1` removes a pending attachment, `/attachments` then reports an empty list, and the status-bar attachment count clears. I152 remains **Review** pending text-only and configured-provider checks. |
| 2026-07-22 | Terminal acceptance | Maintainer verified a text-only turn after attachment operations sends and renders normally, with no image summary and no attachment count left in the status bar. I152 remains **Review** pending the maintainer-owned live Anthropic-compatible provider check. |
| 2026-07-22 | External acceptance gate | Maintainer has no usable Anthropic-compatible Provider or credential in the current environment. The live Anthropic image request is therefore not executed; deterministic Anthropic wire fixtures remain the available automated evidence. I152 remains **Review** until a maintainer can supply that environment. |

## Validation

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | ✅ clean |
| `cargo clippy --workspace --locked -- -D warnings` | ✅ exit 0 |
| `cargo test --workspace --locked` | ✅ all pass |
| `scripts/validate_project_governance.sh .` | ✅ 0 warnings |
| `git diff --check` | ✅ clean |

## Remaining: Real Terminal Acceptance

- Real-terminal walkthrough (requires human verifier): Supported/Unsupported/Unknown attachment gate; external-path approval; attach/list/detach/send; OpenAI-compatible and Anthropic-compatible configured-provider behavior; history safe summary; text-only regression.
- A live credential/provider check is maintainer-owned and remains separate from deterministic mock coverage.
