# OBS-001: Observability and Prompt Assets

## Outcome

Talos has bounded local file logs and reviewable built-in prompt assets embedded at compile time.

## Status

Complete (I047, 2026-06-25). Originally selected into I018; completed in I047 as the
prerequisite-closure slice for I019. Log rotation was delivered by I045 (`RotatingWriter` with
size-based rotation + retention, ADR-014). Embedded prompt assets delivered by I047 (moved to
`crates/talos-agent/prompts/` with required-asset tests, ADR-015). I018 is fulfilled/superseded.

## Priority

P1.

## Required Reads

- `docs/iterations/I018-observability-prompt-assets.md`
- `docs/iterations/I047-v012-release-readiness-and-runtime-polish.md`
- `docs/tasks/2026-06-25-i047-i019-memory-release-sequence.md`
- `docs/decisions/014-log-retention-and-rotation.md`
- `docs/decisions/015-embedded-prompt-assets.md`
- `docs/backlog/active/MEM-001-layered-memory-foundation.md`
- `docs/backlog/active/RES-001-exploration-library.md`

## Acceptance Criteria

- [x] File logging has in-process rotation/cleanup and bounded retention. (I045 `RotatingWriter`)
- [x] TUI file logging cannot grow unbounded. (I045: size-based rotation + max_files retention)
- [x] Built-in prompt text is stored as standalone repository assets and embedded at compile time. (I047: `crates/talos-agent/prompts/` + `include_str!`)
- [x] Runtime user-editable prompt packs remain out of scope.
- [x] README/usage docs are updated if config shape changes. (config shape unchanged; prompt asset location is internal)

## Residual Work Destination

Structured JSON logs and shared span contracts stay in #ARCH-S8 R3 or a follow-up ADR.
