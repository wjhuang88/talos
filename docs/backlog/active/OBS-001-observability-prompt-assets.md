# OBS-001: Observability and Prompt Assets

## Outcome

Talos has bounded local file logs and reviewable built-in prompt assets embedded at compile time.

## Status

Planned. Selected into I018.

## Priority

P1.

## Required Reads

- `docs/iterations/I018-observability-prompt-assets.md`
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

