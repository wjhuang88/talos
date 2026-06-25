# OBS-001: Observability and Prompt Assets

## Outcome

Talos has bounded local file logs and reviewable built-in prompt assets embedded at compile time.

## Status

Planned. Originally selected into I018; selected into I047 as the prerequisite-closure slice for
I019. If I047 delivers this acceptance, I018 should be recorded as fulfilled/superseded by I047
during closeout without rewriting the I018 baseline.

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

- [ ] File logging has in-process rotation/cleanup and bounded retention.
- [ ] TUI file logging cannot grow unbounded.
- [ ] Built-in prompt text is stored as standalone repository assets and embedded at compile time.
- [ ] Runtime user-editable prompt packs remain out of scope.
- [ ] README/usage docs are updated if config shape changes.

## Residual Work Destination

Structured JSON logs and shared span contracts stay in #ARCH-S8 R3 or a follow-up ADR.
