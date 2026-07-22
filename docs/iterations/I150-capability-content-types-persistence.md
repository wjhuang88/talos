# Iteration I150: MODEL-009-B Capability Model, Content Types, And Persistence Foundation

> Document status: Review
> Published plan date: 2026-07-20
> Activated: 2026-07-20 (after I149 ADR-050 Accepted)
> Prerequisite: I149 ADR Accepted ✅
> Status changed to Review: 2026-07-22 (implementation commit `b3cc943` and its
> downstream provider/session integration have passed locked validation; maintainer
> terminal evidence remains bundled with I151/I152).

## Published Baseline

- Selected Ready story: MODEL-009-B, under ADR-050.
- Dependencies satisfied: I149 ADR-050 Accepted.
- Implement Talos-owned typed ordered content parts (`ContentPart` enum in `talos-core/src/message.rs`).
- Add `Message::Multimodal { parts: Vec<ContentPart> }` variant (additive, pre-1.0 semver break).
- Add `ImageInputCapability` enum in `talos-core/src/model.rs` with `from_metadata` and `allows_attachment`.
- All existing text-only requests, history, resume, export, and copy maintain their current wire shape and behavior.
- Provider adapters extract text from Multimodal messages (image parts ignored until I152).
- Session serialization handles Multimodal (text extraction for preview, full serde for persistence).
- TUI scrollback returns None for Multimodal (no rendering until I151/I152).
- No image binary in terminal/history/copy/export by default.
- All new public API items have `///` doc comments.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-20 | Planning | Baseline published. ADR-050 Accepted, hard gate cleared. |
| 2026-07-20 | Implementation | 1. `talos-core/src/message.rs`: `ContentPart` enum (Text + Image with path/mime/byte_count), `Message::Multimodal` variant. 2. `talos-core/src/model.rs`: `ImageInputCapability` enum (Supported/Unsupported/Unknown) with `from_metadata` and `allows_attachment`. 3. `talos-session/src/jsonl.rs`: `message_parts` handles Multimodal (text + image summary). 4. `talos-session/src/durable.rs`: `filtered_message` handles Multimodal (redact text, preserve image path ref). 5. `talos-provider/src/openai_request.rs`: Multimodal → extract text, ignore images. 6. `talos-provider/src/anthropic_request.rs`: Multimodal → extract text, ignore images. 7. `talos-agent/src/compaction/engine.rs`: Multimodal → extract text for compaction. 8. `talos-agent/src/token.rs`: Multimodal → token estimate for text + image metadata. 9. `talos-tui/src/scrollback.rs`: Multimodal → None (no rendering until I151). |
| 2026-07-22 | Review evidence | Implementation landed in `b3cc943`. I151/I152 subsequently exercised the type through validated ingestion, session forwarding, provider mapping, safe scrollback summary, capability gating, authorization, and digest checks. I150 is Review, not Complete: its user-visible proof is part of the combined I151/I152 terminal walkthrough. |

## Validation

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | ✅ clean |
| `cargo check --workspace --locked` | ✅ exit 0 |
| `cargo clippy --workspace --locked -- -D warnings` | ✅ exit 0 |
| `cargo test --workspace --locked` | ✅ all tests pass |
| `scripts/validate_project_governance.sh .` | ✅ 0 warnings |
| `git diff --check` | ✅ clean |

## Remaining: Combined Terminal Acceptance

I150 has no standalone interactive surface. Its `ContentPart` and capability semantics are
verified through the I151/I152 combined terminal packet: Supported/Unknown/Unsupported gating,
an accepted attachment, safe history summary, and unchanged text-only submission. Do not mark
I150 Complete until that packet is recorded with its evidence commit.
